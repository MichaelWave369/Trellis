# Trellis Roadmap

## v0.3 (current) — Vineyard Registry

- official default registry model (`vineyard-core`)
- versioned registry source config and deterministic local index materialization
- malformed-spec-tolerant indexing with actionable diagnostics
- registry-aware `update`, `search`, `info`, `install`, and `doctor`
- trust/provenance surfaced at registry and package index layers
- mirror/fallback configuration groundwork (local-first runtime)

## v0.4 (planned)

- dependency resolution strategy (still deterministic/local-first)
- lock state model for reproducible install plans
- fallback/mirror runtime behavior beyond metadata placeholders
- improved local integrity enforcement policies

## Later

- scoped remote sync/publish workflows
- transactional rollback primitives
- broader ecosystem interoperability tooling
