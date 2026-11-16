# Parts catalog contract

Each package declares one `catalog` entry with `effect: read`. The current host can
inspect that entry and explicitly invoke it. It does not provide an automatic
provider registry or a unified browser. These packages do not invoke each other;
wheel assemblies reuse the same Rust bearing factory as the Bearings package.

## Requests and responses

List without a revision to discover the current catalog:

```json
{"version":1,"request":{"operation":"list","query":"625","offset":0,"limit":16}}
```

The response contains `version`, `catalog` (`id`, `name`, `revision`), and `result`.
List results contain sorted `items`, `total` and nullable `next_offset`. IDs are
stable within the catalog and are independent of display labels. Search matches
IDs, names, families and designations. Pages contain 1–64 items, queries at most
256 bytes, requests at most 16 KiB and responses at most 512 KiB.

Describe a product or configurable family:

```json
{"version":1,"request":{"operation":"describe","part":"30"}}
```

`result.part` contains its summary, dimensions with units, source references,
supplier/manufacturer/standard attribution, configuration schema, detail choices,
named attachment frames, insertion route and limitations. A missing supplier
number or manufacturer means unknown. An engineering designation is not a stock
claim. Family dimensions describe defaults where dimensions require configuration.

Resolve with the revision returned by discovery:

```json
{"version":1,"revision":"1","request":{"operation":"resolve","part":"30","configuration":{}}}
```

`result.part` contains:

- `representation`: the fully resolved product and its CAD representation.
- `description`: dimensions and attachment frames for this configuration.
- `members`: ordered child occurrences for an assembly, each with full product and
  representation identities and its local placement frame. Repetition expresses
  quantity. Leaves have no members.
- `insertion`: `plugin`, `entry` and `params` for the real insertion workflow.

Pass these parameters to that workflow; placement/mounting fields may be added.
The returned `catalog_selection` pins the protocol version, catalog revision and
full representation. The workflow independently parses and resolves its inputs
and rejects a stale or changed selection before sending model commands. It does
not trust client-supplied dimensions or command sequences. Changing the selected
product requires another resolve request. Ordinary dialog runs resolve directly
against the current catalog.

The host's current form builder cannot display nested API-only fields. Therefore
`catalog_selection` travels through allowed additional properties rather than
appearing as a dialog control. `catalog` is a visible read entry; it is not a
hidden host capability. No output-schema field is invented in the manifest.

## Identity and composition

Product identity is catalog authority, stable part/family ID, classification,
typed engineering configuration and any purchasing members. Classifications are
`supplier_product`, `standard_configuration`, `custom`, and `assembly`. A screw
with nominal standard dimensions and arbitrary length is a dimensional
configuration; no grade, finish, supplier availability or certification is implied.
Explicit dimension overrides are custom even if they happen to equal table values.

Representation identity adds the geometry/default-appearance revision, detail
mode and resolved child representation identities. Cosmetic and modeled threads
can share purchasing identity while retaining different representations. A child
geometry revision changes its assembly's representation, not its purchasing
composition. Recipe revisions cover changes to member placement frames.

Identities are canonical JSON strings, bounded to 16 KiB, with sorted object keys
and ordered arrays. Full canonical equality is authoritative. A digest is only a
lookup key. Placement, selection, display names and creator plugin are excluded.
The creator remains provenance in a separate saved record.

All advertised frames are right-handed local frames in millimetres: `origin`,
`x_axis`, `z_axis`; Y follows Z cross X. They describe part geometry before a
document occurrence transform. Bearing and wheel origins are axle centres with
Z along the axle. Screws seat under the head at Z=0 and extend down −Z; nuts seat
at Z=0 and extend up +Z; inserts start at their top and extend down −Z. Rails start
at Z=0 and extend up +Z. Slot nuts seat at their shoulders with their length along X.

Wheel core assemblies are phantom BOM parents containing separate purchased
wheel, two bearing occurrences and one internal shim. They are not retail kits.
The bearing is one purchase containing its rings and closure faces; closure
bodies do not become BOM purchases. Mounting axle, external spacer and nut are
outside the core recipe.

## Ownership and evolution

`parts` owns the pure protocol and identity types. `plugin-kit` owns native
inspection, definition reuse, composition and placement. Domain crates own
measurements, standards and geometry. Thin provider modules adapt the same
validated parameters used by insertion; there is no second dimension table.

`tools/catalog-tool` generates catalog-owned manifest choices and the shared
query declaration. `build.sh` refuses drift. Workflow-specific placement and
machining controls remain hand-authored. Bump a catalog revision when its choices
or resolution rules change; bump a geometry revision when dimensions, geometry,
default appearances or recipe placements change. Do not silently adopt obsolete
identities or restore removed field names through compatibility aliases.

Native acceptance tests exercise every packaged item and insertion route, verify
that queries leave the design context untouched, reject stale selections, and
verify shared wheel/bearing reuse, edited-part preservation and transaction rollback.
