#!/usr/bin/env sh
set -eu

# Local-first Trellis bootstrap helper for development and onboarding.
# This script does not fetch remote code or perform opaque system modifications.

TRELLIS_BIN="${TRELLIS_BIN:-./target/debug/trellis}"
TRELLIS_HOME="${TRELLIS_HOME:-${XDG_DATA_HOME:-$HOME/.local/share}/trellis}"
REGISTRY_ROOT="${TRELLIS_REGISTRY_ROOT:-$(pwd)/packages}"

if [ ! -x "$TRELLIS_BIN" ]; then
  echo "Building Trellis binary at $TRELLIS_BIN"
  cargo build
fi

echo "Running Trellis seed onboarding"
"$TRELLIS_BIN" --home "$TRELLIS_HOME" --registry-root "$REGISTRY_ROOT" seed

echo "Bootstrap complete. Next suggested command:"
echo "  $TRELLIS_BIN --home \"$TRELLIS_HOME\" --registry-root \"$REGISTRY_ROOT\" install vineyard-core"
