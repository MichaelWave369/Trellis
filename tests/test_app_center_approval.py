"""Rung 6 local approval gate security tests. Never installs real packages."""
from __future__ import annotations

import hashlib
import http.client
import json
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from app_center_bridge import build_server  # noqa: E402
from app_center_install_gate import ApprovalGate, GateError  # noqa: E402


class FakeRunner:
    """Simulates local Trellis preflight and installation without subprocesses."""
    def __init__(self):
        self.calls: list[list[str]] = []
        self.preflight_error = False
        self.install_error = False
        self.block_install: threading.Event | None = None

    def __call__(self, argv: list[str], timeout: int) -> tuple[int, str]:
        self.calls.append(argv[:])
        if "--apply" in argv:
            if self.block_install is not None:
                self.block_install.wait(timeout=2)
            return (123 if self.install_error else 0), "Mock install process completed."
        if self.preflight_error:
            return 42, "Mock preflight blocked"
        plan = Path(argv[argv.index("--plan") + 1]).read_bytes()
        catalog = Path(argv[argv.index("--catalog") + 1]).read_bytes()
        value = hashlib.sha256(plan + catalog).hexdigest()
        return 0, (
            "Φ Trellis WinGet Batch · 1 candidate\n"
            "WinGet metadata for Git.Git:\nFound Git [Git.Git]\n"
            "Version: 2.50\nPublisher: Test Publisher\n"
            "Approval hash: " + value + "\n"
            "DRY RUN: nothing installed.\n"
        )


class ApprovalTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.app_root = self.root / "app-center"
        self.app_root.mkdir()
        (self.app_root / "index.html").write_text("Trellis", encoding="utf-8")
        (self.app_root / "app.js").write_text("", encoding="utf-8")
        (self.app_root / "styles.css").write_text("", encoding="utf-8")
        self.catalog = self.app_root / "catalog.json"
        self.catalog.write_text(json.dumps({
            "schema_version": "trellis.app-center.v0.1",
            "apps": [
                {
                    "id": "winget:Git.Git", "kind": "winget",
                    "package_id": "Git.Git", "status": "external_preflight",
                    "platforms": ["windows"],
                },
                {
                    "id": "winget:VideoLAN.VLC", "kind": "winget",
                    "package_id": "VideoLAN.VLC", "status": "external_preflight",
                    "platforms": ["windows"],
                },
                {
                    "id": "phi:BTM", "kind": "phi_repo", "status": "review_required",
                    "platforms": ["phios"],
                },
            ],
        }), encoding="utf-8")
        self.home = self.root / "trellis-home"
        self.home.mkdir()
        self.runner = FakeRunner()
        self.gate = ApprovalGate(
            self.catalog, self.root / "trellis-test.exe", self.home, runner=self.runner
        )
        self.token = "local-r6-test-token"
        self.server = build_server(
            self.app_root, self.home, self.token, port=0, approval_gate=self.gate
        )
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=3)
        self.temp.cleanup()

    def http(self, path: str, *, method="GET", data=None,
             origin=True, token=True, headers=None):
        port = self.server.server_port
        merged = {}
        if token:
            merged["X-Trellis-Bridge-Token"] = self.token
        if origin:
            merged["Origin"] = f"http://127.0.0.1:{port}"
        if data is not None:
            merged["Content-Type"] = "application/json"
            body = json.dumps(data).encode()
        else:
            body = None
        merged.update(headers or {})
        connection = http.client.HTTPConnection("127.0.0.1", port, timeout=4)
        connection.request(method, path, body=body, headers=merged)
        response = connection.getresponse()
        status = response.status
        payload = response.read()
        connection.close()
        try:
            return status, json.loads(payload)
        except (ValueError, UnicodeError):
            return status, payload.decode(errors="replace")

    def valid_preview(self):
        status, preview = self.http(
            "/api/bridge/v1/preview",
            method="POST", data={"ids": ["winget:Git.Git"]}
        )
        self.assertEqual(status, 200, preview)
        return preview

    def wait_for_done(self):
        for _ in range(100):
            report = self.gate.status()
            if report["job"]["state"] != "running":
                return report
            time.sleep(0.01)
        self.fail("fake install worker did not finish")

    def test_read_only_bridge_by_default(self):
        readonly = build_server(self.app_root, self.home, self.token, port=0)
        t = threading.Thread(target=readonly.serve_forever, daemon=True)
        t.start()
        try:
            conn = http.client.HTTPConnection("127.0.0.1", readonly.server_port, timeout=3)
            conn.request("POST", "/api/bridge/v1/preview", body=b"{}",
                         headers={
                             "Host": f"127.0.0.1:{readonly.server_port}",
                             "Origin": f"http://127.0.0.1:{readonly.server_port}",
                             "X-Trellis-Bridge-Token": self.token,
                             "Content-Type": "application/json",
                         })
            res = conn.getresponse()
            self.assertEqual(res.status, 403)
            res.read()
            conn.close()
        finally:
            readonly.shutdown()
            readonly.server_close()
            t.join(timeout=3)

    def test_preview_requires_token_and_same_origin(self):
        for opts in ({"token": False}, {"origin": False},
                     {"headers": {"Origin": "https://attacker.example"}}):
            status, _ = self.http(
                "/api/bridge/v1/preview", method="POST",
                data={"ids": ["winget:Git.Git"]}, **opts
            )
            self.assertIn(status, (401, 403))
        self.assertEqual(self.runner.calls, [])

    def test_local_allowlist_blocks_repository_and_fake_id(self):
        for bad in (["phi:BTM"], ["winget:Unknown.App"],
                    ["winget:Git.Git", "phi:BTM"],
                    ["winget:Git.Git", "winget:Git.Git"]):
            status, _ = self.http(
                "/api/bridge/v1/preview", method="POST", data={"ids": bad}
            )
            self.assertEqual(status, 400)
        self.assertEqual(self.runner.calls, [])

    def test_only_exact_request_shape_and_no_client_command(self):
        status, _ = self.http("/api/bridge/v1/preview", method="POST",
                              data={"ids": ["winget:Git.Git"], "command": "powershell -c evil"})
        self.assertEqual(status, 400)
        self.assertEqual(self.runner.calls, [])

    def test_preview_does_not_install_and_uses_fixed_args(self):
        preview = self.valid_preview()
        self.assertEqual(preview["schema_version"], "trellis.approval-preview.v0.1")
        self.assertIn("Found Git [Git.Git]", preview["preflight"])
        self.assertEqual(len(preview["approval_hash"]), 64)
        self.assertEqual(len(self.runner.calls), 1)
        command = self.runner.calls[0]
        self.assertEqual(command[0], str(self.root / "trellis-test.exe"))
        self.assertEqual(command[1], "--home")
        self.assertEqual(Path(command[2]).resolve(), self.home.resolve())
        self.assertEqual(command[3], "winget-batch")
        self.assertNotIn("--apply", command)
        self.assertEqual(preview["ids"], ["winget:Git.Git"])
        self.assertFalse((self.home / "external").exists())

    def test_must_retype_full_hash_and_explicitly_acknowledge(self):
        preview = self.valid_preview()
        for digest, ack in (("bad", True), (preview["approval_hash"], False)):
            status, _ = self.http("/api/bridge/v1/apply", method="POST",
                                  data={
                                      "preview_id": preview["preview_id"],
                                      "approval_hash": digest,
                                      "acknowledge": ack,
                                  })
            self.assertEqual(status, 403)
        self.assertEqual(len(self.runner.calls), 1)
        status, job = self.http("/api/bridge/v1/apply", method="POST",
                                data={
                                    "preview_id": preview["preview_id"],
                                    "approval_hash": preview["approval_hash"],
                                    "acknowledge": True,
                                })
        self.assertEqual(status, 202, job)
        self.assertTrue(job["accepted"])
        self.assertEqual(self.wait_for_done()["job"]["state"], "finished")
        self.assertEqual(len(self.runner.calls), 2)
        self.assertEqual(self.runner.calls[1][-3:], [
            "--apply", "--confirm", preview["approval_hash"]
        ])
        # Replay of the one-time approval must fail.
        status, _ = self.http("/api/bridge/v1/apply", method="POST",
                              data={
                                  "preview_id": preview["preview_id"],
                                  "approval_hash": preview["approval_hash"],
                                  "acknowledge": True,
                              })
        self.assertEqual(status, 409)
        self.assertEqual(len(self.runner.calls), 2)

    def test_expired_preview_fails_without_install(self):
        preview = self.valid_preview()
        with self.gate._lock:
            self.gate._pending["expires_at"] = time.monotonic() - 1
        status, _ = self.http("/api/bridge/v1/apply", method="POST",
                              data={
                                  "preview_id": preview["preview_id"],
                                  "approval_hash": preview["approval_hash"],
                                  "acknowledge": True,
                              })
        self.assertEqual(status, 409)
        self.assertEqual(len(self.runner.calls), 1)

    def test_only_one_active_batch(self):
        preview = self.valid_preview()
        unblock = threading.Event()
        self.runner.block_install = unblock
        try:
            status, _ = self.http("/api/bridge/v1/apply", method="POST",
                                  data={
                                      "preview_id": preview["preview_id"],
                                      "approval_hash": preview["approval_hash"],
                                      "acknowledge": True,
                                  })
            self.assertEqual(status, 202)
            status, _ = self.http("/api/bridge/v1/preview", method="POST",
                                  data={"ids": ["winget:VideoLAN.VLC"]})
            self.assertEqual(status, 409)
        finally:
            unblock.set()
        self.assertEqual(self.wait_for_done()["job"]["state"], "finished")

    def test_failure_does_not_report_success(self):
        preview = self.valid_preview()
        self.runner.install_error = True
        status, _ = self.http("/api/bridge/v1/apply", method="POST", data={
            "preview_id": preview["preview_id"],
            "approval_hash": preview["approval_hash"], "acknowledge": True
        })
        self.assertEqual(status, 202)
        report = self.wait_for_done()
        self.assertEqual(report["job"]["state"], "failed")
        self.assertEqual(report["job"]["exit_code"], 123)

    def test_only_paired_can_read_job(self):
        status, _ = self.http("/api/bridge/v1/approval-state", token=False)
        self.assertEqual(status, 401)
        status, data = self.http("/api/bridge/v1/approval-state")
        self.assertEqual(status, 200)
        self.assertTrue(data["enabled"])
        self.assertEqual(data["job"]["state"], "idle")

    def test_no_arbitrary_install_endpoint(self):
        status, _ = self.http("/api/bridge/v1/install", method="POST", data={
            "command": "powershell -Command whoami"
        })
        self.assertEqual(status, 405)
        self.assertEqual(self.runner.calls, [])


if __name__ == "__main__":
    unittest.main()
