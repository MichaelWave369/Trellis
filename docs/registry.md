# Registry Model (v1.0.0-rc1)

Trellis uses a local materialized index to keep command behavior deterministic and inspectable.

## Local registry state

- source config: `$TRELLIS_HOME/registry/sources.json`
- materialized index: `$TRELLIS_HOME/registry/index.json`
- cache directory: `$TRELLIS_HOME/registry/cache/`

## Update behavior

`trellis update`:

1. reads enabled registry sources
2. discovers `*.trellis.yaml` specs
3. validates and indexes valid package metadata
4. records malformed entries in `skipped`
5. writes deterministic `index.json`

## Index consumers

- `search` queries indexed package metadata
- `info` resolves package metadata through index
- `install <pkg>` resolves package + dependencies through index
- `doctor` checks index readability and malformed/duplicate conditions

## Scope boundaries

- mirror/fallback fields may appear as metadata but runtime mirror transport is not implemented in rc1
- no hosted publish service is implied by this repo
