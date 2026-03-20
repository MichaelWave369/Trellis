# Trellis Author Guide (v0.2)

## Goal

A package author can add a package to the local Trellis registry without changing Trellis core code.

## Basic flow

1. Create package folder under `packages/<name>/`.
2. Add payload files under `payload/`.
3. Create `<name>.trellis.yaml`.
4. Run `trellis validate <path-to-spec>`.
5. Run `trellis inspect <path-to-spec>`.
6. Run `trellis install --from <path-to-spec>`.

## Minimal binary example

```yaml
schema_version: "0.2"
name: my-tool
version: 0.1.0
description: My local tool
homepage: https://example.org/my-tool
kind: binary
source:
  type: local_dir
  path: payload
install:
  strategy: copy
  entries: [bin]
bin:
  my-tool: bin/my-tool
dependencies: []
provenance:
  publisher: Example Org
  license: MIT
  registry: vineyard-core
platform:
  os: [linux, macos]
  arch: [x86_64, aarch64]
```

## Minimal source-kind example

Use `kind: source` when package payload is source-oriented but still installable through local copy strategy.

## Trust and provenance in v0.2

- Add `checksum_sha256` when the source is a file.
- Use `signature` as a local placeholder metadata field.
- Verify metadata through `trellis inspect` and receipt output.
