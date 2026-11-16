# Bearings reference

`make-bearing` inserts a fixed catalog product. `catalog` advertises its dimensions,
sources, attachment frames and insertion route without reading or editing a design.

## Parameters

| Field | Default | Meaning |
|---|---|---|
| `size` | `625 2RS` | Bearing designation from the catalog |
| `sketch` | empty | Named sketch; otherwise use current selection |
| `sketch_point` | empty | Point or circle-center IDs, separated by commas |

No diameter, width or seal overrides are accepted. With no point selection, insert
at the origin. The local origin is the centre plane, with Z along the axle.
The catalog advertises `axle-centre`, `negative-seat` and `positive-seat` frames.
Choose up to 32 points; creation or inspection of a complex document can require a
smaller batch to stay within the host command limit.

## Sizes

<!-- catalog-table:start -->

| Designation | Supplier | Part number | Bore × OD × width (mm) | Closure |
|---|---|---|---|---|
| 625 2RS | OpenBuilds | 30 | 5 × 16 × 5 | Rubber seals, both sides |
| 105ZZ | OpenBuilds | 800 | 5 × 10 × 4 | Metal shields, both sides |
| B626 | MISUMI | B626 | 6 × 19 × 6 | Open, visible balls |
| B626ZZ | MISUMI | B626ZZ | 6 × 19 × 6 | Metal shields, both sides |
| B626DDU | MISUMI | B626DDU | 6 × 19 × 6 | Rubber seals, both sides |
| B608ZZ | MISUMI | B608ZZ | 8 × 22 × 7 | Metal shields, both sides |

<!-- catalog-table:end -->

OpenBuilds and MISUMI are the identified suppliers. An underlying bearing
manufacturer is not inferred from the supplier name.
The designation and dimensions do not establish a tolerance class, load rating,
seal compound or current stock availability. These are exterior CAD representations.

The [coverage checklist](../../docs/bearing-coverage.md) compares these products with
the main app's thirteen nominal sizes and records the exact references still needed.

## Geometry and sources

Each bearing contains separate inner/outer rings. Closed variants have two annular
surface bodies at the source's measured recess; rubber seals are dark and metal
shields are gray. Closures have zero thickness because the source does not establish
an internal cross-section. MISUMI profiles retain their 0.3 mm rounded edges; open
B626 includes the exposed raceway grooves and eight sourced balls, with no closures.
Closed bearings omit hidden balls and internals. Cages are not modeled. Volumes and masses therefore do not represent a
complete physical bearing; closure surfaces are intentionally not watertight solids.

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

A component is named `Bearing <designation>`, for example `Bearing B626ZZ`. Its bodies end in
`Inner ring`, `Outer ring`, `Seal front/back`, `Shield front/back`, or `Ball 1`–`Ball 8`.
Standalone placements go in the phantom `Bearings` folder. The BOM counts one
purchase per bearing occurrence, using the supplier and exact part number (such as
`OpenBuilds 30` or `MISUMI B626ZZ`), regardless of body count. Wheel assemblies and direct insertion share the same bearing
definitions; the Bearings plugin need not be installed to build a wheel assembly.

Renaming a component preserves reuse. Geometry, appearance, material, purchasing
metadata or child edits invalidate its saved content check. The edited component
remains intact and subsequent insertions use a clean definition. See
[verified reuse](../../docs/parts-library.md).

## Older field names

This is a new workflow. Unknown fields are rejected.
