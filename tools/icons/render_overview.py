#!/usr/bin/env python3
"""Regenerate the README icon overview from the shipped SVGs, using Chrome or Chromium.

    python3 tools/icons/render_overview.py --host-version 0.45.0

The release version comes from Cargo.toml. The host version is a caption supplied after
compatibility testing; this script does not run the host tests. Set CHROME if needed.
"""

import argparse
import html
import json
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from render_html import render_html

# Editorial order and captions for the approved overview; icon paths come from manifests.
CARDS = {
    "techsquid-screws/make-screw": ("Screws", "Standard & custom fasteners"),
    "techsquid-nuts/make-nut": ("Hex nuts", "Plain & nylon-insert"),
    "techsquid-inserts/make-insert": ("Heat-set inserts", "Brass inserts & mounting holes"),
    "techsquid-extrusions/make-extrusion": ("Extrusions", "T-slot & V-slot profiles"),
    "techsquid-bearings/make-bearing": ("Bearings", "Radial bearings"),
    "techsquid-bearings/make-csk": ("CSK clutches", "Presets and custom clutches"),
    "techsquid-bearings/make-bearing-by-dimensions": ("Bearing by dimensions", "Match a radial bearing"),
    "techsquid-bearings/make-csk-by-dimensions": ("CSK clutch by dimensions", "Match a CSK preset"),
    "techsquid-extrusions/make-t-nut": ("Slot nuts", "Pre- & post-assembly"),
    "techsquid-v-wheels/make-v-wheel": ("V-wheels", "Bare wheels & core assemblies"),
}


def count_word(count):
    words = ("Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Eleven", "Twelve")
    return words[count] if count < len(words) else str(count)


def overview(host_version):
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    icons = {}
    plugins = set()
    for manifest in sorted((ROOT / "plugins").glob("*/plugin.json")):
        plugin = json.loads(manifest.read_text(encoding="utf-8"))
        plugins.add(plugin["id"])
        for entry in plugin["runtime"]["entries"]:
            icons[f"{plugin['id']}/{entry['name']}"] = manifest.parent / entry["input_schema"]["x-solidcraft-icon"]
    if set(icons) != set(CARDS):
        sys.exit("Update CARDS in tools/icons/render_overview.py for changed entries: " + ", ".join(sorted(set(icons) ^ set(CARDS))))

    width = 1600
    rows = (len(CARDS) + 3) // 4
    footer = 264 + rows * 324
    height = footer + 88
    version = html.escape(version)
    count = count_word(len(icons))
    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        f'<title>TechSquid SolidCraft Plugins {version} — all {count.lower()} command icons</title>',
        f'<rect width="{width}" height="{height}" fill="#f4f7fb"/>',
        '<g font-family="Arial, Helvetica, sans-serif" fill="#26384e">',
        '<text x="84" y="67" font-size="18" font-weight="700" letter-spacing="4" fill="#328bd3">TECHSQUID</text>',
        '<text x="80" y="137" font-size="62" font-weight="700" letter-spacing="-2">SolidCraft Plugins</text>',
        f'<text x="84" y="184" font-size="24" fill="#66768b">{count} commands. {count_word(len(plugins))} plugins. Native, editable parts.</text>',
        '<rect x="1360" y="53" width="156" height="48" rx="24" fill="#e3effa"/>',
        f'<text x="1438" y="85" text-anchor="middle" font-size="23" font-weight="700" fill="#236ead">v{version}</text>',
    ]
    for index, (key, (title, caption)) in enumerate(CARDS.items()):
        x = 84 + (index % 4) * 366
        y = 234 + (index // 4) * 324
        center = x + 170
        svg.append(f'<rect x="{x}" y="{y}" width="340" height="300" rx="22" fill="#fff" stroke="#e2e8ef"/>')
        source = icons[key].read_text(encoding="utf-8")
        svg.append(source.replace('<svg ', f'<svg x="{center - 85}" y="{y + 22}" width="170" height="170" ', 1))
        svg.append(f'<text x="{center}" y="{y + 235}" text-anchor="middle" font-size="25" font-weight="700">{html.escape(title)}</text>')
        svg.append(f'<text x="{center}" y="{y + 266}" text-anchor="middle" font-size="17" fill="#738196">{html.escape(caption)}</text>')
    svg.extend([
        f'<path d="M84 {footer} H1516" stroke="#dbe3ed"/>',
        f'<circle cx="90" cy="{footer + 41}" r="5" fill="#328bd3"/>',
        f'<text x="108" y="{footer + 48}" font-size="18" fill="#52677f">Verified on SolidCraft {html.escape(host_version)}</text>',
        f'<text x="1516" y="{footer + 48}" font-size="16" text-anchor="end" letter-spacing="1.5" fill="#738196">TECHSQUID SOLIDCRAFT PLUGINS</text>',
        '</g></svg>',
    ])
    page = (
        '<!doctype html><meta charset="utf-8">'
        f'<style>html,body{{margin:0;width:{width}px;height:{height}px;overflow:hidden}}svg{{display:block}}</style>'
        + "\n".join(svg)
    )
    return page, (width, height)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--host-version", required=True, help="SolidCraft version already verified by compatibility tests (caption only)")
    parser.add_argument("--out", type=Path, default=ROOT / "docs/images/plugin-icons.png", help="output PNG (default: docs/images/plugin-icons.png)")
    args = parser.parse_args()
    subprocess.run([sys.executable, str(Path(__file__).with_name("draw.py")), "--check"], check=True)
    page, size = overview(args.host_version)
    with tempfile.TemporaryDirectory(prefix="plugin-icon-overview-") as directory:
        layout = Path(directory) / "overview.html"
        layout.write_text(page, encoding="utf-8")
        render_html(layout, args.out, size)
    print(f"Rendered {len(CARDS)} command icons to {args.out} ({size[0]} x {size[1]}).")


if __name__ == "__main__":
    main()
