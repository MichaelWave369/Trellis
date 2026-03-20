# Environment Profiles (v1.0.0-rc1)

Profiles are lightweight execution namespaces selected with `--profile`.

Default profiles created by `trellis init`:
- `default`
- `dev`
- `minimal`
- `diagnostics`

## Current effect

- install writes lock state to `locks/<profile>.lock.json`
- verify/repair read lock state for the selected profile when available

Profiles do not currently represent a full policy/config system.
