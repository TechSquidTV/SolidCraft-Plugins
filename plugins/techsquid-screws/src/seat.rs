//! Seat screws on named hole ends, so the joints follow the cuts on rebuild.

use fasteners::seat::{dot, near, same, unique, Faces, Seat};
use plugin_kit::{component::Activation, place::Spot};
use plugin_kit::{Placement, Runner};
use serde_json::{json, Value};

use crate::screw::{Hole, HoleKind};

pub struct Attachment {
    seat: Seat,
    frame: Placement,
}

impl Attachment {
    pub fn attach<R: Runner>(&self, r: &mut R, occurrence: u64, name: &str) -> Result<(), String> {
        self.seat.attach(r, occurrence, &self.frame, name)
    }
}

/// Use each sketch's supporting plane or face, evaluated just after that sketch.
/// Layout planes need no solid yet; later thread cuts cannot split support faces.
/// Keep the plane in its sketch's component frame and restore the user's context.
pub fn tap_axes<R: Runner>(r: &mut R, spots: &[Spot], name: &str) -> Result<Vec<String>, String> {
    if spots.is_empty() {
        return Ok(Vec::new());
    }
    let doc = r.run(
        "document.inspect",
        json!({"timeline": false, "bodies": false}),
    )?;
    let marker = doc
        .get("marker")
        .and_then(Value::as_u64)
        .ok_or("document.inspect did not return the history marker")?;
    let active = Activation::read(r)?;
    let mut component = active.component;
    let mut changed_activation = false;
    let mut added = 0;
    let mut source: Option<(&str, Value)> = None;
    let axes: Result<Vec<String>, String> = spots
        .iter()
        .map(|spot| {
            if source.as_ref().map(|(sketch, _)| *sketch) != Some(spot.sketch.as_str()) {
                let inspected = r.run("timeline.inspect", json!({"feature": spot.sketch}))?;
                let definition = &inspected["definition"];
                let plane = definition
                    .get("plane")
                    .filter(|plane| plane.is_object())
                    .ok_or("timeline.inspect did not return the sketch plane reference")?;
                // The saved feature omits component 0 (the document root).
                let owner = definition["component"].as_u64().unwrap_or(0);
                if component != owner {
                    r.run("component.activate", json!({"component": owner}))?;
                    component = owner;
                    changed_activation = true;
                }
                r.run("timeline.roll_to", json!({"feature": spot.sketch}))?;
                // Face-backed sketch planes store a snapshot as well as their
                // attachment. Construction axes need the live face reference;
                // evaluating that saved plane directly would use the snapshot.
                let support = if plane["type"] == "face" {
                    json!({"type": "face", "at": plane["at"], "name": plane["name"]})
                } else {
                    json!({"type": "plane", "plane": plane})
                };
                source = Some((&spot.sketch, support));
            }
            let support = &source.as_ref().expect("inspected above").1;
            let reply = r.run(
                "construct.axis.normal_to_face",
                json!({
                    "name": format!("{name}: Seat axis {} {}", spot.sketch, spot.point),
                    "face": support,
                    "point": {"sketch": spot.sketch, "point": spot.point},
                }),
            )?;
            added += 1;
            reply
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| {
                    "construct.axis.normal_to_face did not return the screw seat axis".into()
                })
        })
        .collect();
    let restored = r.run("timeline.roll_to", json!({"position": marker + added}));
    let activated = if changed_activation {
        active.restore(r)
    } else {
        Ok(())
    };
    let axes = axes?;
    restored?;
    activated?;
    Ok(axes)
}

pub fn find_all<R: Runner>(
    r: &mut R,
    bodies: &[String],
    places: &[Placement],
    axes: &[String],
    hole: &Hole,
    major: f64,
) -> Result<Vec<Attachment>, String> {
    let faces = Faces::read_all(r, bodies)?;
    places
        .iter()
        .enumerate()
        .map(|(i, place)| find(&faces, place, axes.get(i).map(String::as_str), hole, major))
        .collect()
}

fn find(
    bodies: &[(String, Faces)],
    place: &Placement,
    axis: Option<&str>,
    hole: &Hole,
    major: f64,
) -> Result<Attachment, String> {
    let tapped = matches!(hole.kind, HoleKind::Tapped { .. });
    let clearance = fasteners::clearance::medium(major).map(|d| d / 2.0);
    // A countersunk screw's cone meets the clearance bore above the foot of its
    // head. Attach that point to the cone/bore junction, including any head land.
    let shift = if matches!(hole.kind, HoleKind::Countersink { .. }) {
        clearance.ok_or("sketch_point: the countersink has no clearance size")? - major / 2.0
    } else {
        0.0
    };
    let frame = Placement {
        point: place.along(shift),
        ..*place
    };
    let mut found = None;
    for (body, faces) in bodies {
        let mut bores = faces.faces.iter().filter(|f| {
            f.cylinder.as_ref().is_some_and(|c| {
                c.on_axis(place)
                    && if tapped {
                        c.radius < major / 2.0 && !near(c.radius, major / 2.0)
                    } else {
                        clearance.is_some_and(|radius| near(c.radius, radius))
                    }
            })
        });
        let bore = if tapped {
            // Confirm the pilot wall exists, but do not persist a thread face's
            // generated index: it can change when the thread is rebuilt.
            bores.next()
        } else {
            unique(&mut bores)
        };
        let end = unique(faces.faces.iter().filter(|f| match hole.kind {
            HoleKind::Countersink { .. } => {
                // Check a point on the 90-degree cone. Its radius grows one mm
                // per mm along the outward axis from the bore junction.
                f.plane_normal.is_none()
                    && f.cylinder.is_none()
                    && f.point.is_some_and(|point| {
                        let delta = std::array::from_fn(|i| point[i] - frame.point[i]);
                        let along = dot(delta, frame.normal);
                        let radial = (dot(delta, delta) - along * along).max(0.0).sqrt();
                        along > 0.0 && clearance.is_some_and(|radius| near(radial, radius + along))
                    })
            }
            HoleKind::Counterbore { .. } => f.at_seat(&frame) && same(f.centroid, frame.point),
            _ => f.at_seat(&frame),
        }));
        if let (Some(bore), Some(end)) = (bore, end) {
            if found.is_some() {
                return Err("sketch_point: more than one part has this screw seat".into());
            }
            found = Some(Attachment {
                seat: if tapped {
                    Seat::Axis {
                        name: axis.ok_or("tapped screw seat has no axis")?.to_string(),
                    }
                } else {
                    Seat::Hole {
                        body: body.clone(),
                        bore: bore.name.clone(),
                        end: end.name.clone(),
                        turn: None,
                    }
                },
                frame,
            });
        }
    }
    found.ok_or_else(|| "sketch_point: the screw needs a complete hole and seating face to stay attached; move the point clear of edges or overlapping cuts".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn axis_creation_restores_history_and_exact_active_occurrence() {
        #[derive(Default)]
        struct Host {
            calls: Vec<(String, Value)>,
            axes: usize,
            fail_second: bool,
        }
        impl Runner for Host {
            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                self.calls.push((command.into(), args.clone()));
                match command {
                    "document.inspect" => Ok(json!({"marker": 40})),
                    "component.list" => Ok(json!({"active": 7, "active_path": [9, 12]})),
                    "timeline.inspect" => Ok(json!({"definition": {
                        "component": if args["feature"] == "Original" { 7 } else { 0 },
                        "plane": {"type": "origin", "name": "XY"}
                    }})),
                    "construct.axis.normal_to_face" => {
                        self.axes += 1;
                        if self.fail_second && self.axes == 2 {
                            Err("missing seating surface".into())
                        } else {
                            Ok(json!({"name": args["name"]}))
                        }
                    }
                    _ => Ok(json!({})),
                }
            }
        }
        for return_to_original in [false, true] {
            for fail_second in [false, true] {
                let second_sketch = if return_to_original {
                    "Original"
                } else {
                    "Layout"
                };
                let spots =
                    [("Layout", "p"), (second_sketch, "c.center")].map(|(sketch, point)| Spot {
                        sketch: sketch.into(),
                        point: point.into(),
                        place: Placement::on_origin(),
                    });
                let mut host = Host {
                    fail_second,
                    ..Host::default()
                };
                let result = tap_axes(&mut host, &spots, "Screw M3");
                if fail_second {
                    assert_eq!(result.unwrap_err(), "missing seating surface");
                } else {
                    assert_eq!(result.unwrap().len(), 2);
                }
                // Returning to component 7 by ID still loses its selected copy.
                // Both success and failure must restore the complete original path.
                assert_eq!(
                    &host.calls[host.calls.len() - 2..],
                    &[
                        (
                            "timeline.roll_to".into(),
                            json!({"position": if fail_second { 41 } else { 42 }})
                        ),
                        (
                            "component.activate".into(),
                            json!({"component": 7, "occurrence": 12, "path": [9, 12]})
                        ),
                    ]
                );
            }
        }
    }

    #[test]
    fn missing_or_ambiguous_seats_are_refused() {
        let hole = crate::screw::parse(&json!({"create": "Screw and hole"}))
            .unwrap()
            .hole
            .unwrap();
        let faces = || {
            serde_json::from_value(json!({"faces": [
                {"name": "top", "centroid": [10, 10, 0], "plane_normal": [0, 0, 1]},
                {"name": "bore", "centroid": [0, 0, -2], "cylinder": {
                    "origin": [0, 0, -5], "axis": [0, 0, 1], "radius": 1.7
                }}
            ]}))
            .unwrap()
        };
        for index in 0..2 {
            let mut missing: Faces = faces();
            missing.faces.remove(index);
            assert!(find(
                &[("Plate".into(), missing)],
                &Placement::on_origin(),
                None,
                &hole,
                3.0
            )
            .is_err());
            let mut ambiguous: Faces = faces();
            let mut extra: Faces = faces();
            ambiguous.faces.push(extra.faces.remove(index));
            assert!(find(
                &[("Plate".into(), ambiguous)],
                &Placement::on_origin(),
                None,
                &hole,
                3.0
            )
            .is_err());
        }
        assert!(find(
            &[("A".into(), faces()), ("B".into(), faces())],
            &Placement::on_origin(),
            None,
            &hole,
            3.0
        )
        .is_err());
    }
}
