//! Seat screws on named hole ends, so the joints follow the cuts on rebuild.

use fasteners::seat::{dot, near, same, unique, Faces, Seat};
use plugin_kit::place::Spot;
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

/// Insert axes just after their source sketches, before earlier plugin runs can
/// split the seating surface with thread cuts. Restore the user's history marker.
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
    let mut added = 0;
    let mut sketch = None;
    let axes: Result<Vec<String>, String> = spots
        .iter()
        .map(|spot| {
            if sketch != Some(spot.sketch.as_str()) {
                r.run("timeline.roll_to", json!({"feature": spot.sketch}))?;
                sketch = Some(spot.sketch.as_str());
            }
            let reply = r.run(
                "construct.axis.normal_to_face",
                json!({
                    "name": format!("{name}: Seat axis {} {}", spot.sketch, spot.point),
                    "face": {"face": spot.place.point},
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
    let axes = axes?;
    restored?;
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
    fn axis_failure_restores_the_history_marker_with_successful_insertions_counted() {
        #[derive(Default)]
        struct Host {
            calls: Vec<(String, Value)>,
            axes: usize,
        }
        impl Runner for Host {
            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                self.calls.push((command.into(), args.clone()));
                match command {
                    "document.inspect" => Ok(json!({"marker": 40})),
                    "construct.axis.normal_to_face" => {
                        self.axes += 1;
                        if self.axes == 2 {
                            Err("missing seating surface".into())
                        } else {
                            Ok(json!({"name": args["name"]}))
                        }
                    }
                    _ => Ok(json!({})),
                }
            }
        }
        let spots = ["p", "c.center"].map(|point| Spot {
            sketch: "Layout".into(),
            point: point.into(),
            place: Placement::on_origin(),
        });
        let mut host = Host::default();
        assert_eq!(
            tap_axes(&mut host, &spots, "Screw M3").unwrap_err(),
            "missing seating surface"
        );
        assert_eq!(
            host.calls.last(),
            Some(&("timeline.roll_to".into(), json!({"position": 41})))
        );
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
