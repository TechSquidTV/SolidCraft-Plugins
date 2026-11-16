# TechSquid SolidCraft Plugins

Rust workspace of SolidCraft plugins. Each plugin is a WebAssembly component (`craft:plugin@0.1.0`, capability `commands@0.1`) that changes the model only by running SolidCraft host commands.

## Docs

- `README.md` is the short repository index: package links and showcase examples.
- `DEVELOPMENT.md` owns build/test instructions, the repository map and example rendering commands.
- `plugins/<id>/README.md` is a short user guide with a native CAD preview, installation and common use. `REFERENCE.md` beside it owns full parameter tables, standards and sources, naming, limitations and retired-field migration. `build.sh` packages both with their pictures and the shared docs.
- `docs/catalog-contract.md` owns the public discovery contract; `docs/parts-library.md` owns shared reuse and BOM rules. Package guides link to these instead of restating them.
- `docs/sources/` contains source evidence and independent measurements, not runtime dimension tables. Domain crates own the compiled data; `catalog-tool` generates catalog-owned form choices and bearing/wheel reference tables from it.
- Every saved design in `examples/` has a matching `.steps.json` recipe checked by `tests/examples_test.py`. Previews are native captures; keep their rendering commands in `DEVELOPMENT.md`.

A new plugin gets both guides, an example and preview, and a row in the root Plugins table.

## Layout

- `crates/parts`: host-independent discovery, canonical product/representation identity and shared detail modes.
- `crates/bearings` and `crates/wheels`: fixed supplier products and native exterior geometry. Wheel composition and advertised members share one recipe; wheels call the bearing factory directly.
- `crates/plugin-kit`: code any plugin needs. It holds the `Runner` trait and the `guest!` macro that exports the wasm entry, or several entries (`host.rs`), dialog fields and `FieldError` (`input.rs`), the Create choice of a part and its cut (`create.rs`), placement on a selected or named sketch point (`place.rs`), parts placed as components (`component.rs`), timeline folders (`timeline.rs`), and body and feature names (`name.rs`). Nothing fastener-specific goes here.
- `crates/fasteners`: fastener standards and geometry, such as size designations, metric sizes, heads, drives, nuts, clearance holes and the hex profile. It depends on `plugin-kit`.
- `crates/extrusions`: slotted aluminium rails. `slot.rs` holds the T-slot and V-slot shapes with their sources, `profile.rs` builds a rail's section from a slot and a grid of 20 mm cells, `machining.rs` holds the taps and screw holes cut into a rail, and `tnut.rs` holds the slot nuts. It depends on nothing.
- `plugins/techsquid-<name>`: one plugin each, with its `plugin.json` manifest and README. The entry and its parameter parsing live in the plugin, and so does a catalog only one plugin uses (the insert catalog).
- `wit/plugin.wit`: a copy of SolidCraft's `craft:plugin@0.1.0`. `guest!` generates bindings from `../../wit`.
- `examples/`: `<name>.json` is a design that opens in SolidCraft. `<name>.steps.json` builds that design one command at a time.
- `tests/*.py`: host tests that run the built packages through `solidcraft-cli`; `harness.py` and `headless.py` own the shared test transport.
- `tools/`: build-time generators and source-measurement tooling. Generated output belongs in ignored `target/` and `dist/`, never in the source directories.

A new family of plugins gets its own crate beside `fasteners` and `extrusions`, and reuses `plugin-kit`. Don't make it depend on `fasteners` just for one helper. If a helper is generic, move it to `plugin-kit`.

## Plugin ids

The plugin id, the folder name under `plugins/`, the Cargo package name and the `dist/<id>/` folder are all the same string, `techsquid-<name>`. SolidCraft plugin ids share one flat namespace (1 to 64 lowercase letters, digits and inner hyphens), and they key the install folder, Preferences and plugin data saved in designs. Treat an id as permanent once it has shipped.

The workspace members, `build.sh`, both workflows and the host tests find the plugins themselves: they take every folder under `plugins/` (the tests take those with a `plugin.json`). So adding a plugin means:
- a folder `plugins/techsquid-<name>` with `Cargo.toml`, `plugin.json`, `README.md`, `REFERENCE.md` and `src/lib.rs`
- a test in `src/lib.rs` that asserts the manifest id
- a case for it in `CASES` in `tests/cli_test.py`, which fails until it has one
- a row in the Plugins table in README.md
- a small example in `examples/` and a picture rendered from it, beside its README

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

The signed macOS SolidCraft build gets SIGKILLed when it loads a plugin. On that build the tests report SKIP rather than FAIL, so a run that is all SKIP has proved nothing. Use an unsigned, locally built `solidcraft-cli`. After a plugin change that alters geometry, run `examples_test.py --update` to rewrite `examples/*.json`. Then render the pictures again with the snapshot commands in DEVELOPMENT.md.

## Conventions

- **Every standards table names its source** in a comment, and a unit test repeats the table from that source so the code and the source can't drift apart. When a table is added or changed, update the Standards table in that plugin's reference guide too.
- **Errors:** an error that starts with `<field_key>: ` is shown against that dialog field. One that starts with `<command.id>: ` is reported as a failed command. Anything else is a trap. Return `FieldError` from parsing.
- **Field names are shared across plugins** (`size`, `custom_size`, `create`, `sketch`, `sketch_point`, `thread_modeled`). Breaking field changes reject retired keys without compatibility aliases or fallback values. List replacements under Older field names in the plugin guide. Validate workflow fields from the packaged manifest using `input::workflow_fields`.
- **Body names come from `name.rs`:** the kind and then the size (`Screw M3 x 12 socket`), using a plain `x`, never `×`. Reusable definitions reserve a free component/body prefix through `component::Library`; independent bodies use `host::free_body_name`. If the name is taken, the host calls the body `BodyN`, and that name doesn't survive a rebuild.
- **A cut in one placement is a cut in all of them.** Every placement of a component shares its body, so a hole drilled in one rail appears in every copy. Anything cut into a part belongs in its build, and in its name, so the cut part is a definition of its own: a rail's taps and screw holes are `make-extrusion` fields.
- **Every host command a plugin runs** must appear in the `commands` list in its `plugin.json`.
- **A part with a cut thread is built on a site, not on the origin.** SolidCraft 0.41 finds the face a modeled thread goes on by a point among every body, whatever the feature's `targets`, and every component is built where the design's origin is. A bore or shank of the same size on the same axis there takes the thread: an M5 nut used to thread the bore of a 2020 rail standing on the origin, and a second length of a screw used to thread the first. So `Definition::site` is `Site::Compact` for a small part whose thread is cut and `Site::Apart` for a rail whose end is, and `Site::Origin` otherwise. `Library::ensure` works the site out from the new component's id, which no other component has, hands the build that frame, and `Library::place` takes it off each placement. The kernel refuses some cuts away from the origin (a hexagon's chamfers, some heads), so a small part is made on the origin and moved with `solid.move` before its thread is cut. A test host overrides `Runner::site` to stay on the origin. After a change to a build, run parts of one thread in pairs through the host: each must weigh what it weighs alone (`parts_keep_their_own_threads` in `tests/geometry_test.py`).
- **Versions:** all plugins share `workspace.package.version`. A `v*` tag releases them all, and the tag must match the workspace version and every `plugin.json`.
- Write docs and comments in plain, direct sentences like the existing ones.
