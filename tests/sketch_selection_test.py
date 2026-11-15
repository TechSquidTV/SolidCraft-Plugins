#!/usr/bin/env python3
"""Check sketch-driven selection identity through the compiled plugin host.

Run after ./build.sh; SOLIDCRAFT_CLI can select the host executable.
"""
import os

from harness import TMP, check, finish, inspect, killed, package, run, skip

PLUGIN = "techsquid-extrusions"


def cmd(command, **params):
    return {"command": command, "params": params}


code, text, meta = inspect(PLUGIN)
if code:
    check("inspect", False, text[-1000:])
    finish()

steps = []
for x, length in [(0, 100), (200, 160)]:
    steps += [
        cmd("sketch.create", name="Paths", plane={"x_dir": [1, 0, 0], "y_dir": [0, 0, 1]}),
        cmd("sketch.line", points=[[x, 20], [x, 20 + length]]),
        cmd("sketch.constraint.fix", entities=["l1"], fixed=True),
        cmd("sketch.finish"),
    ]
saved = os.path.join(TMP, "duplicate-sketch-names.sc")
code, text, result = run(steps + [cmd("document.inspect", measure=False, sketches=False)], save=saved)
check("duplicate-name fixture builds", code == 0, text[-800:] if code else "")
if code:
    finish()
sketches = result[-1]["sketches"]
check("fixture retains duplicate sketch names", [s["name"] for s in sketches] == ["Paths", "Paths"])
items = [{"type": "sketch_curve", "sketch": s["id"], "id": "l1"} for s in sketches]
make = cmd("plugin.run", path=package(PLUGIN), approval=meta["approval"], entry="make-extrusion",
           params={"mode": "Sketch driven", "slot": "T-slot", "size": "2020"})

code, text, result = run([
    cmd("select.set", items=[items[1]]), make, cmd("inspect.measure", summary=True),
], inp=saved)
if killed(code):
    skip("duplicate-name selection", "host killed loading plugin")
else:
    check("selected duplicate-name sketch builds", code == 0, text[-800:] if code else "")
    if not code:
        made, measured = result[-2:]
        bodies = measured["bodies"]
        check("only the selected 160 mm path gets a rail", made["count"] == len(bodies) == 1
              and abs(bodies[0]["volume_mm3"] - made["area_mm2"] * 160) < 0.01)
        check("rail lies on the second sketch", len(bodies) == 1 and all(
            abs(bodies[0]["bbox"][key][i] - value) < 1e-4
            for key, point in [("min", [190, -10, 20]), ("max", [210, 10, 180])]
            for i, value in enumerate(point)))

code, text, _ = run([cmd("select.set", items=items), make], inp=saved)
if killed(code):
    skip("mixed duplicate-name sketches", "host killed loading plugin")
else:
    check("different sketch IDs with the same name are rejected", code != 0
          and "select lines from one sketch" in text, text[-800:] if code == 0 else "")

finish()
