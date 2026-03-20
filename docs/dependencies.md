# Dependency Resolution (v0.9)

Trellis now resolves declared dependencies during `trellis install <pkg>`.

## Current model

- dependency graph is resolved from indexed package metadata
- deterministic traversal order (sorted dependencies)
- install order is printed before execution
- missing dependencies fail fast
- simple cycle detection is enforced

## Explicit limitations

- this is not a full SAT/global solver
- version conflict handling is intentionally limited
- advanced alternative resolution strategies are deferred
