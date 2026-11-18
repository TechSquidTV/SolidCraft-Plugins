# Bearings reference

The package offers two insertion workflows:

- `make-bearing`: radial bearings selected by base size or nominal dimensions and closure.
- `make-csk`: CSK design specifications with editable keyways and recorded references.

## CSK workflow

Choose a designation, find a reference preset with **By dimensions**, or select **Custom**.
Each named choice shows the exterior basis of its reference preset. Blank custom dimensions
retain the selected reference values. Overrides change the geometry.
For example, CSK12P with `inner_keyway_depth: 1.2` changes its default 1.8 mm depth.
The resulting part does not claim to be a purchase from the reference publisher.

| Field | Default | Meaning |
|---|---|---|
| `size` | `CSK12P` | A listed CSK designation, `By dimensions`, or `Custom` |
| `match_bore` / `match_diameter` / `match_width` | Empty | By dimensions only. Enter one or more nominal dimensions in mm. |
| `custom_bore` / `custom_diameter` / `custom_width` | Preset value | Bore, outside diameter and width in millimetres; all three required for Custom |
| `inner_keyway` / `outer_keyway` | `Preset` | `Preset`, `Enabled`, or `None`; Custom starts without slots |
| `inner_keyway_width` / `inner_keyway_depth` | Existing value | Positive millimetres; both required when adding an inner slot |
| `outer_keyway_width` / `outer_keyway_depth` | Existing value | Positive millimetres; both required when adding an outer slot |
| `inner_keyway_angle` / `outer_keyway_angle` | 0° / 180° | Optional independent clocking in degrees; values wrap at 360°. Requires an enabled keyway. |
| `closure` | `Preset` | `Preset`, `Dust covers`, `Lip seals`, or `Single lip seal`; Custom starts with dust covers. Width is edited separately. |
| `placement` | Empty | Placement, as described below |

**By dimensions** searches the reference envelopes. Leave Closure and either keyway on
Preset to allow any value for that field. An explicit closure or keyway choice filters
the candidates. A 15 mm bore and 11 mm width fit CSK15, CSK15P and CSK15PP. Setting
Outer keyway to Enabled selects CSK15PP. Preview and Run list ambiguous matches and
report a search with no match. Use Custom for an unlisted clutch.

A match starts from the same designation, source and geometry as picking it directly.
Custom dimensions and keyway dimensions or angles edit that baseline after selection.
The saved exterior record describes the resulting section. A base designation or Custom
ignores the three match dimensions.

To add a slot to an unkeyed preset, choose Enabled and provide width and depth.
To remove a slot, choose None and clear its dimension overrides. A disabled slot
rejects dimensions rather than silently ignoring them. Custom mode requires bore,
outside diameter and width; no arbitrary default envelope is supplied.

Keyways pass through the full width. Inner slots default to +Y (0°), outer slots
to -Y (180°), matching the supplied 15PP/40PP references. Angles are independent
and counterclockwise from +Y as viewed from +Z; whole turns normalize for reuse.
Depth is measured radially from the bore/OD apex, width perpendicular to that
direction. Partial-length slots are not modeled.
Dimensions must be positive and finite, the bore smaller than the OD, and slots
must leave material between the bore and OD and between each other. Validation
runs before host edits.

Presets cover documented combinations, not every product sold under CSK names.
The 10 mm sealed family and 8/12 mm PP-2RS combinations lack a fully verified
baseline here; use Custom or edit a documented preset using your part drawing.
Each CSK has two separate chamfered steel rings and two annular cover/seal faces.
Keyways cut only their respective ring. The supplied CAD has flush closure faces;
no recess is invented. Colours distinguish covers from rubber seals but do not
assign materials. Size-8 sealed presets show one seal and one cover. Internal
sprags, raceways, balls and seal cross-sections remain omitted. These are exterior
representations, unsuitable for mass prediction or one-way motion simulation.

Eight supplied STEP files establish measured exterior profiles. Other sizes use
interpolated radial proportions; custom envelopes use the same estimation rule.
The generated table identifies measured versus estimated exteriors.
Deep custom slots widen the respective ring when necessary; combinations leaving
no room for a closure are rejected. The [source ledger](../../docs/sources/csk.md)
describes the measurements and interpolation limits.

## Geometry and reference records

A CSK part saves its actual geometry separately from the original reference baseline
and source URL. A changed preset retains its designation only as its starting point
and is marked custom; it does not claim that the modified shape is a catalog part.
Custom mode saves a null designation and null reference. The generator assigns no
manufacturer, supplier or BOM part number.

Repeated placements with the same starting designation and resolved dimensions
reuse one definition. Explicit overrides equal to defaults also reuse that preset.
Different starting designations remain distinct, even when dimensions coincide;
equal mounting geometry does not establish identical physical bearings.

`org.techsquid.specification` schema version 4 stores geometry, its source-neutral
identity, source, baseline and customization status, plus the exterior dimensions, measured/estimated basis,
source filenames and hashes. Save/reopen preserves these
records. **Breaking change:** the representation revision is now 3; newly generated
CSK definitions do not reuse older definitions. Existing saved geometry remains
intact. There are no legacy field aliases or migration fallbacks.

Purchasing assignment remains
[future work for a shared component workflow](../../docs/parts-library.md#future-component-purchasing-workflow)
after creation, outside this generator's inputs.

## Radial bearing workflow

| Field | Default | Meaning |
|---|---|---|
| `size` | `608` | Base size as a string, or `By dimensions` |
| `closure` | `Shielded` | `Shielded`, `Sealed` or `Open`. Only 626 supports Open. |
| `match_bore`, `match_diameter`, `match_width` | Empty | Nominal bore, outside diameter and width in mm. Used only with By dimensions. |
| `placement` | Empty | Sketch picks as described below |

By dimensions needs at least one dimension. Each value must be greater than 0, at most 1000 mm,
and match the catalog exactly. Switching to a base size ignores the numeric values in these fields.
A 17 mm bore matches 6003 and 6203. Adding a 40 mm outside diameter selects 6203.
Multiple or zero matches produce a message during Preview or on Run. Matching and
choosing the base size directly create the same component.

All sizes offer shielded and sealed closures. For the thirteen new closed variants,
both rings, edge radii and the closure recess come from the measured variant of
the same size. Each estimated component records the section, estimation method and
donor identity in `org.techsquid.bearing.exterior` version 1. Twelve have verified
MISUMI catalog numbers. The sealed 105 is a design specification with a normal BOM
row and an empty part number. See the [catalog evidence](../../docs/sources/misumi.md#catalog-identities-for-estimated-closure-variants).

Open B626 has eight visible balls and measured raceways. Closed models omit hidden
balls. All closure faces have zero thickness.

## Placement and reuse

Pick sketch points or circles for **Placement** in the view; a circle, or a click
inside one, means its centre. Points or circles already selected when the dialog opens fill the field.
With nothing picked, insert at the origin. The centre plane sits on the point, with
local Z along the sketch normal. Up to 32 placements are accepted; complex documents
may require a smaller batch.

Programs pass a list of references, as the dialog does: `{"type": "external_sketch_point", "sketch": 2, "id": "p1"}` for a point, `{"type": "external_sketch_curve", "sketch": 2, "id": "c1"}` for a circle, `{"type": "profile", "sketch": 2, "index": 0}` for the inside of one. `sketch` is the sketch's feature id or its name. A region that is not the inside of a circle is refused against Placement, and so is an edge, face or body.

CSK components are named `CSK clutch <designation>` with `custom` appended when
any dimensions, closure style or keyway configuration differ from the reference. Their bodies are named `Inner ring`, `Outer ring`, and `Cover` or `Seal`
with a front/back suffix. These four bodies remain one BOM component. Radial components use `Bearing <designation>`. Both workflows
place occurrences under the `Bearings` folder. Renamed definitions remain reusable;
edited geometry or metadata is preserved and a new insertion builds a clean
replacement. See [verified reuse](../../docs/parts-library.md).

## Presets and reference dimensions

<!-- catalog-table:start -->

| Designation | Supplier | Part number | Bore × OD × width (mm) | Closure / mounting | Exterior |
|---|---|---|---|---|---|
| 625 2RS | OpenBuilds | 30 | 5 × 16 × 5 | Rubber seals, both sides | Measured |
| 105ZZ | OpenBuilds | 800 | 5 × 10 × 4 | Metal shields, both sides | Measured |
| B626 | MISUMI | B626 | 6 × 19 × 6 | Open, visible balls | Measured |
| B626ZZ | MISUMI | B626ZZ | 6 × 19 × 6 | Metal shields, both sides | Measured |
| B626DDU | MISUMI | B626DDU | 6 × 19 × 6 | Rubber seals, both sides | Measured |
| B608ZZ | MISUMI | B608ZZ | 8 × 22 × 7 | Metal shields, both sides | Measured |
| B6000ZZ | MISUMI | B6000ZZ | 10 × 26 × 8 | Metal shields, both sides | Measured |
| B6001ZZ | MISUMI | B6001ZZ | 12 × 28 × 8 | Metal shields, both sides | Measured |
| B6002ZZ | MISUMI | B6002ZZ | 15 × 32 × 9 | Metal shields, both sides | Measured |
| B6003ZZ | MISUMI | B6003ZZ | 17 × 35 × 10 | Metal shields, both sides | Measured |
| B6004ZZ | MISUMI | B6004ZZ | 20 × 42 × 12 | Metal shields, both sides | Measured |
| B6200ZZ | MISUMI | B6200ZZ | 10 × 30 × 9 | Metal shields, both sides | Measured |
| B6201ZZ | MISUMI | B6201ZZ | 12 × 32 × 10 | Metal shields, both sides | Measured |
| B6202ZZ | MISUMI | B6202ZZ | 15 × 35 × 11 | Metal shields, both sides | Measured |
| B6203ZZ | MISUMI | B6203ZZ | 17 × 40 × 12 | Metal shields, both sides | Measured |
| B6204ZZ | MISUMI | B6204ZZ | 20 × 47 × 14 | Metal shields, both sides | Measured |
| B625ZZ | MISUMI | B625ZZ | 5 × 16 × 5 | Metal shields, both sides | Estimated |
| 105 sealed | Unassigned | — | 5 × 10 × 4 | Rubber seals, both sides | Estimated |
| B608DDU | MISUMI | B608DDU | 8 × 22 × 7 | Rubber seals, both sides | Estimated |
| B6000DDU | MISUMI | B6000DDU | 10 × 26 × 8 | Rubber seals, both sides | Estimated |
| B6001DDU | MISUMI | B6001DDU | 12 × 28 × 8 | Rubber seals, both sides | Estimated |
| B6002DDU | MISUMI | B6002DDU | 15 × 32 × 9 | Rubber seals, both sides | Estimated |
| B6003DDU | MISUMI | B6003DDU | 17 × 35 × 10 | Rubber seals, both sides | Estimated |
| B6004DDU | MISUMI | B6004DDU | 20 × 42 × 12 | Rubber seals, both sides | Estimated |
| B6200DDU | MISUMI | B6200DDU | 10 × 30 × 9 | Rubber seals, both sides | Estimated |
| B6201DDU | MISUMI | B6201DDU | 12 × 32 × 10 | Rubber seals, both sides | Estimated |
| B6202DDU | MISUMI | B6202DDU | 15 × 35 × 11 | Rubber seals, both sides | Estimated |
| B6203DDU | MISUMI | B6203DDU | 17 × 40 × 12 | Rubber seals, both sides | Estimated |
| B6204DDU | MISUMI | B6204DDU | 20 × 47 × 14 | Rubber seals, both sides | Estimated |
| CSK8 | Unassigned | — | 8 × 22 × 9 | Unkeyed ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Measured |
| CSK8P | Unassigned | — | 8 × 22 × 9 | Inner keyway ([reference](https://www.peakbearings.com/keywaysize.htm)) | Estimated |
| CSK8PP | Unassigned | — | 8 × 22 × 9 | Inner and outer keyways ([reference](https://www.peakbearings.com/keywaysize.htm)) | Estimated |
| CSK8-2RS | Unassigned | — | 8 × 22 × 9 | Unkeyed; one lip seal ([reference](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf)) | Estimated |
| CSK8P-2RS | Unassigned | — | 8 × 22 × 9 | Inner keyway; one lip seal ([reference](https://dkc-as.dk/wp-content/uploads/2015/05/DKC-FRIL%C3%98BSLEJER-ENVEJSLEJER-FREEWHEEL-BEARINGS-ONEWAY-CLUTCHES-needle-clutch-brg.pdf)) | Estimated |
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

## Sources

The [CSK ledger](../../docs/sources/csk.md) records all dimensions and vendor
comparisons. For example, Formsprag lists a CSK12P inner keyway of 4 × 1.8 mm;
PEAK lists 4 × 1.2 mm. CSK is a common family designation, not proof of complete
interchangeability. Enter the actual purchased part's keyway measurements when needed.

The [MISUMI ledger](../../docs/sources/misumi.md) records the supplied radial bearing
profiles; the [OpenBuilds ledger](../../docs/sources/openbuilds.md) includes the
105ZZ width correction to 4 mm. Source STEP files are not required at runtime.

## Breaking changes

**Breaking change for radial scripts.** Replace product-valued `size` with a base size
and closure. For example, `{"size":"B6203ZZ"}` becomes
`{"size":"6203","closure":"Shielded"}`. Old values are rejected. To reproduce the old
empty-input default, pass `{"size":"625","closure":"Sealed"}`. The new default is
608 Shielded. Direct plugin runs can omit both fields. Clients invoking the generated
tool pass the two required choices. Refresh catalog selections pinned before revision 11.

CSK insertion now uses `make-csk`. `make-bearing` accepts radial bearings only;
there is no old-route alias. Previous supplier-identified CSK definitions are not reused as the new
source-neutral specifications. Their existing geometry is preserved. Rust callers use
`bearing.dimensions()` for the mounting envelope and `bearing.section()` for ring details.

Earlier PR revisions accepted `part_number` and `purchase_*` fields in `make-csk`.
These inputs are now rejected without aliases. Old saved definitions remain intact
but are not adopted as current definitions.

Both workflows: `sketch` and `sketch_point` are retired and rejected as unsupported fields. Use `placement`, a list of picked sketch points or curves; typed comma-separated ids are gone.
