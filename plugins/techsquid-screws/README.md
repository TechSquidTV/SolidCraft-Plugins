# Screws

Add the right screw to your [SolidCraft](https://github.com/tangency-labs/solidcraft) design in a few clicks—from socket heads and countersunk screws to set screws and threaded rod. Create matching holes at the same time, ready for your assembly.

![A selection of screw heads, drives and tips](screws.png)

## What this adds to SolidCraft's built-in fasteners

Use this plugin when you need detailed screw geometry, custom dimensions, or screws and matching holes created together. The comparison below covers the options exposed by [SolidCraft 0.42's Insert Fastener tool](https://github.com/tangency-labs/solidcraft/blob/0930970ce03b43f8d9e6477cb0903d4a713f17f4/crates/engine/src/cmd/parts.rs); its generated parts can also be edited afterwards.

| Feature | Built-in Insert Fastener | Screws plugin |
|---|---|---|
| Screw and hole together | Inserts hardware into existing holes or onto faces; hole creation is a separate operation. | Creates clearance, counterbored, countersunk or tapped holes and seats the screws in one run. **Hole only** makes the cuts without hardware. |
| Modeled threads | Cosmetic threads on metric screws; no modeled-thread option in the insertion dialog. | Choose actual thread geometry or cosmetic threads for faster rebuilds. |
| Head and part styles | Socket, button, countersunk socket, hex and flat-point set screws. | Adds pan and cheese heads, threaded rod, and additional countersunk variants for Phillips and slotted drives. |
| Drive options | The drive is fixed by the selected screw family. | Choose hex socket, Phillips, slotted, Torx or no drive, with standard sizing and optional size/depth overrides. |
| Custom sizes and heads | Choose a catalog size and its predefined head dimensions. | Enter metric sizes and fine pitches such as `M8x1`, plus custom head diameter and height where needed. |
| Tip options | The tip is fixed by the selected family. | Choose flat, gimlet point or cup; set screws include the ISO 4026 flat-point and ISO 4029 cup-point geometry where sizes are listed. |
| Threaded length | Uses the selected family's thread-length rules. | Uses standard thread lengths where available, with an explicit override for partial or full threading. |
| Reuse across runs | One component is shared across the placements in an insertion. A later insertion creates another component. | Reuses an identical screw definition across later runs, while giving each attached screw its own joint. |

Both tools produce editable timeline features and reusable components, and both can keep screws seated with native rigid joints. The plugin adds the combined screw-and-hole workflow described below.

For placing standard hardware in existing holes, the built-in tool offers a live preview, direct hole picking, and **Change Part Size**. It also includes inch and GOST catalogs, nuts, washers, pins and bearings. This plugin focuses on configurable screws: select sketch points or circles before opening its form, or enter their names. It has no live preview or point picker in the form, and its generated components do not use the built-in **Change Part Size** workflow.

## Get started

Select one or more sketch points or circles and open **Plugins > Screw**. Choose a size, length and head, then press **Run**. Leave **Sketch** and **Sketch point** blank to use your selection; with no selection, a loose screw is placed at the origin.

The default is an M3 × 12 socket-head screw. Choose **Screw and hole** to drill the part at your selected points, or **Hole only** to create just the holes. Countersunk screws, set screws and threaded rod use overall length; other headed screws are measured under the head.

**Screw and hole** attaches each screw to its hole with a rigid joint. Moving the source point moves the screw, and changing a counterbore's depth moves its head with the floor. The attachment is saved in the design and works without the plugin installed. **Screw only** creates loose hardware.

**Breaking change:** new screw-and-hole placements are attached. Regenerate previously created placements to add these joints; upgrading the plugin does not attach existing screws. A missing or ambiguous seat is refused.

Try the [attachment test plate](https://github.com/TechSquidTV/SolidCraft-Plugins/blob/main/examples/screw-attachment-test.md): move counterbored and tapped hole pairs, change the recess depth, then save and reopen to check the seats.

For all options, supported standards, dimensions and placement details, see the [screw reference](REFERENCE.md).

## License

[MIT](../../LICENSE)
