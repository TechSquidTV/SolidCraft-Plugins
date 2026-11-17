# Purchasing assignments

A drawing reference describes evidence for a shape. A purchasing assignment says
which supplier product the user intends to buy. They are independent records.
CSK is the first workflow using this shared assignment API; other generators keep
their existing supplier/standard identities and input forms.

## Inputs

`make-csk` accepts the following optional fields in addition to geometry:

| Field | Meaning |
|---|---|
| `part_number` | Your internal BOM number; may be provided without a purchase |
| `purchase_supplier` + `purchase_sku` | Supplier name and its product identifier; supply both |
| `purchase_manufacturer` + `purchase_mpn` | Optional manufacturer name and manufacturer part number; supply both, along with supplier/SKU |
| `purchase_url` | Optional HTTP(S) listing URL for the assignment |

Blank fields are unassigned. Names and identifiers are trimmed, case-sensitive
text of at most 128 bytes without control characters. URLs allow up to 2048 bytes.
A supplier or manufacturer name alone is insufficient to identify a product.
References never populate these fields. Product identity and dimensional
compatibility are supplied by the user, not verified against supplier stock.

## Identity and BOM grouping

Resolved design geometry remains independent of assignments. The shared
`parts::purchasing::Assignment` combines an optional BOM number and an optional
supplier product identity. `Details` holds the separate listing URL. An assignment
participates in definition reuse; the URL does not. The original unassigned design
identity is unchanged when all assignment fields are blank.

The host's default BOM groups by `part_number`; components without a number group
by definition. Before inserting an assigned part, `Library::ensure_assigned`
checks existing definitions, including unused/excluded definitions. An occupied
BOM number is accepted only when canonical product identity matches and the
existing native definition still verifies. A different shape, supplier, SKU or
unverified/edited definition causes an error before construction or placement.
Choose a unique internal number or leave it blank. The supplier SKU remains a
separate field and is not automatically used as the internal BOM number.

This check applies to insertion through the assignment API. It does not override
the host's rules for direct metadata edits or other tools; manually assigning the
same number to different components can still group them in the host BOM.

Supplier assignments produce `purchased` components. A user number without a
supplier assignment remains a normal design. Custom and unassigned parts remain
countable. Changing a URL alone, even its query parameters, reuses the same
verified definition and preserves one BOM quantity.

## Listing metadata and save/reopen

`org.techsquid.purchasing` schema version 1 stores the assignment, its optional
purchasing identity and the listing URL. CSK's separate
`org.techsquid.specification` version 2 record remains unchanged: actual geometry,
geometry identity, reference baseline and source evidence.

A nonblank URL on insertion updates the shared definition's listing annotation,
including all occurrences of that definition. Blank preserves the existing URL.
This does not change the assignment or reference evidence. The library verifies
the definition first and refreshes its content snapshot after the annotation
update, so later reuse and save/reopen remain valid. Modified unnumbered parts
are preserved and a clean replacement is built; a modified numbered part causes
a collision error instead of a misleading shared BOM row.

There is no general reassignment editor in this change. A different assignment
on insertion creates a separate definition; it does not reassign existing parts.

## Validation and example

See the [purchasing example](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/examples/csk-purchasing.md) for two occurrences
sharing one assignment across a URL change, plus a different supplier/keyway.
After `./build.sh`, run:

```bash
python3 tests/purchasing_test.py
python3 tests/examples_test.py examples/csk-purchasing.steps.json examples/csk-clutches.steps.json
```

The native tests cover URL-independent reuse and quantities, number conflicts
across suppliers/SKUs/keyways and existing catalog products, incomplete identifier
rejection, atomic failure, annotation updates, save/reopen and modified parts.

## Breaking changes from the earlier combined PR

Catalog revision 7 defines this assignment contract. The earlier CSK purchasing
prototype embedded a URL and loose vendor fields in the design identity. Those
records are not adopted. Manufacturer information now requires a manufacturer
part number, and supplier information requires a supplier SKU. No legacy aliases
or fallback identity paths are added. Existing saved geometry is preserved.
