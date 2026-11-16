# OpenBuilds geometry and product evidence

Verified on 2026-11-15. This ledger distinguishes nominal CAD, product descriptions,
and manufacturing fits. The supplied STEP files are measurement references; their
bytes are not packaged or required at runtime. The Rust libraries construct native
profiles from dimensions. No third-party CAD code is used.

## Source revisions and repeatable measurement

[openbuilds-files.json](openbuilds-files.json) identifies all eleven supplied files
by original filename, archive, SHA-256, byte count and GrabCAD page. The user
identified these as official OpenBuilds downloads. GrabCAD's public pages did not
expose enough metadata to independently verify authorship or redistribution terms;
no redistribution permission is asserted and no STEP files are committed.

[openbuilds-measurements.json](openbuilds-measurements.json) records the imported
body bounds, volumes and standalone section vertex inventories. Reproduce with:

```sh
SOLIDCRAFT_CLI=/path/to/unsigned/solidcraft-cli python3 tools/measure_parts.py docs/sources/openbuilds-files.json /path/to/extracted/files
```

The script checks each source hash before importing with SolidCraft. It retains no
private STEP header paths. Standalone files use millimetres and the X axis; section
coordinates in the ledger are `[axial distance from centre, radius]`. Our native
parts use +Z along the axle, with the origin in the centre plane. The assembly STEP
already uses Z. Measured source volumes describe the source CAD, not measured mass.

## Wheel profiles

| Product | OD | Width | Bearing seat | Central opening | Shoulder width |
|---|---:|---:|---:|---:|---:|
| Solid V Wheel | 23.90 | 10.20 | 16.00 | 14.00 | 1.00 |
| Xtreme Solid V Wheel | 23.90 | 10.20 | 16.00 | 14.00 | 1.00 |
| Delrin V Wheel, grooved | 23.90 | 10.23 | 16.00 | 13.90 | 1.00 |
| Delrin Mini V Wheel | 15.23 | 8.80 | 10.00 | 8.64 | 1.00 |
| Xtreme Mini V Wheel | 15.23 | 8.80 | 10.00 | 8.64 | 1.00 |

All dimensions are mm from the identified STEP revisions. Equal profiles share one
geometry definition in code, while material variants remain different products.
The supplied Metal V Wheel has the same nominal section as the grooved Delrin
wheel. Its material grade and intended rail compatibility remain unverified, so it
is a recorded candidate rather than a published product.

The [J Tech Solid V Wheel listing](https://jtechphotonics.com/?product=solid-v-wheel-kit-for-v-slot-linear-rail)
describes black Delrin and quotes 10.23 mm width; the supplied Solid STEP measures
10.20 mm. These are retained as a source discrepancy, not explained away as a
tolerance. The user's two drawings also differ: one gives 23.89/10.23 mm and a
15.97 mm seat; the other gives 23.90/10.20 mm, a 16 mm seat and 4.60 mm seat depth.
The latter agrees with the Solid and Xtreme Solid STEP sections.

The [MakerTechStore mini wheel listing](https://www.makertechstore.com/products/mini-v-wheel-kits)
identifies black Delrin and clear polycarbonate variants and quotes an 8.8 mm width
with a 9.974 ± 0.026 mm seat. That fit spans 9.948–10.000 mm. The native CAD uses the
10 mm STEP seat; it does not pretend to simulate a press fit. Display colours and
transparency are illustrative, not measured optical properties.

## Bearings and the mini width correction

| OpenBuilds product | Supplier SKU | Bore × OD × width | Inner ring OD | Outer ring ID | Closure face recess |
|---|---|---|---:|---:|---:|
| 625 2RS | 30 | 5 × 16 × 5 | 8.6 | 13.6 | 0.3 |
| 105ZZ | 800 | 5 × 10 × 4 | 6.2 | 9.2 | 0.3 |

Supplier identities and nominal sizes appear in the official
[Sphinx parts list](https://builds.openbuilds.com/builds/openbuilds-sphinx-1050-20-x-40.7196/).
The supplier is OpenBuilds; the bearing manufacturer, rubber compound, steel grade,
internal clearances and precision class are not identified. Do not imply those
properties from a generic designation.

The supplied mini STEP is **4.2 mm wide**. In
[post 40, dated 2016-02-27](https://builds.openbuilds.com/threads/first-design-c-beam-router.5939/page-2),
OpenBuilds staff member Ronald van Arkel explicitly corrects the shop's 4.2 mm
specification to 4 mm for the 105ZZ, and the assembled mini wheel to 9 mm. Our 105ZZ
therefore uses 4 mm overall width. Its sourced radial boundaries and 0.3 mm closure
recess are retained, placing the shield faces at ±1.7 mm. This combines corrected
catalog width with the supplier's simplified exterior reference; it is not a new
manufacturer-specific shield drawing. No 4.2 mm product is advertised.

The STEP bearings are single filled solids with recessed annular closure faces,
not detailed bearing internals. Our representation separates the visible inner
ring, outer ring and both closure faces. Rings are solid envelopes and closures
are zero-thickness annular surface bodies: dark rubber seals for 625 2RS and metal
shields for 105ZZ. Balls, cages, internal raceways and hidden seal thickness are
omitted. No chamfer or gap is invented where the supplied reference has none.
These models support external layout and appearance, not bearing mass, internal
contact, section-cut manufacturing detail or rolling kinematics.

The supplied 8 × 16 × 5 and 0.25 × 0.625 × 0.196 inch files are measured but remain
candidates. Their exact supplier designations and closure variants need verified
product evidence before publication; dimensions alone do not establish identity.

## Installed wheel assemblies

The supplied assembly contains a wheel, two bearings, one 10 × 5 × 1 mm shim and an
M5 nylon lock nut. Its axial ranges are:

| Body | Z range, mm |
|---|---|
| Wheel | −5.1 to +5.1 |
| Internal shim | −0.5 to +0.5 |
| Bearings | −5.5 to −0.5 and +0.5 to +5.5 |
| External lock nut | −10.5 to −5.5 |

The wheel core recipe is one wheel, two occurrences of the same bearing definition
and one internal shim. External nut, axle, spacer and any extra shim belong to the
mounting arrangement. The J Tech retail pack includes two shims; that does not put
both inside the wheel. OpenBuilds staff also distinguish the internal shim from
the one against the spacer in [New Eccentrics, post 3](https://builds.openbuilds.com/threads/new-eccentrics.376/).

The full-size shim is OpenBuilds SKU 05, 10 mm OD × 5 mm ID × 1 mm thick, identified
in the [MiniMill parts list](https://builds.openbuilds.com/builds/openbuilds-minimill.5087/)
and independently measured in the assembly. The mini shim is SKU 170 in the Sphinx
list. [CNC Kits NZ](https://cnc-kits-nz.myshopify.com/products/mini-v-wheel-precision-shim)
specifies 8 mm OD × 5 mm ID × 1 mm thick, stainless steel, with thickness tolerance
+0.076/−0 mm. Native geometry uses nominal dimensions. The 10 mm shim is not a
mini substitute: it would intersect the mini wheel's 8.64 mm central opening.

Nominal full-size bearing centres are ±3 mm, giving 11 mm overall and 0.4 mm
protrusion per face for the Solid profile. Mini centres are ±2.5 mm, giving 9 mm
overall and 0.1 mm protrusion. Grooved full-size wheels are 10.23 mm wide, giving
0.385 mm protrusion. These stack dimensions follow from the measured shoulders,
one 1 mm shim and verified bearing widths; they are not retail kit dimensions.

## Validation boundary

Rust tests independently transcribe source vertices and product dimensions. Native
host tests compare wheel volumes and bounds against the imported STEP measurements,
check closure positions and appearance, and reopen generated parts. The same test
places all five wheel types across the existing V-slot 2020 rail. Its top chamfers
run from (3.125, 8.545) to (4.58, 10) mm, as recorded in
`crates/extrusions/src/slot.rs`. Matching the wheel's 45° flanks gives axle heights
above the rail centre of 20.32 mm (Solid), 20.34 mm (grooved Delrin), and 15.925 mm
(Mini). SolidCraft reports contact without overlap for each. Moving a wheel away
by 0.05 mm must create a gap of 0.05/√2 mm; moving it inward must create interference.
This proves nominal contact with this rail profile, not a load rating or a rolling
simulation. Assembly and reuse behavior is covered separately by `tests/parts_library_test.py`.
