# Trellis v0.1.0-alpha Release Notes

Release date: 2026-03-20

## Highlights
- First working Trellis prototype focused on reliable local-first package operations.
- Deterministic install/remove behavior with explicit receipt tracking.
- Filesystem-native registry scanning and local index refresh.
- Baseline trust/provenance groundwork (checksums + provenance fields).
- CI checks for format, lint, and test execution.

## Included commands
- `trellis init`
- `trellis update`
- `trellis search <query>`
- `trellis info <pkg>`
- `trellis install <pkg>`
- `trellis list`
- `trellis remove <pkg>`
- `trellis doctor`

## What is intentionally not in this alpha
- No remote registry transport or publishing.
- No dependency resolver or lockfile.
- No GUI/dashboard.
- No blockchain/social layer.

## 2-minute demo
From repo root:

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" init
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" update
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" list
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" doctor
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" remove vineyard-core
```
