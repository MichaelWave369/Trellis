# Lock State (v1.0.0-rc1)

Trellis writes profile-scoped lock artifacts after successful installs.

- path: `$TRELLIS_HOME/locks/<profile>.lock.json`
- schema: package name/version/registry + generation timestamp

## Behavior

- package entries are sorted deterministically
- duplicates are deduplicated before write
- `verify --profile <name>` uses that profile lock when present
- if lock state is absent, verify falls back to receipt/bin consistency checks
