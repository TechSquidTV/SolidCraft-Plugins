#!/usr/bin/env python3
"""Check what the built plugins actually cut, by volume, through solidcraft-cli.

    SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/geometry_test.py

Needs the packages from ./build.sh and a solidcraft-cli that can run a plugin. Each
case builds a plate with a sketch of two circles on its top face, runs one plugin on
those circles, and compares the plate's volume with the volume worked out by hand
here. A second body under the plate must stay untouched. Head shapes and drive
recesses are checked on a loose screw the same way. A rail is checked by its whole
volume: the area of its section, worked out here from the slot sizes, times its length.

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


def second(name):
    """The second placement of a fastener: every one is a placement of one component,
    and the host names the body of each after the first by its path."""
    return f"{name} (Fasteners:1/{name}:2)"


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
         bodies=["Insert M3 x 5.7", second("Insert M3 x 5.7"), "Plate", "Under"])
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
SCREWS = ["Plate", "Screw M3 x 12 socket", second("Screw M3 x 12 socket"), "Under"]

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
         bodies=["Nut M6", second("Nut M6"), "Plate", "Under"], top=12)
on_plate("nut: pocket only, no gap", *NUT, {"size": "M6", "create": "Pocket only", "pocket_adjust": 0},
         hexagon(10.0, 5.2) + cyl(6.6, 6.8), 12, bodies=["Plate", "Under"])
on_plate("nut: no pocket by default", *NUT, {"size": "M6"}, 0, 12,
         bodies=["Nut M6", second("Nut M6"), "Plate", "Under"], top=17.2)
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

# Rails. The section areas are worked out here from the slot sizes, not read back from
# the plugin: a cell is 400 mm2 less its slots, its bore and the rounding of the corners.
def slot_area(throat, mouth, lip, cavity, core, groove):
    """One slot of a 20 mm cell, from half widths as the drawings give them."""
    w = 1.5 / math.sqrt(2)          # a 1.5 mm web, measured along x or y
    shoulder = 10 - cavity - w      # depth where the web meets the cavity wall
    floor, root = 10 - core, core - w
    chamfer = (mouth - throat) ** 2                     # the two 45 degree chamfers
    opening = 2 * throat * lip + chamfer
    under_lip = 2 * cavity * (shoulder - lip)
    taper = (cavity + root) * (floor - shoulder)
    return opening + under_lip + taper + groove


def misumi_floor():
    """What the MISUMI slot floor takes out of a core face: a 120 degree groove 0.3 deep
    with both edges rounded to R0.5, and beside it a mark whose far side rises at 11
    degrees to meet the floor 2.5 from the middle."""
    depth, flank, r, rise, end = 0.3, math.radians(30), 0.5, math.radians(11), 2.5
    drop = r * (1 - math.cos(flank))                   # where a rounding meets a flank
    top = (depth - drop) / math.tan(flank)             # that point, from the middle
    centre = top + r * math.sin(flank)                 # the centre of each rounding
    # Under one rounding, from its top to where it meets a flank.
    under = r * r * math.sin(flank) - r * r * (flank + math.sin(flank) * math.cos(flank)) / 2
    start = centre + r * math.sin(flank)               # the mark starts off the rounding
    mark = (math.tan(rise) * end - drop + math.tan(flank) * start) / (math.tan(flank) + math.tan(rise))
    mark_depth = math.tan(rise) * (end - mark)
    return (3 * under + (drop + depth) * top
            + (drop + mark_depth) * (mark - start) / 2 + mark_depth * (end - mark) / 2)


def cavity_area(lip, cavity, core):
    """The closed cavity between two cells of a row."""
    w = 1.5 / math.sqrt(2)
    face, leg, root, knee, wall = 10 - core, 10 - cavity - lip, core - w, cavity + lip - w, 10 - lip
    return 2 * ((root + knee) * (face - leg) + 2 * wall * leg)


BORE = PI * 2.1 ** 2
T = dict(throat=3, mouth=3, lip=2, cavity=6, core=4, groove=misumi_floor())
V = dict(throat=3.125, mouth=4.58, lip=1.8, cavity=5.5, core=3.9, groove=0.21 * 0.21)


def rail_area(shape, cells, corner):
    slots = 2 * cells + 2
    between = cavity_area(shape["lip"], shape["cavity"], shape["core"]) * (cells - 1)
    return (400 * cells - (4 - PI) * corner ** 2 - slots * slot_area(**shape)
            - cells * BORE - between)


@case
def loose_rail(name, params, area, size):
    """One rail on the origin: one closed body of the section's area times the length."""
    found, reply = plugin_run("techsquid-extrusions", "make-extrusion", params)
    if found is None:
        check(name, False, "| " + reply)
        return
    want = area * params["length"]
    rail = list(found.values())[0]
    got = rail["volume_mm3"]
    box = [rail["bbox"]["max"][i] - rail["bbox"]["min"][i] for i in range(3)]
    ok = (len(found) == 1 and rail["closed"] and abs(got - want) < 0.5
          and all(abs(box[i] - size[i]) < 1e-6 for i in range(3)))
    check(name, ok, f"| {got:.2f}, expected {want:.2f}; box {box}; bodies {sorted(found)}")


loose_rail("rail: T-slot 2020 x 100", {"slot": "T-slot", "size": "2020", "length": 100},
           rail_area(T, 1, 1.0), (20, 20, 100))
loose_rail("rail: V-slot 2020 x 100", {"slot": "V-slot", "size": "2020", "length": 100},
           rail_area(V, 1, 0.5), (20, 20, 100))
loose_rail("rail: T-slot 2040 x 250", {"slot": "T-slot", "size": "2040", "length": 250},
           rail_area(T, 2, 1.0), (20, 40, 250))
loose_rail("rail: V-slot 2060 x 80", {"slot": "V-slot", "size": "2060", "length": 80},
           rail_area(V, 3, 0.5), (20, 60, 80))
loose_rail("rail: T-slot 2080 x 50", {"slot": "T-slot", "size": "2080", "length": 50},
           rail_area(T, 4, 1.0), (20, 80, 50))
# The end face of MISUMI's STEP model of NFS5-4040 is 479.13 mm2, with each core open to
# the middle.
@case
def rail_4040(name):
    found, reply = plugin_run("techsquid-extrusions", "make-extrusion",
                              {"slot": "T-slot", "size": "4040", "length": 100})
    if found is None:
        check(name, False, "| " + reply)
        return
    rail = list(found.values())[0]
    got = rail["volume_mm3"] / 100
    box = [rail["bbox"]["max"][i] - rail["bbox"]["min"][i] for i in range(3)]
    ok = (len(found) == 1 and rail["closed"] and abs(got - 479.13) < 0.05
          and all(abs(box[i] - (40, 40, 100)[i]) < 1e-6 for i in range(3)))
    check(name, ok, f"| area {got:.2f}, STEP model 479.13; box {box}")


rail_4040("rail: T-slot 4040 x 100 has the STEP model's area")


@case
def rails_on_points(name):
    """Two points on the plate: one rail definition, placed twice, standing on the plate
    and leaving it as it was."""
    found, reply = plugin_run(
        "techsquid-extrusions", "make-extrusion",
        {"slot": "V-slot", "size": "2020", "length": 60, "sketch": "Holes", "sketch_point": "c1, c2"},
        plate(10, False))
    if found is None:
        check(name, False, "| " + reply)
        return
    top = max(b["bbox"]["max"][2] for b in found.values())
    placed = reply.get("component", {})
    ok = (abs(found["Plate"]["volume_mm3"] - 16000) < 0.01
          and reply.get("count") == 2 and len(placed.get("occurrences", [])) == 2
          and placed.get("component") == "V-slot 2020 x 60" and abs(top - 70) < 0.01)
    check(name, ok, f"| top {top:.2f}, expected 70; reply {json.dumps(reply)[:200]}")


rails_on_points("rail: two points get one V-slot 2020 placed twice")
refused("rail: a V-slot 4040 is refused on Size", "techsquid-extrusions", "make-extrusion",
        {"slot": "V-slot", "size": "4040"}, "size")



def rail_and_face(slot):
    """A 60 mm 2020 rail standing on the origin, and a sketch on its +X face with two
    points in the middle of the slot."""
    return [
        {"command": "plugin.run", "params": {
            "path": os.path.join(ROOT, "dist", "techsquid-extrusions"),
            "approval": approval("techsquid-extrusions"), "entry": "make-extrusion",
            "params": {"slot": slot, "size": "2020", "length": 60}}},
        {"command": "sketch.create", "params": {"plane": "YZ", "offset": 10, "name": "Face"}},
        {"command": "sketch.point", "params": {"point": [0, 15], "id": "a"}},
        {"command": "sketch.point", "params": {"point": [0, 40], "id": "b"}},
        {"command": "sketch.finish", "params": {}},
    ]


def nut_step(params):
    return {"command": "plugin.run", "params": {
        "path": os.path.join(ROOT, "dist", "techsquid-extrusions"),
        "approval": approval("techsquid-extrusions"), "entry": "make-t-nut", "params": params}}


def measured(steps):
    """Run the steps, then measure and check for interference. Returns (bodies by name,
    interference count), or (None, the error line)."""
    code, text, results = run(list(steps) + [
        {"command": "inspect.measure", "params": {"summary": True}},
        {"command": "inspect.interference", "params": {}},
    ])
    if killed(code):
        raise Killed(f"plugin.run killed solidcraft-cli (exit {code})")
    if code != 0 or not results:
        return None, last_line(text)
    hits = body(results[-1])
    return ({b["name"]: b for b in body(results[-2])["bodies"]},
            len(hits.get("interferences", [])) + len(hits.get("unchecked", [])))


# The nuts' boxes as MISUMI draws them: 10 x 10 x 4.1, and 17 x 8.3 x 3.9 with the ball
# standing 0.8 out of the bottom.
NUT_BOX = {"Pre-assembly": (10, 10, 4.1), "Post-assembly": (17, 8.3, 4.7)}


@case
def loose_nut(name, kind):
    """One nut on the origin: under the XY plane by the T-slot's 2 mm lip, with a
    modeled thread that takes metal out of the plain hole."""
    volumes = []
    for modeled in (False, True):
        found, reply = plugin_run("techsquid-extrusions", "make-t-nut",
                                  {"type": kind, "thread_modeled": modeled})
        if found is None:
            check(name, False, "| " + reply)
            return
        nut = list(found.values())[0]
        box = [nut["bbox"]["max"][i] - nut["bbox"]["min"][i] for i in range(3)]
        top = nut["bbox"]["max"][2]
        boss = 0.8 if kind == "Pre-assembly" else 0.7
        if not (len(found) == 1 and nut["closed"] and abs(top - (boss - 2)) < 1e-6
                and all(abs(box[i] - NUT_BOX[kind][i]) < 1e-6 for i in range(3))):
            check(name, False, f"| box {box}, top {top}; bodies {sorted(found)}")
            return
        volumes.append(nut["volume_mm3"])
    cut = volumes[0] - volumes[1]
    check(name, 5 < cut < 20, f"| a modeled thread removed {cut:.2f} mm3")


loose_nut("t-nut: a pre-assembly nut is 10 x 10 x 4.1 with a thread", "Pre-assembly")
loose_nut("t-nut: a post-assembly nut is 17 x 8.3 x 3.9 and its ball", "Post-assembly")


@case
def nut_in_slot(name, slot, kind):
    """A nut on a point of the rail's face: under the lips, along the rail, touching
    nothing, and the rail as it was."""
    before = rail_and_face(slot)
    rail_only, _ = measured(before)
    found, hits = measured(before + [nut_step(
        {"type": kind, "slot": slot, "along": "Sketch Y", "sketch": "Face", "sketch_point": "b"})])
    if found is None or rail_only is None:
        check(name, False, "| " + str(hits))
        return
    rail_name = f"{slot} 2020 x 60"
    nut = [b for n, b in found.items() if n != rail_name]
    lip = 2.0 if slot == "T-slot" else 1.8
    boss = 0.8 if kind == "Pre-assembly" else 0.7
    start = 5 if kind == "Pre-assembly" else 6
    ok = (len(nut) == 1 and hits == 0
          and abs(found[rail_name]["volume_mm3"] - rail_only[rail_name]["volume_mm3"]) < 1e-6
          and abs(nut[0]["bbox"]["max"][0] - (10 - lip + boss)) < 1e-6
          and abs(nut[0]["bbox"]["min"][2] - (40 - start)) < 1e-6
          and abs(nut[0]["bbox"]["max"][2] - (40 - start + NUT_BOX[kind][0])) < 1e-6)
    check(name, ok, f"| interferences {hits}; nut box {nut[0]['bbox'] if nut else None}")


for slot in ("T-slot", "V-slot"):
    for kind in ("Pre-assembly", "Post-assembly"):
        nut_in_slot(f"t-nut: a {kind.lower()} nut sits in a {slot} and leaves the rail alone", slot, kind)


@case
def nuts_keep_their_own_threads(name):
    """SolidCraft 0.42 cuts a modeled thread into any bore of that size on the same
    axis. A second kind of nut must leave the first as it was built."""
    before = rail_and_face("T-slot")
    place = {"along": "Sketch Y", "sketch": "Face"}
    first = nut_step(dict(place, type="Pre-assembly", sketch_point="a"))
    alone, _ = measured(before + [first])
    found, hits = measured(before + [first, nut_step(dict(place, type="Post-assembly", sketch_point="b"))])
    if found is None or alone is None:
        check(name, False, "| " + str(hits))
        return
    pre = "T-nut M5 pre-assembly"
    ok = (hits == 0 and len(found) == 3
          and abs(found[pre]["volume_mm3"] - alone[pre]["volume_mm3"]) < 1e-6
          and abs(found["T-slot 2020 x 60"]["volume_mm3"] - alone["T-slot 2020 x 60"]["volume_mm3"]) < 1e-6)
    check(name, ok, f"| {pre} {found[pre]['volume_mm3']:.3f}, alone {alone[pre]['volume_mm3']:.3f}; "
                    f"interferences {hits}")


nuts_keep_their_own_threads("t-nut: a second kind of nut leaves the first nut and the rail as built")
refused("t-nut: an unknown Type is refused", "techsquid-extrusions", "make-t-nut",
        {"type": "Hammer"}, "Pre-assembly or Post-assembly")



def hex_nut_step(params):
    return {"command": "plugin.run", "params": {
        "path": os.path.join(ROOT, "dist", "techsquid-nuts"),
        "approval": approval("techsquid-nuts"), "entry": "make-nut", "params": params}}


@case
def hex_nut_beside_a_rail(name):
    """An M5 nut's tap drill is the 4.2 mm of a rail's bore. With a 2020 rail standing on
    the origin, the nut must take its own thread and the rail must stay as it was."""
    before = rail_and_face("T-slot")
    rail_only, _ = measured(before)
    on_face = {"size": "M5", "sketch": "Face", "sketch_point": "a"}
    plain, _ = measured(before + [hex_nut_step(dict(on_face, thread_modeled=False))])
    found, hits = measured(before + [hex_nut_step(on_face)])
    if found is None or plain is None or rail_only is None:
        check(name, False, "| " + str(hits))
        return
    rail = "T-slot 2020 x 60"
    cut = plain["Nut M5"]["volume_mm3"] - found["Nut M5"]["volume_mm3"]
    ok = (abs(found[rail]["volume_mm3"] - rail_only[rail]["volume_mm3"]) < 1e-6 and 5 < cut < 20)
    check(name, ok, f"| rail {found[rail]['volume_mm3']:.3f}, alone {rail_only[rail]['volume_mm3']:.3f}; "
                    f"the thread removed {cut:.2f} mm3 from the nut")


hex_nut_beside_a_rail("nut: an M5 nut is threaded and a rail on the origin is left alone")


@case
def two_definitions_of_one_nut(name):
    """A nut with a plain hole and the same nut with a modeled thread are two
    definitions. Building the second must not thread the first."""
    sketch = plate(10, True)
    first = hex_nut_step({"size": "M5", "thread_modeled": False, "sketch": "Holes", "sketch_point": "c1"})
    alone, _ = measured(sketch + [first])
    found, hits = measured(sketch + [first, hex_nut_step({"size": "M5", "sketch": "Holes", "sketch_point": "c2"})])
    if found is None or alone is None:
        check(name, False, "| " + str(hits))
        return
    nuts = sorted(b["volume_mm3"] for n, b in found.items() if n.startswith("Nut M5"))
    plain = alone["Nut M5"]["volume_mm3"]
    ok = len(nuts) == 2 and abs(nuts[1] - plain) < 1e-6 and 5 < plain - nuts[0] < 20
    check(name, ok, f"| nuts {[round(v, 3) for v in nuts]}, the plain one alone {plain:.3f}")


two_definitions_of_one_nut("nut: a threaded M5 nut built after a plain one leaves the plain one plain")



def rail_step(params):
    return {"command": "plugin.run", "params": {
        "path": os.path.join(ROOT, "dist", "techsquid-extrusions"),
        "approval": approval("techsquid-extrusions"), "entry": "make-extrusion", "params": params}}


@case
def machined_rail(name, params, plain, removed):
    """A tapped or drilled rail on the origin: one closed body named for what is cut into
    it, lighter than the plain rail by `removed` mm3."""
    found, reply = plugin_run("techsquid-extrusions", "make-extrusion", params)
    if found is None:
        check(name, False, "| " + reply)
        return
    rail = list(found.values())[0]
    want = plain * params["length"] - removed
    ok = (len(found) == 1 and rail["closed"] and abs(rail["volume_mm3"] - want) < 1.5
          and list(found) == [reply["component"]["component"]])
    check(name, ok, f"| {rail['volume_mm3']:.2f}, expected {want:.2f}; bodies {sorted(found)}")


# A hole through a 2020 takes the lips beside the slot over the 9.5 mm access hole, the
# roots of the two webs that hole reaches at the slot floor, and the 8 mm core over the
# 5.5 mm clearance hole, less the bore that already crosses it and the groove in each of
# the two slot floors it passes.
def lips_removed(access=9.5, throat=3.0, lip=2.0):
    r = access / 2
    return lip * (PI * r * r - 2 * (throat * math.sqrt(r * r - throat * throat)
                                    + r * r * math.asin(throat / r)))


def core_removed(clearance=5.5, core=4.0, bore=2.1, steps=4000):
    """A cylinder of the clearance diameter across the core, less the part of it the
    bore along the rail has already taken, by slices across the bore."""
    r = clearance / 2
    solid = PI * r * r * 2 * core
    shared = 0.0
    for i in range(steps):
        x = -bore + 2 * bore * (i + 0.5) / steps
        if abs(x) < r:
            shared += 2 * math.sqrt(bore * bore - x * x) * 2 * math.sqrt(r * r - x * x) * 2 * bore / steps
    return solid - shared


def webs_removed(access=9.5, core=4.0, web=1.5, steps=4000):
    """The access hole is wider than the slot floor, so it cuts into the web on each
    side from the floor up to where the web edge leaves the hole."""
    r, offset = access / 2, web / math.sqrt(2)
    top = r + offset
    total = 0.0
    for i in range(steps):
        a = core + (top - core) * (i + 0.5) / steps - offset
        total += r * r * math.acos(a / r) - a * math.sqrt(r * r - a * a)
    return 2 * total * (top - core) / steps


# The groove is narrow beside the hole, so it is taken as crossing the hole's middle.
HOLE_2020 = lips_removed() + webs_removed() + core_removed() - 2 * misumi_floor() * 5.5
machined_rail("rail: a tapped 2020 is named for its taps and weighs what the plain one does",
              {"slot": "T-slot", "size": "2020", "length": 100, "end_taps": "Both"},
              rail_area(T, 1, 1.0), 0)
machined_rail("rail: two screw holes take the lips and the core out of a 2020",
              {"slot": "T-slot", "size": "2020", "length": 100, "holes": "10, 90", "hole_face": "-Y"},
              rail_area(T, 1, 1.0), 2 * HOLE_2020)


@case
def screwed_joint(name):
    """A drilled base rail, a post with its foot tapped, and two M5 x 25 screws on the
    slot floor: each screw passes the base rail without touching it and ends 11 mm up a
    bore of the post."""
    steps = [
        {"command": "sketch.create", "params": {"plane": "XZ", "offset": -50, "name": "Base section"}},
        {"command": "sketch.point", "params": {"point": [0, -10], "id": "b"}},
        {"command": "sketch.finish", "params": {}},
        rail_step({"slot": "T-slot", "size": "2020", "length": 100, "holes": "40, 60",
                   "hole_face": "+Y", "sketch": "Base section", "sketch_point": "b"}),
        {"command": "sketch.create", "params": {"plane": "XY", "offset": 20, "name": "Post foot"}},
        {"command": "sketch.point", "params": {"point": [0, 0], "id": "p"}},
        {"command": "sketch.finish", "params": {}},
        rail_step({"slot": "T-slot", "size": "2040", "length": 80, "end_taps": "Start",
                   "sketch": "Post foot", "sketch_point": "p"}),
        {"command": "sketch.create", "params": {
            "plane": {"origin": [0, 0, 6], "x_dir": [1, 0, 0], "y_dir": [0, -1, 0]},
            "name": "Screw seats"}},
        {"command": "sketch.point", "params": {"point": [0, 10], "id": "s1"}},
        {"command": "sketch.point", "params": {"point": [0, -10], "id": "s2"}},
        {"command": "sketch.finish", "params": {}},
        {"command": "plugin.run", "params": {
            "path": os.path.join(ROOT, "dist", "techsquid-screws"),
            "approval": approval("techsquid-screws"), "entry": "make-screw",
            "params": {"size": "M5", "length": 25, "head": "Socket head", "create": "Screw only",
                       "sketch": "Screw seats", "sketch_point": "s1, s2"}}},
        {"command": "inspect.measure", "params": {"summary": True}},
        {"command": "inspect.interference", "params": {}},
    ]
    code, text, results = run(steps)
    if killed(code):
        raise Killed(f"plugin.run killed solidcraft-cli (exit {code})")
    if code != 0 or not results:
        check(name, False, "| " + last_line(text))
        return
    bodies = {b["name"]: b for b in body(results[-2])["bodies"]}
    hits = body(results[-1])["interferences"]
    base, post = "T-slot 2020 x 100 holes +Y 40 60", "T-slot 2040 x 80 taps start"
    screws = [b for n, b in bodies.items() if n.startswith("Screw M5 x 25 socket")]
    # A screw's thread overlaps the bore it is screwed into, which is listed as tapped
    # and left at the tap drill's size. It must touch nothing else.
    ok = (len(bodies) == 4 and len(screws) == 2 and len(hits) == 2
          and all(post in (h["a"], h["b"]) and base not in (h["a"], h["b"]) for h in hits)
          and all(abs(s["bbox"]["min"][2] - 1) < 1e-6 and abs(s["bbox"]["max"][2] - 31) < 1e-6
                  for s in screws))
    check(name, ok, f"| bodies {sorted(bodies)}; overlaps {[(h['a'], h['b']) for h in hits]}")


screwed_joint("rail: two screws through a drilled rail hold a tapped post and miss the rail")
refused("rail: a screw hole too close to an end is refused", "techsquid-extrusions", "make-extrusion",
        {"length": 100, "holes": "3"}, "too close to an end")


finish()
