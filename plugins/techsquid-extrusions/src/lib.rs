//! 20 mm series T-slot and V-slot aluminium extrusions for SolidCraft.
//!
//! Two entries. `make-extrusion` builds a rail: its section is centred on a sketch
//! point, or on the origin when nothing is selected, and the rail runs out of the
//! sketch for its length. The section is one sketch of lines and arcs, extruded once.
//! Sketch-driven mode sweeps the same section along straight sketch lines, with
//! path-derived machining planes: see [`path`].
//! `make-t-nut` puts a slot nut under the lips of a rail: see [`tnut`].

pub mod path;
pub mod provider;
pub mod rail;
pub mod tnut;

use extrusions::machining::{self, TAP_DEPTH, TAP_THREAD};
use extrusions::{Profile, Seg};
use plugin_kit::component::{self, Definition, Placed, Site};
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
        base.push_str(" holes");
        base.push_str(&holes_by_face(p, " ", " "));
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
        text.push_str(", M5 screw holes from");
        text.push_str(&holes_by_face(p, " at ", ", "));
        text.push_str(" mm");
    }
    text
}

/// The holes as words, face by face: each face's name, `after_face`, and its distances
/// with `between` between them.
fn holes_by_face(p: &Params, after_face: &str, between: &str) -> String {
    let mut text = String::new();
    let mut last = None;
    for (face, at) in &p.holes {
        if last == Some(*face) {
            text.push_str(between);
        } else {
            if last.is_some() {
                text.push_str(if between == " " { "" } else { " and from" });
            }
            text.push_str(&format!(" {}{after_face}", face.name()));
            last = Some(*face);
        }
        text.push_str(&name::length(*at));
    }
    text
}

/// Where a rail is built. A rail with a thread cut into an end is built away from the
/// origin and from every other such rail: see [`plugin_kit::component`]. Any other rail
/// is built on the origin.
pub fn built_at(p: &Params) -> Site {
    if p.taps == Taps::None || !p.modeled {
        Site::Origin
    } else {
        Site::Apart
    }
}

/// Counts the commands a build sends, with no host behind it. Every sketch it
/// creates is sketch 1, which is all a build reads from a reply.
#[derive(Default)]
pub(crate) struct CallCount(pub(crate) usize);
impl Runner for CallCount {
    fn run(&mut self, _: &str, _: Value) -> Result<Value, String> {
        self.0 += 1;
        Ok(json!({"sketch": 1}))
    }
    fn site(&mut self, _: [f64; 2]) -> Placement {
        Placement::on_origin()
    }
}

/// What the library spends on a new definition beside its build: naming it, its
/// metadata and the record that lets a later run reuse it. Through the host a plain
/// rail took 18 and a tapped rail drilled from two faces took 30.
const SEAL_CALLS: usize = 48;

/// The longest build one run has room for. The library stops a run at 248 commands,
/// and reading the design, one placement and the reply take 20 of them.
const MAX_BUILD_CALLS: usize = 248 - 20 - SEAL_CALLS;

/// The commands `build_rail` sends for this rail.
fn build_calls(profile: &Profile, p: &Params) -> Result<usize, String> {
    let mut count = CallCount::default();
    build_rail(&mut count, profile, p, &Placement::on_origin(), "Rail")?;
    Ok(count.0)
}

/// Build fixed-length components on points or independent rails on sketch lines.
pub fn make_extrusion<R: Runner>(r: &mut R, p: &Params) -> Result<Value, String> {
    if let Some(path) = &p.path {
        return path::make(r, p, path);
    }
    let profile = Profile::new(p.slot, p.grid).ok_or("size: this size has no section")?;
    // Every hole is two circles in each slot it crosses, so a wide rail with many
    // holes can ask for more than one run is allowed. Say so before building any of it.
    let build = build_calls(&profile, p)?;
    if build > MAX_BUILD_CALLS {
        return Err(format!(
            "holes: {} holes in this rail take {build} commands and one run has room for {MAX_BUILD_CALLS}; drill fewer holes",
            p.holes.len()
        ));
    }
    let places = place::resolve_each(r, &p.place)?;
    let base = body_base(p);
    let description = description(p);
    let def = Definition {
        plugin: PLUGIN,
        base: &base,
        description: &description,
        bom_type: "normal",
        site: built_at(p),
    };
    let representation = provider::rail_representation(p);
    let number = component::model_number(&representation.product)?;
    let (placed, report) = component::insert(
        r,
        build + SEAL_CALLS,
        &places,
        FOLDER,
        |library| {
            library.ensure_built(&def, &number, &representation, |mut r, site, name| {
                build_rail(&mut r, &profile, p, site, name)
            })
        },
        |_, _| Ok(()),
    )?;
    let mut reply = summary(p, &profile, &places, &placed);
    reply["library"] = report;
    Ok(reply)
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
    draw_profile(r, profile)?;
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
    drill_cross_holes(r, p, place, body_name)?;
    // The bore is the tap drill already, so a tap only cuts the thread, or lists it.
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
                    // The bore runs the length of the rail, and the tap does not.
                    "thread_depth": TAP_DEPTH,
                    "thread": TAP_THREAD,
                    "modeled": p.modeled,
                    "targets": [body_name],
                    "name": name::part(body_name, &format!("Tap {end} {}", n + 1)),
                }),
            )?;
        }
    }
    Ok(())
}

/// One editable sketch per drilled face. Each pair of circles shares one centre
/// point, and both cuts select profiles by curve id so they follow sketch edits.
fn drill_cross_holes<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body_name: &str,
) -> Result<(), String> {
    for face in machining::FACES {
        let positions: Vec<f64> = p
            .holes
            .iter()
            .filter(|(f, _)| *f == face)
            .map(|(_, at)| *at)
            .collect();
        if positions.is_empty() {
            continue;
        }
        let out = face.outward();
        let across = [-out[1], out[0]];
        let origin: [f64; 3] = std::array::from_fn(|i| {
            place.point[i]
                + face.half(&p.grid) * (out[0] * place.x_dir[i] + out[1] * place.y_dir[i])
        });
        let x_dir: [f64; 3] =
            std::array::from_fn(|i| across[0] * place.x_dir[i] + across[1] * place.y_dir[i]);
        // X runs across the face and Y along the rail: their cross product faces out.
        let created = r.run(
            "sketch.create",
            json!({
                "plane": {"origin": origin, "x_dir": x_dir, "y_dir": place.normal},
                "name": name::part(body_name, &format!("Hole references {}", face.name())),
            }),
        )?;
        let sketch = sketch_id(&created)?;
        let holes = machining::cross_holes(&p.slot, &p.grid, face);
        let mut clearance_profiles = Vec::new();
        let mut access_profiles = Vec::new();
        for along in positions {
            for hole in &holes {
                let id = format!("hole{}", clearance_profiles.len() + 1);
                r.run(
                    "sketch.circle.center",
                    json!({
                        "id": id,
                        "center": [hole.at[0] * across[0] + hole.at[1] * across[1], along],
                        "diameter": hole.clearance.0,
                    }),
                )?;
                let access = format!("access{}", clearance_profiles.len() + 1);
                r.run(
                    "sketch.circle.center",
                    json!({
                        "id": access,
                        "center": format!("{id}.center"),
                        "diameter": hole.access.0,
                    }),
                )?;
                clearance_profiles.push(json!({"curves": [id]}));
                access_profiles.push(json!({"curves": [access]}));
            }
        }
        r.run("sketch.finish", json!({}))?;
        let hole = &holes[0];
        // Hole features start at the first material on their axis, below the open
        // slot. Extrusions start on this face plane, keeping the access depth correct.
        // The access profiles are annuli; the clearance cut removes their centres.
        for (label, profiles, depth) in [
            ("Access", access_profiles, hole.access.1),
            ("Clearance", clearance_profiles, hole.clearance.2),
        ] {
            r.run(
                "solid.extrude",
                json!({
                    "sketch": sketch,
                    "profiles": profiles,
                    "distance": depth,
                    "direction": "negative",
                    "operation": "cut",
                    "targets": [body_name],
                    "name": name::part(body_name, &format!("Screw holes {}: {label}", face.name())),
                }),
            )?;
        }
        r.run(
            "browser.visibility",
            json!({"items": [sketch], "visible": false}),
        )?;
    }
    Ok(())
}

/// Draw the shared catalog section in the active sketch.
fn draw_profile<R: Runner>(r: &mut R, profile: &Profile) -> Result<(), String> {
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
    let reply = json!({
        "slot": p.slot.name,
        "size": p.size,
        "length": p.length,
        "width": p.grid.width(),
        "height": p.grid.height(),
        "area_mm2": (profile.area() * 100.0).round() / 100.0,
        "end_taps": p.taps.label(),
        "holes": p.holes.iter().map(|(face, at)| json!({"face": face.name(), "at": at})).collect::<Vec<_>>(),
        "thread_modeled": p.modeled,
    });
    component::with_placements(reply, places, Some(placed))
}

plugin_kit::guest! {
    manifest: include_str!("../plugin.json"),
    entries: [
        {
            entry: "make-extrusion",
            noun: "extrusion",
            parse: crate::rail::parse,
            make: crate::make_extrusion,
            describe: |p: &crate::rail::Params| if p.path.is_some() { format!("making sketch-driven {} {} rails", p.slot.name, p.size) } else { format!("making a {}", crate::body_base(p)) },
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
                "sketch.create" => {
                    json!({"sketch": self.calls.iter().filter(|(c, _)| c == "sketch.create").count() + 1})
                }
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

    #[test]
    fn the_counted_build_is_the_build_the_host_gets() {
        for v in [
            json!({"slot": "T-slot", "size": "2020", "length": 300}),
            json!({"slot": "T-slot", "size": "2040", "length": 300, "end_taps": "Both",
                "holes": "+Y 20 80 +X 10 90"}),
            json!({"slot": "V-slot", "size": "2040", "length": 120}),
        ] {
            let p = rail::parse(&v).unwrap();
            let profile = Profile::new(p.slot, p.grid).unwrap();
            let mut rec = Rec { calls: vec![] };
            build_rail(&mut rec, &profile, &p, &Placement::on_origin(), "Rail").unwrap();
            assert_eq!(build_calls(&profile, &p).unwrap(), rec.calls.len(), "{v}");
        }
    }

    #[test]
    fn a_rail_with_more_holes_than_a_run_has_room_for_is_refused_on_holes() {
        // 19 places across the four slots of a 2080 are 152 circles.
        let holes = (1..=19)
            .map(|i| (50 * i).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let many = json!({"slot": "T-slot", "size": "2080", "length": 1000,
            "holes": format!("+X {holes}")});
        let p = rail::parse(&many).unwrap();
        let mut rec = Rec { calls: vec![] };
        let err = make_extrusion(&mut rec, &p).unwrap_err();
        assert!(err.starts_with("holes: 19 holes"), "{err}");
        // Nothing was sent: the rail is refused before the design is touched.
        assert!(rec.calls.is_empty());
        // Twelve of them fit, as they do through the host.
        let fewer = json!({"slot": "T-slot", "size": "2080", "length": 1000,
            "holes": "+X 50 100 150 200 250 300 350 400 450 500 550 600"});
        let p = rail::parse(&fewer).unwrap();
        assert!(make_extrusion(&mut Rec { calls: vec![] }, &p).is_ok());
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
        // Two entries make a submenu named after the plugin, holding each entry's name.
        assert_eq!(manifest["name"], "Extrusions");
        let description = manifest["description"].as_str().unwrap();
        assert!(description.chars().count() <= 240, "{description}");
        let entries = &manifest["runtime"]["entries"];
        assert_eq!(
            entries[0]["slots"],
            json!([{"kind": "menu", "name": "Extrusion"}])
        );
        assert_eq!(
            entries[1]["slots"],
            json!([{"kind": "menu", "name": "Slot nut"}])
        );
        let entry = &entries[0];
        assert_eq!(entry["name"], "make-extrusion");
        let schema = &entry["input_schema"];
        assert_eq!(schema["required"], json!(["slot", "size"]));
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
        assert_eq!(schema["properties"]["thread_modeled"]["default"], true);
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
        for command in component::commands() {
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
        // Placing on a picked point reads its sketch.
        for command in ["sketch.inspect", "inspect.measure"] {
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
            "on the origin, because no point was picked"
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
        let p = rail::parse(&json!({
            "size": "2020", "length": 300, "holes": "-Y 10 290 +X 150",
        }))
        .unwrap();
        assert_eq!(body_base(&p), "T-slot 2020 x 300 holes -Y 10 290 +X 150");
        assert!(
            description(&p).ends_with("M5 screw holes from -Y at 10, 290 and from +X at 150 mm")
        );
        // A plain rail keeps the name it had.
        let p = rail::parse(&json!({"size": "2020", "length": 300})).unwrap();
        assert_eq!(body_base(&p), "T-slot 2020 x 300");
    }

    #[test]
    fn screw_holes_share_editable_centres_and_profile_references() {
        let (summary, calls) = commands(json!({
            "size": "2020", "length": 300, "holes": "10, 290", "hole_face": "-Y",
        }));
        let name = "T-slot 2020 x 300 holes -Y 10 290";
        let sketches = all(&calls, "sketch.create");
        assert_eq!(sketches.len(), 2);
        assert_eq!(sketches[1]["name"], format!("{name}: Hole references -Y"));
        assert_eq!(
            sketches[1]["plane"],
            json!({
                "origin": [0.0, -10.0, 0.0], "x_dir": [1.0, 0.0, 0.0], "y_dir": [0.0, 0.0, 1.0],
            })
        );
        let circles = all(&calls, "sketch.circle.center");
        assert_eq!(circles[1]["id"], "hole1");
        assert_eq!(circles[1]["center"], json!([0.0, 10.0]));
        assert_eq!(circles[1]["diameter"], 5.5);
        assert_eq!(circles[2]["id"], "access1");
        assert_eq!(circles[2]["center"], "hole1.center");
        assert_eq!(circles[2]["diameter"], 9.5);
        assert_eq!(circles[3]["center"], json!([0.0, 290.0]));
        assert_eq!(circles[4]["center"], "hole2.center");
        let extrudes = all(&calls, "solid.extrude");
        assert_eq!(extrudes.len(), 3);
        assert_eq!(
            extrudes[1]["name"],
            format!("{name}: Screw holes -Y: Access")
        );
        assert_eq!(
            extrudes[1]["profiles"],
            json!([{"curves": ["access1"]}, {"curves": ["access2"]}])
        );
        assert_eq!(extrudes[1]["distance"], 6.0);
        assert_eq!(
            extrudes[2]["profiles"],
            json!([{"curves": ["hole1"]}, {"curves": ["hole2"]}])
        );
        assert_eq!(extrudes[2]["distance"], 20.0);
        for cut in &extrudes[1..] {
            assert_eq!(cut["sketch"], 2);
            assert_eq!(cut["direction"], "negative");
            assert_eq!(cut["operation"], "cut");
            assert_eq!(cut["targets"], json!([name]));
        }
        assert!(all(&calls, "solid.cylinder").is_empty());
        assert!(all(&calls, "solid.hole").is_empty());
        assert_eq!(
            all(&calls, "browser.visibility"),
            vec![&json!({"items": [2], "visible": false})]
        );
        assert_eq!(
            summary["holes"],
            json!([{"face": "-Y", "at": 10.0}, {"face": "-Y", "at": 290.0}])
        );
    }

    #[test]
    fn each_drilled_face_has_one_sketch_with_every_slot_and_correct_frame() {
        let p = rail::parse(
            &json!({"size": "2040", "length": 200, "holes": "+Y 25 -Y 75 +X 125 -X 175"}),
        )
        .unwrap();
        let profile = Profile::new(p.slot, p.grid).unwrap();
        // An offset, rotated build frame, as used for a tapped rail's build site.
        let place = Placement {
            point: [100.0, 200.0, 300.0],
            x_dir: [0.0, 1.0, 0.0],
            y_dir: [0.0, 0.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            ..Placement::on_origin()
        };
        let mut rec = Rec { calls: vec![] };
        build_rail(&mut rec, &profile, &p, &place, "Rail").unwrap();
        let sketches = all(&rec.calls, "sketch.create");
        assert_eq!(sketches.len(), 5);
        let origins = [
            [100.0, 200.0, 320.0],
            [100.0, 200.0, 280.0],
            [100.0, 210.0, 300.0],
            [100.0, 190.0, 300.0],
        ];
        let across = [
            [0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        let cuts = all(&rec.calls, "solid.extrude");
        for i in 0..4 {
            assert_eq!(sketches[i + 1]["plane"]["origin"], json!(origins[i]));
            assert_eq!(sketches[i + 1]["plane"]["x_dir"], json!(across[i]));
            assert_eq!(sketches[i + 1]["plane"]["y_dir"], json!(place.normal));
            assert_eq!(cuts[2 * i + 1]["sketch"], i + 2);
            assert_eq!(cuts[2 * i + 2]["sketch"], i + 2);
            assert_eq!(cuts[2 * i + 2]["distance"], if i < 2 { 40.0 } else { 20.0 });
            assert_eq!(
                cuts[2 * i + 2]["profiles"].as_array().unwrap().len(),
                if i < 2 { 1 } else { 2 }
            );
        }
        let circles = all(&rec.calls, "sketch.circle.center");
        assert_eq!(circles[6]["center"], json!([-10.0, 125.0]));
        assert_eq!(circles[8]["center"], json!([10.0, 125.0]));
        assert_eq!(circles[10]["center"], json!([10.0, 175.0]));
        assert_eq!(circles[12]["center"], json!([-10.0, 175.0]));
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
            assert_eq!(tap["thread_depth"], 12.0);
            assert_eq!(tap["modeled"], true);
            assert_eq!(tap["targets"], json!([name]));
        }
        assert!(all(&calls, "solid.cylinder").is_empty());
        assert_eq!(summary["end_taps"], "Both");
        let (summary, calls) = commands(json!({
            "size": "2020", "length": 300, "end_taps": "End", "thread_modeled": false,
        }));
        let taps = all(&calls, "solid.hole");
        assert_eq!(taps.len(), 1);
        assert_eq!(taps[0]["modeled"], false);
        assert_eq!(summary["thread_modeled"], false);
    }

    #[test]
    fn a_rail_with_a_cut_thread_is_built_on_a_site_of_its_own() {
        let site = |v: Value| built_at(&rail::parse(&v).unwrap());
        // A plain rail, and one whose thread is only listed, stay on the origin.
        assert_eq!(site(json!({"length": 300})), Site::Origin);
        assert_eq!(site(json!({"length": 300, "holes": "10"})), Site::Origin);
        assert_eq!(
            site(json!({"length": 300, "end_taps": "Both", "thread_modeled": false})),
            Site::Origin
        );
        assert_eq!(
            site(json!({"length": 300, "end_taps": "Start"})),
            Site::Apart
        );
    }

    #[test]
    fn a_finished_rail_is_a_custom_part_in_the_extrusions_folder() {
        struct All(Vec<(String, Value)>);
        impl Runner for All {
            fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
                plugin_kit::Placement::on_origin()
            }

            fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
                self.0.push((command.to_string(), args));
                Ok(component::stand_in(command).unwrap_or(match command {
                    "sketch.create" => json!({"sketch": 1}),
                    _ => Value::Null,
                }))
            }
        }
        let p = rail::parse(&json!({"slot": "T-slot", "size": "2040", "length": 300})).unwrap();
        let mut host = All(vec![]);
        make_extrusion(&mut host, &p).unwrap();
        let created = all(&host.0, "component.create");
        assert!(created.iter().any(|c| c["name"] == FOLDER));
        assert!(created.iter().any(|c| c["name"] == "T-slot 2040 x 300"));
        let meta = all(&host.0, "component.metadata.set");
        let rail = meta.iter().find(|m| m["part_number"].is_string()).unwrap();
        assert!(rail["part_number"]
            .as_str()
            .unwrap()
            .starts_with("Custom nfs5-2040 ["));
        assert_eq!(rail["bom_type"], "normal");
        assert_eq!(
            rail["description"],
            "20 x 40 mm T-slot aluminium extrusion, 300 mm long"
        );
    }
}
