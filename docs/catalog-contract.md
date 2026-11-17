# Internal parts catalog contract

This documents the internal Rust descriptors and query types, not a public plugin
API. Packages export insertion workflows only. The former `catalog` entry and its
runtime handler have been removed: scripts calling that entry now fail, with no
compatibility alias. Discovery and a graphical catalog browser are deferred.

Shared descriptors still own product identity, configuration, sources and assembly
members. These packages do not invoke each other; wheel assemblies reuse the same
Rust bearing factory as the Bearings package.

## Internal requests and responses

The examples below describe serialized Rust queries used by provider tests. They
are not requests accepted by an installed plugin entry. List without a revision
to inspect the current internal catalog:

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

The internal resolver retains its typed `catalog_selection` validation so provider
round-trip tests exercise the real workflow parser. Normal insertion dialogs do
not need a resolved selection. No catalog query handler, hidden entry, or invented
host capability is exported by the packages.

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

All descriptor frames are right-handed local frames in millimetres: `origin`,
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

`tools/catalog-tool` generates catalog-owned insertion choices and reference
tables; it does not add runtime entries. `build.sh` refuses drift. Workflow-specific
placement and machining controls remain hand-authored. Bump a catalog revision when its choices
or resolution rules change; bump a geometry revision when dimensions, geometry,
default appearances or recipe placements change. Do not silently adopt obsolete
identities or restore removed field names through compatibility aliases.

Rust provider tests exercise product descriptions, bounded queries, resolved
configurations and stale-selection rejection. Native tests verify that removed
catalog calls leave the design unchanged, insertion routes still work, and shared
wheel/bearing reuse, edited-part preservation and transaction rollback are intact.
