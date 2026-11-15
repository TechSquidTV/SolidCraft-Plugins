#!/usr/bin/env python3
"""Check nut pocket and bolt-hole links through point, plane and depth edits."""
import json
import math
import os

from harness import TMP, check, finish, inspect, killed, package, run, skip


def cmd(command, **params):
    return {"command": command, "params": params}


def near(a, b):
    return abs(a - b) < 1e-4


def test():
    code, text, info = inspect("techsquid-nuts")
    if code or not info:
        raise SystemExit(f"Build the packages first: {text}")
    saved = os.path.join(TMP, "nut-pockets.json")
    setup = [
        cmd("parameters.change", name="thickness", expression="12 mm"),
        cmd("parameters.change", name="yaw", expression="0 deg"),
        cmd("parameters.change", name="tilt", expression="0 deg"),
        cmd("construct.plane.at_angle", base="XY", axis="Z", angle="yaw", name="Turned base"),
        cmd("construct.plane.at_angle", base="Turned base", axis="X", angle="tilt", name="Plate plane"),
        cmd("sketch.create", plane="Plate plane", name="Plate profile"),
        cmd("sketch.rectangle.two_point", p0=[-50, -50], p1=[50, 50]),
        cmd("sketch.finish"),
        cmd("solid.extrude", sketch="Plate profile", distance="thickness", body_name="Plate"),
        cmd("construct.plane.offset", base="Plate plane", offset="thickness", name="Layout plane"),
        cmd("sketch.create", plane="Layout plane", name="Layout"),
        cmd("sketch.point", point=[15, 15], id="p"),
        cmd("sketch.circle.center", center=[-15, -15], radius=2, id="c"),
        cmd("sketch.finish"),
        cmd("plugin.run", path=package("techsquid-nuts"), approval=info["approval"], entry="make-nut",
            params={"size": "M6", "create": "Pocket only", "sketch": "Layout", "sketch_point": "p,c"}),
        cmd("sketch.inspect", sketch="Layout"),
    ]
    code, text, results = run(setup, save=saved)
    if killed(code):
        skip("nut pocket references", "host killed while loading plugin")
        return
    check("nut pockets: build from a point and a circle", code == 0, text[-1000:])
    if code:
        return
    source = results[-1]
    check("source profiles stay unchanged", len(source["profiles"]) == 1
          and source["profiles"][0]["curves"] == ["c"])
    check("source frame lines are construction geometry", len(source["curves"]) == 3
          and all(c["construction"] for c in source["curves"] if c["type"] == "line"))
    with open(saved) as f:
        doc = json.load(f)
    profiles = [f["id"] for f in doc["features"] if f["type"] == "sketch"
                and f.get("name", "").startswith("Nut pocket M6 profile")]
    pockets = [f["id"] for f in doc["features"] if f["type"] == "extrude" and f["operation"] == "cut"]
    check("each pocket has a linked profile", len(profiles) == len(pockets) == 2)
    reads = [cmd("sketch.inspect", sketch=p) for p in profiles]
    reads += [cmd("model.faces", body="Plate"), cmd("inspect.measure", summary=True)]

    def verify(label, edits, xy, thickness, yaw, depths, tilt=0):
        code, text, results = run(edits + reads, inp=saved, save=saved)
        check(label + ": rebuilds", code == 0, text[-1000:])
        if code:
            return
        data = results[len(edits):]
        a = math.radians(yaw)
        t = math.radians(tilt)
        normal = [0, -math.sin(t), math.cos(t)]
        def world(x, y, z):
            u, v = x * math.cos(a) - y * math.sin(a), x * math.sin(a) + y * math.cos(a)
            return [u, v * math.cos(t) - z * math.sin(t), v * math.sin(t) + z * math.cos(t)]
        centers = [world(x, y, thickness) for x, y in xy]
        def on_axis(point, center):
            delta = [v - w for v, w in zip(point, center)]
            along = sum(v * n for v, n in zip(delta, normal))
            return all(near(v, along * n) for v, n in zip(delta, normal))
        for i, profile in enumerate(data[:2]):
            center = next(p["world"] for p in profile["points"] if p["id"] == "c1.center")
            check(label + f": pocket {i + 1} follows point and plane",
                  all(near(v, w) for v, w in zip(center, centers[i]))
                  and profile["fully_constrained"] and not profile["failing"])
        faces = data[2]["faces"]
        bores = [f["cylinder"] for f in faces if f.get("cylinder") and near(f["cylinder"]["radius"], 3.3)]
        check(label + ": both bolt holes follow", len(bores) == 2
              and all(any(on_axis(c["origin"], center) for c in bores) for center in centers)
              and all(near(abs(sum(v * n for v, n in zip(c["axis"], normal))), 1) for c in bores))
        # Check the actual pocket floors, not only the sketches and total volume:
        # a stale pocket cut could otherwise stay behind while its bolt hole moves.
        for i, (depth, center) in enumerate(zip(depths, centers)):
            floors = [f for f in faces if f.get("plane_normal")
                      and near(abs(sum(v * n for v, n in zip(f["plane_normal"], normal))), 1)
                      and near(sum(v * n for v, n in zip(f["point"], normal)), thickness - depth)]
            check(label + f": pocket {i + 1} floor follows center and depth",
                  any(on_axis(f["centroid"], center) for f in floors))
        hex_area = math.sqrt(3) / 2 * 10.4 ** 2
        removed = sum(hex_area * d + math.pi * 3.3 ** 2 * (thickness - d) for d in depths)
        bodies = data[3]["bodies"]
        check(label + ": exact cut volume and closed body", len(bodies) == 1 and bodies[0]["closed"]
              and abs(bodies[0]["volume_mm3"] - (10000 * thickness - removed)) < 0.01)

    verify("initial", [], [(15, 15), (-15, -15)], 12, 0, [5.2, 5.2])
    move = [cmd("sketch.edit", sketch="Layout"), cmd("sketch.move_point", point="p", to=[25, 20]), cmd("sketch.finish")]
    verify("move point", move, [(25, 20), (-15, -15)], 12, 0, [5.2, 5.2])
    verify("raise source plane", [cmd("parameters.change", name="thickness", expression="16 mm")],
           [(25, 20), (-15, -15)], 16, 0, [5.2, 5.2])
    verify("rotate source plane", [cmd("parameters.change", name="yaw", expression="90 deg")],
           [(25, 20), (-15, -15)], 16, 90, [5.2, 5.2])
    verify("change pocket depth", [
        cmd("parameters.change", name="pocket_depth", expression="7 mm"),
        cmd("timeline.edit", feature=pockets[0], set={"extent.distance": "pocket_depth"}),
    ], [(25, 20), (-15, -15)], 16, 90, [7, 5.2])
    verify("drive depth by parameter", [cmd("parameters.change", name="pocket_depth", expression="8 mm")],
           [(25, 20), (-15, -15)], 16, 90, [8, 5.2])
    verify("tilt source plane", [cmd("parameters.change", name="tilt", expression="60 deg")],
           [(25, 20), (-15, -15)], 16, 90, [8, 5.2], tilt=60)
    move_circle = [cmd("sketch.edit", sketch="Layout"),
                   cmd("sketch.move_point", point="c.center", to=[-25, -20]), cmd("sketch.finish")]
    verify("move circle center", move_circle, [(25, 20), (-25, -20)], 16, 90, [8, 5.2], tilt=60)
    verify("reopen", [], [(25, 20), (-25, -20)], 16, 90, [8, 5.2], tilt=60)


if __name__ == "__main__":
    test()
    finish()
