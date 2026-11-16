#!/usr/bin/env python3
"""Verify host evidence needed by the parts library's edited-definition check.

The fixture is a generic hollow cylinder, not a catalog bearing model. These
checks establish inspection coverage; they do not assert that reuse is implemented.
"""
import os

from harness import TMP, check, finish, run


def cmd(command, **params):
    return {"command": command, "params": params}


saved = os.path.join(TMP, "parts-inspection.sc")
code, text, made = run([
    cmd("component.create", name="Inspection fixture", activate=True, grounded=False),
    cmd("solid.cylinder", radius=8, height=5, body_name="Probe body", name="Outside"),
    cmd("solid.cylinder", radius=2.5, height=5, operation="cut", targets=["Probe body"], name="Bore"),
    cmd("component.list"),
    cmd("document.inspect", measure=False),
    cmd("model.body_features"),
    cmd("inspect.measure", summary=True),
], save=saved)
check("inspection fixture builds", code == 0, text[-1000:] if code else "")
if code:
    finish()

component = made[0]["component"]
outer, bore = made[1]["feature"], made[2]["feature"]
listed, document, history, measured = made[3:]
owned = next(c for c in listed["components"] if c["id"] == component)
check("body history includes creation and its own cut", [f["id"] for f in history["bodies"][0]["features"]] == [outer, bore])

code, text, external = run([
    cmd("component.activate", component=0),
    cmd("solid.cylinder", radius=1, height=5, base=[5, 0, 0], operation="cut", targets=["Probe body"], name="External cut"),
    cmd("component.list"),
    cmd("document.inspect", measure=False),
    cmd("model.body_features"),
], inp=saved)
check("external component can modify the fixture", code == 0, text[-1000:] if code else "")
if not code:
    changed = next(c for c in external[2]["components"] if c["id"] == component)
    check("owned-feature listing alone misses external edits", changed["features"] == owned["features"])
    check("creator ID alone misses external edits", external[3]["bodies"][0]["feature"] == document["bodies"][0]["feature"])
    check("actual body history exposes the external edit", external[1]["feature"] in
          [f["id"] for f in external[4]["bodies"][0]["features"]])

code, text, changed = run([
    cmd("timeline.inspect", feature=bore),
    cmd("timeline.edit", feature=bore, set={"base": [1, 0, 0]}),
    cmd("timeline.inspect", feature=bore),
    cmd("model.body_features"),
    cmd("inspect.measure", summary=True),
], inp=saved)
check("owned cut can change while retaining its identity", code == 0, text[-1000:] if code else "")
if not code:
    before, after = measured["bodies"][0], changed[4]["bodies"][0]
    check("volume and bounds miss a relocated bore", abs(before["volume_mm3"] - after["volume_mm3"]) < 1e-6
          and before["bbox"] == after["bbox"])
    check("body history IDs alone miss feature edits", changed[3] == history)
    check("feature definition reveals the relocated bore", changed[0]["definition"] != changed[2]["definition"])

code, text, copied = run([
    cmd("component.create", name="Parent assembly", activate=True, grounded=False),
], inp=saved, save=os.path.join(TMP, "parent.sc"))
check("parent assembly fixture builds", code == 0, text[-1000:] if code else "")
if not code:
    parent = copied[0]["component"]
    code, text, nested = run([
        cmd("occurrence.copy", component=component, parent=parent),
        cmd("occurrence.copy", component=component, parent=parent),
        cmd("component.list"),
    ], inp=os.path.join(TMP, "parent.sc"), save=os.path.join(TMP, "nested.sc"))
    check("one definition supports two assembly children", code == 0, text[-1000:] if code else "")
    if not code:
        parent_before = next(c for c in nested[-1]["components"] if c["id"] == parent)
        check("children expose definition IDs and local transforms", len(parent_before["children"]) == 2
              and all(c["component"] == component and "transform" in c for c in parent_before["children"]))
        code, text, reopened = run([cmd("component.list")], inp=os.path.join(TMP, "nested.sc"))
        check("nested definitions survive reopening", code == 0 and
              next(c for c in reopened[0]["components"] if c["id"] == parent)["children"] == parent_before["children"],
              text[-1000:] if code else "")

finish()
