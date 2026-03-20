# Trust and Provenance Model (v1.0.0-rc1)

Trellis reports trust/provenance state explicitly and avoids overclaiming.

## Verified now

- checksum verification when `checksum_sha256` is declared
- platform constraint matching before install
- collision checks for receipts, install targets, and exposed binaries

## Recorded now

- signature metadata state (`present`, `missing`, `malformed`, `unsupported`)
- provenance metadata (`publisher`, `license`, declared registry)
- receipt ledger details for installed files/binaries and trust summary

## Deferred / unsupported in rc1

- distributed cryptographic signature trust network
- key management and signature policy enforcement
- transactional rollback guarantees

## Operational commands

- `trellis receipt <pkg>` renders trust/provenance and install ledger details
- `trellis verify` checks receipt/install/bin consistency (+ lock consistency when lock exists)
- `trellis doctor` reports pass/warn/fail health status with remediation hints
