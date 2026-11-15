#!/usr/bin/env python3
"""Build every example in examples/ with the built plugins and measure the result.

    SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/examples_test.py [--update] [example.steps.json ...]

An example is `examples/<name>.steps.json`:
{"description", "steps": [{"caption", "command", "params"}], "expect"}.
A plugin step is a plugin.run whose path is relative to the repository (dist/<id>) and
whose approval is "@approval". This test inspects each package and fills both in,
because the approval changes with every build.

`expect` says what the finished design must measure:

    bodies     how many bodies, all of them closed
    bbox       {min, max} of the whole design
    corners    [[x, y], ...] where the fasteners stand
    parts      [{body, bbox}] the named bodies and their boxes
    holes      [{label, body, radius, floor_z}] one such hole on every corner: a
               cylindrical face of that radius on the corner's axis, starting at floor_z
    fasteners  [{label, diameter, z: [low, high]}] one such body on every corner: its box
               is `diameter` across, centred on the corner, from low to high
    placed     [{label, bbox, volume_mm3}] one body with exactly that box and, to one
               part in ten thousand, that volume. This is for placements that share a
               name, such as the rails of a frame
    overlaps   {pairs, volume_mm3: [least, most]} exactly that many pairs of bodies
               overlap, each by a volume in that range, to within 0.1 mm3. Without it
               nothing is checked for overlap. This is for screws in tapped holes: two
               threads cut out of step overlap a little
    nut_seats  [{body, at: [x, y], floor_z, bolt_radius}] a pocket floor and bolt-hole
               axis at each center, with a healthy rigid joint holding its nut there

Only `bodies` and `bbox` are required.

The saved design must also reopen with the same bodies. Each example also keeps the
design it builds next to it, `examples/<name>.json`, which is what opens in SolidCraft:
the steps file is not a design. That design must match a fresh build, body for body,
with the same names and volumes. `--update` writes it from the fresh build instead,
after a plugin or an example changes.

On the signed macOS build of
SolidCraft 0.41.0 a plugin run kills the CLI with SIGKILL; that example is reported and
skipped.
"""
import glob, json, os, shutil, sys

from harness import ROOT, TMP, check, finish, inspect, killed, package, run, skip

TOL = 1e-3


def near(a, b):
    return abs(a - b) < TOL


def box_is(bbox, want):
    return all(near(bbox[k][i], want[k][i]) for k in ("min", "max") for i in range(3))


def same_bodies(a, b):
    """Saving must preserve placement as well as shape and body names."""
    kept = {body["name"]: body for body in a}
    fresh = {body["name"]: body for body in b}
    return sorted(kept) == sorted(fresh) and all(
        kept[n].get("closed") and fresh[n].get("closed")
        and abs(kept[n]["volume_mm3"] - fresh[n]["volume_mm3"]) <= 1e-6 * max(1.0, abs(fresh[n]["volume_mm3"]))
        and box_is(kept[n]["bbox"], fresh[n]["bbox"]) for n in fresh)


def whole(bodies):
    boxes = [b["bbox"] for b in bodies]
    return {
        "min": [min(b["min"][i] for b in boxes) for i in range(3)],
        "max": [max(b["max"][i] for b in boxes) for i in range(3)],
    }


APPROVALS = {}


def approval(plugin_id):
    if plugin_id not in APPROVALS:
        code, text, info = inspect(plugin_id)
        if code != 0 or not info:
            sys.exit(f"plugin.inspect of dist/{plugin_id} failed, run ./build.sh first: {text[-500:]}")
        APPROVALS[plugin_id] = info["approval"]
    return APPROVALS[plugin_id]


def commands(doc):
    """The example's steps as a script, with each plugin step's path and approval filled in."""
    steps = []
    for step in doc["steps"]:
        params = json.loads(json.dumps(step["params"]))
        if step["command"] == "plugin.run":
            plugin_id = os.path.basename(params["path"])
            params["path"] = package(plugin_id)
            params["approval"] = approval(plugin_id)
        steps.append({"command": step["command"], "params": params})
    return steps


def faces(result):
    return result["faces"] if isinstance(result, dict) else result


def example(path):
    name = os.path.relpath(path, ROOT)
    with open(path) as f:
        doc = json.load(f)
    expect = doc["expect"]
    steps = commands(doc)
    seats = expect.get("nut_seats", [])
    hole_bodies = sorted({h["body"] for h in expect.get("holes", []) + seats})
    measure_at = len(steps)
    steps.append({"command": "inspect.measure", "params": {}})
    steps += [{"command": "model.faces", "params": {"body": b}} for b in hole_bodies]
    joint_at = len(steps)
    if seats:
        steps.append({"command": "joint.inspect", "params": {}})
    if "overlaps" in expect:
        steps.append({"command": "inspect.interference", "params": {}})
    design = path[: -len(".steps.json")] + ".json"
    # A .json path saves SolidCraft's plain design format, the one kept in examples/.
    saved = os.path.join(TMP, os.path.basename(design))

    code, text, results = run(steps, save=saved)
    if killed(code):
        skip(name, f"a plugin run killed solidcraft-cli (exit {code})")
        return
    check(f"{name}: every step runs", code == 0 and results is not None, text[-1500:])
    if code != 0 or results is None:
        return
    bodies = results[measure_at]["bodies"]
    by_name = {b["name"]: b for b in bodies}
    body_faces = {
        b: faces(results[measure_at + 1 + i])
        for i, b in enumerate(hole_bodies)
    }
    cylinders = {b: [f["cylinder"] for f in fs if f.get("cylinder")] for b, fs in body_faces.items()}

    if seats:
        joints = results[joint_at]["joints"]
        check(f"{name}: one healthy rigid joint per pocketed nut", len(joints) == len(seats)
              and all(j["type"] == "rigid" and not j["errors"] and not j["conflicts"] for j in joints))
        for seat in seats:
            x, y = seat["at"]
            floor = [x, y, seat["floor_z"]]
            at_floor = lambda p: all(near(a, b) for a, b in zip(p, floor))
            check(f"{name}: pocket floor at {floor}", any(
                f.get("plane_normal") and near(f["plane_normal"][2], 1) and at_floor(f["centroid"])
                for f in body_faces[seat["body"]]))
            check(f"{name}: bolt-hole axis at ({x:g}, {y:g})", any(
                near(c["radius"], seat["bolt_radius"]) and near(c["origin"][0], x)
                and near(c["origin"][1], y) and near(abs(c["axis"][2]), 1)
                for c in cylinders[seat["body"]]))
            check(f"{name}: nut joint seated at {floor}", sum(
                all(j[side]["resolves"] and at_floor(j[side]["frame"]["origin"])
                    and all(near(a, b) for a, b in zip(j[side]["frame"]["z"], [0, 0, 1]))
                    for side in ("a", "b")) for j in joints) == 1)

    check(f"{name}: {expect['bodies']} bodies", len(bodies) == expect["bodies"], f"got {len(bodies)}")
    check(f"{name}: every body is closed", all(b.get("closed") for b in bodies))
    check(f"{name}: overall box", box_is(whole(bodies), expect["bbox"]), json.dumps(whole(bodies)))
    for part in expect.get("parts", []):
        body = by_name.get(part["body"])
        check(
            f"{name}: {part['body']} box",
            body is not None and box_is(body["bbox"], part["bbox"]),
            json.dumps(body["bbox"]) if body else "no such body",
        )

    corners = expect.get("corners", [])
    for hole in expect.get("holes", []):
        for x, y in corners:
            found = [
                c for c in cylinders[hole["body"]]
                if near(c["radius"], hole["radius"])
                and near(c["origin"][0], x) and near(c["origin"][1], y)
                and near(c["origin"][2], hole["floor_z"])
                and near(abs(c["axis"][2]), 1)
            ]
            check(f"{name}: {hole['label']} in {hole['body']} at ({x:g}, {y:g})", len(found) == 1, f"got {len(found)}")

    named = {part["body"] for part in expect.get("parts", [])}
    loose = [b for b in bodies if b["name"] not in named]
    for kind in expect.get("fasteners", []):
        r, (low, high) = kind["diameter"] / 2, kind["z"]
        for x, y in corners:
            want = {"min": [x - r, y - r, low], "max": [x + r, y + r, high]}
            found = [b for b in loose if box_is(b["bbox"], want)]
            check(f"{name}: one {kind['label']} at ({x:g}, {y:g})", len(found) == 1, f"got {len(found)}")
    for part in expect.get("placed", []):
        found = [
            b for b in loose
            if box_is(b["bbox"], part["bbox"])
            and abs(b["volume_mm3"] - part["volume_mm3"]) <= 1e-4 * part["volume_mm3"]
        ]
        check(f"{name}: one {part['label']}", len(found) == 1, f"got {len(found)}")
    if "overlaps" in expect:
        want = expect["overlaps"]
        report = results[-1]
        hits = report.get("interferences", [])
        least, most = want["volume_mm3"]
        check(
            f"{name}: {want['pairs']} pairs overlap, each by {least} to {most} mm3",
            len(hits) == want["pairs"] and not report.get("unchecked")
            and all(least - 0.1 < h["volume_mm3"] < most + 0.1 for h in hits),
            f"got {len(hits)}: {sorted({round(h['volume_mm3'], 2) for h in hits})}",
        )
    wanted = len(expect.get("fasteners", [])) * len(corners) + len(expect.get("placed", []))
    check(f"{name}: no other bodies", len(loose) == wanted, f"{len(loose)} beside the named parts, expected {wanted}")

    code, text, results = run([{"command": "inspect.measure", "params": {}}], inp=saved)
    reopened = results[0]["bodies"] if results else []
    check(
        f"{name}: the saved design reopens with the same bodies",
        code == 0 and len(reopened) == len(bodies) and same_bodies(reopened, bodies),
        text[-800:] or f"got {len(reopened)}",
    )

    if UPDATE:
        shutil.copyfile(saved, design)
        print(f"wrote {os.path.relpath(design, ROOT)}")
        return
    code, text, results = run([{"command": "inspect.measure", "params": {}}], inp=design)
    kept = results[0]["bodies"] if code == 0 and results else None
    same = kept is not None and len(kept) == len(reopened) and same_bodies(kept, reopened)
    check(
        f"{name}: {os.path.relpath(design, ROOT)} matches a fresh build",
        same,
        "run with --update to write it" if kept is None else
        f"{len(kept)} bodies kept, {len(reopened)} built; after a plugin or example change, run with --update",
    )


UPDATE = "--update" in sys.argv[1:]
paths = [a for a in sys.argv[1:] if a != "--update"] or sorted(glob.glob(os.path.join(ROOT, "examples", "*.steps.json")))
if not paths:
    sys.exit("no examples found")
for path in paths:
    example(os.path.abspath(path))
finish()
