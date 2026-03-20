# Trellis

Trellis is a local-first, infrastructure-grade package manager focused on deterministic state, explicit provenance, and trustworthy package workflows.

## Status

**v0.4 — Trust, Shield, and Provenance**

Trellis now emphasizes install trust clarity: explicit checksum states, signature metadata status scaffolding, upgraded receipts as a trust ledger, safer conflict detection, and stronger doctor integrity checks.

## Scope (v0.4)

- registry-driven package resolution (from v0.3)
- explicit checksum verification paths (`local_file`, `local_archive`, `local_dir`)
- trust state model (`verified`, `unverified`, `unavailable`, `mismatched`)
- signature metadata state model (`present`, `missing`, `malformed`, `unsupported`)
- receipt model upgraded for provenance, trust summary, and rollback groundwork
- install integrity report and explicit conflict detection
- doctor pass/warn/fail with trust/integrity remediation hints

## Explicitly deferred beyond v0.4

- full dependency solver
- lockfiles
- full rollback/transaction engine
- distributed signature network and key infrastructure
- remote registry publishing and mirror transport behavior
- GUI/dashboard, marketplace/social features, blockchain/token mechanics

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

## Quick v0.4 demo

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" init
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" update
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" info vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
cat "$TRELLIS_HOME/receipts/vineyard-core.json"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" doctor
```

## Docs

- `docs/registry.md`
- `docs/trust.md`
- `docs/package-spec.md`
- `docs/roadmap.md`

## License

AGPL-3.0-only.
