# Trellis

Trellis is a local-first, infrastructure-grade package manager focused on deterministic state, explicit provenance, and trustworthy package workflows.

## Status

**v0.9 — Advanced Resolution and Ecosystem Growth**

Trellis now adds a disciplined first dependency resolver, profile-scoped lock state, and usable verify/repair commands to move toward durable ecosystem infrastructure.

## Quick infrastructure demo

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed

# dependency-aware install + lock write
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" --profile default install overstrings-cli
cat "$TRELLIS_HOME/locks/default.lock.json"

# verify / repair surface
./target/debug/trellis --home "$TRELLIS_HOME" verify
./target/debug/trellis --home "$TRELLIS_HOME" repair
```

## v0.9 highlights

- deterministic direct-dependency resolution with cycle/missing-dependency failure modes
- profile-scoped lock state (`locks/<profile>.lock.json`)
- simple environment profile model (`--profile`)
- verify/repair commands for receipt/install/bin drift handling
- explicit trust-policy documentation for recorded vs enforced behavior

## Native package catalog

| Package | Role | Primary value |
|---|---|---|
| `overstrings-cli` | Flagship utility | text normalization/formatting commands |
| `vineyard-core` | Ecosystem substrate | platform/path/operator baseline commands |
| `tiekat-pulse` | Diagnostics tool | runtime snapshots and process pulse checks |

## Explicitly deferred beyond v0.9

- full global dependency solving
- lockfile policy orchestration beyond current deterministic lock artifacts
- full transactional rollback execution
- remote publishing service and mirror transport runtime
- GUI/dashboard, package marketplace/community features, blockchain/token mechanics

## Commands

- `trellis init`
- `trellis seed`
- `trellis bootstrap`
- `trellis scaffold <package-name> [--kind binary|source] [--out <path>]`
- `trellis readiness <spec-or-package>`
- `trellis update`
- `trellis search <query>`
- `trellis info <pkg-or-spec-path>`
- `trellis validate <pkg-or-spec-path>`
- `trellis inspect <pkg-or-spec-path>`
- `trellis install <pkg>`
- `trellis install --from <spec-path>`
- `trellis verify`
- `trellis repair`
- `trellis receipt <installed-pkg>`
- `trellis list`
- `trellis remove <pkg>`
- `trellis doctor`

## Docs

- `docs/dependencies.md`
- `docs/lock-state.md`
- `docs/profiles.md`
- `docs/repair.md`
- `docs/trust-policy.md`
- `docs/authoring.md`
- `docs/submission.md`
- `docs/onboarding.md`
- `docs/packages.md`
- `docs/registry.md`
- `docs/trust.md`
- `docs/package-spec.md`
- `docs/roadmap.md`

## License

AGPL-3.0-only.
