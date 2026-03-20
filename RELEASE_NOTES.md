# Trellis v1.0.0-rc1 Release Notes

Release date: 2026-03-20

## Release candidate intent

This release candidate is a hardening pass, not a feature-wave expansion. The focus is command coherence, workflow reliability, and trust-language honesty.

## What shipped in rc1

- CLI contract cleanup and consistency pass
- profile-aware verify/repair lock evaluation
- clearer dependency/install messaging for indexed vs local-path installs
- consolidated docs that match runtime behavior
- release-candidate positioning and scope boundaries

## Evaluation workflows

### 1) New user path

```bash
cargo build
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" search cli
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" install overstrings-cli
"$TRELLIS_HOME/bin/overstrings" normalize "Hello Trellis"
./target/debug/trellis --home "$TRELLIS_HOME" receipt overstrings-cli
./target/debug/trellis --home "$TRELLIS_HOME" doctor
```

### 2) Author path

```bash
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" init
./target/debug/trellis scaffold my-tool --kind binary --out /tmp
./target/debug/trellis validate /tmp/my-tool/my-tool.trellis.yaml
./target/debug/trellis inspect /tmp/my-tool/my-tool.trellis.yaml
./target/debug/trellis --home "$TRELLIS_HOME" install --from /tmp/my-tool/my-tool.trellis.yaml
./target/debug/trellis readiness /tmp/my-tool/my-tool.trellis.yaml
```

### 3) Verify/repair path

```bash
export TRELLIS_HOME="$(mktemp -d)"
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" seed
./target/debug/trellis --home "$TRELLIS_HOME" --registry-root "$(pwd)/packages" --profile default install overstrings-cli
./target/debug/trellis --home "$TRELLIS_HOME" --profile default verify
rm "$TRELLIS_HOME/bin/overstrings"
./target/debug/trellis --home "$TRELLIS_HOME" --profile default repair
./target/debug/trellis --home "$TRELLIS_HOME" --profile default verify
```

## Explicit non-goals in rc1

- GUI/dashboard features
- marketplace/community systems
- blockchain/token mechanics
- major speculative architecture additions
