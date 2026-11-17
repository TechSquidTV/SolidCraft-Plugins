# CSK geometry references

CSK is a family designation used by multiple suppliers. The references do not
establish a universal product standard or geometric interchangeability. DIN 6885
references concern keyways, not complete clutch specifications. Reference publishers
are not assigned as manufacturers or sellers by the generator.

## Preset coverage and defaults

The generator contains 55 reference presets. P means an inner keyway; PP means both
inner and outer keyways. Sealed variants have independently tabulated widths.

| Bore (mm) | Plain / P / PP | 2RS / P-2RS / PP-2RS | Plain bore × OD × width (mm) | Sealed width (mm) |
|---|---|---|---|---|
| 8 | All three | 2RS and P-2RS | 8 × 22 × 9 | 9 |
| 10 | All three | No verified preset | 10 × 30 × 9 | — |
| 12 | All three | 2RS and P-2RS | 12 × 32 × 10 | 14 |
| 15 | All three | All three | 15 × 35 × 11 | 16 |
| 17 | All three | All three | 17 × 40 × 12 | 17 |
| 20 | All three | All three | 20 × 47 × 14 | 19 |
| 25 | All three | All three | 25 × 52 × 15 | 20 |
| 30 | All three | All three | 30 × 62 × 16 | 21 |
| 35 | All three | All three | 35 × 72 × 17 | 22 |
| 40 | All three | All three | 40 × 80 × 22 | 27 |

This matrix describes implemented reference presets, not a claim that omitted
combinations do not exist. Custom mode and dimensional/keyway overrides can model
other combinations, including supplier-specific envelopes, without inventing a
reference or asserting catalog availability. The dropdown, README and reference
tables are generated from the same compiled preset list.

Default evidence, reviewed November 17, 2026:

- The original 42 presets retain the [Formsprag P-1352-FC](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)
  envelope values and [MCC-P-956-FC-EN-US 11/25, pages 28–29](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf)
  keyed geometry. This includes CSK30PP-2RS and the 4 × 1.8 mm CSK12P inner slot.
- [PEAK's catalog](https://www.peakbearings.com/onewaybrg.htm) and
  [keyway table](https://www.peakbearings.com/keywaysize.htm) establish CSK8P,
  CSK8PP and CSK12PP. Size 8 uses inner 3 × 0.5 and outer 2 × 0.5 mm;
  size 12PP uses inner 4 × 1.2 and outer 2 × 0.6 mm.
- [Yuki's CSK table](https://www.ywsbearing.com/25x52x15mm-internal-and-external-keyway-without-groove-One-way-bearing-CSK25PP-BB25-CSK6205-CSK25P-CSK25-pd46892679.html)
  supplies the size-10 envelope and inner 3 × 0.6 / outer 2 × 0.5 mm keyways.
  PEAK independently lists CSK10, CSK10P and CSK10PP at 10 × 30 × 9 mm.
  The size-10 sealed width is not inferred from an unsealed table.
- [DKC's catalog, page 3](https://dkc-as.dk/wp-content/uploads/2015/05/DKC-FRIL%C3%98BSLEJER-ENVEJSLEJER-FREEWHEEL-BEARINGS-ONEWAY-CLUTCHES-needle-clutch-brg.pdf)
  explicitly supplies CSK8P-2RS at 8 × 22 × 9 mm, with a 3 × 0.5 mm inner slot.
  Its outer-slot values are not substituted for PEAK's unsealed 8PP baseline.
- [PTI's CSK series sheet](https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf)
  documents PP from size 15 upward, sealed widths, and the PP-2RS suffix combination.
  It supplies the added 15, 17, 20, 25, 35 and 40PP-2RS presets. The
  [PTI CSK20PP.2RS product listing](https://shop.pti.eu/en/friloebslejer-og-tilbageloebsspaerre/freewheel-bearing-w-dot--keyway-in-inner--and-outer-ring-kk20pp-dot-2rs)
  also confirms 20 × 47 × 19 mm and both keyways.

For sizes 15–40, default inner keyways are respectively 5 × 1.2, 5 × 1.2,
6 × 1.6, 8 × 2, 8 × 2, 10 × 2.4 and 12 × 3.3 mm. Outer keyways are
2 × 0.6, 2 × 1, 3 × 1.5, 6 × 2, 6 × 2, 8 × 2.5 and 10 × 3 mm.
These nominal dimensions are editable, including the presence of either slot.

## Geometry and evidence limits

Slots are rectangular and through the full width. Inner keyways default to local
+Y (0°); outer keyways default to -Y (180°), matching the supplied 15PP/40PP CAD.
Each angle can be overridden counterclockwise from +Y as viewed from +Z.
Depth is radial from the bore/OD apex; width is perpendicular to that direction. The model
shows separate rings and closure faces, not a mass model: hidden raceways, balls,
sprags, seal cross-sections, tolerances and one-way motion are not represented. Torque and load ratings are not
assigned to generic or modified geometry. Current stock was not checked.

Presets retain their original reference when customized. Custom mode has no catalog preset; its estimated exterior still records the CAD
sources used for its proportions. Purchasing assignment remains future shared-component work.

The seller table supplied earlier matches [PEAK's WCB6205-2RS table](https://www.peakbearings.com/WCB6205-2RS.htm)
and cannot establish the seller's manufacturer. WCB6205-2RS is 25 × 52 × 15 mm,
not the 20 mm-wide CSK25-2RS, and is not a catalog alias here.

## Supplied reference files

- Formsprag P-1352-FC, March 2018, four pages: SHA-256
  `a125f118c7aafabb5b76097284f9b3b49b3f8dee9566a265037cc52a2be712a1`.
- Stieber P-7426-SC-A4 9/17, two-page keyed extract: SHA-256
  `55807df529d39d6d7ee245dadf8fb5a8da0f9ea6aabd82a838b7c2ab69198cb9`.
- CSK8.zip: SHA-256
  `9a48350e6fe7f65b71927b25b03df56186b0f092264aaa82da44baffb24be963`.
  Its STEP AP214 model, `FREILAUF_CSK8_CDS_ASM`, confirms 8 mm bore, 22 mm OD
  and 9 mm axial extent. It does not establish other sizes' internal geometry.


## Supplied exterior CAD and estimation

The eight-file [source ledger](csk-files.json) contains the STEP and ZIP hashes,
envelopes and measured exterior sections. The [native measurement report](csk-measurements.json)
records imported bodies and each source solid's axial/radial vertices. Source
STEP files remain external; generated models use native geometry.

| Supplied model | Inner ring outside radius | Outer ring inside radius | Inner chamfer, radial × axial | Outer chamfer, radial × axial |
|---|---:|---:|---|---|
| CSK8 | 5.720 | 9.000 | 0.181985 × 0.5 | 0.363970 × 1.0 |
| CSK12-2RS | 9.105 | 13.295 | 0.363970 × 1.0 | 0.363970 × 1.0 |
| CSK15P, CSK15PP, CSK15-2RS | 11.055 | 15.245 | 0.363970 × 1.0 | 0.363970 × 1.0 |
| CSK40P, CSK40PP, CSK40-2RS | 28.515 | 33.495 | 1.273896 × 3.5 | 1.273896 × 3.5 |

All dimensions are millimetres. The outermost cover/seal faces lie at ±half-width
in all eight references: recess is zero. Chamfers are approximately 20 degrees,
not 45-degree or rounded edges. Small numerical deviations in imported keyed
edges are normalized using the unkeyed 2RS ring surfaces and the nominal envelope.
The supplied PP inner slots face +Y and outer slots face -Y in source coordinates
(with source X as the bearing axis); the generator preserves their 180° relative
orientation, with editable clocking. The 15 mm and 40 mm P/PP ring boundaries agree; 2RS uses the same boundaries with
its wider catalog width. Hidden raceways are intentionally filled within each
ring envelope; closure thickness is not modeled. The imported cage has geometry
warnings, so its surfaces are not used to generate the exterior.

Only the eight exact supplied designations at their measured envelope are marked
**measured**. Other variants are marked **estimated**, including transfers between
variants of the same size. Radial shoulder and chamfer dimensions are normalized
by the radial space between bore and OD, interpolated linearly by bore between
8, 12, 15 and 40 mm, and applied to the requested envelope. Outside that range,
the nearest reference's radial proportions are retained. Axial chamfer lengths
are interpolated in millimetres and limited to one quarter of the requested width.
For thin rings, radial chamfers are limited to 45% of ring thickness.

A keyway must leave a radial web behind its floor/corners. When necessary, the
ring is widened using a modeling allowance of 2% of the total radial space;
this is an estimation rule, not a physical design rating. A remaining closure
gap of at least that allowance is required. Adapted sections are marked estimated,
with `adapted_for_keyways` recorded. Closure style can be overridden independently;
it changes visible faces and colour, not the envelope width.

Geometry metadata records resolved exterior dimensions, basis, method, source
filenames/hashes and keyway adaptation. Original catalog references remain separate
from these CAD-derived exterior references and from future purchasing assignment.

Reproduce the source measurement report with the extracted, hash-matching files:

```sh
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tools/measure_parts.py docs/sources/csk-files.json /path/to/extracted/files > docs/sources/csk-measurements.json
```
