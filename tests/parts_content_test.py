#!/usr/bin/env python3
"""Run the Rust content inspector against native CAD mutations and saved designs.

Build the driver first: cargo build -p plugin-kit --example inspect_content
"""
import json
import os
import subprocess

from harness import CLI, ROOT, TMP, check, finish, run

DRIVER = os.path.join(os.environ.get("CARGO_TARGET_DIR", os.path.join(ROOT, "target")),
                      "debug", "examples", "inspect_content")


def cmd(command, **params):
    return {"command": command, "params": params}


def snapshot(path, component):
    result = subprocess.run([DRIVER, CLI, path, str(component)], capture_output=True, text=True)
    return (json.loads(result.stdout) if result.returncode == 0 else None, result.stderr)


base = os.path.join(TMP, "content-base.sc")
code, text, created = run([
    cmd("component.create", name="Part", activate=True, grounded=False),
    cmd("solid.cylinder", radius=8, height=5, name="Outside", body_name="Probe body"),
    cmd("solid.cylinder", radius=2.5, height=5, name="Bore", operation="cut", targets=["Probe body"]),
], save=base)
check("content fixture builds", code == 0, text[-1000:] if code else "")
if code:
    finish()
component, occurrence = created[0]["component"], created[0]["occurrence"]
outer, bore = created[1]["feature"], created[2]["feature"]
original, error = snapshot(base, component)
check("native contents are inspectable", original is not None, error[-1000:])
if original is None:
    finish()
check("simple part inspection leaves creation budget", original["reads"] < 20, str(original["reads"]))


def compare(label, steps, same=False, expected_error=None, source=base, target=component, reference=original):
    path = os.path.join(TMP, f"{label.replace(' ', '-')}.sc")
    code, text, results = run(steps, inp=source, save=path)
    check(f"{label}: edit succeeds", code == 0, text[-1000:] if code else "")
    if code:
        return
    changed, error = snapshot(path, target)
    if expected_error:
        check(f"{label}: unverifiable content is refused", changed is None and expected_error in error, error[-500:])
    else:
        check(f"{label}: {'preserves' if same else 'changes'} snapshot", changed is not None
              and (changed["snapshot"] == reference["snapshot"]) == same, error[-1000:])
    return results


compare("unchanged reopen", [], same=True)
compare("rename component", [cmd("component.rename", component=component, name="My part")], same=True)
compare("rename feature", [cmd("timeline.rename", feature=bore, name="Bore renamed")], same=True)
compare("duplicate name in root", [cmd("component.activate", component=0),
        cmd("solid.cylinder", radius=1, height=2, base=[100, 0, 0], name="Outside", body_name="Other body")], same=True)
compare("move placement", [cmd("occurrence.move", occurrence=occurrence, translate=[20, 7, 9])], same=True)
compare("self metadata", [cmd("component.metadata.set", component=component,
        values={"org.techsquid.parts": {"identity": "example", "snapshot": "ignored"}})], same=True)
compare("unrelated parameter", [cmd("parameters.add", name="radius", expression=17)], same=True)
compare("body-name parameter", [cmd("parameters.add", name="Probe", expression=17)], same=True)
compare("BOM metadata", [cmd("component.metadata.set", component=component, part_number="Custom")])
compare("relocate same-volume bore", [cmd("timeline.edit", feature=bore, set={"base": [1, 0, 0]})])
compare("material", [cmd("material.assign", bodies=["Probe body"], material="Steel")])
compare("appearance", [cmd("appearance.assign", bodies=["Probe body"], color=[255, 0, 0])])
compare("owned cut", [cmd("component.activate", component=component),
        cmd("solid.cylinder", radius=1, height=5, base=[5, 0, 0], operation="cut", targets=["Probe body"])])
compare("external cut", [cmd("component.activate", component=0),
        cmd("solid.cylinder", radius=1, height=5, base=[5, 0, 0], operation="cut", targets=["Probe body"])],
        expected_error="outside its definition")
compare("suppression", [cmd("timeline.suppress", feature=bore, suppressed=True)], expected_error="suppressed")

sketch_path = os.path.join(TMP, "sketch-part.sc")
code, text, made = run([
    cmd("parameters.add", name="sketch_base_radius", expression=2),
    cmd("parameters.add", name="sketch_radius", expression="2 * sketch_base_radius"),
    cmd("component.create", name="Sketch part", activate=True, grounded=False),
    cmd("sketch.create", name="Profile", plane="XY"),
    cmd("sketch.circle.center", center=[2, 3], radius="sketch_radius", id="c"),
    cmd("sketch.finish"),
    cmd("solid.extrude", distance=3, body_name="Sketch body"),
], save=sketch_path)
check("sketch fixture builds", code == 0, text[-1000:] if code else "")
if not code:
    sketch_component, sketch_occurrence = made[2]["component"], made[2]["occurrence"]
    sketch_id = made[3]["sketch"]
    sk, error = snapshot(sketch_path, sketch_component)
    check("full sketch contents are inspectable", sk is not None, error[-1000:])
    if sk:
        compare("sketch part placement", [cmd("occurrence.move", occurrence=sketch_occurrence, translate=[20, 30, 40])],
                same=True, source=sketch_path, target=sketch_component, reference=sk)
        edited = compare("sketch geometry edit", [cmd("sketch.edit", sketch=sketch_id),
                cmd("sketch.move_point", point="c.center", to=[1, 2]), cmd("sketch.finish"), cmd("sketch.inspect", sketch=sketch_id)],
                source=sketch_path, target=sketch_component, reference=sk)
        if edited:
            check("sketch edit actually changed local geometry", next(p for p in edited[-1]["points"] if p["id"] == "c.center")["at"] == [1, 2])
        # These parameters drive only a sketch constraint, never a solid's
        # expression. Equal-value edits also require dependency inspection to
        # change the snapshot: evaluated geometry alone cannot catch them.
        for label, name, expression in [
            ("sketch-only direct parameter", "sketch_radius", "3 * sketch_base_radius"),
            ("sketch-only transitive parameter", "sketch_base_radius", 3),
            ("sketch-only equal-value direct expression", "sketch_radius", "sketch_base_radius + sketch_base_radius"),
            ("sketch-only equal-value transitive expression", "sketch_base_radius", "1 + 1"),
        ]:
            compare(label, [cmd("parameters.change", name=name, expression=expression)],
                    source=sketch_path, target=sketch_component, reference=sk)

# Parameters must come from actual native dependencies, including indirect feature
# inputs and placement expressions. Administrative parameter edits are not geometry.
parametric = os.path.join(TMP, "parametric-part.sc")
code, text, made = run([
    cmd("parameters.add", name="base_radius", expression=3),
    cmd("parameters.add", name="outside", expression="2 * base_radius"),
    cmd("parameters.add", name="shift", expression=1),
    cmd("component.create", name="Parametric part", activate=True, grounded=False),
    cmd("solid.cylinder", radius="outside", height=5, base=["shift", 0, 0], body_name="Parametric body"),
], save=parametric)
check("parameter fixture builds", code == 0, text[-1000:] if code else "")
if not code:
    param_component = made[3]["component"]
    param_snapshot, error = snapshot(parametric, param_component)
    check("parameter dependencies are inspectable", param_snapshot is not None, error[-1000:])
    if param_snapshot:
        cases = [
            ("direct parameter", [cmd("parameters.change", name="outside", expression="3 * base_radius")], False),
            ("transitive parameter", [cmd("parameters.change", name="base_radius", expression=4)], False),
            ("placement parameter", [cmd("parameters.change", name="shift", expression=2)], False),
            ("parameter comment", [cmd("parameters.comment", name="outside", comment="A new note")], True),
            ("parameter favourite", [cmd("parameters.favorite", name="outside", favorite=True)], True),
            ("unrelated dependent", [cmd("parameters.add", name="elsewhere", expression="outside + 1")], True),
        ]
        for label, steps, same in cases:
            compare(label, steps, same=same, source=parametric, target=param_component, reference=param_snapshot)


assembly = os.path.join(TMP, "assembly.sc")
code, text, made = run([cmd("component.create", name="Assembly", activate=True, grounded=False)], inp=base, save=assembly)
check("assembly fixture builds", code == 0, text[-1000:] if code else "")
if not code:
    parent, parent_occurrence = made[0]["component"], made[0]["occurrence"]
    code, text, made = run([
        cmd("occurrence.copy", component=component, parent=parent),
        cmd("occurrence.copy", component=component, parent=parent),
    ], inp=assembly, save=assembly)
    check("repeated assembly children build", code == 0, text[-1000:] if code else "")
    if not code:
        child = made[1]["occurrence"]
        parent_snapshot, error = snapshot(assembly, parent)
        check("recursive assembly contents are inspectable", parent_snapshot is not None, error[-1000:])
        if parent_snapshot:
            check("shared child is inspected once", parent_snapshot["reads"] < 24, str(parent_snapshot["reads"]))
            compare("parent placement", [cmd("occurrence.move", occurrence=parent_occurrence, translate=[0, 0, 50])],
                    same=True, source=assembly, target=parent, reference=parent_snapshot)
            compare("child transform", [cmd("occurrence.move", occurrence=child, translate=[0, 0, 8])],
                    source=assembly, target=parent, reference=parent_snapshot)
            compare("child geometry", [cmd("timeline.edit", feature=bore, set={"base": [1, 0, 0]})],
                    source=assembly, target=parent, reference=parent_snapshot)

finish()
