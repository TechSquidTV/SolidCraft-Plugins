//! Seat screws on named hole ends, so the joints follow the cuts on rebuild.

use fasteners::seat::{dot, near, same, unique, Body, Faces, Seat};
use plugin_kit::{component::Activation, name::part, place::Spot};
use plugin_kit::{
    place::{self, PlaceInput, Source},
    FieldError, Placement, Runner,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::screw::{Head, Hole, HoleKind, Params};

/// Keep source indices so a host-deduplicated bore batch can be interleaved with
/// sketch picks (which are independently deduplicated by their centre point).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picks {
    pub sketches: PlaceInput,
    sketch_indices: Vec<usize>,
    bores: Vec<(usize, Value)>,
}

impl Picks {
    pub fn parse(v: &Value) -> Result<Self, FieldError> {
        const WANTED: &str = "bore rims, bore walls, sketch points or circles";
        let mut picks = Self::default();
        let items = place::selection_items(v, WANTED)?;
        if items.len() > plugin_kit::component::MAX_PLACEMENTS {
            return Err(FieldError::new(
                place::FIELD,
                "choose at most 32 placement picks",
            ));
        }
        for (i, item) in items.iter().enumerate() {
            if matches!(item["type"].as_str(), Some("edge" | "face")) {
                picks.bores.push((i, item.clone()));
            } else if matches!(
                item["type"].as_str(),
                Some("external_sketch_point" | "external_sketch_curve" | "profile")
            ) {
                picks
                    .sketches
                    .picks
                    .push(place::sketch_pick(item, "sketch points or circles")?);
                picks.sketch_indices.push(i);
            } else {
                return Err(FieldError::new(place::FIELD, format!("pick {WANTED}")));
            }
        }
        Ok(picks)
    }

    pub fn has_bores(&self) -> bool {
        !self.bores.is_empty()
    }
}

#[derive(Deserialize)]
struct BoreEnd {
    pick_index: usize,
    origin: Value,
    point: [f64; 3],
    normal: [f64; 3],
    x_dir: [f64; 3],
    shank_offset: f64,
}

#[derive(Deserialize)]
struct BoreSeats {
    seats: Vec<BoreEnd>,
}

/// Before the new screw can obscure its pick, ask the host for the named
/// seat at each distinct bore. Its rim and seating normal are also the frame
/// for the joint, including when the screw is turned over in that frame.
pub fn picked_bores<R: Runner>(
    r: &mut R,
    p: &Params,
    spots: &[Spot],
    major: f64,
) -> Result<(Vec<Placement>, Vec<Option<Attachment>>), String> {
    let picks = &p.place;
    let mut args = json!({"picks": picks.bores.iter().map(|(_, pick)| pick).collect::<Vec<_>>()});
    if p.head == Head::Countersunk {
        args["countersunk_shank_diameter"] = json!(major);
    }
    let reply = r
        .run("parts.seats", args)
        .map_err(|err| format!("placement: {err}"))?;
    let reply: BoreSeats = serde_json::from_value(reply)
        .map_err(|e| format!("parts.seats: invalid screw seat: {e}"))?;
    if reply.seats.is_empty() {
        return Err("placement: pick a bore rim or wall".into());
    }
    // A pick uses the displayed pose. A rigid joint solves in the captured pose.
    // The host gives the exact path, including ancestors of a nested copy.
    let components = r.run("component.list", json!({"compact": true}))?;
    let pending = components["pending_moves"]
        .as_array()
        .ok_or("component.list: missing pending moves")?;
    let mut ordered: Vec<_> = spots
        .iter()
        .map(|spot| (picks.sketch_indices[spot.pick_index], spot.place, None))
        .collect();
    for end in reply.seats {
        let (index, _) = picks
            .bores
            .get(end.pick_index)
            .ok_or("parts.seats: invalid pick index")?;
        if !end.origin.is_object() {
            return Err("parts.seats: missing bore joint origin".into());
        }
        if end.origin["path"]
            .as_array()
            .is_some_and(|path| path.iter().any(|id| pending.contains(id)))
        {
            return Err("placement: use Capture Position to keep the picked part's move, or Revert Position to undo it. In an exploded view, return to the assembled view before picking the bore.".into());
        }
        let frame = Placement {
            point: end.point,
            normal: end.normal,
            x_dir: end.x_dir,
            y_dir: place::cross(end.normal, end.x_dir),
            source: Source::Bore,
        };
        let place = Placement {
            point: frame.along(if p.flip {
                -end.shank_offset
            } else {
                end.shank_offset
            }),
            ..frame
        };
        let attachment = Attachment {
            seat: Seat::Bore { origin: end.origin },
            frame,
        };
        ordered.push((*index, place, Some(attachment)));
    }
    ordered.sort_by_key(|(index, _, _)| *index);
    Ok(ordered
        .into_iter()
        .map(|(_, place, attachment)| (place, attachment))
        .unzip())
}

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
                    "name": part(&format!("Seat for {name}"), &format!("Axis {} {}", spot.sketch, spot.point)),
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
    // The cone/bore junction is the actual seat of a countersunk screw, slightly
    // above its origin. Use the same frames for bounds filtering and face matching.
    let shift = if matches!(hole.kind, HoleKind::Countersink { .. }) {
        fasteners::clearance::medium(major)
            .ok_or("placement: the countersink has no clearance size")?
            / 2.0
            - major / 2.0
    } else {
        0.0
    };
    let frames: Vec<_> = places
        .iter()
        .map(|place| Placement {
            point: place.along(shift),
            ..*place
        })
        .collect();
    // Axis origins still bind the first placement in the host. Keep copied tapped
    // seats refused instead of finding a copy and attaching to a different one.
    let mut faces = Faces::read_all(
        r,
        bodies,
        &frames,
        !matches!(hole.kind, HoleKind::Tapped { .. }),
    )?;
    // A groove or a step leaves several faces in the seat's plane. The edges say
    // which of them the hole is in; a body with one such face is not asked for them.
    let flat = matches!(
        hole.kind,
        HoleKind::Clearance { .. } | HoleKind::Tapped { .. }
    );
    for (_, faces) in faces.iter_mut().filter(|_| flat) {
        let shared = frames
            .iter()
            .any(|frame| faces.faces.iter().filter(|f| f.at_seat(frame)).count() > 1);
        if shared {
            faces.read_edges(r)?;
        }
    }
    frames
        .iter()
        .enumerate()
        .map(|(i, place)| find(&faces, place, axes.get(i).map(String::as_str), hole, major))
        .collect()
}

fn find(
    bodies: &[(Body, Faces)],
    place: &Placement,
    axis: Option<&str>,
    hole: &Hole,
    major: f64,
) -> Result<Attachment, String> {
    let tapped = matches!(hole.kind, HoleKind::Tapped { .. });
    let clearance = fasteners::clearance::medium(major).map(|d| d / 2.0);
    let frame = *place;
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
            _ => {
                // The hole's rim is no wider than the hole: the clearance bore, or
                // the thread's crests.
                let rim = if tapped { Some(major / 2.0) } else { clearance };
                f.at_seat(&frame)
                    && (faces.edges.is_empty()
                        || rim.is_some_and(|radius| faces.rims(f, &frame, radius)))
            }
        }));
        if let (Some(bore), Some(end)) = (bore, end) {
            if found.is_some() {
                return Err("placement: more than one part has this screw seat".into());
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
    found.ok_or_else(|| "placement: the screw needs a complete hole and seating face to stay attached; move the point clear of edges or overlapping cuts".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_pending_moves_on_the_picked_occurrence_path_block_a_bore_seat() {
        struct Host {
            origin: Value,
            pending: Vec<u64>,
        }
        impl Runner for Host {
            fn run(&mut self, command: &str, _: Value) -> Result<Value, String> {
                match command {
                    "parts.seats" => Ok(json!({"seats": [{"pick_index": 0, "origin": self.origin,
                        "point": [0,0,0], "normal": [0,0,1], "x_dir": [1,0,0], "shank_offset": 0}]})),
                    "component.list" => Ok(json!({"pending_moves": self.pending})),
                    _ => panic!("unexpected {command}"),
                }
            }
        }
        let p = crate::screw::parse(
            &json!({"placement": [{"type":"edge", "body":"Block", "index":4}]}),
        )
        .unwrap();
        for (origin, pending, blocked) in [
            (json!({"occurrence": 0}), vec![9], false),
            (json!({"occurrence": 12, "path": [12]}), vec![12], true),
            (json!({"occurrence": 12, "path": [9, 12]}), vec![9], true),
            (json!({"occurrence": 12, "path": [15, 12]}), vec![9], false),
        ] {
            let mut host = Host { origin, pending };
            let error = picked_bores(&mut host, &p, &[], 3.0).err();
            if blocked {
                let error = error.expect("a pending host move must block the screw");
                assert!(error.starts_with("placement: ") && error.contains("Capture Position"));
            } else {
                assert_eq!(error, None);
            }
        }
    }

    #[test]
    fn native_seats_preserve_mixed_order_and_countersink_alignment() {
        struct Bores(Vec<(String, Value)>);
        impl Runner for Bores {
            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                self.0.push((command.into(), args));
                Ok(match command {
                    "parts.seats" => json!({"seats": [
                        {"pick_index": 0, "point": [40, 0, 0], "normal": [1, 0, 0], "x_dir": [0, 1, 0],
                         "shank_offset": -0.25, "origin": {"circle": "wall", "end": "mouth", "occurrence": 12, "path": [9, 12]}}
                    ]}),
                    "component.list" => json!({"pending_moves": []}),
                    "joint.create" => json!({"joint": 1, "errors": [], "conflicts": []}),
                    _ => panic!("unexpected {command}"),
                })
            }
        }
        let edge = json!({"type": "edge", "body": "Rail", "index": 0, "point": [40, 2, 0]});
        let sketch = json!({"type": "external_sketch_point", "sketch": 8, "id": "p1"});
        let spot = Spot {
            pick_index: 0,
            sketch: "Layout".into(),
            point: "p1".into(),
            place: Placement::on_origin(),
        };
        for flip in [false, true] {
            let p = crate::screw::parse(
                &json!({"placement": [edge, sketch, edge], "head": "Countersunk", "flip": flip}),
            )
            .unwrap();
            let mut host = Bores(Vec::new());
            let (places, seats) =
                picked_bores(&mut host, &p, std::slice::from_ref(&spot), 3.0).unwrap();
            assert_eq!(
                host.0[0],
                (
                    "parts.seats".into(),
                    json!({"picks": [edge, edge], "countersunk_shank_diameter": 3.0})
                )
            );
            assert_eq!(places.len(), 2);
            assert_eq!(
                places[0].point,
                [if flip { 40.25 } else { 39.75 }, 0.0, 0.0]
            );
            assert_eq!(places[1], spot.place);
            assert!(seats[1].is_none());
            seats[0]
                .as_ref()
                .unwrap()
                .attach(&mut host, 21, "Screw")
                .unwrap();
            let joint = &host.0[2].1;
            assert_eq!(joint["a"]["end"], "mouth");
            assert_eq!(joint["a"]["path"], json!([9, 12]));
            assert_eq!(joint["b"]["point"], json!([40.0, 0.0, 0.0]));
        }
    }

    #[test]
    fn drilling_requires_sketch_picks() {
        for create in ["Hole only", "Screw and hole"] {
            let err = crate::screw::parse(&json!({"create": create, "placement":[{"type":"face", "body":"Rail", "index":5,"point":[0,0,0]}]})).unwrap_err();
            assert_eq!(err.field, "placement");
        }
    }

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
                        pick_index: 0,
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
                &[(
                    Body {
                        definition: "Plate".into(),
                        path: vec![]
                    },
                    missing
                )],
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
                &[(
                    Body {
                        definition: "Plate".into(),
                        path: vec![]
                    },
                    ambiguous
                )],
                &Placement::on_origin(),
                None,
                &hole,
                3.0
            )
            .is_err());
        }
        assert!(find(
            &[
                (
                    Body {
                        definition: "A".into(),
                        path: vec![]
                    },
                    faces()
                ),
                (
                    Body {
                        definition: "B".into(),
                        path: vec![]
                    },
                    faces()
                )
            ],
            &Placement::on_origin(),
            None,
            &hole,
            3.0
        )
        .is_err());
    }
}
