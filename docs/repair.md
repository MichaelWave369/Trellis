# Verify and Repair (v1.0.0-rc1)

Trellis provides two maintenance commands:

- `trellis verify`
- `trellis repair`

## verify

Checks:
- receipt file readability
- install-root existence
- exposed binary existence and target existence
- lock vs receipt consistency for the selected profile (if lock exists)

## repair

Attempts focused local remediation:
- recreates missing exposed binaries when recorded targets still exist

Then reruns verify checks and reports whether state is consistent.

## Scope boundaries

- no transactional rollback guarantees
- no automatic payload re-fetch/reinstall in repair
