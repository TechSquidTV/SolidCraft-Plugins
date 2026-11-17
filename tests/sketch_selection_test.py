#!/usr/bin/env python3
"""Check that sketch-driven picks name their sketch by id, through the compiled plugin host.

SolidCraft 0.44 refuses two sketches with one name, so the two sketches are `Paths` and
`Paths 2`, each with its own `l1`: line ids repeat from one sketch to the next.

Run after ./build.sh; SOLIDCRAFT_CLI can select the host executable.
"""
import os

from harness import TMP, check, finish, inspect, killed, package, picks, run, skip

PLUGIN = "techsquid-extrusions"


def cmd(command, **params):
    return {"command": command, "params": params}


code, text, meta = inspect(PLUGIN)
if code:
    check("inspect", False, text[-1000:])
    finish()

steps = []
for name, x, length in [("Paths", 0, 100), ("Paths 2", 200, 160)]:
    steps += [
        cmd("sketch.create", name=name, plane={"x_dir": [1, 0, 0], "y_dir": [0, 0, 1]}),
        cmd("sketch.line", points=[[x, 20], [x, 20 + length]]),
        cmd("sketch.constraint.fix", entities=["l1"], fixed=True),
        cmd("sketch.finish"),
    ]
saved = os.path.join(TMP, "repeated-line-ids.sc")
code, text, result = run(steps + [cmd("document.inspect", measure=False, sketches=False)], save=saved)
check("two-sketch fixture builds", code == 0, text[-800:] if code else "")
if code:
    finish()
sketches = result[-1]["sketches"]
check("both sketches have a line l1", [s["name"] for s in sketches] == ["Paths", "Paths 2"])
items = [picks(s["id"], "l1", curve=True)[0] for s in sketches]


def make(placement):
    return cmd("plugin.run", path=package(PLUGIN), approval=meta["approval"], entry="make-extrusion",
               params={"mode": "Sketch driven", "slot": "T-slot", "size": "2020", "placement": placement})


code, text, result = run([make([items[1]]), cmd("inspect.measure", summary=True)], inp=saved)
if killed(code):
    skip("pick by sketch id", "host killed loading plugin")
else:
    check("l1 picked in the second sketch builds", code == 0, text[-800:] if code else "")
    if not code:
        made, measured = result[-2:]
        bodies = measured["bodies"]
        check("only the picked 160 mm path gets a rail", made["count"] == len(bodies) == 1
              and abs(bodies[0]["volume_mm3"] - made["area_mm2"] * 160) < 0.01)
        check("rail lies on the second sketch", len(bodies) == 1 and all(
            abs(bodies[0]["bbox"][key][i] - value) < 1e-4
            for key, point in [("min", [190, -10, 20]), ("max", [210, 10, 180])]
            for i, value in enumerate(point)))

code, text, _ = run([make(items)], inp=saved)
if killed(code):
    skip("lines from two sketches", "host killed loading plugin")
else:
    check("lines from two sketches are refused", code != 0
          and "straight lines in one sketch" in text, text[-800:] if code == 0 else "")

finish()
