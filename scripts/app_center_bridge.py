#!/usr/bin/env python3
"""Trellis Rung 5: token-gated, read-only loopback progress bridge.

Serves only the bundled App Center and bounded, sanitized progress observations.
Does not run installers, accept uploaded plans, or expose the filesystem.
"""

from __future__ import annotations

import argparse
import hmac
import json
import os
import re
import secrets
import sys
from app_center_install_gate import ApprovalGate, GateError
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

MAX_JOURNAL_BYTES = 512 * 1024
MAX_SESSIONS = 12
MAX_EVENTS = 1000
HASH_PATTERN = re.compile(r"[a-fA-F0-9]{64}\Z")
PACKAGE_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{2,99}\Z")
PHASES = {"started", "finished", "launch_failed"}
OBSERVATIONS = {
    None, "observed_in_winget_list", "not_confirmed_in_winget_list", "unknown"
}
STATIC = {
    "/app-center/": ("index.html", "text/html; charset=utf-8"),
    "/app-center/index.html": ("index.html", "text/html; charset=utf-8"),
    "/app-center/app.js": ("app.js", "text/javascript; charset=utf-8"),
    "/app-center/styles.css": ("styles.css", "text/css; charset=utf-8"),
    "/app-center/catalog.json": ("catalog.json", "application/json; charset=utf-8"),
}


def default_home() -> Path:
    xdg = os.environ.get("XDG_DATA_HOME")
    if xdg:
        return Path(xdg) / "trellis"
    if os.name == "nt" and os.environ.get("APPDATA"):
        return Path(os.environ["APPDATA"]) / "Trellis"
    return Path(os.environ.get("HOME", ".")) / ".local" / "share" / "trellis"


def safe_event(value: object) -> dict | None:
    """Only pass through the documented, bounded WinGet journal fields."""
    if not isinstance(value, dict):
        return None
    if value.get("schema_version") != "trellis.winget-progress.v0.1":
        return None
    package = value.get("package_id")
    if not isinstance(package, str) or not PACKAGE_PATTERN.fullmatch(package):
        return None
    index = value.get("index")
    total = value.get("total")
    if type(index) is not int or type(total) is not int or not (1 <= index <= total <= 100):
        return None
    phase = value.get("phase")
    if phase not in PHASES:
        return None
    exit_code = value.get("exit_code")
    if exit_code is not None and (type(exit_code) is not int or abs(exit_code) > 2**32):
        return None
    observation = value.get("observation")
    if observation not in OBSERVATIONS:
        return None
    timestamp = value.get("observed_at_utc")
    if not isinstance(timestamp, str) or len(timestamp) > 64:
        return None
    return {
        "package_id": package, "index": index, "total": total,
        "phase": phase, "exit_code": exit_code, "observation": observation,
        "observed_at_utc": timestamp,
    }


def progress_snapshot(home: Path) -> dict:
    base = home / "external" / "winget"
    sessions: list[dict] = []
    if base.is_symlink() or not base.is_dir():
        return {"schema_version": "trellis.bridge-progress.v0.1", "sessions": sessions}
    try:
        candidates = []
        for entry in base.iterdir():
            if (not HASH_PATTERN.fullmatch(entry.name) or entry.is_symlink()
                    or not entry.is_dir()):
                continue
            journal = entry / "events.jsonl"
            if journal.is_symlink() or not journal.is_file():
                continue
            size = journal.stat().st_size
            if not (0 < size <= MAX_JOURNAL_BYTES):
                continue
            candidates.append((journal.stat().st_mtime_ns, entry.name, journal))
        candidates.sort(key=lambda item: item[0], reverse=True)
        for _, digest, journal in candidates[:MAX_SESSIONS]:
            events = []
            try:
                with journal.open("r", encoding="utf-8", errors="replace") as stream:
                    for line in stream:
                        if len(events) >= MAX_EVENTS:
                            break
                        if len(line) > 8192:
                            continue
                        try:
                            parsed = safe_event(json.loads(line))
                        except (ValueError, TypeError):
                            continue
                        if parsed is not None:
                            events.append(parsed)
            except (OSError, UnicodeError):
                continue
            if events:
                sessions.append({"approval_hash": digest, "events": events})
    except OSError:
        pass
    return {"schema_version": "trellis.bridge-progress.v0.1", "sessions": sessions}


def build_server(app_root: Path, home: Path, token: str, port: int = 8765,
                 approval_gate: ApprovalGate | None = None) -> ThreadingHTTPServer:
    """Bind exclusively to 127.0.0.1. Token only lives in this process."""
    app_root = app_root.resolve()
    home = home.expanduser().resolve()

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, fmt: str, *args: object) -> None:
            # Omit query strings and request headers from logs, even on rejected requests.
            method = str(getattr(self, "command", None) or "unknown")
            path = str(getattr(self, "path", None) or "/")
            sys.stderr.write("[trellis-bridge] " + method + " "
                             + urlsplit(path).path + "\n")

        def headers_common(self) -> None:
            self.send_header("Cache-Control", "no-store, max-age=0")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Referrer-Policy", "no-referrer")
            self.send_header("X-Frame-Options", "DENY")
            self.send_header("Cross-Origin-Resource-Policy", "same-origin")
            self.send_header(
                "Content-Security-Policy",
                "default-src 'none'; script-src 'self'; style-src 'self'; "
                "connect-src 'self' https://api.github.com; "
                "img-src 'self' data:; base-uri 'none'; frame-ancestors 'none'; "
                "object-src 'none'; form-action 'none'",
            )

        def respond(self, code: int, body: bytes, content_type: str = "text/plain; charset=utf-8") -> None:
            self.send_response(code)
            self.headers_common()
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def allowed_request(self) -> bool:
            # Host check rejects DNS rebinding and alternate domain origins.
            expected_host = f"127.0.0.1:{self.server.server_port}"
            expected_origin = f"http://{expected_host}"
            if self.headers.get("Host") != expected_host:
                self.respond(403, b"Invalid loopback host\n")
                return False
            origin = self.headers.get("Origin")
            if origin is not None and origin != expected_origin:
                self.respond(403, b"Cross-origin requests forbidden\n")
                return False
            return True

        def do_GET(self) -> None:
            if not self.allowed_request():
                return
            target = urlsplit(self.path)
            if target.query or target.fragment:
                self.respond(400, b"Query parameters not supported\n")
                return
            if target.path == "/":
                self.send_response(302)
                self.headers_common()
                self.send_header("Location", "/app-center/")
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            if target.path == "/api/bridge/v1/progress":
                submitted = self.headers.get("X-Trellis-Bridge-Token", "")
                if not hmac.compare_digest(submitted, token):
                    self.respond(401, b"Bridge pairing token required\n")
                    return
                payload = json.dumps(progress_snapshot(home), separators=(",", ":")).encode()
                self.respond(200, payload, "application/json; charset=utf-8")
                return
            if target.path == "/api/bridge/v1/approval-state":
                submitted = self.headers.get("X-Trellis-Bridge-Token", "")
                if not hmac.compare_digest(submitted, token):
                    self.respond(401, b"Bridge pairing token required\n")
                    return
                report = (approval_gate.status() if approval_gate is not None else {
                    "schema_version": "trellis.approval-status.v0.1",
                    "enabled": False, "job": {"state": "idle"},
                })
                self.respond(200, json.dumps(report).encode(), "application/json; charset=utf-8")
                return
            resource = STATIC.get(target.path)
            if resource is None:
                self.respond(404, b"Not found\n")
                return
            filename, content_type = resource
            local = app_root / filename
            # Do not permit symlinked site content or filesystem traversal.
            if local.is_symlink() or local.parent != app_root:
                self.respond(404, b"Not found\n")
                return
            try:
                payload = local.read_bytes()
            except OSError:
                self.respond(404, b"Not found\n")
                return
            self.respond(200, payload, content_type)

        def do_POST(self) -> None:
            if not self.allowed_request():
                return
            target = urlsplit(self.path)
            if target.query or target.fragment:
                self.respond(400, b"Query parameters not supported\n")
                return
            if target.path not in {"/api/bridge/v1/preview", "/api/bridge/v1/apply"}:
                self.respond(405, b"Unknown or disabled bridge action\n")
                return
            if approval_gate is None:
                self.respond(403, b"Install approval API is disabled by default\n")
                return
            expected_origin = f"http://127.0.0.1:{self.server.server_port}"
            if self.headers.get("Origin") != expected_origin:
                self.respond(403, b"Same-origin user action required\n")
                return
            submitted = self.headers.get("X-Trellis-Bridge-Token", "")
            if not hmac.compare_digest(submitted, token):
                self.respond(401, b"Bridge pairing token required\n")
                return
            if self.headers.get("Content-Type", "").split(";")[0].strip() != "application/json":
                self.respond(415, b"JSON body required\n")
                return
            try:
                length = int(self.headers.get("Content-Length", "0"))
            except ValueError:
                self.respond(400, b"Invalid body length\n")
                return
            if not 0 < length <= 8192:
                self.respond(413, b"Body must be 1 to 8192 bytes\n")
                return
            try:
                self.connection.settimeout(15)
                body = self.rfile.read(length)
                data = json.loads(body)
            except (OSError, ValueError, TimeoutError):
                self.respond(400, b"Invalid JSON request\n")
                return
            if not isinstance(data, dict):
                self.respond(400, b"JSON object required\n")
                return
            try:
                if target.path.endswith("/preview"):
                    if set(data) != {"ids"}:
                        raise GateError(400, "Only selected application IDs are accepted")
                    result = approval_gate.preview(data["ids"])
                    status = 200
                else:
                    if set(data) != {"preview_id", "approval_hash", "acknowledge"}:
                        raise GateError(400, "Unexpected approval parameters")
                    result = approval_gate.apply(
                        data["preview_id"], data["approval_hash"], data["acknowledge"]
                    )
                    status = 202
            except GateError as error:
                self.respond(error.status, str(error).encode()[:4000])
                return
            except Exception:
                self.respond(500, b"Local approval bridge encountered an internal error\n")
                return
            self.respond(status, json.dumps(result).encode(), "application/json; charset=utf-8")

        def do_OPTIONS(self) -> None:
            if not self.allowed_request():
                return
            self.respond(405, b"Cross-origin access not available\n")

        def do_HEAD(self) -> None:
            if not self.allowed_request():
                return
            self.respond(405, b"HEAD not supported\n")

    return ThreadingHTTPServer(("127.0.0.1", port), Handler)


def main() -> None:
    parser = argparse.ArgumentParser(description="Read-only, token-gated Trellis App Center localhost bridge")
    parser.add_argument("--home", type=Path, default=default_home(),
                        help="Trellis home containing external/winget progress journals")
    parser.add_argument("--port", type=int, default=8765, help="127.0.0.1-only HTTP port (default 8765)")
    parser.add_argument("--enable-install-api", action="store_true",
                        help="Explicitly opt in to token-gated local Windows WinGet installs")
    parser.add_argument("--trellis-cli", type=Path,
                        help="Reviewed local Trellis executable used ONLY with --enable-install-api")
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("port must be between 1 and 65535")
    root = Path(__file__).resolve().parent.parent / "app-center"
    token = secrets.token_urlsafe(32)
    gate = None
    if args.enable_install_api:
        if os.name != "nt":
            parser.error("opt-in installation actions are Windows-only")
        if args.trellis_cli is None:
            parser.error("--enable-install-api requires an explicit --trellis-cli PATH")
        if args.trellis_cli.is_symlink() or not args.trellis_cli.is_file():
            parser.error("--trellis-cli must be a real local Trellis executable, not a symlink")
        gate = ApprovalGate(root / "catalog.json", args.trellis_cli.resolve(), args.home)
    elif args.trellis_cli is not None:
        parser.error("--trellis-cli requires --enable-install-api")
    server = build_server(root, args.home, token, args.port, approval_gate=gate)
    print(f"Trellis App Center: http://127.0.0.1:{server.server_port}/app-center/", flush=True)
    print(f"Bridge pairing token (private, do not share): {token}", flush=True)
    if gate is None:
        print("Read-only local progress bridge. Install API disabled.", flush=True)
    else:
        print("OPT-IN Windows install API active. Every batch still requires preview, "
              "full-hash re-entry and explicit acknowledgement.", flush=True)
    print("Ctrl+C to stop.", flush=True)
    try:
        server.serve_forever(poll_interval=0.3)
    except KeyboardInterrupt:
        print("\nBridge stopped.", flush=True)
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
