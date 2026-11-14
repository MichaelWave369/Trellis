"""Security and journal tests for the optional Rung 5 localhost progress bridge."""

from __future__ import annotations

import http.client
import json
import sys
import tempfile
import threading
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import app_center_bridge as bridge  # noqa: E402


class BridgeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.workspace = tempfile.TemporaryDirectory()
        root = Path(self.workspace.name)
        self.app_root = root / "app-center"
        self.app_root.mkdir()
        for name, content in {
            "index.html": "<!doctype html><title>Trellis test</title>",
            "app.js": "console.log('test');",
            "styles.css": "body { color: black; }",
            "catalog.json": '{"apps":[]}',
        }.items():
            (self.app_root / name).write_text(content, encoding="utf-8")
        self.home = root / "trellis-home"
        self.home.mkdir()
        self.token = "secret-example-for-tests"
        self.server = bridge.build_server(self.app_root, self.home, self.token, 0)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.port = self.server.server_port
        self.host = f"127.0.0.1:{self.port}"

    def tearDown(self) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=3)
        self.workspace.cleanup()

    def request(self, path, *, method="GET", headers=None):
        conn = http.client.HTTPConnection("127.0.0.1", self.port, timeout=4)
        conn.request(method, path, headers=headers or {})
        reply = conn.getresponse()
        output = (reply.status, dict(reply.getheaders()), reply.read())
        conn.close()
        return output

    def write_event(self, digest="a" * 64, package_id="Git.Git"):
        folder = self.home / "external" / "winget" / digest
        folder.mkdir(parents=True, exist_ok=True)
        event = {
            "schema_version": "trellis.winget-progress.v0.1",
            "observed_at_utc": "2026-11-14T05:01:00Z",
            "package_id": package_id,
            "index": 1, "total": 2, "phase": "finished",
            "exit_code": 0, "observation": "observed_in_winget_list",
        }
        journal = folder / "events.jsonl"
        journal.write_text(json.dumps(event) + "\n")
        return journal

    def test_static_site_served_with_security_headers(self):
        status, headers, data = self.request("/app-center/")
        self.assertEqual(status, 200)
        self.assertIn(b"Trellis test", data)
        self.assertEqual(headers["Cache-Control"], "no-store, max-age=0")
        self.assertIn("frame-ancestors 'none'", headers["Content-Security-Policy"])
        self.assertNotIn("Access-Control-Allow-Origin", headers)

    def test_bridge_progress_requires_token(self):
        self.write_event()
        denied, _, _ = self.request("/api/bridge/v1/progress")
        self.assertEqual(denied, 401)
        wrong, _, _ = self.request("/api/bridge/v1/progress", headers={
            "X-Trellis-Bridge-Token": "incorrect"
        })
        self.assertEqual(wrong, 401)
        allowed, _, data = self.request("/api/bridge/v1/progress", headers={
            "X-Trellis-Bridge-Token": self.token
        })
        self.assertEqual(allowed, 200)
        report = json.loads(data)
        self.assertEqual(report["schema_version"], "trellis.bridge-progress.v0.1")
        self.assertEqual(len(report["sessions"]), 1)
        self.assertEqual(report["sessions"][0]["events"][0]["package_id"], "Git.Git")
        self.assertNotIn(b"secret-example-for-tests", data)

    def test_rejects_dns_rebinding_host(self):
        status, _, _ = self.request("/api/bridge/v1/progress", headers={
            "Host": "attacker.example",
            "X-Trellis-Bridge-Token": self.token,
        })
        self.assertEqual(status, 403)

    def test_rejects_cross_origin_request_even_with_token(self):
        status, _, _ = self.request("/api/bridge/v1/progress", headers={
            "Origin": "http://attacker.example",
            "X-Trellis-Bridge-Token": self.token,
        })
        self.assertEqual(status, 403)
        allowed, _, _ = self.request("/api/bridge/v1/progress", headers={
            "Origin": f"http://{self.host}",
            "X-Trellis-Bridge-Token": self.token,
        })
        self.assertEqual(allowed, 200)

    def test_no_command_or_upload_endpoints(self):
        status, _, _ = self.request("/api/bridge/v1/progress", method="POST", headers={
            "X-Trellis-Bridge-Token": self.token
        })
        self.assertEqual(status, 405)
        status, _, _ = self.request("/api/bridge/v1/install", headers={
            "X-Trellis-Bridge-Token": self.token
        })
        self.assertEqual(status, 404)
        status, _, _ = self.request("/src/core/winget.rs")
        self.assertEqual(status, 404)
        status, _, _ = self.request("/app-center/../scripts/app_center_bridge.py")
        self.assertIn(status, (400, 404))

    def test_api_does_not_accept_query_token(self):
        status, _, _ = self.request("/api/bridge/v1/progress?token=" + self.token)
        self.assertEqual(status, 400)

    def test_limits_and_filters_progress_events(self):
        journal = self.write_event()
        with journal.open("a", encoding="utf-8") as f:
            f.write("{not json\n")
            f.write(json.dumps({
                "schema_version": "trellis.winget-progress.v0.1",
                "observed_at_utc": "2026-11-14",
                "package_id": "Other.App",
                "index": 0, "total": 1000, "phase": "finished",
                "exit_code": 0, "observation": "unknown",
            }) + "\n")
        result = bridge.progress_snapshot(self.home)
        self.assertEqual(len(result["sessions"][0]["events"]), 1)
        with journal.open("a", encoding="utf-8") as f:
            f.write("x" * (bridge.MAX_JOURNAL_BYTES + 1))
        result = bridge.progress_snapshot(self.home)
        self.assertEqual(result["sessions"], [])

    def test_journal_symlink_is_not_followed(self):
        base = self.home / "external" / "winget"
        folder = base / ("b" * 64)
        folder.mkdir(parents=True)
        target = Path(self.workspace.name) / "private.txt"
        target.write_text("private")
        try:
            (folder / "events.jsonl").symlink_to(target)
        except (OSError, NotImplementedError):
            self.skipTest("symlink privilege not available on test host")
        self.assertEqual(bridge.progress_snapshot(self.home)["sessions"], [])

    def test_no_progress_directory_is_empty(self):
        self.assertEqual(bridge.progress_snapshot(self.home)["sessions"], [])


if __name__ == "__main__":
    unittest.main()
