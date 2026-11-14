# Aluminium extrusions

20 mm series T-slot and V-slot aluminium extrusions for [SolidCraft](https://github.com/tangency-labs/solidcraft), cut to any length, and the slot nuts that go in them. Two entries: `make-extrusion` builds a rail, and `make-t-nut` puts a nut in a rail's slot. Each builds its part as a component of its own, once, and places it on every point (see [Components](../../README.md#components)).

A T-slot is the MISUMI 5 series rail (HFS5, NFS5): a 6 mm slot for M5 hardware. A V-slot is the OpenBuilds rail: the same 20 mm cell and Ø4.2 bore, with the slot faces chamfered at 45° so a wheel can run on them. The two slots are close, and they are not the same. A V-slot is 6.25 mm at the throat, 11 mm wide under a 1.8 mm lip. A T-slot is 6 mm, 12 mm and 2 mm.

## Install

You need a SolidCraft build that has the plugin host (the `commands@0.1` capability). SolidCraft 0.41 has it.

1. Download `techsquid-extrusions-<version>.zip` from the latest release.
2. Unzip it into SolidCraft's plugins folder, so you end up with a `techsquid-extrusions` folder holding `plugin.json` and `plugin.wasm`:

| OS | Plugins folder |
|---|---|
| Windows | `%LOCALAPPDATA%\SolidCraft\plugins\` |
| macOS | `~/Library/Application Support/SolidCraft/plugins/` |
| Linux | `$XDG_DATA_HOME/solidcraft/plugins/` (usually `~/.local/share/solidcraft/plugins/`) |

3. Start SolidCraft. Preferences > Plugins lists it. Listing never runs any code. Each run needs your approval of the exact package contents.

## Use

All sizes are millimetres. The form opens on a 2020 T-slot, 500 mm long.

Select a sketch point, or a circle, before you open the dialog. Leave Sketch and Sketch point empty. The rail's section is centred on that point, and the rail runs out of the sketch along its normal for its length. The section's width is along the sketch X and its height along the sketch Y, so a 2040 is 20 mm along X and 40 mm along Y. With nothing selected, the rail stands on the origin: the section is on the XY plane and the rail runs along +Z. The reply says which of these happened in `placed`. Select several points or circles and each one gets a rail, as one undo step; `count` and `points` in the reply list them. By name, Sketch point takes several ids with commas, such as `c1, c2`.

Name Sketch and Sketch point together to place the rail without a selection. Sketch point is a point id such as `c1.center`, or a circle id such as `c1`.

| Parameter | Default | Meaning |
|---|---|---|
| `slot` | `T-slot` | `T-slot` or `V-slot` |
| `size` | `2020` | `2020`, `2040`, `2060`, `2080` or `4040`. A V-slot comes as `2020` to `2080` |
| `length` | 500 | How long the rail is cut, mm, up to 6000. Kept to 0.01 mm, as the rail's name shows it |
| `end_taps` | `None` | `None`, `Start`, `End` or `Both`: the ends whose bores are tapped M5. The start is the end on the sketch point |
| `thread_modeled` | on | On cuts the thread into a tapped end. Off lists the thread and leaves the bore plain |
| `holes` | empty | Where M5 screw holes go across the rail, as distances from the start in mm: `10, 290`. A face name puts the distances after it on that face: `+X 10, 290 -Y 150` |
| `hole_face` | `+Y` | `+Y`, `-Y`, `+X` or `-X`: the face the screws go in from, for the holes that name none. Y is the section's height and X its width |
| `sketch` | empty | Sketch that holds the point. Empty, with Sketch point empty, uses the selection or the origin |
| `sketch_point` | empty | A point or circle in that sketch, such as `c1` or `c1.center` |

The reply gives `slot`, `size`, `length`, `width`, `height`, `area_mm2` (the area of the section), `end_taps`, `thread_modeled` and `holes`, each with its `face` and its distance `at`.

### Tapped ends and screw holes

Rails screw together with no bracket when one is drilled and the other tapped, which is how a Creality Ender 5 frame is held. An M5 screw goes in through a slot of the first rail, across its core, and into the end of the second. The plugin cuts both into a rail as it builds it.

- **Tapped ends.** `end_taps` taps every bore of an end M5, 12 mm deep. A rail's bore is 4.2 mm, the tap drill of M5, so only the thread is cut. That takes a few seconds for each bore, and SolidCraft gives a plugin run 30, so a thread is cut into at most four bores of one rail: both ends of a 2040, or one end of a 2080 or a 4040. With Modeled thread off the thread is listed on the hole, the bore stays plain, and any rail can have both ends tapped.
- **Screw holes.** Each distance in `holes` gets a hole from the middle of a slot on `hole_face`, straight through the rail. It is a 9.5 mm access hole through the lips, down to the slot floor, and a 5.5 mm clearance hole from there out through the far face. The head of an M5 socket head screw goes through the access hole and sits on the slot floor, 6 mm under the face of a T-slot and 6.1 mm under the face of a V-slot. A face with two slots or more gets a hole in each. A hole must be at least 5.75 mm from an end. A rail can have holes from several faces: write a face name in front of its distances. Holes from two faces meet in the core, so they must be 9.5 mm apart along the rail.

An M5 x 25 screw on the slot floor of a 2020 rail reaches 11 mm into the end it holds, and an M5 x 20 reaches 6 mm. Place it with the screw plugin, Create set to Screw only, on a point on the slot floor: sketch on a plane 6 mm inside the drilled rail's face, facing out. Where a rail has a screw in its end and a screw hole 10 mm from that end, the first screw must be the M5 x 20, or the two meet inside the rail.

No washer goes under the head. The slot floor is 5.9 mm wide between the webs, and the access hole cuts just enough of them for the 8.5 mm head. An M5 washer is 10 mm across, or 9 mm in the small series, and neither fits there.

A cut thread and the screw in it still overlap a little, by about 14 mm³ for each M5 screw. SolidCraft cuts each thread where its own part starts it, so the two are not in step.

A tapped or drilled rail is a part of its own. Its name says what is cut into it, such as `T-slot 2020 x 300 taps both holes -Y 10 290 +X 150`, and every placement of it has the same cuts, so two rails drilled differently are two parts.

### Slot nuts

Plugins > Slot nut puts a nut under the lips of a slot. Sketch on the rail's face, put a point in the middle of the slot where each screw goes, select the points, and run it. The nut hangs under the lips with its tapped hole on the point: its shoulders are 2 mm below the face of a T-slot and 1.8 mm below the face of a V-slot, which is the lip of each. The form opens on an M5 pre-assembly nut in a T-slot.

The nut does not look for the slot. It goes on the point you give it, and Rail runs along says which sketch axis the slot follows, so the nut lies along it. With nothing selected the nut hangs under the origin, its hole on the Z axis and its length along X.

| Parameter | Default | Meaning |
|---|---|---|
| `type` | `Pre-assembly` | `Pre-assembly` slides in from the end of the rail. `Post-assembly` goes in through the slot and has a spring ball under it |
| `size` | `M5` | The thread. `M5` is the one size |
| `slot` | `T-slot` | `T-slot` or `V-slot`, the rail the nut sits in. It sets how far under the face the nut hangs |
| `along` | `Sketch X` | `Sketch X` or `Sketch Y`, the sketch axis the rail runs along |
| `thread_modeled` | on | On cuts the thread into the hole. Off drills the 4.2 mm tap hole and lists the thread |
| `sketch` | empty | Sketch that holds the point. Empty, with Sketch point empty, uses the selection or the origin |
| `sketch_point` | empty | A point or circle in that sketch, such as `c1` or `c1.center`. Several go in one box with commas |

The reply gives `type`, `part` (the MISUMI part number), `thread`, `slot`, `along`, `length`, `width`, `height` and `under_face`.

The package calls `selection.get`, `inspect.measure` (for the body names, so a new body gets a free one), `sketch.inspect`, `sketch.create`, `sketch.line`, `sketch.arc.three_point`, `sketch.circle.center`, `sketch.finish`, `solid.extrude`, `solid.hole` (a tap, and a nut's thread), `solid.cylinder` (a screw hole, and a nut's ball recess), `solid.sphere` (a nut's ball), and `component.list`, `component.create`, `component.metadata`, `component.metadata.set`, `component.activate`, `occurrence.copy`, `occurrence.move` and `occurrence.reparent` to place it.

## Sizes

A rail is a grid of 20 mm cells. Each cell has a square core with a Ø4.2 bore, a 1.5 mm web from each core corner, and a slot on every side that faces out. Between two cells the webs and walls close a cavity.

| Size | Width x height | T-slot area, mm² | V-slot area, mm² |
|---|---|---:|---:|
| 2020 | 20 x 20 | 181.3 | 171.4 |
| 2040 | 20 x 40 | 322.4 | 300.0 |
| 2060 | 20 x 60 | 463.5 | 428.6 |
| 2080 | 20 x 80 | 604.6 | 557.2 |
| 4040 | 40 x 40 | 479.1 | not offered |

At 2.70 g/cm³ a T-slot 2020 weighs 0.49 kg/m and a 2040 0.87 kg/m.

In a T-slot 4040 no web runs to the middle. The four cavities are one, and each core is open to it through a 53° wedge that faces the middle of the rail, so each bore is a C. That is MISUMI's STEP model of NFS5-4040. The catalog draws the opening differently and lists 472 mm², which is 1.5% under the model.

Each T-slot floor is the one in MISUMI's STEP models: a 120° groove, 0.3 mm deep, with both edges rounded to R0.5, and beside it a shallower mark. The 2040 model also rounds the corners of its closed cavity to R0.3, which the 2060 and 2080 models do not; here they are sharp in every size.

### Slot nuts

Both nuts are MISUMI's for the 5 series slot, and both fit a V-slot as well. A nut is one section along its whole length: a 5.8 mm boss that sits in the slot's throat, shoulders that bear under the lips, and flanks that slope in to clear the rail's webs.

| Type | MISUMI part | Length | Width | Height | Hole from the end |
|---|---|---:|---:|---:|---:|
| Pre-assembly | HNTT5-5 | 10 | 10 | 4.1 | 5 |
| Post-assembly | HNTP5-5 | 17 | 8.3 | 3.9 | 6 |

The post-assembly nut has an R2 ball 7.5 mm past its hole, standing 0.8 mm out of the bottom. On a real nut a spring pushes it against the slot floor to hold the nut in place. Here it is part of the body, and in a T-slot it just touches the floor.

MISUMI sells both nuts tapped M3 and M4 too. Only the M5 models have been checked, so M5 is the one size.

What is not here: the closed-face rails (HFSF5, HFST5), the 30 and 40 mm series, the V-slot 40x40 and C-Beam, washers, and brackets, plates and other nuts.

## Standards

| What | Source | Test |
|---|---|---|
| T-slot: 6 slot, 2 lip, 12 cavity, 8 core, Ø4.2 bore, R1 corners, 1.5 webs | MISUMI catalog 2019 US p. 2686 to 2687, Slot Details and the HFS5-2020 and HFS5-4040 sections | `slot::tests::the_t_slot_is_the_misumi_5_series_slot` |
| T-slot floor: the 120° groove, 0.3 deep, its R0.5 roundings and the mark beside it | The same pages, Slot Bottom Details, and the MISUMI STEP models below | `slot::tests::the_t_slot_floor_is_the_misumi_floor` |
| T-slot 2020 outline vertices | MISUMI STEP model `NFS5-2020-50.stp` (PARTcommunity) | `profile::tests::the_t_slot_2020_has_the_misumi_step_vertices` |
| T-slot 2040 outline and cavity vertices | MISUMI STEP model `NFS5-2040-50.stp` | `profile::tests::the_t_slot_2040_has_the_misumi_step_vertices` |
| T-slot 2060 cavities, 143.03 mm² each | MISUMI STEP model `NFS5-2060-2060.stp` | `profile::tests::the_t_slot_2060_has_the_misumi_step_cavities` |
| T-slot 4040 outline and middle cavity vertices, and the opening of each core | MISUMI STEP model `NFS5-4040-50.stp` | `profile::tests::the_t_slot_4040_has_the_misumi_step_vertices` |
| T-slot areas 181.26, 322.44, 463.49, 604.60 and 479.13 mm² for 2020, 2040, 2060, 2080 and 4040, to within 0.03 mm² (0.1 for the 2040) | The end faces of those models and of `NFS5-2080-50.stp` | `profile::tests::the_t_slot_areas_are_the_misumi_step_areas` |
| T-slot areas 183 and 324 mm² for 2020 and 2040, to within 1% | MISUMI catalog 2019 US p. 2686 to 2687, Sectional Area | `profile::tests::the_t_slot_areas_are_near_the_misumi_catalog_areas` |
| V-slot: 6.25 throat, 9.16 mouth, 11 cavity, 1.80 lip, 1.50 webs, Ø4.20 bore | OpenBuilds drawing "V-Slot 20x20 Linear Rail" | `slot::tests::the_v_slot_is_the_openbuilds_slot` |
| V-slot outline and cavity vertices, and the area of each section | OpenBuilds `V-Slot 20x20.dxf`, `20x40.dxf`, `20x60.dxf` and `20x80.dxf` | `profile::tests::the_v_slot_outline_has_the_dxf_vertices`, `the_v_slot_2040_cavity_has_the_dxf_vertices`, `the_v_slot_areas_are_the_openbuilds_dxf_areas` |
| Pre-assembly nut: its section, 10 long, the hole in the middle | MISUMI STEP model `HNTT5-5.stp` and the drawing on its product page | `tnut::tests::the_pre_assembly_nut_is_the_misumi_hntt5` |
| Post-assembly nut: its section, 17 long, the hole 6 from the end, the ball and its recess | MISUMI STEP model `HNTP5-5.stp` and the drawing on its product page | `tnut::tests::the_post_assembly_nut_is_the_misumi_hntp5` |
| Screw holes: 5.5 mm clearance, 9.5 mm access, and the 4.2 mm bore as the M5 tap drill | ISO 273 (medium fit) for the clearance hole, ISO 4762 for the 8.5 mm head with 0.5 mm round it, and the slot rows above for the bore | `machining::tests::the_holes_are_for_an_m5_socket_head_screw` |
| Both nuts clear the lips, walls and webs of both slots | The slot and nut rows above | `tnut::tests::both_nuts_fit_both_slots` |

The catalog's areas are a loose check: its 2040 is 0.5% over MISUMI's own model and its 4040 is 1.5% under.

## Names

| What | Name |
|---|---|
| Rail body and component | `T-slot 2040 x 500`, `V-slot 2020 x 122.5` |
| A tapped or drilled rail | `T-slot 2040 x 300 taps start`, `T-slot 2020 x 300 taps both holes -Y 10 290 +X 150` |
| Its sketch and feature | `T-slot 2040 x 500: Section`, `T-slot 2040 x 500: Rail` |
| Its taps and screw holes | `: Tap start 1`, `: Tap end 2`, `: Hole at 10: Access`, `: Hole at 10: Clearance`, and `: Hole at 50 (2): Access` on a face with two slots |
| Nut body and component | `T-nut M5 pre-assembly`, `T-nut M5 post-assembly` |
| Its sketch and features | `T-nut M5 pre-assembly: Section`, `: Bar`, `: Thread`, and `: Ball recess` and `: Ball` on a post-assembly nut |
| The folder its placements go in | `Extrusions` |

In the bill of materials a rail is `purchased`, its part number is its name, and its description reads `20 x 40 mm T-slot aluminium extrusion, 500 mm long`. A nut is `purchased` too, and its description reads `MISUMI HNTT5-5 pre-assembly insertion nut for a 6 mm slot`.

## Older field names

None.
