//! A six-sided profile extruded along the sketch normal.
//!
//! `sketch.polygon.circumscribed` is the circle through the flats, so the radius is half
//! the width across the flats. The screw's hex head and the ISO 4032 nut both extrude that
//! profile. The centre stays on the sketch origin, and one side is horizontal so the flats
//! sit square.

use serde_json::json;

use crate::host::{body_name_of, sketch_id};
use crate::Runner;

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
pub fn extrude_reply<R: Runner>(r: &mut R, hex: &Hex) -> Result<serde_json::Value, String> {
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
    r.run(
        "sketch.polygon.circumscribed",
        json!({
            "center": [0.0, 0.0],
            "radius": hex.across_flats / 2.0,
            "sides": 6,
        }),
    )?;
    // The centre is already on the origin. The circle through the flats fixes the size.
    r.run(
        "sketch.dimension",
        json!({"entities": ["c1"], "type": "diameter", "value": hex.across_flats}),
    )?;
    r.run(
        "sketch.constraint.horizontal_vertical",
        json!({"line": "l1", "mode": "horizontal"}),
    )?;
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
