---
name: techsquid-plugins
description: Use when finding where a plugin, entry or crate lives, deciding which crate code belongs in, or adding a new plugin or a new family crate to the TechSquid SolidCraft plugins.
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

Every plugin also depends on `plugin-kit` and `parts`. Inside a plugin, `src/lib.rs` holds `parse`, `make`, the `plugin_kit::guest!` block and the unit tests; `src/provider.rs` holds `REVISION`, `representation` and `part_number` (catalog discovery lives in its test module); other files hold the part's own build.

## Which crate

- **`plugin-kit`**: anything any plugin could use. Host runner and `guest!` (`host.rs`), field parsing and `FieldError` (`input.rs`), placement on sketch points (`place.rs`), components, reuse and sites (`component.rs`, `component/`), names (`name.rs`), revolved sections (`revolve.rs`), timeline folders (`timeline.rs`).
- **`parts`**: host-independent product and representation identity. No host calls.
- **A domain crate** (`fasteners`, `extrusions`, `bearings`, `wheels`): standards, sourced dimensions and the geometry of one family. A plugin that needs another family's part depends on that crate, as `wheels` depends on `bearings`; plugins never call each other at runtime.
- **The plugin**: dialog fields, selection, cuts into the receiving part, joints, and catalog data only it uses.

A helper used by two families moves to `plugin-kit`. Don't make a new family depend on `fasteners` for one helper.

## Adding a plugin

1. Pick the id `techsquid-<name>`: lowercase letters, digits and inner hyphens, at most 64 characters. It keys the install folder, Preferences and data saved in designs, so it is permanent once shipped.
2. Copy the smallest plugin, `plugins/techsquid-v-wheels`, to `plugins/techsquid-<name>`. Set `name` in `Cargo.toml`; keep `crate-type = ["cdylib", "rlib"]` and the workspace fields. The workspace, `build.sh` and CI pick the folder up with no other edit.
3. Write `plugin.json`: `id`, `name`, `description`, `version` (the workspace version), one entry per dialog, and the host commands it runs. See `skills/techsquid-fields/SKILL.md`.
4. In `src/lib.rs`, set `pub const PLUGIN`, write `parse` (fields to typed `Params`) and `make` (one `component::insert` call for a placed part), and list each entry in `plugin_kit::guest!`. Add a unit test that asserts the manifest's `id` equals `PLUGIN`.
5. Put the part's geometry in its domain crate. A new family gets a new crate beside `fasteners` and `extrusions`, added to `[workspace.dependencies]`. See `skills/techsquid-parts/SKILL.md`.
6. Add a case to `CASES` in `tests/cli_test.py`; it fails until you do.
7. Add `README.md` and `REFERENCE.md`, a small `examples/<name>.json` with its `.steps.json`, a picture beside the README, and a row in the Plugin List of the root `README.md` and the table in `DEVELOPMENT.md`. See `skills/techsquid-docs/SKILL.md`.

Don't put anything but plugins under `plugins/`, or anything but library crates under `crates/`. Tools go under `tools/`.
