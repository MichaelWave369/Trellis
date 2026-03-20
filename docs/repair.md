# Verify and Repair (v0.9)

Trellis exposes first usable integrity maintenance commands:

- `trellis verify`
- `trellis repair`

## verify

Checks receipt/install/bin consistency and lock-vs-receipt alignment.

## repair

Attempts practical local repair:
- recreates missing exposed binaries when targets still exist

`repair` is intentionally scoped and does not claim full transactional rollback.
