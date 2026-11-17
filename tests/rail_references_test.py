#!/usr/bin/env python3
"""Check that rail reference edits move both cuts, including placed copies and reopen.

Run after ./build.sh with SOLIDCRAFT_CLI pointing to an unsigned host build.
"""
import os

from harness import TMP, check, finish, inspect, killed, package, picks, run, skip

PLUGIN = "techsquid-extrusions"


def cmd(command, **params):
    return {"command": command, "params": params}


def near(a, b):
    return abs(a - b) < 1e-4


def cylinders(reply, radius):
    return [f["cylinder"] for f in reply["faces"]
            if f.get("cylinder") and near(f["cylinder"]["radius"], radius)]


def case(face, slot="T-slot", size="2020", rotated=False, tapped=False):
    label = f"{slot} {size} {face}" + (" rotated copies" if rotated else "") + (" tapped" if tapped else "")
    params = dict(slot=slot, size=size, length=100, holes="25,75", hole_face=face)
    before = []
    if tapped:
        params["end_taps"] = "Start"
    if rotated:
        before = [cmd("sketch.create", name="Places", plane={
            "origin": [100, 200, 300], "x_dir": [0, 1, 0], "y_dir": [0, 0, 1]}),
            cmd("sketch.point", id="a", point=[0, 0]),
            cmd("sketch.point", id="b", point=[60, 0]), cmd("sketch.finish")]
        params.update(placement=picks("Places", "a,b"))
    name = f"{slot} {size} x 100" + (" taps start" if tapped else "") + f" holes {face} 25 75"
    sketch = name + f": Hole references {face}"
    saved = os.path.join(TMP, "rail.json")
    steps = before + [cmd("plugin.run", path=package(PLUGIN), approval=APPROVAL,
                          entry="make-extrusion", params=params),
                      cmd("sketch.inspect", sketch=sketch), cmd("inspect.measure", summary=True)]
    code, text, results = run(steps, save=saved)
    if killed(code):
        skip(label, "host killed while loading plugin")
        return
    check(label + ": builds", code == 0, text[-600:])
    if code:
        return
    reference, measured = results[-2:]
    curves = {c["id"]: c for c in reference["curves"]}
    check(label + ": shared centers and hidden sketch",
          all(curve["center"] == curves["access" + key[4:]]["center"] == key + ".center"
              for key, curve in curves.items() if key.startswith("hole"))
          and reference["visible"] is False)
    across = curves["hole1"]["center_at"][0]
    names = [b["name"] for b in measured["bodies"]]
    projected = face == "+Y" and not rotated and not tapped
    if projected:
        code, text, _ = run([
            cmd("sketch.create", name="Dependent", plane=reference["plane"]),
            cmd("sketch.project", refs=[{"sketch": sketch, "curve": "hole1"}]),
            cmd("sketch.finish"),
        ], inp=saved, save=saved)
        check(label + ": projects center to another sketch", code == 0, text[-600:])
        if code:
            return
    # Read every occurrence's actual faces, not only the component's sketch or volume.
    reads = [cmd("model.faces", body=n) for n in names] + [cmd("inspect.measure", summary=True)]
    edits = [cmd("sketch.edit", sketch=sketch),
             cmd("sketch.move_point", point="hole1.center", to=[across, 40]), cmd("sketch.finish")]
    code, text, moved = run(edits + reads, inp=saved, save=saved)
    check(label + ": edits", code == 0, text[-600:])
    if code:
        return
    # Local X/Y/Z becomes world Y/Z/X for the rotated placements.
    local_axis = 0 if face.endswith("X") else 1
    axis = (local_axis + 1) % 3 if rotated else local_axis
    along = 0 if rotated else 2
    cross = 2 if rotated else 1 - local_axis
    longitudinal_origin = 100 if rotated else 0
    transverse_origin = 300 if rotated else 0
    transverse = across * (1 if face in ("+X", "-Y") else -1) + transverse_origin
    for suffix, values in [("edited", moved[len(edits):]), ("reopened", None)]:
        if values is None:
            code, text, values = run(reads, inp=saved)
            check(label + ": reopens", code == 0, text[-600:])
            if code:
                return
        for i, faces in enumerate(values[:-1]):
            for radius in (2.75, 4.75):
                wall = cylinders(faces, radius)
                relevant = [c for c in wall if near(c["origin"][cross], transverse)]
                locations = {round(c["origin"][along] - longitudinal_origin, 4) for c in relevant}
                check(f"{label}: {suffix} copy {i + 1} radius {radius} follows center",
                      locations == {40.0, 75.0} and all(near(abs(c["axis"][axis]), 1) for c in wall),
                      str(locations))
                if size == "2040" and local_axis == 0:
                    other = {round(c["origin"][along] - longitudinal_origin, 4)
                             for c in wall if near(c["origin"][cross], 2 * transverse_origin - transverse)}
                    check(f"{label}: {suffix} copy {i + 1} radius {radius} keeps other slot",
                          other == {25.0, 75.0}, str(other))
                if radius == 4.75:
                    # Access cylinders end at the slot floor, not a depth measured from the core.
                    outward = 1 if face.startswith("+") else -1
                    half = 20 if size == "2040" and local_axis == 1 else 10
                    floor = outward * (half - (6.1 if slot == "V-slot" else 6))
                    offset = 200 + 60 * i if rotated else 0
                    check(f"{label}: {suffix} copy {i + 1} recess depth",
                          bool(wall) and all(near(c["origin"][axis], floor + offset) for c in wall))
        after = {b["name"]: b for b in values[-1]["bodies"]}
        check(label + f": {suffix} bodies stay closed and volume preserved",
              len(names) == (2 if rotated else 1) and set(after) == set(names)
              and all(after[b["name"]]["closed"]
                      and near(b["volume_mm3"], after[b["name"]]["volume_mm3"])
                      for b in measured["bodies"]))
    if projected:
        code, text, linked = run([cmd("sketch.inspect", sketch="Dependent")], inp=saved)
        check(label + ": projected center follows after reopening", code == 0
              and any(c.get("center_at") == [0.0, 40.0] for c in linked[0]["curves"]), text[-600:])


if __name__ == "__main__":
    code, text, info = inspect(PLUGIN)
    if code or not info:
        raise SystemExit(f"Build the plugins first: {text}")
    APPROVAL = info["approval"]
    for face in ("+Y", "-Y", "+X", "-X"):
        case(face)
    case("+X", "V-slot", "2040", rotated=True)
    case("-Y", tapped=True)
    finish()
