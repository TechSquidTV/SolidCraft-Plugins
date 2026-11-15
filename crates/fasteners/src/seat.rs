//! Named hole walls and seating faces shared by fastener attachments.

use plugin_kit::{Placement, Runner};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct Faces {
    pub faces: Vec<Face>,
}

#[derive(Deserialize)]
pub struct Face {
    pub name: String,
    pub centroid: [f64; 3],
    pub point: Option<[f64; 3]>,
    pub plane_normal: Option<[f64; 3]>,
    pub cylinder: Option<Cylinder>,
}

#[derive(Deserialize)]
pub struct Cylinder {
    pub origin: [f64; 3],
    pub axis: [f64; 3],
    pub radius: f64,
}

pub fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

pub fn same(a: [f64; 3], b: [f64; 3]) -> bool {
    a.into_iter().zip(b).all(|(a, b)| near(a, b))
}

pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

pub fn unique<T>(mut items: impl Iterator<Item = T>) -> Option<T> {
    let first = items.next()?;
    items.next().is_none().then_some(first)
}

impl Cylinder {
    pub fn on_axis(&self, place: &Placement) -> bool {
        let delta = std::array::from_fn(|i| self.origin[i] - place.point[i]);
        let along = dot(delta, place.normal);
        near(dot(self.axis, place.normal).abs(), 1.0)
            && same(delta, place.normal.map(|n| n * along))
    }
}

impl Face {
    /// A plane through the seat with its normal pointing out of the material.
    pub fn at_seat(&self, place: &Placement) -> bool {
        self.plane_normal.is_some_and(|n| same(n, place.normal))
            && near(
                dot(
                    std::array::from_fn(|i| self.centroid[i] - place.point[i]),
                    place.normal,
                ),
                0.0,
            )
    }
}

impl Faces {
    /// Read each modified body once, before any hardware can obscure the seat.
    pub fn read_all<R: Runner>(
        r: &mut R,
        bodies: &[String],
    ) -> Result<Vec<(String, Self)>, String> {
        bodies
            .iter()
            .map(|body| {
                let reply = r.run("model.faces", json!({"body": body}))?;
                let faces = serde_json::from_value(reply)
                    .map_err(|e| format!("model.faces: cannot read fastener seat faces: {e}"))?;
                Ok((body.clone(), faces))
            })
            .collect()
    }
}

pub enum Seat {
    Hole {
        body: String,
        bore: String,
        end: String,
        /// A pocket wall keeps a hex nut's turn tied to its pocket.
        turn: Option<String>,
    },
    /// A native axis built before thread cuts split the seating face.
    Axis { name: String },
}

impl Seat {
    /// Both sides are initially at `place`; the named host end drives later edits.
    pub fn attach<R: Runner>(
        &self,
        r: &mut R,
        occurrence: u64,
        place: &Placement,
        name: &str,
    ) -> Result<(), String> {
        let host = match self {
            Self::Hole {
                body,
                bore,
                end,
                turn,
            } => json!({
                "body": body, "circle": bore, "end": end,
                "x": turn.as_ref().map_or_else(|| json!(place.y_dir), |face| json!(face)),
            }),
            Self::Axis { name } => json!({"axis": name, "x": place.y_dir}),
        };
        let reply = r.run(
            "joint.create",
            json!({
                "type": "rigid", "name": format!("{name}: Seat {occurrence}"),
                "a": host,
                "b": {"occurrence": occurrence, "point": place.point,
                      "z": place.normal, "x": place.y_dir},
                "flip": true, "angle": 0,
            }),
        )?;
        let joint = reply
            .get("joint")
            .and_then(Value::as_u64)
            .ok_or("joint.create did not return the fastener seat id")?;
        for key in ["errors", "conflicts"] {
            let problems = reply
                .get(key)
                .and_then(Value::as_array)
                .ok_or_else(|| format!("joint.create did not report fastener seat {key}"))?;
            if let Some(problem) = problems.iter().find(|p| p["joint"].as_u64() == Some(joint)) {
                return Err(format!("joint.create: fastener seat {key}: {problem}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_reports_its_solve_failure_without_blame_for_unrelated_joints() {
        struct Host(Value);
        impl Runner for Host {
            fn run(&mut self, _: &str, _: Value) -> Result<Value, String> {
                Ok(self.0.clone())
            }
        }
        let seat = Seat::Hole {
            body: "Plate".into(),
            bore: "bore".into(),
            end: "floor".into(),
            turn: None,
        };
        for key in ["errors", "conflicts"] {
            let mut reply = json!({"joint": 8, "errors": [], "conflicts": []});
            reply[key] = json!([{"joint": 8, "why": "lost floor"}]);
            assert!(seat
                .attach(
                    &mut Host(reply.clone()),
                    3,
                    &Placement::on_origin(),
                    "Screw M6"
                )
                .unwrap_err()
                .contains("lost floor"));
            reply[key][0]["joint"] = json!(9);
            seat.attach(&mut Host(reply), 3, &Placement::on_origin(), "Screw M6")
                .unwrap();
        }
        assert!(seat
            .attach(&mut Host(json!({})), 3, &Placement::on_origin(), "Screw M6")
            .is_err());
    }
}
