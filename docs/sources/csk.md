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
documents keyed P and PP variants. These are not silently treated as standard CSK
products: their keyways, mounting requirements and some 2RS widths differ.

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
