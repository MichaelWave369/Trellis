# Trellis Package Spec (v0.1)

File suffix: `*.trellis.yaml`

## Required fields

- `name`
- `version`
- `description`
- `homepage`
- `source`
  - `type`: must be `file` in v0.1
  - `path`: local path relative to spec file directory
  - `checksum_sha256`: optional
- `install`
  - `strategy`: must be `copy`
  - `entries`: list of files/directories copied into cellar
- `provenance`
  - `publisher`
  - `license`
  - `registry`

## Optional fields

- `bin`: command-name to relative path map within install root
- `dependencies`: list of package names (declared, not yet resolved recursively)
- `health.notes`

## Design goals

- readable and diff-friendly
- deterministic installs
- explicit trust metadata
