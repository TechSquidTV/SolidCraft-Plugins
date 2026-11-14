# TechSquid SolidCraft Plugins

TechSquid's plugins for [SolidCraft](https://github.com/tangency-labs/solidcraft). Each one is a WebAssembly component (`craft:plugin@0.1.0`, capability `commands@0.1`), the same package shape as [solidcraft-gears](https://github.com/tangency-labs/solidcraft-gears). A plugin changes the model only by running SolidCraft commands.

## Plugins

| Plugin | Id | Family | What it builds |
|---|---|---|---|
| [Screws](plugins/techsquid-screws/README.md) | `techsquid-screws` | Fasteners | Metric machine screws to ISO standards on selected sketch points (socket, button, countersunk, hex, pan and cheese heads, set screws and threaded rod), with any drive, and optionally the clearance, counterbored, countersunk or tapped hole |
| [Heat-set inserts](plugins/techsquid-inserts/README.md) | `techsquid-inserts` | Fasteners | CNC Kitchen heat-set inserts, metric and UN: the straight or stepped hole drilled into the part, and the knurled insert as its own body |
| [Hexagon nuts](plugins/techsquid-nuts/README.md) | `techsquid-nuts` | Fasteners | ISO 4032 hexagon nuts on selected sketch points, and optionally a hexagonal pocket with a bolt hole |
| [Aluminium extrusions](plugins/techsquid-extrusions/README.md) | `techsquid-extrusions` | Extrusions | 20 mm series T-slot (MISUMI 5 series) and V-slot (OpenBuilds) rails, 2020 to 2080 and 4040, cut to any length, with tapped ends and screw holes so they screw together, and the MISUMI M5 slot nuts that go in them |

Each plugin's README covers installing it, its dialog fields, the standards it follows and the names it gives bodies and features. Each release has one zip per plugin. To install a plugin, unzip its folder into SolidCraft's plugins folder.

## Fasteners

Threads themselves are a host command. `solid.thread` cuts or annotates a cylindrical face from a designation such as `M6` or `M8x1`. The screw plugin places the screw and passes that designation through. The insert and nut plugins use `solid.hole` for the bore. Each plugin does the fastener geometry, and the host owns the thread form.

The three fastener plugins share field names (`size`, `custom_size`, `create`, `sketch`, `sketch_point`, `thread_modeled`) and a naming scheme for bodies: the kind of part, then its size, as in `Screw M3 x 12 socket`.

## Extrusions

A rail is one sketch of lines and arcs, extruded once. Its section comes from `crates/extrusions`, which builds it from a slot shape and a grid of 20 mm cells. The T-slot is checked against MISUMI's STEP models and the V-slot against OpenBuilds' DXF files; the plugin's README lists each source and its test.

A rail can be built with its ends tapped M5 and with screw holes across it, so rails screw together with no bracket: see the plugin's README.

A slot nut is one section too, extruded along the slot and tapped. `make-t-nut`, the plugin's second entry, hangs it under the lips of a slot, on a point on the rail's face. Its shape is MISUMI's STEP model of the pre-assembly nut HNTT5-5 or the post-assembly nut HNTP5-5.

## Layout

```
Cargo.toml                 workspace
crates/plugin-kit/         what every plugin shares: the host runner and wasm guest, dialog fields, placement, components, and names
crates/fasteners/          designations, metric sizes, standard heads, ISO 4032 nuts, ISO 273 clearance holes, drives, and the hex profile
crates/extrusions/         the 20 mm T-slot and V-slot shapes, the section of a rail built from them, its taps and screw holes, and the slot nuts
wit/plugin.wit             copy of SolidCraft's craft:plugin@0.1.0
plugins/techsquid-screws   screw entry and its parameters
plugins/techsquid-inserts  heat-set insert entry, catalog, and parameters
plugins/techsquid-nuts     ISO 4032 hexagon nut entry and its parameters
plugins/techsquid-extrusions  T-slot and V-slot rail entry, slot nut entry, and their parameters
build.sh                   the packages land in dist/<id>/, one per plugin
examples/                  designs built with the plugins: <name>.json opens in SolidCraft, <name>.steps.json builds it
tests/cli_test.py          runs the built package through solidcraft-cli
tests/geometry_test.py     checks by volume what each plugin cuts and builds
tests/examples_test.py     builds every example through solidcraft-cli and measures it
```

A plugin id is the folder name under `plugins/` and the folder you install. Preferences matches those. Each plugin owns its entry and the fields a person sets. Every plugin id starts with `techsquid-`, because SolidCraft plugin ids share one namespace. Code any plugin can use lives in `crates/plugin-kit`; fastener geometry lives in `crates/fasteners` and rail sections in `crates/extrusions`, and a new family of plugins gets its own crate beside them. The insert catalog stays in the insert plugin.

## Build

You need Rust with the `wasm32-unknown-unknown` target and [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools).

```
cargo test
./build.sh
```

`cargo test` checks the parameter rules and the commands each run sends. `./build.sh` wraps each core module as a component and leaves one `dist/<id>/` for every folder under `plugins/`.

## Test against the host

`tests/cli_test.py` needs the package from `./build.sh` and a `solidcraft-cli` with the plugin host (SolidCraft 0.41 or newer). The tests are run on a development build of SolidCraft 0.41.0.

```
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/cli_test.py
```

It inspects each package. A screw run must build an M3, an insert run must build the M3 × 5.7 insert, a nut run must build an ISO 4032 M6, an extrusion run must build a 120 mm V-slot 2040, and a bad `head` must be refused as invalid input. The signed macOS build of SolidCraft 0.41.0 has the hardened runtime and no JIT entitlement, so loading a component kills `solidcraft-cli` (`SIGKILL`, code signature invalid). Inspect still succeeds there, and the test skips a run check when the CLI dies that way.

## Examples

![A PCB enclosure with a heat-set insert and a screw in each corner, the lid shown transparent](examples/pcb-enclosure.png)

![The same enclosure closed: each screw head sits in a recess, flush with the top of the lid](examples/pcb-enclosure-closed.png)

A PCB enclosure. The base is a shelled block with a boss in each corner, filleted where it meets the walls and the floor. One `make-insert` run drills the four bosses and builds four M3 × 5.7 inserts on the points of a sketch on the rim. One `make-screw` run on the points of a sketch on the 5 mm lid drills M3 clearance holes, each with a recess for the screw head, and sets an M3 × 6 socket head screw into each recess, its head flush with the lid. The screws and inserts have modeled threads.

![A frame of T-slot rails on V-slot posts, screwed together](examples/rail-frame.png)

![The frame cut through one side: screws go through the side rails into the ends of a cross rail, and up through the cross rail into the feet of two posts](examples/rail-frame-joint.png)

![A slot nut under the lips of a top rail, its tapped hole showing through the slot](examples/rail-frame-nuts.png)

A 300 mm square frame, 340 mm tall, held together by screws alone, the way a Creality Ender 5 frame is. Five `make-extrusion` runs place twelve rails, already cut for their joints: cross rails with both ends tapped and four screw holes for the posts, side rails with a screw hole at each end, and four V-slot 2040 posts with both ends tapped, standing in the corners. Four `make-screw` runs place 24 M5 socket head screws on the slot floors. Sixteen M5 x 25 go through the cross rails and 11 mm into the ends of the posts. Eight M5 x 20 go through the side rails and 6 mm into the ends of the cross rails, short enough to stop before the post screws that cross those ends. Two `make-t-nut` runs put four M5 post-assembly nuts in the top slots and two M5 pre-assembly nuts in a slot of two posts. Every thread is cut. The only bodies that overlap are the screws and the tapped bores they are in, by the little that two threads cut out of step do.

Each example comes as two files:

- `examples/pcb-enclosure.json` is the design. Open it in SolidCraft (File > Open). It is saved by SolidCraft 0.42, so an older SolidCraft may refuse it as a newer file. SolidCraft rebuilds it as it opens, which takes a few seconds for the modeled threads. The plugins have already run, so opening it needs none of them installed.
- `examples/pcb-enclosure.steps.json` builds that design, one SolidCraft command per step, with a `caption` saying what each step does. It is not a design file, so SolidCraft does not open it. A plugin step is a `plugin.run` with `"path": "dist/<id>"` and `"approval": "@approval"`. The approval changes with every build, so the test inspects each package and fills it in. `expect` holds what the finished design must measure.

```
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/examples_test.py
```

It builds every `*.steps.json` and checks the body count (each placement of a fastener or a rail is a body, measured where it is placed) and the bounding boxes. On the enclosure it checks the insert hole, the clearance hole and the head recess on each corner, and one insert and one screw on each corner. On the frame it checks the box and the volume of every rail, screw and slot nut, and that the only bodies that overlap are the 24 screws and the tapped bores they are in. For both it checks that the saved design reopens, and that the design next to it matches the fresh build body for body. After a plugin or an example changes, `--update` writes that design from the fresh build. The same signed macOS build skips here as well, because every example runs a plugin.

The pictures are rendered from the design:

```
solidcraft-cli snapshot --in examples/pcb-enclosure.json --out examples/pcb-enclosure.png --width 1280 --height 800 --view iso --no-grid --no-axes --body Lid:opacity=0.3 --quality fine
solidcraft-cli snapshot --in examples/pcb-enclosure.json --out examples/pcb-enclosure-closed.png --width 1280 --height 800 --view iso --no-grid --no-axes --quality fine
solidcraft-cli snapshot --in examples/rail-frame.json --out examples/rail-frame.png --width 1280 --height 800 --camera '{"target": [0, 0, 170], "yaw": 0.5, "pitch": 0.4, "distance": 800}' --no-grid --no-axes --quality fine
solidcraft-cli snapshot --in examples/rail-frame.json --out examples/rail-frame-joint.png --width 1280 --height 800 --camera '{"target": [140, 105, 30], "yaw": 0.3, "pitch": 0.15, "distance": 190}' --no-grid --no-axes --quality fine --section x=140,flip
solidcraft-cli snapshot --in examples/rail-frame.json --out examples/rail-frame-nuts.png --width 1280 --height 800 --camera '{"target": [62, -140, 336], "yaw": 0.5, "pitch": 0.7, "distance": 75}' --no-grid --no-axes --quality fine
```

`tests/geometry_test.py` checks what the plugins cut. It builds a plate, runs a plugin on two sketch points, and compares the plate's volume with the figure worked out by hand: insert holes, screw clearance, counterbored, countersunk and tapped holes, nut pockets, and the head shapes and drive recesses of loose screws. A rail is checked the other way round: its volume must be the area of its section, worked out by hand from the slot sizes, times its length. A tapped or drilled rail is checked against the plain one, a screw hole by the metal it takes out, and a drilled rail, a tapped post and two screws are put together to check that each screw passes the first rail without touching it. A slot nut is put in the slot of a rail: it must be the size MISUMI draws, sit under the lips, touch nothing, and leave the rail and any nut built before it as they were.

```
SOLIDCRAFT_CLI=/path/to/solidcraft-cli python3 tests/geometry_test.py
```

It needs a `solidcraft-cli` that can run a plugin, so on the signed macOS build it reports SKIP.

## Components

Each fastener and each rail is a component of its own. This section describes fasteners; a rail is placed the same way, in an `Extrusions` folder instead of `Fasteners`. The holes, pockets and other cuts stay in the part they cut, as features of the component that was active, so the part keeps them when the hardware is hidden or deleted.

- **One definition, many placements.** A run builds the fastener once, in a component of its own, and places it on every point. Four M3 x 12 socket screws are one component, `Screw M3 x 12 socket`, placed four times, so the thread is modeled once. Inside its component the fastener stands with its outer side up +Z; each placement turns that onto the sketch. A part with no cut thread stands on the component's origin. A part with one (a screw, a nut, an insert, a slot nut, a rail with a tapped end) stands on a point of its own away from it, which no other part shares. SolidCraft 0.41 puts a modeled thread on any bore or shank of the same size on the same axis, such as the bore of a rail standing on the origin, a second length of the same screw, or a thicker nut of the same thread.
- **Reused across runs.** A later run places the existing definition again when it would build exactly the same body. The plugin keeps a hash of the commands that build it under `dev.threadcraft` in the component's metadata, with the plugin's id. A different build, such as a cosmetic thread where the first one is modeled, gets a definition of its own.
- **A Fasteners folder.** Every placement goes in one `Fasteners` component in the design, made the first time it is needed. It is a phantom, so the bill of materials counts what it holds and has no row of its own.
- **A bill of materials.** Each definition is `purchased`, its part number is its name and its description the standard it follows (`ISO 4762`, `ISO 4032`) or the insert it is (`CNC Kitchen heat-set insert M3 × 5.7`). `assembly.bom` on the example lists 4 inserts and 4 screws.
- **The active component stays.** Fasteners are placed in the design whichever component is active, and the component that was active before the run is active again afterwards.
- **Moving them.** A placement is free, not grounded, and can be moved, hidden or given joints on its own.

The reply's `component` says which definition was placed (`component`), whether it was `reused`, and the `occurrences` made.

## Releases

Pushing a tag like `v0.1.0` runs `.github/workflows/release.yml`, which builds the plugins and attaches `<id>-0.1.0.zip` for every plugin under `plugins/`, such as `techsquid-screws-0.1.0.zip`, each with a `.sha256`. The tag has to match `workspace.package.version` and every `plugins/*/plugin.json`.

## License

MIT, see [LICENSE](LICENSE).
