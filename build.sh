#!/usr/bin/env bash
# Build every plugin under plugins/ into dist/<id>/ (plugin.wasm + plugin.json).
# The folder name is the plugin id and the Cargo package name.
# Needs: rust with the wasm32-unknown-unknown target, and wasm-tools
# (https://github.com/bytecodealliance/wasm-tools).
set -euo pipefail
cd "$(dirname "$0")"

# Catalog choices and reference tables come from the compiled descriptors.
cargo run --quiet --locked -p catalog-tool -- --check
# Command icons come from tools/icons/draw.py, and every entry names one.
python3 tools/icons/draw.py --check

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
  cp "plugins/$id/README.md" "plugins/$id/REFERENCE.md" LICENSE "$out/"
  mkdir -p "$out/docs/sources"
  cp docs/catalog-contract.md docs/parts-library.md docs/bearing-coverage.md "$out/docs/"
  cp docs/sources/*.md docs/sources/*.json "$out/docs/sources/"
  # Keep installed guides self-contained while repository links remain navigable.
  python3 - "$out" <<'PY'
from pathlib import Path
import json, re, shutil, sys
out = Path(sys.argv[1])
example_links = re.compile(r"(!?\[[^\n\]]*\]\()\.\./\.\./(examples/[^\s)]+)(\))")
repository = "https://github.com/TechSquidTV/SolidCraft-Plugins"

def package_example_link(match):
    prefix, target, suffix = match.groups()
    path, anchor, fragment = target.partition("#")
    source = Path(path)
    if not source.exists():
        raise SystemExit(f"{out.name}: missing example link target {source}")
    if prefix.startswith("!") and source.suffix == ".png":
        dest = out / source
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, dest)
    else:
        view = "tree" if source.is_dir() else "blob"
        target = f"{repository}/{view}/main/{source.as_posix()}{anchor}{fragment}"
    return prefix + target + suffix

# The host limits manifest bytes, not the readable source's indentation.
source = json.loads((Path("plugins") / out.name / "plugin.json").read_text())
manifest = json.dumps(source, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
if len(manifest) > 16 * 1024:
    raise SystemExit(f"{out.name}: compact manifest exceeds the host's 16 KiB limit")
(out / "plugin.json").write_bytes(manifest)
for name in ("README.md", "REFERENCE.md"):
    path = out / name
    text = path.read_text().replace("../../docs/", "docs/").replace("../../LICENSE", "LICENSE")
    text = text.replace("../../DEVELOPMENT.md#components", "docs/parts-library.md")
    path.write_text(example_links.sub(package_example_link, text))
PY
  # The command icons plugin.json names with x-solidcraft-icon.
  cp -R "plugins/$id/icons" "$out/icons"
  echo "built $out"
}

for dir in plugins/*/; do
  build_one "$(basename "$dir")"
done
