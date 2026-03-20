# Official Registry Submission Workflow (v0.8)

Trellis official package submission is repo-native and PR-based.

## Required package layout

- `packages/<name>/<name>.trellis.yaml`
- `packages/<name>/payload/...`

## Contributor checklist

Before opening a PR:

1. `trellis validate <spec-path>`
2. `trellis inspect <spec-path>`
3. `trellis install --from <spec-path>` (in temp home)
4. `trellis readiness <spec-path>`
5. confirm provenance fields are real and complete
6. include checksums/signature metadata where practical
7. include platform constraints intentionally
8. confirm dependency declarations are intentional and resolvable
9. if dependencies changed, include lock/verify evidence from local run

## PR expectations

A submission PR should include:

- package folder + payload + spec
- short package purpose statement
- local test commands run and output summary
- notes on platform scope and trust metadata

## Maintainer review criteria

Maintainers should verify:

- spec passes validation and inspect output is coherent
- metadata is non-placeholder and credible
- payload behavior is minimal but real
- trust fields are sensible for declared source type
- dependency declarations and platform rules are coherent
- package does not misrepresent capability or support matrix

## Acceptance boundaries

This workflow is intentionally simple:

- no hosted publish service
- no marketplace mechanics
- no remote execution requirements
