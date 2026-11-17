# Bearings

Place fixed OpenBuilds and MISUMI bearing products or customizable CSK one-way clutches in [SolidCraft](https://github.com/tangency-labs/solidcraft). The radial bearings have separate steel rings and sourced closures. CSK presets provide mounting geometry with editable keyways and recorded references.

![Six sourced bearings with open, sealed and shielded variants](bearings.png)

## Install

Download `techsquid-bearings-0.3.0.zip` from the repository's releases. In SolidCraft, choose **Plugins > Install plugin…** and select the zip. Check the [host requirements](../../docs/parts-library.md#host-requirements) before installing version 0.3.

## Use

Select sketch points or circle centres, then choose a workflow:

- **Bearings > make-bearing** inserts a fixed OpenBuilds or MISUMI radial bearing.
- **Bearings > make-csk** starts with a CSK designation. Leave keyway fields blank
  for the reference dimensions, or enter overrides. For example, CSK12P can use
  a 4 × 1.2 mm inner keyway while keeping its 12 × 32 × 10 mm envelope.

CSK parts start without an assigned supplier or BOM part number. Optionally enter
a supplier and its SKU together, plus your internal BOM number. Listing URLs are
separate metadata: updating a link preserves the definition and BOM quantity.
Conflicting internal numbers are rejected before insertion. Drawing references
remain separate. See [purchasing rules](../../docs/purchasing.md) and the
[purchasing example](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/examples/csk-purchasing.md).

With nothing selected, the part sits at the origin. Its centre plane sits on the
point and its axle follows the sketch normal. Optional **Sketch** and **Sketch
point** fields address points by name.

Each radial bearing’s rings and closure faces belong to one purchased component. Later insertions reuse an unchanged definition, including bearings already inside V-wheel assemblies. Edited definitions are preserved; a new insertion builds a clean replacement. New occurrences remain free to move or join to your assembly.

The built-in bearing tool provides a simple ring envelope. This plugin shows the sourced exterior ring and closure boundaries, with visible balls and raceways in open B626. CSK clutches use mounting envelopes with through keyways for P and PP variants; their internal sprags, rings and dust protection are not modeled. The CAD does not simulate one-way motion or predict mass. It does not use the built-in **Change Part Size** workflow.

See the [reference](REFERENCE.md) for dimensions, part numbers, placement fields and representation limits. Shared internal definitions drive the insertion choices and verified reuse; no Catalog entry is exposed. The example designs open without the plugin installed.

## CSK example

![Five CSK mounting envelopes comparing reference and custom inner keyways, an outer keyway and a wider sealed variant](csk-clutches.png)

The [CSK walkthrough and saved design](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/examples/csk-clutches.md) show a reference CSK12P, two copies with a 1.2 mm-deep inner keyway, CSK15PP and CSK15P-2RS. The repeated custom part counts twice in the BOM; no supplier is inferred from its reference drawing.
