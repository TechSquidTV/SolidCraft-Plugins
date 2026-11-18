---
name: techsquid-parts
description: Use this skill when adding or changing the geometry of hardware a TechSquid plugin builds - a new size, head, drive, profile, product or variant, a standards or supplier dimension table, a body or part name, a BOM part number, or how a part is reused, placed or threaded. Use it even when the request only names the part ("add DIN 125 washers", "support M2.5 set screws", "the 2080 rail is wrong"). Not for dialog field or manifest changes (techsquid-fields) or for running tests (techsquid-tests).
---

# Build a part that fits the library

Copy the closest existing part rather than starting blank: a revolved part like `wheels::Wheel` or a bearing, a hex part like `fasteners::nut`, a swept section like `extrusions::profile`. [docs/parts-library.md](../../docs/parts-library.md) has the full reuse and BOM rules; read it when you change how a definition is identified or reused.

## Workflow

- [ ] **Table.** Put the dimensions in the domain crate as a table. Name the source in a comment (standard and table, or supplier file) and add a unit test that repeats the source's values. Measured supplier geometry goes in `docs/sources/<supplier>.md` and its `-measurements.json`.
- [ ] **Definition.** Give the part an `ensure` method in its domain crate that calls `library.ensure_built` (a part that is only its own geometry) or `library.ensure` (an assembly that places members with `library.place`, as `wheels::Wheel::ensure` does). Pass a `component::Definition`, a part number, and a `parts` representation from the plugin's `provider.rs`.
- [ ] **Build.** Draw in the frame the build closure is given, never on fixed world coordinates.
- [ ] **Run.** In the plugin's `make`: `place::resolve_each`, then one `component::insert(r, creation_calls, &places, folder, ensure, attach)`. `attach` adds seat joints, or does nothing for a loose part.
- [ ] **Docs.** Update the Standards table in the plugin's REFERENCE.md. If the table feeds catalog choices, run `cargo run -p catalog-tool -- --write`.
- [ ] **Verify.** `cargo test --workspace --locked`, `./build.sh`, then the host suite for the plugin (see `skills/techsquid-tests/SKILL.md`). Compare volume and box with values worked out by hand. If they differ, fix the build, not the expected value, and run again.

## Gotchas

- **Reuse a sourced section, but verify each product's dimensions.** A shared shape can use one parametric builder. Matching bore/OD/width or a few sample models does not establish exact shoulder, recess or edge dimensions for other SKUs. An exact-product STEP is convenient, but a dimensioned drawing, table or documented supplier rule is sufficient if it determines the complete modeled section. Keep inferred details explicitly estimated. The [MISUMI comparison](../../docs/sources/misumi.md#shared-section-and-inference-limits) demonstrates this distinction.
- **The BOM groups rows by part number.** A variant the standard doesn't list (a Torx drive on an ISO 4762 socket head) must get its own generated number and `bom_type` `normal`, or it merges with the standard part. `purchased` is for a part you can buy by that number; `phantom` for an assembly of separate purchases.
- **A changed definition needs a new revision.** Bump the provider's `REVISION` (and the representation revision) when a change makes an old definition differ, so saved designs aren't silently reused with the old shape. There is no migration: say so in the release notes.
- **A run has a budget: 256 host calls and 30 seconds.** `creation_calls` is what one new definition takes. Don't guess it: `component::creation_calls(site, build)` counts the build with no host behind it and adds the sealing margin. A build whose size depends on the input, such as a rail's holes, counts itself with `component::CallCount` as `build_calls` in `plugins/techsquid-extrusions/src/lib.rs` does, to refuse an input that won't fit in `parse`, on the field that causes it (`holes: 19 holes in this rail take 215 commands …`).
- **Validation and build must use the same number.** Compute a limit (thread room, shortest length, hole spacing) from the same function the build uses. A set screw once lost 0.13 mm of thread because the check used the chamfer's radial leg and the build its axial length.
- **A cut in one placement is a cut in all of them.** Placements share their body. Anything cut into the part belongs in its build, its name and its fields: a rail's taps and holes are `make-extrusion` fields. Cuts into the receiving part stay in the receiving part's component.
- **A part with a cut thread is built on a site.** SolidCraft finds the face for a modeled thread by a point among every body, so a bore of the same size on the same axis at the origin steals the thread. Use `Site::for_thread(modeled)` for small parts and `Site::Apart` for a rail with a tapped end. Some cuts (hexagon chamfers, some heads) fail away from the origin: build on the origin, `solid.move` to the site, then cut the thread.
- **Names are reserved, not assumed.** Base names come from `plugin_kit::name`: kind, then size, with a plain `x` (`Screw M3 x 12 socket`). `name::reserve` and `name::reserve_part` give `Nut M3 (2)` on a clash. A body the host calls `BodyN` doesn't survive a rebuild.
- **A wrapper `Runner` forwards every method**, including `command_count`, `body_names` and `site`. A missing one silently falls back to the default.
- **Angles round.** `rem_euclid(360.0)` returns exactly 360 for a tiny negative angle; normalise it to 0 before validating.
- After changing a threaded part's build, run two parts of one thread through the host: each must weigh what it weighs alone (`parts_keep_their_own_threads` in `tests/geometry_test.py`).
