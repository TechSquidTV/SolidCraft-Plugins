# TechSquid SolidCraft plugins: instructions for agents

A Rust workspace of [SolidCraft](https://github.com/tangency-labs/solidcraft) plugins. Each plugin is a WebAssembly component (`craft:plugin@0.1.0`, capability `commands@0.1`) that changes the model only by running SolidCraft host commands. This file is a router: the hard rules, then the skill to read for your task. Humans: [CONTRIBUTING.md](CONTRIBUTING.md).

## Map

- `plugins/techsquid-<name>/`: one plugin each. `plugin.json` (dialog fields and the host commands it may run), `src/lib.rs` (parse, make, `guest!`), `README.md` (short user guide), `REFERENCE.md` (full fields, standards, limits).
- `crates/`: library crates only. `plugin-kit` (host, fields, placement, components, names), `parts` (product identity), and one domain crate per family: `fasteners`, `extrusions`, `bearings`, `wheels`.
- `tests/`: Python host tests through `solidcraft-cli`. `examples/`: saved designs, their `.steps.json` recipes and pictures.
- `docs/`: shared contracts (`catalog-contract.md`, `parts-library.md`) and `docs/sources/` evidence. `tools/catalog-tool` generates catalog choices and tables.

## Hard rules (what enforces each)

- A plugin changes the model only through host commands, and every command it runs is listed in its `plugin.json`: the host refuses the rest. `skills/techsquid-fields/SKILL.md`.
- The plugin id, folder, Cargo package and `dist/<id>/` are the same `techsquid-<name>`, and a shipped id never changes: each plugin's manifest-id unit test, `tests/cli_test.py`. `skills/techsquid-plugins/SKILL.md`.
- Only plugins under `plugins/`, only library crates under `crates/`: the workspace, `build.sh`, CI and the tests glob both folders.
- Shared behaviour goes in a shared crate. A generic helper goes in `plugin-kit`; a new family gets its own crate and doesn't depend on `fasteners` for one helper. `skills/techsquid-plugins/SKILL.md`.
- Every standards table names its source in a comment, and a unit test repeats it from that source. `skills/techsquid-parts/SKILL.md`.
- Body names come from `name.rs`: kind, then size, with a plain `x`. A cut belongs to the part's own build and name, because every placement shares the body. A part with a cut thread is built on a site, not the origin. `skills/techsquid-parts/SKILL.md`.
- Errors start with `<field_key>: ` or `<command.id>: `; anything else traps. Retired field keys are refused, with no alias. `skills/techsquid-fields/SKILL.md`.
- Every saved example has a `.steps.json` recipe, and geometry changes regenerate examples and pictures: `tests/examples_test.py`. `skills/techsquid-tests/SKILL.md`.
- Host tests need an unsigned, locally built `solidcraft-cli`; a run that is all SKIP proved nothing. `skills/techsquid-tests/SKILL.md`.
- Catalog choices and reference tables are generated: `cargo run -p catalog-tool -- --write`, checked by `./build.sh`.
- Every entry names a command icon, and the icons are drawn by `python3 tools/icons/draw.py`: `./build.sh`. A new or reworked icon is shown as three candidate views first, and the user picks. `skills/techsquid-icons/SKILL.md`.
- All plugins share `workspace.package.version`; a `v*` tag must match it and every `plugin.json`. `skills/techsquid-contributing/SKILL.md`.
- Each fact has one home: README is the short guide, REFERENCE the full one, DEVELOPMENT.md the build and test notes. Link instead of restating. `skills/techsquid-docs/SKILL.md`.
- Before every commit: `cargo fmt --all -- --check`, both clippy runs with `-D warnings`, `cargo test --workspace --locked`, `./build.sh`, and `python3 tests/skills_test.py` when you touch this file, `skills/` or CONTRIBUTING.md.

## Working on X: read the skill first

| Working on | Read |
|---|---|
| A branch, commit, PR, CI or release | `skills/techsquid-contributing/SKILL.md` |
| Where a plugin lives, a new plugin or a new family crate | `skills/techsquid-plugins/SKILL.md` |
| A new part or component: geometry, standards, names, reuse | `skills/techsquid-parts/SKILL.md` |
| Dialog fields, `plugin.json`, errors, host commands | `skills/techsquid-fields/SKILL.md` |
| A command icon: drawing, redrawing or reviewing one | `skills/techsquid-icons/SKILL.md` |
| Unit tests, host tests, examples, pictures | `skills/techsquid-tests/SKILL.md` |
| A plugin guide, reference, source ledger or release notes | `skills/techsquid-docs/SKILL.md` |
