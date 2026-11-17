---
name: techsquid-parts
description: Use when adding or changing a part or component in a TechSquid SolidCraft plugin - its geometry, standards table, body name, reusable definition, build site, cuts or catalog identity.
---

# Build a part that fits the library

Read [docs/parts-library.md](../../docs/parts-library.md) for reuse and BOM rules and [docs/catalog-contract.md](../../docs/catalog-contract.md) for identity. Copy the closest existing part rather than starting blank: a revolved part like `wheels::Wheel` or a bearing, a hex part like `fasteners::nut`, a swept section like `extrusions::profile`.

## Data and standards

- Dimensions live in the domain crate as tables. Every table names its source in a comment (standard and table, or supplier file), and a unit test repeats the source's values so the code and the source can't drift.
- Measured supplier geometry goes in `docs/sources/<supplier>.md` and its `-measurements.json`; the crate cites it. Never read dimensions from a local Downloads path at runtime.
- When a table changes, update the Standards table in the plugin's REFERENCE.md. If it feeds catalog choices or generated tables, run `cargo run -p catalog-tool -- --write`.

## The definition

A fixed part is a reusable component. Give it an `ensure` method in the domain crate that calls `library.ensure` (or `ensure_built` for a part that is only its own geometry) with:

- `component::Definition { plugin, base, description, bom_type, site }`. `base` is the body name from `plugin_kit::name`: kind, then size, with a plain `x` (`Screw M3 x 12 socket`, `Nut M3`). `bom_type` is `purchased`, or `phantom` for an assembly of separate purchases.
- A part number and a `parts` representation from the plugin's `provider.rs`. Bump that provider's `REVISION` when a change makes old definitions differ.
- A build closure that draws the part in the frame it is given, never on fixed world coordinates.

The plugin's `make` resolves placements with `place::resolve_each` and calls `component::insert(r, creation_calls, &places, folder, ensure, attach)`. `creation_calls` is the host-call budget for building one new definition; measure it through the host rather than guessing. `attach` adds seat joints, or does nothing for a loose part.

An assembly places other definitions inside its build with `library.place`, as `wheels::Wheel::ensure` does with its bearings and shim.

## Rules that bite

- **A cut in one placement is a cut in all of them.** Placements share their body. Anything cut into the part belongs in its build and its name, so the cut part is a definition of its own: a rail's taps and holes are `make-extrusion` fields.
- **Cuts into the receiving part stay in the receiving part's component**, not in the hardware definition.
- **A part with a cut thread is built on a site.** SolidCraft finds the face for a modeled thread by a point among every body, so a bore of the same size on the same axis at the origin steals the thread. Use `Site::for_thread(modeled)` for small parts (`Site::Compact`) and `Site::Apart` for a rail with a tapped end; `Site::Origin` otherwise. Some kernel cuts fail away from the origin: build those on the origin, `solid.move` to the site, then cut the thread. A test host overrides `Runner::site` to stay on the origin.
- **Names are reserved, not assumed.** Use `name::reserve` or `name::reserve_part` so a second run gets `Nut M3 (2)`; the library reserves component, body and feature prefixes together. A body the host names `BodyN` doesn't survive a rebuild.
- After changing a threaded part's build, run two parts of one thread through the host: each must weigh what it weighs alone (`parts_keep_their_own_threads` in `tests/geometry_test.py`).

## Verify

Unit-test the commands `make` sends against a recording `Runner`. Then build it through the host and compare volume and bounding box with values worked out by hand: see `skills/techsquid-tests/SKILL.md`.
