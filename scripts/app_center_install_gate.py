"""Opt-in local installation approval gate for Trellis App Center (Rung 6).

No arbitrary commands, paths, package sources or approval hashes are trusted from
the browser. The CLI performs its own independent preflight again on apply.
"""
from __future__ import annotations

import json
import re
import secrets
import subprocess
import tempfile
import threading
import time
from pathlib import Path

HASH = re.compile(r"^[a-f0-9]{64}$")
PACKAGE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{2,99}$")
MAX_IDS = 20
MAX_CATALOG = 2 * 1024 * 1024
MAX_COMMAND_OUTPUT = 128 * 1024
PREVIEW_TTL_SECONDS = 120
PREVIEW_TIMEOUT_SECONDS = 120
INSTALL_TIMEOUT_SECONDS = 3600


class GateError(Exception):
    def __init__(self, status: int, message: str):
        super().__init__(message)
        self.status = status


def run_cli(argv: list[str], timeout: int) -> tuple[int, str]:
    """Run a fixed local Trellis executable without a shell or inherited stdin.

    Use a bounded tempfile for command output to avoid unbounded memory growth.
    WinGet may itself launch an installer/UAC prompt after explicit consent.
    """
    try:
        with tempfile.TemporaryFile(mode="w+b") as stream:
            result = subprocess.run(
                argv, stdin=subprocess.DEVNULL, stdout=stream,
                stderr=subprocess.STDOUT, timeout=timeout, check=False,
            )
            size = stream.tell()
            stream.seek(max(0, size - MAX_COMMAND_OUTPUT))
            return result.returncode, stream.read(MAX_COMMAND_OUTPUT).decode(
                "utf-8", errors="replace"
            )
    except subprocess.TimeoutExpired as error:
        raise GateError(504, "Local Trellis command exceeded its timeout") from error
    except OSError as error:
        raise GateError(503, "Unable to start local Trellis CLI") from error


class ApprovalGate:
    """Single-use preview and at most one active install at a time.

    The only executable path is the operator-selected Trellis CLI binary, with
    fixed arguments generated from allowlisted, canonical package identifiers.
    """

    def __init__(self, catalog: Path, trellis_cli: Path, runner=run_cli):
        self.catalog = Path(catalog).resolve()
        self.trellis_cli = Path(trellis_cli)
        self.runner = runner
        self._lock = threading.Lock()
        self._pending: dict | None = None
        self._job: dict | None = None

    def _catalog_ids(self) -> set[str]:
        try:
            if self.catalog.is_symlink() or self.catalog.stat().st_size > MAX_CATALOG:
                raise GateError(503, "Local catalog is unavailable or too large")
            catalog = json.loads(self.catalog.read_bytes())
        except (OSError, ValueError) as error:
            raise GateError(503, "Unable to read reviewed App Center catalog") from error
        if catalog.get("schema_version") != "trellis.app-center.v0.1":
            raise GateError(503, "Unsupported local catalog format")
        apps = catalog.get("apps")
        if not isinstance(apps, list):
            raise GateError(503, "Invalid local catalog")
        ids: set[str] = set()
        for app in apps:
            if not isinstance(app, dict) or app.get("kind") != "winget":
                continue
            package_id = app.get("package_id")
            item_id = app.get("id")
            if (not isinstance(package_id, str)
                    or not PACKAGE.fullmatch(package_id)
                    or item_id != "winget:" + package_id
                    or app.get("status") != "external_preflight"
                    or "windows" not in app.get("platforms", [])):
                raise GateError(503, "Invalid WinGet entry in local catalog")
            if item_id.casefold() in {value.casefold() for value in ids}:
                raise GateError(503, "Duplicate WinGet entry in local catalog")
            ids.add(item_id)
        return ids

    def _validate_ids(self, supplied: object) -> list[str]:
        if (not isinstance(supplied, list) or not 1 <= len(supplied) <= MAX_IDS
                or any(not isinstance(value, str) for value in supplied)):
            raise GateError(400, "Select between 1 and 20 Windows WinGet apps")
        available = self._catalog_ids()
        if len(set(supplied)) != len(supplied):
            raise GateError(400, "Duplicate package selections are not allowed")
        if any(item not in available for item in supplied):
            raise GateError(400, "One or more items are not in the local WinGet allowlist")
        return sorted(supplied)

    def _argv(self, path: Path, apply_hash: str | None = None) -> list[str]:
        args = [
            str(self.trellis_cli), "winget-batch",
            "--plan", str(path),
            "--catalog", str(self.catalog),
        ]
        if apply_hash is not None:
            args += ["--apply", "--confirm", apply_hash]
        return args

    def _cleanup_pending(self) -> None:
        if self._pending is not None:
            self._pending["temporary"].cleanup()
            self._pending = None

    def _state(self) -> dict:
        current = self._job
        if current is None:
            return {"state": "idle"}
        return {key: value for key, value in current.items()
                if key in {"state", "ids", "started_at", "finished_at",
                           "exit_code", "message"}}

    def status(self) -> dict:
        with self._lock:
            return {"schema_version": "trellis.approval-status.v0.1",
                    "enabled": True, "job": self._state()}

    def preview(self, ids: object) -> dict:
        # Lock held during preflight to prevent races with another approval.
        with self._lock:
            if self._job is not None and self._job["state"] == "running":
                raise GateError(409, "An installation batch is already active")
            self._cleanup_pending()
            selected = self._validate_ids(ids)
            directory = tempfile.TemporaryDirectory(prefix="trellis-preview-")
            plan_path = Path(directory.name) / "selection.json"
            plan = {
                "schema_version": "trellis.selection-plan.v0.1",
                "approval": "not_granted", "execution": "none",
                "items": [{"id": item, "kind": "winget",
                           "status": "external_preflight"} for item in selected],
            }
            try:
                plan_path.write_text(json.dumps(plan, separators=(",", ":")), encoding="utf-8")
                code, output = self.runner(
                    self._argv(plan_path), PREVIEW_TIMEOUT_SECONDS
                )
                if code != 0:
                    raise GateError(422, "WinGet preflight did not pass. " +
                                    output[-3000:].strip())
                hashes = re.findall(r"(?m)^Approval hash: ([0-9a-f]{64})\s*$", output)
                if len(hashes) != 1 or not HASH.fullmatch(hashes[0]):
                    raise GateError(502, "WinGet preflight returned no unique approval hash")
                preview_id = secrets.token_urlsafe(24)
                self._pending = {
                    "preview_id": preview_id,
                    "hash": hashes[0],
                    "ids": selected,
                    "plan_path": plan_path,
                    "temporary": directory,
                    "expires_at": time.monotonic() + PREVIEW_TTL_SECONDS,
                }
                # Metadata is for text-only display, not safe to interpret as markup.
                readable = "".join(char for char in output
                                   if char in "\n\t" or (char.isprintable() and char != "\x1b"))
                return {
                    "schema_version": "trellis.approval-preview.v0.1",
                    "preview_id": preview_id,
                    "approval_hash": hashes[0],
                    "ids": selected,
                    "expires_in_seconds": PREVIEW_TTL_SECONDS,
                    "preflight": readable[-32000:],
                }
            except BaseException:
                directory.cleanup()
                raise

    def apply(self, preview_id: object, submitted_hash: object,
              acknowledged: object) -> dict:
        with self._lock:
            if self._job is not None and self._job["state"] == "running":
                raise GateError(409, "An installation batch is already active")
            pending = self._pending
            if pending is None:
                raise GateError(409, "Run a new preview before approving an installation")
            if time.monotonic() > pending["expires_at"]:
                self._cleanup_pending()
                raise GateError(409, "Preview expired. Run a new preview.")
            if (not isinstance(preview_id, str)
                    or not secrets.compare_digest(preview_id, pending["preview_id"])
                    or not isinstance(submitted_hash, str)
                    or not secrets.compare_digest(submitted_hash, pending["hash"])
                    or acknowledged is not True):
                raise GateError(403, "Exact approval hash and acknowledgement required")
            # Consume a preview exactly once, BEFORE starting any subprocess.
            self._pending = None
            self._job = {
                "state": "running", "ids": pending["ids"],
                "started_at": int(time.time()), "message": "Approved batch launched",
            }
            thread = threading.Thread(
                target=self._apply_worker, args=(pending,), daemon=True,
                name="trellis-approval-install",
            )
            thread.start()
            return {"schema_version": "trellis.approval-job.v0.1",
                    "accepted": True, "job": self._state()}

    def _apply_worker(self, pending: dict) -> None:
        message = "WinGet batch completed; verify applications in Windows"
        state = "finished"
        code = -1
        try:
            code, output = self.runner(
                self._argv(pending["plan_path"], pending["hash"]),
                INSTALL_TIMEOUT_SECONDS,
            )
            if code != 0:
                state = "failed"
                # No raw stdout in the browser; it can contain local paths.
                message = "Trellis reported an installation failure. Review local CLI receipts."
        except GateError:
            state = "failed"
            message = "Trellis installer failed or timed out. Review local journals."
        except Exception:
            state = "failed"
            message = "Local installer encountered an unexpected error."
        finally:
            pending["temporary"].cleanup()
            with self._lock:
                self._job = {
                    "state": state, "ids": pending["ids"],
                    "started_at": self._job["started_at"] if self._job else None,
                    "finished_at": int(time.time()), "exit_code": code,
                    "message": message,
                }
