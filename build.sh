#!/usr/bin/env bash
# Build every plugin under plugins/ into dist/<id>/ (plugin.wasm + plugin.json).
# The folder name is the plugin id and the Cargo package name.
# Needs: rust with the wasm32-unknown-unknown target, and wasm-tools
# (https://github.com/bytecodealliance/wasm-tools).
set -euo pipefail
cd "$(dirname "$0")"

build_one() {
  local id="$1"
  local wasm_name="${id//-/_}"
  cargo build --release --lib --target wasm32-unknown-unknown -p "$id"
  local out="dist/$id"
  rm -rf "$out"
  mkdir -p "$out"
  wasm-tools component new \
    "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/${wasm_name}.wasm" \
    -o "$out/plugin.wasm"
  cp "plugins/$id/plugin.json" "$out/"
  cp "plugins/$id/README.md" "plugins/$id/REFERENCE.md" LICENSE "$out/"
  # The pictures the README shows sit beside it.
  for png in "plugins/$id/"*.png; do
    [ -e "$png" ] && cp "$png" "$out/"
  done
  echo "built $out"
}

for dir in plugins/*/; do
  build_one "$(basename "$dir")"
done
