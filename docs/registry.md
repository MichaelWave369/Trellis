# Registry (v0.1)

Trellis v0.1 supports a single local registry concept anchored by the `packages/` directory.

## Flow

1. `trellis update` scans for `*.trellis.yaml` files.
2. Valid specs are serialized into `registry/index.json` in Trellis home.
3. `search` and `doctor` consume this index.
4. `install` resolves exact package names by scanning registry specs.

## Extensibility points

Future versions can add:

- multiple registries/taps
- remote sync transports
- signed indexes
- publication workflows
