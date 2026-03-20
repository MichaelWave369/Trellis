# Trellis Package Author Guide (v1.0.0-rc1)

This guide covers the maintained author workflow without hidden steps.

## Author workflow

```bash
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" init

./target/debug/trellis scaffold my-tool --kind binary --out /tmp
./target/debug/trellis validate /tmp/my-tool/my-tool.trellis.yaml
./target/debug/trellis inspect /tmp/my-tool/my-tool.trellis.yaml
./target/debug/trellis --home "$TRELLIS_HOME" install --from /tmp/my-tool/my-tool.trellis.yaml
./target/debug/trellis readiness /tmp/my-tool/my-tool.trellis.yaml
```

## Trust/provenance honesty

- `checksum_sha256` is verified during install when present.
- signature fields are recorded and structurally assessed (`present/missing/malformed/unsupported`).
- Trellis does not claim distributed cryptographic signature verification in rc1.

## Dependencies

- declare dependencies in spec metadata.
- indexed installs (`trellis install <pkg>`) resolve dependencies deterministically.
- local path installs (`trellis install --from`) install only the target spec; they do not expand dependency graph from index.

## Pre-submission checklist

- spec validates (`trellis validate`)
- metadata is inspectable and intentional (`trellis inspect`)
- local install succeeds (`trellis install --from`)
- readiness output is acceptable (`trellis readiness`)
- exposed binaries run as described
