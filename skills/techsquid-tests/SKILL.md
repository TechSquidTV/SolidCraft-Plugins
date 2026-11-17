---
name: techsquid-tests
description: Use when writing or running tests for the TechSquid SolidCraft plugins - Rust unit tests, Python host tests through solidcraft-cli, saved examples and their .steps.json recipes, or the pictures rendered from them.
---

# Tests, examples and pictures

## Rust unit tests

`cargo test --workspace --locked`. Tests sit in each file's `mod tests`. They check field parsing, standards tables against their sources, and the commands `make` sends, using a recording `Runner` (see the `Rec` and `Taken` runners in `plugins/techsquid-nuts/src/lib.rs`). They need no host.

## Host tests

The Python suites in `tests/` drive a real `solidcraft-cli`. Build first, then point them at the CLI:

```bash
./build.sh
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/cli_test.py
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/geometry_test.py
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/examples_test.py
```

- Use an unsigned, locally built `solidcraft-cli` that meets the [host requirements](../../docs/parts-library.md#host-requirements). The signed macOS build is SIGKILLed when it loads a plugin and the tests report SKIP. A run that is all SKIP proved nothing; never report it as passing.
- Run every suite whose area you touched; before a release or a shared-crate change, run all of `tests/*_test.py`. Report each file's pass count in the PR.
- Shared transport and helpers belong in `tests/harness.py` and `tests/headless.py`, not in a single suite.
- Geometry is checked against values worked out by hand or from source measurements, never against the plugin's own output.

## Examples

Every `examples/<name>.json` has a `<name>.steps.json` recipe: one command per step with a `caption`, plugin steps as `plugin.run` with `"path": "dist/<id>"` and `"approval": "@approval"`, and an `expect` block. `tests/examples_test.py` rebuilds each recipe, measures it, and checks the saved design matches.

After a change that alters geometry:

```bash
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/examples_test.py --update
```

Then check the diff of `examples/*.json` is what you meant.

## Pictures

Pictures are native captures with `solidcraft-cli snapshot`. Each one's exact command is in [DEVELOPMENT.md](../../DEVELOPMENT.md#examples). Re-render the pictures whose designs changed, with the same command. A new picture adds its command there.
