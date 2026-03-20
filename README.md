# Trellis

Trellis is a local-first, infrastructure-grade package manager focused on deterministic state, explicit provenance, and trustworthy package workflows.

## Status

**v0.5 — UX and Identity Layer**

Trellis now adds a cohesive CLI output system, clearer progress/status reporting, refined package discovery/install summaries, and human-readable receipt rendering via `trellis receipt <pkg>`.

## CLI UX philosophy

- clear over clever
- trust signals over decoration
- stable, terminal-friendly structure
- practical remediation guidance

## Scope (v0.5)

- consistent section headers and status markers across commands
- lightweight step/status reporting for update/install/doctor flows
- polished search/list/info layouts for faster scanning
- install resolution summary before apply
- human-readable receipt rendering (`trellis receipt <pkg>`)
- refined health/trust indicator presentation

## Health and trust indicators

Trellis uses concise status labels:

- command statuses: `[✓]`, `[!]`, `[x]`, `[i]`, `[>]`
- doctor check states: `PASS`, `WARN`, `FAIL`
- trust states in info/receipt/install output (checksum + signature)

## Explicitly deferred beyond v0.5

- full dependency solver
- lockfiles
- rollback engine
- remote registry publishing and mirror transport runtime
- GUI/dashboard, marketplace/community features, blockchain/token mechanics

## Commands

- `trellis init`
- `trellis update`
- `trellis search <query>`
- `trellis info <pkg-or-spec-path>`
- `trellis validate <pkg-or-spec-path>`
- `trellis inspect <pkg-or-spec-path>`
- `trellis install <pkg>`
- `trellis install --from <spec-path>`
- `trellis receipt <installed-pkg>`
- `trellis list`
- `trellis remove <pkg>`
- `trellis doctor`

## Quick v0.5 demo

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" init
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" update
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" search vineyard
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" receipt vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" doctor
```

## Docs

- `docs/registry.md`
- `docs/trust.md`
- `docs/package-spec.md`
- `docs/roadmap.md`

## License

AGPL-3.0-only.
