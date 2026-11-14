#!/usr/bin/env bash
# Build each plugin into dist/<id>/ (plugin.wasm + plugin.json).
# Needs: rust with the wasm32-unknown-unknown target, and wasm-tools
# (https://github.com/bytecodealliance/wasm-tools).
set -euo pipefail
cd "$(dirname "$0")"

build_one() {
  local id="$1"
  local wasm_name="$2"
  cargo build --release --lib --target wasm32-unknown-unknown -p "$id"
  local out="dist/$id"
  rm -rf "$out"
  mkdir -p "$out"
  wasm-tools component new \
    "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/${wasm_name}.wasm" \
    -o "$out/plugin.wasm"
  cp "plugins/$id/plugin.json" "$out/"
  cp "plugins/$id/README.md" LICENSE "$out/"
  echo "built $out"
}

build_one techsquid-screws techsquid_screws
build_one techsquid-inserts techsquid_inserts
build_one techsquid-nuts techsquid_nuts
