# Registry (v0.2)

Trellis v0.2 uses a local filesystem registry rooted at `packages/`.

## Flow

1. `trellis update` scans for `*.trellis.yaml` files.
2. Valid specs are materialized into `TRELLIS_HOME/registry/index.json`.
3. `search`, `info`, `validate`, and `inspect` can resolve by package name.
4. `install` can resolve by package name or directly from `--from <spec-path>`.

## Official local registry fixtures

- `vineyard-core`
- `overstrings-cli`
- `tiekat-pulse`

## Deferred

- no remote sync/publish in v0.2
- no tap cloning/network transport in v0.2
