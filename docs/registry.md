# Vineyard Registry Model (v0.3)

Trellis v0.3 introduces a first-class registry layer while keeping operation local/offline-first.

## Official registry

The default official registry source is `vineyard-core`.

- source content root: local `packages/` tree (or `--registry-root` override)
- optional registry metadata file: `<registry-root>/registry.yaml`
- optional `featured_packages` list for curated catalog presentation
- package specs: `*.trellis.yaml`

## Registry source config

Trellis materializes source configuration at:

- `$TRELLIS_HOME/registry/sources.json`

Format (versioned, inspectable):

- `schema_version`
- `sources[]`
  - `name`
  - `kind` (`local_path` in v0.3)
  - `enabled`
  - `official`
  - `location`
  - `mirrors[]` (placeholder shape for future fallback logic)

## Materialized index/cache state

Trellis separates source content from local state:

- source registry: raw package specs + `registry.yaml`
- materialized index: `$TRELLIS_HOME/registry/index.json`
- cache directory reserved: `$TRELLIS_HOME/registry/cache/`

`index.json` includes:

- schema + generation timestamp
- per-registry summary (counts, revision, mirrors)
- flattened package entries for command resolution
- featured package markers for curated catalog UX
- skipped/malformed spec diagnostics

Index generation is deterministic:

- spec discovery sorted by path
- package output sorted by registry/name/version

## Update/refresh behavior

`trellis update`:

1. reads/creates `sources.json`
2. scans each enabled registry source
3. validates specs and indexes valid entries
4. records malformed specs in `skipped`
5. rewrites `index.json`
6. prints registry summary (counts, skipped, timestamp/revision)

## Registry-aware commands

- `trellis search <query>` reads materialized index and prints package name/version/kind/description/registry
- `trellis info <pkg>` resolves through index and prints provenance, platform, dependencies, integrity fields
- `trellis install <pkg>` resolves through index and reports chosen registry entry
- `trellis doctor` validates config/index freshness/integrity and malformed entry state

## Trust and provenance posture (v0.3)

Trellis exposes trust metadata but keeps scope disciplined:

- registry-level provenance via `registry.yaml`
- package-level provenance from spec (`publisher`, `license`, `registry`)
- integrity visibility (`checksum_sha256`, `signature`) in info/index/install context
- doctor checks for stale, malformed, duplicate, and inconsistent local index state

No remote signature trust network is introduced in v0.3.

## Fallback/mirror groundwork

Mirrors are modeled in config as metadata only. Runtime fallback transport is intentionally deferred. Future behavior (v0.4+) will:

- support ordered mirror resolution
- support local cache hydration and fallback reads
- keep deterministic update semantics and inspectable state transitions
