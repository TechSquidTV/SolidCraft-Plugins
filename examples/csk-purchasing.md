# CSK purchasing example

[Open the saved design](csk-purchasing.json) or inspect the
[reproducible recipe](csk-purchasing.steps.json). All seller names, SKUs and URLs
in this example are fictional. This companion uses the geometry generator from
the [CSK example](csk-clutches.md) with explicit purchasing assignments.

| Position | CSK12P inner keyway | Assignment | Internal BOM number |
|---|---|---|---|
| Left | 4 × 1.2 mm | Example seller A, SKU 12P | DEMO-A |
| Centre | 4 × 1.2 mm | Same seller and SKU, different listing URL | DEMO-A |
| Right | 4 × 1.8 mm | Example seller B, SKU 12P | DEMO-B |

Every envelope is 12 × 32 × 10 mm. The left and centre parts share one definition.
The default flat BOM contains **DEMO-A: quantity 2** and **DEMO-B: quantity 1**.

The first insertion uses:

```json
{
  "size": "CSK12P",
  "inner_keyway_depth": 1.2,
  "part_number": "DEMO-A",
  "purchase_supplier": "Example seller A",
  "purchase_sku": "12P",
  "purchase_url": "https://example.com/clutch"
}
```

The second changes only the URL to `https://example.com/clutch?ref=other` and its
placement. The latest nonblank URL is recorded on the shared definition; it does
not create another purchasing identity or split the quantity. Omitting the URL
on a later insertion preserves the existing link.

The third selects the reference keyway and seller B with internal number
`DEMO-B`. Attempting to use `DEMO-A` for this different assignment or geometry
fails before making model changes. Supplier SKUs can overlap across sellers;
your internal BOM numbers must distinguish incompatible assignments. Leaving an
internal number blank groups equivalent placements by their definition instead.

Manufacturer identity is unassigned throughout. Formsprag remains a source for
the reference dimensions, and its source record is unchanged by purchases. See
[assignment rules](../docs/purchasing.md) for manufacturer part numbers, collision
handling, metadata, limitations and save/reopen behavior.
