#!/usr/bin/env python3
"""Validate the optional Trellis App Center's advisory catalog (stdlib only)."""
from __future__ import annotations
import json
import re
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "app-center" / "catalog.json"
KINDS = {"trellis", "winget", "flatpak", "phi_repo"}
EXPECTED_STATUS = {
    "trellis": {"native_catalog"},
    "winget": {"external_preflight"},
    "flatpak": {"external_preflight"},
    "phi_repo": {"review_required", "archived_review"},
}
PLATFORMS = {"linux", "macos", "windows", "phios"}
PACKAGE_ID = re.compile(r"^[a-zA-Z0-9][a-zA-Z0-9.+_-]{1,100}$")
REPO_NAME = re.compile(r"^[a-zA-Z0-9._-]+$")

def main() -> None:
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    assert catalog["schema_version"] == "trellis.app-center.v0.1"
    assert catalog["public_snapshot_complete"] is True
    apps = catalog["apps"]
    assert isinstance(apps, list) and apps
    seen: set[str] = set()
    counts = {kind: 0 for kind in KINDS}
    for i, app in enumerate(apps):
        assert isinstance(app, dict), f"entry {i} is not an object"
        kind = app["kind"]
        assert kind in KINDS, f"unrecognized kind: {kind}"
        assert app["status"] in EXPECTED_STATUS[kind], f"bad {kind} status"
        identifier = app["id"]
        if kind == "phi_repo":
            assert identifier.startswith("phi:"), identifier
        else:
            assert identifier == f"{kind}:{app.get('package_id', '')}", identifier
        assert identifier.casefold() not in seen, f"duplicate identifier: {identifier}"
        seen.add(identifier.casefold())
        assert isinstance(app.get("name"), str) and app["name"].strip(), identifier
        assert isinstance(app.get("category"), str) and app["category"].strip(), identifier
        assert isinstance(app.get("description"), str), identifier
        platforms = app["platforms"]
        assert isinstance(platforms, list) and platforms and set(platforms) <= PLATFORMS, identifier
        source = urlparse(app["source"])
        assert source.scheme == "https" and source.hostname, identifier
        if kind in {"trellis", "winget", "flatpak"}:
            assert PACKAGE_ID.fullmatch(app["package_id"]), identifier
        if kind == "trellis":
            assert source.hostname == "github.com", identifier
            spec = ROOT / "packages" / app["package_id"] / (app["package_id"] + ".trellis.yaml")
            assert spec.is_file(), f"missing real native spec: {spec}"
        if kind == "phi_repo":
            repo = app["repo"]
            assert repo.startswith("MichaelWave369/"), identifier
            name = repo.split("/", 1)[1]
            assert REPO_NAME.fullmatch(name), identifier
            assert identifier == "phi:" + name, identifier
            assert app["source"] == "https://github.com/" + repo, identifier
            if "homepage" in app:
                assert urlparse(app["homepage"]).scheme == "https", identifier
            if app["status"] == "archived_review":
                assert app.get("archived") is True, identifier
        counts[kind] += 1
    assert counts["trellis"] == 3, "Native baseline must match actual bundled specs"
    assert counts["winget"] > 0 and counts["flatpak"] > 0 and counts["phi_repo"] > 0
    print("App Center catalog validated:", counts, "total", len(apps))

if __name__ == "__main__":
    main()
