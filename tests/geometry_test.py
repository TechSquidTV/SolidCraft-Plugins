#!/usr/bin/env python3
"""Check what the built plugins actually cut, by volume, through solidcraft-cli.

    SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/geometry_test.py

Needs the packages from ./build.sh and a solidcraft-cli that can run a plugin. Each
case builds a plate with a sketch of two circles on its top face, runs one plugin on
those circles, and compares the plate's volume with the volume worked out by hand
here. A second body under the plate must stay untouched. Head shapes and drive
recesses are checked on a loose screw the same way.

The signed macOS build of SolidCraft 0.41.0 cannot load a component (hardened
runtime, no JIT entitlement) and kills the CLI with SIGKILL. A case whose own run was
killed that way is reported as SKIP, as in cli_test.py; any other failure is a FAIL.
"""
import functools, json, math, os, subprocess, sys

from harness import CLI, ROOT, TMP, check, finish, killed, last_line, skip

PI = math.pi


def run(steps):
    """Run the steps: (exit code, output text, the raw results or None)."""
    script, out = os.path.join(TMP, "s.json"), os.path.join(TMP, "o.json")
    if os.path.exists(out):
        os.remove(out)
    with open(script, "w") as f:
        json.dump(steps, f)
    p = subprocess.run([CLI, "run", script, "--results", "--quiet", "--json-out", out],
                       capture_output=True, text=True)
    text = p.stdout + p.stderr
    if not os.path.exists(out):
        return p.returncode, text, None
    with open(out) as f:
        return p.returncode, text, json.load(f)


def body(item):
    return item.get("result", item)


@functools.cache
def approval(plugin_id):
    code, text, results = run([{"command": "plugin.inspect",
                                "params": {"path": os.path.join(ROOT, "dist", plugin_id)}}])
    if code != 0 or not results:
        sys.exit(f"FAIL {plugin_id}: inspect, run ./build.sh first: {text[-300:]}")
    return body(results[0])["approval"]


def plate(thickness, loose=False):
    """A 40 x 40 plate, a body 3 mm under it, and two circles on the plate's top."""
    plane = ({"origin": [0, 0, thickness], "x_dir": [1, 0, 0], "y_dir": [0, 1, 0]}
             if loose else {"face": [20, 20, thickness]})
    return [
        {"command": "solid.box", "params": {"length": 40, "width": 40, "height": thickness,
                                            "body_name": "Plate"}},
        {"command": "solid.box", "params": {"length": 40, "width": 40, "height": 5,
                                            "corner": [0, 0, -8], "body_name": "Under"}},
        {"command": "sketch.create", "params": {"plane": plane, "name": "Holes"}},
        {"command": "sketch.circle.center", "params": {"center": [10, 10], "diameter": 3}},
        {"command": "sketch.circle.center", "params": {"center": [30, 10], "diameter": 3}},
        {"command": "sketch.finish", "params": {}},
    ]


class Killed(Exception):
    """This case's own run was killed with SIGKILL: report it as SKIP."""


def plugin_run(plugin_id, entry, params, before=()):
    """Run one plugin after `before`, then measure. Returns (bodies by name, reply), or
    (None, the error line). Raises Killed when solidcraft-cli was killed with SIGKILL."""
    steps = list(before) + [
        {"command": "plugin.run", "params": {
            "path": os.path.join(ROOT, "dist", plugin_id), "approval": approval(plugin_id),
            "entry": entry, "params": params}},
        {"command": "inspect.measure", "params": {"summary": True}},
    ]
    code, text, results = run(steps)
    if killed(code):
        raise Killed(f"plugin.run killed solidcraft-cli (exit {code})")
    if code != 0 or not results:
        return None, last_line(text) if code != 0 else f"no results: {last_line(text)}"
    measured = body(results[-1])
    reply = body(results[-2])
    return {b["name"]: b for b in measured["bodies"]}, reply.get("result", reply)


def case(fn):
    """Run one case; a case whose run was killed with SIGKILL is a SKIP."""
    def wrapped(name, *args, **kwargs):
        try:
            fn(name, *args, **kwargs)
        except Killed as killed_run:
            skip(name, str(killed_run))
    return wrapped


@case
def on_plate(name, plugin_id, entry, params, removed_each, thickness=10, loose=False,
             bodies=None, top=None):
    """Two points on the plate: each must remove `removed_each` mm3 from it."""
    named = dict({"sketch": "Holes", "sketch_point": "c1, c2"}, **params)
    found, reply = plugin_run(plugin_id, entry, named, plate(thickness, loose))
    if found is None:
        check(name, False, "| " + reply)
        return
    want = 1600 * thickness - 2 * removed_each
    got = found["Plate"]["volume_mm3"]
    ok = abs(got - want) < 0.3 and abs(found["Under"]["volume_mm3"] - 8000) < 0.01
    if bodies is not None:
        ok = ok and sorted(found) == sorted(bodies)
    if top is not None:
        ok = ok and abs(max(b["bbox"]["max"][2] for b in found.values()) - top) < 0.01
    check(name, ok, f"| plate {got:.2f}, expected {want:.2f}; bodies {sorted(found)}")


@case
def refused(name, plugin_id, entry, params, words, before=()):
    found, reply = plugin_run(plugin_id, entry, params, before)
    check(name, found is None and words in reply, "| " + str(reply)[:160])


@case
def loose_screw(name, params, volume):
    """One screw on the origin: its whole volume. The thread is cosmetic unless `params`
    asks for it, so the shank is a plain cylinder and the volume is the head and drive's."""
    found, reply = plugin_run("techsquid-screws", "make-screw",
                              dict({"thread_modeled": False}, **params))
    if found is None:
        check(name, False, "| " + reply)
        return
    got = sum(b["volume_mm3"] for b in found.values())
    check(name, len(found) == 1 and abs(got - volume) < 0.05, f"| {got:.3f}, expected {volume:.3f}")


def cyl(d, h):
    return PI * d * d / 4 * h


def frustum(d0, d1, h):
    return PI * h / 12 * (d0 * d0 + d0 * d1 + d1 * d1)


def hexagon(across_flats, h):
    return math.sqrt(3) / 2 * across_flats ** 2 * h


def strip(d, width, depth):
    """A slot `width` wide right across a round face `d` across."""
    r, h = d / 2, width / 2
    return 2 * (h * math.sqrt(r * r - h * h) + r * r * math.asin(h / r)) * depth


INSERT = ("techsquid-inserts", "make-insert")
SCREW = ("techsquid-screws", "make-screw")
NUT = ("techsquid-nuts", "make-nut")

# Heat-set inserts: M3 x 5.7. The plastic hole is 4.0 across and 6.7 deep.
on_plate("insert: hole and insert", *INSERT, {}, cyl(4.0, 6.7),
         bodies=["Insert M3 x 5.7", "Insert M3 x 5.7 (2)", "Plate", "Under"])
on_plate("insert: hole only, +0.1", *INSERT, {"create": "Hole only", "hole_adjust": 0.1},
         cyl(4.1, 6.7), bodies=["Plate", "Under"])
on_plate("insert: through hole on a face sketch", *INSERT,
         {"create": "Hole only", "hole_depth": "Through"}, cyl(4.0, 10))
STEP = cyl(4.6, 1.7) + cyl(4.4, 2.5)
on_plate("insert: glue-in stepped, blind", *INSERT,
         {"create": "Hole only", "hole_type": "Glue-in (stepped)"}, STEP + cyl(4.0, 2.5))
on_plate("insert: glue-in stepped, through", *INSERT,
         {"create": "Hole only", "hole_type": "Glue-in (stepped)", "hole_depth": "Through"},
         STEP + cyl(4.0, 5.8))
on_plate("insert: blind hole on a loose sketch", *INSERT, {"create": "Hole only"},
         cyl(4.0, 6.7), loose=True)
refused("insert: through hole on a loose sketch is refused", *INSERT,
        {"sketch": "Holes", "sketch_point": "c1, c2", "hole_depth": "Through"},
        "Hole depth", plate(10, loose=True))
refused("insert: a hole with nothing selected says what to select", *INSERT, {},
        "select the sketch points")

# Two M3 x 12 socket head screws: the second body name gets a number.
SCREWS = ["Plate", "Screw M3 x 12 socket", "Screw M3 x 12 socket (2)", "Under"]

# Screw holes: M3 x 12 in a 12 mm plate. ISO 273 normal clearance is 3.4, the tap drill 2.5.
on_plate("screw: clearance hole", *SCREW, {"create": "Hole only"}, cyl(3.4, 12), 12)
on_plate("screw: counterbored hole, socket head flush", *SCREW,
         {"create": "Screw and hole", "hole_type": "Clearance, head recessed"},
         cyl(6.5, 3) + cyl(3.4, 9), 12, bodies=SCREWS, top=12)
# A countersunk M3 x 12 is 12 mm overall, head and all, so with the head flush its hole
# is 12 deep: the countersink, then the clearance hole to the bottom of the plate.
CONE = (6.72 - 3.4) / 2
on_plate("screw: countersunk hole, ISO 10642 head flush", *SCREW,
         {"head": "Countersunk", "create": "Screw and hole",
          "hole_type": "Clearance, head recessed"},
         frustum(6.72, 3.4, CONE) + cyl(3.4, 12 - CONE), 12, top=12)
CONE = (6.3 - 3.4) / 2
on_plate("screw: countersunk hole, ISO 2009 head with its land flush", *SCREW,
         {"head": "Countersunk", "drive": "Slotted", "create": "Screw and hole",
          "hole_type": "Clearance, head recessed"},
         frustum(6.3, 3.4, CONE) + cyl(3.4, 12 - CONE), 12, top=12)
on_plate("screw: tapped hole", *SCREW,
         {"create": "Hole only", "hole_type": "Tapped", "thread_modeled": False},
         cyl(2.5, 12), 12)
on_plate("screw: blind clearance hole for a shorter screw", *SCREW,
         {"create": "Screw and hole", "length": 8}, cyl(3.4, 8), 12, top=15)
on_plate("screw: no hole by default", *SCREW, {}, 0, 12,
         bodies=SCREWS)
refused("screw: a hole with nothing selected says what to select", *SCREW,
        {"create": "Hole only"}, "select the sketch points")

# Nut pocket: ISO 4032 M6 is 10 across and 5.2 high. 0.2 a side, and a 6.6 bolt hole below.
on_plate("nut: pocket and nut, flush", *NUT, {"size": "M6", "create": "Nut and pocket"},
         hexagon(10.4, 5.2) + cyl(6.6, 6.8), 12,
         bodies=["Nut M6", "Nut M6 (2)", "Plate", "Under"], top=12)
on_plate("nut: pocket only, no gap", *NUT, {"size": "M6", "create": "Pocket only", "pocket_adjust": 0},
         hexagon(10.0, 5.2) + cyl(6.6, 6.8), 12, bodies=["Plate", "Under"])
on_plate("nut: no pocket by default", *NUT, {"size": "M6"}, 0, 12,
         bodies=["Nut M6", "Nut M6 (2)", "Plate", "Under"], top=17.2)
refused("nut: a pocket with nothing selected says what to select", *NUT,
        {"create": "Pocket only"}, "select the sketch points")

# Sizes the host's own clearance table lacks: the plugins pass the ISO 273 medium hole as
# a diameter. M3.5 is 3.9; an ISO 4032 M3.5 nut is 6 across and 2.8 high.
on_plate("nut: M3.5 pocket, ISO 273 bolt hole by diameter", *NUT,
         {"custom_size": "M3.5", "create": "Pocket only", "pocket_adjust": 0},
         hexagon(6.0, 2.8) + cyl(3.9, 12 - 2.8), 12, bodies=["Plate", "Under"])
on_plate("screw: M3.5 clearance hole by diameter", *SCREW,
         {"custom_size": "M3.5", "create": "Hole only"}, cyl(3.9, 12), 12)
on_plate("screw: M3.5 counterbored hole by diameter, typed head", *SCREW,
         {"custom_size": "M3.5", "create": "Hole only", "hole_type": "Clearance, head recessed",
          "head_diameter": 6, "head_height": 3.5},
         cyl(7.0, 3.5) + cyl(3.9, 12 - 3.5), 12)
# Hole only needs no head: ISO 1580 stops at M10, but an M12 clearance hole still builds.
on_plate("screw: hole only for a pan head with no standard size", *SCREW,
         {"size": "M12", "head": "Pan head", "create": "Hole only"}, cyl(13.5, 12), 12)

# Loose M3 x 12 screws: head shapes and drive recesses.
SHANK = cyl(3, 12)
CAP = SHANK + cyl(5.5, 3)
loose_screw("screw: socket head, 2.5 mm hex socket", {}, CAP - hexagon(2.5, 1.3))
loose_screw("screw: socket head, no drive", {"drive": "None"}, CAP)
loose_screw("screw: cheese head, DIN 84 slot", {"head": "Cheese head"},
            SHANK + cyl(5.5, 2.0) - strip(5.5, 0.8, 0.85))
# ISO 4026 set screw, M3 x 6: a 45 degree flat point down to dp = 2, the socket end
# chamfered to the thread root (0.6134 pitches), and the deeper 2 mm socket of a length
# below the shaded ones. M3 x 3 is shaded and keeps the 1.2 mm socket.
def chamfer_ring(d, c, length=None):
    """What a chamfer removes: `c` off the radius, over `length` of the shank (c at 45 degrees)."""
    length = c if length is None else length
    return cyl(d, length) - frustum(d, d - 2 * c, length)


END = 0.6134 * 0.5
# A shaded, short length has a 120 degree socket end: the same radial step over less shank.
SHORT_END = chamfer_ring(3, END, END / math.tan(math.radians(60)))
loose_screw("screw: ISO 4026 set screw, flat point and 2 mm deep socket",
            {"head": "Set screw", "length": 6},
            cyl(3, 6) - hexagon(1.5, 2.0) - chamfer_ring(3, 0.5) - chamfer_ring(3, END))
loose_screw("screw: ISO 4026 set screw, shaded M3 x 3: 1.2 mm socket, 120 degree end",
            {"head": "Set screw", "length": 3},
            cyl(3, 3) - hexagon(1.5, 1.2) - chamfer_ring(3, 0.5) - SHORT_END)
# ISO 4029 cup point, M3 x 6: chamfered at 45 degrees from 3 mm to the 1.4 mm mouth,
# then a 120 degree cup, which is a cone 1.4 across and 1.4 / (2 sqrt 3) deep.
CUP = frustum(1.4, 0, 1.4 / (2 * math.sqrt(3)))
loose_screw("screw: ISO 4029 set screw, cup point and its chamfer",
            {"head": "Set screw", "tip": "Cup", "length": 6},
            cyl(3, 6) - hexagon(1.5, 2.0) - chamfer_ring(3, 0.8) - CUP - chamfer_ring(3, END))
loose_screw("screw: ISO 4029 set screw, shaded M3 x 4: 1.2 mm socket, 120 degree end",
            {"head": "Set screw", "tip": "Cup", "length": 4},
            cyl(3, 4) - hexagon(1.5, 1.2) - chamfer_ring(3, 0.8) - CUP - SHORT_END)
loose_screw("screw: set screw with no drive, flat point only",
            {"head": "Set screw", "drive": "None", "length": 6},
            cyl(3, 6) - chamfer_ring(3, 0.5))
# ISO 10642 measures the length overall: a 1.86 mm cone head on 10.14 mm of shank.
loose_screw("screw: ISO 10642 countersunk, 90 degree cone", {"head": "Countersunk", "drive": "None"},
            cyl(3, 12 - 1.86) + frustum(3, 6.72, 1.86))
# ISO 1580 pan: a 5.6 x 1.8 cylinder with its top edge rounded to 0.9.
ROUNDED = 2 * PI * (2.8 - 0.2234 * 0.9) * 0.9 ** 2 * (1 - PI / 4)
loose_screw("screw: ISO 1580 pan head, rounded edge", {"head": "Pan head", "drive": "None"},
            SHANK + cyl(5.6, 1.8) - ROUNDED)
# ISO 7380-1 button: the side is an arc of radius 3.5 from (2.85, 0) to (1.3, 1.65).
CX, CY, RF, STEPS = -0.3388794552318144, -1.4425837306723108, 3.5, 20000
BUTTON = sum(PI * (CX + math.sqrt(RF * RF - ((i + 0.5) * 1.65 / STEPS - CY) ** 2)) ** 2
             for i in range(STEPS)) * 1.65 / STEPS
loose_screw("screw: ISO 7380-1 button head, domed", {"head": "Button head", "drive": "None"},
            SHANK + BUTTON)


@case
def threaded_by_default(name, params, smooth):
    """A run that does not mention Modeled thread cuts the thread: the screw comes out
    lighter than the same screw with a plain shank, and the cut is about the thread's
    share of the shank, not the whole shank."""
    found, reply = plugin_run("techsquid-screws", "make-screw", params)
    if found is None:
        check(name, False, "| " + reply)
        return
    got = sum(b["volume_mm3"] for b in found.values())
    cut = smooth - got
    # An M3 x 0.5 thread over 12 mm takes roughly 10 to 20 mm3 out of the shank.
    check(name, len(found) == 1 and 5 < cut < 30, f"| {got:.3f}, smooth {smooth:.3f}, cut {cut:.3f}")


threaded_by_default("screw: the thread is modeled by default", {}, CAP - hexagon(2.5, 1.3))

finish()
