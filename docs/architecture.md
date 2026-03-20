# Trellis v0.1 Architecture

Trellis is a single-binary Rust CLI with a filesystem-first control plane.

## Modules

- `cli`: argument parsing and command dispatch
- `core`: path resolution, state init, install/remove flows, receipts
- `registry`: package spec scanning and local index sync
- `spec`: YAML schema and validation
- `doctor`: environment checks
- `trust`: checksum helpers

## State model

Trellis home defaults to XDG data dir when available, otherwise OS-appropriate fallback.

Subdirectories:

- `cache/`
- `cellar/`
- `receipts/`
- `registry/`
- `bin/`

## Deterministic install path

`cellar/<name>/<version>/...`

Install receipts are JSON files at `receipts/<name>.json` and include provenance plus exposed binaries.

## Registry model

v0.1 supports local filesystem scanning of `*.trellis.yaml` specs. `update` creates a materialized index used by search and doctor.
