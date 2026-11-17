# Shared parts libraries

All six packages use shared internal part definitions and one native component
library. The packages remain independently installable. V-wheels calls the same bearing factory
as Bearings through shared Rust code; it never invokes another plugin at runtime.

## Host requirements

Version 0.3 requires a SolidCraft build exposing `component.list.active_path` and
`sketch.inspect.world_plane`. Earlier 0.42 and 0.43 builds lack these fields.
Use a build containing merged [SolidCraft PR #17999](https://github.com/tangency-labs/solidcraft/pull/17999),
validated on commit `a28a0b777`; the version label alone is insufficient.
Missing fields are refused; the library never guesses a selected occurrence or
uses component-local axes as world axes. Saved native designs do not depend on
these new inspection fields.

## Responsibilities

- `parts` owns typed discovery, product and representation identities. It has no
  host dependency. Providers adapt the validated parameters used by insertion.
- Domain crates own standards, source measurements and geometry. A wheel's one
  typed recipe drives its purchasing members, representation dependencies,
  advertised child frames and actual child placements.
- `plugin-kit::component::Library` owns inspection, reuse, composition, naming,
  placement, context restoration and command accounting.
- Plugin workflows own selection, receiving-part cuts and attachment joints.
  Cut geometry remains in the receiving component. Sketch-driven rails remain
  independent native path features rather than reusable stock definitions.

See [the internal catalog contract](catalog-contract.md) for ownership and identity
rules. Packages expose only insertion workflows. The `catalog` entry and its guest
handler are removed; script calls to it are rejected. Internal descriptors remain
shared, but there is no public discovery API, provider registry or unified browser.

## Verified component reuse

A definition stores `org.techsquid.parts` metadata containing record version,
canonical product and representation, creator provenance, build frame and a snapshot
of its native contents. The document's `org.techsquid.parts.index` directory maps a
representation digest to candidate component IDs. That directory is advisory:
missing IDs are ignored; full identity and current content must still match.
Unknown or malformed directory versions are refused. Snapshot version 2 supersedes
the earlier parameter-token check; version 1 snapshots are not adopted. Legacy
`dev.threadcraft` command hashes are not adopted.

Inspection reads owned feature definitions and inputs, full sketch geometry and
constraints, body history, materials, resolved body/face appearances, purchasing
metadata, parameter dependencies, and child identities/transforms recursively.
Sketch planes and point coordinates are compared in the definition's local frame;
the host's `world_plane` and world point coordinates are placement data and excluded.
Parameter candidates are confirmed against the host’s dependency walker by feature
ID, then followed through the host’s parameter dependency list. Labels and unrelated
parameters do not qualify. Parameter comments, favourites and reverse-use lists are
not geometry. Results are cached within the same inspection budget.
An external cut can alter a body without entering its component's owned timeline;
body-history checks catch that case. Geometry with equal volume or bounds is not
assumed equal. Failed/suppressed/rolled-back features, unsupported heavy payloads,
incomplete wires, jointed children and pending child moves are not certified.

The boundary is the native host's public inspection API. It compares exposed
editable inputs and owned state, not every private kernel field or adversarially
edited document byte. Feature reference bookkeeping omitted by `timeline.inspect`
is managed by the host. Native regression tests cover ordinary edits, external
cuts, references, nested occurrences and save/reopen.

Placement, display component/feature renaming and creator plugin do not determine
purchasing identity. A modified or unverifiable candidate is preserved and reported
in `library.preserved_modified_definitions`; new placements get a clean definition.
An edited child invalidates its assembly too. Body names remain part of inspected
contents. New definitions reserve their component, body and feature names together,
including derived prefixes and case variants. Renaming both a component and its
bodies therefore leaves its original sketches protected; a replacement gets a
free numbered name and can be reused on the next insertion. Generated sketches
and subpart features use `Part: Feature`; bodies use `Part Subpart`. Unrelated
layout sketches do not claim the body-name prefix.

## Creation, placement and transactions

Each invocation reads an index and caches inspection results; mutations invalidate
that view. `ensure` resolves or creates one definition. `geometry` groups its own
features. `place` consumes the new definition's first occurrence once, then copies
it; a wheel's two bearings therefore share one definition. `place_all` places the
root part in a phantom folder with exactly one placement at the design root. If
that folder has been copied, insertion uses a separate numbered folder and reuses
it on later runs. The copied folder's children stay unchanged, so insertion adds
only the requested parts or assemblies. `finish` writes the directory and restores the
original active component and its complete root-to-leaf occurrence path. The same
context is restored after temporary switches during nested assembly creation and
placement. Consecutive insertions therefore keep using the selected copy, even
when the same leaf occurrence appears under multiple parent paths. Placement uses
the host's world sketch plane, matching its world point coordinates. Unplaced new
definitions are an error.

Pocket preparation also restores this context after editing a face-attached source
sketch, since the host switches to the sketch's original authoring copy during the
edit. Screw and nut seat inspection shares one resolver: faces are inspected in
world placement, while hole-based joints use the definition name and exact occurrence path.
It measures the cut bodies' placed bounds in one batch, then reads faces and paths
only for copies containing a requested seating point. Distant copies do not add
host calls per insertion. Overlapping candidates remain eligible so ambiguous
seats are still refused. Countersunk screws use their cone/bore junction for this
filter, including when the screw origin lies outside a thin plate.
Tapped screws keep their existing first-placement limitation because the host's
axis-based joint origins do not honor an exact nested path.
The host exposes that body-to-path mapping through selection details; the resolver
batches these reads and restores the previous selection even if inspection fails.
It never infers a path from a displayed body label.

An assembly recipe is limited to eight levels and 32 distinct definitions. A run
accepts 1–32 root placements. Before new construction, the library combines the
recipe's conservative creation estimate with inspection and placement costs.
A counted runner includes commands already spent by the workflow and caps the
operation at 248 calls, below the host's 256-call limit. Content inspection is
bounded to 128 reads; the directory is limited to 60 KiB. Complex geometry or a
large batch can require fewer points. A late failure propagates through the host's
single edit transaction and rolls back geometry, metadata, context and undo state.

Parts with modeled threads retain the isolated build-site strategy required by
the host's point-based thread targeting. The site derives from the definition ID;
occurrence placement removes the build-site offset and aligns the part with the
selected frame. Nested wheel/bearing parts use the origin. Fixed-length rails,
screws, nuts, inserts and slot nuts use the same placement service.

## Purchasing and BOM

Supplier products, standards-based configurations, custom geometry and composed
assemblies have separate classifications. A dimensional standard does not establish
supplier availability, grade, finish or certification. Explicit overrides are
custom even if they reproduce table dimensions. Cut/machined rails retain stock
profile provenance but identify a custom finished configuration.

Supplier and standard numbers are stable across cosmetic/modeled thread detail.
Custom models use `Custom <canonical part ID> [<product digest>]`. The entire number
comes from the canonical product; raw size spellings and display labels never enter
it. Equivalent spellings and different thread detail therefore share a BOM row,
while distinct custom configurations stay separate. The full product identity,
not that digest, authorizes reuse.
Unknown wheel SKUs are left unknown; their display labels are not claimed as SKUs.

A wheel core is a phantom parent of one purchased wheel, two purchased bearing
occurrences and one purchased internal shim. It is not a retail kit. A bearing's
rings and closure faces are bodies of one purchase. Folder components are phantom
as well, so neither folders nor closure faces add purchase rows.

## Validation

Run an unsigned local CLI meeting the [host requirements](#host-requirements).
A signed macOS build may be killed when
loading Wasm; skipped runs provide no runtime evidence.

| Check | Evidence |
|---|---|
| `plugin_entries_test.py` | Insertion-only packages, removed catalog route rejection without edits, no residual guest handler, invalid fields and non-XY insertion |
| `catalog_reuse_test.py` | Existing families, detail versus purchase identity, BOM grouping, component/body renaming together, preserved sketches, modified appearance, custom identity, equivalent size spellings, unrelated parameters and mounting invariance |
| `parts_library_test.py` | Cross-plugin bearing reuse, exact core composition, nested transforms, BOM, copied-folder isolation and replacement reuse, consecutive fresh/reused insertions through copied and rotated nested occurrences, exact authoring context, undo/redo, save/reopen, modified children, late rollback and batch limits |
| `parts_directory_test.py` | Missing IDs, obsolete revisions/snapshots, incorrect digest candidates, malformed/unsupported directories and unchanged failed transactions |
| `parts_content_test.py` | Same-volume edits, sketches, external cuts, materials/appearances, direct/transitive/placement parameters, isolated sketch-only dependencies (including equal-value expression edits), parameter annotations, nested structure and unsupported states |
| `parts_inspection_test.py` | Native host inspection assumptions and shared nested definitions after reopen |
| `bearing_geometry_test.py` | MISUMI sourced curved rings, bore/shoulder radii and closure measurements, open race depths and section vertices, sourced visible balls, purchase identities and native reuse |
| `wheel_geometry_test.py` | Source volumes, ring/closure bounds and areas, width correction, shims and V-slot contact |
| `examples_test.py` | Reproducible examples, all-six-package showcase, saved geometry, catalog identities and BOM numbers |
| Existing geometry and reference suites | Fastener cuts, isolated modeled threads, rail machining and seated screw/nut edits |

The native inspection fixture used by Rust geometry-command recorders is deliberately
a fixed cylinder, not a CAD emulator. Those unit tests assert builder commands;
the native suites above establish actual reuse and edited-content behavior.

## Design specifications before purchasing

The CSK generator identifies a resolved `bearings::csk::Geometry` independently
of its drawing sources. A default CSK is a `design_specification`; changed keyway
dimensions make it `custom`. Neither implies a supplier purchase or a universal
CSK standard. Optional assignments use the separate [purchasing API](purchasing.md), shared
by design factories. The CSK geometry record remains unchanged.

Equivalent resolved geometry reuses a definition and counts its occurrences in
the BOM without assigning a part number. The version 2
`org.techsquid.specification` record stores actual geometry, its identity, the
reference baseline and customization status. Existing supplier-product workflows
remain valid for explicitly chosen catalog products; using a reference drawing
alone must not create that identity.

`Library::ensure_assigned` applies assignment identity, checks occupied BOM numbers
against current native definitions and saves listing metadata separately. The
first workflow using this API is `make-csk`; existing fixed-product workflows
retain their current inputs. See [assignment rules](purchasing.md).
