//! Heat-set insert generator for SolidCraft.
//!
//! One entry, `make-insert`. Every selected sketch point or circle gets the hole for the
//! insert, drilled into the part under it, and a placement of the insert component. The face end
//! sits on the point and the tip runs into the part. With nothing selected, an insert
//! alone sits on the origin. The outside is a 16-point star that
//! twists into a herringbone knurl. `solid.hole` bores that body and threads it.

pub mod catalog;
pub mod insert;
pub mod knurl;

use fasteners::FOLDER;
use insert::{HoleCut, HoleShape, Params};
use plugin_kit::component::{self as fasten, Definition, Placed};
use plugin_kit::host::{body_name_of, sketch_id};
use plugin_kit::name;
use plugin_kit::place::{self, opposite, Placement, Spot};
use plugin_kit::Runner;
use serde_json::{json, Value};

/// A hole needs a part, and the origin has none to find.
const NOTHING_SELECTED: &str = "sketch_point: select the sketch points or circles where the holes go, then run this again; or set Create to Insert only for a loose insert on the origin";

/// Drill the holes, then build the inserts. Returns a summary of what was made.
pub fn make_insert<R: Runner>(r: &mut R, p: &Params) -> Result<Value, String> {
    let spots = place::resolve_all(r, &p.place)?;
    if spots.is_empty() {
        if p.create.hole() {
            return Err(NOTHING_SELECTED.into());
        }
        let places = [Placement::on_origin()];
        let placed = place_inserts(r, p, &places)?;
        return Ok(summary(p, &places, &[], Some(&placed)));
    }
    // The holes go first: the insert would otherwise be a body under the point too.
    let mut drilled: Vec<String> = Vec::new();
    if let Some(cut) = &p.cut {
        let mut sketches: Vec<&str> = Vec::new();
        for spot in &spots {
            if !sketches.contains(&spot.sketch.as_str()) {
                sketches.push(&spot.sketch);
            }
        }
        for sketch in sketches {
            for body in cut_holes(r, p, cut, sketch, &spots)? {
                if !drilled.contains(&body) {
                    drilled.push(body);
                }
            }
        }
    }
    let places: Vec<Placement> = spots.iter().map(|spot| spot.place).collect();
    let placed = if p.create.insert() {
        Some(place_inserts(r, p, &places)?)
    } else {
        None
    };
    Ok(summary(p, &places, &drilled, placed.as_ref()))
}

/// The plugin id, which an insert component's metadata names.
pub const PLUGIN: &str = "techsquid-inserts";

/// One insert component, built once and placed on every point.
fn place_inserts<R: Runner>(r: &mut R, p: &Params, places: &[Placement]) -> Result<Placed, String> {
    let base = body_base(p);
    let description = format!("CNC Kitchen heat-set insert {}", p.size);
    let def = Definition {
        plugin: PLUGIN,
        base: &base,
        description: &description,
        folder: FOLDER,
        bom_type: "purchased",
        // Column 3 is the inserts': one site for a plain bore, one for a thread.
        site: fasten::site(3, u8::from(p.modeled)),
    };
    fasten::place_all(r, &def, places, |mut r, name, site| {
        build_insert(&mut r, p, site, name)
    })
}

/// The body name before any number: `Insert M3 x 5.7`. The thread and the length are
/// the ones built, so a custom thread or length shows in the name.
pub fn body_base(p: &Params) -> String {
    format!("Insert {}", name::sized(&p.thread, p.length))
}

/// The hole feature's name: its kind and the insert it is for, `Heat-set hole M3 x 5.7`.
/// With Hole only there is no insert body, so this is the only place the size shows.
pub fn hole_name(p: &Params, cut: &HoleCut, feature: &str) -> String {
    let kind = match cut.shape {
        HoleShape::Straight { .. } => "Heat-set",
        HoleShape::Stepped { .. } => "Glue-in",
    };
    format!("{kind} {feature} {}", name::sized(&p.thread, p.length))
}

/// One hole feature for every point of `sketch`, so the holes follow the sketch. No
/// target is named: the host drills the body under each point. Returns those bodies.
fn cut_holes<R: Runner>(
    r: &mut R,
    p: &Params,
    cut: &HoleCut,
    sketch: &str,
    spots: &[Spot],
) -> Result<Vec<String>, String> {
    let points: Vec<&str> = spots
        .iter()
        .filter(|spot| spot.sketch == sketch)
        .map(|spot| spot.point.as_str())
        .collect();
    let mut hole = json!({"sketch": sketch, "points": points, "name": hole_name(p, cut, "hole")});
    let mut pilot = None;
    match cut.shape {
        HoleShape::Straight { diameter } => {
            hole["diameter"] = json!(diameter);
            if let Some(depth) = cut.depth {
                hole["depth"] = json!(depth);
            }
        }
        HoleShape::Stepped { d1, d2, d3, l1, l2 } => {
            hole["type"] = json!("counterbore");
            hole["cb_diameter"] = json!(d1);
            hole["cb_depth"] = json!(l1);
            hole["diameter"] = json!(d2);
            hole["depth"] = json!(l2);
            // The host starts a hole at the surface under the point, which is now the
            // floor of the step at `l2`. The pilot's depth is what is left below it.
            let mut below = json!({
                "sketch": sketch,
                "points": points,
                "diameter": d3,
                "name": hole_name(p, cut, "pilot"),
            });
            if let Some(depth) = cut.depth {
                below["depth"] = json!(depth - l2);
            }
            pilot = Some(below);
        }
    }
    let through = cut.depth.is_none();
    let mut bodies = drilled(&r.run("solid.hole", hole)?, through && pilot.is_none())?;
    if let Some(pilot) = pilot {
        for body in drilled(&r.run("solid.hole", pilot)?, through)? {
            if !bodies.contains(&body) {
                bodies.push(body);
            }
        }
    }
    Ok(bodies)
}

/// The bodies a hole changed. A through hole on a sketch that is not on a face of the
/// part also drills whatever lies beyond it, and that is refused.
fn drilled(reply: &Value, through: bool) -> Result<Vec<String>, String> {
    let Some(changed) = reply.get("changed_bodies").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let bodies: Vec<String> = changed
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    if bodies.is_empty() {
        return Err(
            "sketch_point: there is no part under the point to drill; put the sketch on the part, or set Create to Insert only"
                .into(),
        );
    }
    if through && bodies.len() > 1 {
        return Err(format!(
            "hole_depth: a through hole here drills {} bodies ({}); use Blind, or put the sketch on a face of the part",
            bodies.len(),
            bodies.join(", ")
        ));
    }
    Ok(bodies)
}

/// One insert body, named `body_name`, with its face end on `place`.
fn build_insert<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body_name: &str,
) -> Result<(), String> {
    let shape = knurl::shape(p.length, p.diameter, p.major);
    let tip = place.tip(p.length);
    // Every feature and sketch of this insert starts with its name.
    let named = |feature: &str| name::part(body_name, feature);
    let made = r.run(
        "solid.cylinder",
        json!({
            "diameter": shape.gap_diameter,
            "height": p.length,
            "base": tip,
            "axis": place.normal,
            "operation": "new",
            "name": named("Core"),
            "body_name": body_name,
        }),
    )?;
    let body = body_name_of(&made).ok_or_else(|| "the insert body was not named".to_string())?;
    r.run(
        "solid.cylinder",
        json!({
            "diameter": shape.lead_diameter,
            "height": shape.lead,
            "base": tip,
            "axis": place.normal,
            "operation": "join",
            "targets": [body],
            "name": named("Lead-in"),
        }),
    )?;
    sweep_band(
        r,
        place,
        &body,
        &shape,
        0.0,
        shape.twist_deg,
        &named("Upper knurl"),
    )?;
    sweep_band(
        r,
        place,
        &body,
        &shape,
        shape.band + shape.gap,
        -shape.twist_deg,
        &named("Lower knurl"),
    )?;
    let edge = place.on_shank(p.length, 0.0, shape.root);
    r.run(
        "solid.chamfer",
        json!({
            "edges": [edge],
            "distance": shape.chamfer,
            "body": body,
            "name": named("Tip chamfer"),
        }),
    )?;
    let mut hole = json!({
        "position": place.point,
        "direction": opposite(place.normal),
        "depth": p.length,
        "targets": [body],
        "name": named("Bore"),
    });
    // `#2-56` has no tap name in this host, so the bore is a plain diameter.
    if let Some(diameter) = p.plain_bore {
        hole["diameter"] = json!(diameter);
        hole["modeled"] = json!(false);
    } else {
        hole["thread"] = json!(p.thread);
        hole["modeled"] = json!(p.modeled);
    }
    r.run("solid.hole", hole)?;
    Ok(())
}

/// One helical band. `from_face` is where it starts, toward the tip.
fn sweep_band<R: Runner>(
    r: &mut R,
    place: &Placement,
    body: &str,
    shape: &knurl::Shape,
    from_face: f64,
    twist_deg: f64,
    name: &str,
) -> Result<(), String> {
    let origin = place.tip(from_face);
    let profile = r.run(
        "sketch.create",
        json!({
            "name": format!("{name} profile"),
            "plane": {
                "origin": origin,
                "x_dir": place.x_dir,
                "y_dir": place.y_dir,
            },
        }),
    )?;
    let profile_id = sketch_id(&profile)?;
    r.run(
        "sketch.line",
        json!({
            "sketch": profile_id,
            "closed": true,
            "points": knurl::star(shape.crest, shape.root),
        }),
    )?;
    r.run("sketch.finish", json!({}))?;
    let path = r.run(
        "sketch.create",
        json!({
            "name": format!("{name} path"),
            "plane": {
                "origin": origin,
                "x_dir": place.x_dir,
                "y_dir": opposite(place.normal),
            },
        }),
    )?;
    let path_id = sketch_id(&path)?;
    let drawn = r.run(
        "sketch.line",
        json!({
            "sketch": path_id,
            "points": [[0.0, 0.0], [0.0, shape.band]],
        }),
    )?;
    let curve = created_curve(&drawn)?;
    r.run("sketch.finish", json!({}))?;
    r.run(
        "solid.sweep",
        json!({
            "sketch": profile_id,
            "path_sketch": path_id,
            "path": [curve],
            "twist": twist_deg,
            "operation": "join",
            "targets": [body],
            "name": name,
        }),
    )?;
    // A sweep leaves both sketches shown, where an extrude or a revolve hides its own.
    // The path starts on the insert's axis at its open face, the very point a screw is
    // placed on, and a screw placed on a point of the path would take the path's
    // sketch normal, square to the axis, and lie on its side. Hidden, neither can be picked.
    // Hiding is cosmetic, so a host that refuses it still keeps the insert.
    let _ = r.run(
        "browser.visibility",
        json!({"visible": false, "items": [profile_id, path_id]}),
    );
    Ok(())
}

fn created_curve(reply: &Value) -> Result<String, String> {
    reply
        .get("created")
        .and_then(|created| created.get("curves"))
        .and_then(Value::as_array)
        .and_then(|curves| curves.first())
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "sketch.line did not return a curve id".to_string())
}

fn summary(p: &Params, places: &[Placement], drilled: &[String], placed: Option<&Placed>) -> Value {
    let place = &places[0];
    let hole = p.cut.map(|cut| match cut.shape {
        HoleShape::Straight { diameter } => json!({
            "type": insert::STRAIGHT,
            "diameter": diameter,
            "depth": cut.depth,
        }),
        HoleShape::Stepped { d1, d2, d3, l1, l2 } => json!({
            "type": insert::STEPPED,
            "d1": d1, "d2": d2, "d3": d3, "l1": l1, "l2": l2,
            "depth": cut.depth,
        }),
    });
    json!({
        "create": p.create.label(),
        "count": places.len(),
        "points": places.iter().map(|place| place.point).collect::<Vec<_>>(),
        "hole": hole,
        "drilled": drilled,
        "size": p.size,
        "thread": p.thread,
        "length": p.length,
        "diameter": p.diameter,
        "modeled": p.modeled,
        "at": place.point,
        "normal": place.normal,
        "placed": place.source.words(),
        "blind_depth": p.length + 1.0,
        "plastic_hole": p.hole,
        "min_wall": p.wall,
        "component": placed.map(Placed::json),
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
                "sketch.line" => json!({"created": {"curves": ["l1"]}}),
                "solid.cylinder"
                    if args.get("operation").and_then(Value::as_str) == Some("new") =>
                {
                    json!({"created_bodies": [args["body_name"]]})
                }
                _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
            };
            self.calls.push((command.to_string(), args));
            Ok(reply)
        }
    }

    /// The older tests build a loose insert on the origin, which is Insert only.
    fn loose(mut v: Value) -> Value {
        if v.get("create").is_none() {
            v["create"] = json!("Insert only");
        }
        v
    }

    fn commands(v: Value) -> (Value, Vec<(String, Value)>) {
        let p = insert::parse(&loose(v)).unwrap();
        let mut rec = Rec {
            calls: vec![],
            sketch: 0,
        };
        let summary = make_insert(&mut rec, &p).unwrap();
        (summary, rec.calls)
    }

    fn call<'a>(calls: &'a [(String, Value)], command: &str) -> &'a Value {
        &calls.iter().find(|(name, _)| name == command).unwrap().1
    }

    fn last<'a>(calls: &'a [(String, Value)], command: &str) -> &'a Value {
        &calls.iter().rfind(|(name, _)| name == command).unwrap().1
    }

    #[test]
    fn an_m3_insert_is_a_herringbone_and_a_bore() {
        let (summary, calls) = commands(json!({}));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "solid.cylinder",
                "sketch.create",
                "sketch.line",
                "sketch.finish",
                "sketch.create",
                "sketch.line",
                "sketch.finish",
                "solid.sweep",
                "browser.visibility",
                "sketch.create",
                "sketch.line",
                "sketch.finish",
                "sketch.create",
                "sketch.line",
                "sketch.finish",
                "solid.sweep",
                "browser.visibility",
                "solid.chamfer",
                "solid.hole",
            ]
        );
        // Each band hides its own profile and path, so no point of either is picked
        // later to place a screw.
        let hidden: Vec<&Value> = calls
            .iter()
            .filter(|(c, _)| c == "browser.visibility")
            .map(|(_, a)| a)
            .collect();
        assert_eq!(hidden[0], &json!({"visible": false, "items": [1, 2]}));
        assert_eq!(hidden[1], &json!({"visible": false, "items": [3, 4]}));
        let shape = knurl::shape(5.7, 4.6, 3.0);
        let core = &calls[1].1;
        assert_eq!(core["diameter"], json!(shape.gap_diameter));
        assert_eq!(core["height"], 5.7);
        assert_eq!(core["base"], json!([0.0, 0.0, -5.7]));
        assert_eq!(core["axis"], json!([0.0, 0.0, 1.0]));
        assert_eq!(core["operation"], "new");
        assert_eq!(core["body_name"], "Insert M3 x 5.7");
        assert_eq!(core["name"], "Insert M3 x 5.7: Core");
        assert_eq!(calls[2].1["diameter"], json!(shape.lead_diameter));
        assert_eq!(calls[2].1["operation"], "join");
        let star = &calls[4].1["points"];
        assert_eq!(calls[4].1["sketch"], 1);
        assert_eq!(star.as_array().unwrap().len(), 32);
        assert_eq!(star[0][0], json!(shape.crest));
        assert_eq!(calls[7].1["sketch"], 2);
        assert_eq!(calls[9].1["twist"], json!(shape.twist_deg));
        assert_eq!(calls[17].1["twist"], json!(-shape.twist_deg));
        assert_eq!(calls[9].1["targets"], json!(["Insert M3 x 5.7"]));
        assert_eq!(calls[17].1["targets"], json!(["Insert M3 x 5.7"]));
        let hole = &calls[20].1;
        assert_eq!(hole["position"], json!([0.0, 0.0, 0.0]));
        assert_eq!(hole["direction"], json!([0.0, 0.0, -1.0]));
        assert_eq!(hole["depth"], 5.7);
        assert_eq!(hole["thread"], "M3");
        assert_eq!(hole["modeled"], true);
        assert!(hole.get("diameter").is_none());
        assert_eq!(hole["targets"], json!(["Insert M3 x 5.7"]));
        assert_eq!(summary["thread"], "M3");
        assert_eq!(summary["diameter"], 4.6);
        assert_eq!(summary["length"], 5.7);
        assert_eq!(summary["blind_depth"], 6.7);
        assert_eq!(summary["size"], "M3 × 5.7");
        assert_eq!(summary["plastic_hole"], 4.0);
        assert_eq!(summary["min_wall"], 1.6);
        assert_eq!(
            summary["placed"],
            "on the origin, because nothing was selected"
        );
        assert_eq!(summary["at"], json!([0.0, 0.0, 0.0]));
    }

    #[test]
    fn a_longer_insert_keeps_the_catalog_crest() {
        let (summary, calls) = commands(json!({"custom_length": 8}));
        let shape = knurl::shape(8.0, 4.6, 3.0);
        assert_eq!(calls[1].0, "solid.cylinder");
        assert_eq!(calls[1].1["height"], 8.0);
        assert_eq!(calls[1].1["diameter"], json!(shape.gap_diameter));
        assert_eq!(calls[1].1["base"], json!([0.0, 0.0, -8.0]));
        assert_eq!(last(&calls, "solid.hole")["depth"], 8.0);
        assert_eq!(summary["diameter"], 4.6);
        assert_eq!(summary["length"], 8.0);
        assert_eq!(summary["blind_depth"], 9.0);
    }

    #[test]
    fn a_diameter_override_moves_the_crest() {
        let (summary, calls) = commands(json!({"custom_diameter": 5}));
        let shape = knurl::shape(5.7, 5.0, 3.0);
        assert_eq!(
            call(&calls, "sketch.line")["points"][0][0],
            json!(shape.crest)
        );
        assert_eq!(summary["diameter"], 5.0);
        assert_eq!(summary["thread"], "M3");
        assert_eq!(
            last(&calls, "solid.hole")["targets"],
            json!(["Insert M3 x 5.7"])
        );
    }

    #[test]
    fn a_number_two_insert_is_a_plain_bore() {
        let (_, calls) = commands(json!({"size": "#2-56 × 3.2"}));
        assert_eq!(calls[1].1["height"], 3.2);
        let hole = last(&calls, "solid.hole");
        assert_eq!(hole["diameter"], json!(0.086 * 25.4 - 25.4 / 56.0));
        assert_eq!(hole["modeled"], false);
        assert!(hole.get("thread").is_none());
    }

    #[test]
    fn a_renamed_body_is_the_bore_target() {
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
                    "sketch.line" => json!({"created": {"curves": ["l1"]}}),
                    "solid.cylinder"
                        if args.get("operation").and_then(Value::as_str) == Some("new") =>
                    {
                        json!({"created_bodies": ["Body1"]})
                    }
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
                };
                self.calls.push((command.to_string(), args));
                Ok(reply)
            }
        }
        let p = insert::parse(&loose(json!({}))).unwrap();
        let mut rec = Renamed {
            calls: vec![],
            sketch: 0,
        };
        make_insert(&mut rec, &p).unwrap();
        assert_eq!(last(&rec.calls, "solid.hole")["targets"], json!(["Body1"]));
        assert_eq!(last(&rec.calls, "solid.sweep")["targets"], json!(["Body1"]));
        assert_eq!(call(&rec.calls, "solid.chamfer")["body"], "Body1");
    }

    #[test]
    fn a_body_without_a_name_is_not_bored() {
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
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
                })
            }
        }
        let p = insert::parse(&loose(json!({}))).unwrap();
        let mut rec = Mute(vec![]);
        let err = make_insert(&mut rec, &p).unwrap_err();
        assert_eq!(err, "the insert body was not named");
        assert_eq!(rec.0, ["selection.get", "solid.cylinder"]);
    }

    #[test]
    fn a_named_point_puts_the_face_end_there() {
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
                    "sketch.create" => json!({"sketch": 5}),
                    "sketch.line" => json!({"created": {"curves": ["l1"]}}),
                    "solid.cylinder"
                        if args.get("operation").and_then(Value::as_str) == Some("new") =>
                    {
                        json!({"created_bodies": ["Insert"]})
                    }
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
                };
                self.0.push((command.to_string(), args));
                Ok(reply)
            }
        }
        let p = insert::parse(&loose(json!({"sketch": "Holes", "sketch_point": "c1"}))).unwrap();
        let mut rec = Named(vec![]);
        let summary = make_insert(&mut rec, &p).unwrap();
        assert_eq!(rec.0[0].0, "sketch.inspect");
        // The insert is built in its own component, its face end on the origin.
        assert_eq!(rec.0[1].1["base"], json!([0.0, 0.0, -5.7]));
        let hole = last(&rec.0, "solid.hole");
        assert_eq!(hole["position"], json!([0.0, 0.0, 0.0]));
        assert_eq!(hole["direction"], json!([0.0, 0.0, -1.0]));
        assert_eq!(summary["at"], json!([12.0, 10.0, 8.0]));
    }

    #[test]
    fn a_modeled_unified_thread_builds_a_cosmetic_bore() {
        let (summary, calls) = commands(json!({"size": "1/4-20 × 12.7", "thread_modeled": true}));
        let hole = last(&calls, "solid.hole");
        assert_eq!(hole["thread"], "1/4-20 UNC");
        assert_eq!(hole["modeled"], false);
        assert_eq!(summary["modeled"], false);
    }

    /// A plate with a sketch of two circles on its top face, both selected.
    struct Plate {
        calls: Vec<(String, Value)>,
        sketch: i64,
        /// What a hole reports as `changed_bodies`.
        changed: Value,
    }
    impl Runner for Plate {
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
                "sketch.create" => {
                    self.sketch += 1;
                    json!({"sketch": self.sketch})
                }
                "sketch.line" => json!({"created": {"curves": ["l1"]}}),
                "solid.cylinder"
                    if args.get("operation").and_then(Value::as_str) == Some("new") =>
                {
                    json!({"created_bodies": [format!("Insert{}", self.sketch)]})
                }
                "solid.hole" if args.get("sketch").is_some() => {
                    json!({"changed_bodies": self.changed})
                }
                _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
            };
            self.calls.push((command.to_string(), args));
            Ok(reply)
        }
    }

    fn on_plate(v: Value, changed: Value) -> (Result<Value, String>, Vec<(String, Value)>) {
        let p = insert::parse(&v).unwrap();
        let mut rec = Plate {
            calls: vec![],
            sketch: 0,
            changed,
        };
        let made = make_insert(&mut rec, &p);
        (made, rec.calls)
    }

    #[test]
    fn two_selected_circles_get_one_hole_feature_and_two_inserts() {
        let (summary, calls) = on_plate(json!({}), json!(["Plate"]));
        let summary = summary.unwrap();
        // The holes come before any insert body exists.
        assert_eq!(calls[0].0, "selection.get");
        assert_eq!(calls[1].0, "sketch.inspect");
        assert_eq!(calls[2].0, "solid.hole");
        let hole = &calls[2].1;
        assert_eq!(hole["sketch"], "Holes");
        assert_eq!(hole["points"], json!(["c1.center", "c2.center"]));
        assert_eq!(hole["diameter"], 4.0);
        assert_eq!(hole["depth"], 6.7);
        assert!(hole.get("targets").is_none());
        let bodies: Vec<&Value> = calls
            .iter()
            .filter(|(name, args)| name == "solid.cylinder" && args["operation"] == "new")
            .map(|(_, args)| &args["base"])
            .collect();
        // One insert is built, and placed with its face end on each circle.
        assert_eq!(bodies, [&json!([0.0, 0.0, -5.7])]);
        let placed: Vec<&Value> = calls
            .iter()
            .filter(|(name, _)| name == "occurrence.move")
            .map(|(_, args)| &args["to_matrix"])
            .collect();
        assert_eq!(
            placed,
            [
                &json!([
                    [1.0, 0.0, 0.0, 10.0],
                    [0.0, 1.0, 0.0, 10.0],
                    [0.0, 0.0, 1.0, 10.0]
                ]),
                &json!([
                    [1.0, 0.0, 0.0, 30.0],
                    [0.0, 1.0, 0.0, 10.0],
                    [0.0, 0.0, 1.0, 10.0]
                ]),
            ]
        );
        // The bore drills the insert, not the plate.
        let bores: Vec<&Value> = calls
            .iter()
            .filter(|(name, args)| name == "solid.hole" && args.get("sketch").is_none())
            .map(|(_, args)| &args["targets"])
            .collect();
        assert_eq!(bores, [&json!(["Insert0"])]);
        assert_eq!(summary["component"]["component"], "Insert M3 x 5.7");
        assert_eq!(summary["count"], 2);
        assert_eq!(summary["drilled"], json!(["Plate"]));
        assert_eq!(summary["hole"]["diameter"], 4.0);
        assert_eq!(summary["placed"], "on the selected sketch point");
    }

    #[test]
    fn hole_only_builds_no_insert() {
        let (summary, calls) = on_plate(json!({"create": "Hole only"}), json!(["Plate"]));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            ["selection.get", "sketch.inspect", "solid.hole"]
        );
        assert_eq!(summary.unwrap()["create"], "Hole only");
    }

    #[test]
    fn a_glue_in_hole_is_a_counterbore_and_a_pilot_from_its_floor() {
        let (_, calls) = on_plate(
            json!({"create": "Hole only", "hole_type": insert::STEPPED}),
            json!(["Plate"]),
        );
        let upper = &calls[2].1;
        assert_eq!(upper["type"], "counterbore");
        assert_eq!(upper["cb_diameter"], 4.6);
        assert_eq!(upper["cb_depth"], 1.7);
        assert_eq!(upper["diameter"], 4.4);
        assert_eq!(upper["depth"], 4.2);
        let pilot = &calls[3].1;
        assert_eq!(pilot["diameter"], 4.0);
        assert_eq!(pilot["points"], json!(["c1.center", "c2.center"]));
        // 6.7 mm from the surface is 2.5 mm below the step at 4.2 mm.
        assert!((pilot["depth"].as_f64().unwrap() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn a_through_hole_that_reaches_a_second_body_is_refused() {
        let (made, _) = on_plate(json!({"hole_depth": "Through"}), json!(["Plate"]));
        assert!(made.is_ok());
        let (made, calls) = on_plate(json!({"hole_depth": "Through"}), json!(["Plate", "Under"]));
        let err = made.unwrap_err();
        assert!(err.starts_with("hole_depth: "), "{err}");
        assert!(err.contains("Plate, Under"), "{err}");
        assert!(calls[2].1.get("depth").is_none());
        // A blind hole across two parts is what was asked for.
        let (made, _) = on_plate(json!({}), json!(["Plate", "Under"]));
        assert!(made.is_ok());
        let (made, _) = on_plate(json!({}), json!([]));
        assert!(made.unwrap_err().starts_with("sketch_point: "));
    }

    #[test]
    fn a_hole_with_nothing_selected_says_what_to_select() {
        let p = insert::parse(&json!({})).unwrap();
        let mut rec = Rec {
            calls: vec![],
            sketch: 0,
        };
        let err = make_insert(&mut rec, &p).unwrap_err();
        assert!(err.starts_with("sketch_point: select"), "{err}");
        assert!(err.contains("Insert only"), "{err}");
        assert_eq!(rec.calls.len(), 1);
    }

    #[test]
    fn the_manifest_asks_for_a_catalog_size() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(manifest["id"], "techsquid-inserts");
        assert_eq!(manifest["name"], "Heat-set inserts");
        let description = manifest["description"].as_str().unwrap();
        assert!(description.chars().count() <= 240, "{description}");
        let entry = &manifest["runtime"]["entries"][0];
        assert_eq!(entry["name"], "make-insert");
        // The menu row, the dialog title and the status line all use this name.
        assert_eq!(
            entry["slots"],
            json!([{"kind": "menu", "name": "Heat-set insert"}])
        );
        let schema = &entry["input_schema"];
        assert_eq!(
            schema["required"],
            json!(["size", "create", "hole_type", "hole_depth", "hole_adjust"])
        );
        let props = &schema["properties"];
        assert_eq!(props["size"]["default"], catalog::DEFAULT_SIZE);
        assert_eq!(props["thread_modeled"]["default"], true);
        assert_eq!(props["create"]["default"], "Hole and insert");
        assert_eq!(
            props["hole_type"]["enum"],
            json!([insert::STRAIGHT, insert::STEPPED])
        );
        assert_eq!(props["hole_adjust"]["maximum"], insert::MAX_ADJUST);
        // Every choice the dialog offers is one the plugin takes.
        for key in ["create", "hole_type", "hole_depth"] {
            for choice in props[key]["enum"].as_array().unwrap() {
                insert::parse(&json!({ key: choice })).unwrap();
            }
        }
        // The dialog lists optional fields by key, so the keys are the order.
        let mut optional: Vec<&str> = props
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .filter(|key| !schema["required"].as_array().unwrap().contains(&json!(key)))
            .collect();
        optional.sort();
        assert_eq!(
            optional,
            [
                "custom_diameter",
                "custom_length",
                "custom_thread",
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
                "sketch.line",
                "sketch.finish",
                "solid.cylinder",
                "solid.sweep",
                "browser.visibility",
                "solid.chamfer",
                "solid.hole",
                "component.list",
                "component.create",
                "component.metadata",
                "component.metadata.set",
                "component.activate",
                "occurrence.copy",
                "occurrence.move",
                "occurrence.reparent"
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
    /// A run builds one insert, whatever the number of points.
    #[test]
    fn every_new_insert_asks_for_a_free_body_name() {
        let p = insert::parse(&json!({"sketch": "Points", "sketch_point": "a, b"})).unwrap();
        let mut host = design(&["Base"]);
        make_insert(&mut host, &p).unwrap();
        make_insert(&mut host, &p).unwrap();
        assert_eq!(host.asked, ["Insert M3 x 5.7", "Insert M3 x 5.7 (2)"]);
    }

    #[test]
    fn a_body_name_is_the_thread_and_length_built() {
        let base = |v: Value| body_base(&insert::parse(&loose(v)).unwrap());
        assert_eq!(base(json!({})), "Insert M3 x 5.7");
        assert_eq!(base(json!({"size": "M3 × 3 short"})), "Insert M3 x 3");
        assert_eq!(
            base(json!({"size": "#4-40 × 5.7"})),
            "Insert #4-40 UNC x 5.7"
        );
        assert_eq!(base(json!({"custom_length": 8})), "Insert M3 x 8");
    }

    #[test]
    fn every_feature_of_an_insert_starts_with_its_body_name() {
        let (_, calls) = commands(json!({}));
        for (command, args) in &calls {
            if let Some(name) = args.get("name").and_then(Value::as_str) {
                assert!(
                    name.starts_with("Insert M3 x 5.7: "),
                    "{command} is named {name}"
                );
            }
        }
    }

    #[test]
    fn a_hole_names_its_kind_and_the_insert() {
        let named_hole = |v: Value, feature: &str| {
            let p = insert::parse(&v).unwrap();
            hole_name(&p, p.cut.as_ref().unwrap(), feature)
        };
        assert_eq!(named_hole(json!({}), "hole"), "Heat-set hole M3 x 5.7");
        let stepped = json!({"hole_type": insert::STEPPED});
        assert_eq!(named_hole(stepped.clone(), "hole"), "Glue-in hole M3 x 5.7");
        assert_eq!(named_hole(stepped, "pilot"), "Glue-in pilot M3 x 5.7");
    }
}

plugin_kit::guest! {
    entry: "make-insert",
    noun: "insert",
    manifest: include_str!("../plugin.json"),
    parse: crate::insert::parse,
    make: crate::make_insert,
    describe: |p: &crate::insert::Params| {
        format!("{}: {} insert, {} mm", p.create.label(), p.thread, p.length)
    },
    aliases: &[],
}
