# V-wheels reference

`make-v-wheel` inserts one of five fixed OpenBuilds wheel products, bare or
assembled. It is the package’s only entry.

## Parameters

| Field | Default | Meaning |
|---|---|---|
| `size` | `Solid V Wheel` | Product name from the catalog |
| `assembled` | `true` | Include two matching bearings and one internal shim |
| `placement` | empty | Sketch points or circles picked in the view; a circle, or a click inside one, means its centre |

The local origin is the wheel centre plane, with Z along the axle. Points or circles
already selected when the dialog opens fill Placement. With nothing picked, insert
at the origin. Up to 32 points can reuse a definition; a new
assembly may need a smaller batch. Errors leave the design unchanged.
Programs pass a list of references, as the dialog does: `{"type": "external_sketch_point", "sketch": 2, "id": "p1"}` for a point, `{"type": "external_sketch_curve", "sketch": 2, "id": "c1"}` for a circle, `{"type": "profile", "sketch": 2, "index": 0}` for the inside of one. `sketch` is the sketch's feature id or its name. A region that is not the inside of a circle is refused against Placement, and so is an edge, face or body.
No arbitrary diameter, seat, material or bearing substitutions are accepted.

## Sizes and composition

<!-- catalog-table:start -->

| Part | OpenBuilds SKU | OD × bare width (mm) | Bearings | Internal shim, OD × bore × width (mm) |
|---|---|---|---|---|
| Solid V Wheel | Unconfirmed | 23.9 × 10.2 | 2 × 625 2RS | 10 × 5 × 1 |
| Xtreme Solid V Wheel | 55 | 23.9 × 10.2 | 2 × 625 2RS | 10 × 5 × 1 |
| Delrin V Wheel | Unconfirmed | 23.9 × 10.23 | 2 × 625 2RS | 10 × 5 × 1 |
| Delrin Mini V Wheel | Unconfirmed | 15.23 × 8.8 | 2 × 105ZZ | 8 × 5 × 1 |
| Xtreme Mini V Wheel | 874 | 15.23 × 8.8 | 2 × 105ZZ | 8 × 5 × 1 |

<!-- catalog-table:end -->

“Unconfirmed” means no supplier SKU has been established; the product name remains
its label. Library IDs are not presented as supplier part numbers.

Full-size bearings are centred at Z = −3 and +3 mm, producing an 11 mm overall core
width. Mini bearings are centred at Z = −2.5 and +2.5 mm, producing a 9 mm core width.
The shim is centred at Z = 0. Its supplier SKU is 05 for the full-size 10 mm shim
and 170 for the mini 8 mm shim. Bearings seat against the 1 mm centre shoulder.
The shared internal recipe defines the centre, both bearing-centre frames and
ordered member frames for the complete core.

## Geometry and sources

These are nominal CAD profiles measured from the supplied OpenBuilds STEP files,
not manufacturing tolerances or a press-fit recommendation. Solid and Xtreme Solid
use the same section; Delrin Mini and Xtreme Mini share another. The full-size
Delrin V Wheel has a distinct central tread groove. Xtreme appearances suggest
clear polycarbonate; they are not measured optical or material properties.

The Solid reference is 10.20 mm wide, while the supplied retailer drawing says
10.23 mm. The catalog follows the corresponding STEP and records the discrepancy.
The mini bearing uses the documented 4 mm width correction. Rings are solid exterior
envelopes; seals and shields are recessed surface faces with no invented thickness.
There are no balls, cages or internal raceways, so assembled mass is not a physical
bearing mass. See the [source ledger](../../docs/sources/openbuilds.md).

| Data | Source | Validation |
|---|---|---|
| Five wheel profiles | OpenBuilds GrabCAD STEP references, hashes in the source ledger | `wheels` section tests; independent source volumes and bounds in `wheel_geometry_test.py` |
| Full-size core and 10 × 5 × 1 shim | OpenBuilds Solid V Wheel Assembly STEP and MiniMill BOM | Assembly geometry and BOM in `parts_library_test.py` |
| Mini shim and bearing | OpenBuilds SKU references, mini-shim listing and staff width correction | Mini child transforms and 8 × 5 × 1 dimensions in `parts_library_test.py` |
| V-slot contact | Existing sourced V-slot 2020 profile | Nominal contact and controlled clearance/interference in `wheel_geometry_test.py` |

Metal V Wheel and the supplied 8 × 16 × 5 and inch bearings remain source candidates.
Their exact product/material or closure evidence is incomplete, so they are not
published as interchangeable catalog products.

## Names and bill of materials

Standalone placements go in a phantom `Wheels` folder. A bare wheel uses its product
name; a core adds `assembly`. The core has four child occurrences: wheel, bearing,
bearing and precision shim. One bearing definition supplies both bearing occurrences,
and can also serve standalone insertions. The core is phantom in the BOM; its
members are purchased. There is no extra retail-kit purchase or separate seal purchase.

Definitions are native editable features. Display renaming preserves reuse;
modifying contents preserves that definition and causes a clean replacement on a
later insertion. Editing a child also invalidates reuse of its containing core.
See [verified reuse](../../docs/parts-library.md) and the [catalog contract](../../docs/catalog-contract.md).

## Older field names

`sketch` and `sketch_point` are retired and rejected as unsupported fields. Use `placement`, a list of picked sketch points or curves; typed comma-separated ids are gone. Unknown fields are rejected.
