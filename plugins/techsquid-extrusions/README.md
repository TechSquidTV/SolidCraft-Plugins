# Extrusions

Go from a sketch to a frame with configurable T-slot and V-slot rails for [SolidCraft](https://github.com/tangency-labs/solidcraft). Cut rails to length, prepare their joints and add slot nuts without modeling each profile by hand.

![T-slot and V-slot rail profiles](rails.png)

- Build MISUMI 5 series T-slot rails in 2020, 2040, 2060, 2080 and 4040 sizes, or OpenBuilds V-slot rails from 2020 through 2080.
- Add M5 tapped ends and screw holes for rail-to-rail connections. Edit a hole reference sketch to move both the access and clearance cuts together.
- Add MISUMI pre-assembly or post-assembly slot nuts to your rails.
- Place multiple fixed-length copies as reusable components, or create independently editable rails along sketch lines.

## Get started

Select one or more sketch points or circles and open **Plugins > Extrusion**. Choose a profile, size and length, then press **Run**. Leave **Sketch** and **Sketch point** blank to use your selection; with no selection, the rail starts at the origin. The default is a 500 mm T-slot 2020 rail.

## Sketch-driven rails

Choose **Sketch driven** in the Extrusion dialog. Select straight lines from one
sketch, or enter its name in **Sketch**. **Path lines** accepts ids such as `l1, l2`;
leave it empty with a named sketch to use all its drawn straight lines. Each run
supports up to 16 lines, all with the chosen profile. Machining reduces the batch
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
chosen line set is captured when you run the plugin: adding another line later does
not add a rail until you run the plugin on it. Deleting a source line breaks its rail.

At creation, each rail must be 0.01–6000 mm long and fit its requested machining.
Keep it long enough for its holes and taps after editing. A run can model up to four
taps across all rails; use cosmetic threads or smaller batches for more. Geometry
kernel failures roll the plugin run back.

Open the [four-rail example](../../examples/sketch-driven-rails.json) and edit
`Rail_Rise`, `Rail_Tilt`, `Rail_X` or `Rail_Z` to try it.

This is an additional mode. Existing fixed-length instances are not converted.

For a slot nut, sketch a point at the center of the slot on the rail's face, select it and open **Plugins > Slot nut**. Set the rail direction to match the slot.

![Pre-assembly and post-assembly slot nuts in a transparent rail](slot-nuts.png)

For joint dimensions, tapping limits, slot nut placement and all options, see the [extrusion reference](REFERENCE.md).

## License

[MIT](../../LICENSE)
