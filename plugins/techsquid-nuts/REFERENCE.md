# Hexagon nuts

An ISO 4032 hexagon nut for [SolidCraft](https://github.com/tangency-labs/solidcraft). One entry, `make-nut`. It builds the nut as a component of its own, once, and places it on every point (see [Components](../../DEVELOPMENT.md#components)). The bore is a tapped hole.

![An ISO 4032 M3 hexagon nut, chamfered on both ends, with its bore countersunk and tapped](nut.png)

An M3 nut, the size the form opens on.

The sizes below are the ISO 4032 table, preferred and second-choice. ISO 4032 replaced DIN 934, and most sizes match, but not all: a DIN 934 nut is wider across the flats at M10 (17 mm, not 16), M12 (19, not 18), M14 (22, not 21) and narrower at M22 (32, not 34), and it is a little lower at most sizes (M6 5.0 mm, M8 6.5 mm). For a DIN 934 nut at those sizes, type its width and height in Custom width and Custom thickness, so the pocket fits it. The width across the flats is the nominal. The height is the standard maximum. Both ends are chamfered.

## Use

All sizes are millimetres. The form opens on an M3 nut, the size the screw form opens on. Size lists the common sizes, M1.6 to M24, with the coarse thread. For any other metric size, such as `M8x1` or `M3.5`, type it in Custom size, which replaces Size.

Select a sketch point, or a circle, before you open the dialog. Leave Sketch and Sketch point empty. The bearing face sits on that point and the nut stands on the outer side of the sketch. On a face sketch the normal already points out of the material, so the nut stands off the face. With nothing selected, the nut sits on the origin: the bearing face is on the XY plane and the nut runs along +Z. The reply says which of these happened in `placed`. Select several points or circles and each one gets a nut, as one undo step; `count` and `points` in the reply list them. By name, Sketch point takes several ids with commas, such as `c1, c2`.

Name Sketch and Sketch point together to place the nut without a selection. Sketch point is a point id such as `c1.center`, or a circle id such as `c1`.

Create decides whether the part gets a pocket. `Nut only`, the default, builds the nut and leaves the part alone. `Nut and pocket` first cuts a hexagonal pocket in the part under each selected point, as deep as the nut is tall, with an ISO 273 medium clearance hole for the bolt through the rest of that part, and then sets the nut in the pocket so it ends flush. `Pocket only` cuts without a nut. A pocket needs selected points on a part. With nothing selected, `Nut and pocket` puts a loose nut on the origin and its reply's `note` says no pocket was cut, and `Pocket only` asks you to select the points. Pocket adjustment is the gap on each flat: printed pockets come out small, so raise it until the nut drops in.

Leave the custom width and thickness empty to use the table. Fill one and the other stays the standard. A size with no row here, such as `M7x1`, needs both. A pocket also needs an ISO 273 clearance hole for the size: every ISO 273 size from M1 to M64 works, including M3.5, M7 and M18, and a size ISO 273 does not list is refused before anything is cut. A fine pitch such as `M8x1` keeps the M8 nut and taps that pitch. `M3.5` is in the table and takes its ISO 261 coarse pitch, 0.6.

The chamfer on each end is 30° from the face, the steep side of the drawing's 15° to 30°. It runs out to the across-flats circle, and it stops at `c` max when that would be a taller chamfer than the standard allows. The bore is countersunk on both ends, about 120° included, out to `da` max. A listed size keeps that countersink, so a width you type has to stay wider than `da` and a height has to stay taller than the two countersinks. A size you type in yourself has the chamfer and no countersink.

| Parameter | Default | Meaning |
|---|---|---|
| `size` | `M3` | Shown as Size. A common size, `M1.6` to `M24` |
| `create` | `Nut only` | `Nut only`, `Nut and pocket` or `Pocket only`. A pocket needs selected points on a part; with none, `Nut and pocket` makes the nut alone |
| `pocket_adjust` | 0.2 | Gap between each flat of the nut and its pocket, mm, 0 to 1 |
| `custom_size` | empty | Shown as Custom size. Any metric size, such as `M8x1` or `M3.5`. Replaces Size when filled |
| `custom_thickness` | standard | Nut height, mm |
| `custom_width` | standard | Width across the flats, mm |
| `sketch` | empty | Sketch that holds the point. Empty, with Sketch point empty, uses the selection or the origin |
| `sketch_point` | empty | A point or circle in that sketch, such as `c1` or `c1.center` |
| `thread_modeled` | true | Cut the internal thread. False leaves a cosmetic thread, which rebuilds faster |

The package calls `selection.get`, `inspect.measure` (for the body names, so a new body gets a free one), `sketch.inspect`, `sketch.create`, `sketch.polygon.circumscribed`, `sketch.dimension`, `sketch.constraint.horizontal_vertical`, `sketch.line`, `sketch.finish`, `solid.extrude`, `solid.revolve`, `solid.hole`, `solid.move` (the nut is cut on the origin and moved aside before it is tapped), `solid.cone`, and `component.list`, `component.create`, `component.metadata`, `component.metadata.set`, `component.activate`, `occurrence.copy`, `occurrence.move` and `occurrence.reparent` to place it. The hexagon is the same profile a hex-head screw uses.

## Sizes

| Size | Pitch | Across flats | Height |
|---|---:|---:|---:|
| M1.6 | 0.35 | 3.2 | 1.3 |
| M2 | 0.4 | 4 | 1.6 |
| M2.5 | 0.45 | 5 | 2 |
| M3 | 0.5 | 5.5 | 2.4 |
| M3.5 | 0.6 | 6 | 2.8 |
| M4 | 0.7 | 7 | 3.2 |
| M5 | 0.8 | 8 | 4.7 |
| M6 | 1 | 10 | 5.2 |
| M8 | 1.25 | 13 | 6.8 |
| M10 | 1.5 | 16 | 8.4 |
| M12 | 1.75 | 18 | 10.8 |
| M14 | 2 | 21 | 12.8 |
| M16 | 2 | 24 | 14.8 |
| M18 | 2.5 | 27 | 15.8 |
| M20 | 2.5 | 30 | 18 |
| M22 | 2.5 | 34 | 19.4 |
| M24 | 3 | 36 | 21.5 |
| M27 | 3 | 41 | 23.8 |
| M30 | 3.5 | 46 | 25.6 |
| M33 | 3.5 | 50 | 28.7 |
| M36 | 4 | 55 | 31 |
| M39 | 4 | 60 | 33.4 |
| M42 | 4.5 | 65 | 34 |
| M45 | 4.5 | 70 | 36 |
| M48 | 5 | 75 | 38 |
| M52 | 5 | 80 | 42 |
| M56 | 5.5 | 85 | 45 |
| M60 | 5.5 | 90 | 48 |
| M64 | 6 | 95 | 51 |

## Standards

The nut and its pocket follow these standards.

| Fastener | Dialog choice | Standard | Sizes | From the standard | Source | Test |
|---|---|---|---|---|---|---|
| Hexagon nut | Hex nut | ISO 4032 (replaces DIN 934, which is wider at M10, M12, M14) | M1.6 to M64 | width, height, chamfer, countersink | [fasteners.eu ISO 4032](https://www.fasteners.eu/standards/ISO/4032/) | yes |
| Clearance hole | Nut pocket bolt hole | ISO 273, medium | M1 to M64 | hole diameter | ISO 273:1979, Table 1 | yes |

"Test: yes" means a unit test repeats the table from the source named, so the code and the source cannot drift apart unnoticed. Each table in `crates/fasteners/src/` names its source in a comment.

## Names

The body, the hole and every feature say what the part is and its size, so a design with several fasteners stays readable. Sizes use a plain `x`, with spaces around it so a fine pitch stays apart from the length. A second body with the same name gets a number in brackets, such as `(2)`.

| What | Name |
|---|---|
| Nut body | `Nut M3`, `Nut M8x1` |
| Nut pocket | `Nut pocket M3` and `Bolt hole M3` |
| Pocket folder | `Nut pockets M3`, holding one run's pockets, their sketches and the bolt holes, folded |
| Features and sketches of a nut | the nut's name, then the feature: `Nut M3: Thread` |
| Timeline folder | the nut's name, `Nut M3`, holding every sketch and feature of the nut, folded |

A pocket keeps its kind and size in its own name because it stays in the part, and with Pocket only there is no nut body at all.

## Older field names

The plugins share one set of field names. Scripts written for the earlier names still work: `designation` for `size`, `custom_designation` for `custom_size`, and `custom_height` for `custom_thickness`.

## License

MIT, see [LICENSE](../../LICENSE).
