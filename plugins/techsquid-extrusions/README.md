# Aluminium extrusions

20 mm series T-slot and V-slot aluminium extrusions for [SolidCraft](https://github.com/tangency-labs/solidcraft), cut to any length. One entry, `make-extrusion`. It builds the rail as a component of its own, once, and places it on every point (see [Components](../../README.md#components)).

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
| `sketch` | empty | Sketch that holds the point. Empty, with Sketch point empty, uses the selection or the origin |
| `sketch_point` | empty | A point or circle in that sketch, such as `c1` or `c1.center` |

The reply gives `slot`, `size`, `length`, `width`, `height` and `area_mm2`, the area of the section.

The package calls `selection.get`, `inspect.measure` (for the body names, so a new body gets a free one), `sketch.inspect`, `sketch.create`, `sketch.line`, `sketch.arc.three_point`, `sketch.circle.center`, `sketch.finish`, `solid.extrude`, and `component.list`, `component.create`, `component.metadata`, `component.metadata.set`, `component.activate`, `occurrence.copy`, `occurrence.move` and `occurrence.reparent` to place it.

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

What is not here: the closed-face rails (HFSF5, HFST5), the 30 and 40 mm series, and the V-slot 40x40 and C-Beam.

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

The catalog's areas are a loose check: its 2040 is 0.5% over MISUMI's own model and its 4040 is 1.5% under.

## Names

| What | Name |
|---|---|
| Rail body and component | `T-slot 2040 x 500`, `V-slot 2020 x 122.5` |
| Its sketch and feature | `T-slot 2040 x 500: Section`, `T-slot 2040 x 500: Rail` |
| The folder its placements go in | `Extrusions` |

In the bill of materials a rail is `purchased`, its part number is its name, and its description reads `20 x 40 mm T-slot aluminium extrusion, 500 mm long`.

## Older field names

None.
