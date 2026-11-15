#!/usr/bin/env python3
"""Check nut pocket and bolt-hole links through point, plane and depth edits."""
import copy
import json
import math
import os

from harness import TMP, check, finish, inspect, killed, package, run, skip


def cmd(command, **params):
    return {"command": command, "params": params}


def near(a, b):
    return abs(a - b) < 1e-4


def test(with_nuts=False, modeled=False, reuse=False, large_moves=False):
    code, text, info = inspect("techsquid-nuts")
    if code or not info:
        raise SystemExit(f"Build the packages first: {text}")
    saved = os.path.join(TMP, f"nut-pockets-{with_nuts}-{modeled}-{large_moves}.json")
    initial = [(-14, 0), (14, 0)] if large_moves else [(15, 15), (-15, -15)]
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
        cmd("sketch.point", point=initial[0], id="p"),
        cmd("sketch.circle.center", center=initial[1], radius=2, id="c"),
        cmd("sketch.finish"),
        cmd("plugin.run", path=package("techsquid-nuts"), approval=info["approval"], entry="make-nut",
            params={"size": "M6", "create": "Nut and pocket" if with_nuts else "Pocket only",
                    "thread_modeled": modeled, "sketch": "Layout", "sketch_point": "p,c"}),
        cmd("sketch.inspect", sketch="Layout"),
    ]
    if reuse:
        second = copy.deepcopy(setup[-2])
        setup[-2]["params"]["params"]["sketch_point"] = "p"
        second["params"]["params"]["sketch_point"] = "c"
        setup.insert(-1, second)
        # Build on an already rotated and tilted frame; the second invocation must
        # reuse the off-origin threaded definition without moving the first nut.
        setup[1]["params"]["expression"] = "90 deg"
        setup[2]["params"]["expression"] = "60 deg"
        setup.insert(0, cmd("component.create", name="Carrier", activate=True, grounded=True))
    code, text, results = run(setup, save=saved)
    if killed(code):
        skip("nut pocket references", "host killed while loading plugin")
        return
    check("nut pockets: build from a point and a circle", code == 0, text[-1000:])
    if code:
        return
    if reuse:
        first, second = [r["component"] for r in results[-3:-1]]
        check("separate runs reuse the nut definition", not first["reused"] and second["reused"]
              and first["component"] == second["component"]
              and first["occurrences"] != second["occurrences"])
    source = results[-1]
    check("source profiles stay unchanged", len(source["profiles"]) == 1
          and source["profiles"][0]["curves"] == ["c"])
    check("source frame lines are construction geometry", len(source["curves"]) == (5 if reuse else 3)
          and all(c["construction"] for c in source["curves"] if c["type"] == "line"))
    with open(saved) as f:
        doc = json.load(f)
    profiles = [f["id"] for f in doc["features"] if f["type"] == "sketch"
                and f.get("name", "").startswith("Nut pocket M6 profile")]
    pockets = [f["id"] for f in doc["features"] if f["type"] == "extrude" and f["operation"] == "cut"]
    check("each pocket has a linked profile", len(profiles) == len(pockets) == 2)
    reads = [cmd("sketch.inspect", sketch=p) for p in profiles]
    reads += [cmd("model.faces", body="Plate"), cmd("inspect.measure", summary=True), cmd("joint.inspect")]

    def verify(label, edits, xy, thickness, yaw, depths, tilt=0, shift=(0, 0, 0), widths=(10.4, 10.4)):
        code, text, results = run(edits + reads, inp=saved, save=saved)
        check(label + ": rebuilds", code == 0, text[-1000:])
        if code:
            return
        data = results[len(edits):]
        a = math.radians(yaw)
        t = math.radians(tilt)
        normal = [0, -math.sin(t), math.cos(t)]
        def rotated(x, y, z):
            u, v = x * math.cos(a) - y * math.sin(a), x * math.sin(a) + y * math.cos(a)
            return [u, v * math.cos(t) - z * math.sin(t), v * math.sin(t) + z * math.cos(t)]
        def world(x, y, z):
            return [v + offset for v, offset in zip(rotated(x, y, z), shift)]
        centers = [world(x, y, thickness) for x, y in xy]
        def on_axis(point, center):
            delta = [v - w for v, w in zip(point, center)]
            along = sum(v * n for v, n in zip(delta, normal))
            return all(near(v, along * n) for v, n in zip(delta, normal))
        for i, profile in enumerate(data[:2]):
            center = next(p["world"] for p in profile["points"] if p["id"] == "c1.center")
            check(label + f": pocket {i + 1} follows point and plane",
                  all(near(v, w) for v, w in zip(center, centers[i]))
                  and profile["fully_constrained"] and not profile["failing"]
                  and len(profile["profiles"]) == 1)
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
                      and near(sum(v * n for v, n in zip(f["point"], normal)), thickness - depth + sum(v * n for v, n in zip(shift, normal)))]
            check(label + f": pocket {i + 1} floor follows center and depth",
                  any(on_axis(f["centroid"], center) for f in floors))
        removed = sum(math.sqrt(3) / 2 * w ** 2 * d + math.pi * 3.3 ** 2 * (thickness - d)
                      for w, d in zip(widths, depths))
        bodies = [b for b in data[3]["bodies"] if b["name"] == "Plate"]
        check(label + ": exact cut volume and closed body", len(bodies) == 1 and bodies[0]["closed"]
              and abs(bodies[0]["volume_mm3"] - (10000 * thickness - removed)) < 0.01)

        joints = data[4]["joints"]
        check(label + ": one healthy seat per nut", len(joints) == (2 if with_nuts else 0)
              and all(not j["errors"] and not j["conflicts"] for j in joints))
        if with_nuts:
            nuts = [b for b in data[3]["bodies"] if b["name"] != "Plate"]
            check(label + ": both nut bodies stay closed", len(nuts) == 2 and all(b["closed"] for b in nuts))
            for i, (joint, center, depth) in enumerate(zip(joints, centers, depths)):
                floor = [v - n * depth for v, n in zip(center, normal)]
                # X of the seat is the inward normal of the low-Y pocket wall.
                turn = rotated(0, 1, 0)
                check(label + f": nut {i + 1} follows seat position and rotation",
                      all(joint[side]["resolves"]
                          and all(near(v, w) for v, w in zip(joint[side]["frame"]["origin"], floor))
                          and all(near(v, w) for v, w in zip(joint[side]["frame"]["z"], normal))
                          and all(near(v, w) for v, w in zip(joint[side]["frame"]["x"], turn))
                          for side in ("a", "b")))
                middle = [v + n * 2.6 for v, n in zip(floor, normal)]
                check(label + f": actual nut {i + 1} body follows seat",
                      any(all(near((lo + hi) / 2, want)
                              for lo, hi, want in zip(b["bbox"]["min"], b["bbox"]["max"], middle))
                          for b in nuts))

    if large_moves:
        verify("initial horizontal layout", [], initial, 12, 0, [5.2, 5.2])
        # The first move used to collapse the hex. Subsequent moves cross both
        # axes, passing through zero and changing quadrants by many hex widths.
        for xy in ([(-22, 0), (22, 0)], [(-40, 0), (40, 0)], [(40, 0), (-40, 0)],
                   [(0, 40), (0, -40)], [(0, -40), (0, 40)], [(-30, -30), (30, 30)]):
            verify(f"large move {xy}", [
                cmd("sketch.edit", sketch="Layout"),
                cmd("sketch.move_point", point="p", to=xy[0]),
                cmd("sketch.move_point", point="c.center", to=xy[1]),
                cmd("sketch.finish"),
            ], xy, 12, 0, [5.2, 5.2])
        width = next(c["param"] for f in doc["features"] if f["id"] == profiles[0]
                     for c in f["sketch"]["constraints"] if c["type"] == "diameter")
        verify("change linked pocket width", [cmd("parameters.change", name=width, expression="12.4 mm")],
               xy, 12, 0, [5.2, 5.2], widths=(12.4, 10.4))
        verify("change depth after large moves", [
            cmd("parameters.change", name="pocket_depth", expression="8 mm"),
            cmd("timeline.edit", feature=pockets[0], set={"extent.distance": "pocket_depth"}),
        ], xy, 12, 0, [8, 5.2], widths=(12.4, 10.4))
        verify("reopen after large moves and width and depth edits", [], xy, 12, 0, [8, 5.2],
               widths=(12.4, 10.4))
        return

    if reuse:
        verify("initial rotated reused definition", [], [(15, 15), (-15, -15)],
               12, 90, [5.2, 5.2], tilt=60)
    verify("initial", [cmd("parameters.change", name="yaw", expression="0 deg"),
                       cmd("parameters.change", name="tilt", expression="0 deg")] if reuse else [],
           [(15, 15), (-15, -15)], 12, 0, [5.2, 5.2])
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
    if reuse:
        verify("move carrier component", [cmd("occurrence.move", occurrence="Carrier:1",
                                               translate=[100, 50, 20], move_grounded=True)],
               [(25, 20), (-25, -20)], 16, 90, [8, 5.2], tilt=60, shift=(100, 50, 20))
        verify("reopen moved carrier", [], [(25, 20), (-25, -20)], 16, 90, [8, 5.2],
               tilt=60, shift=(100, 50, 20))


if __name__ == "__main__":
    test()
    test(with_nuts=True)
    test(with_nuts=True, modeled=True, reuse=True)
    test(with_nuts=True, large_moves=True)
    finish()
