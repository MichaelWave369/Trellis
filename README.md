# Trellis

Trellis is a local-first package manager focused on deterministic state, explicit metadata, and trustworthy package workflows.

## Status

**v0.2 Formula System + Native Packages**

Trellis now supports package authoring workflows so package authors can add packages without editing Trellis core code.

## Scope (v0.2)

- local filesystem registry (`packages/`)
- package spec validation and inspection
- install by package name or `--from <spec-path>`
- deterministic installs/removals with receipts
- package kind and platform constraint checks
- dependency declaration parsing (non-recursive)

## Non-goals (v0.2)

- no remote registry publishing/sync
- no lockfiles
- no full dependency solver
- no GUI/dashboard
- no blockchain/social features

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

## Official local packages (v0.2)

- `vineyard-core`
- `overstrings-cli`
- `tiekat-pulse`

## 2-minute demo

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" init
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" update
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" validate overstrings-cli
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" inspect overstrings-cli
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install overstrings-cli
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" list
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" doctor
```

## Authoring guide

See `docs/authoring.md` and `docs/package-spec.md` for the full v0.2 authoring model.

## Trust/provenance model

Trellis keeps trust explicit and local:

- provenance fields are required
- checksum fields are supported and validated
- receipt records include provenance and integrity metadata
- signatures are metadata placeholders in v0.2 (no remote signing infra)

## License

AGPL-3.0-only.
