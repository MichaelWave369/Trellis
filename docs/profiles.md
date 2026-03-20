# Environment Profiles (v0.9)

Trellis now supports a simple profile concept via `--profile`.

Default profiles:
- `default`
- `dev`
- `minimal`
- `diagnostics`

Current usage in v0.9:
- lock state is written per profile (`<profile>.lock.json`)
- install/verify flows can be segmented by profile identity

This is intentionally lightweight and not a full config-management system.
