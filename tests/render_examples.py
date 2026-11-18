#!/usr/bin/env python3
"""Regenerate the native screenshots and README banner with the built plugins."""

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from harness import CLI, ROOT, recipe_commands, run

EXAMPLES = Path(ROOT) / "examples"
FIXTURES = Path(ROOT) / "tests" / "fixtures" / "examples"
CPU_RENDERER = {**os.environ, "SOLIDCRAFT_GPU": "cpu"}
POCKET_CAMERA = ([0, 0, 6], 0.03, 0.22, 100)
POCKET_SECTION = ("--section", "y=0,flip")
INSERT_DETAIL_SIZE = (1100, 780)
INSERT_DETAIL_CAMERA = ([26, 19, 23], -0.61, 0.27, 52)


def snapshot(model, output, size, camera=None, *, options=(), quality="fine", script=None):
    output.parent.mkdir(parents=True, exist_ok=True)
    command = [CLI, "snapshot"]
    if script is not None:
        command.append(script)
    width, height = size
    command.extend(["--in", model, "--out", output, "--width", str(width), "--height", str(height),
                    "--no-grid", "--no-axes", "--quality", quality])
    if camera is None:
        command.extend(["--view", "iso"])
    else:
        target, yaw, pitch, distance = camera
        command.extend(["--camera", json.dumps(dict(target=target, yaw=yaw, pitch=pitch, distance=distance))])
    result = subprocess.run([*command, *options], env=CPU_RENDERER, capture_output=True, text=True)
    if result.returncode != 0 or not output.is_file():
        sys.exit(f"Could not render {output.name}:\n{result.stderr or result.stdout}")


def before_pocket_edits(recipe, output):
    """Reset a temporary session to the mounting plate recipe's original parameters and nut centers."""
    steps = json.loads(recipe.read_text())["steps"]
    initial = {}
    edited = set()
    for step in steps:
        if step["command"] == "parameters.change":
            name = step["params"]["name"]
            if name in initial:
                edited.add(name)
            initial.setdefault(name, step)
    reset = [step for name, step in initial.items() if name in edited]

    sketch = "Nut centers"
    start = next(i for i, step in enumerate(steps)
                 if step["command"] == "sketch.create" and step["params"]["name"] == sketch)
    reset.append({"command": "sketch.edit", "params": {"sketch": sketch}})
    for step in steps[start + 1:]:
        kind, params = step["command"], step["params"]
        if kind == "sketch.finish":
            break
        if kind == "sketch.point":
            reset.append({"command": "sketch.move_point", "params": {"point": params["id"], "to": params["point"]}})
        if kind == "sketch.circle.center":
            reset.append({"command": "sketch.move_point",
                          "params": {"point": params["id"] + ".center", "to": params["center"]}})
    reset.append({"command": "sketch.finish", "params": {}})
    output.write_text(json.dumps(reset))
    return output


def render_gallery(staging):
    plate = "editable-mounting-plate"
    before = before_pocket_edits(EXAMPLES / plate / f"{plate}.steps.json", staging / "before-pocket-edits.json")
    enclosure_camera = ([0, 0, 15], -0.785, 0.615, 150)
    views = {
        "v-slot-gantry": [
            ("v-slot-gantry", (1600, 960), ([0, 0, 12], 0.28, 0.44, 220),
             {"options": ("--body", "V-slot 2020 x 200:color=#929DA7")}),
            ("axle-section", (1280, 810), ([-34, 0, 23], -1.52, 0.15, 110),
             {"options": ("--section", "x=-34")}),
            ("frame-joint", (1120, 720), ([-80, 0, 11], -0.12, 0.11, 65),
             {"options": ("--section", "y=0,flip")}),
        ],
        "pcb-enclosure": [
            ("pcb-enclosure", (1280, 800), enclosure_camera, {"options": ("--body", "Lid:opacity=0.3")}),
            ("pcb-enclosure-closed", (1280, 800), enclosure_camera, {}),
            ("insert-detail", INSERT_DETAIL_SIZE, INSERT_DETAIL_CAMERA,
             {"options": ("--body", "Lid:opacity=0.22", "--body", "Base:opacity=0.30")}),
        ],
        plate: [
            (plate, (1440, 900), ([0, 0, 6], 0.34, 0.67, 155), {}),
            ("pockets-before", (1280, 560), POCKET_CAMERA, {"options": POCKET_SECTION, "script": before}),
            ("pockets-after", (1280, 560), POCKET_CAMERA, {"options": POCKET_SECTION}),
        ],
        "sketch-driven-rails": [
            ("sketch-driven-rails", (1280, 800), ([245, 0, 140], 0.35, 0.4, 650), {}),
        ],
        "bearings-and-clutches": [
            ("bearings-and-clutches", (1440, 1250), ([0, 48, 0], 0, 1.12, 385), {}),
            ("clutch-detail", (1440, 720), ([0, -50, 0], 0, 0.95, 245), {"options": ("--section", "y=7")}),
        ],
    }
    for example, captures in views.items():
        model = EXAMPLES / example / f"{example}.json"
        for name, size, camera, settings in captures:
            snapshot(model, staging / example / f"{name}.png", size, camera, **settings)


def render_catalog(staging):
    portraits = [
        ("screws", "editable-mounting-plate/reference/screw-catalog.png",
         (1600, 700), ([0, 0, -5], 0, 0.6, 92), {}),
        ("nut", "editable-mounting-plate/reference/standalone-nut.png",
         (1280, 800), None, {}),
        ("insert", "pcb-enclosure/reference/standalone-insert.png",
         (1280, 800), ([0, 0, -2.85], 0.5, 0.55, 14), {"quality": "ultra"}),
        ("rails", "sketch-driven-rails/reference/rail-profiles.png",
         (1280, 800), ([80, 0, 20], 0.3, 0.8, 330), {}),
        ("slot-nuts", "v-slot-gantry/reference/slot-nut-styles.png",
         (1280, 800), ([0, 0, 14], 0.45, 0.6, 95), {"options": ("--body", "T-slot 2020 x 80:opacity=0.3")}),
    ]
    for fixture, output, size, camera, settings in portraits:
        model = staging / f"{fixture}.json"
        recipe = json.loads((FIXTURES / f"{fixture}.steps.json").read_text())
        code, message, results = run(recipe_commands(recipe), save=model)
        if code != 0 or results is None:
            sys.exit(f"Could not build the {fixture} portrait from its regression recipe:\n{message}")
        snapshot(model, staging / output, size, camera, **settings)


def chrome_binary():
    configured = os.environ.get("CHROME")
    candidates = (configured,) if configured else (
        "google-chrome", "chromium", "chromium-browser", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    )
    for candidate in candidates:
        if browser := shutil.which(candidate):
            return browser
    sys.exit("Install Chrome or Chromium, or set CHROME to its executable path to render the README banner.")


def render_banner(staging, browser):
    output = staging / "readme-banner.png"
    with tempfile.TemporaryDirectory(prefix="banner-", dir=staging) as directory:
        work = Path(directory)
        layout = work / "readme-banner.html"
        shutil.copyfile(FIXTURES / layout.name, layout)

        colors = work / "insert-appearance.json"
        colors.write_text(json.dumps([
            {"command": "appearance.assign", "params": {"components": ["Insert M3 x 5.7"], "color": "#CDA058"}},
        ]))
        snapshot(EXAMPLES / "pcb-enclosure" / "pcb-enclosure.json", work / "insert-detail.png",
                 INSERT_DETAIL_SIZE, INSERT_DETAIL_CAMERA, script=colors,
                 options=("--body", "Lid:opacity=0.18", "--body", "Base:opacity=0.20"))

        command = [
            browser, "--headless", "--no-first-run", "--no-default-browser-check", "--disable-background-networking",
            "--force-color-profile=srgb", "--force-device-scale-factor=1", "--hide-scrollbars",
            "--window-size=1600,800", f"--user-data-dir={work / 'chrome-profile'}",
            f"--screenshot={output}", layout.as_uri(),
        ]
        result = subprocess.run(command, capture_output=True, text=True)
        if result.returncode != 0 or not output.is_file():
            sys.exit(f"Could not render the README banner:\n{result.stderr or result.stdout}")


def main():
    if not shutil.which(CLI):
        sys.exit("Set SOLIDCRAFT_CLI to the path of an unsigned, locally built solidcraft-cli.")
    browser = chrome_binary()
    # Stage all captures before replacing any published PNGs, on the same filesystem.
    with tempfile.TemporaryDirectory(prefix=".screenshots-", dir=EXAMPLES) as directory:
        staging = Path(directory)
        render_gallery(staging)
        render_banner(staging, browser)
        render_catalog(staging)
        captures = sorted(staging.rglob("*.png"))
        updated = 0
        for capture in captures:
            output = EXAMPLES / capture.relative_to(staging)
            if not output.is_file() or output.read_bytes() != capture.read_bytes():
                output.parent.mkdir(parents=True, exist_ok=True)
                capture.replace(output)
                print(output.relative_to(ROOT))
                updated += 1
        print(f"Rendered {len(captures)} screenshots ({updated} updated).")


if __name__ == "__main__":
    main()
