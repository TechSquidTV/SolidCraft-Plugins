# MISUMI bearing geometry evidence

Measured on 2026-11-15 from the four user-supplied MISUMI STEP archives.
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

Ring boundaries are shoulder radii, not ball pitch diameters. Every model has four
0.3 mm rounded edges: two at the bore and two at the outside diameter. B626's
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
checks closure area, axial recess, appearance, ring bounds, curved faces, ball
count/radii/centres, native
reuse and purchasing BOM rows. For the three closed variants, subtracting the
known annular fill between the source closure faces yields the source ring volume;
the native ring volumes agree within numerical tolerance. Open B626
uses separate sphere bodies rather than the source’s joined rolling-element surface
topology, so summed body volumes are not compared with its single imported solid.
The nominal overlap and absent cage make these models unsuitable for mass prediction.

Rust source-table tests independently transcribe the measured profiles. The saved
Bearings example displays all six products, including the two OpenBuilds bearings.
