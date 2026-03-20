# Trellis

Trellis is a local-first, infrastructure-grade package manager focused on deterministic state, explicit provenance, and trustworthy package workflows.

## Status

**v0.8 — Ecosystem Authoring**

Trellis now includes a contributor-ready author workflow so external developers can scaffold, validate, inspect, test-install, and prepare package submissions with low friction.

## Fastest author workflow

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed
./target/debug/trellis scaffold my-tool
./target/debug/trellis validate packages/my-tool/my-tool.trellis.yaml
./target/debug/trellis inspect packages/my-tool/my-tool.trellis.yaml
./target/debug/trellis --home "$TRELLIS_HOME" install --from packages/my-tool/my-tool.trellis.yaml
./target/debug/trellis readiness packages/my-tool/my-tool.trellis.yaml
```

## What changed in v0.8

- new package scaffolding command (`trellis scaffold <name>`)
- source/binary scaffold kind support (`--kind source|binary`)
- submission readiness helper (`trellis readiness <spec-or-package>`)
- expanded author and submission documentation
- maintained local-first install/validate/inspect/test loop

## Native package catalog

| Package | Role | Primary value |
|---|---|---|
| `overstrings-cli` | Flagship utility | text normalization/formatting commands |
| `vineyard-core` | Ecosystem substrate | platform/path/operator baseline commands |
| `tiekat-pulse` | Diagnostics tool | runtime snapshots and process pulse checks |

## Explicitly deferred beyond v0.8

- full dependency resolver
- lockfiles
- transactional rollback execution
- remote publishing service and mirror runtime transport
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
- `trellis receipt <installed-pkg>`
- `trellis list`
- `trellis remove <pkg>`
- `trellis doctor`

## Docs

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
