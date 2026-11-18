# Contributing to the TechSquid SolidCraft plugins

Agents: start from the router in [AGENTS.md](AGENTS.md), which points to a short skill in [`skills/`](skills/README.md) for each kind of work. This page is the full human guide.

These plugins add configurable hardware to [SolidCraft](https://github.com/tangency-labs/solidcraft): screws, heat-set inserts, nuts, aluminium extrusions, bearings and V-wheels, with the holes and pockets they need. Each plugin is a WebAssembly component (`craft:plugin@0.1.0`, capability `commands@0.1`) that changes the model only by running SolidCraft host commands. Help is welcome, from a one-line fix to a new family of parts.

## Get it running

You need [rustup](https://rustup.rs/) with the `wasm32-unknown-unknown` target, [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools), and Python 3.

```bash
rustup target add wasm32-unknown-unknown
cargo test --workspace --locked
./build.sh
```

`./build.sh` writes one package per plugin to `dist/<id>/`. The host tests and example recipes run plugins from those folders. A release zips each one for **Plugins > Install plugin…** in SolidCraft.

The host tests in `tests/` drive a real SolidCraft. They need an unsigned, locally built `solidcraft-cli` from the [SolidCraft repository](https://github.com/tangency-labs/solidcraft) that meets the [host requirements](docs/parts-library.md#host-requirements):

```bash
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/cli_test.py
```

The signed macOS SolidCraft build is killed when it loads a plugin, so on it the tests report SKIP. A run that is all SKIP has proved nothing.

## How the code is laid out

| Where | What |
|---|---|
| `plugins/techsquid-<name>/` | One plugin: `plugin.json` (dialogs and the host commands it may run), `src/lib.rs`, a short `README.md` and a full `REFERENCE.md` |
| `crates/plugin-kit` | What every plugin shares: the host runner, dialog fields and errors, placement on sketch points, reusable components, names |
| `crates/parts` | Product and representation identity, with no host dependency |
| `crates/fasteners`, `crates/extrusions`, `crates/bearings`, `crates/wheels` | One crate per family of parts: standards, sourced dimensions and geometry |
| `tools/catalog-tool` | Generates catalog choices and reference tables from the crates |
| `tools/icons` | Draws each command's icon into `plugins/techsquid-<name>/icons/` |
| `tests/` | Python tests that build parts through `solidcraft-cli` |
| `examples/` | Example folders with a README, saved design, replay recipe and pictures |
| `docs/` | Shared contracts and `docs/sources/`, the evidence behind measured dimensions |

The six plugins and the crate each one uses:

| Plugin | Entries | Family crate |
|---|---|---|
| [Screws](plugins/techsquid-screws/README.md) | `make-screw` | `fasteners` |
| [Heat-set inserts](plugins/techsquid-inserts/README.md) | `make-insert` | `fasteners` |
| [Hexagon nuts](plugins/techsquid-nuts/README.md) | `make-nut` | `fasteners` |
| [Extrusions](plugins/techsquid-extrusions/README.md) | `make-extrusion`, `make-t-nut` | `extrusions` |
| [Bearings](plugins/techsquid-bearings/README.md) | `make-bearing`, `make-csk` | `bearings` |
| [V-wheels](plugins/techsquid-v-wheels/README.md) | `make-v-wheel` | `wheels`, which uses `bearings` |

Shared behaviour belongs in a shared crate. A generic helper goes in `plugin-kit`. A new family of parts gets its own crate and doesn't depend on `fasteners` for one helper. Plugins never call each other at runtime; V-wheels reuses the bearing factory through Rust. [DEVELOPMENT.md](DEVELOPMENT.md) has the architecture notes, and [docs/parts-library.md](docs/parts-library.md) the reuse and BOM rules.

## Adding a part or a plugin

**A new part in an existing family** (a new screw head, a new rail profile) goes in that family's crate, with its plugin exposing it through a field. Copy the closest existing part. The [parts skill](skills/techsquid-parts/SKILL.md) lists what a part needs: a sourced table, a body name, a reusable definition, and a build site if its thread is cut.

**A new plugin** is a folder `plugins/techsquid-<name>`. The workspace, `build.sh`, CI and the tests find it on their own. It needs:

- `Cargo.toml`, `plugin.json`, `README.md`, `REFERENCE.md` and `src/lib.rs`. Copy `plugins/techsquid-v-wheels`, the smallest plugin.
- A unit test in `src/lib.rs` that asserts the manifest id.
- A case in `CASES` in `tests/cli_test.py`, which fails until it has one.
- An example in `examples/` that demonstrates the new plugin, a native image linked from the plugin guide, and a row in the root Plugin List.

The id `techsquid-<name>` is also the folder, the Cargo package and the `dist/` folder. SolidCraft uses it for the install folder, Preferences and plugin data saved in designs, so it can't change once shipped. The [plugins skill](skills/techsquid-plugins/SKILL.md) walks through the steps.

## Rules

1. **Plugins change the model only through host commands**, and every command a plugin runs is listed in its `plugin.json`.
2. **Every standards table names its source** in a comment, and a unit test repeats the table from that source. A changed table updates the Standards table in that plugin's REFERENCE.md.
3. **Errors name what they are about.** An error that starts with `<field_key>: ` shows against that dialog field. One that starts with `<command.id>: ` is reported as a failed command. Anything else is a trap.
4. **Field names are shared** across plugins (`size`, `custom_size`, `create`, `placement`, `thread_modeled`). A breaking field change refuses the old key, with no alias, and lists it under Older field names in the reference.
5. **Body names** are the kind and then the size, with a plain `x`: `Screw M3 x 12 socket`.
6. **A cut in one placement is a cut in all of them.** Every placement of a component shares its body, so anything cut into a part belongs in its own build and name.
7. **A part with a cut thread is built on a site, not on the origin**, so its thread can't land on another part's bore. The [parts skill](skills/techsquid-parts/SKILL.md) explains why.
8. **Measured, not guessed.** Geometry tests compare against values worked out by hand or from source measurements, never against the plugin's own output.
9. **One version.** All plugins share `workspace.package.version`.

## Tests and examples

| What | Run |
|---|---|
| Formatting and lints | `cargo fmt --all -- --check`, then `cargo clippy --workspace --all-targets --locked -- -D warnings` and the same with `--target wasm32-unknown-unknown` |
| Unit tests | `cargo test --workspace --locked` |
| Packages | `./build.sh` (also checks the generated catalog tables) |
| Agent docs | `python3 tests/skills_test.py` |
| Host tests | `SOLIDCRAFT_CLI=… python3 tests/<suite>_test.py`, after `./build.sh` |

CI runs the cargo checks and `./build.sh`, so run `skills_test.py` and the suites for the area you changed and list their results in the PR. Every saved design in `examples/` has a `.steps.json` recipe that `tests/examples_test.py` rebuilds and measures. After a change that alters geometry, run `examples_test.py --update`, then render the pictures again with the commands in [DEVELOPMENT.md](DEVELOPMENT.md#examples).

## Sending a change

1. Check open pull requests and issues for the same work first.
2. Branch from `main` and keep the PR to one topic. Name the branch `<type>/<area>/<topic>`, such as `fix/nuts/pocket-depth`.
3. Use conventional commit titles scoped by plugin or crate: `fix(screws): …`, `feat(bearings)!: …`. The PR title becomes the commit on `main`, so write it for someone who hasn't seen the issue: which plugin changes, and what a user can now do.
4. Run the checks above before you push.
5. Start the description with what the change does in plain words. Then list the changes, any behaviour a user or a saved design sees differently, what is left out on purpose, and each check you ran with its results. For geometry, give the measured and expected numbers. For a visible change, include a native capture.

Use whatever tools you like, AI agents included, as long as the change meets the rules above. Agents should run the reproduction and capture the evidence themselves.

## Releases

Set the new version in `Cargo.toml`, `Cargo.lock` and every `plugins/*/plugin.json`, and add `release-notes/v<version>.md` with breaking changes and migration steps. A pushed `v*` tag checks the versions match, builds one zip per plugin with its checksum, and drafts the GitHub release for a maintainer to publish.

## Licence

By contributing you agree that your contribution is licensed under the [MIT licence](LICENSE), like the rest of the repository.
