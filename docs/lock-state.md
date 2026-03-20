# Lock State (v0.9)

Trellis writes profile-scoped lock state after installs.

- location: `$TRELLIS_HOME/locks/<profile>.lock.json`
- schema: package name/version/registry + generation timestamp

## Behavior

- written after successful install flow
- deterministic package ordering in lock output
- used by `trellis verify` as a consistency reference

If lock state is missing, Trellis still operates but verify reports only receipt-based checks.
