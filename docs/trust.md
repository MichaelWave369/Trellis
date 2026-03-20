# Trust, Shield, and Provenance (v0.4)

Trellis v0.4 deepens trust without pretending to provide guarantees it does not yet implement.

## What Trellis verifies today

- **Checksum verification**
  - `local_file` and `local_archive`: SHA-256 over file bytes.
  - `local_dir`: deterministic directory SHA-256 over sorted relative path + file content stream.
- **Platform constraints** are evaluated before install.
- **Conflict checks** block trust-critical overwrites:
  - existing package receipt collisions
  - exposed binary collisions
  - conflicting install target paths

## Trust states

Checksum state:
- `verified`
- `unverified`
- `unavailable`
- `mismatched`

Signature state:
- `present`
- `missing`
- `malformed`
- `unsupported`

Notes:
- `present` means metadata exists and is structurally parseable.
- Trellis does **not** cryptographically validate distributed signatures yet.

## Receipts as install ledger

Receipts record:
- package identity + kind + install time
- registry source and source metadata
- expected/actual checksum
- signature/provenance metadata and status
- platform evaluation
- declared dependencies
- installed files and exposed binaries
- post-install action declarations
- trust summary and warnings
- transaction identifier (groundwork for future rollback)

## `trellis doctor` trust model

Doctor reports pass/warn/fail with remediation hints for:
- registry config/index freshness and consistency
- malformed/duplicate registry data
- receipt parseability and conflict state
- trust state visibility and malformed signature metadata
- exposed binary integrity

## Deferred beyond v0.4

- distributed signature trust network
- key management and signature policy enforcement
- full transactional rollback engine
- remote publishing/mirror transport behavior


## Human-readable receipts (v0.5)

Use `trellis receipt <pkg>` to render the machine receipt into an operator-friendly summary for quick incident/debug review.
