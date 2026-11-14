//! 20 mm series T-slot and V-slot aluminium extrusions for SolidCraft.
//!
//! Two entries. `make-extrusion` builds a rail: its section is centred on a sketch
//! point, or on the origin when nothing is selected, and the rail runs out of the
//! sketch for its length. The section is one sketch of lines and arcs, extruded once.
//! `make-t-nut` puts a slot nut under the lips of a rail: see [`tnut`].

pub mod rail;
pub mod tnut;

use extrusions::machining::{self, TAP_DEPTH, TAP_THREAD};
use extrusions::{Profile, Seg};
use plugin_kit::component::{self, Definition, Placed};
use plugin_kit::host::sketch_id;
use plugin_kit::name;
use plugin_kit::place::{self, Placement};
use plugin_kit::Runner;
use rail::{Params, Taps};
use serde_json::{json, Value};

/// The plugin id, which a rail component's metadata names.
pub const PLUGIN: &str = "techsquid-extrusions";

/// The component every rail placement goes in.
pub const FOLDER: &str = "Extrusions";

/// The body name before any number: `T-slot 2040 x 500`. What is cut into the rail
/// follows, because a tapped or drilled rail is another part: `T-slot 2020 x 300 taps
/// both holes -Y 10 290`.
pub fn body_base(p: &Params) -> String {
    let mut base = format!("{} {}", p.slot.name, name::sized(&p.size, p.length));
    if p.taps != Taps::None {
        base.push_str(&format!(" taps {}", p.taps.label().to_lowercase()));
    }
    if !p.holes.is_empty() {
        base.push_str(&format!(" holes {}", p.hole_face.name()));
        for at in &p.holes {
            base.push_str(&format!(" {}", name::length(*at)));
        }
    }
    base
}

/// What the bill of materials says a rail is.
pub fn description(p: &Params) -> String {
    let mut text = format!(
        "{} x {} mm {} aluminium extrusion, {} mm long",
        name::length(p.grid.width()),
        name::length(p.grid.height()),
        p.slot.name,
        name::length(p.length)
    );
    match p.taps {
        Taps::None => {}
        Taps::Both => text.push_str(&format!(", both ends tapped {TAP_THREAD}")),
        one => text.push_str(&format!(
            ", {} tapped {TAP_THREAD}",
            one.label().to_lowercase()
        )),
    }
    if !p.holes.is_empty() {
        let at: Vec<String> = p.holes.iter().map(|at| name::length(*at)).collect();
        text.push_str(&format!(
            ", M5 screw holes from {} at {} mm",
            p.hole_face.name(),
            at.join(", ")
        ));
    }
    text
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
        site: component::ORIGIN,
    };
    let placed = component::place_all(r, &def, &places, |mut r, name, site| {
        build_rail(&mut r, &profile, p, site, name)
    })?;
    Ok(summary(p, &profile, &places, &placed))
}

/// One rail named `body_name`, its section centred on `place` and its length along the
/// normal, with its taps and screw holes.
pub fn build_rail<R: Runner>(
    r: &mut R,
    profile: &Profile,
    p: &Params,
    place: &Placement,
    body_name: &str,
) -> Result<(), String> {
    let length = p.length;
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
    // A point of the section, `along` the rail from its start.
    let at = |point: [f64; 2], along: f64| -> [f64; 3] {
        std::array::from_fn(|i| {
            place.point[i]
                + point[0] * place.x_dir[i]
                + point[1] * place.y_dir[i]
                + along * place.normal[i]
        })
    };
    for along in &p.holes {
        let holes = machining::cross_holes(&p.slot, &p.grid, p.hole_face);
        for (n, hole) in holes.iter().enumerate() {
            let axis: [f64; 3] = std::array::from_fn(|i| {
                hole.inward[0] * place.x_dir[i] + hole.inward[1] * place.y_dir[i]
            });
            let label = if holes.len() > 1 {
                format!("Hole at {} ({})", name::length(*along), n + 1)
            } else {
                format!("Hole at {}", name::length(*along))
            };
            // The head goes through the lips and sits on the slot floor.
            let (access, seat) = hole.access;
            r.run(
                "solid.cylinder",
                json!({
                    "diameter": access,
                    "height": seat,
                    "base": at(hole.at, *along),
                    "axis": axis,
                    "operation": "cut",
                    "targets": [body_name],
                    "name": name::part(body_name, &format!("{label}: Access")),
                }),
            )?;
            let (clearance, start, run) = hole.clearance;
            let from = [
                hole.at[0] + start * hole.inward[0],
                hole.at[1] + start * hole.inward[1],
            ];
            r.run(
                "solid.cylinder",
                json!({
                    "diameter": clearance,
                    "height": run,
                    "base": at(from, *along),
                    "axis": axis,
                    "operation": "cut",
                    "targets": [body_name],
                    "name": name::part(body_name, &format!("{label}: Clearance")),
                }),
            )?;
        }
    }
    // The bore is the tap drill already, so a tap is listed and cuts nothing.
    let ends = [
        (p.taps.start(), "start", 0.0, place.normal),
        (p.taps.end(), "end", length, place::opposite(place.normal)),
    ];
    for (tapped, end, along, into) in ends {
        if !tapped {
            continue;
        }
        for (n, bore) in p.grid.centres().iter().enumerate() {
            r.run(
                "solid.hole",
                json!({
                    "position": at(*bore, along),
                    "direction": into,
                    "depth": TAP_DEPTH,
                    "thread": TAP_THREAD,
                    "modeled": false,
                    "targets": [body_name],
                    "name": name::part(body_name, &format!("Tap {end} {}", n + 1)),
                }),
            )?;
        }
    }
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
        "end_taps": p.taps.label(),
        "holes": p.holes,
        "hole_face": p.hole_face.name(),
        "count": places.len(),
        "placed": places[0].source.words(),
        "points": places.iter().map(|place| place.point).collect::<Vec<_>>(),
        "component": placed.json(),
    })
}

plugin_kit::guest! {
    manifest: include_str!("../plugin.json"),
    aliases: &[],
    entries: [
        {
            entry: "make-extrusion",
            noun: "extrusion",
            parse: crate::rail::parse,
            make: crate::make_extrusion,
            describe: |p: &crate::rail::Params| format!("making a {}", crate::body_base(p)),
        },
        {
            entry: "make-t-nut",
            noun: "nut",
            parse: crate::tnut::parse,
            make: crate::tnut::make_t_nut,
            describe: |p: &crate::tnut::Params| format!("making a {}", crate::tnut::body_base(p)),
        },
    ],
}

#[cfg(test)]
mod tests {
    use super::*;
    use extrusions::profile::SIZES;

    struct Rec {
        calls: Vec<(String, Value)>,
    }
    impl Runner for Rec {
        fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
            plugin_kit::Placement::on_origin()
        }

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
        assert_eq!(
            schema["properties"]["end_taps"]["enum"],
            json!(["None", "Start", "End", "Both"])
        );
        assert_eq!(
            schema["properties"]["end_taps"]["default"],
            rail::DEFAULT_TAPS
        );
        let faces: Vec<&str> = extrusions::machining::FACES
            .iter()
            .map(|f| f.name())
            .collect();
        assert_eq!(schema["properties"]["hole_face"]["enum"], json!(faces));
        assert_eq!(
            schema["properties"]["hole_face"]["default"],
            rail::DEFAULT_HOLE_FACE
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
            json!({"size": "2020", "length": 100, "end_taps": "Both", "holes": "50"}),
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

    /// The host joins pieces that share a point exactly, so the commands as sent must
    /// chain end to start with no gap at all: across the corner arcs, the roundings of a
    /// T-slot floor, and the bore arcs of a 4040.
    #[test]
    fn every_piece_starts_where_the_piece_before_it_ended() {
        for v in [
            json!({"slot": "V-slot", "size": "2040", "length": 100}),
            json!({"slot": "T-slot", "size": "2040", "length": 100}),
            json!({"slot": "T-slot", "size": "4040", "length": 100}),
        ] {
            let (_, calls) = commands(v.clone());
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
                        assert_eq!(last, first, "{v}");
                        first = Some(start);
                        loops += 1;
                    }
                }
                last = Some(end);
            }
            assert_eq!(last, first, "{v}");
            // The outline and one cavity.
            assert_eq!(loops, 2, "{v}");
        }
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
    fn a_machined_rail_says_so_in_its_name_and_its_description() {
        let p = rail::parse(&json!({
            "size": "2020", "length": 300, "end_taps": "Both",
            "holes": "10, 290", "hole_face": "-Y",
        }))
        .unwrap();
        assert_eq!(body_base(&p), "T-slot 2020 x 300 taps both holes -Y 10 290");
        assert_eq!(
            description(&p),
            "20 x 20 mm T-slot aluminium extrusion, 300 mm long, both ends tapped M5, M5 screw holes from -Y at 10, 290 mm"
        );
        let p = rail::parse(&json!({"size": "2040", "length": 300, "end_taps": "Start"})).unwrap();
        assert_eq!(body_base(&p), "T-slot 2040 x 300 taps start");
        assert!(description(&p).ends_with("300 mm long, start tapped M5"));
        // A plain rail keeps the name it had.
        let p = rail::parse(&json!({"size": "2020", "length": 300})).unwrap();
        assert_eq!(body_base(&p), "T-slot 2020 x 300");
    }

    #[test]
    fn a_screw_hole_is_an_access_hole_and_a_clearance_hole_cut_from_the_face() {
        let (summary, calls) = commands(json!({
            "size": "2020", "length": 300, "holes": "10, 290", "hole_face": "-Y",
        }));
        let cuts = all(&calls, "solid.cylinder");
        assert_eq!(cuts.len(), 4);
        let name = "T-slot 2020 x 300 holes -Y 10 290";
        // The first hole is 10 along the rail, in from the -Y face.
        assert_eq!(cuts[0]["name"], format!("{name}: Hole at 10: Access"));
        assert_eq!(cuts[0]["diameter"], 9.5);
        assert_eq!(cuts[0]["height"], 6.0);
        assert_eq!(cuts[0]["base"], json!([0.0, -10.0, 10.0]));
        assert_eq!(cuts[0]["axis"], json!([0.0, 1.0, 0.0]));
        assert_eq!(cuts[1]["name"], format!("{name}: Hole at 10: Clearance"));
        assert_eq!(cuts[1]["diameter"], 5.5);
        assert_eq!(cuts[1]["height"], 14.0);
        assert_eq!(cuts[1]["base"], json!([0.0, -4.0, 10.0]));
        assert_eq!(cuts[2]["base"], json!([0.0, -10.0, 290.0]));
        for cut in &cuts {
            assert_eq!(cut["operation"], "cut");
            assert_eq!(cut["targets"], json!([name]));
        }
        assert!(all(&calls, "solid.hole").is_empty());
        assert_eq!(summary["holes"], json!([10.0, 290.0]));
        assert_eq!(summary["hole_face"], "-Y");
        // A 2040's wide face has two slots, so each position gets two holes.
        let (_, calls) = commands(json!({
            "size": "2040", "length": 100, "holes": "50", "hole_face": "+X",
        }));
        let cuts = all(&calls, "solid.cylinder");
        assert_eq!(cuts.len(), 4);
        assert_eq!(cuts[0]["base"], json!([10.0, -10.0, 50.0]));
        assert_eq!(cuts[2]["base"], json!([10.0, 10.0, 50.0]));
        assert_eq!(
            cuts[2]["name"],
            "T-slot 2040 x 100 holes +X 50: Hole at 50 (2): Access"
        );
    }

    #[test]
    fn a_tapped_end_lists_an_m5_thread_in_every_bore_and_cuts_nothing() {
        let (summary, calls) = commands(json!({"size": "2040", "length": 300, "end_taps": "Both"}));
        let taps = all(&calls, "solid.hole");
        assert_eq!(taps.len(), 4);
        let name = "T-slot 2040 x 300 taps both";
        assert_eq!(taps[0]["name"], format!("{name}: Tap start 1"));
        assert_eq!(taps[0]["position"], json!([0.0, -10.0, 0.0]));
        assert_eq!(taps[0]["direction"], json!([0.0, 0.0, 1.0]));
        assert_eq!(taps[1]["position"], json!([0.0, 10.0, 0.0]));
        assert_eq!(taps[2]["name"], format!("{name}: Tap end 1"));
        assert_eq!(taps[2]["position"], json!([0.0, -10.0, 300.0]));
        assert_eq!(taps[2]["direction"], json!([0.0, 0.0, -1.0]));
        for tap in &taps {
            assert_eq!(tap["thread"], "M5");
            assert_eq!(tap["depth"], 12.0);
            assert_eq!(tap["modeled"], false);
            assert_eq!(tap["targets"], json!([name]));
        }
        assert!(all(&calls, "solid.cylinder").is_empty());
        assert_eq!(summary["end_taps"], "Both");
        let (_, calls) = commands(json!({"size": "2020", "length": 300, "end_taps": "End"}));
        assert_eq!(all(&calls, "solid.hole").len(), 1);
    }

    #[test]
    fn the_rail_is_a_purchased_part_in_the_extrusions_folder() {
        struct All(Vec<(String, Value)>);
        impl Runner for All {
            fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
                plugin_kit::Placement::on_origin()
            }

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
