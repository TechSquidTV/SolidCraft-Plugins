# Bearing coverage and completion tracker

Parity means covering the main app's nominal bearing sizes with identified,
purchasable products. A base size, a supplier part number and a CAD representation
are different things in the shared catalog contract.

## Scope and baseline

The [SolidCraft bearing table at commit 0930970](https://github.com/tangency-labs/solidcraft/blob/0930970ce03b43f8d9e6477cb0903d4a713f17f4/crates/engine/src/cmd/parts.rs#L235)
lists thirteen envelopes. This is a dated comparison baseline, not a runtime table
or evidence of a particular manufacturer's ring or seal geometry.

The original products cover 625, 626 and 608. The completed rows below expand
that coverage to all thirteen baseline envelopes. OpenBuilds 105ZZ is additional
coverage used by mini wheels. The [generated product table](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/plugins/techsquid-bearings/REFERENCE.md#presets-and-reference-dimensions)
is authoritative for products currently included.

The initial expansion targets one sourced, double-shielded MISUMI product for each
missing envelope below. This matches the supplier and closure family already used
by B626ZZ and B608ZZ. Completion means all thirteen baseline envelopes have at
least one implemented and validated supplier product; it does not mean every seal,
shield, material or manufacturer variant is covered.

## Radial bearing coverage

Audit date: 2026-11-17. MISUMI's catalog identifies all ten target part numbers and
their nominal envelopes. The [catalog evidence record](sources/misumi.md#catalog-evidence-for-the-coverage-expansion)
records the exact PDF revision and what it establishes. All ten supplier STEP files
were received and measured on 2026-11-17; no further references are needed for
this baseline expansion.

The supplied references share one parametric section. The new products use the
existing closed-bearing builder with their measured dimensions. The [sample comparison](sources/misumi.md#shared-section-and-inference-limits)
records why their detail dimensions cannot yet be derived reliably for other sizes.
For future products, a supplier dimension table or documented family formula can
also supply these values; a separate CAD file is not mandatory for every row.

| Base size | MISUMI target | Bore x OD x width, mm | Product / envelope evidence | Exterior CAD or section drawing | Measurements | Native model / catalog | Validation |
|---|---|---|---|---|---|---|---|
| 6000 | B6000ZZ | 10 x 26 x 8 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6001 | B6001ZZ | 12 x 28 x 8 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6002 | B6002ZZ | 15 x 32 x 9 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6003 | B6003ZZ | 17 x 35 x 10 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6004 | B6004ZZ | 20 x 42 x 12 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6200 | B6200ZZ | 10 x 30 x 9 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6201 | B6201ZZ | 12 x 32 x 10 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6202 | B6202ZZ | 15 x 35 x 11 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6203 | B6203ZZ | 17 x 40 x 12 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |
| 6204 | B6204ZZ | 20 x 47 x 14 | Catalog confirmed | [STEP received](sources/misumi-files.json) | [Measured](sources/misumi-measurements.json) | [Implemented](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/83e189866e11b26b19c3b9639c8e08bc48220dbb/crates/bearings/src/lib.rs) | [Passed](https://github.com/TechSquidTV/SolidCraft-Plugins/pull/19) |

Replace each status with a link to its evidence or completion record as work lands.
"Catalog confirmed" establishes the historical supplier designation and envelope,
not current availability or verified exterior geometry. Keep the target SKU on the
download or drawing so another brand, closure or stainless variant is not mistaken
for the same product.

## Reference requirements

All ten target STEP archives are recorded in the supplier ledger. The reference
requirements below apply to products advertised with measured exteriors. See [Closure coverage](#closure-coverage) for estimated variants.

The reference package must establish:

- **Identity:** supplier, exact SKU and closure on both sides, product/download URL,
  source date or revision, original filename, byte count and SHA-256.
- **Envelope:** bore, outside diameter and overall width, cross-checked against
  the catalog row. Resolve any disagreement before building.
- **Visible section:** inner-ring outer radius, outer-ring inner radius, shield
  recess on each side, and actual edge radii/chamfers. Preserve asymmetry if
  present. The catalog's mounting dimensions and minimum chamfer allowance do not
  supply these nominal CAD measurements.
- **Independent checks:** imported bounds, body layout, volume where meaningful,
  section vertices and analytic radii. A dimensioned drawing can replace STEP if
  it specifies the complete visible section. A table or documented rule must supply
  the same parameters; derive expected geometry from that evidence.

The existing [MISUMI evidence ledger](sources/misumi.md) and
[OpenBuilds ledger](sources/openbuilds.md) remain valid for their recorded products.
No new references are needed for those unchanged models. A catalog envelope,
product photo or generic bearing STEP alone cannot establish an exact SKU's
visible ring and closure geometry. Keep suffixes attached to the source's brand.

## Closure coverage

All fourteen base sizes offer Shielded and Sealed. The additional thirteen closed
variants use the other closure's measured section at the same size. Classify the
entire exterior as estimated and record its donor. Assign a supplier number only
when the catalog confirms the exact closure and envelope. The sealed 105 is a
supplier-neutral specification. The [MISUMI ledger](sources/misumi.md#catalog-identities-for-estimated-closure-variants)
records the catalog evidence and the limits of this estimate.

Open requires evidence for the exposed rings, raceways and balls. Only B626
qualifies. A single-sided shield such as [NTN 626Z](https://us.misumi-ec.com/vona2/detail/221000058378/?HissuCode=626Z) also requires a source for its exposed side.
The [OpenBuilds ledger](sources/openbuilds.md#bearings-and-the-mini-width-correction)
records the outstanding references for the 8 x 16 x 5 and inch-bearing candidates.

## Completion checklist

The table records per-product completion. Measured dimensions belong in the
supplier evidence records and the domain crate.

- [x] Acquire and identify the reference; retain original files outside Git and
  record STEP revisions/hashes in `docs/sources/misumi-files.json`. Record a drawing
  revision/hash in the supplier ledger when a drawing supplies the section.
- [x] Record STEP measurements in `docs/sources/misumi-measurements.json` and describe
  discrepancies or representation limits in the supplier ledger. Use
  `tools/measure_parts.py` with an unsigned SolidCraft CLI after checking the
  file's units and axis against the tool's assumptions. Record drawing-derived
  measurements and calculations in the supplier ledger if no STEP is supplied.
- [x] Add the product to `crates/bearings`, reusing its section builder and product
  identity methods. Add independent source-table tests. Expand the builder only
  where an acquired reference requires a new section shape.
- [x] Regenerate choices and product tables with
  `cargo run -p catalog-tool -- --write`; update affected guide text.
- [x] Extend `tests/bearing_geometry_test.py` with source-derived bounds, ring
  sections, shield areas/recesses, meaningful volumes, purchasing identity, reuse
  and command-budget checks. Run the applicable host suites listed in
  [DEVELOPMENT.md](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/DEVELOPMENT.md).
- [x] Regenerate affected examples and pictures and rerun their checks; verify
  existing bearings and V-wheel assemblies still retain their product identities.
- [x] Pass formatting, native/wasm clippy, workspace tests and `./build.sh` before
  committing. Mark Validation complete only after required host checks pass with
  no skips and the example/picture work is complete.

The additions preserve existing product IDs and definitions.
**Breaking-change warning:** if reference review requires correcting an existing
definition, bump its representation/provider revision and document that existing
saved instances are not migrated. Regenerate affected examples; do not introduce
compatibility aliases or silently reuse the old geometry.

## Adding products without duplication

`crates/bearings` remains the owner of product identity, closure type, measurements
and native geometry. Both the Bearings workflow and wheel recipes use its factory.
A product is one row of `products!` in `crates/bearings/src/lib.rs`: its saved identity,
optional supplier, catalog key, base size, closure and measured section or donor.
The `sizes!` table in `crates/bearings/src/selection.rs` defines each supported envelope once.
`crates/bearings/src/build.rs` builds the rings and closure faces for radial bearings
and CSK clutches alike.
Keep measured and estimated exteriors distinct in the catalog and retain each sourced purchasing identity. Extend the
catalog-generated choices/reference table and independent geometry tests together.
Visible balls are included for sourced open bearings; sealed and shielded variants
omit hidden balls. Cages remain outside this representation. Never fill an open
bearing with an invented closure.

The [CSK series](sources/csk.md) is a separate one-way clutch family. Its 55
reference presets provide separate rings and closures with editable envelopes and
keyways. Eight supplied STEP references establish exterior measurements; other
variants use explicitly recorded estimates.
They do not count toward the radial-bearing parity table above. The source ledger
identifies each preset baseline; the generator does not assign a purchasing identity.
