//! A six-sided profile extruded along the sketch normal.
//!
//! Screw heads and loose nuts use a circumscribed polygon centered on the sketch
//! origin. Linked pockets locate each corner with a directed construction radius
//! so large edits cannot fold the hex. Both use one across-flats width dimension.

use serde_json::{json, Value};

use plugin_kit::host::{body_name_of, sketch_id};
use plugin_kit::Runner;

/// What to extrude. `operation` is `new` or `join`, the words `solid.extrude` takes.
pub struct Hex {
    pub origin: [f64; 3],
    pub x_dir: [f64; 3],
    pub y_dir: [f64; 3],
    pub across_flats: f64,
    pub height: f64,
    pub sketch_name: String,
    pub name: String,
    pub operation: &'static str,
    pub body_name: Option<String>,
    /// The body a join attaches to. A new body leaves this empty.
    pub target: Option<String>,
}

/// Extrude the hex. The `Some` is the body the extrude named, when the host names one.
pub fn extrude<R: Runner>(r: &mut R, hex: &Hex) -> Result<Option<String>, String> {
    extrude_reply(r, hex).map(|made| body_name_of(&made))
}

/// Extrude the hex and hand back the host's whole reply. A cut reads the bodies it
/// changed from it.
fn extrude_reply<R: Runner>(r: &mut R, hex: &Hex) -> Result<serde_json::Value, String> {
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": hex.origin,
                "x_dir": hex.x_dir,
                "y_dir": hex.y_dir,
            },
            "name": hex.sketch_name,
        }),
    )?;
    let sketch = sketch_id(&created)?;
    draw_profile(r, hex.across_flats, None)?;
    r.run("sketch.finish", json!({}))?;
    let mut extrude = json!({
        "distance": hex.height,
        "operation": hex.operation,
        "sketch": sketch,
        "name": hex.name,
    });
    if let Some(target) = &hex.target {
        extrude["targets"] = json!([target]);
    }
    if let Some(body_name) = &hex.body_name {
        extrude["body_name"] = json!(body_name);
    }
    r.run("solid.extrude", extrude)
}

/// Draw a dimensioned hex in a fresh sketch. With a projected center, constrain
/// the support circle to it; otherwise drawing connects the center to the origin.
pub fn draw_profile<R: Runner>(
    r: &mut R,
    across_flats: f64,
    center: Option<&str>,
) -> Result<(), String> {
    if let Some(center) = center {
        return draw_linked_profile(r, across_flats, center);
    }
    r.run(
        "sketch.polygon.circumscribed",
        json!({"center": [0.0, 0.0], "radius": across_flats / 2.0, "sides": 6}),
    )?;
    // The circle through the flats fixes the size without fixing the center.
    r.run(
        "sketch.dimension",
        json!({"entities": ["c1"], "type": "diameter", "value": across_flats}),
    )?;
    r.run(
        "sketch.constraint.horizontal_vertical",
        json!({"line": "l1", "mode": "horizontal"}),
    )?;
    Ok(())
}

/// Locate each corner by a directed construction radius. Equal sides on circles
/// admit folded polygons; a large center edit can lead the solver into one of
/// those solutions. Six independent rays keep the corners ordered, even when
/// the center crosses the origin. Every radius reads the same width dimension.
fn draw_linked_profile<R: Runner>(
    r: &mut R,
    across_flats: f64,
    center: &str,
) -> Result<(), String> {
    let inspected = r.run("sketch.inspect", json!({}))?;
    let at = inspected["points"]
        .as_array()
        .and_then(|points| points.iter().find(|p| p["id"] == center))
        .and_then(|p| p.get("at"))
        .ok_or("sketch.inspect did not return the projected hex center")?;
    let at: [f64; 2] = serde_json::from_value(at.clone())
        .map_err(|_| "sketch.inspect returned an invalid projected hex center")?;
    let made = r.run(
        "sketch.circle.center",
        json!({
            "center": at, "radius": across_flats / 2.0, "construction": true, "connect": false,
        }),
    )?;
    let circle = curves(&made, 1, "sketch.circle.center")?.remove(0);
    let circle_center = format!("{circle}.center");
    r.run(
        "sketch.constraint.coincident",
        json!({"a": circle_center, "b": center}),
    )?;
    let dimension = r.run(
        "sketch.dimension",
        json!({
            "entities": [circle], "type": "diameter", "value": across_flats,
        }),
    )?;
    let width = dimension["param"]
        .as_str()
        .filter(|p| !p.is_empty())
        .ok_or("sketch.dimension did not return the hex width parameter")?;
    let radius = across_flats / 3.0_f64.sqrt();
    let angles = [60.0_f64, 120.0, 180.0, 240.0, 300.0, 0.0];
    let vertices: Vec<[f64; 2]> = angles
        .iter()
        .map(|angle| {
            let (sin, cos) = angle.to_radians().sin_cos();
            [at[0] + radius * cos, at[1] + radius * sin]
        })
        .collect();
    let drawn = r.run(
        "sketch.line",
        json!({"points": vertices, "closed": true, "connect": false}),
    )?;
    let sides = curves(&drawn, 6, "sketch.line")?;
    for (side, angle) in sides.iter().zip(angles) {
        let drawn = r.run("sketch.line", json!({
            "points": [circle_center, format!("{side}.start")], "construction": true, "connect": false,
        }))?;
        let spoke = curves(&drawn, 1, "sketch.line")?.remove(0);
        r.run(
            "sketch.dimension",
            json!({"entities": [spoke], "type": "length", "value": format!("{width} / sqrt(3)")}),
        )?;
        r.run(
            "sketch.dimension",
            json!({
                "entities": [spoke], "type": "angle", "value": format!("{angle} deg"),
            }),
        )?;
    }
    Ok(())
}

fn curves(reply: &Value, count: usize, command: &str) -> Result<Vec<String>, String> {
    let ids = reply["curves"]
        .as_array()
        .filter(|ids| ids.len() == count)
        .and_then(|ids| {
            ids.iter()
                .map(|id| id.as_str().filter(|id| !id.is_empty()).map(str::to_string))
                .collect()
        });
    ids.ok_or_else(|| format!("{command} did not return {count} hex curve ids"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    struct Rec(Vec<(String, Value)>);
    impl Runner for Rec {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            let reply = match command {
                "sketch.create" => json!({"sketch": 7}),
                "solid.extrude" => json!({"created_bodies": ["Nut"]}),
                _ => Value::Null,
            };
            self.0.push((command.to_string(), args));
            Ok(reply)
        }
    }

    fn sample() -> Hex {
        Hex {
            origin: [0.0, 0.0, 0.0],
            x_dir: [1.0, 0.0, 0.0],
            y_dir: [0.0, 1.0, 0.0],
            across_flats: 10.0,
            height: 5.2,
            sketch_name: "Nut M6: Hex profile".into(),
            name: "Nut M6: Hex".into(),
            operation: "new",
            body_name: Some("Nut".into()),
            target: None,
        }
    }

    #[test]
    fn a_new_hex_is_dimensioned_across_the_flats() {
        let mut rec = Rec(vec![]);
        let body = extrude(&mut rec, &sample()).unwrap();
        assert_eq!(body.as_deref(), Some("Nut"));
        assert_eq!(
            rec.0.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "sketch.create",
                "sketch.polygon.circumscribed",
                "sketch.dimension",
                "sketch.constraint.horizontal_vertical",
                "sketch.finish",
                "solid.extrude",
            ]
        );
        assert_eq!(rec.0[1].1["radius"], 5.0);
        assert_eq!(rec.0[1].1["sides"], 6);
        assert_eq!(rec.0[2].1["value"], 10.0);
        assert_eq!(rec.0[5].1["distance"], 5.2);
        assert_eq!(rec.0[5].1["operation"], "new");
        assert_eq!(rec.0[5].1["sketch"], 7);
        assert_eq!(rec.0[5].1["body_name"], "Nut");
        assert_eq!(rec.0[0].1["name"], "Nut M6: Hex profile");
        assert_eq!(rec.0[5].1["name"], "Nut M6: Hex");
        assert!(rec.0[5].1.get("targets").is_none());
    }
}
