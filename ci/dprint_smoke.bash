#!/usr/bin/env bash
# Builds the dprint plugin for wasm and formats the smoke fixtures with the real dprint CLI,
# which checks the wasm exports and additive file matching that native tests can't reach.
# The TypeScript plugin claims card.tsx, so its expected output needs both plugins to run
set -euo pipefail

DPRINT_VERSION=0.63.0
# the checksum makes dprint reject a plugin that changed upstream
TYPESCRIPT_PLUGIN=https://plugins.dprint.dev/typescript-0.95.13.wasm@d353247b160c1e81eb043930de6f940adcd3d713651221d3a0284d4c30ea43c4
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SMOKE="$ROOT/rustywind-dprint/tests/smoke"

cargo build -p dprint-plugin-rustywind --release --target wasm32-unknown-unknown

# cargo puts the wasm under the workspace target dir, which CARGO_TARGET_DIR can move
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
PLUGIN="$TARGET_DIR/wasm32-unknown-unknown/release/dprint_plugin_rustywind.wasm"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cp -R "$SMOKE/input/." "$WORK"

cd "$WORK"
npx --yes "dprint@$DPRINT_VERSION" fmt --plugins "$TYPESCRIPT_PLUGIN" "$PLUGIN"
diff -r "$SMOKE/expected" "$WORK"

# a second pass must be a no-op, or dprint would keep reformatting the file
npx --yes "dprint@$DPRINT_VERSION" check --plugins "$TYPESCRIPT_PLUGIN" "$PLUGIN"
echo "dprint plugin smoke test passed"
