//! ISO 4032 hexagon nut (the size DIN 934 was replaced by) for SolidCraft.
//!
//! One entry, `make-nut`. The bearing face sits on a sketch point, or on the origin when
//! nothing is selected. The nut stands on the outer side of that plane. The body is the
//! same six-sided profile as a hex head. Both ends are chamfered, the bore is tapped, and
//! both ends of the bore are countersunk.

pub mod nut;
mod seat;

use fasteners::hex::{self, Hex};
use fasteners::nut::{self as din, EndChamfer};
use fasteners::FOLDER;
use nut::{Params, Pocket};
use plugin_kit::component::{self as fasten, Definition, Placed, Site};
use plugin_kit::host::{add_new, changed_bodies, free_body_name, sketch_id};
use plugin_kit::name;
use plugin_kit::place::{self, opposite, Placement};
use plugin_kit::timeline;
use plugin_kit::Runner;
use serde_json::{json, Value};

/// The body name before any number: `Nut M3`, `Nut M8x1`.
pub fn body_base(p: &Params) -> String {
    format!("Nut {}", p.designation)
}

/// Build the nut. Returns a summary of what was made.
pub fn make_nut<R: Runner>(r: &mut R, p: &Params) -> Result<Value, String> {
    let spots = place::resolve_all(r, &p.place)?;
    let mut drilled: Vec<String> = Vec::new();
    // Nut and pocket with nothing selected still makes the nut. Only a run that would
    // make nothing but a pocket is refused.
    let mut skipped = false;
    let places: Vec<Placement> = if spots.is_empty() {
        skipped = nut::CREATE.nothing_selected(p.create)?;
        vec![Placement::on_origin()]
    } else {
        // The pockets go first: a nut would otherwise be cut by the next one's pocket.
        let sink = match &p.pocket {
            Some(pocket) => {
                // A sketch and a cut for every pocket and a bolt hole for every sketch:
                // one folder in the part's timeline.
                drilled = timeline::folded(r, &pockets_name(p), |mut r| {
                    let mut drilled: Vec<String> = Vec::new();
                    for (sketch, _) in place::by_sketch(&spots) {
                        let plane = pocket_plane(&mut r, sketch, &p.designation)?;
                        for spot in spots.iter().filter(|spot| spot.sketch == sketch) {
                            add_new(&mut drilled, cut_pocket(&mut r, p, pocket, spot, &plane)?);
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
                    Ok(drilled)
                })?;
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
    // Resolve the finished pockets before adding hardware, so the nut's own bore
    // cannot be mistaken for the bolt hole it sits over.
    let seats = if p.create.part() && !spots.is_empty() {
        match &p.pocket {
            Some(pocket) => seat::Seat::find_all(r, &drilled, &places, pocket, p.major)?,
            None => Vec::new(),
        }
    } else {
        Vec::new()
    };
    let end = din::end_chamfer(p.across_flats, p.chamfer_max);
    let placed = if p.create.part() {
        let base = body_base(p);
        let def = Definition {
            plugin: PLUGIN,
            base: &base,
            description: "ISO 4032",
            folder: FOLDER,
            bom_type: "purchased",
            site: Site::for_thread(p.modeled),
        };
        let placed = fasten::place_all(r, &def, &places, |mut r, name, site| {
            build_nut(&mut r, p, site, name).map(|_| ())
        })?;
        for ((seat, place), occurrence) in seats.iter().zip(&places).zip(&placed.occurrences) {
            seat.attach(r, *occurrence, place, &p.designation)?;
        }
        Some(placed)
    } else {
        None
    };
    Ok(summary(
        p,
        &places,
        &end,
        &drilled,
        skipped,
        placed.as_ref(),
    ))
}

/// The timeline folder one run's pockets and bolt holes go in: `Nut pockets M6`.
pub fn pockets_name(p: &Params) -> String {
    format!("Nut pockets {}", p.designation)
}

/// Keep the pocket plane tied to the source sketch's axes. Construction lines do
/// not split its profiles, and their ends are not standalone hole-placement points.
fn pocket_plane<R: Runner>(r: &mut R, sketch: &str, designation: &str) -> Result<Value, String> {
    // Construction planes reject duplicate names, unlike auto-numbered sketches.
    // A later run (or another source sketch) needs a free name too.
    let listed = r.run("component.list", json!({}))?;
    let mut names = listed
        .get("components")
        .and_then(Value::as_array)
        .ok_or("component.list did not return components for the pocket plane name")?
        .iter()
        .flat_map(|c| {
            ["features", "sketches", "construction"]
                .into_iter()
                .flat_map(move |key| c.get(key).and_then(Value::as_array).into_iter().flatten())
        })
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    let name = free_body_name(&mut names, &format!("Nut pockets {designation}: Plane"));
    r.run("sketch.edit", json!({"sketch": sketch}))?;
    let drawn = r.run(
        "sketch.line",
        json!({
            "points": [[0, 0], [1, 0], [1, 1]], "construction": true, "connect": false,
        }),
    )?;
    let curves = drawn
        .get("curves")
        .and_then(Value::as_array)
        .filter(|ids| {
            ids.len() == 2
                && ids
                    .iter()
                    .all(|id| id.as_str().is_some_and(|id| !id.is_empty()))
        })
        .ok_or("sketch.line did not return the two pocket frame lines")?;
    r.run("sketch.constraint.fix", json!({"entities": curves}))?;
    r.run("sketch.finish", json!({}))?;
    let made = r.run(
        "construct.plane.two_edges",
        json!({
            "a": {"sketch": sketch, "curve": curves[0]},
            "b": {"sketch": sketch, "curve": curves[1]},
            "name": name,
        }),
    )?;
    let plane = made
        .get("feature")
        .filter(|id| id.as_u64().is_some())
        .ok_or("construct.plane.two_edges did not return a plane id")?
        .clone();
    r.run(
        "browser.visibility",
        json!({"items": [plane], "visible": false}),
    )?;
    Ok(plane)
}

/// A pocket starts on the source plane and cuts inward. Its center is projected
/// from the same point used by the bolt hole, never copied from world coordinates.
fn cut_pocket<R: Runner>(
    r: &mut R,
    p: &Params,
    pocket: &Pocket,
    spot: &place::Spot,
    plane: &Value,
) -> Result<Vec<String>, String> {
    let created = r.run(
        "sketch.create",
        json!({
            "plane": plane, "name": format!("Nut pocket {} profile", p.designation),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    let projected = r.run(
        "sketch.project",
        json!({
            "refs": [{"sketch": spot.sketch, "point": spot.point}], "projection_link": true,
        }),
    )?;
    let points = projected
        .get("created")
        .and_then(|v| v.get("points"))
        .and_then(Value::as_array)
        .filter(|points| points.len() == 1)
        .ok_or("sketch.project did not return one pocket center")?;
    let center = points[0]
        .as_str()
        .filter(|id| !id.is_empty())
        .ok_or("sketch.project did not name the pocket center")?;
    hex::draw_profile(r, pocket.across_flats, Some(center))?;
    r.run("sketch.finish", json!({}))?;
    let made = r.run(
        "solid.extrude",
        json!({
            "sketch": sketch, "distance": pocket.depth, "direction": "negative",
            "operation": "cut", "name": format!("Nut pocket {}", p.designation),
        }),
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

/// The plugin id, which a nut component's metadata names.
pub const PLUGIN: &str = "techsquid-nuts";

/// One nut named `body_name` with its bearing face on `site`, whose axes are the
/// world's. Returns the chamfer it was cut with.
///
/// The hexagon and its chamfers are cut on the origin and the blank is then moved to the
/// site, where the hole is drilled and tapped. The kernel refuses the chamfer of some
/// sizes away from the origin, and the tapped hole must be away from it: see
/// [`plugin_kit::component`].
fn build_nut<R: Runner>(
    r: &mut R,
    p: &Params,
    site: &Placement,
    body_name: &str,
) -> Result<EndChamfer, String> {
    let place = Placement {
        point: [0.0, 0.0, 0.0],
        ..*site
    };
    // Every feature and sketch of this nut starts with its name.
    let named = |feature: &str| name::part(body_name, feature);
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
            body_name: Some(body_name.to_string()),
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
    chamfer(
        r,
        &place,
        &body,
        &end,
        p.height,
        opposite(place.normal),
        &named("Outer chamfer"),
    )?;
    if site.point != place.point {
        r.run(
            "solid.move",
            json!({"bodies": [body], "translate": site.point, "name": named("Move to site")}),
        )?;
    }
    let place = *site;
    let far = place.along(p.height);
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

fn summary(
    p: &Params,
    places: &[Placement],
    end: &EndChamfer,
    drilled: &[String],
    skipped: bool,
    placed: Option<&Placed>,
) -> Value {
    let pocket = p.pocket.as_ref().filter(|_| !skipped).map(|pocket| {
        json!({
            "across_flats": pocket.across_flats,
            "depth": pocket.depth,
            "bolt_hole": pocket.clearance.describe(),
        })
    });
    let reply = json!({
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
        "create": nut::CREATE.label(p.create),
        "pocket": pocket,
        "drilled": drilled,
        "note": skipped.then(|| nut::CREATE.skipped()),
    });
    fasten::with_placements(reply, places, placed)
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
        fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
            plugin_kit::Placement::on_origin()
        }

        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            // Placing the fastener is checked on its own; these check the build.
            if let Some(reply) = plugin_kit::component::stand_in(command) {
                return Ok(reply);
            }
            let reply = match command {
                "selection.get" => json!({"count": 0, "items": []}),
                "sketch.create" => {
                    self.sketch += 1;
                    json!({"sketch": self.sketch})
                }
                "solid.extrude" => json!({"created_bodies": ["Nut"]}),
                _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
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

    /// On a site the blank is cut on the origin, moved, and tapped where it lands.
    #[test]
    fn on_a_site_the_blank_is_cut_on_the_origin_and_tapped_on_the_site() {
        let p = nut::parse(&json!({"size": "M6"})).unwrap();
        let mut rec = Rec {
            calls: vec![],
            sketch: 0,
        };
        let site = plugin_kit::component::site_at([5.0, 25.0]);
        build_nut(&mut rec, &p, &site, "Nut M6").unwrap();
        let names: Vec<&str> = rec.calls.iter().map(|(c, _)| c.as_str()).collect();
        let moved = names.iter().position(|c| *c == "solid.move").unwrap();
        let tapped = names.iter().position(|c| *c == "solid.hole").unwrap();
        // Both chamfers are revolved before the move, and the hole comes after it.
        assert_eq!(
            names[..moved]
                .iter()
                .filter(|c| **c == "solid.revolve")
                .count(),
            2
        );
        assert!(moved < tapped);
        assert_eq!(rec.calls[0].1["plane"]["origin"], json!([0.0, 0.0, 0.0]));
        assert_eq!(rec.calls[moved].1["translate"], json!([5.0, 25.0, 0.0]));
        assert_eq!(rec.calls[tapped].1["position"], json!([5.0, 25.0, 0.0]));
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert!(manifest["runtime"]["commands"]
            .as_array()
            .unwrap()
            .contains(&json!("solid.move")));
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
            fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
                plugin_kit::Placement::on_origin()
            }

            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                // Placing the fastener is checked on its own; these check the build.
                if let Some(reply) = plugin_kit::component::stand_in(command) {
                    return Ok(reply);
                }
                let reply = match command {
                    "selection.get" => json!({"count": 0, "items": []}),
                    "sketch.create" => {
                        self.sketch += 1;
                        json!({"sketch": self.sketch})
                    }
                    "solid.extrude" => json!({"created_bodies": ["Body1"]}),
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
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
            fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
                plugin_kit::Placement::on_origin()
            }

            fn run(&mut self, command: &str, _args: Value) -> Result<Value, String> {
                // Placing the fastener is checked on its own; these check the build.
                if let Some(reply) = plugin_kit::component::stand_in(command) {
                    return Ok(reply);
                }
                self.0.push(command.to_string());
                Ok(match command {
                    "selection.get" => json!({"count": 0, "items": []}),
                    "sketch.create" => json!({"sketch": 1}),
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
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
            fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
                plugin_kit::Placement::on_origin()
            }

            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                // Placing the fastener is checked on its own; these check the build.
                if let Some(reply) = plugin_kit::component::stand_in(command) {
                    return Ok(reply);
                }
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
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
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
        // The nut is built in its own component, its bearing face on the origin.
        let hole = call(&rec.0, "solid.hole");
        assert_eq!(hole["position"], json!([0.0, 0.0, 0.0]));
        assert_eq!(hole["direction"], json!([0.0, 0.0, 1.0]));
        assert_eq!(last(&rec.0, "solid.cone")["base"], json!([0.0, 0.0, 5.2]));
        assert_eq!(summary["at"], json!([12.0, 10.0, 8.0]));
        assert_eq!(summary["placed"], "on the named sketch point");
    }

    #[test]
    fn the_manifest_asks_for_a_size() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(manifest["id"], "techsquid-nuts");
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
                "sketch.edit",
                "sketch.project",
                "sketch.constraint.fix",
                "sketch.constraint.coincident",
                "construct.plane.two_edges",
                "browser.visibility",
                "sketch.polygon.circumscribed",
                "sketch.dimension",
                "sketch.constraint.horizontal_vertical",
                "sketch.line",
                "sketch.finish",
                "solid.extrude",
                "solid.revolve",
                "solid.hole",
                "solid.move",
                "solid.cone",
                "component.list",
                "component.create",
                "component.metadata",
                "component.metadata.set",
                "component.activate",
                "occurrence.copy",
                "occurrence.move",
                "occurrence.reparent",
                "timeline.group",
                "model.faces",
                "joint.create"
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
        fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
            plugin_kit::Placement::on_origin()
        }

        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            // Placing the fastener is checked on its own; these check the build.
            if let Some(reply) = plugin_kit::component::stand_in(command) {
                return Ok(reply);
            }
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
                _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
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
        fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
            plugin_kit::Placement::on_origin()
        }

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
                "sketch.line" => json!({"curves": ["frame_x", "frame_y"]}),
                "construct.plane.two_edges" => json!({"feature": 100 + self.calls.len()}),
                "sketch.project" => json!({"created": {"points": ["projected_center"]}}),
                "sketch.create" => {
                    self.sketch += 1;
                    json!({"sketch": self.sketch})
                }
                "model.faces" => json!({"faces": ([10.0, 30.0].into_iter().flat_map(|x| [
                    json!({"name": format!("floor{x}"), "centroid": [x, 10.0, 4.8], "plane_normal": [0, 0, 1]}),
                    json!({"name": format!("wall{x}"), "centroid": [x, 4.8, 7.4], "plane_normal": [0, 1, 0]}),
                    json!({"name": format!("bore{x}"), "centroid": [x, 10.0, 2.4], "cylinder": {
                        "origin": [x, 10.0, 0], "axis": [0, 0, 1], "radius": 3.3
                    }}),
                ]).collect::<Vec<_>>())}),
                "joint.create" => json!({"joint": 999, "errors": [], "conflicts": []}),
                // A cut and a bolt hole are numbered features, as in SolidCraft.
                "solid.extrude" if args["operation"] == "cut" => {
                    json!({"changed_bodies": self.changed, "feature": 100 + self.calls.len()})
                }
                "solid.hole" if args.get("sketch").is_some() => {
                    json!({"feature": 100 + self.calls.len()})
                }
                "solid.extrude" => json!({"created_bodies": [args["body_name"]]}),
                _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
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
    fn a_runs_pockets_and_bolt_holes_fold_into_one_folder() {
        let (made, calls) = on_plate(
            json!({"size": "M6", "create": "Nut and pocket"}),
            json!(["Plate"]),
        );
        made.unwrap();
        let folds: Vec<&Value> = calls
            .iter()
            .filter(|(c, args)| c == "timeline.group" && args["name"] == "Nut pockets M6")
            .map(|(_, args)| args)
            .collect();
        assert_eq!(folds.len(), 1);
        // The linked plane, both pocket sketches, both cuts and the bolt hole. The
        // plate numbers sketches 1, 2, … and features by their place in the calls.
        let mut sketches = 0;
        let mut rows: Vec<u64> = Vec::new();
        for (i, (c, args)) in calls.iter().enumerate() {
            match c.as_str() {
                "sketch.create" => {
                    sketches += 1;
                    if args["name"] == "Nut pocket M6 profile" {
                        rows.push(sketches);
                    }
                }
                "construct.plane.two_edges" => rows.push(100 + i as u64),
                "solid.extrude" if args["operation"] == "cut" => rows.push(100 + i as u64),
                "solid.hole" if args.get("sketch").is_some() => rows.push(100 + i as u64),
                _ => {}
            }
        }
        assert_eq!(rows.len(), 6);
        assert_eq!(folds[0]["features"], json!(rows));
        // The folder is made before the nut is built.
        let fold = calls
            .iter()
            .position(|(c, _)| c == "timeline.group")
            .unwrap();
        let nut = calls
            .iter()
            .position(|(c, _)| c == "component.create")
            .unwrap();
        assert!(fold < nut);
        // Nut only cuts nothing and folds nothing in the part.
        let (_, calls) = on_plate(json!({"size": "M6"}), json!(["Plate"]));
        assert!(!calls
            .iter()
            .any(|(c, args)| c == "timeline.group" && args["name"] == "Nut pockets M6"));
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
        // Both profiles follow the source frame and cut inward from its surface.
        assert_eq!(cuts[0]["direction"], "negative");
        assert_eq!(cuts[1]["direction"], "negative");
        let plane_at = calls
            .iter()
            .position(|(c, _)| c == "construct.plane.two_edges")
            .unwrap();
        let plane = &calls[plane_at].1;
        assert_eq!(plane["a"], json!({"sketch": "Holes", "curve": "frame_x"}));
        assert_eq!(plane["b"], json!({"sketch": "Holes", "curve": "frame_y"}));
        let profile = named(&calls, "sketch.create", "Nut pocket M6 profile");
        assert_eq!(profile["plane"], 100 + plane_at);
        let projected: Vec<&Value> = calls
            .iter()
            .filter(|(c, _)| c == "sketch.project")
            .map(|(_, a)| a)
            .collect();
        assert_eq!(
            projected[0]["refs"],
            json!([{"sketch": "Holes", "point": "c1.center"}])
        );
        assert_eq!(
            projected[1]["refs"],
            json!([{"sketch": "Holes", "point": "c2.center"}])
        );
        assert_eq!(
            call(&calls, "sketch.constraint.coincident")["b"],
            "projected_center"
        );
        assert_eq!(
            call(&calls, "sketch.polygon.circumscribed")["connect"],
            false
        );
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
        // The bearing face is on the pocket floor, so the nut ends flush. One nut is
        // built and placed in both pockets.
        assert!((summary["points"][0][2].as_f64().unwrap() - 4.8).abs() < 1e-9);
        let placed: Vec<&Value> = calls
            .iter()
            .filter(|(c, _)| c == "occurrence.move")
            .map(|(_, args)| &args["to_matrix"][2][3])
            .collect();
        assert_eq!(placed.len(), 2);
        assert!(placed
            .iter()
            .all(|z| (z.as_f64().unwrap() - 4.8).abs() < 1e-9));
        let nuts = calls
            .iter()
            .filter(|(c, args)| c == "solid.extrude" && args["operation"] == "new")
            .count();
        assert_eq!(nuts, 1);
        assert_eq!(calls.iter().filter(|(c, _)| c == "model.faces").count(), 1);
        let joints: Vec<&Value> = calls
            .iter()
            .filter(|(c, _)| c == "joint.create")
            .map(|(_, a)| a)
            .collect();
        assert_eq!(joints.len(), 2);
        for (i, joint) in joints.iter().enumerate() {
            let x = [10, 30][i];
            assert_eq!(
                joint["a"],
                json!({
                    "body": "Plate", "circle": format!("bore{x}"),
                    "end": format!("floor{x}"), "x": format!("wall{x}"),
                })
            );
            assert_eq!(joint["b"]["occurrence"], [3, 4][i]);
            assert_eq!(joint["b"]["point"], summary["points"][i]);
            assert_eq!(joint["b"]["x"], json!([0.0, 1.0, 0.0]));
            assert_eq!(joint["flip"], true);
            assert_eq!(joint["angle"], 0);
        }
        assert_eq!(summary["drilled"], json!(["Plate"]));
        assert_eq!(summary["pocket"]["bolt_hole"], "M6");
    }

    #[test]
    fn nut_and_pocket_with_nothing_selected_makes_a_loose_nut() {
        let (summary, calls) = commands(json!({"create": "Nut and pocket"}));
        // No pocket and no bolt hole: there is no part to cut.
        assert!(!calls
            .iter()
            .any(|(c, args)| c == "solid.hole" && args.get("sketch").is_some()));
        assert_eq!(summary["pocket"], Value::Null);
        assert_eq!(summary["drilled"], json!([]));
        assert_eq!(summary["note"], nut::CREATE.skipped());
        assert_eq!(
            summary["placed"],
            "on the origin, because nothing was selected"
        );
        assert_eq!(summary["at"], json!([0.0, 0.0, 0.0]));
        let (summary, _) = commands(json!({}));
        assert_eq!(summary["note"], Value::Null);
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

plugin_kit::guest! {
    entry: "make-nut",
    noun: "nut",
    manifest: include_str!("../plugin.json"),
    parse: crate::nut::parse,
    make: crate::make_nut,
    describe: |p: &crate::nut::Params| format!("making a {} nut", p.designation),
    aliases: &[],
}
