---
name: techsquid-icons
description: Use this skill when a TechSquid plugin command needs an icon or its icon looks wrong - a new dialog entry, a command still showing the generic plugin icon in SolidCraft's Plugins menu, a request to draw, redraw, restyle or review an icon or a dropdown option's picture, or feedback on how one looks ("the wheel doesn't look right", "it reads as a screw head", "make the balls bigger"), even when nobody says "icon" or "SVG". Also use it before touching tools/icons/draw.py or a plugin's icons folder. It gives the SVG rules SolidCraft enforces, the drawing method, and the required review where three candidate views are shown and the user picks one. Not for dialog fields or other plugin.json keys (techsquid-fields), nor for README pictures and example renders (techsquid-tests).
---

# Command icons

Each entry has one icon, `plugins/techsquid-<name>/icons/<entry>.svg`, named at the top of the entry's `input_schema` by `"x-solidcraft-icon": "icons/<entry>.svg"`. SolidCraft 0.44 and later draw it in the Plugins menu and the dialog header, at 16 to 20 px. Nobody writes the SVG by hand: `tools/icons/draw.py` draws all of them, and `./build.sh` runs its `--check` and copies `icons/` into the package.

```bash
python3 tools/icons/draw.py                    # write the icons
python3 tools/icons/draw.py --check            # what ./build.sh runs
python3 tools/icons/draw.py --sheet sheet.html # preview: candidates and icons, both themes, five sizes
```

Each mode exits non-zero with one line saying what to fix; `--help` lists them.

## The object is the icon

Draw the part itself as a small, real, solid object, seen from above in the app's isometric view. No arrows, badges, letters or abstract marks. Someone who has held the part should name it at 20 px.

Start from the truth, not from memory. Read the part's real section and proportions in its domain crate before drawing anything (`SOLID` in `crates/wheels/src/lib.rs`, the tables in `crates/bearings`, `crates/extrusions`, `crates/fasteners`), and put the numbers and where they came from in the drawing function's docstring. The first V-wheel icons were drawn from an idea of a V-wheel, with long bevels meeting at a point, and were rejected three times; the measured section has a flat tread 5.9 wide on a 10.2 wide wheel and a bearing that fills the hub.

## Checklist: a new or reworked icon

- [ ] **Read the real geometry** and write down the few proportions that make the part recognisable.
- [ ] **Write the drawing function with its view as a parameter** (an axis, or a scale and a turn), as `v_wheel(axis, scale)` does, so one function draws every candidate.
- [ ] **Draw three views and ask which. Always, even when one seems obvious.** Put three entries in `CANDIDATES` in `tools/icons/draw.py`, labelled A, B and C. Good defaults: lying flat seen from above, standing three-quarter, and a third that shows the feature the first two hide (nearer edge-on, end up, or cut away).
- [ ] **Render the sheet, read it, fix, repeat**: `python3 tools/icons/draw.py --sheet <scratch>/sheet.html`, then turn it into a picture you can read (on macOS `qlmanage -t -s 1500 -o <scratch> <scratch>/sheet.html`; elsewhere open the page in a browser and screenshot it). Go round until nothing is clipped, merged or out of order. Don't show the user a sheet you haven't looked at.
- [ ] **Show the user the sheet and ask which view**, in the form below. Wait for the answer; don't pick for them.
- [ ] **Move the chosen view into `ICONS`** as that function's default, and empty `CANDIDATES`. `--check` fails while it holds anything.
- [ ] **Name the icon in `plugin.json`** when the entry is new (`skills/techsquid-fields/SKILL.md`).
- [ ] **Check the small sizes in both themes** on the sheet: the part must still read at 20 and 16 px, light and dark.
- [ ] `python3 tools/icons/draw.py && ./build.sh`

While comparing, `CANDIDATES` holds one function drawn three ways:

```python
CANDIDATES = {
    "A lying flat": lambda: v_wheel((0, 0, 1), 1.7),
    "B three-quarter": lambda: v_wheel((0, 1, 0)),
    "C nearer edge-on": lambda: v_wheel((math.cos(math.radians(112)), math.sin(math.radians(112)), 0)),
}
```

Ask with the picture and this shape, then stop:

```markdown
Three views of the <part>, drawn from <where the proportions came from>:

- **A, <view>:** shows <feature>; hides or weakens <feature>.
- **B, <view>:** ...
- **C, <view>:** ...

I'd pick <letter> because <one reason>. Which one?
```

Feedback on one detail of an accepted icon (bigger balls, slanted knurl) needs no new candidates: change it, render the sheet, show the result. Feedback that the part "doesn't look right" does: go back to the real geometry and draw three views again.

## What SolidCraft accepts

It reads the SVG with its own small parser and leaves out an icon it refuses, logging why; the command then shows the generic plugin icon and nothing fails. `draw.py` checks the same rules in both modes (`refused`), so use its helpers rather than writing markup.

- `viewBox="0 0 64 64"`, no `width` or `height`. Keep the drawing inside about 4 to 60, centred on 32, 32.
- Paths only, with absolute `M L C Q A Z`. No transforms, styles, classes, text, images or `<use>`. `fill-rule="evenodd"` works, for a ring with a hole.
- Every colour is one of the icon palette's role colours, so the icon follows the theme. Strokes need a width and round joins and caps. At most 64 KiB a file and 64 images a package.

| Constant | Role | Use it for |
|---|---|---|
| `INK` | outline | every outline; a deep hole or a dark raceway as a fill |
| `P1` `P2` `P3` | passive grey | the part: top face, screen-left face, screen-right face |
| `A1` `A2` `A3` | active blue | the one functional feature: thread, socket, balls, bearing |
| `O1` `O2` `O3` | brass | a brass part, light to dark like `P1` to `P3` |

A new colour is added to the constants, `PALETTE` and `DARK` together, from the `source` and `dark` values of `palette.toml` in the icon design folder of SolidCraft's `look` crate.

## Drawing rules

- **Solids, painted back to front.** Every shape is filled and outlined in `INK`; what is nearer is drawn later and hides what is behind. No see-through edges or dashed hidden lines.
- **Tone by facing.** Top faces `P1`, screen-left faces `P2`, screen-right faces `P3`. Don't swap them to flatter one drawing. On a turned part, the band facing the viewer is lighter than the one facing away.
- **One blue cue.** Blue marks what the part does (the threaded bore, the hex socket, the rolling elements, the bearing in a wheel). Everything else is grey or brass.
- **What sits below a lip is drawn before the lip.** The bearing's balls are drawn sunk into the raceway, then both rings' top faces are drawn over them (`raceway`, then `rings`). Drawing the balls last put them on top of the rings, which read as a layer-order mistake.
- **Elements that touch in the part touch in the icon.** Size and count repeated parts from the geometry (`r = pitch * sin(pi / count)` for balls round a pitch circle), not by eye.
- **A texture follows its surface.** Knurl lines lean round the cylinder and run from the band's top edge to its bottom edge; two bands knurled in opposite hands lean opposite ways. Thread turns are curves across the shank.
- **Spend detail where the part is recognised.** The end face of a T-slot rail carries the real slot section; a rail drawn as four plain notches read as four posts. Outlines are `OUTLINE` (2.55), inner detail `DETAIL` (2.0) down to about 1.5; thinner lines vanish and thicker ones merge at 20 px.
- **Choose the view that shows the feature.** A face-on view hides the far side of a turned part, and two strips of a few pixels merge into one ink blob. If the feature that names the part is a couple of pixels wide, change the view rather than the line weight.
- **Keep the set one family.** Same view from above, same outline weight, similar size in the box. Compare a new icon with its neighbours on the sheet, especially parts that look alike (bearing, clutch, wheel).

## Helpers in tools/icons/draw.py

| Helper | Draws |
|---|---|
| `path(data, fill, stroke, width)` | one shape; every helper below returns these |
| `ellipse(cx, cy, rx)` | a horizontal circle (its `ry` is `rx * K`); pass `ry` for a ball |
| `cylinder(cx, top, bottom, rx, side, shade, face)` | an upright cylinder with its darker right strip |
| `bore(cx, cy, rx)` | a blue threaded hole in a top face |
| `prism(outline, z0, z1, scale, cx, cy)`, `box(...)` | a plan outline raised into a solid, its visible walls sorted by depth |
| `ring(axis, radius, axial, scale)` | the screen points of a circle about any axis |
| `revolved(axis, rim, tones, scale)` | a turned part about any axis from its `(radius, axial)` rim, far band first, plus a `face(radius)` for circles on the near end |
| `hull(points)` | the outline round two rings, for one band of a turned part |

The projection is the app's: x runs right and down, y left and down, z straight up, 30 degree diagonals. An upright round part is cheapest with `cylinder` and `ellipse`; a part turned to another axis, or compared across views, uses `revolved`.

## Gotchas

- **ImageMagick drops strokes when it rasterises these SVGs.** The first contact sheet showed icons with no outlines, threads or sprags. Render the HTML sheet with a browser or `qlmanage`.
- **A candidate that fills its box at one scale overflows at another view.** Check the 128 px drawing isn't clipped; pass each view its own `scale`.
- **Dark-theme `INK` is nearly black on a dark panel.** A shape that is only an `INK` stroke disappears there; give it a fill.
- **`--check` proves the rules, not the look.** It can't see that a wheel reads as a screw head. Read the sheet.
- **The sheet's dark theme is a colour swap, not the app.** Final proof is the package installed in SolidCraft through Plugins > Install plugin…, with the icon in the Plugins menu and nothing about it in the log.
