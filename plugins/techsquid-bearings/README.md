# Bearings

Place fixed OpenBuilds and MISUMI bearing products or Formsprag CSK one-way clutches in [SolidCraft](https://github.com/tangency-labs/solidcraft). The radial bearings have separate steel rings and sourced closures. CSK products provide mounting envelopes. Choose a catalog designation instead of entering arbitrary dimensions.

![Six sourced bearings with open, sealed and shielded variants](bearings.png)

## Install

Download `techsquid-bearings-0.3.0.zip` from the repository's releases. In SolidCraft, choose **Plugins > Install plugin…** and select the zip. Check the [host requirements](../../docs/parts-library.md#host-requirements) before installing version 0.3.

## Use

Select sketch points or circle centers, then run **Bearings > make-bearing**. Choose a listed supplier product: OpenBuilds, MISUMI, or an unkeyed Formsprag CSK8–CSK40 clutch. With nothing selected, the part sits at the origin. Its centre plane sits on the point and its axle follows the sketch normal. Optional **Sketch** and **Sketch point** fields address points by name.

The rings and any closure faces belong to one purchased component. Later insertions reuse an unchanged definition, including bearings already inside V-wheel assemblies. Edited definitions are preserved; a new insertion builds a clean replacement. New occurrences remain free to move or join to your assembly.

The built-in bearing tool provides a simple ring envelope. This plugin shows the sourced exterior ring and closure boundaries, with visible balls and raceways in open B626. CSK clutches use one annular mounting envelope per size; their internal sprags, rings and dust protection are not modeled. The CAD does not simulate one-way motion or predict mass. It does not use the built-in **Change Part Size** workflow.

See the [reference](REFERENCE.md) for dimensions, part numbers, placement fields and representation limits. Shared internal definitions drive the insertion choices and verified reuse; no Catalog entry is exposed. The example design opens without the plugin installed.
