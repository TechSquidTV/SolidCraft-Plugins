//! Revolve a radial section on a placement's axis. Points are [radius, axial distance].

use serde_json::{json, Value};

use crate::{host::sketch_id, Placement, Runner};

/// A piece of a radial section: x is radius and y is axial distance.
#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Line(Vec<[f64; 2]>),
    Arc {
        start: [f64; 2],
        through: [f64; 2],
        end: [f64; 2],
    },
}

/// Draw a connected section in an existing sketch. Shared by screw crowns and
/// sourced bearing rings; geometry ownership remains in the domain crates.
pub fn draw(r: &mut impl Runner, sketch: &Value, segments: &[Segment]) -> Result<(), String> {
    for segment in segments {
        match segment {
            Segment::Line(points) => {
                r.run("sketch.line", json!({"sketch": sketch, "points": points}))?;
            }
            Segment::Arc {
                start,
                through,
                end,
            } => {
                r.run(
                    "sketch.arc.three_point",
                    json!({"sketch": sketch, "start": start, "through": through, "end": end}),
                )?;
            }
        }
    }
    Ok(())
}

/// A closed polygon makes a solid; a line makes an annular surface without an
/// invented thickness. The caller supplies the feature name and solid operation.
pub enum Section<'a> {
    Solid(&'a [[f64; 2]]),
    CurvedSolid(&'a [Segment]),
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
        Section::CurvedSolid(segments) => {
            draw(r, &sketch, segments)?;
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

/// Cut a triangle revolved about `place`'s axis from the end face `along` above its
/// point. `into` points from that face into the part: the triangle's x is the radius
/// and its y runs that way. A revolved cut is the same whichever way the part points,
/// where a chamfer with two unequal legs depends on which of its faces the host takes
/// first. Hexagon heads, nuts and set screws all chamfer their ends with it.
pub fn cut_end(
    r: &mut impl Runner,
    place: &Placement,
    body: &str,
    along: f64,
    into: [f64; 3],
    triangle: &[[f64; 2]; 3],
    name: &str,
) -> Result<(), String> {
    build(
        r,
        &Placement {
            point: place.along(along),
            normal: into,
            ..*place
        },
        &format!("{name} profile"),
        Section::Solid(triangle),
        json!({
            "operation": "cut",
            "targets": [body],
            "name": name,
        }),
    )?;
    Ok(())
}
