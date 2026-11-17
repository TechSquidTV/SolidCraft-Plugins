#!/usr/bin/env python3
"""Check screw seats and actual bodies through sketch, recess and placement edits."""
import json
import math
import os

from harness import TMP, check, finish, inspect, killed, package, picks, run, skip
from headless import Headless


def cmd(command, **params):
    return {"command": command, "params": params}


def near(a, b):
    return abs(a - b) < 1e-4


def same(a, b):
    return all(near(x, y) for x, y in zip(a, b))


def rotate(point, yaw, tilt):
    a, t = math.radians(yaw), math.radians(tilt)
    x, y, z = point
    u, v = x * math.cos(a) - y * math.sin(a), x * math.sin(a) + y * math.cos(a)
    return [u, v * math.cos(t) - z * math.sin(t), v * math.sin(t) + z * math.cos(t)]


def test(kind, modeled=False, reuse=False, layout="plane", active_root=False):
    label = (f"{kind}, {'modeled' if modeled else 'cosmetic'}" + (", reused" if reuse else "")
             + (", layout first" if layout == "first" else ", on face" if layout == "face" else "")
             + (", root active" if active_root else ""))
    code, text, info = inspect("techsquid-screws")
    if code or not info:
        raise SystemExit(f"Build the packages first: {text}")
    saved = os.path.join(TMP, f"screws-{kind}-{modeled}-{reuse}-{layout}-{active_root}.json")
    recessed = kind in ("counterbore", "countersink")
    sink = 3.0 if kind == "counterbore" else 1.86 if kind == "countersink" else 0.0
    rim = 0.2 if kind == "countersink" else 0.0
    yaw, tilt = (35, 25) if reuse and layout != "face" else (0, 0)
    params = {
        "size": "M3", "length": 8, "create": "Screw and hole",
        "hole_type": "Clearance, head recessed" if recessed else "Tapped" if kind == "tapped" else "Clearance",
        "head": "Countersunk" if kind == "countersink" else "Socket head",
        "thread_modeled": modeled,
    }
    setup = [
        cmd("parameters.change", name="thickness", expression="16 mm"),
        cmd("parameters.change", name="yaw", expression=f"{yaw} deg"),
        cmd("parameters.change", name="tilt", expression=f"{tilt} deg"),
        cmd("construct.plane.at_angle", base="XY", axis="Z", angle="yaw", name="Turned base"),
        cmd("construct.plane.at_angle", base="Turned base", axis="X", angle="tilt", name="Plate plane"),
    ]
    plate = [
        cmd("sketch.create", plane="Plate plane", name="Plate profile"),
        cmd("sketch.rectangle.two_point", p0=[-50, -50], p1=[50, 50]),
        cmd("sketch.finish"),
        cmd("solid.extrude", sketch="Plate profile", distance="thickness", body_name="Plate"),
    ]
    layout_steps = [
        cmd("construct.plane.offset", base="Plate plane", offset="thickness", name="Layout plane"),
        cmd("sketch.create", plane={"face": [0, 0, 16]} if layout == "face" else "Layout plane",
            name="Layout", project_edges=False),
        cmd("sketch.point", point=[15, 15], id="p"),
        cmd("sketch.circle.center", center=[-15, -15], radius=2, id="c"),
        cmd("sketch.finish"),
    ]
    setup += layout_steps + plate if layout == "first" else plate + layout_steps
    if reuse:
        setup.insert(0, cmd("component.create", name="Carrier", activate=True, grounded=True))
    if active_root:
        setup.append(cmd("component.activate", component=0))
    point, circle = picks("Layout", "p"), picks("Layout", "c", curve=True)
    for placement in (point, circle) if reuse else (point + circle,):
        setup.append(cmd("plugin.run", path=package("techsquid-screws"), approval=info["approval"],
                         entry="make-screw", params={**params, "placement": placement}))
    code, text, results = run(setup + [cmd("component.list")], save=saved)
    if killed(code):
        skip(label, "host killed while loading plugin")
        return
    check(label + ": build", code == 0, text[-1200:])
    if code:
        return
    if reuse:
        first, second = [r["component"] for r in results[-3:-1]]
        check(label + ": same definition reused", not first["reused"] and second["reused"]
              and first["component"] == second["component"])
    with open(saved) as f:
        doc = json.load(f)
    active = next(c["id"] for c in doc["components"] if c["name"] == "Carrier") if reuse and not active_root else 0
    check(label + ": active component restored", results[-1]["active"] == active)
    holes = [f["id"] for f in doc["features"] if f["type"] == "hole"]
    reads = [cmd("model.faces", body="Plate"), cmd("inspect.measure", summary=True), cmd("joint.inspect")]
    baseline = []

    def verify(stage, edits, xy, thickness, sinks, yaw=0, tilt=0, shift=(0, 0, 0)):
        code, text, results = run(edits + reads, inp=saved, save=saved)
        check(label + f": {stage} rebuild", code == 0, text[-1200:])
        if code:
            return
        faces, measure, joints = results[-3:]
        screws = [b for b in measure["bodies"] if b["name"] != "Plate"]
        seats = joints["joints"]
        check(label + f": {stage} healthy joints", len(seats) == 2
              and all(not j["errors"] and not j["conflicts"] for j in seats))
        check(label + f": {stage} closed bodies", len(screws) == 2 and all(b["closed"] for b in screws))
        normal = rotate([0, 0, 1], yaw, tilt)

        def world(point):
            return [v + d for v, d in zip(rotate(point, yaw, tilt), shift)]

        def on_axis(cylinder, center):
            delta = [a - b for a, b in zip(cylinder["origin"], center)]
            along = sum(a * b for a, b in zip(delta, normal))
            return same(delta, [along * v for v in normal])

        centers = [world([x, y, thickness - depth]) for (x, y), depth in zip(xy, sinks)]
        if not baseline:
            # Check axial and radial mass-center offsets independently of the
            # reported joint frame. The joint's fixed turn reference need not
            # follow sketch yaw, and a modeled helix is slightly asymmetric.
            for center in centers:
                b = min(screws, key=lambda b: math.dist(b["center_of_mass"], center))
                delta = [a - b for a, b in zip(b["center_of_mass"], center)]
                axial = sum(a * b for a, b in zip(delta, normal))
                radial = math.sqrt(max(0, sum(v * v for v in delta) - axial * axial))
                baseline.append((axial, radial, b["volume_mm3"]))
        for i, ((x, y), depth, center) in enumerate(zip(xy, sinks, centers)):
            target = world([x, y, thickness - depth + rim])
            matching = [j for j in seats if j["a"]["resolves"] and same(j["a"]["frame"]["origin"], target)]
            def referenced(snap):
                if kind == "tapped":
                    return snap["type"] == "axis" and bool(snap.get("axis"))
                return snap["type"] == "circle_center" and bool(snap.get("face")) and bool(snap.get("end"))
            check(label + f": {stage} seat {i + 1} follows hole", len(matching) == 1
                  and referenced(matching[0]["a"]["snap"])
                  and all(matching[0][side]["resolves"]
                          and same(matching[0][side]["frame"]["origin"], target)
                          and same(matching[0][side]["frame"]["z"], normal) for side in ("a", "b")))
            axial, radial, volume = baseline[i]
            def seated(body):
                delta = [a - b for a, b in zip(body["center_of_mass"], center)]
                along = sum(a * b for a, b in zip(delta, normal))
                across = math.sqrt(max(0, sum(v * v for v in delta) - along * along))
                return near(along, axial) and near(across, radial) and near(body["volume_mm3"], volume)
            check(label + f": {stage} screw {i + 1} actually moves", any(seated(b) for b in screws))
            check(label + f": {stage} bore {i + 1} follows", any(
                f.get("cylinder") and on_axis(f["cylinder"], center) for f in faces["faces"]))
            if kind == "counterbore":
                check(label + f": {stage} floor {i + 1} follows depth", any(
                    f.get("plane_normal") and same(f["plane_normal"], normal)
                    and same(f["centroid"], center) for f in faces["faces"]))

    xy, sinks = [(15, 15), (-15, -15)], [sink, sink]
    verify("initial", [], xy, 16, sinks, yaw, tilt)
    xy[0] = (25, 20)
    verify("move point", [cmd("sketch.edit", sketch="Layout"), cmd("sketch.move_point", point="p", to=xy[0]),
                          cmd("sketch.finish")], xy, 16, sinks, yaw, tilt)
    xy[1] = (-25, -20)
    verify("move circle", [cmd("sketch.edit", sketch="Layout"), cmd("sketch.move_point", point="c.center", to=xy[1]),
                           cmd("sketch.finish")], xy, 16, sinks, yaw, tilt)
    verify("raise plane", [cmd("parameters.change", name="thickness", expression="20 mm")], xy, 20, sinks, yaw, tilt)
    if recessed:
        key, original = ("hole.cb_depth", 3) if kind == "counterbore" else ("hole.cs_diameter", 6.72)
        # Increasing a 90-degree countersink's diameter by 2 deepens its seat by 1.
        scale = 1 if kind == "counterbore" else 2
        sinks = [sink + 1, sink if reuse else sink + 1]
        verify("edit recess", [cmd("parameters.change", name="recess", expression=f"{original + scale} mm"),
                               cmd("timeline.edit", feature=holes[0], set={key: "recess"})], xy, 20, sinks, yaw, tilt)
        sinks = [sink + 2, sink if reuse else sink + 2]
        verify("drive recess", [cmd("parameters.change", name="recess", expression=f"{original + 2 * scale} mm")],
               xy, 20, sinks, yaw, tilt)
    verify("rotate and tilt", [cmd("parameters.change", name="yaw", expression="90 deg"),
                               cmd("parameters.change", name="tilt", expression="60 deg")], xy, 20, sinks, 90, 60)
    verify("reopen", [], xy, 20, sinks, 90, 60)
    if reuse:
        verify("move carrier", [cmd("occurrence.move", occurrence="Carrier:1", translate=[100, 50, 20], move_grounded=True)],
               xy, 20, sinks, 90, 60, (100, 50, 20))
        verify("reopen moved carrier", [], xy, 20, sinks, 90, 60, (100, 50, 20))


def test_thin_countersunk_seat():
    """Filter by the cone/bore junction, even when the screw origin is below the plate."""
    with Headless() as host:
        host.command('solid.box', length=30, width=30, height=1.75,
                     corner=[-15, -15, 0], body_name='Plate')
        sketch = host.command('sketch.create', plane={'face': [0, 0, 1.75]}, name='Mount')['sketch']
        host.command('sketch.point', point=[0, 0], id='p')
        host.command('sketch.finish')
        approval = host.command('plugin.inspect', path=package('techsquid-screws'))['approval']
        result = host.command('plugin.run', path=package('techsquid-screws'), approval=approval,
                              entry='make-screw', allow_error=True, params={
                                  'size': 'M3', 'length': 8, 'create': 'Screw and hole',
                                  'hole_type': 'Clearance, head recessed', 'head': 'Countersunk',
                                  'thread_modeled': False, 'placement': picks(sketch, 'p')})
        check('thin countersunk seat builds', 'error' not in result, str(result))
        if 'error' in result:
            return
        check('thin countersunk fixture puts the screw origin below the plate', result['result']['at'][2] < 0)
        joints = host.command('joint.inspect')['joints']
        check('thin countersunk seat binds the junction inside the plate', len(joints) == 1
              and not joints[0]['errors'] and not joints[0]['conflicts']
              and all(same(joints[0][side]['frame']['origin'], [0, 0, 0.09]) for side in ['a', 'b']))


def test_copied_tapped_seat_is_refused():
    """Do not turn the existing unsupported case into a joint on the wrong copy."""
    with Headless() as host:
        carrier = host.command('component.create', name='Carrier', activate=True, grounded=False)
        host.command('solid.box', length=40, width=40, height=12,
                     corner=[-20, -20, 0], body_name='Plate')
        sketch = host.command('sketch.create', plane={'face': [0, 0, 12]}, name='Mount')['sketch']
        host.command('sketch.point', point=[0, 0], id='p')
        host.command('sketch.finish')
        copied = host.command('occurrence.copy', component=carrier['component'],
                              parent=0, translate=[100, 0, 0])['occurrence']
        host.command('component.activate', component=carrier['component'], occurrence=copied, path=[copied])
        approval = host.command('plugin.inspect', path=package('techsquid-screws'))['approval']
        before = host.command('document.inspect', measure=False), host.command('component.list')
        result = host.command('plugin.run', path=package('techsquid-screws'), approval=approval,
                              entry='make-screw', allow_error=True, params={
                                  'size': 'M4', 'length': 8, 'create': 'Screw and hole',
                                  'hole_type': 'Tapped', 'thread_modeled': False,
                                  'placement': picks(sketch, 'p')})
        check('copied tapped seat refuses an unsupported axis placement',
              'complete hole and seating face' in result.get('error', ''), str(result))
        check('copied tapped seat failure preserves geometry and exact activation',
              (host.command('document.inspect', measure=False), host.command('component.list')) == before)


if __name__ == "__main__":
    test_thin_countersunk_seat()
    test_copied_tapped_seat_is_refused()
    for kind in ("clearance", "counterbore", "countersink", "tapped"):
        test(kind)
    test("counterbore", modeled=True, reuse=True)
    test("countersink", modeled=True)
    test("tapped", modeled=True)
    test("tapped", modeled=True, reuse=True)
    test("tapped", layout="first")
    test("tapped", modeled=True, reuse=True, layout="first", active_root=True)
    test("tapped", modeled=True, reuse=True, layout="face", active_root=True)
    finish()
