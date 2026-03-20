# Onboarding and First-Time Workflow (v1.0.0-rc1)

`trellis seed` (alias: `trellis bootstrap`) is the recommended first-run path.

## What seed does

1. Initializes local Trellis state directories/config.
2. Refreshes registry index from enabled sources.
3. Runs core health/trust checks.
4. Prints featured packages and next-step commands.
5. Shows key local paths and PATH guidance.

The flow is local-first and safe to re-run.

## End-to-end new-user demo

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

## Scripted bootstrap helper

A local helper exists at `scripts/trellis-bootstrap.sh` for development/demo setup.
