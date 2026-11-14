//! 20 mm series T-slot and V-slot aluminium extrusions for SolidCraft.
//!
//! One entry, `make-extrusion`. The rail's section is centred on a sketch point, or on
//! the origin when nothing is selected, and the rail runs out of the sketch for its
//! length. The section is one sketch of lines and arcs, extruded once.

pub mod rail;

use extrusions::{Profile, Seg};
use plugin_kit::component::{self, Definition, Placed};
use plugin_kit::host::sketch_id;
use plugin_kit::name;
use plugin_kit::place::{self, Placement};
use plugin_kit::Runner;
use rail::Params;
use serde_json::{json, Value};

/// The plugin id, which a rail component's metadata names.
pub const PLUGIN: &str = "techsquid-extrusions";

/// The component every rail placement goes in.
pub const FOLDER: &str = "Extrusions";

/// The body name before any number: `T-slot 2040 x 500`.
pub fn body_base(p: &Params) -> String {
    format!("{} {}", p.slot.name, name::sized(&p.size, p.length))
}

/// What the bill of materials says a rail is.
pub fn description(p: &Params) -> String {
    format!(
        "{} x {} mm {} aluminium extrusion, {} mm long",
        name::length(p.grid.width()),
        name::length(p.grid.height()),
        p.slot.name,
        name::length(p.length)
    )
}

/// Build the rail on every point. Returns a summary of what was made.
pub fn make_extrusion<R: Runner>(r: &mut R, p: &Params) -> Result<Value, String> {
    let profile = Profile::new(p.slot, p.grid).ok_or("size: this size has no section")?;
    let spots = place::resolve_all(r, &p.place)?;
    let places: Vec<Placement> = if spots.is_empty() {
        vec![Placement::on_origin()]
    } else {
        spots.iter().map(|spot| spot.place).collect()
    };
    let base = body_base(p);
    let description = description(p);
    let def = Definition {
        plugin: PLUGIN,
        base: &base,
        description: &description,
        folder: FOLDER,
        bom_type: "purchased",
    };
    let placed = component::place_all(r, &def, &places, |mut r, name| {
        build_rail(&mut r, &profile, p.length, &Placement::on_origin(), name)
    })?;
    Ok(summary(p, &profile, &places, &placed))
}

/// One rail named `body_name`, its section centred on `place` and its length along the
/// normal.
pub fn build_rail<R: Runner>(
    r: &mut R,
    profile: &Profile,
    length: f64,
    place: &Placement,
    body_name: &str,
) -> Result<(), String> {
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": place.point,
                "x_dir": place.x_dir,
                "y_dir": place.y_dir,
            },
            "name": name::part(body_name, "Section"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    draw_loop(r, &profile.outer)?;
    for cavity in &profile.cavities {
        draw_loop(r, cavity)?;
    }
    for bore in &profile.bores {
        r.run(
            "sketch.circle.center",
            json!({"center": bore, "diameter": profile.slot.bore}),
        )?;
    }
    r.run("sketch.finish", json!({}))?;
    // The region with the metal in it, so the cavities and bores stay open.
    r.run(
        "solid.extrude",
        json!({
            "sketch": sketch,
            "profiles": [{"point": profile.material_point()}],
            "distance": length,
            "operation": "new",
            "body_name": body_name,
            "name": name::part(body_name, "Rail"),
        }),
    )?;
    Ok(())
}

/// Draw a closed loop: each run of lines is one polyline, and each arc is its own
/// command. Every piece starts on the exact point the one before it ended on.
fn draw_loop<R: Runner>(r: &mut R, segs: &[Seg]) -> Result<(), String> {
    let mut run: Vec<[f64; 2]> = Vec::new();
    for seg in segs {
        match *seg {
            Seg::Line(a, b) => {
                if run.is_empty() {
                    run.push(a);
                }
                run.push(b);
            }
            Seg::Arc {
                start,
                end,
                through,
                ..
            } => {
                flush(r, &mut run)?;
                r.run(
                    "sketch.arc.three_point",
                    json!({"start": start, "end": end, "through": through}),
                )?;
            }
        }
    }
    flush(r, &mut run)
}

fn flush<R: Runner>(r: &mut R, run: &mut Vec<[f64; 2]>) -> Result<(), String> {
    if run.len() > 1 {
        r.run("sketch.line", json!({"points": run}))?;
    }
    run.clear();
    Ok(())
}

fn summary(p: &Params, profile: &Profile, places: &[Placement], placed: &Placed) -> Value {
    json!({
        "slot": p.slot.name,
        "size": p.size,
        "length": p.length,
        "width": p.grid.width(),
        "height": p.grid.height(),
        "area_mm2": (profile.area() * 100.0).round() / 100.0,
        "count": places.len(),
        "placed": places[0].source.words(),
        "points": places.iter().map(|place| place.point).collect::<Vec<_>>(),
        "component": placed.json(),
    })
}

plugin_kit::guest! {
    entry: "make-extrusion",
    noun: "extrusion",
    manifest: include_str!("../plugin.json"),
    parse: crate::rail::parse,
    make: crate::make_extrusion,
    describe: |p: &crate::rail::Params| format!("making a {}", crate::body_base(p)),
    aliases: &[],
}

#[cfg(test)]
mod tests {
    use super::*;
    use extrusions::profile::SIZES;

    struct Rec {
        calls: Vec<(String, Value)>,
    }
    impl Runner for Rec {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            // Placing the rail is checked in plugin-kit; these check the build.
            if let Some(reply) = component::stand_in(command) {
                return Ok(reply);
            }
            let reply = match command {
                "selection.get" => json!({"count": 0, "items": []}),
                "sketch.create" => json!({"sketch": 1}),
                _ => Value::Null,
            };
            self.calls.push((command.to_string(), args));
            Ok(reply)
        }
    }

    fn commands(v: Value) -> (Value, Vec<(String, Value)>) {
        let p = rail::parse(&v).unwrap();
        let mut rec = Rec { calls: vec![] };
        let summary = make_extrusion(&mut rec, &p).unwrap();
        (summary, rec.calls)
    }

    fn all<'a>(calls: &'a [(String, Value)], command: &str) -> Vec<&'a Value> {
        calls
            .iter()
            .filter(|(c, _)| c == command)
            .map(|(_, args)| args)
            .collect()
    }

    #[test]
    fn the_manifest_is_this_plugin() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(manifest["id"], PLUGIN);
        let description = manifest["description"].as_str().unwrap();
        assert!(description.chars().count() <= 240, "{description}");
        let entry = &manifest["runtime"]["entries"][0];
        assert_eq!(entry["name"], "make-extrusion");
        let schema = &entry["input_schema"];
        assert_eq!(schema["required"], json!(["slot", "size", "length"]));
        let sizes: Vec<Value> = SIZES.iter().map(|(name, _)| json!(name)).collect();
        assert_eq!(schema["properties"]["size"]["enum"], json!(sizes));
        assert_eq!(
            schema["properties"]["slot"]["enum"],
            json!(["T-slot", "V-slot"])
        );
        assert_eq!(schema["properties"]["size"]["default"], rail::DEFAULT_SIZE);
        assert_eq!(
            schema["properties"]["length"]["default"],
            rail::DEFAULT_LENGTH
        );
        assert_eq!(
            schema["properties"]["length"]["maximum"],
            rail::MAX_LENGTH_MM
        );
    }

    #[test]
    fn the_manifest_lists_every_command_sent() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let listed = manifest["runtime"]["commands"].as_array().unwrap();
        for command in component::COMMANDS {
            assert!(listed.contains(&json!(command)), "{command}");
        }
        for v in [
            json!({"slot": "T-slot", "size": "4040", "length": 100}),
            json!({"slot": "V-slot", "size": "2040", "length": 100}),
        ] {
            let (_, calls) = commands(v);
            for (command, _) in &calls {
                assert!(listed.contains(&json!(command)), "{command}");
            }
        }
        // Placing on a point reads the selection and the sketch.
        for command in ["selection.get", "sketch.inspect", "inspect.measure"] {
            assert!(listed.contains(&json!(command)), "{command}");
        }
    }

    #[test]
    fn a_2020_is_one_sketch_and_one_extrude() {
        let (summary, calls) = commands(json!({"slot": "T-slot", "size": "2020", "length": 250}));
        let sketch = all(&calls, "sketch.create");
        assert_eq!(sketch.len(), 1);
        assert_eq!(sketch[0]["name"], "T-slot 2020 x 250: Section");
        assert_eq!(sketch[0]["plane"]["origin"], json!([0.0, 0.0, 0.0]));
        // Each side is three runs of lines between the two roundings of its slot floor,
        // and ends in a corner arc. Then the bore.
        assert_eq!(all(&calls, "sketch.line").len(), 12);
        assert_eq!(all(&calls, "sketch.arc.three_point").len(), 12);
        let bores = all(&calls, "sketch.circle.center");
        assert_eq!(bores.len(), 1);
        assert_eq!(bores[0]["diameter"], 4.2);
        let extrude = all(&calls, "solid.extrude");
        assert_eq!(extrude.len(), 1);
        assert_eq!(extrude[0]["distance"], 250.0);
        assert_eq!(extrude[0]["operation"], "new");
        assert_eq!(extrude[0]["sketch"], 1);
        assert_eq!(summary["count"], 1);
        assert_eq!(
            summary["placed"],
            "on the origin, because nothing was selected"
        );
        assert_eq!(summary["component"]["component"], "T-slot 2020 x 250");
    }

    #[test]
    fn every_polyline_starts_where_the_piece_before_it_ended() {
        let (_, calls) = commands(json!({"slot": "V-slot", "size": "2040", "length": 100}));
        let mut last: Option<Value> = None;
        let mut first: Option<Value> = None;
        let mut loops = 0;
        for (command, args) in &calls {
            let (start, end) = match command.as_str() {
                "sketch.line" => {
                    let points = args["points"].as_array().unwrap();
                    (points[0].clone(), points[points.len() - 1].clone())
                }
                "sketch.arc.three_point" => (args["start"].clone(), args["end"].clone()),
                _ => continue,
            };
            match &last {
                Some(end_before) if *end_before == start => {}
                _ => {
                    // A new loop starts; the one before must have closed.
                    assert_eq!(last, first);
                    first = Some(start);
                    loops += 1;
                }
            }
            last = Some(end);
        }
        assert_eq!(last, first);
        // The outline and the cavity between the two cells.
        assert_eq!(loops, 2);
    }

    #[test]
    fn a_2080_has_three_cavities_and_four_bores() {
        let (_, calls) = commands(json!({"slot": "T-slot", "size": "2080", "length": 100}));
        // Ten slots with three runs of lines each, less one run where a slot follows
        // another on a long side, and three closed cavities.
        assert_eq!(all(&calls, "sketch.line").len(), 27);
        // Two roundings in each of the ten slot floors, and four corners.
        assert_eq!(all(&calls, "sketch.arc.three_point").len(), 24);
        assert_eq!(all(&calls, "sketch.circle.center").len(), 4);
    }

    #[test]
    fn a_4040_draws_its_bores_as_arcs_of_the_middle_cavity() {
        let (summary, calls) = commands(json!({"slot": "T-slot", "size": "4040", "length": 100}));
        assert!(all(&calls, "sketch.circle.center").is_empty());
        // Four corners, two roundings in each of the eight slot floors, and two arcs
        // round each of the four bores.
        assert_eq!(all(&calls, "sketch.arc.three_point").len(), 28);
        assert_eq!(summary["width"], 40.0);
        assert_eq!(summary["height"], 40.0);
    }

    #[test]
    fn the_extruded_region_is_picked_by_a_point_in_the_metal() {
        let (_, calls) = commands(json!({"slot": "T-slot", "size": "2040", "length": 100}));
        let extrude = all(&calls, "solid.extrude")[0];
        let point = &extrude["profiles"][0]["point"];
        // In the lower cell's core, beside its bore.
        assert!(point[0].as_f64().unwrap() < -2.1 / 2.0);
        assert!(point[1].as_f64().unwrap() < -10.0 - 2.1 / 2.0);
        assert!(point[1].as_f64().unwrap() > -10.0 - 4.0);
    }

    #[test]
    fn names_say_the_slot_the_size_and_the_length() {
        let p = rail::parse(&json!({"slot": "V-slot", "size": "2040", "length": 500})).unwrap();
        assert_eq!(body_base(&p), "V-slot 2040 x 500");
        assert_eq!(
            description(&p),
            "20 x 40 mm V-slot aluminium extrusion, 500 mm long"
        );
        let p = rail::parse(&json!({"size": "2020", "length": 122.5})).unwrap();
        assert_eq!(body_base(&p), "T-slot 2020 x 122.5");
    }

    #[test]
    fn the_rail_is_a_purchased_part_in_the_extrusions_folder() {
        struct All(Vec<(String, Value)>);
        impl Runner for All {
            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                self.0.push((command.to_string(), args));
                Ok(component::stand_in(command).unwrap_or(match command {
                    "selection.get" => json!({"count": 0, "items": []}),
                    "sketch.create" => json!({"sketch": 1}),
                    _ => Value::Null,
                }))
            }
        }
        let p = rail::parse(&json!({"slot": "T-slot", "size": "2040", "length": 300})).unwrap();
        let mut host = All(vec![]);
        make_extrusion(&mut host, &p).unwrap();
        let created = all(&host.0, "component.create");
        assert_eq!(created[0]["name"], FOLDER);
        assert_eq!(created[1]["name"], "T-slot 2040 x 300");
        let meta = all(&host.0, "component.metadata.set");
        let rail = meta.iter().find(|m| m["part_number"].is_string()).unwrap();
        assert_eq!(rail["part_number"], "T-slot 2040 x 300");
        assert_eq!(rail["bom_type"], "purchased");
        assert_eq!(
            rail["description"],
            "20 x 40 mm T-slot aluminium extrusion, 300 mm long"
        );
    }
}
