# CSK geometry, references and purchasing

[Open the saved design](csk-clutches.json) in SolidCraft, or inspect its
[captioned build steps](csk-clutches.steps.json). Opening the design does not
require the plugin. Rebuilding it requires the built Bearings package and the
[documented host API](../docs/parts-library.md#host-requirements).

![Six CSK mounting envelopes, ordered as described below](../plugins/techsquid-bearings/csk-clutches.png)

From left to right:

| Position | Designation | Envelope, bore × OD × width (mm) | Keyways, width × depth (mm) | Purchasing |
|---|---|---|---|---|
| 1 | CSK12P | 12 × 32 × 10 | Inner 4 × 1.8, reference values | Unassigned |
| 2 | CSK12P custom | 12 × 32 × 10 | Inner 4 × 1.2 | Unassigned |
| 3 | CSK12P custom | 12 × 32 × 10 | Inner 4 × 1.2, same definition as position 2 | Unassigned |
| 4 | CSK12P custom | 12 × 32 × 10 | Inner 4 × 1.2, same geometry as positions 2–3 | Fictional example assignment |
| 5 | CSK15PP | 15 × 35 × 11 | Inner 5 × 1.2; outer 2 × 0.6 | Unassigned |
| 6 | CSK15P-2RS | 15 × 35 × 16 | Inner 5 × 1.2 | Unassigned |

Use **Bearings > make-csk**. Blank keyway fields retain the selected preset's
reference values. For positions 2–3, select `CSK12P` and enter `1.2` for inner
keyway depth. The corresponding workflow parameters are:

```json
{
  "size": "CSK12P",
  "inner_keyway_depth": 1.2
}
```

The bore, OD and width stay fixed by the designation. Only existing keyways can
be overridden: P has an inner slot, PP has both slots, and plain CSK has neither.
Keyways face local +Y and pass through the full width. Depth is measured from
the bore or OD apex, not from a mating shaft's keyway. These dimensions do not
specify a complete fit or tolerance system.

Position 4 uses the same parameters plus:

```json
{
  "part_number": "EXAMPLE-CSK12P",
  "purchase_supplier": "Example supplier",
  "purchase_sku": "DEMO-12",
  "purchase_url": "https://example.com/clutch"
}
```

These purchasing details are fictional, included only to demonstrate assignment.
They do not identify a real seller or validate a purchasable part. Manufacturer
is deliberately empty. For your own part, enter the identity and dimensions from
the listing or drawing you intend to use.

## Expected BOM

The saved design has six occurrences and five definitions. Its flat BOM has:

| Definition | Quantity | BOM part number | Classification |
|---|---:|---|---|
| CSK12P reference | 1 | Unassigned | Normal |
| CSK12P custom, unassigned | 2 | Unassigned | Normal |
| CSK12P custom, example assignment | 1 | EXAMPLE-CSK12P | Purchased |
| CSK15PP | 1 | Unassigned | Normal |
| CSK15P-2RS | 1 | Unassigned | Normal |

The two unsourced custom occurrences share a definition and count together.
The assigned custom part stays separate, even though its geometry matches.
Reference drawings remain recorded independently and do not fill purchasing
fields. The specification metadata retains actual dimensions, the reference
baseline, customization status and the optional purchasing assignment after
save/reopen. The current BOM counts parts and displays part numbers; the separate
metadata supports future richer BOM selection and export.

## What the example verifies

The reproducible recipe checks six closed mounting envelopes, their placed
bounding boxes and volumes, and five definitions. Expected volumes subtract the
rectangular slot intersections from an annulus using the dimensions above.
Saved-example checks compare geometry, catalog identities and specification
metadata against a fresh build. The separate CSK test suite covers all 42 presets,
keyway overrides, purchasing assignments, BOM quantities and rejected inputs.

These envelopes represent mounting geometry. Internal sprags, races and seals
are not modeled. Torque ratings do not change the CAD shape and are not assigned
to these generic specifications. Equal designation or envelope dimensions alone
do not establish supplier interchangeability. See the [CSK source ledger](../docs/sources/csk.md)
and [workflow reference](../plugins/techsquid-bearings/REFERENCE.md) for source
differences and supported combinations.
