# Developer reference

TechSquid's plugins for [SolidCraft](https://github.com/tangency-labs/solidcraft). Each one is a WebAssembly component (`craft:plugin@0.1.0`, capability `commands@0.1`), the same package shape as [solidcraft-gears](https://github.com/tangency-labs/solidcraft-gears). A plugin changes the model only by running SolidCraft commands.

## Plugins

| Plugin | Id | Family | What it builds |
|---|---|---|---|
| [Screws](plugins/techsquid-screws/README.md) | `techsquid-screws` | Fasteners | Metric machine screws to ISO standards on picked sketch points (socket, button, countersunk, hex, pan and cheese heads, set screws and threaded rod), with any drive, and optionally the clearance, counterbored, countersunk or tapped hole |
| [Heat-set inserts](plugins/techsquid-inserts/README.md) | `techsquid-inserts` | Fasteners | CNC Kitchen heat-set inserts, metric and UN: the straight or stepped hole drilled into the part, and the knurled insert as its own body |
| [Hexagon nuts](plugins/techsquid-nuts/README.md) | `techsquid-nuts` | Fasteners | ISO 4032 hexagon nuts on picked sketch points, and optionally a hexagonal pocket with a bolt hole |
| [Extrusions](plugins/techsquid-extrusions/README.md) | `techsquid-extrusions` | Extrusions | 20 mm series T-slot (MISUMI 5 series) and V-slot (OpenBuilds) rails, 2020 to 2080 and 4040, cut to any length, with tapped ends and screw holes so they screw together, and the MISUMI M5 slot nuts that go in them |
| [Bearings](plugins/techsquid-bearings/README.md) | `techsquid-bearings` | Bearings and clutches | Sourced OpenBuilds and MISUMI radial bearings, plus 55 CSK presets with editable envelopes, keyway configuration and source references |
| [V-wheels](plugins/techsquid-v-wheels/README.md) | `techsquid-v-wheels` | Wheels | Five OpenBuilds products, bare or composed with bearings and an internal shim |

Each plugin's README links to its detailed reference for dialog fields, standards and naming. Each release has one zip per plugin.

## Preparing a release

Keep the workspace version, Cargo lockfile and every plugin manifest on the same version. Add release notes at `release-notes/v<version>.md`, including breaking changes and migration steps, then run the build and relevant checks before tagging the release commit. A pushed `v*` tag builds packages and checksums and creates or updates an unpublished GitHub release draft using those notes. Review and publish that draft separately; rerunning the workflow leaves an already published release untouched.

A draft can also be prepared from a pushed release branch before its tag exists. Set its target to the release commit and upload the versioned packages and example designs. Merge the release preparation before publishing.

## Fasteners

Threads themselves are a host command. `solid.thread` cuts or annotates a cylindrical face from a designation such as `M6` or `M8x1`. The screw plugin places the screw and passes that designation through. The insert and nut plugins use `solid.hole` for the bore. Each plugin does the fastener geometry, and the host owns the thread form.

The three fastener plugins share field names (`size`, `custom_size`, `create`, `placement`, `thread_modeled`) and a naming scheme for bodies: the kind of part, then its size, as in `Screw M3 x 12 socket`.

## Extrusions

A fixed-length rail is one sketch of lines and arcs, extruded once. Sketch-driven mode uses the same section in a native straight-path sweep; start, end and side machining planes reference the source path rather than generated faces. Its section comes from `crates/extrusions`, which builds it from a slot shape and a grid of 20 mm cells. The T-slot is checked against MISUMI's STEP models and the V-slot against OpenBuilds' DXF files; the plugin's reference lists each source and its test.

A rail can be built with its ends tapped M5 and with screw holes across it, so rails screw together with no bracket: see the plugin's README.

A slot nut is one section too, extruded along the slot and tapped. `make-t-nut`, the plugin's second entry, hangs it under the lips of a slot, on a point on the rail's face. Its shape is MISUMI's STEP model of the pre-assembly nut HNTT5-5 or the post-assembly nut HNTP5-5.

## Architecture

A plugin id is its folder, Cargo package and installed package name. Plugins own
workflow fields and their catalog adapters. Shared identity and protocol stay in
`parts`; host state and reusable definitions stay in `plugin-kit`. Domain crates
own compiled dimensions and geometry. The insert catalog stays in its only consumer.
Wheel and bearing packages share factories through Rust dependencies, not runtime
plugin calls.

Each concern has one owner. Short package READMEs link to their
full reference guides and shared architecture docs. Generated catalog tables are
checked against domain data; independent source measurements remain separate test
evidence. Test transports belong in `harness.py`/`headless.py`, not individual suites.
Every saved example has a build recipe and is checked for geometry, catalog identity
and BOM metadata. Rust build output, packages and Python caches are ignored.

The plugins have no build dependency on the adjacent SolidCraft source checkout;
runtime insertion needs the [host inspection fields](docs/parts-library.md#host-requirements).
The [bearing coverage checklist](docs/bearing-coverage.md) records the app parity
baseline and source evidence still needed before adding purchasable variants.

## Build

You need Rust with the `wasm32-unknown-unknown` target, [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools), and Python 3 for packaging manifests and guides.

```
cargo test
./build.sh
```

### Internal part definitions

`crates/parts` owns the host-independent product and representation identities and
internal list/describe/resolve types. Domain providers own configurations, source
measurements and assembly members. These descriptors remain the single source for
insertion and reuse; packages do not export a `catalog` entry or query handler.
See the [internal contract](docs/catalog-contract.md) and [shared library](docs/parts-library.md).

**Rust API change:** `Bearing::product_url()` returns `String`, constructed from
the supplier family and the existing SKU. Rust callers expecting `&'static str`
must accept the owned string or borrow it for the duration of use. Product URLs
and purchasing identities remain the same.

`tools/catalog-tool` generates catalog-owned insertion choices and documentation
tables from the same data, including the Bearings README and reference table.
Run `cargo run -p catalog-tool -- --write` after changing them;
`./build.sh` runs `--check` and refuses drift. Workflow controls remain hand-authored.
Source manifests stay formatted for review; `build.sh` writes compact UTF-8 JSON
to each package and enforces the host's 16 KiB limit on those bytes. All fields,
choices and help text are preserved. The guest reads the same source content;
`tests/cli_test.py` checks that the packaged JSON equals it and fits the limit.

Each entry has a command icon: `"x-solidcraft-icon": "icons/<entry>.svg"` at the top
of its `input_schema`, which SolidCraft 0.44 and later draw in the Plugins menu and
the dialog header. `tools/icons/draw.py` draws every icon in SolidCraft's icon SVG
subset and palette, so they follow the theme; change an icon there and run
`python3 tools/icons/draw.py`. `--sheet FILE.html` previews every icon in both themes;
[skills/techsquid-icons](skills/techsquid-icons/SKILL.md) has the drawing method. `./build.sh` runs `--check`, which refuses drift, an
entry without an icon and an icon outside the subset, and copies `icons/` into the
package. SolidCraft leaves out an icon it refuses and logs why; the command still runs.
The generator must not add API entries.

The internal resolver's `catalog_selection` precondition pins the protocol version,
catalog revision and representation for provider round-trip tests. The ordinary
insertion dialogs resolve directly from their inputs. There is no installed query
route for obtaining a resolved selection; discovery is deferred until the host has
a suitable presentation contract.

Run `SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/plugin_entries_test.py`
after building. It checks that every package offers only insertion workflows,
removed catalog calls preserve the model/selection/undo state, no dormant guest
handler can be enabled by redeclaring the entry, and non-XY insertion still works.
Rust tests retain the internal provider query and round-trip coverage.
`catalog_reuse_test.py` checks the existing families' identity and verified reuse;
`parts_library_test.py` validates real assemblies and composed reuse.

`cargo test` checks the parameter rules and the commands each run sends. `./build.sh` wraps each core module as a component and leaves one `dist/<id>/` for every folder under `plugins/`.

## Test against the host

`tests/cli_test.py` needs the package from `./build.sh` and an unsigned
`solidcraft-cli` meeting the [host requirements](docs/parts-library.md#host-requirements).
The occurrence-context regressions need both new inspection fields; a
0.42 or 0.43 build from before host PR #17999 is insufficient.

```
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/cli_test.py
```

It inspects each package. A screw run must build an M3, an insert run must build the M3 × 5.7 insert, a nut run must build an ISO 4032 M6, an extrusion run must build a 120 mm V-slot 2040, and a bad `head` must be refused as invalid input. The signed macOS build of SolidCraft 0.41.0 has the hardened runtime and no JIT entitlement, so loading a component kills `solidcraft-cli` (`SIGKILL`, code signature invalid). Inspect still succeeds there, and the test skips a run check when the CLI dies that way.

### Rail hole references

Run `SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/rail_references_test.py` after building the packages. It edits the driving circle centers on all four rail faces and checks the actual access and clearance walls, recess depth and volume before and after reopening, including rotated copies, a wide V-slot rail and a tapped rail built away from the origin.

Sketch-driven mode was verified with SolidCraft 0.42.0. Run
`SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/sketch_rails_test.py` after building.
It covers every profile with independent path lengths, translation and tilt edits,
modeled end taps, end-relative cross-holes, a picked subset of lines, repeated runs,
save/reopen, and the 16-rail limit. Run `python3 tests/sketch_selection_test.py` for the
focused check that a pick names its sketch by id: two sketches each have a line `l1`, and it
checks the picked rail’s position and length and refuses lines from both sketches.
`python3 tests/profile_picks_test.py` checks that the inside of a circle places, in every
entry, exactly what the circle itself places, and that any other region is refused.
The plugin estimates its command count before creation;
complex machining can require smaller batches. Its execution allowance also covers
fixing copied section geometry, while the host's 256-call and 30-second limits remain.

The [sketch-driven rail walkthrough](examples/sketch-driven-rails/) has the saved six-rail design and editable parameters.

### Nut pocket references

The naming regression builds multi-pocket batches beside existing names in another
component, including case variants and reserved suffixes, then saves, reopens and
inserts another batch. It checks distinct profile/cut names and their native links.

The same suite checks two consecutive insertions through translated copies and
rotated nested assemblies using a face-attached sketch, with and without nuts.
It verifies the active path, mounting coordinates, actual nut positions and seat
joints, then checks failed-operation rollback and save/reopen. Shared seat tests
also verify that temporary body inspection restores selection after an error.
The same cases run with 100 translated or rotated nested copies and assert that
both consecutive insertions use the same host-call budget as the one-copy case.
Shared unit tests cover multiple requested seats among 600 distant copies,
overlapping candidates, bounds tolerance and failed bounds inspection.

Run `SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/nut_references_test.py` after building the packages. It checks a standalone point and a circle, moves the point, changes the source plane's offset and orientation, edits pocket depth, and reopens the design. Pocket centers, bolt-hole axes, floor depths and exact cut volumes must agree, and the source sketch keeps its existing profiles. The same edits check pocketed nuts with cosmetic and modeled threads: rigid seat joints and the actual nut bodies must follow the floor, axis and hex rotation. Separate runs reuse a modeled nut definition on an initially tilted plane without displacing the first nut. A horizontal layout covers the previously failing move from -14 to -22 mm, larger horizontal and vertical moves across the origin, and pocket-width and depth edits followed by reopening.

### Screw seat references

The copied-tapped-seat regression preserves the existing unsupported-case refusal
and checks full rollback, so shared face inspection cannot attach to another copy.
A thin-plate countersink regression keeps its cone/bore junction attached even
when the screw's origin lies just below the plate's bounds.

Run `SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/screw_references_test.py` after building. It checks clearance, counterbored, countersunk and tapped holes with cosmetic and modeled threads. Point and circle moves, plane changes, parameter-driven recess depths, reused definitions, carrier movement and save/reopen must preserve each seat and the actual screw body's axial/radial mass-center offsets and volume. Tapped cases also cover layouts created before the solid, sketches attached to faces, and insertion from outside the sketch's component. `crates/fasteners/src/seat.rs` shares face inspection and rigid-joint creation with the nut plugin; each plugin resolves its own seating geometry before adding hardware. Tapped screws use construction axes through the source points, normal to each sketch's supporting plane or attached face. The axes are inserted immediately after their source sketches, in the same component, with the original history marker and active component restored afterwards. The solid need not exist at that history position, and later thread cuts cannot split the support face. Clearance and recessed screws use named round ends so their seats follow recess edits.

## Examples

The [gallery](examples/) links to each example folder. The folder contains a
README, native screenshots, `<name>.json` (open in SolidCraft) and
`<name>.steps.json` (replay with the test runner). Opening the saved design
uses its existing geometry. Replaying it requires the built plugins.

A recipe has captioned SolidCraft commands and an `expect` object with its
measurements. Use `dist/<id>` and `@approval` in plugin steps. The test runner
loads the built package and supplies its approval at replay time. It checks the
finished geometry and compares the saved design and catalog records against
the fresh build. Example-specific checks include seat joints, wheel contact,
interference and the bill of materials.

The same runner exercises the [private regression recipes](tests/fixtures/examples/).
They cover the screw heads, drives and tips, each stock rail profile, all five
assembled wheel variants, the standalone nut and insert, both slot-nut styles,
and the full modeled-thread rail frame. Each saves and reopens a temporary design.
The all-package coexistence and shared bearing checks live in
[the parts-library suite](tests/parts_library_test.py).

```sh
export SOLIDCRAFT_CLI=/path/to/solidcraft-cli
./build.sh
python3 tests/examples_test.py
python3 tests/examples_test.py --fixtures
python3 tests/examples_test.py --update examples/v-slot-gantry/v-slot-gantry.steps.json
python3 tests/examples_test.py examples/v-slot-gantry/v-slot-gantry.steps.json
```

After changing geometry, run `--update` on the affected recipe, rerun it and
render its screenshots. A recipe that fails its measurements leaves the saved
design intact. Gallery paths use `examples/<name>/<name>.json`. Scripts and links
that used the old top-level files need the new paths.

### Render the screenshots

After building the packages with `./build.sh`, regenerate every gallery screenshot
with an unsigned SolidCraft CLI. Run this from the repository root.

```sh
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/render_examples.py
```

The renderer uses the CPU backend for repeatable captures with the same CLI
version. It reads the saved example designs, applies the mounting plate's
initial parameters and nut centers for the before shot, and builds temporary
models from the private regression recipes for the catalog portraits. It stages
all captures before updating the checked-in PNGs.

Camera, resolution and section settings live in the
[render script](tests/render_examples.py). Add new gallery screenshots there.
The package build copies the images linked by plugin guides into each ZIP and
points example links to the GitHub gallery.

## Geometry tests

`tests/geometry_test.py` checks what the plugins cut. It builds a plate, runs a plugin on two sketch points, and compares the plate's volume with the figure worked out by hand: insert holes, screw clearance, counterbored, countersunk and tapped holes, nut pockets, and the head shapes and drive recesses of loose screws. A rail is checked the other way round: its volume must be the area of its section, worked out by hand from the slot sizes, times its length. A tapped or drilled rail is checked against the plain one, a screw hole by the metal it takes out, and a drilled rail, a tapped post and two screws are put together to check that each screw passes the first rail without touching it. A slot nut is put in the slot of a rail: it must be the size MISUMI draws, sit under the lips, touch nothing, and leave the rail and any nut built before it as they were.

```
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/geometry_test.py
```

It needs a `solidcraft-cli` that can run a plugin, so on the signed macOS build it reports SKIP.

## Wheel and bearing source geometry

The `bearings` and `wheels` crates own radial-bearing and wheel supplier products
and their exterior geometry. The `bearings::csk` module separately owns editable
CSK design specifications. The [OpenBuilds ledger](docs/sources/openbuilds.md) records wheel profiles,
assembly references and the 105ZZ width correction. The [MISUMI ledger](docs/sources/misumi.md)
records rounded ring profiles, closures and the open B626's visible balls. Both
retain source hashes and independent measurements. No source STEP files or
Downloads paths are runtime dependencies. Radial profiles and closure surfaces use
`plugin-kit::revolve`; screws and bearings share its arc/line section drawing code.

Build and check the native reference geometry with:

```bash
cargo build -p wheels --example reference_geometry
cargo build -p plugin-kit --example inspect_content
SOLIDCRAFT_CLI=/path/to/unsigned/solidcraft-cli python3 tests/wheel_geometry_test.py
SOLIDCRAFT_CLI=/path/to/unsigned/solidcraft-cli python3 tests/bearing_geometry_test.py
SOLIDCRAFT_CLI=/path/to/unsigned/solidcraft-cli python3 tests/csk_geometry_test.py
```

The test compares wheels to independently imported STEP measurements, verifies
bearing rings and recessed closure surfaces, checks wheel/rail contact against the
built Extrusions plugin, and checks saved geometry through the current-content
inspector. Run `./build.sh` first for the rail check. A zero-thickness closure is an intentional exterior
representation, not a seal cross-section or a mass model. The assembly and cross-plugin reuse checks live in `tests/parts_library_test.py`.
The bearing suite separately verifies MISUMI ring volumes against the STEP ledgers,
bore/shoulder radii, closure recesses and colours, the open bearing's race depths,
eight spheres, and every closed product's complete section vertices and rounded
edges, plus purchase/reuse behavior through the installed workflow. The
[coverage tracker](docs/bearing-coverage.md) links each added product to its
source, measurements and validation record.

The CSK suite checks all 55 presets, measured shoulders/chamfers, closure areas
and appearance, custom dimensions and keyway presence, exterior estimation
metadata, BOM quantities, definition
reuse, invalid-input rollback and save/reopen. It uses the built Bearings package;
run `./build.sh` first. The [CSK walkthrough](examples/bearings-and-clutches/#csk-references-and-overrides) explains the
shared keyway override and the display layout. Rebuild and check it
with:

```bash
python3 tests/examples_test.py --update examples/bearings-and-clutches/bearings-and-clutches.steps.json
python3 tests/examples_test.py examples/bearings-and-clutches/bearings-and-clutches.steps.json
```

## Components

All fixed parts use [one shared library](docs/parts-library.md) for canonical identity,
current-content inspection, reuse and placement. Component names are presentation;
purchasing identity and geometry representation are separate. Modified definitions
are preserved, and later insertions create clean replacements. The former creator
and command-hash implementation has been removed.

The reply's `component` identifies the placed definition, `reused` and occurrence
IDs. Its `library` report includes host-call count and modified candidates preserved.
Screws with holes and nuts with pockets retain their native seat joints. Loose
parts remain free to move. The original active component is restored after a run.

## Releases

Pushing a tag like `v0.1.0` runs `.github/workflows/release.yml`, which builds the plugins and attaches `<id>-0.1.0.zip` for every plugin under `plugins/`, such as `techsquid-screws-0.1.0.zip`, each with a `.sha256`. The tag has to match `workspace.package.version` and every `plugins/*/plugin.json`.

## License

MIT, see [LICENSE](LICENSE).
