# Bearings reference

The package offers two insertion workflows:

- `make-bearing`: fixed OpenBuilds and MISUMI radial bearing products.
- `make-csk`: CSK design specifications with editable keyways and recorded references.

## CSK workflow

Choose a designation, then leave keyway overrides blank to use the documented
Formsprag reference profile. Overrides change actual through-cut geometry. For
example, `CSK12P` with `inner_keyway_depth: 1.2` retains its 12 × 32 × 10 mm envelope
and 4 mm-wide keyway, with a 1.2 mm depth instead of the reference 1.8 mm.
The resulting part does not claim to be a Formsprag purchase. See the
[CSK example](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/examples/csk-clutches.md)
for a saved design, reproducible commands and expected BOM rows.

| Field | Default | Meaning |
|---|---|---|
| `size` | `CSK12P` | Preset designation; fixes bore, OD, width and which keyways exist |
| `inner_keyway_width` / `inner_keyway_depth` | Reference value | Optional positive millimetres; P and PP only |
| `outer_keyway_width` / `outer_keyway_depth` | Reference value | Optional positive millimetres; PP only |
| `sketch` / `sketch_point` | Empty | Placement, as described below |

Keyways face local +Y; widths are measured along X, depths radially from the
bore/OD apex. Slots pass through the full bearing width. Invalid dimensions,
slots that reach the opposite surface, and inner/outer slots that meet are rejected
before editing the model. A designation without an outer slot rejects outer-slot
overrides. Choose the appropriate P or PP preset first.

The preset list reflects the documented references, not every product sold under
CSK names. Exact envelope overrides and arbitrary new combinations are outside
this workflow. Internal sprags, races, balls and seal surfaces are not modeled;
2RS presets use their individually tabulated mounting widths. Geometry does not
simulate one-way motion or predict mass.

## Geometry and reference records

A CSK part saves its resolved geometry, including the selected designation and
actual keyway values, separately from reference evidence and the original profile.
No manufacturer, supplier or BOM part number is assigned by this workflow.

An unassigned CSK design still appears in the BOM. Repeated placements count as
occurrences of one definition. Different keyway values create distinct definitions;
an explicit override equal to the reference value reuses the default definition.

`org.techsquid.specification` metadata stores geometry, its source-neutral identity,
and reference evidence with schema version 2. Save/reopen preserves these records.
Purchasing assignments are a separate feature, outside this generator's inputs.

## Radial bearing workflow

`make-bearing` accepts `size` (default `625 2RS`), `sketch` and `sketch_point`.
It inserts fixed supplier products and rejects dimensional overrides. OpenBuilds
and MISUMI purchasing identities remain explicit for these catalog products.
Their separate rings and source-measured seals, shields or exposed raceways are
unchanged. Closed variants omit hidden balls; open B626 includes eight sourced
balls. Cages and closure thickness are not inferred.

## Placement and reuse

With no point selection, insert at the origin. The centre plane sits on the point,
with local Z along the sketch normal. Select sketch points or circle centres, or
address them with `sketch` and comma-separated `sketch_point` IDs. Up to 32 placements
are accepted; complex documents may require a smaller batch.

CSK components are named `CSK clutch <designation>` with `custom` appended when
keyway dimensions differ from the reference. Their single body ends in
`Mounting envelope`. Radial components use `Bearing <designation>`. Both workflows
place occurrences under the `Bearings` folder. Renamed definitions remain reusable;
edited geometry or metadata is preserved and a new insertion builds a clean
replacement. See [verified reuse](../../docs/parts-library.md).

## Presets and reference dimensions

<!-- catalog-table:start -->

| Designation | Supplier | Part number | Bore × OD × width (mm) | Closure / mounting |
|---|---|---|---|---|
| 625 2RS | OpenBuilds | 30 | 5 × 16 × 5 | Rubber seals, both sides |
| 105ZZ | OpenBuilds | 800 | 5 × 10 × 4 | Metal shields, both sides |
| B626 | MISUMI | B626 | 6 × 19 × 6 | Open, visible balls |
| B626ZZ | MISUMI | B626ZZ | 6 × 19 × 6 | Metal shields, both sides |
| B626DDU | MISUMI | B626DDU | 6 × 19 × 6 | Rubber seals, both sides |
| B608ZZ | MISUMI | B608ZZ | 8 × 22 × 7 | Metal shields, both sides |
| CSK8 | Unassigned (Formsprag reference) | — | 8 × 22 × 9 | Unkeyed |
| CSK12 | Unassigned (Formsprag reference) | — | 12 × 32 × 10 | Unkeyed |
| CSK15 | Unassigned (Formsprag reference) | — | 15 × 35 × 11 | Unkeyed |
| CSK17 | Unassigned (Formsprag reference) | — | 17 × 40 × 12 | Unkeyed |
| CSK20 | Unassigned (Formsprag reference) | — | 20 × 47 × 14 | Unkeyed |
| CSK25 | Unassigned (Formsprag reference) | — | 25 × 52 × 15 | Unkeyed |
| CSK30 | Unassigned (Formsprag reference) | — | 30 × 62 × 16 | Unkeyed |
| CSK35 | Unassigned (Formsprag reference) | — | 35 × 72 × 17 | Unkeyed |
| CSK40 | Unassigned (Formsprag reference) | — | 40 × 80 × 22 | Unkeyed |
| CSK8-2RS | Unassigned (Formsprag reference) | — | 8 × 22 × 9 | Unkeyed; one lip seal |
| CSK12-2RS | Unassigned (Formsprag reference) | — | 12 × 32 × 14 | Unkeyed; 2RS |
| CSK15-2RS | Unassigned (Formsprag reference) | — | 15 × 35 × 16 | Unkeyed; 2RS |
| CSK17-2RS | Unassigned (Formsprag reference) | — | 17 × 40 × 17 | Unkeyed; 2RS |
| CSK20-2RS | Unassigned (Formsprag reference) | — | 20 × 47 × 19 | Unkeyed; 2RS |
| CSK25-2RS | Unassigned (Formsprag reference) | — | 25 × 52 × 20 | Unkeyed; 2RS |
| CSK30-2RS | Unassigned (Formsprag reference) | — | 30 × 62 × 21 | Unkeyed; 2RS |
| CSK35-2RS | Unassigned (Formsprag reference) | — | 35 × 72 × 22 | Unkeyed; 2RS |
| CSK40-2RS | Unassigned (Formsprag reference) | — | 40 × 80 × 27 | Unkeyed; 2RS |
| CSK12P | Unassigned (Formsprag reference) | — | 12 × 32 × 10 | Inner keyway |
| CSK15P | Unassigned (Formsprag reference) | — | 15 × 35 × 11 | Inner keyway |
| CSK17P | Unassigned (Formsprag reference) | — | 17 × 40 × 12 | Inner keyway |
| CSK20P | Unassigned (Formsprag reference) | — | 20 × 47 × 14 | Inner keyway |
| CSK25P | Unassigned (Formsprag reference) | — | 25 × 52 × 15 | Inner keyway |
| CSK30P | Unassigned (Formsprag reference) | — | 30 × 62 × 16 | Inner keyway |
| CSK35P | Unassigned (Formsprag reference) | — | 35 × 72 × 17 | Inner keyway |
| CSK40P | Unassigned (Formsprag reference) | — | 40 × 80 × 22 | Inner keyway |
| CSK15PP | Unassigned (Formsprag reference) | — | 15 × 35 × 11 | Inner and outer keyways |
| CSK17PP | Unassigned (Formsprag reference) | — | 17 × 40 × 12 | Inner and outer keyways |
| CSK20PP | Unassigned (Formsprag reference) | — | 20 × 47 × 14 | Inner and outer keyways |
| CSK25PP | Unassigned (Formsprag reference) | — | 25 × 52 × 15 | Inner and outer keyways |
| CSK30PP | Unassigned (Formsprag reference) | — | 30 × 62 × 16 | Inner and outer keyways |
| CSK35PP | Unassigned (Formsprag reference) | — | 35 × 72 × 17 | Inner and outer keyways |
| CSK40PP | Unassigned (Formsprag reference) | — | 40 × 80 × 22 | Inner and outer keyways |
| CSK12P-2RS | Unassigned (Formsprag reference) | — | 12 × 32 × 14 | Inner keyway; 2RS |
| CSK15P-2RS | Unassigned (Formsprag reference) | — | 15 × 35 × 16 | Inner keyway; 2RS |
| CSK17P-2RS | Unassigned (Formsprag reference) | — | 17 × 40 × 17 | Inner keyway; 2RS |
| CSK20P-2RS | Unassigned (Formsprag reference) | — | 20 × 47 × 19 | Inner keyway; 2RS |
| CSK25P-2RS | Unassigned (Formsprag reference) | — | 25 × 52 × 20 | Inner keyway; 2RS |
| CSK30P-2RS | Unassigned (Formsprag reference) | — | 30 × 62 × 21 | Inner keyway; 2RS |
| CSK35P-2RS | Unassigned (Formsprag reference) | — | 35 × 72 × 22 | Inner keyway; 2RS |
| CSK40P-2RS | Unassigned (Formsprag reference) | — | 40 × 80 × 27 | Inner keyway; 2RS |
| CSK30PP-2RS | Unassigned (Formsprag reference) | — | 30 × 62 × 21 | Inner and outer keyways; 2RS |

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

CSK insertion now uses `make-csk`. `make-bearing` accepts radial bearings only;
there is no old-route alias. Catalog revision 6 requires refreshing saved resolved
selections. Previous supplier-identified CSK definitions are not reused as the new
source-neutral specifications. Their existing geometry is preserved. Ring details
in the Rust `Dimensions` API live under `section`.

Earlier PR revisions accepted `part_number` and `purchase_*` fields in `make-csk`.
These inputs are now rejected without aliases. Purchasing moves to a separate PR;
old saved definitions remain intact but are not adopted as current definitions.
