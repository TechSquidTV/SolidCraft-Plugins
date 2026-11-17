# Formsprag CSK one-way clutch sources

The nine unkeyed CSK products in the bearing catalog use the metric mounting
dimensions on page 3 of [Formsprag P-1352-FC (March 2018)](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf).
The user supplied a four-page PDF copy with SHA-256
`a125f118c7aafabb5b76097284f9b3b49b3f8dee9566a265037cc52a2be712a1`.
The [Formsprag CSK product page](https://www.formsprag.com/products/General-Purpose-Clutches/Bearing-Envelope-Design/CSK-Series)
identifies the sprag clutch family and its distinct unkeyed, P, PP and 2RS
mounting variants. Standard CSK sizes are 8, 12, 15, 17, 20, 25, 30, 35 and 40.

The supplied two-page Stieber P-7426-SC-A4 9/17 extract, SHA-256
`55807df529d39d6d7ee245dadf8fb5a8da0f9ea6aabd82a838b7c2ab69198cb9`,
documents keyed P and PP variants. These are distinct CSK products: their keyways, mounting requirements and some
2RS widths differ. The implemented keyway dimensions come from Formsprag.

The supplied `CSK8.zip`, SHA-256
`9a48350e6fe7f65b71927b25b03df56186b0f092264aaa82da44baffb24be963`,
contains `CSK8.stp` (STEP AP214, `FREILAUF_CSK8_CDS_ASM`). Its cylindrical faces
confirm an 8 mm bore and 22 mm OD, and its axial extent is 9 mm. It is a size-8
CAD reference only. It does not establish the ring or clutch sections of the other
eight sizes, so those details are omitted from all CSK representations. The
generated annulus is a placement and clearance envelope, not a solid mass model.

| Product | Bore × OD × width (mm) |
|---|---|
| CSK8 | 8 × 22 × 9 |
| CSK12 | 12 × 32 × 10 |
| CSK15 | 15 × 35 × 11 |
| CSK17 | 17 × 40 × 12 |
| CSK20 | 20 × 47 × 14 |
| CSK25 | 25 × 52 × 15 |
| CSK30 | 30 × 62 × 16 |
| CSK35 | 35 × 72 × 17 |
| CSK40 | 40 × 80 × 22 |

The source's torque and load ratings describe the physical product. The plugin
does not use them for mechanical analysis. Current stock was not checked.

## Naming and interchangeability

CSK is a family designation used by multiple suppliers. No universal CSK product
standard was established by this research. The referenced DIN 6885 documents define
keyways, not the complete clutch, its performance or every available variant.
The chooser uses `CSK…`; Formsprag supplies the default reference dimensions, independently of purchasing
identity. Keyway overrides save the actual geometry alongside the reference baseline.
The generator does not assign a manufacturer or seller until the user supplies one.

The seller table supplied for comparison matches [PEAK's table](https://www.peakbearings.com/WCB6205-2RS.htm)
and [Konlon's table](https://www.konlonbearing.com/html/en/products/onewaybearing/415.html),
including WCB6205-2RS. It therefore cannot establish the seller's manufacturer.
Its CSK2RS mounting dimensions agree with the Formsprag dimensions, but torque
ratings differ (for example CSK25-2RS: 68 versus 105 N·m in P-1352-FC).
WCB6205-2RS is 25 × 52 × 15 mm, not the 20 mm-wide CSK25-2RS; it is not an alias
and is outside this CSK catalog.

## Keyed and sealed coverage

The generator offers 42 reference presets: nine plain CSK, nine CSK-2RS,
eight CSKP (12–40), seven CSKPP (15–40), eight CSKP-2RS (12–40), and CSK30PP-2RS.
The final combination is explicitly listed on page 28 of the newer
[Formsprag MCC-P-956-FC-EN-US 11/25 extract](https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf).
Pages 28–29 establish the keyed size restrictions and keyway dimensions. Unsupported
combinations are rejected rather than inferred from another vendor's offerings.

| Bore (mm) | Inner keyway width × depth (mm) | Outer keyway width × depth (mm) |
|---|---|---|
| 12 | 4 × 1.8 | Not listed |
| 15 | 5 × 1.2 | 2 × 0.6 |
| 17 | 5 × 1.2 | 2 × 1.0 |
| 20 | 6 × 1.6 | 3 × 1.5 |
| 25 | 8 × 2.0 | 6 × 2.0 |
| 30 | 8 × 2.0 | 6 × 2.0 |
| 35 | 10 × 2.4 | 8 × 2.5 |
| 40 | 12 × 3.3 | 10 × 3.0 |

P cuts the inner slot; PP cuts both slots. Nominal rectangular slots extend through
the full bearing width and face local +Y as shown in the reference drawing. Depth
is measured radially from the bore/OD apex. Slot widths are along local X. Rotate
the component to align the keyways; placement follows the selected sketch frame.
Sealed widths are explicitly tabulated: 9, 14, 16, 17, 19, 20, 21, 22, 27 mm for
sizes 8–40. CSK8-2RS has one lip seal. Seal internals and tolerances are not modeled.

PEAK's [keyway table](https://www.peakbearings.com/keywaysize.htm) gives CSK12
an inner keyway of 4 × 1.2 mm and an outer keyway of 2 × 0.6 mm. Formsprag gives
CSK12P a 4 × 1.8 mm inner keyway and does not list CSK12PP. PEAK also lists keyed
CSK8. These are concrete reasons to preserve the reference profile rather than
claim universal geometric interchangeability from the CSK designation alone.
