//! Named hole walls and seating faces shared by fastener attachments.

use plugin_kit::{Placement, Runner};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub definition: String,
    pub path: Vec<u64>,
}

#[derive(Deserialize)]
struct BodyList<T> {
    bodies: Vec<T>,
}

#[derive(Deserialize)]
struct WorldBody {
    name: String,
    definition: String,
}

#[derive(Deserialize)]
struct MeasuredBody {
    name: String,
    bbox: Bounds,
}

#[derive(Deserialize)]
struct Bounds {
    min: [f64; 3],
    max: [f64; 3],
}

impl Bounds {
    fn contains(&self, point: [f64; 3]) -> bool {
        (0..3).all(|i| {
            (point[i] >= self.min[i] || near(point[i], self.min[i]))
                && (point[i] <= self.max[i] || near(point[i], self.max[i]))
        })
    }
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
    /// The placed body these are the faces of, as `model.faces` was asked for it.
    #[serde(skip)]
    pub body: String,
    /// The body's edges, once [`Faces::read_edges`] has asked for them.
    #[serde(skip)]
    pub edges: Vec<Edge>,
}

/// One edge of a body: the host names it `<face A>|<face B>` from the faces it joins,
/// with `~k` after it when several edges join the same two.
#[derive(Deserialize)]
pub struct Edge {
    pub name: Option<String>,
    pub mid: [f64; 3],
}

impl Edge {
    /// Whether `face` is one of the two faces this edge joins.
    pub fn borders(&self, face: &str) -> bool {
        self.name.as_deref().is_some_and(|name| {
            let name = match name.rsplit_once('~') {
                Some((joined, k)) if !k.is_empty() && k.bytes().all(|b| b.is_ascii_digit()) => {
                    joined
                }
                _ => name,
            };
            name.strip_prefix(face)
                .is_some_and(|rest| rest.starts_with('|'))
                || name
                    .strip_suffix(face)
                    .is_some_and(|rest| rest.ends_with('|'))
        })
    }
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
    /// Cuts return definition names. Bound candidate copies by the actual seating
    /// points before inspecting faces or resolving paths. Keep all overlapping
    /// candidates so the caller can still reject ambiguous seats.
    /// Axis-based attachments retain first-placement inspection: the host's axis
    /// joint origin does not honor an exact nested path like a named hole does.
    pub fn read_all<R: Runner>(
        r: &mut R,
        bodies: &[String],
        places: &[Placement],
        include_copies: bool,
    ) -> Result<Vec<(Body, Self)>, String> {
        if bodies.is_empty() || places.is_empty() {
            return Ok(Vec::new());
        }
        let world: BodyList<WorldBody> =
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
        if placed.is_empty() {
            return Ok(Vec::new());
        }
        // One batched measurement, not one host call per copy. Only measure the
        // cut definitions; unrelated hardware need not be measured at all.
        let measured: BodyList<MeasuredBody> = serde_json::from_value(r.run(
            "inspect.measure",
            json!({"bodies": placed.iter().map(|body| &body.name).collect::<Vec<_>>()}),
        )?)
        .map_err(|e| format!("inspect.measure: cannot read fastener seat bounds: {e}"))?;
        let bounds: BTreeMap<_, _> = measured
            .bodies
            .into_iter()
            .map(|body| (body.name, body.bbox))
            .collect();
        let mut candidates = Vec::new();
        for body in placed {
            let bounds = bounds.get(&body.name).ok_or_else(|| {
                format!("inspect.measure did not return bounds for {}", body.name)
            })?;
            if places.iter().any(|place| bounds.contains(place.point)) {
                candidates.push(body);
            }
        }
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let placed = candidates;
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
                let mut faces: Faces = serde_json::from_value(reply)
                    .map_err(|e| format!("model.faces: cannot read fastener seat faces: {e}"))?;
                faces.body = body.name.clone();
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

impl Faces {
    /// Ask the host for the body's edges. A seat needs them only to tell apart faces
    /// that lie in one plane, such as the two sides of a grooved top.
    pub fn read_edges<R: Runner>(&mut self, r: &mut R) -> Result<(), String> {
        #[derive(Deserialize)]
        struct Edges {
            edges: Vec<Edge>,
        }
        let reply = r.run("model.edges", json!({"body": self.body}))?;
        let read: Edges = serde_json::from_value(reply)
            .map_err(|e| format!("model.edges: cannot read fastener seat edges: {e}"))?;
        self.edges = read.edges;
        Ok(())
    }

    /// Whether `face` has an edge on the seat plane within `radius` of the hole's axis:
    /// the rim of the hole at `place`.
    pub fn rims(&self, face: &Face, place: &Placement, radius: f64) -> bool {
        self.edges.iter().any(|edge| {
            let delta: [f64; 3] = std::array::from_fn(|i| edge.mid[i] - place.point[i]);
            let along = dot(delta, place.normal);
            let radial = (dot(delta, delta) - along * along).max(0.0).sqrt();
            near(along, 0.0) && radial < radius + 1e-5 && edge.borders(&face.name)
        })
    }
}

pub enum Seat {
    /// A round end resolved by the host from a viewport pick.
    Bore { origin: Value },
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
            Self::Bore { origin } => {
                let mut host = origin.clone();
                host["x"] = json!(place.y_dir);
                host
            }
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

    /// The host names an edge from the two faces it joins, and numbers repeats.
    #[test]
    fn an_edge_borders_the_two_faces_in_its_name() {
        let edge = |name: &str| Edge {
            name: Some(name.into()),
            mid: [0.0; 3],
        };
        let top = "F1:+z#[F1:-y,F2:-y]";
        assert!(edge(&format!("{top}|F3:side")).borders(top));
        assert!(edge(&format!("F3:side|{top}~2")).borders(top));
        assert!(edge(&format!("F3:side|{top}")).borders("F3:side"));
        // The other side of the groove is a different face with a longer name.
        assert!(!edge("F1:+z#[F1:+y,F2:+y]|F3:side").borders(top));
        assert!(!edge("F3:side|F4:+z").borders("F3"));
        assert!(!Edge {
            name: None,
            mid: [0.0; 3]
        }
        .borders(top));
    }

    /// Of two faces in the seat's plane, the one with the hole's rim is the seat.
    #[test]
    fn the_face_with_the_rim_of_the_hole_is_its_seat() {
        let face = |name: &str, y: f64| Face {
            name: name.into(),
            centroid: [20.0, y, 10.0],
            point: None,
            plane_normal: Some([0.0, 0.0, 1.0]),
            cylinder: None,
        };
        let faces = Faces {
            faces: vec![face("near", 8.5), face("far", 31.5)],
            body: "Plate".into(),
            edges: vec![
                Edge {
                    name: Some("near|bore".into()),
                    mid: [11.7, 8.0, 10.0],
                },
                Edge {
                    name: Some("far|side".into()),
                    mid: [20.0, 23.0, 10.0],
                },
            ],
        };
        let place = Placement {
            point: [10.0, 8.0, 10.0],
            ..Placement::on_origin()
        };
        assert!(faces.rims(&faces.faces[0], &place, 1.7));
        assert!(!faces.rims(&faces.faces[1], &place, 1.7));
        // A wider edge of the right face is not the rim of this hole.
        assert!(!faces.rims(&faces.faces[0], &place, 1.0));
    }

    struct Host {
        selection: Value,
        inspected: Option<Result<Value, String>>,
        bodies: Vec<Value>,
        calls: Vec<(String, Value)>,
    }
    impl Host {
        fn new(bodies: Vec<Value>) -> Self {
            Self {
                selection: json!([{"type": "external_sketch_point", "sketch": 42, "id": "p"}]),
                inspected: None,
                bodies,
                calls: Vec::new(),
            }
        }
    }
    impl Runner for Host {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            self.calls.push((command.into(), args.clone()));
            match command {
                "inspect.measure" => Ok(json!({"bodies": self.bodies.iter().filter(|body| {
                        args["bodies"].as_array().is_none_or(|names| names.contains(&body["name"]))
                    }).collect::<Vec<_>>()})),
                "select.set" => {
                    self.selection = args["items"].clone();
                    Ok(Value::Null)
                }
                "selection.get" if args["detail"] == false => Ok(json!({"items": self.selection})),
                "selection.get" => self.inspected.clone().unwrap_or_else(|| {
                    Ok(json!({
                        "items": self.selection.as_array().unwrap().iter().map(|item| {
                            self.bodies.iter().find(|body| body["name"] == item["name"]).unwrap()
                        }).collect::<Vec<_>>()
                    }))
                }),
                "model.faces" => Ok(json!({"faces": []})),
                "joint.create" => Ok(json!({"joint": 7, "errors": [], "conflicts": []})),
                _ => panic!("unexpected command: {command}"),
            }
        }
    }

    fn body(name: &str, definition: &str, path: &[u64], x: f64) -> Value {
        json!({"name": name, "definition": definition, "occurrence_path": path,
               "bbox": {"min": [x, -10, 0], "max": [x + 10.0, 10, 12]}})
    }

    #[test]
    fn placed_seats_keep_definition_identity_paths_and_selection() {
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
            let mut host = Host::new(vec![
                body("Plate (original)", "Plate (original)", &[], 0.0),
                body("A label, not a path", "Plate (original)", &[9, 12], 0.0),
                body("Unrelated", "Unrelated", &[], 0.0),
            ]);
            host.inspected = Some(inspected);
            let places = [Placement::on_origin()];
            let result = Faces::read_all(&mut host, &["Plate (original)".into()], &places, true);
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
                    Faces::read_all(&mut host, &["Plate (original)".into()], &places, false)
                        .unwrap();
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
    fn distant_copies_do_not_add_face_reads_or_exceed_selection_limits() {
        for copies in [1, 600] {
            let mut host = Host::new(
                (0..=copies)
                    .map(|i| {
                        body(
                            &format!("Plate {i}"),
                            "Plate",
                            &[1000 + i, 12],
                            100.0 * i as f64,
                        )
                    })
                    .collect(),
            );
            let selection = host.selection.clone();
            // Two requested copies, including points on a box edge and just
            // within the face-matching tolerance of the top boundary.
            let places =
                [[0.0, 0.0, 0.0], [100.0 * copies as f64, 0.0, 12.0 + 1e-6]].map(|point| {
                    Placement {
                        point,
                        ..Placement::on_origin()
                    }
                });
            let found = Faces::read_all(&mut host, &["Plate".into()], &places, true).unwrap();
            assert_eq!(found.len(), 2);
            assert_eq!(found[0].0.path, [1000, 12]);
            assert_eq!(found[1].0.path, [1000 + copies, 12]);
            assert_eq!(host.selection, selection);
            assert_eq!(host.calls.len(), 8);
            assert_eq!(
                host.calls
                    .iter()
                    .filter(|(c, _)| c == "model.faces")
                    .count(),
                2
            );
            assert_eq!(
                host.calls
                    .iter()
                    .find(|(c, _)| c == "select.set")
                    .unwrap()
                    .1["items"]
                    .as_array()
                    .unwrap()
                    .len(),
                2
            );
        }
    }

    #[test]
    fn absent_seats_skip_selection_and_uninspectable_bounds_are_refused() {
        let mut host = Host::new(vec![body("Plate", "Plate", &[], 0.0)]);
        let selection = host.selection.clone();
        let places = [Placement {
            point: [0.0, 0.0, 13.0],
            ..Placement::on_origin()
        }];
        assert!(Faces::read_all(&mut host, &["Plate".into()], &places, true)
            .unwrap()
            .is_empty());
        assert_eq!(host.calls.len(), 2);
        assert_eq!(host.selection, selection);
        host.bodies[0].as_object_mut().unwrap().remove("bbox");
        assert!(Faces::read_all(&mut host, &["Plate".into()], &places, true)
            .err()
            .unwrap()
            .contains("bounds"));
        assert_eq!(host.selection, selection);
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
