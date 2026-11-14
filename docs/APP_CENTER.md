# Φ Trellis App Center (preview)

This **optional, static, unprivileged interface** is the start of a Ninite-style app picker for PhiOS and Trellis. It is deliberately **not** an installer, remote execution service, marketplace, or replacement for Trellis CLI's local package state.

## Run

From the Trellis checkout root:

    python3 -m http.server 8000

Visit http://127.0.0.1:8000/app-center/ in a browser. Opening index.html as a file may prevent catalog.json loading due to browser-origin restrictions.

The interface supports search, platform/source filters, checkboxes, a readable selection plan, downloadable JSON, and copying of review commands. The plan contains approval: not_granted and execution: none. **Nothing is installed by clicking a checkbox or exporting a plan.**

## Seed catalog and trust boundaries

app-center/catalog.json uses schema trellis.app-center.v0.1 and currently includes:

- **Native Trellis**: the three real local package specifications in packages/. These can be evaluated through the existing Trellis CLI on the declared Linux/macOS platforms; fixture status is not production certification.
- **WinGet targets**: suggested package IDs for Windows. They are not Trellis native packages. They require an installed WinGet, exact package lookup, provenance/license review, and affirmative operator execution.
- **Flatpak targets**: suggested Flathub IDs for Linux. The Flathub remote, package identity, runtime and permissions require local review.
- **Φ public source repositories**: a snapshot of the publicly visible FieldConstellation inventory, captured 2026-11-04. Every repo is classified as review_required or archived_review, **not install-ready**. The public inventory includes libraries, frameworks, documentation and websites that may never qualify as installed apps.

The optional Refresh public Φ repos button directly calls GitHub's **unauthenticated, public** user-repository API from the browser and fully paginates up to 1,000 entries. A failed or incomplete refresh leaves the bundled snapshot intact. Successful refresh replaces the browser's source list, without writing to disk or changing the on-disk catalog. It never queries private repos, requires tokens, or gains install authority. GitHub rate limits apply.

Catalog entries are rendered as text nodes (not untrusted innerHTML); outbound links use HTTPS and noreferrer/noopener. Selectable repository URLs never imply the source was built or reviewed. Any generated shell commands are suggestions to review on the **appropriate target OS**, never commands executed by the page.

## PhiOS integration architecture

1. **Discovery**: RepoNerve provides locally sourced, timestamped inventory and revision evidence; FieldConstellation offers the public map. Neither repo counts nor GitHub Pages links imply installability.
2. **Profile**: An explicit PhiOS App Platform command (phi-app profile-github https://github.com/OWNER/REPO) review uses the existing bounded repository profiler. It can report blockers, not automatic install approval. This CLI must be available in the target environment.
3. **Promotion**: Each repo needs a human-reviewed manifest, supported platform/runtime, license/provenance check, exact source commit, reproducible build evidence, verified artifacts, and operator consent before promotion into a native installable catalog.
4. **Execution adapters**: WinGet on Windows, Flatpak/OS-native packages on Linux, Trellis local specs, and PhiOS governed app lifecycle remain separate adapters. Do not silently swap package managers or broaden their authority.
5. **Batch install** (not implemented): preflight the full selection, report incompatible/blocked entries, request separate confirmation for material effects, install sequentially with per-app logs, keep versioned receipts and restart/retry options. No script is executed merely by importing this plan.
6. **PhiOS UX** (future): embed or recreate this optional surface in PhiOS Web, Linux shell, Mobile and XR as context allows; not all targets can install native programs.

The generated selection-plan.v0.1 JSON is an advisory interoperability proposal, not currently accepted as an executable Trellis or PhiOS command input.

## Validation

    python3 scripts/check_app_center.py
    node --check app-center/app.js

Both checks run in GitHub CI. Runtime acceptance still requires real browsers and target-platform checks. No installation, Windows package lookup, Linux Flatpak, or physical PhiOS machine was exercised by these checks.

## Why not add all GitHub repositories to the native registry?

Trellis's real package specs have sources, install entries, platform declarations, and provenance. Arbitrary repositories cannot honestly be represented as verified installable packages. The App Center keeps them discoverable as **review-only** until those gates are satisfied.

See also docs/PHIOS_APP_PLATFORM_V0.58_REPO_PROFILER.md in the PhiOS repository.
