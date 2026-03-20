# Trellis

Trellis is a local-first, infrastructure-grade package manager focused on deterministic state, explicit provenance, and trustworthy package workflows.

## Status

**v0.7 — Seed Installer Experience**

Trellis now includes a guided first-run onboarding flow (`trellis seed` / `trellis bootstrap`) that gets a new user from zero to first useful package in minutes.

## Fastest path quickstart

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
"$TRELLIS_HOME/bin/vineyard-core" status
```

## Local development quickstart

```bash
scripts/trellis-bootstrap.sh
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" search cli
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install overstrings-cli
"$TRELLIS_HOME/bin/overstrings" normalize "Hello Trellis"
```

## What just happened?

`trellis seed` performs:

1. state init/config materialization
2. registry refresh/index materialization
3. doctor confidence checkpoint
4. featured package guidance and first-package recommendation
5. path/model output so users know where state was written

## Native package catalog

| Package | Role | Primary value |
|---|---|---|
| `overstrings-cli` | Flagship utility | text normalization/formatting commands |
| `vineyard-core` | Ecosystem substrate | platform/path/operator baseline commands |
| `tiekat-pulse` | Diagnostics tool | runtime snapshots and process pulse checks |

## First package walkthrough (all three)

```bash
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install overstrings-cli
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install tiekat-pulse

"$TRELLIS_HOME/bin/vineyard-core" paths
"$TRELLIS_HOME/bin/overstrings" stats "hello world"
"$TRELLIS_HOME/bin/tiekat-pulse" snapshot
```

## Explicitly deferred beyond v0.7

- full dependency resolver
- lockfiles
- transactional rollback execution
- remote publishing and mirror transport runtime
- GUI/dashboard, package marketplace/community features, blockchain/token mechanics

## Commands

- `trellis init`
- `trellis seed`
- `trellis bootstrap`
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

## Docs

- `docs/onboarding.md`
- `docs/packages.md`
- `docs/registry.md`
- `docs/trust.md`
- `docs/package-spec.md`
- `docs/roadmap.md`

## License

AGPL-3.0-only.
