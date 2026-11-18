---
name: techsquid-plugins
description: Use this skill to find your way around the TechSquid SolidCraft plugins workspace - which plugin, entry or crate holds a behaviour, which crate new code belongs in - and when adding a new plugin or a new family of parts. Use it when the request names a kind of hardware the repository doesn't make yet ("add washers", "a plugin for linear rails"), or asks where something is implemented. For a new size or variant of an existing part, use techsquid-parts instead.
---

# Where things live, and adding a plugin

## The plugins

| Plugin folder | Entries | Domain crate | Builds |
|---|---|---|---|
| `plugins/techsquid-screws` | `make-screw` | `fasteners` | ISO metric screws, set screws, threaded rod, and their holes |
| `plugins/techsquid-inserts` | `make-insert` | `fasteners` | CNC Kitchen heat-set inserts and their holes; the insert catalog stays in the plugin, its only user |
| `plugins/techsquid-nuts` | `make-nut` | `fasteners` | ISO 4032 nuts, pockets and bolt holes |
| `plugins/techsquid-extrusions` | `make-extrusion`, `make-t-nut` | `extrusions` | T-slot and V-slot rails, fixed or sketch-driven, and slot nuts |
| `plugins/techsquid-bearings` | `make-bearing`, `make-csk` | `bearings` | Radial bearings and CSK clutches |
| `plugins/techsquid-v-wheels` | `make-v-wheel` | `wheels` (uses `bearings`) | OpenBuilds V-wheels, bare or assembled |

Every plugin also depends on `plugin-kit` and `parts`. Inside a plugin, `src/lib.rs` holds `parse`, `make`, the `plugin_kit::guest!` block and the unit tests. `src/provider.rs` holds the catalog `REVISION`, plus `representation` and `part_number` when the plugin owns them (V-wheels takes both from the `wheels` crate); catalog discovery lives in its test module. Other files hold the part's own build.

## Which crate

- **`plugin-kit`**: anything any plugin could use. Host runner and `guest!` (`host.rs`), field parsing and `FieldError` (`input.rs`), placement on sketch points (`place.rs`), components, reuse and sites (`component.rs`, `component/`), names (`name.rs`), revolved sections (`revolve.rs`), timeline folders (`timeline.rs`).
- **`parts`**: host-independent product and representation identity. No host calls.
- **A domain crate** (`fasteners`, `extrusions`, `bearings`, `wheels`): standards, sourced dimensions and the geometry of one family. A plugin that needs another family's part depends on that crate, as `wheels` depends on `bearings`; plugins never call each other at runtime.
- **The plugin**: dialog fields, selection, cuts into the receiving part, joints, and catalog data only it uses.

A helper used by two families moves to `plugin-kit`. Don't make a new family depend on `fasteners` for one helper.

**New family or existing crate?** A part that goes with a fastener and is sized by its metric thread (a washer, a spacer, a threaded insert) belongs in `fasteners`, which already has the metric sizes and ISO 273 clearances. A new crate is for parts whose standards and sources share nothing with the existing crates, as `bearings` and `wheels` did.

## Adding a plugin

- [ ] Pick the id `techsquid-<name>` and check no other plugin or release uses it.
- [ ] Copy `plugins/techsquid-v-wheels`, the smallest plugin, to `plugins/techsquid-<name>`. Set `name` in `Cargo.toml`; keep `crate-type = ["cdylib", "rlib"]` and the workspace fields. Nothing else needs registering.
- [ ] Write `plugin.json`: `id`, `name`, `description`, `version`, one entry per dialog with a menu slot, and the host commands it runs (`skills/techsquid-fields/SKILL.md`).
- [ ] Draw the entry's icon (`skills/techsquid-icons/SKILL.md`): three candidate views for the user to pick from, then the chosen one in `ICONS` in `tools/icons/draw.py`, and `icons/<entry>.svg` named in the entry's `input_schema`.
- [ ] In `src/lib.rs`, set `pub const PLUGIN`. `parse` starts with `input::workflow_fields`, ends with `parts::catalog::validate_selection(v, provider::REVISION, &representation)` and `place::parse_input`. `make` is `place::resolve_each` then `component::insert`. List each entry in `plugin_kit::guest!` with `entry`, `noun`, `parse`, `make` and `describe`.
- [ ] Add unit tests: the manifest's `id` equals `PLUGIN`, bad input is refused, and the manifest's `runtime.commands` equals the commands the code sends. V-wheels and Bearings lack the last one; copy it from `plugins/techsquid-nuts/src/lib.rs`.
- [ ] Put the geometry in its domain crate (`skills/techsquid-parts/SKILL.md`). A new family gets a new crate beside `fasteners` and `extrusions`, added to `[workspace.dependencies]`.
- [ ] Add a case to `CASES` in `tests/cli_test.py`: `"<id>": (entry, params, label, (across, height))`, the size of the part it builds. A second entry goes in `OTHER_ENTRIES`; a part that isn't one solid body goes in `BODY_COUNTS`.
- [ ] Add a gallery example or extend an existing one. Save the recipe as `examples/<name>/<name>.steps.json` alongside the README and screenshots. Plugin steps use `"path": "dist/<id>"` and `"approval": "@approval"`. Work out at least `bodies` and `bbox` for `expect`, then run `tests/examples_test.py --update examples/<name>/<name>.steps.json` to write the saved design.
- [ ] Add `README.md`, `REFERENCE.md`, a picture beside the README, a row in the root README's Plugin List, and rows in both DEVELOPMENT.md tables, Plugins and the standalone examples (`skills/techsquid-docs/SKILL.md`). Adding the part to the showcase example is optional.
- [ ] Run `cargo test --workspace --locked`, `./build.sh` (check `dist/techsquid-<name>/plugin.wasm` exists), then `tests/cli_test.py` and `tests/examples_test.py` on an unsigned CLI. Fix and repeat until they pass with no SKIPs.

## Gotchas

- **The id is permanent.** It keys the install folder, Preferences and plugin data saved in designs. It is 1 to 64 lowercase letters, digits and inner hyphens, and equals the folder, the Cargo package and `dist/<id>/`.
- **The globs pick up every folder.** The workspace, `build.sh`, CI and the tests take each folder under `plugins/` as a plugin and under `crates/` as a library. Tools go under `tools/`; anything else breaks the build.
- **`tests/cli_test.py` fails until the plugin has a case**, and a case for a plugin that doesn't exist fails too.
- **`tests/plugin_entries_test.py` checks that unknown keys are refused.** Its `retired` map names one retired key per plugin; a new plugin needs no entry until it retires a field.
- **The nuts are not a simple template.** They resolve places with `place::resolve_all` and cut pockets on their own path. Copy V-wheels for the plugin shape, and the nut only for hexagon geometry.
- **Plugins never call each other.** A V-wheel's bearings come from the `bearings` crate, not the Bearings plugin, so each package installs alone.
- **`provider.rs` catalog code is test-only.** Keep `REVISION`, `representation` and `part_number` public; put any `Catalog` implementation in its test module. No entry exposes discovery.
