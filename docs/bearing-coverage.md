# Bearing coverage and source checklist

Parity means covering the main app's nominal bearing sizes with identified,
purchasable products. It does not mean copying a generic envelope and inventing its
seal details. A base size, a supplier part number and a CAD representation are
different things in the shared catalog contract.

## App baseline

The [SolidCraft bearing table at commit 0930970](https://github.com/tangency-labs/solidcraft/blob/0930970ce03b43f8d9e6477cb0903d4a713f17f4/crates/engine/src/cmd/parts.rs#L235)
lists thirteen envelopes. This is a dated comparison baseline, not a runtime table
or evidence of a particular manufacturer's ring or seal geometry.

| Base size | Bore × outside diameter × width, mm | Plugin coverage |
|---|---|---|
| 625 | 5 × 16 × 5 | OpenBuilds 30, 625 2RS |
| 626 | 6 × 19 × 6 | MISUMI B626, B626ZZ, B626DDU |
| 608 | 8 × 22 × 7 | MISUMI B608ZZ |
| 6000 | 10 × 26 × 8 | Awaiting product reference |
| 6001 | 12 × 28 × 8 | Awaiting product reference |
| 6002 | 15 × 32 × 9 | Awaiting product reference |
| 6003 | 17 × 35 × 10 | Awaiting product reference |
| 6004 | 20 × 42 × 12 | Awaiting product reference |
| 6200 | 10 × 30 × 9 | Awaiting product reference |
| 6201 | 12 × 32 × 10 | Awaiting product reference |
| 6202 | 15 × 35 × 11 | Awaiting product reference |
| 6203 | 17 × 40 × 12 | Awaiting product reference |
| 6204 | 20 × 47 × 14 | Awaiting product reference |

OpenBuilds 800, 105ZZ (5 × 10 × 4 mm), is additional coverage used by mini wheels.
The generated table in each Bearings reference guide is authoritative for products
currently included. The three covered base sizes are 625, 626 and 608. Next prioritize
the 6000 and 6200 families, and sealed 608 if wanted.

## References to acquire

For each variant, retain its exact manufacturer/supplier part number, product URL,
and a STEP model or drawing showing the visible rings and closures. Cross-check the
bore, outside diameter and width. Record the source revision and hash as with the
OpenBuilds references. Boundary dimensions alone do not establish closure recesses,
ring shoulders or edge profiles.

The supplied MISUMI selector and subsequent STEP archives establish this status:

| Product | Purpose | Status |
|---|---|---|
| [MISUMI B626ZZ](https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B626ZZ) | Double metal shields | Supplied STEP measured; included |
| [MISUMI B626DDU](https://us.misumi-ec.com/vona2/detail/110302273630/?HissuCode=B626DDU) | Rubber seals | Supplied STEP measured; included |
| [NTN 626Z](https://us.misumi-ec.com/vona2/detail/221000058378/?HissuCode=626Z) | Optional single shield | Awaiting exact CAD/drawing |
| [MISUMI B626](https://us.misumi-ec.com/vona2/detail/110302273540/?HissuCode=B626) | Open, visible balls | Supplied STEP measured; included |

B608ZZ is also included from its supplied STEP. The [MISUMI evidence ledger](sources/misumi.md)
records exact revisions and representation limits. Current stock is not verified.
The supplier pages' dynamically loaded CAD details were not independently accessible;
geometry comes from the supplied files. Keep suffixes attached to the source's
brand; do not infer that different brands' seals or rings share geometry because
their envelope sizes match.

## Adding products without duplication

`crates/bearings` remains the owner of product identity, closure type, measurements
and native geometry. Both the Bearings workflow and wheel recipes use its factory.
Reuse a section across variants only when their references establish the match;
retain separate purchasing identities for different sourced products. Extend the
catalog-generated choices/reference table and independent geometry tests together.
Visible balls are included for sourced open bearings; sealed and shielded variants
omit hidden balls. Cages remain outside this representation. Never fill an open
bearing with an invented closure.
