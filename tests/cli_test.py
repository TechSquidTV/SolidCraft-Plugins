#!/usr/bin/env python3
"""Check the built plugins through solidcraft-cli.

    SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/cli_test.py

The packages are dist/techsquid-screws/, dist/techsquid-inserts/, and
dist/techsquid-nuts/. Needs a solidcraft-cli with the plugin host (SolidCraft
0.41 or newer).

plugin.inspect must accept each package and list its entry. Each run must build
one closed body of the measured size: an M3 x 12 socket head screw 5.5 across and
15 tall, an M3 x 5.7 insert 4.6 across and 5.7 long, an M6 nut 10 across the flats
and 5.2 high. On the signed macOS build of SolidCraft 0.41.0, loading a component
kills the CLI with SIGKILL (hardened runtime, no JIT entitlement), and that case is
reported and skipped.
"""
from harness import check, finish, inspect, killed, package, run, skip

TOL = 1e-3

PACKAGES = [
    (
        "techsquid-screws",
        "make-screw",
        {"size": "M3", "length": 12, "head": "Socket head", "drive": "Hex socket"},
        "builds an M3 x 12 socket head screw",
        (5.5, 15.0),
    ),
    (
        "techsquid-inserts",
        "make-insert",
        {"size": "M3 × 5.7", "create": "Insert only"},
        "builds the M3 x 5.7 insert",
        (4.6, 5.7),
    ),
    (
        "techsquid-nuts",
        "make-nut",
        {"size": "M6"},
        "builds the M6 nut",
        (10.0, 5.2),
    ),
]


def size_of(bbox):
    """(across, height): the narrower of the two sides in X and Y, which is the width
    across the flats of a hexagon, and the height along Z."""
    dx, dy, dz = (bbox["max"][i] - bbox["min"][i] for i in range(3))
    return min(dx, dy), dz


screws_ran = False


for plugin_id, entry, params, label, (across, height) in PACKAGES:
    path = package(plugin_id)
    code, text, info = inspect(plugin_id)
    check(f"{plugin_id}: inspect", code == 0 and info, text[-500:])
    if code != 0 or not info:
        continue
    approval = info.get("approval")
    check(f"{plugin_id}: approval", isinstance(approval, str) and len(approval) > 8)
    names = [e.get("name") for e in info["manifest"]["runtime"]["entries"]]
    commands = info["manifest"]["runtime"]["commands"]
    check(f"{plugin_id}: lists {entry}", names == [entry], str(names))
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
    got = size_of(bodies[0]["bbox"]) if len(bodies) == 1 else None
    check(
        f"{plugin_id}: {label}",
        code == 0
        and got is not None
        and bodies[0].get("closed")
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
