# Screw attachment test plate

Open [the saved design](screw-attachment-test.json) in SolidCraft 0.42.0 or newer. The plugins are not needed to edit this saved model.

The 100 × 60 × 12 mm plate has two gold M4 × 6 socket-head screws in counterbores on the left and two teal M4 × 8 screws in tapped holes on the right. Threads are cosmetic so changes rebuild quickly. Four native rigid joints attach the screws. Both layout sketches were created before the plate to exercise layout-first tapped placement.

Open **Change Parameters** in the Model toolbar and try these edits:

| Parameter | Starting value | Try | Expected result |
|---|---:|---:|---|
| `counterbore_x` | 25 mm | 32 mm | Left holes and gold screws move farther left. |
| `counterbore_y` | 15 mm | 18 mm | Left pair spreads apart. |
| `tapped_x` | 25 mm | 18 mm | Right holes and teal screws move toward the center. |
| `tapped_y` | 15 mm | 20 mm | Right pair spreads apart. |
| `recess_depth` | 4 mm | 6 mm | Gold screws move down 2 mm with the counterbore floors. |
| `plate_thickness` | 12 mm | 14 mm | Plate top, holes and screw seats rise 2 mm. |

The position parameters drive the points in **Counterbore positions** and **Tapped positions**. Edit those parameters rather than dragging the constrained points. Hole depths follow the plate thickness. Keep the holes inside the plate and the recess between 3 and 6 mm for this example.

Save, close and reopen after editing: the screws should remain seated. The starting file has been restored to the values above. All four joint frames, actual screw bounding boxes, bore locations and the calculated plate volume were verified after the listed edits and after reopening.

The [build steps](screw-attachment-test.steps.json) reproduce the example. Run `python3 tests/examples_test.py examples/screw-attachment-test.steps.json` after building the plugins to check its geometry and saved design.
