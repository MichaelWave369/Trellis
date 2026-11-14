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
5. **Native batch Rung 2**: the Trellis CLI now imports exported selection-plan.v0.1 files, performs a dry-run preflight, and accepts explicit checksum-bound approval for existing native Trellis packages only. WinGet, Flatpak, archived repos, and Φ repo-review entries are held; a mixed plan never partially installs the native entries. WinGet support is implemented separately in Rung 3; Flatpak and Φ repo installation remain future work.
6. **PhiOS UX** (future): embed or recreate this optional surface in PhiOS Web, Linux shell, Mobile and XR as context allows; not all targets can install native programs.

The generated selection-plan.v0.1 JSON is advisory and cannot grant authority. Trellis accepts selected package identifiers only through separately approved local native and Windows adapters; PhiOS does not automatically import the plan.

## Rung 2: Native batch from the downloaded plan

1. Select one or more entries in the **Native Trellis** category, then download the JSON plan from the browser.
2. On a Linux/macOS host supported by the actual local package specifications, build Trellis and initialize/update its index:

       cargo build
       ./target/debug/trellis --registry-root packages seed

3. Preview the entire plan (this does not install or modify profile state):

       ./target/debug/trellis --registry-root packages batch --plan trellis-selection-plan.json

4. Read every line, especially HELD warnings, supported OS and the full **Approval hash**. Only for an all-native plan that passes preflight, explicitly apply:

       ./target/debug/trellis --registry-root packages batch --plan trellis-selection-plan.json --apply --confirm <FULL_APPROVAL_HASH_FROM_PREVIEW>

The hash binds the exact plan file bytes, resolved local package specifications, and their local payload SHA-256 checksums. Editing the plan or local spec/payload requires a new preview and authorization. Browser-provided suggested_commands are treated as **untrusted data and never executed**. Version mismatches and checksum failures block the batch. Native dependencies are resolved in deterministic order. Package receipts and a profile lock are written after successful steps so earlier progress remains visible if a later step fails.

**Limits:** This is not a transactional batch, a fully cryptographic signature or a general-purpose installation sandbox. It does not roll back partial installs, elevate privileges, use network package managers, or install directly from GitHub. A package's checksum establishes content integrity against the locally declared expected digest; it is not publisher authentication. Inspect the local registry source and run on a test system first.

Windows WinGet targets are *discovery and review candidates*, not executable via Trellis's native batch command. The current native Trellis specs declare Linux/macOS platforms. PhiOS App Platform approval remains a separate authority boundary.

## Rung 3: WinGet batch on Windows

The App Center's existing Windows target list can now be passed into a **Windows-only** Trellis adapter. The browser still cannot install anything. Install authorization happens only in a locally launched Trellis CLI on Windows.

From a current Trellis checkout with Rust, Windows App Installer / WinGet installed, and its community source configured:

1. In App Center, set **Source type → WinGet** or **Target → Windows**, select one or more WinGet apps, and download the JSON selection plan. Do not mix native Trellis, Flatpak or Φ repo items into this plan.
2. Build and verify the local Trellis project:

       cargo build

3. Run the WinGet **preflight** from the Trellis checkout root. It queries WinGet for each exact ID from the bundled catalog and prints one full approval hash. No installers are launched by this step.

       .\target\debug\trellis.exe winget-batch --plan trellis-selection-plan.json

4. Inspect the exact ID, publisher, license and installer metadata shown by the native WinGet commands. Only after those checks, run the explicitly approved batch with the hash printed in step 3:

       .\target\debug\trellis.exe winget-batch --plan trellis-selection-plan.json --apply --confirm FULL_APPROVAL_HASH

The default catalog file is app-center/catalog.json relative to the current working directory. Use --catalog PATH only when working from another directory and make sure it is the reviewed bundled file.

**Security and effects**:

- Only entries with kind=winget, status=external_preflight, platform=windows and exact package IDs present in the local App Center catalog are eligible. A mixed, forged, noncatalog or malformed plan fails **before** installing anything.
- WinGet commands use the exact --id, --exact and --source winget options. They execute without a shell and never use a browser-supplied command string. The source *name* is fixed but a local WinGet configuration could point that name to a different endpoint; verify with `winget source list --name winget` before approving. Package metadata is displayed for human review, not treated as authenticated publisher evidence.
- Preflight uses winget show and the local WinGet version. The approval hash covers the exact raw plan and catalog bytes, plus the returned metadata output and WinGet version. This hash is an **explicit confirmation guard**, not a publisher signature or immutable version pin.
- The adapter uses --no-upgrade to avoid unrequested upgrades. It never silently accepts source or package agreement terms, never suppresses installer security hash checks, and never forces elevated privileges. A package installer may still request UAC approval or require a license agreement; blocked terms are left for the operator to resolve separately.
- Preflight may contact the WinGet source or update its caches. On apply, each install runs in order and a local external record with the WinGet process exit code is written under TRELLIS_HOME/external/winget/<approval-hash>/. Nonzero exits stop later installations.
- A WinGet exit code of zero is recorded as **reported success, not installed-state verification**. WinGet is a third-party package manager that may run installer executables. Verify the resulting software independently on the Windows machine. There is no rollback, version locking, sandbox guarantee or transactional batch recovery.

The WinGet adapter is separate from the native Trellis batch and from PhiOS App Platform's governed repository builder. Windows installation and privileges remain the responsibility of the local operator and WinGet. The repo's Linux CI runs deterministic mock-backend tests; Windows CI exercises the same tests with a mock, not actual package installs.

## Rung 4: WinGet observation and progress (Windows)

Rung 4 separates *what the external installer reported* from *what Windows Package Manager can currently observe*:

1. On Windows, export a WinGet-only plan from App Center.
2. Run a read-only installed-state observation:

       .\target\debug\trellis.exe winget-status --plan trellis-selection-plan.json --output trellis-winget-status.json

3. In the App Center browser page, manually choose the newly created JSON under **Import a local WinGet status report**. It displays three conservative labels: `observed_in_winget_list`, `not_confirmed_in_winget_list`, and `unknown`. The report never installs software and is read by the browser **only after explicit user file selection**.

A package appearing in `winget list --id PACKAGE_ID --exact --source winget` is a **package-manager observation**, not proof that the program launches, works, is secure, or can be safely upgraded. A negative or failed query does not prove the software is absent. If the WinGet CLI fails or returns unusable output, Trellis reports `unknown`. Status reports are only written to new paths with `--output`; existing files are never silently overwritten. Avoid storing status reports in publicly served folders unless you explicitly want those observations visible.

For a confirmed WinGet-only batch executed with `winget-batch --apply --confirm HASH`, Trellis now records two per-package progress events (`started`, then `finished` or `launch_failed`) in a local append-only JSONL journal:

    TRELLIS_HOME/external/winget/<approval-hash>/events.jsonl

Successful WinGet process exit and later list observation are separate fields in each package's external receipt. An interrupted process may leave only its started event. The journal can help diagnose where execution stopped; it is **not** a transactional recovery, independent installer health test, rollback system, or streaming UI connection. The optional Rung 5 bridge below can expose a strictly read-only view of this journal after explicit local token pairing; it never grants install authority.

This rung runs CLI and mocked backend tests on Linux and Windows GitHub runners; no live Windows applications are installed during CI. Physical Windows verification is still required.

## Rung 5: Paired, read-only localhost progress bridge

This optional **Python standard-library-only** service serves the existing App Center from loopback and exposes a bounded view of *local WinGet batch progress journals*. It does not expose installation or plan-upload endpoints.

On Windows, from the Trellis repository root, using the same home folder as the Trellis CLI:

    python .\scripts\app_center_bridge.py --home "$env:APPDATA\Trellis"

On Linux/macOS development machines, use a test Trellis home:

    python3 scripts/app_center_bridge.py --home /path/to/test/trellis-home

The service binds **127.0.0.1:8765 only** by default and prints an ephemeral random pairing token in the local console. Open **http://127.0.0.1:8765/app-center/** on that same machine, paste the token in **Local progress bridge**, then select **Pair and watch**. Keep the console open; Ctrl+C stops the service. The token is never stored in localStorage, cookies or the catalog, and is cleared on disconnect/page exit.

The paired page polls the exact same-origin endpoint \`GET /api/bridge/v1/progress\` every three seconds with \`X-Trellis-Bridge-Token\`. The endpoint returns only a bounded, schema-checked view of the latest WinGet batch journals found at:

    TRELLIS_HOME/external/winget/<approval-hash>/events.jsonl

This is **near-live journal monitoring, not a new installer process**. Run a separately reviewed \`trellis winget-batch --apply --confirm HASH\` in another local terminal after its own preflight to produce progress events. A \`started\` event without a \`finished\` event may represent an interrupted process, not a currently running program. A WinGet list observation is not proof of functional installation.

**Trust boundaries:** The bridge serves only allowlisted App Center files and the one authenticated progress endpoint; it rejects non-loopback Host values, foreign Origin headers, unpaired API requests, query-token passing, and all POST/OPTIONS actions. It sets strict same-origin/CSP/no-store headers, reads only bounded journal files under hexadecimal session directories, skips symlinks, and strips unexpected fields from events. The service does not spawn executables, write install state, request elevation, forward browser credentials, or bind to the LAN. There is no CORS access for external websites.

The pairing token guards against other webpages querying the service, but it does not defend against untrusted code already running with the same user's filesystem/process access, browser extensions with broad permissions, or a compromised local machine. Keep the token private. Avoid forwarding the port publicly. Public GitHub catalog refresh remains an optional unauthenticated request to api.github.com.

**Testing:** 

    python3 -m unittest discover -s tests -p 'test_app_center_bridge.py'
    node --check app-center/app.js

CI runs the Python bridge suite on both Ubuntu and Windows, including unauthorized access, DNS-rebinding-style Host rejection, cross-origin rejection, read-only endpoints, malicious journal input, and symlink handling. It does not perform real installer execution or verify physical Windows application health.

## Validation

    python3 scripts/check_app_center.py
    node --check app-center/app.js

Both checks run in GitHub CI. Runtime acceptance still requires real browsers and target-platform checks. No installation, Windows package lookup, Linux Flatpak, or physical PhiOS machine was exercised by these checks.

## Why not add all GitHub repositories to the native registry?

Trellis's real package specs have sources, install entries, platform declarations, and provenance. Arbitrary repositories cannot honestly be represented as verified installable packages. The App Center keeps them discoverable as **review-only** until those gates are satisfied.

See also docs/PHIOS_APP_PLATFORM_V0.58_REPO_PROFILER.md in the PhiOS repository.
