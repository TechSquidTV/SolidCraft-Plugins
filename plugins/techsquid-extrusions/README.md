# Extrusions

Check the [host requirements](../../docs/parts-library.md#host-requirements) before installing version 0.3.

Go from a sketch to a frame with configurable T-slot and V-slot rails for [SolidCraft](https://github.com/tangency-labs/solidcraft). Cut rails to length, prepare their joints and add slot nuts without modeling each profile by hand.

![T-slot and V-slot rail profiles](rails.png)

- Build MISUMI 5 series T-slot rails in 2020, 2040, 2060, 2080 and 4040 sizes, or OpenBuilds V-slot rails from 2020 through 2080.
- Add M5 tapped ends and screw holes for rail-to-rail connections. Edit a hole reference sketch to move both the access and clearance cuts together.
- Add MISUMI M3, M4 or M5 pre-assembly or post-assembly slot nuts to your rails.
- Place multiple fixed-length copies as reusable components, or create independently editable rails along sketch lines.

## Get started

Open **Plugins > Extrusion** and pick one or more sketch points or circles for **Placement** in the view. Points or circles already selected when the dialog opens fill the field. Choose a profile, size and length, then press **Run**. With nothing picked, the rail starts at the origin. The default is a 500 mm T-slot 2020 rail.

## Sketch-driven rails

![Paired T-slot 2020, T-slot 4040 and V-slot 2080 rails with path-driven machining](sketch-driven-rails.png)

Choose **Sketch driven** in the Extrusion dialog and pick straight lines from one
sketch for **Placement**. Each run supports up to 16 lines, all with the chosen
profile. Machining reduces the batch
size under SolidCraft’s command limit; oversized batches are rejected before
creation. Run again for another profile or another batch.
All listed T-slot and V-slot sizes are supported.

Edit the source sketch to move, tilt or resize each rail independently. **Length**
is ignored in this mode. The section and machining share planes derived from the
path, so taps and cross-holes follow without referring to generated rail faces.
**Hole distances from** chooses Start or End for all screw-hole offsets in that run.
Start and End follow the line's drawing direction.

Sketch-driven rails are separate bodies in the active component, grouped in the
timeline. They do not share fixed-length component definitions. Their profile uses
the path-normal construction plane's axes; this differs from Weldments' world-up
orientation. Use the generated section sketch to inspect its X/Y orientation.

Paths must be drawn, non-construction straight lines in a 2D sketch; the sketch can
lie on any plane. Arcs, projected lines and 3D sketch wires are not accepted. The
picked line set is captured when you run the plugin: adding another line later does
not add a rail until you run the plugin on it. Deleting a source line breaks its rail.

At creation, each rail must be 0.01–6000 mm long and fit its requested machining.
Keep it long enough for its holes and taps after editing. A run can model up to four
taps across all rails; use cosmetic threads or smaller batches for more. Geometry
kernel failures roll the plugin run back.

Open the [six-rail example](../../examples/sketch-driven-rails.json): two T-slot 2020,
two T-slot 4040 and two V-slot 2080 rails, each with end-relative cross-holes.
The 2020 rails have modeled M5 taps; the larger profiles use cosmetic taps.

Try these parameter edits in SolidCraft:

- Set `Rail_Tilt` from `-35 mm` to `35 mm` to reverse the lean.
- Set `Rail_Rise` from `150 mm` to `190 mm` to lengthen every path while preserving their different lengths.
- Change `Rail_X` or `Rail_Z` to move the layout.

The holes remain 25 mm and 55 mm from each path's end. Keep the horizontal
endpoint dimensions positive; for example, use `Rail_X = 90 mm` for a combined edit.

This is an additional mode. Existing fixed-length instances are not converted.

For a slot nut, sketch a point at the center of the slot on the rail's face, open **Plugins > Slot nut** and pick the point for **Placement**. Choose **Size** for the screw's M3, M4 or M5 thread, then set **Rail runs along** to match the slot.

![M3 pre-assembly and M4 post-assembly slot nuts in a transparent rail](slot-nuts.png)

The M3 pre-assembly nut is on the left; the M4 post-assembly nut is on the right. Their outer shapes follow their nut styles, while the hole sizes follow **Size**.

For joint dimensions, tapping limits, slot nut placement and all options, see the [extrusion reference](REFERENCE.md).

## License

[MIT](../../LICENSE)

The package exposes its insertion tools only. [Shared part-definition and reuse rules](../../docs/parts-library.md) explain product identity, modified-part preservation and BOM grouping.
