# Threadcraft Screws

A screw generator for [SolidCraft](https://github.com/tangency-labs/solidcraft). One entry, `make-screw`. It builds a head and a shank with ordinary SolidCraft commands, then calls `solid.thread` on the shank. The result is a normal timeline.

## Install

You need a SolidCraft build that has the plugin host (the `commands@0.1` capability). SolidCraft 0.41 has it.

1. Download `threadcraft-screws-<version>.zip` from the latest release.
2. Unzip it into SolidCraft's plugins folder, so you end up with a `threadcraft-screws` folder holding `plugin.json` and `plugin.wasm`:

| OS | Plugins folder |
|---|---|
| Windows | `%LOCALAPPDATA%\SolidCraft\plugins\` |
| macOS | `~/Library/Application Support/SolidCraft/plugins/` |
| Linux | `$XDG_DATA_HOME/solidcraft/plugins/` (usually `~/.local/share/solidcraft/plugins/`) |

3. Start SolidCraft. Preferences > Plugins lists it. Listing never runs any code. Each run needs your approval of the exact package contents.

## Use

All sizes are millimetres. The top of the shank sits on a sketch point, and the head stands on the outer side of that sketch. `length` is the length as the standard lists it: under the head, into the part. A countersunk screw is measured overall, head and all, as ISO 10642, ISO 2009 and ISO 7046-1 measure it, so an M3×12 countersunk screw is 12 mm from the top of the head to the tip. For a set screw or a threaded rod it is the overall length too.

Select a sketch point, or a circle, before you open the dialog. Leave Sketch and Sketch point empty. The dialog has no point picker, so those boxes stay blank and the selection is used when you press Run. The sketch normal is the outside of the surface. With nothing selected, the screw sits on the origin: the head stands above the XY plane and the shank runs along −Z. The reply says which of these happened in `placed`. Select several points or circles and each one gets a screw, as one undo step; `count` and `points` in the reply list them. By name, Sketch point takes several ids with commas, such as `c1, c2`.

Name Sketch and Sketch point together to place the screw without a selection. Sketch point is a point id such as `c1.center`, or a circle id such as `c1`.

The form opens on an M3 socket-head screw, 12 mm long, with its 2.5 mm hex socket. Drive is Standard, which follows the Head, so changing the Head alone gives a sensible screw. Size lists the common machine screw sizes, M1.6 to M20, with the coarse thread. For any other metric size, such as `M8x1` or `M3.5x0.6`, type it in Custom size, which replaces Size. Leave the custom head diameter and height empty to use the standard for that size and head. Fill one and the other stays the standard, except a countersunk head, which takes both or neither. A size with no standard head here needs both. On a hex head the diameter is the width across the flats.

Blank head sizes are filled from the standard of the head and drive: socket M1.6 to M64 (DIN 912 / ISO 4762); button M3 to M16 (ISO 7380-1); countersunk with a hex socket M3 to M20 (ISO 10642, a 90° cone to a sharp edge); countersunk with a slot or a Phillips recess M1.6 to M10 (ISO 2009 and ISO 7046-1, a 90° cone with a cylindrical land above it, M3 being 5.5 by 1.65 mm); hex M1.6 to M64 (ISO 4017); pan with a slot M1.6 to M10 (ISO 1580, M3 being 5.6 by 1.8 mm); pan with a Phillips recess M1.6 to M10 (ISO 7045, the same width and taller, M3 being 5.6 by 2.4 mm); cheese M1.6 to M10 (ISO 1207).

| Parameter | Default | Meaning |
|---|---|---|
| `size` | `M3` | Shown as Size. A common machine screw size, `M1.6` to `M20` |
| `custom_size` | empty | Shown as Custom size. Any metric size, such as `M8x1`. Replaces Size when filled |
| `length` | 12 | Length under the head, mm. Overall on a countersunk head, a set screw and a threaded rod |
| `head` | `Socket head` | `Socket head`, `Button head`, `Countersunk`, `Hex head`, `Pan head`, `Cheese head`, `Set screw` or `Threaded rod` |
| `drive` | `Standard` | `Standard`, `Hex socket`, `Phillips`, `Slotted`, `Torx` or `None`. Standard is the usual drive for the head: a hex socket in a socket, button or countersunk head and in a set screw, Phillips in a pan head and in a countersunk head below M3 (ISO 10642 starts at M3), a slot in a cheese head, none in a hex head or a threaded rod |
| `drive_size` | standard | Shown as Drive size. Sizes whichever drive is chosen. Hex socket: the key size, or the key size and the depth, such as `2.5` or `2.5x1.3`. Torx: `T10`. Phillips: `PH1` or `PH2x1.7`. Slotted: width and depth, such as `0.8x0.7`. Empty uses the standard for that screw |
| `tip` | `Flat` | `Flat`, `Point` or `Cup`. Point is a gimlet one diameter long, included in the length, and it is not itself threaded. Cup is a recess in the end |
| `create` | `Screw only` | `Screw only`, `Screw and hole` or `Hole only`. A hole needs selected points on a part. Hole only skips the drive, and the head unless the hole recesses it |
| `hole_type` | `Clearance` | `Clearance` (ISO 273 normal fit), `Clearance, head recessed` or `Tapped`. Used when Create drills a hole |
| `sketch` | empty | Sketch that holds the point. Empty, with Sketch point empty, uses the selection or the origin |
| `sketch_point` | empty | A point or circle in that sketch, such as `c1` or `c1.center` |
| `head_diameter` | standard | Shown as Custom head diameter, mm. On a hex head, the width across the flats |
| `head_height` | standard | Shown as Custom head height, mm. One of the two is enough, except on a countersunk head |
| `thread_length` | standard | Threaded length from the tip of the cylinder, mm. A point is not included. Empty threads the whole shank on a short screw and the standard length `b` on a long one. Anything up to the full length threads all of the cylinder: above a point, and between the chamfers of a set screw |
| `thread_modeled` | true | Cut the thread profile. False leaves a cosmetic thread, which rebuilds faster |

A set screw and a threaded rod have no head, so both head sizes stay empty.

Create decides whether the part gets a hole. `Screw only`, the default, builds the screw and leaves the part alone. `Screw and hole` first drills the part under each selected point, and `Hole only` drills without a screw. Both need selected points on a part; the hole is one ordinary `solid.hole` feature on those sketch points, so it follows the sketch and can be edited afterwards. Every hole is as deep as the screw reaches, so it goes through a part thinner than that.

- **Clearance.** The ISO 273 normal clearance hole for the size: 3.4 mm for an M3. The head sits on the surface. A size SolidCraft's own clearance table leaves out, such as M3.5 or M18, is drilled at its ISO 273 diameter instead.
- **Clearance, head recessed.** The same hole with a counterbore the head sits in, 0.5 mm wider than the head all round and as deep as the head is tall: 6.5 by 3 mm for an M3 socket head. A countersunk head gets a 90° countersink. The screw is set down into the recess, so the top of its head ends flush with the surface. A set screw and a rod have no head to recess.
- **Tapped.** A hole tapped for the screw's own thread, cosmetic or modeled as Modeled thread says.

The reply names the standard the screw follows in `standard`, such as `ISO 4762`, and lists `hole` and the bodies it `drilled`.

A standard head has its standard top. A button head is the ISO 7380-1 dome, leaning in from the bearing face to a flat. A Phillips pan head is the ISO 7045 shape, a cylindrical side under a spherical top, and a slotted pan head is the ISO 1580 one, flat with its edge rounded. A hex head is chamfered at 30° on top. A round head whose diameter or height you type is a plain cylinder. A typed countersunk head is a cone from the shank up to that diameter, with a land above it when it is taller than the 90° cone and a flatter cone when it is lower. A typed hex head keeps its top chamfer.

Thread length follows the standard too. Each standard draws a line in its length table: screws above it are threaded to the head, and screws below it have a plain shank and are threaded for `b` from the tip. Socket and button heads (`b` is `2d + 12`) keep a shank once they are longer than `b` plus eight and a half pitches, so an M3 socket head is threaded to the head up to 20 mm and for 18 mm from 25 mm on. A countersunk socket head (ISO 10642) starts later, at 30 mm for M3 and 50 mm for M8, counting the head. For the slotted and Phillips heads `b` is 25 mm up to M3 and 38 mm above: a Phillips pan head (ISO 7045) is threaded to the head up to 25 mm for M3 and smaller and up to 40 mm above; a slotted pan or cheese head (ISO 1580, ISO 1207) up to 30 mm and 40 mm; a flat countersunk head (ISO 2009, ISO 7046-1) up to 30 mm and 45 mm. A hex head, a set screw and a rod are threaded all the way. Type the full length in Thread length to thread a long screw to the head.

`tip` is the end of the shank. Flat is square. On a set screw it is the ISO 4026 flat point: the end is chamfered at 45° from the thread down to a flat `dp` across, 2 mm on an M3 and 4 mm on an M6, for the sizes that standard lists (M1.6 to M24). A hexagon socket set screw is also chamfered round its socket end, down to the root of the thread: at 90° included, or at 120° on the short lengths the standard shades in its table, such as M3 × 3. The 45° chamfers are chamfer features, so they can be edited or deleted afterwards; the 120° one is a revolved cut. Point is a self-tapping gimlet: a cone one major diameter long, and that cone is part of `length`, so an M3×12 point still reaches 12 mm. The thread starts where the cone meets the cylinder. Cup is a 120° recess in the end face, the usual set-screw tip, and it does not change the outside length. On a set screw it is the ISO 4029 cup point for the sizes that standard lists (M1.6 to M24): the cup is `dz` across at its mouth, 1.4 mm on an M3, and the outside of the point is chamfered at 45° down to that mouth, so the rim between them is sharp. The socket depth then follows the ISO 4029 length table, which keeps the shallower socket up to 4 mm on an M3. A cup on a headed screw, or on a size the standard does not list, is the plain recess with no chamfer. Any head can take any tip.

Torx cuts an ISO 10664 hexalobular recess into the outer face. The lobes are Table 1 `A` across the tips and `B` across the valleys, with the Annex A lobe radius `0.1 A`. The depth is the ISO 14579 minimum penetration for that socket, so an M3 socket head gets a T10 recess 1.01 mm deep. A screw size that table does not list needs a socket in Drive size, such as `T10`. A head too low for the standard depth gets a shallower recess that leaves 0.3 mm under it, down to 0.4 mm deep, and `torx_depth` in the reply gives the depth cut. A set screw takes the recess in its end, and the standard socket for a headed screw of that diameter is often too wide for the shank.

Hex socket cuts a hexagon into the outer face, and into the end of a set screw. The key size `s` and the least depth `t` come from the standard of the head: DIN 912 (ISO 4762) for a socket head, ISO 7380-1 for a button head, ISO 10642 for a countersunk head and ISO 4026 for a set screw. An M3 socket head takes a 2.5 mm key, 1.3 mm deep. ISO 4026 gives a set screw two depths: the shallower for the short lengths it shades in its table and the deeper for every longer one, so an M3 set screw is 1.2 mm deep at 3 mm long and 2 mm deep from 4 mm on. The floor is flat: the drill point under a real socket is not cut. A pan head and a hex head have no standard socket, so Hex socket on those needs a Drive size.

Slotted cuts a straight slot across the outer face, and into the end of a set screw. ISO 2380-1 sizes the screwdriver blade. The slot width is the nominal thickness from that standard, which the screw standard calls `n`. A pan head uses the ISO 1580 depth, so an M3 slot is 0.8 mm wide and 0.7 mm deep. A countersunk head uses the shallower ISO 2009 slot, 0.6 mm on an M3. A cheese head uses the deeper ISO 1207 slot, 0.85 mm on an M3. A set screw uses the smaller ISO 4766 slot, so an M3 set screw is 0.4 mm wide and 0.8 mm deep. The slot runs past both sides of the head. A size those tables do not list needs a width and depth in Drive size, such as `1.6x1.4`.

Phillips cuts an ISO 4757 type H cross recess. The wings are cut to the width the product standard gives across their tips at the face, `m`: 3 mm on an M3 pan head (ISO 7045) and 3.2 mm on an M3 countersunk head (ISO 7046-1). From there the wing tips narrow at 26°30′ to the screw axis down to the width `g`, where a 28° point closes the recess. Wing thickness is the nominal `b`, with straight walls. The side lean, the blunt point, the corner radius and the gauge are left out. The penetration in those standards is a gauge reading taken on the wings, so the recess ends deeper than it: an M3 pan head is gauged at 1.4 mm and its point ends 2.07 mm under the face. The reply gives all three as `phillips_depth`, `phillips_opening` and `phillips_reach`. In a low head the point runs on into the shank, which is allowed while the recess is narrower than the root of the thread there and is refused otherwise. A set screw and a cheese head have no Phillips table here, so they need a size such as `PH1x0.8`, and so does a size outside M1.6 to M10. A typed size such as `PH2x2.1` is taken as the penetration, and its width follows the same line the standards' rows lie on.

The package calls `selection.get`, `inspect.measure` (for the body names, so a new body gets a free one), `sketch.inspect`, `solid.cylinder`, `solid.cone`, `sketch.create`, `sketch.polygon.circumscribed`, `sketch.dimension`, `sketch.constraint.horizontal_vertical`, `sketch.arc.three_point`, `sketch.rectangle.center`, `sketch.line`, `sketch.finish`, `solid.extrude`, `solid.combine`, `solid.revolve`, `solid.chamfer`, `solid.thread` and `solid.hole`. A socket or cheese head is a cylinder. A button or pan head is its half profile, revolved. A countersunk head is a cone from the shank diameter up to the head diameter, with the land a flat head has above it. A hex head is a six-sided sketch, chamfered on top with a revolved cut. Set screw chamfers are `solid.chamfer` features. A hole is a `solid.hole` feature. A point is a cone joined to a shorter shank. A cup is a cone cut into the end. A Torx recess is twelve tangent arcs, cut from a sketch just outside the outer face. A slot is a rectangle on that same sketch plane. A Phillips recess is the wing-tip cone intersected with a straight plus, then cut from the screw.

## Sources

The head follows the drive, so a slotted or Phillips pan or countersunk screw is a different standard from a hex socket one. The table of every standard this plugin builds, with its sizes, its source and whether a test pins it, is under Supported standards in the [repository README](../../README.md#supported-standards).

## License

MIT, see [LICENSE](../../LICENSE).
