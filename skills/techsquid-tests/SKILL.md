---
name: techsquid-tests
description: Use this skill when proving a TechSquid plugin change works - writing Rust unit tests, running the Python host tests through solidcraft-cli, regenerating saved examples and their .steps.json recipes, rendering README pictures, or deciding which suites a change needs. Use it before reporting any change to geometry as done, and whenever tests SKIP, a saved example no longer matches, or someone asks "does it still build the same parts?".
compatibility: Host tests need python3, ./build.sh output and an unsigned, locally built solidcraft-cli. Rendering the README banner also needs Chrome or Chromium.
---

# Tests, examples and pictures

## Which tests

| Changed | Run |
|---|---|
| Anything | `cargo test --workspace --locked` and `./build.sh` |
| What a plugin builds | the plugin's host suites, `tests/geometry_test.py` and `tests/examples_test.py` |
| A plugin's entries | `tests/cli_test.py` and `tests/plugin_entries_test.py` |
| `plugin-kit`, `parts` or reuse | every `tests/*_test.py` |
| Agent docs | `python3 tests/skills_test.py` |

Host suites by plugin: screws `screw_references_test.py`, nuts `nut_references_test.py`, extrusions `rail_references_test.py`, `sketch_rails_test.py`, `sketch_selection_test.py`, bearings `bearing_geometry_test.py`, `csk_geometry_test.py`, V-wheels `wheel_geometry_test.py`, the Placement field `profile_picks_test.py`, shared library `catalog_reuse_test.py`, `parts_library_test.py`, `parts_content_test.py`, `parts_directory_test.py`, `parts_inspection_test.py`.

## Unit tests

Tests sit in each file's `mod tests` and need no host. A bug fix starts with a test that fails on `main`, then the fix. Check the commands `make` sends with a recording `Runner`, as in `plugins/techsquid-nuts/src/lib.rs`.

## Host tests

```bash
./build.sh
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/geometry_test.py
```

Build the CLI from a SolidCraft checkout that meets the [host requirements](../../docs/parts-library.md#host-requirements): `cargo build --profile ci -p solidcraft-cli` gives `target/ci/solidcraft-cli`. The big suites (examples, screw and nut references) take many minutes; run them in the background.

Shared helpers belong in `tests/harness.py` and `tests/headless.py`. Expected values come from hand calculation or source measurements, never from the plugin's own output.

## Examples and pictures

Every `examples/<name>/` folder contains the README, design, recipe and screenshots. The `<name>.steps.json` recipe builds the saved design; `tests/examples_test.py` rebuilds it and checks the saved design matches. After a change that alters geometry or BOM data:

1. `SOLIDCRAFT_CLI=… python3 tests/examples_test.py --update`
2. `git diff --stat examples/`: every changed design must be one your change explains.
3. Run `examples_test.py` again without `--update`; it must pass.
4. Run `SOLIDCRAFT_CLI=… python3 tests/render_examples.py` to regenerate the gallery screenshots and README banner. [Rendering workflow](../../DEVELOPMENT.md#render-the-screenshots)

## Gotchas

- **The signed macOS SolidCraft is SIGKILLed when it loads a plugin**, and the suites print SKIP. A run that is all SKIP checked nothing; never report it as passing.
- **A stale `dist/` tests the old code.** Run `./build.sh` after every change before a host suite. Keep packages stable while host suites run: rebuilding can invalidate cached approvals. If that happens, rerun the affected suite against the finished package.
- **Floating-point boxes differ in the last digits** between hosts; compare with a small absolute tolerance, as the suites do, not with `==`.
- **Report every file with its pass, fail and skip counts** in the PR, and say which suites you didn't run.
