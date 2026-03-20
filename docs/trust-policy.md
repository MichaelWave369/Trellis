# Trust Policy Boundaries (v1.0.0-rc1)

Trellis separates three concepts and labels them explicitly:

1. **Verified now**
   - checksum verification when expected checksum is provided
   - platform compatibility check during install

2. **Recorded now**
   - signature metadata state and note
   - provenance metadata in specs/receipts

3. **Deferred / unsupported**
   - distributed signature trust network
   - keyring/policy enforcement framework
   - transactional rollback guarantees

Use `info`, `receipt`, `doctor`, and `verify` together for a full trust-state picture.
