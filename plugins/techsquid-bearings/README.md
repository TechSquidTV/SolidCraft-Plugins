# Bearings

Place fixed OpenBuilds and MISUMI bearing products in [SolidCraft](https://github.com/tangency-labs/solidcraft), with separate steel rings and recessed rubber seals or metal shields. Choose a catalog designation instead of entering arbitrary dimensions.

![Six sourced bearings with open, sealed and shielded variants](bearings.png)

## Install

Download `techsquid-bearings-0.3.0.zip` from the repository's releases. In SolidCraft, choose **Plugins > Install plugin…** and select the zip. Use SolidCraft 0.42 or newer.

## Use

Select sketch points or circle centers, then run **Bearings > make-bearing**. Choose a listed supplier product: the original OpenBuilds bearings or MISUMI B626, B626ZZ, B626DDU and B608ZZ. With nothing selected, the bearing sits at the origin. Its centre plane sits on the point and its axle follows the sketch normal. Optional **Sketch** and **Sketch point** fields address points by name.

The rings and any closure faces belong to one purchased component. Later insertions reuse an unchanged definition, including bearings already inside V-wheel assemblies. Edited definitions are preserved; a new insertion builds a clean replacement. New occurrences remain free to move or join to your assembly.

The built-in bearing tool provides a simple ring envelope. This plugin shows the exterior ring and closure boundaries, with visible balls and raceways in open B626. Closed variants omit hidden balls and internals; cages are not modeled. It does not use the built-in **Change Part Size** workflow.

See the [reference](REFERENCE.md) for dimensions, part numbers, placement fields and representation limits. The [catalog contract](../../docs/catalog-contract.md) documents the read-only discovery entry for scripts. The example design opens without the plugin installed.
