# Changelog

## v0.2.0-alpha - 2026-03-20

### Added
- v0.2 package spec fields: schema version, package kind, local source shape, platform constraints, post-install policy, signature placeholder.
- Authoring commands: `trellis validate` and `trellis inspect`.
- Local author install workflow: `trellis install --from <spec-path>`.
- Additional official local registry fixtures: `overstrings-cli`, `tiekat-pulse`.
- Author-focused docs (`docs/authoring.md`) and updated spec/roadmap references.
- Integration tests for validate/inspect/local-install/platform mismatch and invalid naming rules.

### Changed
- Install planning now enforces platform constraints and reports declared dependencies as non-resolved.
- Receipts include package kind, checksum verification status, signature metadata, and declared dependencies.

### Deferred (v0.3)
- automatic dependency resolution
- lockfiles
- rollback engine
- remote sync/publish
