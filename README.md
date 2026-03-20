# Trellis

Trellis is a local-first package manager prototype designed for trustworthy, deterministic package workflows.

## Why Trellis exists

Modern package tooling is powerful, but often over-coupled to remote systems and opaque state. Trellis starts from infrastructure basics:

- local-first operations
- explicit install state
- human-readable metadata
- trust and provenance groundwork

## How Trellis differs from Homebrew

- **Scope**: v0.1 is intentionally narrow and filesystem-centric.
- **Model**: package specs are explicit YAML (`*.trellis.yaml`) and registry indexing is transparent.
- **Trust posture**: provenance fields and optional checksums are first-class from day one.

## Current status

**v0.1 bootstrap prototype**. The core flows work end-to-end against a local registry fixture.

## Launch scope

Implemented commands:

- `trellis init`
- `trellis update`
- `trellis search <query>`
- `trellis info <pkg>`
- `trellis install <pkg>`
- `trellis list`
- `trellis remove <pkg>`
- `trellis doctor`

## Non-goals for v0.1

- no blockchain
- no social layer
- no GUI/dashboard
- no remote registry publishing yet
- no "replace every package manager" claims
- no overbuilt mythology

## Quickstart (repo-local demo)

Run from the repository root:

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" init
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" update
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" search vineyard
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" info vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" list
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" doctor
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" remove vineyard-core
```

## Package spec example

```yaml
name: vineyard-core
version: 0.1.0
description: Core utilities for the Trellis reference registry.
homepage: https://example.org/vineyard-core
source:
  type: file
  path: payload
install:
  strategy: copy
  entries:
    - bin
bin:
  vineyard-core: bin/vineyard-core
dependencies: []
provenance:
  publisher: Trellis Maintainers
  license: MIT
  registry: vineyard-core
health:
  notes: Baseline fixture package for end-to-end testing.
```

## Local registry model

v0.1 uses a filesystem-driven registry rooted at `packages/`. `trellis update` scans specs and writes a local index into `TRELLIS_HOME/registry/index.json`.

## Trust and provenance philosophy

Trellis favors explicit package metadata over implicit trust:

- provenance fields (`publisher`, `license`, `registry`) are required
- checksums are supported for file-based sources
- install receipts are written for every install and drive list/remove behavior

## Roadmap

See `docs/roadmap.md` for phased milestones toward richer registries, signatures, lock state, and resolver improvements.
