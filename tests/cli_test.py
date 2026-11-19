#!/usr/bin/env python3
"""Check the built plugins through solidcraft-cli.

    SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/cli_test.py

Every folder under plugins/ is checked from its package in dist/<id>/, and each one
needs a case in CASES, so a new plugin cannot be left out unnoticed. Needs a
solidcraft-cli with the plugin host (SolidCraft 0.41 or newer).

plugin.inspect must accept each package and list its entries. Each run must build
the expected solid/surface bodies and overall size: an M3 x 12 socket head screw 5.5 across and
15 tall, an M3 x 5.7 insert 4.6 across and 5.7 long, an M6 nut 10 across the flats
and 5.2 high, a V-slot 2040 rail 20 across and 120 long. On the signed macOS build of SolidCraft 0.41.0, loading a component
kills the CLI with SIGKILL (hardened runtime, no JIT entitlement), and that case is
reported and skipped.
"""
import json
from pathlib import Path

from harness import ROOT, check, finish, inspect, killed, package, plugin_ids, run, skip

TOL = 1e-3

# Plugin id: (entry, params, what the run builds, its size as (across, height)).
CASES = {
    "techsquid-bearings": (
        "make-bearing", {"size": "625", "closure": "Sealed"}, "builds a sealed 625 bearing", (16.0, 5.0),
    ),
    "techsquid-v-wheels": (
        "make-v-wheel", {"size": "Solid V Wheel", "assembled": False}, "builds a bare Solid V wheel", (23.9, 10.2),
    ),
    "techsquid-screws": (
        "make-screw",
        {"size": "M3", "length": 12, "head": "Socket head", "drive": "Hex socket"},
        "builds an M3 x 12 socket head screw",
        (5.5, 15.0),
    ),
    "techsquid-inserts": (
        "make-insert",
        {"size": "M3 × 5.7", "create": "Insert only"},
        "builds the M3 x 5.7 insert",
        (4.6, 5.7),
    ),
    "techsquid-nuts": (
        "make-nut",
        {"size": "M6"},
        "builds the M6 nut",
        (10.0, 5.2),
    ),
    "techsquid-extrusions": (
        "make-extrusion",
        {"slot": "V-slot", "size": "2040", "length": 120},
        "builds a 120 mm V-slot 2040",
        (20.0, 120.0),
    ),
}


# The entries a plugin lists beside the one its case runs.
OTHER_ENTRIES = {
    "techsquid-bearings": [
        "make-csk", "make-bearing-by-dimensions", "make-csk-by-dimensions",
    ],
    "techsquid-extrusions": ["make-t-nut"],
}

# Bearings have two solid ring envelopes and two annular closure surfaces.
BODY_COUNTS = {"techsquid-bearings": (2, 2)}


def size_of(bbox):
    """(across, height): the narrower of the two sides in X and Y, which is the width
    across the flats of a hexagon, and the height along Z."""
    dx, dy, dz = (bbox["max"][i] - bbox["min"][i] for i in range(3))
    return min(dx, dy), dz


screws_ran = False


PLUGINS = plugin_ids()
for plugin_id in sorted(set(CASES) - set(PLUGINS)):
    check(f"{plugin_id}: has a plugin for its case", False, "no plugins/<id>/plugin.json")

for plugin_id in PLUGINS:
    if plugin_id not in CASES:
        check(f"{plugin_id}: has a case in CASES", False, "add one to tests/cli_test.py")
        continue
    entry, params, label, (across, height) = CASES[plugin_id]
    path = package(plugin_id)
    manifest = (Path(path) / "plugin.json").read_bytes()
    source = json.loads((Path(ROOT) / "plugins" / plugin_id / "plugin.json").read_text())
    check(f"{plugin_id}: packaged manifest preserves every source field", json.loads(manifest) == source)
    check(f"{plugin_id}: packaged manifest fits the host limit", len(manifest) <= 16 * 1024)
    code, text, info = inspect(plugin_id)
    check(f"{plugin_id}: inspect", code == 0 and info, text[-500:])
    if code != 0 or not info:
        continue
    approval = info.get("approval")
    check(f"{plugin_id}: approval", isinstance(approval, str) and len(approval) > 8)
    names = [e.get("name") for e in info["manifest"]["runtime"]["entries"]]
    commands = info["manifest"]["runtime"]["commands"]
    entries = [entry] + OTHER_ENTRIES.get(plugin_id, [])
    check(f"{plugin_id}: lists {', '.join(entries)}", names == entries, str(names))
    check(f"{plugin_id}: names its commands", isinstance(commands, list) and len(commands) > 0, str(commands))

    code, text, results = run([
        {
            "command": "plugin.run",
            "params": {"path": path, "approval": approval, "entry": entry, "params": params},
        },
        {"command": "inspect.measure", "params": {}},
    ])
    if killed(code):
        skip(f"{plugin_id}: {label}", f"plugin.run killed solidcraft-cli (exit {code})")
        continue
    bodies = results[1]["bodies"] if results else []
    got = size_of({
        "min": [min(b["bbox"]["min"][i] for b in bodies) for i in range(3)],
        "max": [max(b["bbox"]["max"][i] for b in bodies) for i in range(3)],
    }) if bodies else None
    counts = (sum(b.get("closed") is True for b in bodies), sum(b.get("closed") is False for b in bodies))
    check(
        f"{plugin_id}: {label}",
        code == 0
        and got is not None
        and counts == BODY_COUNTS.get(plugin_id, (1, 0))
        and abs(got[0] - across) < TOL
        and abs(got[1] - height) < TOL,
        f"measured {got}, expected {(across, height)}" if code == 0 else text[-800:],
    )
    if plugin_id == "techsquid-screws":
        screws_ran = True


if screws_ran:
    path = package("techsquid-screws")
    _, _, info = inspect("techsquid-screws")
    approval = info.get("approval") if info else None
    code, text, _ = run([{
        "command": "plugin.run",
        "params": {
            "path": path,
            "approval": approval,
            "entry": "make-screw",
            "params": {"head": "wing"},
        },
    }])
    check(
        "techsquid-screws: unknown head is invalid input",
        code != 0 and "head" in text and "not implemented" not in text,
        text[-800:],
    )
else:
    skip("techsquid-screws: unknown head is invalid input", "the screw run did not finish")

finish()
