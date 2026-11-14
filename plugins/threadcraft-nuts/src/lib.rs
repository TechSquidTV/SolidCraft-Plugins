//! ISO 4032 hexagon nut (the size DIN 934 was replaced by) for SolidCraft.
//!
//! One entry, `make-nut`. The bearing face sits on a sketch point, or on the origin when
//! nothing is selected. The nut stands on the outer side of that plane. The body is the
//! same six-sided profile as a hex head. Both ends are chamfered, the bore is tapped, and
//! both ends of the bore are countersunk.

pub mod nut;

use nut::{Params, Pocket};
use serde_json::{json, Value};
use threadcraft::hex::{self, Hex};
use threadcraft::host::{changed_bodies, free_body_name, sketch_id};
use threadcraft::name;
use threadcraft::nut::{self as din, EndChamfer};
use threadcraft::place::{self, opposite, Placement};
use threadcraft::Runner;

/// The body name before any number: `Nut M3`, `Nut M8x1`.
pub fn body_base(p: &Params) -> String {
    format!("Nut {}", p.designation)
}

/// Build the nut. Returns a summary of what was made.
pub fn make_nut<R: Runner>(r: &mut R, p: &Params) -> Result<Value, String> {
    let spots = place::resolve_all(r, &p.place)?;
    let mut drilled: Vec<String> = Vec::new();
    let places: Vec<Placement> = if spots.is_empty() {
        if p.create.pocket() {
            return Err(place::nothing_selected(
                "pockets",
                "Nut only for a loose nut on the origin",
            ));
        }
        vec![Placement::on_origin()]
    } else {
        // The pockets go first: a nut would otherwise be cut by the next one's pocket.
        let sink = match &p.pocket {
            Some(pocket) => {
                for spot in &spots {
                    for body in cut_pocket(r, p, pocket, &spot.place)? {
                        if !drilled.contains(&body) {
                            drilled.push(body);
                        }
                    }
                }
                for (sketch, points) in place::by_sketch(&spots) {
                    let mut hole = json!({
                        "sketch": sketch,
                        "points": points,
                        "name": format!("Bolt hole {}", p.designation),
                    });
                    pocket.clearance.apply(&mut hole);
                    // Through the parts the pockets are in, and nothing beyond them.
                    if !drilled.is_empty() {
                        hole["targets"] = json!(drilled);
                    }
                    r.run("solid.hole", hole)?;
                }
                pocket.depth
            }
            None => 0.0,
        };
        // In a pocket the bearing face is on its floor, so the nut ends flush.
        spots
            .iter()
            .map(|spot| Placement {
                point: spot.place.along(-sink),
                ..spot.place
            })
            .collect()
    };
    let end = din::end_chamfer(p.across_flats, p.chamfer_max);
    if p.create.nut() {
        for place in &places {
            build_nut(r, p, place)?;
        }
    }
    Ok(summary(p, &places, &end, &drilled))
}

/// One hexagonal pocket, cut down from the surface at `place`. Returns the bodies it
/// cut, when the host says.
fn cut_pocket<R: Runner>(
    r: &mut R,
    p: &Params,
    pocket: &Pocket,
    place: &Placement,
) -> Result<Vec<String>, String> {
    let made = hex::extrude_reply(
        r,
        &Hex {
            origin: place.along(-pocket.depth),
            x_dir: place.x_dir,
            y_dir: place.y_dir,
            across_flats: pocket.across_flats,
            height: pocket.depth,
            sketch_name: format!("Nut pocket {} profile", p.designation),
            name: format!("Nut pocket {}", p.designation),
            operation: "cut",
            body_name: None,
            target: None,
        },
    )?;
    match changed_bodies(&made) {
        Some(bodies) if bodies.is_empty() => Err(
            "sketch_point: there is no part under the point for a pocket; put the sketch on the part, or set Create to Nut only"
                .into(),
        ),
        Some(bodies) => Ok(bodies),
        None => Ok(Vec::new()),
    }
}

/// One nut with its bearing face on `place`. Returns the chamfer it was cut with.
fn build_nut<R: Runner>(r: &mut R, p: &Params, place: &Placement) -> Result<EndChamfer, String> {
    let place = *place;
    let body_name = free_body_name(&mut r.body_names(), &body_base(p));
    // Every feature and sketch of this nut starts with its name.
    let named = |feature: &str| name::part(&body_name, feature);
    let made = hex::extrude(
        r,
        &Hex {
            origin: place.point,
            x_dir: place.x_dir,
            y_dir: place.y_dir,
            across_flats: p.across_flats,
            height: p.height,
            sketch_name: named("Hex profile"),
            name: named("Hex"),
            operation: "new",
            body_name: Some(body_name.clone()),
            target: None,
        },
    )?;
    let body = made.ok_or_else(|| "solid.extrude did not name the nut body".to_string())?;
    let end = din::end_chamfer(p.across_flats, p.chamfer_max);
    chamfer(
        r,
        &place,
        &body,
        &end,
        0.0,
        place.normal,
        &named("Bearing chamfer"),
    )?;
    let far = place.along(p.height);
    chamfer(
        r,
        &place,
        &body,
        &end,
        p.height,
        opposite(place.normal),
        &named("Outer chamfer"),
    )?;
    r.run(
        "solid.hole",
        json!({
            "position": place.point,
            "direction": place.normal,
            "depth": p.height,
            "thread": p.thread,
            "modeled": p.modeled,
            "targets": [body],
            "name": named("Thread"),
        }),
    )?;
    if let Some(da) = p.countersink {
        if da > p.major {
            let depth = din::countersink_depth(p.major, da);
            countersink(
                r,
                &body,
                place.point,
                place.normal,
                p.major,
                da,
                depth,
                &named("Bearing countersink"),
            )?;
            countersink(
                r,
                &body,
                far,
                opposite(place.normal),
                p.major,
                da,
                depth,
                &named("Outer countersink"),
            )?;
        }
    }
    Ok(end)
}

/// One conical chamfer. `into` points from that end face into the nut.
fn chamfer<R: Runner>(
    r: &mut R,
    place: &Placement,
    body: &str,
    end: &EndChamfer,
    along: f64,
    into: [f64; 3],
    name: &str,
) -> Result<(), String> {
    let origin = place.along(along);
    let created = r.run(
        "sketch.create",
        json!({
            "name": format!("{name} profile"),
            "plane": {
                "origin": origin,
                "x_dir": place.x_dir,
                "y_dir": into,
            },
        }),
    )?;
    let sketch = sketch_id(&created)?;
    r.run(
        "sketch.line",
        json!({
            "sketch": sketch,
            "closed": true,
            "points": end.triangle,
        }),
    )?;
    r.run("sketch.finish", json!({}))?;
    r.run(
        "solid.revolve",
        json!({
            "sketch": sketch,
            "axis": {"origin": place.point, "dir": place.normal},
            "angle": 360,
            "operation": "cut",
            "targets": [body],
            "name": name,
        }),
    )?;
    Ok(())
}

// Each argument is one dimension or handle the countersink command needs.
#[allow(clippy::too_many_arguments)]
fn countersink<R: Runner>(
    r: &mut R,
    body: &str,
    base: [f64; 3],
    into: [f64; 3],
    major: f64,
    da: f64,
    depth: f64,
    name: &str,
) -> Result<(), String> {
    r.run(
        "solid.cone",
        json!({
            "diameter": da,
            "top_diameter": major,
            "height": depth,
            "base": base,
            "axis": into,
            "operation": "cut",
            "targets": [body],
            "name": name,
        }),
    )?;
    Ok(())
}

fn summary(p: &Params, places: &[Placement], end: &EndChamfer, drilled: &[String]) -> Value {
    let place = &places[0];
    let pocket = p.pocket.as_ref().map(|pocket| {
        json!({
            "across_flats": pocket.across_flats,
            "depth": pocket.depth,
            "bolt_hole": pocket.clearance.describe(),
        })
    });
    json!({
        "standard": "ISO 4032",
        "designation": p.designation,
        "thread": p.thread,
        "major": p.major,
        "pitch": p.pitch,
        "across_flats": p.across_flats,
        "height": p.height,
        "chamfer_axial": end.axial,
        "countersink": p.countersink,
        "modeled": p.modeled,
        "create": p.create.label(),
        "pocket": pocket,
        "drilled": drilled,
        "count": places.len(),
        "points": places.iter().map(|place| place.point).collect::<Vec<_>>(),
        "at": place.point,
        "normal": place.normal,
        "placed": place.source.words(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Rec {
        calls: Vec<(String, Value)>,
        sketch: i64,
    }
    impl Runner for Rec {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            let reply = match command {
                "selection.get" => json!({"count": 0, "items": []}),
                "sketch.create" => {
                    self.sketch += 1;
                    json!({"sketch": self.sketch})
                }
                "solid.extrude" => json!({"created_bodies": ["Nut"]}),
                _ => Value::Null,
            };
            self.calls.push((command.to_string(), args));
            Ok(reply)
        }
    }

    fn commands(v: Value) -> (Value, Vec<(String, Value)>) {
        let p = nut::parse(&v).unwrap();
        let mut rec = Rec {
            calls: vec![],
            sketch: 0,
        };
        let summary = make_nut(&mut rec, &p).unwrap();
        (summary, rec.calls)
    }

    fn call<'a>(calls: &'a [(String, Value)], command: &str) -> &'a Value {
        &calls.iter().find(|(name, _)| name == command).unwrap().1
    }

    fn named<'a>(calls: &'a [(String, Value)], command: &str, name: &str) -> &'a Value {
        &calls
            .iter()
            .find(|(c, args)| c == command && args["name"] == name)
            .unwrap()
            .1
    }

    fn last<'a>(calls: &'a [(String, Value)], command: &str) -> &'a Value {
        &calls.iter().rfind(|(name, _)| name == command).unwrap().1
    }

    #[test]
    fn an_m6_nut_is_a_hex_two_chamfers_a_thread_and_two_countersinks() {
        let (summary, calls) = commands(json!({"size": "M6"}));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "sketch.create",
                "sketch.polygon.circumscribed",
                "sketch.dimension",
                "sketch.constraint.horizontal_vertical",
                "sketch.finish",
                "solid.extrude",
                "sketch.create",
                "sketch.line",
                "sketch.finish",
                "solid.revolve",
                "sketch.create",
                "sketch.line",
                "sketch.finish",
                "solid.revolve",
                "solid.hole",
                "solid.cone",
                "solid.cone",
            ]
        );
        assert_eq!(calls[2].1["radius"], 5.0);
        assert_eq!(calls[2].1["sides"], 6);
        assert_eq!(calls[6].1["distance"], 5.2);
        assert_eq!(calls[6].1["operation"], "new");
        assert_eq!(calls[6].1["body_name"], "Nut M6");
        assert_eq!(calls[6].1["name"], "Nut M6: Hex");
        assert_eq!(calls[6].1["sketch"], 1);
        let bearing = &calls[7].1;
        assert_eq!(bearing["plane"]["origin"], json!([0.0, 0.0, 0.0]));
        assert_eq!(bearing["plane"]["y_dir"], json!([0.0, 0.0, 1.0]));
        let outer = &calls[11].1;
        assert_eq!(outer["plane"]["origin"], json!([0.0, 0.0, 5.2]));
        assert_eq!(outer["plane"]["y_dir"], json!([0.0, 0.0, -1.0]));
        let triangle = &calls[8].1["points"];
        assert_eq!(triangle[0][0], calls[12].1["points"][0][0]);
        assert_eq!(calls[10].1["operation"], "cut");
        assert_eq!(calls[10].1["targets"], json!(["Nut"]));
        assert_eq!(calls[10].1["sketch"], 2);
        assert_eq!(calls[10].1["angle"], 360);
        assert_eq!(calls[14].1["sketch"], 3);
        let hole = call(&calls, "solid.hole");
        assert_eq!(hole["position"], json!([0.0, 0.0, 0.0]));
        assert_eq!(hole["direction"], json!([0.0, 0.0, 1.0]));
        assert_eq!(hole["depth"], 5.2);
        assert_eq!(hole["thread"], "M6");
        assert_eq!(hole["modeled"], true);
        assert_eq!(hole["targets"], json!(["Nut"]));
        let near = call(&calls, "solid.cone");
        assert_eq!(near["diameter"], 6.75);
        assert_eq!(near["top_diameter"], 6.0);
        assert_eq!(near["base"], json!([0.0, 0.0, 0.0]));
        assert_eq!(near["axis"], json!([0.0, 0.0, 1.0]));
        assert_eq!(near["operation"], "cut");
        let far = last(&calls, "solid.cone");
        assert_eq!(far["base"], json!([0.0, 0.0, 5.2]));
        assert_eq!(far["axis"], json!([0.0, 0.0, -1.0]));
        assert_eq!(summary["standard"], "ISO 4032");
        assert_eq!(summary["across_flats"], 10.0);
        assert_eq!(summary["height"], 5.2);
        assert_eq!(summary["at"], json!([0.0, 0.0, 0.0]));
        assert_eq!(summary["normal"], json!([0.0, 0.0, 1.0]));
        assert_eq!(
            summary["placed"],
            "on the origin, because nothing was selected"
        );
    }

    #[test]
    fn an_unlisted_size_has_no_countersink() {
        let (_, calls) = commands(json!({
            "designation": "M7x1",
            "custom_width": 11.0,
            "custom_height": 5.5,
            "thread_modeled": true,
        }));
        assert!(calls.iter().all(|(name, _)| name != "solid.cone"));
        let hole = call(&calls, "solid.hole");
        assert_eq!(hole["thread"], "M7x1");
        assert_eq!(hole["modeled"], true);
        assert_eq!(hole["depth"], 5.5);
    }

    #[test]
    fn a_renamed_body_is_the_one_the_cuts_use() {
        struct Renamed {
            calls: Vec<(String, Value)>,
            sketch: i64,
        }
        impl Runner for Renamed {
            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                let reply = match command {
                    "selection.get" => json!({"count": 0, "items": []}),
                    "sketch.create" => {
                        self.sketch += 1;
                        json!({"sketch": self.sketch})
                    }
                    "solid.extrude" => json!({"created_bodies": ["Body1"]}),
                    _ => Value::Null,
                };
                self.calls.push((command.to_string(), args));
                Ok(reply)
            }
        }
        let p = nut::parse(&json!({})).unwrap();
        let mut rec = Renamed {
            calls: vec![],
            sketch: 0,
        };
        make_nut(&mut rec, &p).unwrap();
        assert_eq!(call(&rec.calls, "solid.hole")["targets"], json!(["Body1"]));
        assert_eq!(
            call(&rec.calls, "solid.revolve")["targets"],
            json!(["Body1"])
        );
        assert_eq!(last(&rec.calls, "solid.cone")["targets"], json!(["Body1"]));
    }

    #[test]
    fn a_body_without_a_name_is_not_chamfered() {
        struct Mute(Vec<String>);
        impl Runner for Mute {
            fn run(&mut self, command: &str, _args: Value) -> Result<Value, String> {
                self.0.push(command.to_string());
                Ok(match command {
                    "selection.get" => json!({"count": 0, "items": []}),
                    "sketch.create" => json!({"sketch": 1}),
                    _ => Value::Null,
                })
            }
        }
        let p = nut::parse(&json!({})).unwrap();
        let mut rec = Mute(vec![]);
        let err = make_nut(&mut rec, &p).unwrap_err();
        assert_eq!(err, "solid.extrude did not name the nut body");
        assert_eq!(
            rec.0,
            [
                "selection.get",
                "sketch.create",
                "sketch.polygon.circumscribed",
                "sketch.dimension",
                "sketch.constraint.horizontal_vertical",
                "sketch.finish",
                "solid.extrude",
            ]
        );
    }

    #[test]
    fn a_named_point_puts_the_bearing_face_there() {
        struct Named(Vec<(String, Value)>);
        impl Runner for Named {
            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                let reply = match command {
                    "sketch.inspect" => json!({
                        "plane": {
                            "origin": [0.0, 0.0, 8.0],
                            "x_dir": [1.0, 0.0, 0.0],
                            "y_dir": [0.0, 1.0, 0.0]
                        },
                        "points": [{"id": "c1.center", "world": [12.0, 10.0, 8.0]}],
                        "curves": [{"id": "c1", "center": "c1.center"}]
                    }),
                    "sketch.create" => json!({"sketch": 4}),
                    "solid.extrude" => json!({"created_bodies": ["Nut"]}),
                    _ => Value::Null,
                };
                self.0.push((command.to_string(), args));
                Ok(reply)
            }
        }
        let p =
            nut::parse(&json!({"size": "M6", "sketch": "Holes", "sketch_point": "c1"})).unwrap();
        let mut rec = Named(vec![]);
        let summary = make_nut(&mut rec, &p).unwrap();
        assert_eq!(rec.0[0].0, "sketch.inspect");
        assert_eq!(rec.0[0].1["sketch"], "Holes");
        assert_eq!(call(&rec.0, "solid.extrude")["distance"], 5.2);
        let hole = call(&rec.0, "solid.hole");
        assert_eq!(hole["position"], json!([12.0, 10.0, 8.0]));
        assert_eq!(hole["direction"], json!([0.0, 0.0, 1.0]));
        assert_eq!(
            last(&rec.0, "solid.cone")["base"],
            json!([12.0, 10.0, 13.2])
        );
        assert_eq!(summary["at"], json!([12.0, 10.0, 8.0]));
        assert_eq!(summary["placed"], "on the named sketch point");
    }

    #[test]
    fn the_manifest_asks_for_a_size() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(manifest["id"], "threadcraft-nuts");
        assert_eq!(manifest["name"], "Hex nuts");
        let description = manifest["description"].as_str().unwrap();
        assert!(description.chars().count() <= 240, "{description}");
        let entry = &manifest["runtime"]["entries"][0];
        assert_eq!(entry["name"], "make-nut");
        // The menu row, the dialog title and the status line all use this name.
        assert_eq!(entry["slots"], json!([{"kind": "menu", "name": "Hex nut"}]));
        let schema = &entry["input_schema"];
        assert_eq!(
            schema["required"],
            json!(["size", "create", "pocket_adjust"])
        );
        assert_eq!(schema["properties"]["create"]["default"], "Nut only");
        assert_eq!(
            schema["properties"]["pocket_adjust"]["default"],
            nut::DEFAULT_POCKET_ADJUST
        );
        let props = &schema["properties"];
        assert_eq!(props["size"]["default"], nut::DEFAULT_DESIGNATION);
        assert!(props["size"]["enum"]
            .as_array()
            .unwrap()
            .contains(&json!(nut::DEFAULT_DESIGNATION)));
        assert_eq!(props["thread_modeled"]["default"], true);
        // The dialog lists optional fields by key, so the keys are the order.
        let mut optional: Vec<&str> = props
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .filter(|key| !["size", "create", "pocket_adjust"].contains(key))
            .collect();
        optional.sort();
        assert_eq!(
            optional,
            [
                "custom_size",
                "custom_thickness",
                "custom_width",
                "sketch",
                "sketch_point",
                "thread_modeled"
            ]
        );
        assert_eq!(
            manifest["runtime"]["commands"],
            json!([
                "selection.get",
                "inspect.measure",
                "sketch.inspect",
                "sketch.create",
                "sketch.polygon.circumscribed",
                "sketch.dimension",
                "sketch.constraint.horizontal_vertical",
                "sketch.line",
                "sketch.finish",
                "solid.extrude",
                "solid.revolve",
                "solid.hole",
                "solid.cone"
            ])
        );
    }

    /// A design that already holds bodies. It records the body name each new body asks for.
    struct Taken {
        names: Vec<String>,
        asked: Vec<String>,
        sketch: i64,
    }
    impl Runner for Taken {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            Ok(match command {
                "selection.get" => json!({"count": 0, "items": []}),
                "sketch.create" => {
                    self.sketch += 1;
                    json!({"sketch": self.sketch})
                }
                "sketch.line" => json!({"created": {"curves": ["l1"]}}),
                "sketch.inspect" => json!({
                    "plane": {"x_dir": [1.0, 0.0, 0.0], "y_dir": [0.0, 1.0, 0.0]},
                    "points": [
                        {"id": "a", "world": [10.0, 0.0, 5.0]},
                        {"id": "b", "world": [20.0, 0.0, 5.0]},
                    ],
                    "curves": [],
                }),
                "solid.hole" if args.get("sketch").is_some() => {
                    json!({"changed_bodies": ["Base"]})
                }
                _ if args.get("operation").and_then(Value::as_str) == Some("new")
                    && args.get("body_name").is_some() =>
                {
                    let name = args["body_name"].as_str().unwrap().to_string();
                    self.asked.push(name.clone());
                    self.names.push(name.clone());
                    json!({"created_bodies": [name]})
                }
                _ => Value::Null,
            })
        }

        fn body_names(&mut self) -> Vec<String> {
            self.names.clone()
        }
    }

    fn design(names: &[&str]) -> Taken {
        Taken {
            names: names.iter().map(|n| n.to_string()).collect(),
            asked: vec![],
            sketch: 0,
        }
    }

    /// A taken name makes the host number the body, and that number changes on a rebuild.
    #[test]
    fn a_second_nut_asks_for_a_free_body_name() {
        let p = nut::parse(&json!({})).unwrap();
        let mut host = design(&["Base", "Nut M3"]);
        make_nut(&mut host, &p).unwrap();
        make_nut(&mut host, &p).unwrap();
        assert_eq!(host.asked, ["Nut M3 (2)", "Nut M3 (3)"]);
    }

    #[test]
    fn a_body_name_is_the_size() {
        let base = |v: Value| body_base(&nut::parse(&v).unwrap());
        assert_eq!(base(json!({})), "Nut M3");
        assert_eq!(base(json!({"custom_size": "M8x1"})), "Nut M8x1");
    }

    #[test]
    fn every_feature_of_a_nut_starts_with_its_body_name() {
        let p = nut::parse(&json!({"size": "M6"})).unwrap();
        let mut rec = Rec {
            calls: vec![],
            sketch: 0,
        };
        make_nut(&mut rec, &p).unwrap();
        for (command, args) in &rec.calls {
            if let Some(name) = args.get("name").and_then(Value::as_str) {
                assert!(name.starts_with("Nut M6: "), "{command} is named {name}");
            }
        }
    }

    /// Two selected circles on a sketch on the top face of a plate.
    struct OnPlate {
        calls: Vec<(String, Value)>,
        sketch: i64,
        changed: Value,
    }
    impl Runner for OnPlate {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            let reply = match command {
                "selection.get" => json!({"count": 2, "items": [
                    {"kind": "sketch_curve", "entity": "circle", "id": "c1", "sketch_name": "Holes"},
                    {"kind": "sketch_curve", "entity": "circle", "id": "c2", "sketch_name": "Holes"}
                ]}),
                "sketch.inspect" => json!({
                    "plane": {"origin": [0.0, 0.0, 10.0], "x_dir": [1.0, 0.0, 0.0], "y_dir": [0.0, 1.0, 0.0]},
                    "points": [
                        {"id": "c1.center", "world": [10.0, 10.0, 10.0]},
                        {"id": "c2.center", "world": [30.0, 10.0, 10.0]}
                    ],
                    "curves": [{"id": "c1", "center": "c1.center"}, {"id": "c2", "center": "c2.center"}]
                }),
                "sketch.create" => {
                    self.sketch += 1;
                    json!({"sketch": self.sketch})
                }
                "solid.extrude" if args["operation"] == "cut" => {
                    json!({"changed_bodies": self.changed})
                }
                "solid.extrude" => json!({"created_bodies": [args["body_name"]]}),
                _ => Value::Null,
            };
            self.calls.push((command.to_string(), args));
            Ok(reply)
        }
    }

    fn on_plate(v: Value, changed: Value) -> (Result<Value, String>, Vec<(String, Value)>) {
        let p = nut::parse(&v).unwrap();
        let mut rec = OnPlate {
            calls: vec![],
            sketch: 0,
            changed,
        };
        let made = make_nut(&mut rec, &p);
        (made, rec.calls)
    }

    #[test]
    fn two_selected_circles_get_two_nuts_and_no_pocket_by_default() {
        let (summary, calls) = on_plate(json!({}), json!(["Plate"]));
        let summary = summary.unwrap();
        assert!(calls
            .iter()
            .all(|(c, args)| !(c == "solid.extrude" && args["operation"] == "cut")));
        assert_eq!(summary["count"], 2);
        assert_eq!(summary["create"], "Nut only");
        assert_eq!(summary["pocket"], Value::Null);
        assert_eq!(summary["points"][1], json!([30.0, 10.0, 10.0]));
    }

    #[test]
    fn nut_and_pocket_cuts_both_pockets_and_a_bolt_hole_then_sinks_the_nuts() {
        let (summary, calls) = on_plate(
            json!({"size": "M6", "create": "Nut and pocket"}),
            json!(["Plate"]),
        );
        let summary = summary.unwrap();
        let cuts: Vec<&Value> = calls
            .iter()
            .filter(|(c, args)| c == "solid.extrude" && args["operation"] == "cut")
            .map(|(_, args)| args)
            .collect();
        assert_eq!(cuts.len(), 2);
        assert_eq!(cuts[0]["distance"], 5.2);
        assert_eq!(cuts[0]["name"], "Nut pocket M6");
        assert!(cuts[0].get("targets").is_none());
        // The pocket sketch is on the pocket floor, 5.2 below the 10 mm surface, and
        // 10.4 across: the nut and 0.2 on each flat.
        let floor = named(&calls, "sketch.create", "Nut pocket M6 profile");
        assert!((floor["plane"]["origin"][2].as_f64().unwrap() - 4.8).abs() < 1e-9);
        assert!(
            (call(&calls, "sketch.polygon.circumscribed")["radius"]
                .as_f64()
                .unwrap()
                - 5.2)
                .abs()
                < 1e-9
        );
        // One bolt hole for both points, through the plate the pockets are in.
        let bolt = &calls
            .iter()
            .find(|(c, args)| c == "solid.hole" && args["name"] == "Bolt hole M6")
            .unwrap()
            .1;
        assert_eq!(bolt["clearance"], "M6");
        assert_eq!(bolt["points"], json!(["c1.center", "c2.center"]));
        assert_eq!(bolt["targets"], json!(["Plate"]));
        assert!(bolt.get("depth").is_none());
        // Every pocket and the bolt hole come before the first nut.
        let first_nut = calls
            .iter()
            .position(|(c, args)| c == "solid.extrude" && args["operation"] == "new")
            .unwrap();
        let last_cut = calls
            .iter()
            .rposition(|(c, args)| {
                (c == "solid.extrude" && args["operation"] == "cut")
                    || args["name"] == "Bolt hole M6"
            })
            .unwrap();
        assert!(last_cut < first_nut);
        // The bearing face is on the pocket floor, so the nut ends flush.
        assert!((summary["points"][0][2].as_f64().unwrap() - 4.8).abs() < 1e-9);
        assert_eq!(summary["drilled"], json!(["Plate"]));
        assert_eq!(summary["pocket"]["bolt_hole"], "M6");
    }

    #[test]
    fn a_pocket_needs_a_selection_and_a_part() {
        let p = nut::parse(&json!({"create": "Pocket only"})).unwrap();
        let mut rec = Rec {
            calls: vec![],
            sketch: 0,
        };
        let err = make_nut(&mut rec, &p).unwrap_err();
        assert!(
            err.starts_with("sketch_point: select the sketch points"),
            "{err}"
        );
        assert!(err.contains("Nut only"), "{err}");
        let (made, _) = on_plate(json!({"create": "Pocket only"}), json!([]));
        assert!(made
            .unwrap_err()
            .starts_with("sketch_point: there is no part"));
        let (made, calls) = on_plate(json!({"create": "Pocket only"}), json!(["Plate"]));
        assert!(made.is_ok());
        assert!(calls
            .iter()
            .all(|(c, args)| !(c == "solid.extrude" && args["operation"] == "new")));
    }
}

threadcraft::guest! {
    entry: "make-nut",
    noun: "nut",
    manifest: include_str!("../plugin.json"),
    parse: crate::nut::parse,
    make: crate::make_nut,
    describe: |p: &crate::nut::Params| format!("making a {} nut", p.designation),
    aliases: &[],
}
