# Onboarding and Seed Flow (v0.7)

Trellis v0.7 introduces a safe, inspectable onboarding command:

- `trellis seed`
- alias: `trellis bootstrap`

## What `seed` does

1. Ensures Trellis state directories/config exist.
2. Refreshes registry metadata/index.
3. Runs health/trust checks (doctor core checks).
4. Shows featured package hints.
5. Recommends first install (`vineyard-core`).
6. Prints key paths (`home`, `registry/index`, `bin`, `receipts`) and PATH guidance.

The command is safe to re-run and intentionally local-first.

## Fastest path

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install vineyard-core
```

## Bootstrap script (repo-local)

A minimal helper script is provided at:

- `scripts/trellis-bootstrap.sh`

Usage:

```bash
scripts/trellis-bootstrap.sh
```

This script is local/dev oriented and does **not** rely on opaque remote execution.

## Eventual one-line installer shape (future concept)

An eventual hosted installer may look like:

```bash
curl -fsSL https://example.org/trellis/install.sh | sh
```

This is **not** implemented in this repository today; treat it as future distribution shape only.
