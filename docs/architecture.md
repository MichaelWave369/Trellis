# Trellis v0.2 Architecture

Trellis is a single Rust CLI binary with explicit filesystem state.

## Core modules

- `cli`: parsing and command dispatch
- `spec`: package schema + validation rules
- `registry`: local filesystem index scanning/materialization
- `core`: install/remove/state/receipts
- `trust`: checksum helpers
- `doctor`: environment checks

## Authoring model

v0.2 adds:

- `validate` for schema/rules checking
- `inspect` for package metadata/trust summary
- `install --from <spec-path>` for local author workflow

## Deterministic state

Trellis home contains:

- `cache/`
- `cellar/`
- `receipts/`
- `registry/`
- `bin/`

Each install writes a receipt with provenance and integrity fields.
