# Bearings reference

`make-bearing` inserts a fixed catalog product. It is the package’s only entry;
product dimensions, sources and placement conventions are documented below.

## Parameters

| Field | Default | Meaning |
|---|---|---|
| `size` | `625 2RS` | Bearing designation from the catalog |
| `sketch` | empty | Named sketch; otherwise use current selection |
| `sketch_point` | empty | Point or circle-center IDs, separated by commas |

No diameter, width or seal overrides are accepted. With no point selection, insert
at the origin. The local origin is the centre plane, with Z along the axle.
The internal descriptor defines `axle-centre`, `negative-seat` and `positive-seat` frames.
Choose up to 32 points; creation or inspection of a complex document can require a
smaller batch to stay within the host command limit.

## Sizes

<!-- catalog-table:start -->

| Designation | Supplier | Part number | Bore × OD × width (mm) | Closure / mounting |
|---|---|---|---|---|
| 625 2RS | OpenBuilds | 30 | 5 × 16 × 5 | Rubber seals, both sides |
| 105ZZ | OpenBuilds | 800 | 5 × 10 × 4 | Metal shields, both sides |
| B626 | MISUMI | B626 | 6 × 19 × 6 | Open, visible balls |
| B626ZZ | MISUMI | B626ZZ | 6 × 19 × 6 | Metal shields, both sides |
| B626DDU | MISUMI | B626DDU | 6 × 19 × 6 | Rubber seals, both sides |
| B608ZZ | MISUMI | B608ZZ | 8 × 22 × 7 | Metal shields, both sides |
| CSK8 | Formsprag | CSK8 | 8 × 22 × 9 | Unkeyed |
| CSK12 | Formsprag | CSK12 | 12 × 32 × 10 | Unkeyed |
| CSK15 | Formsprag | CSK15 | 15 × 35 × 11 | Unkeyed |
| CSK17 | Formsprag | CSK17 | 17 × 40 × 12 | Unkeyed |
| CSK20 | Formsprag | CSK20 | 20 × 47 × 14 | Unkeyed |
| CSK25 | Formsprag | CSK25 | 25 × 52 × 15 | Unkeyed |
| CSK30 | Formsprag | CSK30 | 30 × 62 × 16 | Unkeyed |
| CSK35 | Formsprag | CSK35 | 35 × 72 × 17 | Unkeyed |
| CSK40 | Formsprag | CSK40 | 40 × 80 × 22 | Unkeyed |
| CSK8-2RS | Formsprag | CSK8-2RS | 8 × 22 × 9 | Unkeyed; one lip seal |
| CSK12-2RS | Formsprag | CSK12-2RS | 12 × 32 × 14 | Unkeyed; 2RS |
| CSK15-2RS | Formsprag | CSK15-2RS | 15 × 35 × 16 | Unkeyed; 2RS |
| CSK17-2RS | Formsprag | CSK17-2RS | 17 × 40 × 17 | Unkeyed; 2RS |
| CSK20-2RS | Formsprag | CSK20-2RS | 20 × 47 × 19 | Unkeyed; 2RS |
| CSK25-2RS | Formsprag | CSK25-2RS | 25 × 52 × 20 | Unkeyed; 2RS |
| CSK30-2RS | Formsprag | CSK30-2RS | 30 × 62 × 21 | Unkeyed; 2RS |
| CSK35-2RS | Formsprag | CSK35-2RS | 35 × 72 × 22 | Unkeyed; 2RS |
| CSK40-2RS | Formsprag | CSK40-2RS | 40 × 80 × 27 | Unkeyed; 2RS |
| CSK12P | Formsprag | CSK12P | 12 × 32 × 10 | Inner keyway |
| CSK15P | Formsprag | CSK15P | 15 × 35 × 11 | Inner keyway |
| CSK17P | Formsprag | CSK17P | 17 × 40 × 12 | Inner keyway |
| CSK20P | Formsprag | CSK20P | 20 × 47 × 14 | Inner keyway |
| CSK25P | Formsprag | CSK25P | 25 × 52 × 15 | Inner keyway |
| CSK30P | Formsprag | CSK30P | 30 × 62 × 16 | Inner keyway |
| CSK35P | Formsprag | CSK35P | 35 × 72 × 17 | Inner keyway |
| CSK40P | Formsprag | CSK40P | 40 × 80 × 22 | Inner keyway |
| CSK15PP | Formsprag | CSK15PP | 15 × 35 × 11 | Inner and outer keyways |
| CSK17PP | Formsprag | CSK17PP | 17 × 40 × 12 | Inner and outer keyways |
| CSK20PP | Formsprag | CSK20PP | 20 × 47 × 14 | Inner and outer keyways |
| CSK25PP | Formsprag | CSK25PP | 25 × 52 × 15 | Inner and outer keyways |
| CSK30PP | Formsprag | CSK30PP | 30 × 62 × 16 | Inner and outer keyways |
| CSK35PP | Formsprag | CSK35PP | 35 × 72 × 17 | Inner and outer keyways |
| CSK40PP | Formsprag | CSK40PP | 40 × 80 × 22 | Inner and outer keyways |
| CSK12P-2RS | Formsprag | CSK12P-2RS | 12 × 32 × 14 | Inner keyway; 2RS |
| CSK15P-2RS | Formsprag | CSK15P-2RS | 15 × 35 × 16 | Inner keyway; 2RS |
| CSK17P-2RS | Formsprag | CSK17P-2RS | 17 × 40 × 17 | Inner keyway; 2RS |
| CSK20P-2RS | Formsprag | CSK20P-2RS | 20 × 47 × 19 | Inner keyway; 2RS |
| CSK25P-2RS | Formsprag | CSK25P-2RS | 25 × 52 × 20 | Inner keyway; 2RS |
| CSK30P-2RS | Formsprag | CSK30P-2RS | 30 × 62 × 21 | Inner keyway; 2RS |
| CSK35P-2RS | Formsprag | CSK35P-2RS | 35 × 72 × 22 | Inner keyway; 2RS |
| CSK40P-2RS | Formsprag | CSK40P-2RS | 40 × 80 × 27 | Inner keyway; 2RS |
| CSK30PP-2RS | Formsprag | CSK30PP-2RS | 30 × 62 × 21 | Inner and outer keyways; 2RS |

<!-- catalog-table:end -->

OpenBuilds and MISUMI are the identified suppliers for the radial bearings.
Formsprag supplies the CSK one-way clutches. The 42 CSK choices cover plain, 2RS, P, PP and P-2RS variants, plus CSK30PP-2RS.
Only combinations documented by the source are offered. P has an inner keyway;
PP has inner and outer keyways. Both slots face local +Y and pass through the full
width. Sealed variants use their individually tabulated widths.
An underlying manufacturer is not inferred from the OpenBuilds or MISUMI names.
The designation and dimensions do not establish a tolerance class, load rating,
seal compound or current stock availability. These are exterior CAD representations.

The [coverage checklist](../../docs/bearing-coverage.md) compares the radial bearings with
the main app's thirteen nominal sizes and records the exact references still needed.

## Geometry and sources

Each radial bearing contains separate inner/outer rings. Closed variants have two annular
surface bodies at the source's measured recess; rubber seals are dark and metal
shields are gray. Closures have zero thickness because the source does not establish
an internal cross-section. MISUMI profiles retain their 0.3 mm rounded edges; open
B626 includes the exposed raceway grooves and eight sourced balls, with no closures.
Closed bearings omit hidden balls and internals. Cages are not modeled. Volumes and masses therefore do not represent a
complete physical bearing; closure surfaces are intentionally not watertight solids.

CSK products are mounting envelopes with catalog bore, OD, width and actual
rectangular keyway cuts. Inner and outer slot widths and depths appear in the
[CSK source ledger](../../docs/sources/csk.md). Internal rings, sprags, balls, dust protection and directional clutch
behavior are not represented. The envelope is useful for placement and clearance,
not mass prediction or motion simulation. The supplied CSK8 STEP corroborates its
8 × 22 × 9 mm envelope; its internal geometry is not generalized to other sizes.
The [CSK source ledger](../../docs/sources/csk.md) records the supplied files and
the primary Formsprag catalog.

The [MISUMI source ledger](../../docs/sources/misumi.md) records the supplied B626,
B626ZZ, B626DDU and B608ZZ profiles. The 626 variants share their standard envelope
but have different ring boundaries and closure recesses; they are distinct products.

The old OpenBuilds mini bearing STEP is 4.2 mm wide. An OpenBuilds staff correction
specifies 4 mm; the catalog uses 4 mm while retaining the measured radial section
and recess. The [source ledger](../../docs/sources/openbuilds.md) records this correction,
original file hashes and the independently measured geometry. No source STEP is
required or distributed in the plugin.

| Data | Source | Validation |
|---|---|---|
| Exterior ring sections and recesses | OpenBuilds GrabCAD bearing STEP references in the source ledger | `bearings::tests::exterior_dimensions_match_source_and_mini_correction`; native `wheel_geometry_test.py` |
| MISUMI rounded profiles, closures and exposed B626 races | Exact supplied STEP revisions in the MISUMI source ledger | `bearings::tests::misumi_sections_match_supplied_source_revisions`; native `bearing_geometry_test.py` |
| 105ZZ width correction | OpenBuilds staff, 27 February 2016, linked in the source ledger | Nominal width and closure bounds in `wheel_geometry_test.py` |
| Supplier SKUs 30 and 800 | OpenBuilds Sphinx build BOM, linked in the source ledger | `bearings::tests::supplier_variants_do_not_conflate_closure_or_identity` |

## Names and bill of materials

A component is named `Bearing <designation>`, for example `Bearing B626ZZ`. Radial bearing
bodies end in `Inner ring`, `Outer ring`, `Seal front/back`, `Shield front/back`, or
`Ball 1`–`Ball 8`. A CSK body ends in `Mounting envelope`.
Standalone placements go in the phantom `Bearings` folder. The BOM counts one
purchase per bearing occurrence, using the supplier and exact part number (such as
`OpenBuilds 30`, `MISUMI B626ZZ` or `Formsprag CSK8`), regardless of body count. Wheel assemblies and direct insertion share the same radial bearing
definitions; the Bearings plugin need not be installed to build a wheel assembly.

Renaming a component preserves reuse. Geometry, appearance, material, purchasing
metadata or child edits invalidate its saved content check. The edited component
remains intact and subsequent insertions use a clean definition. See
[verified reuse](../../docs/parts-library.md).

## Older field names

This is a new workflow. Unknown fields are rejected.

The CSK catalog uses the Formsprag reference profile. CSK is a common family name,
but vendor keyways can differ: Formsprag CSK12P uses 4 × 1.8 mm, while PEAK lists
4 × 1.2 mm. Check the purchased part's drawing. The chooser labels remain `CSK…`;
the supplier in the BOM identifies the modeled reference product.

**Breaking change:** catalog revision 4 requires refreshing saved resolved selections.
CSK configurations now store the exact variant under the `formsprag-csk` family;
previous serialized CSK configurations are rejected without aliases. Existing
model geometry is preserved, while new insertions use the current identity.
