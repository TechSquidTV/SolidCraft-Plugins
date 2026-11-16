#!/usr/bin/env bash
# Build every plugin under plugins/ into dist/<id>/ (plugin.wasm + plugin.json).
# The folder name is the plugin id and the Cargo package name.
# Needs: rust with the wasm32-unknown-unknown target, and wasm-tools
# (https://github.com/bytecodealliance/wasm-tools).
set -euo pipefail
cd "$(dirname "$0")"

# Catalog choices and query schemas come from the compiled descriptors.
cargo run --quiet --locked -p catalog-tool -- --check

build_one() {
  local id="$1"
  local wasm_name="${id//-/_}"
  cargo build --locked --release --lib --target wasm32-unknown-unknown -p "$id"
  local out="dist/$id"
  rm -rf "$out"
  mkdir -p "$out"
  wasm-tools component new \
    "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/${wasm_name}.wasm" \
    -o "$out/plugin.wasm"
  cp "plugins/$id/plugin.json" "$out/"
  cp "plugins/$id/README.md" "plugins/$id/REFERENCE.md" LICENSE "$out/"
  mkdir -p "$out/docs/sources"
  cp docs/catalog-contract.md docs/parts-library.md docs/bearing-coverage.md "$out/docs/"
  cp docs/sources/*.md docs/sources/*.json "$out/docs/sources/"
  # Keep installed guides self-contained while repository links remain navigable.
  python3 - "$out" <<'PY'
from pathlib import Path
import sys
for name in ("README.md", "REFERENCE.md"):
    path = Path(sys.argv[1]) / name
    text = path.read_text().replace("../../docs/", "docs/").replace("../../LICENSE", "LICENSE")
    text = text.replace("../../DEVELOPMENT.md#components", "docs/parts-library.md")
    path.write_text(text)
PY
  # The pictures the README shows sit beside it.
  for png in "plugins/$id/"*.png; do
    [ -e "$png" ] && cp "$png" "$out/"
  done
  echo "built $out"
}

for dir in plugins/*/; do
  build_one "$(basename "$dir")"
done
