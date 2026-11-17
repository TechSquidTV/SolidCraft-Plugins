//! Named hole walls and seating faces shared by fastener attachments.

use plugin_kit::{Placement, Runner};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub definition: String,
    pub path: Vec<u64>,
}

#[derive(Deserialize)]
struct WorldBodies {
    bodies: Vec<WorldBody>,
}

#[derive(Deserialize)]
struct WorldBody {
    name: String,
    definition: String,
}

#[derive(Deserialize)]
struct Selection {
    items: Vec<SelectedBody>,
}

#[derive(Deserialize)]
struct SelectedBody {
    name: String,
    // The host omits the occurrence path for root bodies.
    #[serde(default)]
    occurrence_path: Vec<u64>,
}

// Select/get is the host's structured world-body → occurrence-path resolver.
// Preserve the user's selection, including on inspection errors. Never parse the
// formatted world-body labels: names may themselves contain occurrence syntax.
fn body_paths<R: Runner>(r: &mut R, bodies: &[&WorldBody]) -> Result<Selection, String> {
    let saved = r.run("selection.get", json!({"detail": false}))?;
    let items = saved
        .get("items")
        .and_then(Value::as_array)
        .ok_or("selection.get did not return the selection to restore")?;
    r.run(
        "select.set",
        json!({"items": bodies.iter().map(|body|
        json!({"type": "body", "name": body.name})).collect::<Vec<_>>()}),
    )?;
    let inspected = r.run("selection.get", json!({"detail": true}));
    let restored = r.run("select.set", json!({"items": items}));
    restored?;
    let reply = inspected?;
    if reply.get("detail_limit").is_some() {
        return Err("too many fastener seat placements to inspect".into());
    }
    serde_json::from_value(reply)
        .map_err(|e| format!("selection.get: cannot read seat placements: {e}"))
}

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
    /// Cuts return definition names. Inspect each placed copy before hardware can
    /// obscure the seat; the seat's world frame selects the matching occurrence.
    /// Axis-based attachments retain first-placement inspection: the host's axis
    /// joint origin does not honor an exact nested path like a named hole does.
    pub fn read_all<R: Runner>(
        r: &mut R,
        bodies: &[String],
        include_copies: bool,
    ) -> Result<Vec<(Body, Self)>, String> {
        if bodies.is_empty() {
            return Ok(Vec::new());
        }
        let world: WorldBodies =
            serde_json::from_value(r.run("inspect.measure", plugin_kit::host::body_names_query())?)
                .map_err(|e| {
                    format!("inspect.measure: cannot read fastener seat body placements: {e}")
                })?;
        for body in bodies {
            if !world
                .bodies
                .iter()
                .any(|placed| &placed.definition == body || &placed.name == body)
            {
                return Err(format!("fastener seat body {body} has no placed geometry"));
            }
        }
        let placed: Vec<_> = world
            .bodies
            .iter()
            .filter(|placed| bodies.contains(&placed.definition) || bodies.contains(&placed.name))
            .filter(|placed| include_copies || placed.name == placed.definition)
            .collect();
        let paths = body_paths(r, &placed)?;
        placed
            .into_iter()
            .map(|body| {
                let selected = paths
                    .items
                    .iter()
                    .find(|item| item.name == body.name)
                    .ok_or_else(|| {
                        format!("selection.get did not resolve seat body {}", body.name)
                    })?;
                let reply = r.run("model.faces", json!({"body": body.name}))?;
                let faces = serde_json::from_value(reply)
                    .map_err(|e| format!("model.faces: cannot read fastener seat faces: {e}"))?;
                Ok((
                    Body {
                        definition: body.definition.clone(),
                        path: selected.occurrence_path.clone(),
                    },
                    faces,
                ))
            })
            .collect()
    }
}

pub enum Seat {
    Hole {
        body: Body,
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
            } => {
                let mut origin = json!({
                    "body": body.definition, "circle": bore, "end": end,
                    "occurrence": body.path.last().copied().unwrap_or(0),
                    "x": turn.as_ref().map_or_else(|| json!(place.y_dir), |face| json!(face)),
                });
                if !body.path.is_empty() {
                    origin["path"] = json!(body.path);
                }
                origin
            }
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
    fn placed_seats_keep_definition_identity_paths_and_selection() {
        struct Host {
            selection: Value,
            inspected: Result<Value, String>,
            calls: Vec<(String, Value)>,
        }
        impl Runner for Host {
            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                self.calls.push((command.into(), args.clone()));
                match command {
                    "inspect.measure" => Ok(json!({"bodies": [
                        {"name": "Plate (original)", "definition": "Plate (original)"},
                        {"name": "A label, not a path", "definition": "Plate (original)"},
                        {"name": "Unrelated", "definition": "Unrelated"},
                    ]})),
                    "select.set" => {
                        self.selection = args["items"].clone();
                        Ok(Value::Null)
                    }
                    "selection.get" if args["detail"] == false => {
                        Ok(json!({"items": self.selection}))
                    }
                    "selection.get" => self.inspected.clone(),
                    "model.faces" => Ok(json!({"faces": []})),
                    "joint.create" => Ok(json!({"joint": 7, "errors": [], "conflicts": []})),
                    _ => panic!("unexpected command: {command}"),
                }
            }
        }
        let original = json!([{"type": "external_sketch_point", "sketch": 42, "id": "p"}]);
        for inspected in [
            Ok(json!({"items": [
                {"name": "Plate (original)"},
                {"name": "A label, not a path", "occurrence_path": [9, 12]},
            ]})),
            Err("cannot inspect selection".into()),
            Ok(json!({"items": [], "detail_limit": 500})),
            Ok(json!({"items": []})),
        ] {
            let success = inspected
                .as_ref()
                .is_ok_and(|v| v["items"].as_array().unwrap().len() == 2);
            let mut host = Host {
                selection: original.clone(),
                inspected,
                calls: Vec::new(),
            };
            let result = Faces::read_all(&mut host, &["Plate (original)".into()], true);
            assert_eq!(result.is_ok(), success);
            assert_eq!(host.selection, original);
            if let Ok(bodies) = result {
                assert_eq!(bodies.len(), 2);
                assert_eq!(bodies[1].0.definition, "Plate (original)");
                assert_eq!(bodies[1].0.path, [9, 12]);
                for (body, _) in bodies {
                    let path = body.path.clone();
                    Seat::Hole {
                        body,
                        bore: "bore".into(),
                        end: "floor".into(),
                        turn: None,
                    }
                    .attach(&mut host, 13, &Placement::on_origin(), "Nut")
                    .unwrap();
                    let origin = &host.calls.last().unwrap().1["a"];
                    assert_eq!(origin["body"], "Plate (original)");
                    assert_eq!(origin["occurrence"], path.last().copied().unwrap_or(0));
                    assert_eq!(
                        origin.get("path"),
                        (!path.is_empty()).then(|| json!(path)).as_ref()
                    );
                }
                assert_eq!(
                    host.calls
                        .iter()
                        .filter(|(c, _)| c == "model.faces")
                        .count(),
                    2
                );
                host.calls.clear();
                let first =
                    Faces::read_all(&mut host, &["Plate (original)".into()], false).unwrap();
                assert_eq!(first.len(), 1);
                assert!(first[0].0.path.is_empty());
                assert_eq!(
                    host.calls
                        .iter()
                        .filter(|(c, _)| c == "model.faces")
                        .count(),
                    1
                );
                assert_eq!(host.selection, original);
            }
        }
    }

    #[test]
    fn attachment_reports_its_solve_failure_without_blame_for_unrelated_joints() {
        struct Host(Value);
        impl Runner for Host {
            fn run(&mut self, _: &str, _: Value) -> Result<Value, String> {
                Ok(self.0.clone())
            }
        }
        let seat = Seat::Hole {
            body: Body {
                definition: "Plate".into(),
                path: vec![],
            },
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
