#!/usr/bin/env python3
"""Draw each command's icon into plugins/<id>/icons/<entry>.svg (stdlib only).

    python3 tools/icons/draw.py           write the icons
    python3 tools/icons/draw.py --check   fail when a saved icon differs from what this draws
    python3 tools/icons/draw.py --sheet FILE.html
                                          every icon, and every entry of CANDIDATES, in the light
                                          and dark themes at 128, 48, 32, 20 and 16 px

How to draw a new icon, and why three candidate views come first: skills/techsquid-icons/SKILL.md.

SolidCraft draws a plugin's command icon from the package, named by `x-solidcraft-icon` at the
top of the entry's `input_schema`. It accepts only its own icon SVG subset: a 64 by 64 view box,
absolute M L C Q A Z paths, and the icon palette's role colours, so the icon follows the theme.
Source: SolidCraft docs/plugins.md, "Icons and pictures", and crates/look/icons/STYLE_GUIDE.md
(isometric solids, top face lightest, screen-left face mid, screen-right face darkest, one blue
cue). Both modes also check that every manifest entry names its icon and that each icon keeps to
the subset, because SolidCraft leaves out an icon it refuses without failing the plugin.
"""
import argparse
import json
import math
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

# Role colours: the `source` values of SolidCraft crates/look/icons/design/palette.toml.
INK = "#34465b"
P1, P2, P3 = "#e2e8ef", "#b8c3cf", "#8e9faf"  # passive: top, screen-left, screen-right
A1, A2, A3 = "#b9ddf8", "#328bd3", "#236ead"  # active blue: the functional feature
O1, O2, O3 = "#f1d2af", "#c06f00", "#a45000"  # component orange: brass

K = 1 / math.sqrt(3)  # a horizontal circle's screen ellipse: ry = rx * K
COS30 = math.sqrt(3) / 2
OUTLINE, DETAIL = 2.55, 2.0
PALETTE = {INK, P1, P2, P3, A1, A2, A3, O1, O2, O3}
# Each role's dark-theme value in the same palette.toml, for the --sheet preview only.
DARK = {INK: "#1f2021", P1: "#8da3c3", P2: "#6d81a2", P3: "#546782", A1: "#4ea1dc", A2: "#3287d1", A3: "#2976b8", O1: "#ba8f6a", O2: "#b86f04", O3: "#95682d"}
MAX_BYTES = 64 * 1024  # SolidCraft's limit for one package image


def n(v):
    text = f"{v:.2f}".rstrip("0").rstrip(".")
    return "0" if text == "-0" else text


def d(*parts):
    return " ".join(p if isinstance(p, str) else n(p) for p in parts)


def path(data, fill=None, stroke=None, width=OUTLINE):
    out = f'  <path d="{data}" fill="{fill or "none"}"'
    if stroke:
        out += f' stroke="{stroke}" stroke-width="{n(width)}" stroke-linejoin="round" stroke-linecap="round"'
    return out + "/>"


def ellipse(cx, cy, rx, ry=None):
    ry = rx * K if ry is None else ry
    return d("M", cx - rx, cy, "A", rx, ry, "0 1 0", cx + rx, cy, "A", rx, ry, "0 1 0", cx - rx, cy, "Z")


def polygon(points):
    return d("M", *[v for p in points for v in (*p, "L")][:-1], "Z")


def cylinder(cx, top, bottom, rx, side, shade, face=None):
    """An upright cylinder: its side with the darker screen-right strip, then its top face."""
    ry = rx * K
    left, right, strip = cx - rx, cx + rx, cx + 0.35 * rx
    body = d("M", left, top, "L", left, bottom, "A", rx, ry, "0 0 0", right, bottom, "L", right, top, "Z")
    drop = ry * math.sqrt(1 - 0.35**2)
    out = [
        path(body, side),
        path(d("M", strip, top, "L", strip, bottom + drop, "A", rx, ry, "0 0 0", right, bottom, "L", right, top, "Z"), shade),
        path(body, stroke=INK),
    ]
    if face:
        out.append(path(ellipse(cx, top, rx), face, INK))
    return out


def bore(cx, cy, rx):
    """A threaded hole in a top face: dark blue wall, paler floor, as SolidCraft's hole icon."""
    ry = rx * K
    inner_rx, inner_ry = rx * 0.72, ry * 0.6
    return [
        path(ellipse(cx, cy, rx), A3),
        path(ellipse(cx, cy + ry - inner_ry - 0.4, inner_rx, inner_ry), A2),
        path(ellipse(cx, cy, rx), stroke=INK, width=DETAIL),
    ]


def prism(outline, z0, z1, scale, cx, cy, width=DETAIL):
    """A plan outline (counter-clockwise x, y) raised from z0 to z1: visible walls, then the top."""

    def at(x, y, z):
        return (cx + (x - y) * COS30 * scale, cy + (x + y) * 0.5 * scale - z * scale)

    walls = []
    for (x0, y0), (x1, y1) in zip(outline, outline[1:] + outline[:1]):
        nx, ny = y1 - y0, x0 - x1  # outward normal of a counter-clockwise edge
        if nx + ny > 1e-9:  # faces the viewer
            quad = polygon([at(x0, y0, z1), at(x1, y1, z1), at(x1, y1, z0), at(x0, y0, z0)])
            walls.append((x0 + x1 + y0 + y1, path(quad, P3 if nx > ny else P2, INK, width)))
    out = [wall for _, wall in sorted(walls, key=lambda w: w[0])]
    out.append(path(polygon([at(x, y, z1) for x, y in outline]), P1, INK, width))
    return out


def box(x, y, z0, z1, scale, cx, cy):
    return prism([(x, -y), (x, y), (-x, y), (-x, -y)], z0, z1, scale, cx, cy)


def screw():
    """A socket head cap screw, head up: blue hex socket, two thread turns on the shank."""
    out = cylinder(32, 27.5, 54, 7, P2, P3)
    for y in (42.5, 50):
        out.append(path(d("M", 25, y, "Q", 32, y + 5, 39, y - 5), stroke=INK))
    out += cylinder(32, 15, 27.5, 15, P2, P3, P1)
    hexagon = [(32 + 8 * math.cos(math.radians(a)), 15 + 8 * K * math.sin(math.radians(a))) for a in range(0, 360, 60)]
    out.append(path(polygon(hexagon), A3, INK, DETAIL))
    return out


def nut():
    """A hex nut lying flat: three visible flats and a blue threaded bore."""
    cx, cy, r, height = 32, 24, 25, 15
    corner = [(cx + r * math.cos(math.radians(a)), cy + r * K * math.sin(math.radians(a))) for a in range(0, 360, 60)]
    out = []
    for i, tone in ((0, P3), (1, P2), (2, P2)):
        (ax, ay), (bx, by) = corner[i], corner[i + 1]
        out.append(path(polygon([(ax, ay), (bx, by), (bx, by + height), (ax, ay + height)]), tone, INK))
    out.append(path(polygon(corner), P1, INK))
    return out + bore(cx, cy, 11.5)


def insert():
    """A brass heat-set insert: two bands knurled in opposite hands, a groove, a threaded bore."""

    def knurl(top, bottom, rx, lean):
        ticks = []
        for a in range(34, 150, 19):
            upper, lower = math.radians(a - lean / 2), math.radians(a + lean / 2)
            ticks.append(d("M", 32 + rx * math.cos(upper), top + rx * K * math.sin(upper) + 2.6, "L", 32 + rx * math.cos(lower), bottom + rx * K * math.sin(lower) - 2.6))
        return path(" ".join(ticks), stroke=INK, width=1.7)

    out = cylinder(32, 36, 49.5, 17, O2, O3, O1) + [knurl(36, 49.5, 17, -24)]
    out += cylinder(32, 29.5, 36, 13, O2, O3)
    out += cylinder(32, 15, 29.5, 17, O2, O3, O1) + [knurl(15, 29.5, 17, 24)]
    return out + bore(32, 15, 9)


RING, SLOT, HUB = 27, 22.5, 12  # screen radii: outer ring, raceway opening, inner ring


def raceway(cy):
    """Behind the rolling elements: the outer ring's wall and the dark raceway between the rings."""
    return cylinder(32, cy, cy + 14, RING, P2, P3) + [path(ellipse(32, cy, SLOT), INK)]


def rings(cy, bore_rx):
    """Over the rolling elements: both rings' top faces, so their lips hide what sits below them."""
    lip = ellipse(32, cy, RING) + " " + ellipse(32, cy, SLOT)
    return [
        f'{path(lip, P1)[:-2]} fill-rule="evenodd"/>',
        path(ellipse(32, cy, RING), stroke=INK),
        path(ellipse(32, cy, SLOT), stroke=INK, width=DETAIL),
        path(ellipse(32, cy, HUB), P1, INK, DETAIL),
        path(ellipse(32, cy, bore_rx), P3, INK, DETAIL),
    ]


def bearing():
    """A radial ball bearing lying flat: blue balls seated in the raceway, below the ring lips."""
    cy, pitch, sunk, count = 24, (SLOT + HUB) / 2, 2.4, 12
    r = pitch * math.sin(math.pi / count) - 0.15  # neighbours all but touch round the pitch circle
    out = raceway(cy)
    for a in sorted(range(0, 360, 360 // count), key=lambda a: math.sin(math.radians(a + 15))):  # back to front
        t = math.radians(a + 15)
        x, y = 32 + pitch * math.cos(t), cy + pitch * K * math.sin(t) + sunk
        out.append(path(ellipse(x, y, r, r), A2, INK, 1.3))
        out.append(path(ellipse(x - 1.3, y - 1.5, 1.3, 1.3), A1))
    return out + rings(cy, 7)


def csk():
    """A CSK one-way clutch: tilted blue sprags in the raceway and a keyway in the bore."""
    cy = 24
    out = raceway(cy)
    sprags = []
    for a in range(0, 360, 45):
        inner, outer = math.radians(a + 13), math.radians(a + 32)
        sprags.append(d("M", 32 + 14.5 * math.cos(inner), cy + 14.5 * K * math.sin(inner) + 1.6, "L", 32 + 20 * math.cos(outer), cy + 20 * K * math.sin(outer) + 1.6))
    out.append(path(" ".join(sprags), stroke=A2, width=3.6))
    out += rings(cy, 7.5)
    top, bottom = cy + 7.5 * K - 2.2, cy + 10.6 * K
    out.append(path(polygon([(29.4, top), (34.6, top), (34.6, bottom), (29.4, bottom)]), P3))
    out.append(path(d("M", 29.4, top + 1.6, "L", 29.4, bottom, "L", 34.6, bottom, "L", 34.6, top + 1.6), stroke=INK, width=DETAIL))
    return out


def extrusion():
    """A length of 2020 T-slot rail, end up: the slot section on its end face and down each side.

    The section is the common 20 mm profile, in millimetres from the centre: a 6.2 slot opening
    through a 1.8 lip, an 11 wide pocket behind it, tapering at 45 degrees to the 8.4 core.
    """
    side = [(10, -10), (10, -3.1), (8.2, -3.1), (8.2, -5.5), (7, -5.5), (4.2, -2.7), (4.2, 2.7), (7, 5.5), (8.2, 5.5), (8.2, 3.1), (10, 3.1)]
    outline = []
    for turn in range(4):  # the same side four times, a quarter turn on each
        for x, y in side:
            for _ in range(turn):
                x, y = -y, x
            outline.append((x, y))
    scale, length, cy = 1.6, 11, 21.8
    out = prism(outline, 0, length, scale, 32, cy + length * scale, 1.5)
    return out + [path(ellipse(32, cy, 4.1), A3, INK, 1.5)]


def t_nut():
    """A slot nut: the wide flange that catches the slot, the neck above it, a threaded hole."""
    scale, cy = 1.8, 37.85
    out = box(11, 6.5, 0, 4, scale, 32, cy) + box(11, 3.5, 4, 8, scale, 32, cy)
    return out + bore(32, cy - 8 * scale, 5.1)


def hull(points):
    """The convex outline round screen points, as a closed path."""
    points = sorted(set(points))

    def half(run):
        kept = []
        for p in run:
            while len(kept) > 1 and (kept[-1][0] - kept[-2][0]) * (p[1] - kept[-2][1]) <= (kept[-1][1] - kept[-2][1]) * (p[0] - kept[-2][0]):
                kept.pop()
            kept.append(p)
        return kept[:-1]

    return polygon(half(points) + half(points[::-1]))


def ring(axis, radius, axial, scale, cx=32, cy=32, count=48):
    """Screen points of a circle about `axis` (x, y, z; z is up), `axial` along it from the centre."""
    length = math.sqrt(sum(c * c for c in axis))
    a = [c / length for c in axis]
    u = [a[1] * 0.8 - a[2] * 0.5, a[2] * 0.3 - a[0] * 0.8, a[0] * 0.5 - a[1] * 0.3]  # any direction across the axis
    length = math.sqrt(sum(c * c for c in u))
    u = [c / length for c in u]
    v = [a[1] * u[2] - a[2] * u[1], a[2] * u[0] - a[0] * u[2], a[0] * u[1] - a[1] * u[0]]
    points = []
    for i in range(count):
        t = 2 * math.pi * i / count
        x, y, z = (a[j] * axial + radius * (u[j] * math.cos(t) + v[j] * math.sin(t)) for j in range(3))
        points.append((cx + (x - y) * COS30 * scale, cy + (x + y) * 0.5 * scale - z * scale))
    return points


def revolved(axis, rim, tones, scale, cx=32, cy=32):
    """A turned part about any axis, from its rim as (radius, axial) pairs: one band between each
    pair, far side first, in `tones`. Also returns `face(radius)`, a circle on the near end face."""
    if sum(axis) < 0:  # the viewer looks from +x +y +z: keep the near end last
        axis, rim = [-c for c in axis], [(r, -s) for r, s in reversed(rim)]
    bands = [path(hull(ring(axis, r0, s0, scale, cx, cy) + ring(axis, r1, s1, scale, cx, cy)), tone, INK) for (r0, s0), (r1, s1), tone in zip(rim, rim[1:], tones)]
    return bands, lambda radius: polygon(ring(axis, radius, rim[-1][1], scale, cx, cy))


def v_wheel(axis=(0, 1, 0), scale=1.79):
    """A Solid V wheel standing on its tread, three-quarter view: flat tread, a chamfer each side, its bearing.

    The rim is the wheel's measured section, [radius, axial] in millimetres, from `SOLID` in
    crates/wheels/src/lib.rs; the 16 mm pocket holds the 625 bearing with its 5 mm bore.
    """
    rim = [(9.8, -5.1), (11.95, -2.95), (11.95, 2.95), (9.8, 5.1)]
    out, face = revolved(axis, rim, (P3, P2, P1), scale)
    return out + [path(face(9.8), P1, INK), path(face(8), A2, INK, 1.8), path(face(4.3), A1, INK, 1.5), path(face(2.5), INK)]


ICONS = {
    "techsquid-screws/make-screw": ("Screw", screw),
    "techsquid-nuts/make-nut": ("Nut", nut),
    "techsquid-inserts/make-insert": ("Heat-set insert", insert),
    "techsquid-bearings/make-bearing": ("Bearing", bearing),
    "techsquid-bearings/make-csk": ("CSK clutch", csk),
    "techsquid-bearings/make-bearing-by-dimensions": ("Bearing by dimensions", bearing),
    "techsquid-bearings/make-csk-by-dimensions": ("CSK clutch by dimensions", csk),
    "techsquid-extrusions/make-extrusion": ("Extrusion", extrusion),
    "techsquid-extrusions/make-t-nut": ("Slot nut", t_nut),
    "techsquid-v-wheels/make-v-wheel": ("V-wheel", v_wheel),
}


# Views being compared for one new icon: label -> a function returning its paths, as in ICONS.
# Draw three, render them with --sheet, ask which, then move the chosen one into ICONS and empty
# this again; --check refuses to pass while it holds anything.
CANDIDATES = {}


def document(title, paths):
    return "\n".join(['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">', f"  <title>{title}</title>", *paths, "</svg>"]) + "\n"


def sheet(target):
    """Every candidate and icon in both themes at the sizes SolidCraft draws them, as one page."""
    drawings = [(label, document(label, draw())) for label, draw in CANDIDATES.items()]
    drawings += [(key.split("/")[1], document(title, draw())) for key, (title, draw) in ICONS.items()]
    page = ["<meta charset=utf-8><style>body{margin:0;font:12px sans-serif}.t{padding:12px;display:flex;flex-wrap:wrap;gap:18px}.c{text-align:center}.s{display:flex;gap:8px;justify-content:center;align-items:center;margin:6px 0}</style>"]
    for background, ink, dark in (("#f6f7f9", "#262a32", False), ("#272b34", "#dee2e8", True)):
        page.append(f'<div class=t style="background:{background};color:{ink}">')
        for label, svg in drawings:
            if dark:
                svg = re.sub("|".join(DARK), lambda m: DARK[m.group()], svg)
            sizes = "".join(svg.replace("<svg ", f"<svg width={px} height={px} ", 1) for px in (48, 32, 20, 16))
            page.append(f'<div class=c>{svg.replace("<svg ", "<svg width=128 height=128 ", 1)}<div class=s>{sizes}</div>{label}</div>')
        page.append("</div>")
    Path(target).write_text("\n".join(page), encoding="utf-8")
    print(f"sheet: {len(CANDIDATES)} candidates and {len(ICONS)} icons in {target}")


def refused(svg):
    """Why SolidCraft would refuse this icon, or None: the rules of its lint_icons.py."""
    if len(svg.encode("utf-8")) > MAX_BYTES:
        return "larger than 64 KiB"
    root = ET.fromstring(svg)
    if root.get("viewBox") != "0 0 64 64" or root.get("width") or root.get("height"):
        return "the view box must be 0 0 64 64, with no width or height"
    for element in root.iter():
        tag = element.tag.split("}")[-1]
        if tag not in {"svg", "title", "path"}:
            return f"<{tag}> is not an element these icons use"
        if tag != "path":
            continue
        if not re.fullmatch(r"[MLCQAZ0-9 .\-]+", element.get("d", "")):
            return "path data may use only absolute M L C Q A Z"
        fill, stroke = element.get("fill"), element.get("stroke")
        if {fill, stroke} - {None, "none"} - PALETTE:
            return f"{fill} or {stroke} is not a palette colour"
        if stroke and not (element.get("stroke-width") and element.get("stroke-linejoin") == element.get("stroke-linecap") == "round"):
            return "a stroke needs a width and round joins and caps"
    return None


NUMBER = re.compile(r"-?\d+(?:\.\d+)?")


def same(saved, drawn):
    """Whether two icons are one drawing: the same markup, each number within a rounding step.
    Sines and cosines differ in their last bit between platforms, which can move a second decimal."""
    if NUMBER.sub("#", saved) != NUMBER.sub("#", drawn):
        return False
    return all(abs(float(a) - float(b)) < 0.011 for a, b in zip(NUMBER.findall(saved), NUMBER.findall(drawn)))


def main():
    parser = argparse.ArgumentParser(
        description="Draw each command's icon into plugins/<id>/icons/<entry>.svg. With no option, write them.",
        epilog="Examples:\n  python3 tools/icons/draw.py\n  python3 tools/icons/draw.py --check\n  python3 tools/icons/draw.py --sheet /tmp/sheet.html\n\nExits 1 with one line saying what to fix. Method: skills/techsquid-icons/SKILL.md",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="write nothing; fail when a saved icon differs, an entry has no icon, an icon breaks SolidCraft's SVG rules, or CANDIDATES is not empty")
    mode.add_argument("--sheet", metavar="FILE.html", help="write nothing but FILE.html: every CANDIDATES view and every icon, light and dark, at 128, 48, 32, 20 and 16 px")
    args = parser.parse_args()
    check = args.check
    for label, draw in CANDIDATES.items():
        if why := refused(document(label, draw())):
            sys.exit(f"candidate {label}: {why}")
    if args.sheet:
        if not args.sheet.endswith(".html"):
            sys.exit(f"--sheet writes an HTML page: give a path ending in .html, not {args.sheet}")
        return sheet(args.sheet)
    if check and CANDIDATES:
        sys.exit("CANDIDATES still holds views: move the chosen one into ICONS and empty it")
    stale = []
    named = set()
    for manifest in sorted(ROOT.glob("plugins/*/plugin.json")):
        for entry in json.loads(manifest.read_text(encoding="utf-8"))["runtime"]["entries"]:
            key = f"{manifest.parent.name}/{entry['name']}"
            named.add(key)
            if entry["input_schema"].get("x-solidcraft-icon") != f"icons/{entry['name']}.svg":
                sys.exit(f"{key}: input_schema needs \"x-solidcraft-icon\": \"icons/{entry['name']}.svg\"")
    if named != set(ICONS):
        sys.exit("entries and icons differ: " + ", ".join(sorted(named ^ set(ICONS))))
    for key, (title, draw) in ICONS.items():
        plugin, entry = key.split("/")
        target = ROOT / "plugins" / plugin / "icons" / f"{entry}.svg"
        svg = document(title, draw())
        if why := refused(svg):
            sys.exit(f"{key}: {why}")
        if check:
            if not target.exists() or not same(target.read_text(encoding="utf-8"), svg):
                stale.append(target.relative_to(ROOT).as_posix())
        else:
            target.parent.mkdir(exist_ok=True)
            target.write_text(svg, encoding="utf-8")
    if stale:
        sys.exit("icons differ from tools/icons/draw.py; run it to redraw: " + ", ".join(stale))
    print(f"icons: {len(ICONS)} {'match' if check else 'written'}")


if __name__ == "__main__":
    main()
