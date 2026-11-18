#!/usr/bin/env python3
"""Check that the inside of a sketch circle places a part, through the compiled plugin host.

The Placement field takes a profile as Extrude does: a click anywhere inside a circle.
The plugin reads it as the circle's centre, so a run from two picked discs must build
exactly what a run from the two circles builds, in every entry. A region that is not the
inside of a circle has no one centre and is refused, and a sketch-driven rail still
follows lines only.

The fixture is a 60 x 40 x 8 plate with a sketch on its top: the plate's outline and two
circles of radius 3, 30 apart. The sketch then closes three regions: the plate around the
holes, and the two discs.

Run after ./build.sh; SOLIDCRAFT_CLI can select the host executable.
"""
import os

from harness import TMP, check, finish, inspect, killed, package, plugin_ids, run, skip

TOL = 1e-6

# Entry: (plugin, parameters that build the part alone).
ENTRIES = {
    "make-bearing": ("techsquid-bearings", {"size": "625", "closure": "Sealed"}),
    "make-csk": ("techsquid-bearings", {"size": "CSK12P"}),
    "make-v-wheel": ("techsquid-v-wheels", {"size": "Solid V Wheel", "assembled": False}),
    "make-screw": ("techsquid-screws", {"size": "M3", "length": 12, "head": "Socket head",
                                        "drive": "Hex socket"}),
    "make-insert": ("techsquid-inserts", {"size": "M3 × 5.7", "create": "Insert only"}),
    "make-nut": ("techsquid-nuts", {"size": "M6"}),
    "make-extrusion": ("techsquid-extrusions", {"slot": "V-slot", "size": "2040", "length": 120}),
    "make-t-nut": ("techsquid-extrusions", {"type": "Pre-assembly", "size": "M5", "slot": "T-slot",
                                            "along": "Sketch X"}),
}


def cmd(command, **params):
    return {"command": command, "params": params}


approvals = {}
for plugin_id in plugin_ids():
    code, text, meta = inspect(plugin_id)
    if code:
        check(f"{plugin_id}: inspect", False, text[-1000:])
        finish()
    approvals[plugin_id] = meta["approval"]
    for entry in meta["manifest"]["runtime"]["entries"]:
        check(f"{entry['name']}: has a case in ENTRIES", entry["name"] in ENTRIES,
              "" if entry["name"] in ENTRIES else "add one to tests/profile_picks_test.py")

# The plate first, to put the sketch on wherever the box's top is.
plate = [cmd("solid.box", length=60, width=40, height=8, name="Plate")]
code, text, result = run(plate + [cmd("inspect.measure", summary=True)])
check("plate builds", code == 0, text[-800:] if code else "")
if code:
    finish()
box = result[-1]["bodies"][0]["bbox"]
low, high = box["min"], box["max"]
mid = [(low[i] + high[i]) / 2 for i in range(2)]
centres = [[mid[0] - 15, mid[1]], [mid[0] + 15, mid[1]]]

saved = os.path.join(TMP, "discs.sc")
steps = plate + [
    cmd("sketch.create", name="Holes",
        plane={"origin": [0, 0, high[2]], "x_dir": [1, 0, 0], "y_dir": [0, 1, 0]}),
    cmd("sketch.rectangle.two_point", p0=low[:2], p1=high[:2]),
    cmd("sketch.circle.center", center=centres[0], radius=3),
    cmd("sketch.circle.center", center=centres[1], radius=3),
    cmd("sketch.finish"),
    cmd("sketch.inspect", sketch="Holes"),
]
code, text, result = run(steps, save=saved)
check("fixture builds", code == 0, text[-800:] if code else "")
if code:
    finish()
sketch = result[-1]
circles = [c["id"] for c in sketch["curves"] if c["type"] == "circle"]
discs = [p["index"] for c in circles for p in sketch["profiles"] if p["curves"] == [c]]
around = [p["index"] for p in sketch["profiles"] if len(p["curves"]) == 4]
check("the sketch closes the plate and two discs",
      len(circles) == 2 and len(discs) == 2 and len(around) == 1,
      "" if len(discs) == 2 and len(around) == 1 else str(sketch["profiles"]))
if check.failed:
    finish()

by_profile = [{"type": "profile", "sketch": sketch["id"], "index": i} for i in discs]
by_circle = [{"type": "external_sketch_curve", "sketch": sketch["id"], "id": c} for c in circles]


def make(entry, placement, **more):
    plugin_id, params = ENTRIES[entry]
    return cmd("plugin.run", path=package(plugin_id), approval=approvals[plugin_id], entry=entry,
               params={**params, **more, "placement": placement})


def built(entry, placement, **more):
    """The bodies a run leaves, as sorted (box, volume) rows; or None and the output."""
    code, text, result = run([make(entry, placement, **more), cmd("inspect.measure", summary=True)],
                             inp=saved)
    if killed(code):
        return "killed", text
    if code:
        return None, text
    rows = sorted((b["bbox"]["min"] + b["bbox"]["max"], b.get("volume_mm3") or 0.0)
                  for b in result[-1]["bodies"])
    return rows, text


def same(a, b):
    return len(a) == len(b) and all(
        abs(x - y) < TOL for (box_a, vol_a), (box_b, vol_b) in zip(a, b)
        for x, y in zip(box_a + [vol_a], box_b + [vol_b]))


for entry in ENTRIES:
    discs_built, text = built(entry, by_profile)
    if discs_built == "killed":
        skip(f"{entry}: discs", "host killed loading plugin")
        continue
    check(f"{entry}: two picked discs build", discs_built is not None,
          text[-800:] if discs_built is None else "")
    circles_built, text = built(entry, by_circle)
    check(f"{entry}: two picked circles build", circles_built not in (None, "killed"),
          text[-800:] if circles_built is None else "")
    if discs_built and circles_built and circles_built != "killed":
        check(f"{entry}: a disc places what its circle places",
              len(discs_built) > 1 and same(discs_built, circles_built),
              f"{len(discs_built)} bodies from discs, {len(circles_built)} from circles")

# Drilling follows the same points: the plate loses the same material either way.
drilled = {}
for label, placement in [("discs", by_profile), ("circles", by_circle)]:
    rows, text = built("make-screw", placement, create="Screw and hole")
    if rows == "killed":
        skip(f"drill from {label}", "host killed loading plugin")
        continue
    check(f"screws and holes build from {label}", rows is not None, text[-800:] if rows is None else "")
    drilled[label] = rows
if drilled.get("discs") and drilled.get("circles"):
    check("holes from discs match holes from circles", same(drilled["discs"], drilled["circles"]))
    plate_volume = max(volume for _, volume in drilled["discs"])
    check("the plate is drilled under both discs",
          len(drilled["discs"]) == 3 and plate_volume < 60 * 40 * 8 - 1.0,
          f"{len(drilled['discs'])} bodies, plate {plate_volume:.3f} mm3")

# A disc and its own circle are one place.
rows, text = built("make-nut", [by_profile[0], by_circle[0]])
if rows == "killed":
    skip("a disc and its circle", "host killed loading plugin")
else:
    check("a disc picked with its own circle gets one nut",
          rows is not None and len(rows) == 2, text[-800:] if rows is None else f"{len(rows or [])} bodies")

# The plate around the holes has no one centre.
code, text, _ = run([make("make-screw", [{"type": "profile", "sketch": sketch["id"], "index": around[0]}])],
                    inp=saved)
if killed(code):
    skip("a region that is not a circle", "host killed loading plugin")
else:
    check("a region that is not the inside of a circle is refused",
          code != 0 and "the region picked in Holes is not the inside of a circle" in text
          # SolidCraft 0.44 says "invalid input placement", newer hosts "Screws: Placement:".
          and ("invalid input placement" in text or "Placement: " in text), text[-800:])

code, text, _ = run([make("make-extrusion", by_profile, mode="Sketch driven")], inp=saved)
if killed(code):
    skip("a sketch-driven rail from a disc", "host killed loading plugin")
else:
    check("a sketch-driven rail refuses a region",
          code != 0 and "not a point or a region" in text, text[-800:] if code == 0 else "")

finish()
