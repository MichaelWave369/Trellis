# Trellis

Trellis is a local-first, infrastructure-grade package manager focused on deterministic state, explicit provenance, and trustworthy package workflows.

## Status

**v0.3 — Vineyard Registry**

Trellis now includes a credible registry model with an official default registry (`vineyard-core`), materialized local indexes, registry health checks, and registry-aware UX for `update`, `search`, `info`, `install`, and `doctor`.

## Scope (v0.3)

- official registry concept: `vineyard-core`
- versioned registry source config (`$TRELLIS_HOME/registry/sources.json`)
- deterministic registry index materialization (`$TRELLIS_HOME/registry/index.json`)
- malformed-spec tolerant indexing (skip with diagnostics)
- registry-level provenance + package-level integrity/provenance surfacing
- local/offline-first operation via local registry sources
- mirror/fallback config shape groundwork (no remote transport yet)

## Explicitly deferred beyond v0.3

- full dependency solver
- lockfiles
- rollback engine
- GUI/dashboard
- remote registry publishing service
- marketplace/social discovery features
- token/blockchain mechanics

## Commands

- `trellis init`
- `trellis update`
- `trellis search <query>`
- `trellis info <pkg-or-spec-path>`
- `trellis validate <pkg-or-spec-path>`
- `trellis inspect <pkg-or-spec-path>`
- `trellis install <pkg>`
- `trellis install --from <spec-path>`
- `trellis list`
- `trellis remove <pkg>`
- `trellis doctor`

## Official local packages

- `vineyard-core`
- `overstrings-cli`
- `tiekat-pulse`

## Quick v0.3 demo

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" init
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" update
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" search vineyard
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" info vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" doctor
cat "$TRELLIS_HOME/registry/sources.json"
cat "$TRELLIS_HOME/registry/index.json"
```

## Registry docs

- `docs/registry.md`
- `docs/package-spec.md`
- `docs/roadmap.md`

## License

AGPL-3.0-only.
