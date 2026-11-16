#!/usr/bin/env python3
"""Exercise catalog discovery and resolved insertion through the compiled guest.

Run after ./build.sh with an unsigned SOLIDCRAFT_CLI build.
"""
import os

from harness import TMP, check, finish, inspect, killed, package, run, skip

PLUGIN = "techsquid-inserts"


def cmd(command, **params):
    return {"command": command, "params": params}


code, text, info = inspect(PLUGIN)
check("catalog package inspects", code == 0 and info is not None, text[-800:] if code else "")
if code or info is None:
    finish()


def invoke(entry, params):
    return cmd("plugin.run", path=package(PLUGIN), approval=info["approval"], entry=entry, params=params)


saved = os.path.join(TMP, "catalog-context.sc")
code, text, result = run([
    cmd("component.create", name="Carrier", activate=True),
    cmd("solid.cylinder", radius=10, height=5, body_name="Carrier body"),
    cmd("sketch.create", name="Mounts", plane="XY"),
    cmd("sketch.circle.center", center=[3, 2], radius=1),
    cmd("sketch.finish"),
], save=saved)
check("catalog context fixture builds", code == 0, text[-1200:] if code else "")
if code:
    finish()
sketch = result[2]["sketch"]
code, text, result = run([
    cmd("select.set", items=[{"type": "sketch_curve", "sketch": sketch, "id": "c1"}]),
    cmd("component.list"),
    cmd("document.inspect", measure=False),
    invoke("catalog", {"version": 1, "request": {"operation": "list", "limit": 2}}),
    invoke("catalog", {"version": 1, "request": {"operation": "describe", "part": "m3-5.7"}}),
    invoke("catalog", {"version": 1, "revision": "1", "request": {
        "operation": "resolve", "part": "m3-5.7", "configuration": {"detail": "cosmetic"},
    }}),
    cmd("component.list"),
    cmd("document.inspect", measure=False),
], inp=saved)
if killed(code):
    skip("catalog queries", "host killed loading plugin")
    finish()
check("all catalog operations run", code == 0, text[-1200:] if code else "")
if code:
    finish()

before_components, before, listed, described, resolved, after_components, after = result[1:]
check("catalog fixture has an active selection", len(before["selection"]) == 1)
check("catalog discovery keeps component context", before_components == after_components)
check("catalog discovery keeps model, selection and undo history", before == after)
check("catalog pages are bounded", len(listed["result"]["items"]) == 2
      and listed["result"]["total"] == 29 and listed["result"]["next_offset"] == 2)
check("catalog describes exact insert dimensions", {
    d["name"]: d["value"] for d in described["result"]["part"]["dimensions"]
} == {"length": 5.7, "outside_diameter": 4.6, "recommended_hole_diameter": 4.0, "minimum_wall": 1.6})
part = resolved["result"]["part"]
insertion = part["insertion"]
check("resolve advertises the real workflow", insertion["plugin"] == PLUGIN
      and insertion["entry"] == "make-insert")

# The GUI schema intentionally exposes only workflow controls. API preconditions
# arrive in additionalProperties and must survive the host without being stripped.
code, text, inserted = run([
    invoke(insertion["entry"], insertion["params"]),
    cmd("component.list"),
    cmd("inspect.measure", summary=True),
])
check("resolved insertion builds through the real guest", code == 0, text[-1200:] if code else "")
if not code:
    body = inserted[-1]["bodies"][0]
    box = body["bbox"]
    span = [box["max"][i] - box["min"][i] for i in range(3)]
    check("resolved geometry matches sourced dimensions", len(inserted[-1]["bodies"]) == 1
          and all(abs(a - b) < 1e-3 for a, b in zip(span, [4.6, 4.6, 5.7])))

for label, params in [
    ("stale query", {"version": 1, "revision": "old", "request": {"operation": "resolve", "part": "m3-5.7"}}),
    ("unsupported query version", {"version": 2, "request": {"operation": "list"}}),
    ("oversized page", {"version": 1, "request": {"operation": "list", "limit": 65}}),
]:
    code, text, _ = run([invoke("catalog", params)], inp=saved)
    check(f"{label} is refused", code != 0 and not killed(code), text[-800:] if not code else "")

params = dict(insertion["params"], custom_length=7)
code, text, _ = run([invoke("make-insert", params)], inp=saved)
check("tampered resolved insertion is refused by the guest", code != 0
      and "no longer match the resolved part" in text, text[-800:] if not code else "")

finish()
