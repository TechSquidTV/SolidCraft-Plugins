//! Revolve a radial section on a placement's axis. Points are [radius, axial distance].

use serde_json::{json, Value};

use crate::{host::sketch_id, Placement, Runner};

/// A closed polygon makes a solid; a line makes an annular surface without an
/// invented thickness. The caller supplies the feature name and solid operation.
pub enum Section<'a> {
    Solid(&'a [[f64; 2]]),
    Surface(&'a [[f64; 2]; 2]),
}

pub fn build(
    r: &mut impl Runner,
    frame: &Placement,
    sketch_name: &str,
    section: Section<'_>,
    mut feature: Value,
) -> Result<Value, String> {
    let sketch = r.run(
        "sketch.create",
        json!({"name": sketch_name, "plane": {
            "origin": frame.point, "x_dir": frame.x_dir, "y_dir": frame.normal,
        }}),
    )?;
    let sketch = sketch_id(&sketch)?;
    let command = match section {
        Section::Solid(points) => {
            r.run(
                "sketch.line",
                json!({"sketch": sketch, "closed": true, "points": points}),
            )?;
            "solid.revolve"
        }
        Section::Surface(points) => {
            r.run(
                "sketch.line",
                json!({"sketch": sketch, "ids": ["section"], "points": points}),
            )?;
            feature["curves"] = json!(["section"]);
            "surface.revolve"
        }
    };
    r.run("sketch.finish", json!({}))?;
    feature["sketch"] = sketch;
    feature["axis"] = json!("y");
    feature["angle"] = json!(360);
    r.run(command, feature)
}
