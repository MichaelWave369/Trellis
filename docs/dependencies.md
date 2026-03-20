# Dependency Resolution (v1.0.0-rc1)

`trellis install <pkg>` resolves dependency order from the active registry index.

## Behavior

- dependency traversal is deterministic (sorted)
- install order is printed before execution
- missing dependencies fail fast
- cycles fail with a clear error
- already-installed dependencies are skipped explicitly

## Scope boundaries

- this is not a full SAT/global solver
- advanced conflict policy remains post-1.0 work
- `install --from <spec>` does not perform index-driven dependency expansion
