# CSK geometry and references

[Open the saved design](csk-clutches.json) in SolidCraft, or inspect its
[captioned build steps](csk-clutches.steps.json). Opening the design does not
require the plugin. Rebuilding it requires the built Bearings package and the
[documented host API](../docs/parts-library.md#host-requirements).

![Five CSK mounting envelopes, ordered as described below](../plugins/techsquid-bearings/csk-clutches.png)

From left to right:

| Position | Designation | Envelope, bore × OD × width (mm) | Keyways, width × depth (mm) |
|---|---|---|---|
| 1 | CSK12P | 12 × 32 × 10 | Inner 4 × 1.8, reference values |
| 2 | CSK12P custom | 12 × 32 × 10 | Inner 4 × 1.2 |
| 3 | CSK12P custom | 12 × 32 × 10 | Inner 4 × 1.2, same definition as position 2 |
| 4 | CSK15PP | 15 × 35 × 11 | Inner 5 × 1.2; outer 2 × 0.6 |
| 5 | CSK15P-2RS | 15 × 35 × 16 | Inner 5 × 1.2 |

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

## Expected BOM

The saved design has five occurrences and four definitions. Its flat BOM has:

| Definition | Quantity | BOM part number | Classification |
|---|---:|---|---|
| CSK12P reference | 1 | Unassigned | Normal |
| CSK12P custom | 2 | Unassigned | Normal |
| CSK15PP | 1 | Unassigned | Normal |
| CSK15P-2RS | 1 | Unassigned | Normal |

The two custom occurrences share a definition and count together. Reference
drawings do not assign manufacturers or suppliers. The specification metadata
retains actual dimensions, a stable design identity, the reference baseline and
customization status after save/reopen. Purchasing assignment is
[future work for a shared component workflow](../docs/parts-library.md#future-component-purchasing-workflow)
after creation; `make-csk` has no purchasing or part-number inputs.

## What the example verifies

The reproducible recipe checks five closed mounting envelopes, their placed
bounding boxes and volumes, and four definitions. Expected volumes subtract the
rectangular slot intersections from an annulus using the dimensions above.
Saved-example checks compare geometry, catalog identities and reference metadata
against a fresh build. The separate CSK test suite covers all 42 presets, keyway
overrides, BOM quantities and rejected inputs.

These envelopes represent mounting geometry. Internal sprags, races and seals
are not modeled. Torque ratings do not change the CAD shape and are not assigned
to these generic specifications. Equal designation or envelope dimensions alone
do not establish supplier interchangeability. See the [CSK source ledger](../docs/sources/csk.md)
and [workflow reference](../plugins/techsquid-bearings/REFERENCE.md) for source
differences and supported combinations.
