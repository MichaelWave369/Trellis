# Trellis Package Spec (v0.2)

File suffix: `*.trellis.yaml`

## Schema versioning

Set `schema_version: "0.2"` for all new specs. Trellis currently defaults missing schema version to `0.2` for local fixtures, but authors should always write it explicitly.

## Required fields

- `schema_version`
- `name` (lowercase, digits, hyphen, 2-64 chars)
- `version` (semver-like: `1.2.3` or `1.2.3-alpha`)
- `description`
- `homepage` (`http://` or `https://`)
- `kind` (`binary` or `source`)
- `source`
  - `type`: `local_file`, `local_dir`, or `local_archive`
  - `path`: relative path from spec directory
  - `checksum_sha256`: optional 64-char hex digest
  - `signature`: optional local placeholder string
- `install`
  - `strategy`: `copy`
  - `entries`: list of files/directories copied from source path
- `provenance`
  - `publisher`
  - `license`
  - `registry`

## Optional fields

- `bin`: command-to-relative-path map (required for `kind: binary`)
- `dependencies`: declared package names (non-recursive in v0.2)
- `platform`
  - `os`: subset of `linux`, `macos`, `windows`
  - `arch`: subset of `x86_64`, `aarch64`
- `post_install`
  - `policy`: must be `allowlisted`
  - `command`: allowlisted command only (`echo`, `true` in v0.2)
- `health.notes`

## Dependency behavior in v0.2

Dependencies are parsed and shown in `inspect`/`info` and during install planning. Trellis does **not** perform automatic recursive dependency resolution yet.
