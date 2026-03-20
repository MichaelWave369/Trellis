# Trellis

Trellis is a local-first, infrastructure-grade package manager focused on deterministic state, explicit provenance, and trustworthy package workflows.

## Status

**v1.0.0-rc1 — Release Candidate Hardening**

Trellis has completed its original feature map and is now focused on coherence, reliability, and honest trust language for serious evaluation.

## What Trellis is

- a CLI-first package manager with a human-readable local state model
- deterministic registry indexing and dependency ordering for practical workflows
- explicit trust/provenance recording with clear boundaries on what is enforced
- a small but real native package catalog for end-to-end evaluation

## What Trellis is not

- not a GUI/dashboard product
- not a marketplace/community layer
- not a blockchain/token system
- not a mythology-first runtime experience

## What v1.0.0-rc1 includes

- coherent command surface for onboarding, authoring, maintenance, and repair workflows
- profile-scoped lock artifacts (`locks/<profile>.lock.json`)
- deterministic dependency traversal during indexed installs
- verify/repair flows for receipt/bin drift detection and remediation
- consistent docs for trust policy, registry behavior, and release boundaries

## What is explicitly not in 1.0

- full SAT/global dependency solving
- transactional rollback guarantees
- hosted publish service or remote execution model
- mirror-transport failover runtime

## Quickstart (new user path)

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"

./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" search cli
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install overstrings-cli
"$TRELLIS_HOME/bin/overstrings" normalize "Hello Trellis"
./target/debug/trellis --home "$TRELLIS_HOME" receipt overstrings-cli
./target/debug/trellis --home "$TRELLIS_HOME" doctor
```

## Author workflow demo

```bash
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" init

./target/debug/trellis scaffold demo-tool --kind binary --out /tmp
./target/debug/trellis validate /tmp/demo-tool/demo-tool.trellis.yaml
./target/debug/trellis inspect /tmp/demo-tool/demo-tool.trellis.yaml
./target/debug/trellis --home "$TRELLIS_HOME" install --from /tmp/demo-tool/demo-tool.trellis.yaml
./target/debug/trellis readiness /tmp/demo-tool/demo-tool.trellis.yaml
```

## Verify / repair workflow demo

```bash
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" --profile default install overstrings-cli

./target/debug/trellis --home "$TRELLIS_HOME" --profile default verify
rm "$TRELLIS_HOME/bin/overstrings"
./target/debug/trellis --home "$TRELLIS_HOME" --profile default verify
./target/debug/trellis --home "$TRELLIS_HOME" --profile default repair
./target/debug/trellis --home "$TRELLIS_HOME" --profile default verify
```

## Native package catalog

| Package | Role | Primary value |
|---|---|---|
| `overstrings-cli` | Flagship utility | text normalization/formatting commands |
| `vineyard-core` | Ecosystem substrate | platform/path/operator baseline commands |
| `tiekat-pulse` | Diagnostics tool | runtime snapshots and process pulse checks |

## Command surface

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

Global flags:
- `--home <path>`
- `--registry-root <path>`
- `--profile <name>`

## Docs

- `docs/onboarding.md`
- `docs/authoring.md`
- `docs/registry.md`
- `docs/trust.md`
- `docs/trust-policy.md`
- `docs/dependencies.md`
- `docs/lock-state.md`
- `docs/profiles.md`
- `docs/repair.md`
- `docs/packages.md`
- `docs/submission.md`
- `docs/package-spec.md`
- `docs/roadmap.md`
- `CHANGELOG.md`
- `RELEASE_NOTES.md`

## License

AGPL-3.0-only.
