# Trellis Package Author Guide (v0.8)

This guide is for external contributors authoring Trellis-native packages.

## Author workflow (recommended)

1. Scaffold a package:
   - `trellis scaffold my-tool`
   - or `trellis scaffold my-tool --kind source`
2. Edit package metadata and payload files.
3. Validate spec:
   - `trellis validate packages/my-tool/my-tool.trellis.yaml`
4. Inspect rendered metadata:
   - `trellis inspect packages/my-tool/my-tool.trellis.yaml`
5. Test local install:
   - `trellis install --from packages/my-tool/my-tool.trellis.yaml`
6. Check submission readiness:
   - `trellis readiness packages/my-tool/my-tool.trellis.yaml`

## Spec anatomy

Key fields in `<name>.trellis.yaml`:

- `name`, `version`, `description`, `homepage`
- `kind` (`binary` or `source`)
- `source` (`type`, `path`, optional `checksum_sha256`, optional `signature`)
- `install.entries`
- `bin` mappings
- `dependencies` (declaration only; no full solver yet)
- `provenance` (`publisher`, `license`, `registry`)
- optional `platform` constraints
- optional `post_install` with allowlisted policy

## Naming and versioning

- Names: lowercase + digits + hyphen (`2-64` chars)
- Versions: semver-like (`1.2.3`, `1.2.3-alpha`)

## Source types

- `local_dir` (recommended for most authoring)
- `local_file`
- `local_archive`

## Trust/provenance expectations

- Fill all provenance fields with real values.
- Prefer declaring `checksum_sha256` when stable payload hashes are available.
- Signature metadata is currently structural (`sig:<value>`) and not full cryptographic network trust.

## Platform constraints

Use `platform.os` and `platform.arch` to avoid installs where the package cannot run.

## Post-install policy

If `post_install` is used, policy must be allowlisted and command must satisfy current validator constraints.

## Local testing checklist

- `validate` passes
- `inspect` shows expected metadata
- `install --from` succeeds
- exposed binaries run
- receipt output reflects trust/provenance correctly

## Common mistakes

- invalid package name casing
- missing `bin` mapping for `kind: binary`
- absolute or parent-relative `source.path`
- placeholder provenance fields left as `TODO`
- platform constraints that do not match intended target systems
