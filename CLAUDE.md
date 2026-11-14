# TechSquid SolidCraft Plugins

Rust workspace of SolidCraft plugins. Each plugin is a WebAssembly component (`craft:plugin@0.1.0`, capability `commands@0.1`) that changes the model only by running SolidCraft host commands.

## Docs

- `README.md` is the repo index. It has the Plugins table (each row links to that plugin's README), plus layout, build, host tests, examples and releases. Nothing specific to one plugin goes here.
- `plugins/<id>/README.md` is the full user guide for one plugin: Install, Use (with the parameter table), its size tables, Standards (with source and test columns), Names (body, hole and feature names) and Older field names. `build.sh` copies it into the release zip, so it must read on its own without the repo around it.

A new plugin gets its own README with those sections and a row in the main README's Plugins table.

## Layout

- `crates/plugin-kit`: code any plugin needs. It holds the `Runner` trait and the `guest!` macro that exports the wasm entry (`host.rs`), dialog fields and `FieldError` (`input.rs`), placement on a selected or named sketch point (`place.rs`), parts placed as components (`component.rs`), and body and feature names (`name.rs`). Nothing fastener-specific goes here.
- `crates/fasteners`: fastener standards and geometry, such as size designations, metric sizes, heads, drives, nuts, clearance holes and the hex profile. It depends on `plugin-kit`.
- `crates/extrusions`: slotted aluminium rails. `slot.rs` holds the T-slot and V-slot shapes with their sources, and `profile.rs` builds a rail's section from a slot and a grid of 20 mm cells. It depends on nothing.
- `plugins/techsquid-<name>`: one plugin each, with its `plugin.json` manifest and README. The entry and its parameter parsing live in the plugin, and so does a catalog only one plugin uses (the insert catalog).
- `wit/plugin.wit`: a copy of SolidCraft's `craft:plugin@0.1.0`. `guest!` generates bindings from `../../wit`.
- `examples/`: `<name>.json` is a design that opens in SolidCraft. `<name>.steps.json` builds that design one command at a time.
- `tests/*.py`: host tests that run the built packages through `solidcraft-cli`.

A new family of plugins gets its own crate beside `fasteners` and `extrusions`, and reuses `plugin-kit`. Don't make it depend on `fasteners` just for one helper. If a helper is generic, move it to `plugin-kit`.

## Plugin ids

The plugin id, the folder name under `plugins/`, the Cargo package name and the `dist/<id>/` folder are all the same string, `techsquid-<name>`. SolidCraft plugin ids share one flat namespace (1 to 64 lowercase letters, digits and inner hyphens), and they key the install folder, Preferences and plugin data saved in designs. Treat an id as permanent once it has shipped.

The workspace members, `build.sh`, both workflows and the host tests find the plugins themselves: they take every folder under `plugins/` (the tests take those with a `plugin.json`). So adding a plugin means:
- a folder `plugins/techsquid-<name>` with `Cargo.toml`, `plugin.json`, `README.md` and `src/lib.rs`
- a test in `src/lib.rs` that asserts the manifest id
- a case for it in `CASES` in `tests/cli_test.py`, which fails until it has one
- a row in the Plugins table in README.md

Don't put anything but plugins under `plugins/`, or anything but library crates under `crates/`. The globs would pick it up.

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --locked --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace --locked
./build.sh
```

`./build.sh` needs the `wasm32-unknown-unknown` target and `wasm-tools`. The host tests need `./build.sh` to have run first:

```bash
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/cli_test.py
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/geometry_test.py
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/examples_test.py
```

The signed macOS SolidCraft build gets SIGKILLed when it loads a plugin. On that build the tests report SKIP rather than FAIL, so a run that is all SKIP has proved nothing. Use an unsigned, locally built `solidcraft-cli`. After a plugin change that alters geometry, run `examples_test.py --update` to rewrite `examples/*.json`.

## Conventions

- **Every standards table names its source** in a comment, and a unit test repeats the table from that source so the code and the source can't drift apart. When a table is added or changed, update the Standards table in that plugin's README too.
- **Errors:** an error that starts with `<field_key>: ` is shown against that dialog field. One that starts with `<command.id>: ` is reported as a failed command. Anything else is a trap. Return `FieldError` from parsing.
- **Field names are shared across plugins** (`size`, `custom_size`, `create`, `sketch`, `sketch_point`, `thread_modeled`). When a field is renamed, keep reading the old name with `input::field(v, new, old)` so older scripts keep working, and list the old name under Older field names in the plugin's README.
- **Body names come from `name.rs`:** the kind and then the size (`Screw M3 x 12 socket`), using a plain `x`, never `×`. Get a free name from `host::free_body_name` before creating a body. If the name is taken, the host calls the body `BodyN`, and that name doesn't survive a rebuild.
- **Every host command a plugin runs** must appear in the `commands` list in its `plugin.json`.
- **Versions:** all plugins share `workspace.package.version`. A `v*` tag releases them all, and the tag must match the workspace version and every `plugin.json`.
- Write docs and comments in plain, direct sentences like the existing ones.
