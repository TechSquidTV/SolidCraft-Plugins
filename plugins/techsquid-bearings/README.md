# Bearings

Place fixed OpenBuilds and MISUMI bearing products or customizable CSK one-way clutches in [SolidCraft](https://github.com/tangency-labs/solidcraft). The radial bearings have separate steel rings and sourced closures. CSK presets have separate chamfered rings, cover or seal faces, editable keyways, and recorded references. Supplied STEP files establish eight exterior profiles; other presets use estimates based on those measurements.

![Radial bearings, left to right: OpenBuilds 625 2RS and 105ZZ; MISUMI B626, B626ZZ, B626DDU and B608ZZ](bearings.png)

The six radial bearings are shown left to right at the same scale, in the order
of the first six rows of the [supported-parts table](#supported-bearings-and-csk-presets).
Dark faces are rubber seals, gray faces are metal shields, and the open B626 shows
its balls and raceways. See the [reference](REFERENCE.md#sources) for sources and
modeling limits.

## Install

Download `techsquid-bearings-0.3.0.zip` from the repository's releases. In SolidCraft, choose **Plugins > Install plugin…** and select the zip. Check the [host requirements](../../docs/parts-library.md#host-requirements) before installing version 0.3.

## Use

Select sketch points or circle centres, then choose a workflow:

- **Bearings > make-bearing** inserts a fixed OpenBuilds or MISUMI radial bearing.
- **Bearings > make-csk** starts with a CSK designation. Leave dimension fields blank
  for reference values, override bore/OD/width, or add, remove, resize and rotate either keyway.
  Closure style can also be changed independently of width.
  Choose Custom to enter all dimensions without a reference preset. For example, CSK12P can use
  a 4 × 1.2 mm inner keyway while keeping its 12 × 32 × 10 mm envelope.

CSK parts have no assigned manufacturer, supplier or BOM part number. Drawing
references remain separate from geometry. Custom and unsourced parts still count
in the BOM. Purchasing assignment is
[future work for a shared component workflow](../../docs/parts-library.md#future-component-purchasing-workflow)
after creation.

With nothing selected, the part sits at the origin. Its centre plane sits on the
point and its axle follows the sketch normal. Optional **Sketch** and **Sketch
point** fields address points by name.

Each radial bearing’s rings and closure faces belong to one purchased component. Later insertions reuse an unchanged definition, including bearings already inside V-wheel assemblies. Edited definitions are preserved; a new insertion builds a clean replacement. New occurrences remain free to move or join to your assembly.

The built-in bearing tool provides a simple ring envelope. This plugin shows the sourced exterior ring and closure boundaries, with visible balls and raceways in open B626. CSK keyways cut the appropriate ring. Cover and seal faces follow the measured flush boundaries; dark seal colours are illustrative. Hidden sprags, raceways and seal cross-sections are omitted. The CAD does not simulate one-way motion or predict mass. It does not use the built-in **Change Part Size** workflow.

See the [reference](REFERENCE.md) for dimensions, part numbers, placement fields and representation limits. Shared internal definitions drive the insertion choices and verified reuse; no Catalog entry is exposed. The example designs open without the plugin installed.

## CSK example

![Eight CSK exterior models with reference, custom, small keyed and sealed variants](csk-clutches.png)

The [CSK walkthrough and saved design](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/examples/csk-clutches.md) show a reference CSK12P, two copies with a 1.2 mm-deep inner keyway, CSK15PP, CSK15P-2RS, keyed 8/10 mm presets, and a fully custom outer-keyed part. The repeated custom part counts twice in the BOM; no supplier is inferred from its reference drawing.

## Supported bearings and CSK presets

CSK presets are reference geometry, with no purchasing identity. Envelope dimensions, closure style and keyways can be customized. Custom mode has no catalog baseline; its exterior proportions are estimated. The Exterior column describes ring boundaries and edge chamfers, not internal construction.

<!-- catalog-table:start -->

| Designation | Supplier | Part number | Bore × OD × width (mm) | Closure / mounting | Exterior |
|---|---|---|---|---|---|
| 625 2RS | OpenBuilds | 30 | 5 × 16 × 5 | Rubber seals, both sides | Measured |
| 105ZZ | OpenBuilds | 800 | 5 × 10 × 4 | Metal shields, both sides | Measured |
| B626 | MISUMI | B626 | 6 × 19 × 6 | Open, visible balls | Measured |
| B626ZZ | MISUMI | B626ZZ | 6 × 19 × 6 | Metal shields, both sides | Measured |
| B626DDU | MISUMI | B626DDU | 6 × 19 × 6 | Rubber seals, both sides | Measured |
| B608ZZ | MISUMI | B608ZZ | 8 × 22 × 7 | Metal shields, both sides | Measured |
| CSK8 | Unassigned | — | 8 × 22 × 9 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Measured |
| CSK8P | Unassigned | — | 8 × 22 × 9 | Inner keyway ([reference](https://www.peakbearings.com/keywaysize.htm)) | Estimated |
| CSK8PP | Unassigned | — | 8 × 22 × 9 | Inner and outer keyways ([reference](https://www.peakbearings.com/keywaysize.htm)) | Estimated |
| CSK8-2RS | Unassigned | — | 8 × 22 × 9 | Unkeyed; one lip seal ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK8P-2RS | Unassigned | — | 8 × 22 × 9 | Inner keyway; 2RS ([reference](https://dkc-as.dk/wp-content/uploads/2015/05/DKC-FRIL%C3%98BSLEJER-ENVEJSLEJER-FREEWHEEL-BEARINGS-ONEWAY-CLUTCHES-needle-clutch-brg.pdf)) | Estimated |
| CSK10 | Unassigned | — | 10 × 30 × 9 | Unkeyed ([reference](https://www.ywsbearing.com/25x52x15mm-internal-and-external-keyway-without-groove-One-way-bearing-CSK25PP-BB25-CSK6205-CSK25P-CSK25-pd46892679.html)) | Estimated |
| CSK10P | Unassigned | — | 10 × 30 × 9 | Inner keyway ([reference](https://www.ywsbearing.com/25x52x15mm-internal-and-external-keyway-without-groove-One-way-bearing-CSK25PP-BB25-CSK6205-CSK25P-CSK25-pd46892679.html)) | Estimated |
| CSK10PP | Unassigned | — | 10 × 30 × 9 | Inner and outer keyways ([reference](https://www.ywsbearing.com/25x52x15mm-internal-and-external-keyway-without-groove-One-way-bearing-CSK25PP-BB25-CSK6205-CSK25P-CSK25-pd46892679.html)) | Estimated |
| CSK12 | Unassigned | — | 12 × 32 × 10 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK12P | Unassigned | — | 12 × 32 × 10 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK12PP | Unassigned | — | 12 × 32 × 10 | Inner and outer keyways ([reference](https://www.peakbearings.com/keywaysize.htm)) | Estimated |
| CSK12-2RS | Unassigned | — | 12 × 32 × 14 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Measured |
| CSK12P-2RS | Unassigned | — | 12 × 32 × 14 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK15 | Unassigned | — | 15 × 35 × 11 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK15P | Unassigned | — | 15 × 35 × 11 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Measured |
| CSK15PP | Unassigned | — | 15 × 35 × 11 | Inner and outer keyways ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Measured |
| CSK15-2RS | Unassigned | — | 15 × 35 × 16 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Measured |
| CSK15P-2RS | Unassigned | — | 15 × 35 × 16 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK15PP-2RS | Unassigned | — | 15 × 35 × 16 | Inner and outer keyways; 2RS ([reference](https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf)) | Estimated |
| CSK17 | Unassigned | — | 17 × 40 × 12 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK17P | Unassigned | — | 17 × 40 × 12 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK17PP | Unassigned | — | 17 × 40 × 12 | Inner and outer keyways ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK17-2RS | Unassigned | — | 17 × 40 × 17 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK17P-2RS | Unassigned | — | 17 × 40 × 17 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK17PP-2RS | Unassigned | — | 17 × 40 × 17 | Inner and outer keyways; 2RS ([reference](https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf)) | Estimated |
| CSK20 | Unassigned | — | 20 × 47 × 14 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK20P | Unassigned | — | 20 × 47 × 14 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK20PP | Unassigned | — | 20 × 47 × 14 | Inner and outer keyways ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK20-2RS | Unassigned | — | 20 × 47 × 19 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK20P-2RS | Unassigned | — | 20 × 47 × 19 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK20PP-2RS | Unassigned | — | 20 × 47 × 19 | Inner and outer keyways; 2RS ([reference](https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf)) | Estimated |
| CSK25 | Unassigned | — | 25 × 52 × 15 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK25P | Unassigned | — | 25 × 52 × 15 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK25PP | Unassigned | — | 25 × 52 × 15 | Inner and outer keyways ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK25-2RS | Unassigned | — | 25 × 52 × 20 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK25P-2RS | Unassigned | — | 25 × 52 × 20 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK25PP-2RS | Unassigned | — | 25 × 52 × 20 | Inner and outer keyways; 2RS ([reference](https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf)) | Estimated |
| CSK30 | Unassigned | — | 30 × 62 × 16 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK30P | Unassigned | — | 30 × 62 × 16 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK30PP | Unassigned | — | 30 × 62 × 16 | Inner and outer keyways ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK30-2RS | Unassigned | — | 30 × 62 × 21 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK30P-2RS | Unassigned | — | 30 × 62 × 21 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK30PP-2RS | Unassigned | — | 30 × 62 × 21 | Inner and outer keyways; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK35 | Unassigned | — | 35 × 72 × 17 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK35P | Unassigned | — | 35 × 72 × 17 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK35PP | Unassigned | — | 35 × 72 × 17 | Inner and outer keyways ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK35-2RS | Unassigned | — | 35 × 72 × 22 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK35P-2RS | Unassigned | — | 35 × 72 × 22 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK35PP-2RS | Unassigned | — | 35 × 72 × 22 | Inner and outer keyways; 2RS ([reference](https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf)) | Estimated |
| CSK40 | Unassigned | — | 40 × 80 × 22 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK40P | Unassigned | — | 40 × 80 × 22 | Inner keyway ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Measured |
| CSK40PP | Unassigned | — | 40 × 80 × 22 | Inner and outer keyways ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Measured |
| CSK40-2RS | Unassigned | — | 40 × 80 × 27 | Unkeyed; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Measured |
| CSK40P-2RS | Unassigned | — | 40 × 80 × 27 | Inner keyway; 2RS ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)) | Estimated |
| CSK40PP-2RS | Unassigned | — | 40 × 80 × 27 | Inner and outer keyways; 2RS ([reference](https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf)) | Estimated |

<!-- catalog-table:end -->
