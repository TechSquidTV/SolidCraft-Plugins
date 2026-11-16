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

| Designation | OpenBuilds SKU | Bore × OD × width (mm) | Closure |
|---|---|---|---|
| 625 2RS | 30 | 5 × 16 × 5 | Rubber seals, both sides |
| 105ZZ | 800 | 5 × 10 × 4 | Metal shields, both sides |

<!-- catalog-table:end -->

OpenBuilds is the supplier. The underlying bearing manufacturer is not identified.
The designation and dimensions do not establish a tolerance class, load rating,
seal compound or current stock availability. These are exterior CAD representations.

## Geometry and sources

Each bearing contains solid inner/outer ring envelopes and two annular surface
bodies at a 0.3 mm recess. The 625 has dark rubber seal faces; the 105ZZ has metal
shield faces. Closures have zero thickness because the source does not establish
an internal cross-section. Balls, cages, internal raceways and unsupported edge
chamfers are omitted. Volumes and masses therefore do not represent a complete
physical bearing; surfaces are intentionally not watertight solids.

The old OpenBuilds mini bearing STEP is 4.2 mm wide. An OpenBuilds staff correction
specifies 4 mm; the catalog uses 4 mm while retaining the measured radial section
and recess. The [source ledger](../../docs/sources/openbuilds.md) records this correction,
original file hashes and the independently measured geometry. No source STEP is
required or distributed in the plugin.

| Data | Source | Validation |
|---|---|---|
| Exterior ring sections and recesses | OpenBuilds GrabCAD bearing STEP references in the source ledger | `bearings::tests::exterior_dimensions_match_source_and_mini_correction`; native `wheel_geometry_test.py` |
| 105ZZ width correction | OpenBuilds staff, 27 February 2016, linked in the source ledger | Nominal width and closure bounds in `wheel_geometry_test.py` |
| Supplier SKUs 30 and 800 | OpenBuilds Sphinx build BOM, linked in the source ledger | `bearings::tests::supplier_variants_do_not_conflate_closure_or_identity` |

## Names and bill of materials

A component is named `Bearing 625 2RS` or `Bearing 105ZZ`. Its bodies end in
`Inner ring`, `Outer ring`, `Seal front/back`, or `Shield front/back`.
Standalone placements go in the phantom `Bearings` folder. The BOM counts one
purchase per bearing occurrence, using `OpenBuilds 30` or `OpenBuilds 800`, regardless
of its four bodies. Wheel assemblies and direct insertion share the same bearing
definitions; the Bearings plugin need not be installed to build a wheel assembly.

Renaming a component preserves reuse. Geometry, appearance, material, purchasing
metadata or child edits invalidate its saved content check. The edited component
remains intact and subsequent insertions use a clean definition. See
[verified reuse](../../docs/parts-library.md).

## Older field names

This is a new workflow. Unknown fields are rejected.
