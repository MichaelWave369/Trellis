# Changelog

## v0.1.0-alpha - 2026-03-20

### Added
- Initial Trellis Rust CLI with init/update/search/info/install/list/remove/doctor commands.
- Local-first Trellis home model with deterministic cellar/receipts/bin/registry/cache directories.
- YAML package spec support (`*.trellis.yaml`) with validation.
- Filesystem registry indexing and package discovery.
- Install receipts with provenance metadata and exposed binary mapping.
- Seeded `vineyard-core` fixture package for end-to-end demos/tests.
- Integration tests for core command flows.
- Project documentation for architecture, spec format, registry model, and roadmap.
- GitHub Actions CI for rustfmt, clippy, and tests.

### Changed
- Hardened install safety by refusing reinstall when receipt already exists.
- Hardened remove safety with path-bound checks and clearer failures.
- Improved doctor clarity with per-check details plus summary counts.
- Improved CLI help text and README quickstart to explicitly use repo-local registry path.

### Deferred
- Remote registries and publishing workflows.
- Dependency resolution/lockfile support.
- Signature verification and expanded trust policies.
