# Trellis v0.2.0-alpha Release Notes

Release date: 2026-03-20

## Highlights

- Trellis now supports package authoring workflows without changing core code.
- `validate` and `inspect` make package spec development deterministic and auditable.
- `install --from <spec-path>` lets authors test local packages directly.
- Official local registry now includes multiple native package fixtures.

## What shipped

- Expanded v0.2 spec model (kind/source/platform/provenance/post-install policy)
- `trellis validate <pkg-or-spec-path>`
- `trellis inspect <pkg-or-spec-path>`
- `trellis install --from <spec-path>`
- New package fixtures: `overstrings-cli`, `tiekat-pulse`

## Explicitly deferred

- remote registry publishing/sync
- full dependency solver
- lockfiles
- rollback engine
- GUI/social/blockchain features
