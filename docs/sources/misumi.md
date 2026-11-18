# MISUMI bearing geometry evidence

Measured on 2026-11-15 from the original four user-supplied MISUMI STEP archives,
and on 2026-11-17 from four further coverage samples.
[misumi-files.json](misumi-files.json) records exact filenames, SHA-256 hashes, byte
counts and product links. [misumi-measurements.json](misumi-measurements.json)
records imported native bounds/volumes, section vertices and analytic surface radii.
Source STEP bytes are neither committed nor packaged.

The supplier product pages are dynamic and did not expose their CAD detail to the
text browser. The user supplied the 626 selector identifying open, double-shielded
and rubber-sealed products, followed by these exact named downloads. The files
establish nominal geometry; they do not establish current stock, load ratings,
precision class, steel grade or seal compound. MISUMI is recorded as supplier,
without inferring an underlying bearing manufacturer.

Reproduce the measurements with an unsigned SolidCraft CLI:

```sh
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tools/measure_parts.py docs/sources/misumi-files.json /path/to/extracted/files
```

Files use mm, with their axle along X and one end at X=0. Native parts centre the
width on Z=0 and use +Z for the axle. Section vertices in the ledger are
`[axial distance from centre, radius]`; cylindrical/toroidal surface radii retain
the source's analytic values rather than fitting polygonal approximations.

## Measured visible profiles

| Supplier part | Bore × OD × width, mm | Inner ring outer radius | Outer ring inner radius | Closure recess | Edge radius |
|---|---|---:|---:|---:|---:|
| [B626](https://us.misumi-ec.com/vona2/detail/110302273540/?HissuCode=B626) | 6 × 19 × 6 | 5.06 | 7.55 | Open | 0.3 |
| [B626ZZ](https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B626ZZ) | 6 × 19 × 6 | 4.75 | 8.15 | 0.5 | 0.3 |
| [B626DDU](https://us.misumi-ec.com/vona2/detail/110302273630/?HissuCode=B626DDU) | 6 × 19 × 6 | 4.215 | 8.1775 | 0.11 | 0.3 |
| [B608ZZ](https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B608ZZ) | 8 × 22 × 7 | 6.1 | 9.65 | 0.6 | 0.3 |
| [B6000ZZ](https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B6000ZZ) | 10 × 26 × 8 | 6.95 | 11.1 | 0.33 | 0.3 |
| [B6003ZZ](https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B6003ZZ) | 17 × 35 × 10 | 10.7 | 15.4 | 0.435 | 0.3 |
| [B6004ZZ](https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B6004ZZ) | 20 × 42 × 12 | 13.2 | 18.4 | 0.435 | 0.6 |
| [B6204ZZ](https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B6204ZZ) | 20 × 47 × 14 | 14 | 20.8 | 0.53 | 1 |

Ring boundaries are shoulder radii, not ball pitch diameters. Each source has four
rounded edges: two at the bore and two at the outside diameter, with the radius
listed above. B626's
exposed raceways are toroidal surfaces with a 6.25 mm pitch radius and 1.95 mm
section radius. Native construction retains these grooves and the eight visible balls, each radius 1.975 mm,
on the 6.25 mm pitch circle at 45° intervals in the centre plane. The source sphere
centres and radii are recorded in the measurement ledger. No cage or closure is
modeled for the open product. The sourced ball radius exceeds the race section
radius by 0.025 mm; the model preserves that nominal CAD discrepancy without
claiming interference-free rolling contact.

The other files represent a filled solid with recessed closure faces. Native
construction separates two ring envelopes and two zero-thickness annular faces.
Rubber seals are dark; shields and rings are gray. Colours are illustrative.
The STEP files do not establish hidden seal thickness or internal raceways behind
the closures, so those details are not inferred.

The 626 variants share their standard envelope but **do not share their complete
ring sections**. Their measurements remain separate supplier records. All use the
same native section/arc factory; no duplicated plugin geometry is needed.

## Validation boundary

`bearing_geometry_test.py` builds each product through the packaged plugin and
checks closure area, axial recess, appearance, ring bounds, bore/shoulder radii,
curved faces, ball count/radii/centres, native
reuse and purchasing BOM rows. For the three closed variants, subtracting the
known annular fill between the source closure faces yields the source ring volume;
the native ring volumes agree within numerical tolerance. Open B626's native ring
edge vertices are compared with the measured source section; race seam midpoints
also verify the groove depths at the centre plane. Open B626
uses separate sphere bodies rather than the source’s joined rolling-element surface
topology, so summed body volumes are not compared with its single imported solid.
The nominal overlap and absent cage make these models unsuitable for mass prediction.

Rust source-table tests independently transcribe the original four MISUMI profiles.
The saved Bearings example displays those products and the two OpenBuilds bearings.
The four new samples have been measured but are not yet implemented or covered by
native plugin geometry tests; their status is in the completion tracker.

## Shared section and inference limits

The 2026-11-17 samples B6000ZZ, B6003ZZ, B6004ZZ and B6204ZZ have the same
section structure as the existing closed MISUMI bearings: rounded bore and OD
edges, straight ring shoulders, and a recessed annulus on each side. All four
imports have six cylindrical faces, four toroidal faces and six planar faces.
Their sections are symmetric about the width centre. One shared generator can
construct this geometry from the envelope and four section parameters in the table;
no product-specific geometry algorithm is needed for these samples.

The measured vertices all match that common section. Its analytic volume also
matches each imported solid to within 0.000001 mm³. For bore radius `r`, outside
radius `R`, width `B`, shoulder radii `ri` and `ro`, shield recess `s` and edge
radius `f`, the supplier's filled reference volume is:

`V = pi * (R² - r²) * B - 2 * pi * (ro² - ri²) * s - pi * (R + r) * f² * (4 - pi)`.

The second term removes the two annular recesses; the third removes the four
rounded corners. This checks the simplified source CAD, not bearing mass or a
completed plugin implementation.

The samples do not establish a reliable rule for predicting all section parameters
from bore, OD and width. B6003ZZ and B6004ZZ share a 0.435 mm shield recess despite
their different widths. Their inner-ring radial thicknesses are 2.2 and 3.2 mm,
respectively: about 24.4% and 29.1% of the available radial span `(OD - bore) / 2`.
A constant proportional section therefore does not reproduce both references.
The edge radii also change discretely. A fitted formula could match these four
samples without predicting the remaining sizes correctly.

The remaining six products need evidence for their shoulder radii, shield recesses
and actual edge profiles. An exact-product drawing, dimension table or documented
supplier family formula is sufficient; a separate STEP is only the most convenient
way to obtain and independently check those values. Standard envelopes remain
usable for accurate placement/clearance, while inferred ring and shield details
would need to be identified as estimates.

## Catalog evidence for the coverage expansion

Reviewed on 2026-11-17: MISUMI's 2010 US catalog, printed page 893,
"Deep Groove Ball Bearings - Double Shielded", in
[`p0893.pdf`](https://us.misumi-ec.com/pdf/fa/2010/p0893.pdf). The single PDF page is
a spread containing printed pages 893 and 894; use the left-hand B-series table,
not the right-hand stainless SB-series table.

- Downloaded file size: 358168 bytes.
- SHA-256: `a0cf0e4133611f2c569f8ffb9be8523c79d9cd3a413b6ecb35adcdfc6c999dd1`.
- Checked by reading the table text and rendering the full page for visual review.
- The PDF bytes are not committed or packaged. This catalog record is separate
  from `misumi-files.json`, whose entries identify STEP inputs to measurement.

The B6000ZZ through B6004ZZ and B6200ZZ through B6204ZZ rows confirm the exact
catalog designations, double-shielded closure family and bore/OD/width in the
[completion tracker](../bearing-coverage.md#missing-radial-bearings). They provide
an independent envelope cross-check for incoming CAD. The historical catalog does
not establish current stock or availability. No supplier STEP files were acquired
in the initial web audit; four samples were subsequently supplied and measured as
recorded above.

The drawing labels `Ds`, `dh` and `R` as installation dimensions; these are not
the visible ring shoulder radii or shield boundaries. Its `r (min)` is a minimum
bearing chamfer allowance, not a measured nominal CAD fillet. The catalog does not
dimension the shield recesses or complete visible ring sections. The remaining
targets still need the section parameters established by STEP, a detailed drawing,
a supplier dimension table or a documented family sizing rule.

Existing B626ZZ and B608ZZ measurements illustrate the required evidence format,
but do not establish dimensions for the larger products. Reuse the measurement
and native geometry workflow, not their section values. Record each acquired
reference and its measurements before implementing the corresponding tracker row.
