//! The `make-t-nut` entry: a slot nut under the lips of a rail. Sizes are millimetres.
//!
//! The point is on the rail's outer face, in the middle of a slot. The nut hangs under
//! the lips there with its tapped hole on the point, and its length runs along the rail.

use extrusions::slot::{Slot, T5, V5};
use extrusions::tnut::{TNut, NUTS, THREADS};
use plugin_kit::component::{self, Definition, Placed, Site};
use plugin_kit::host::sketch_id;
use plugin_kit::input::{self, FieldError};
use plugin_kit::name;
use plugin_kit::place::{self, opposite, PlaceInput, Placement};
use plugin_kit::Runner;
use serde_json::{json, Value};

use crate::rail::DEFAULT_SLOT;
use crate::{draw_loop, FOLDER, PLUGIN};

pub const DEFAULT_TYPE: &str = "Pre-assembly";
pub const DEFAULT_SIZE: &str = "M5";
pub const DEFAULT_ALONG: &str = "Sketch X";

/// Where a nut is built: on a site of its own when its thread is cut, on the origin
/// when the thread is only listed. See [`plugin_kit::component`].
pub fn built_at(p: &Params) -> Site {
    Site::for_thread(p.modeled)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub nut: TNut,
    /// The thread, such as `M5`.
    pub thread: String,
    /// The rail the nut sits in. Its lip says how far under the face the shoulders are.
    pub slot: Slot,
    /// Whether the rail runs along the sketch Y. Otherwise it runs along the sketch X.
    pub along_y: bool,
    /// `solid.hole`'s `modeled`. The dialog calls this `thread_modeled`.
    pub modeled: bool,
    pub place: PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-t-nut")?;
    let nut = match input::choice(v, "type", DEFAULT_TYPE)? {
        "Pre-assembly" => NUTS[0],
        "Post-assembly" => NUTS[1],
        other => {
            return Err(FieldError::new(
                "type",
                format!("must be Pre-assembly or Post-assembly, not {other}"),
            ))
        }
    };
    let thread = input::choice(v, "size", DEFAULT_SIZE)?;
    if !THREADS.contains(&thread) {
        return Err(FieldError::new(
            "size",
            format!("must be one of {}, not {thread}", THREADS.join(", ")),
        ));
    }
    let slot = match input::choice(v, "slot", DEFAULT_SLOT)? {
        "T-slot" => T5,
        "V-slot" => V5,
        other => {
            return Err(FieldError::new(
                "slot",
                format!("must be T-slot or V-slot, not {other}"),
            ))
        }
    };
    let along_y = match input::choice(v, "along", DEFAULT_ALONG)? {
        "Sketch X" => false,
        "Sketch Y" => true,
        other => {
            return Err(FieldError::new(
                "along",
                format!("must be Sketch X or Sketch Y, not {other}"),
            ))
        }
    };
    let p = Params {
        nut,
        thread: thread.to_string(),
        slot,
        along_y,
        modeled: input::flag(v, "thread_modeled", true)?,
        place: place::parse_input(v)?,
    };
    parts::catalog::validate_selection(
        v,
        crate::provider::REVISION,
        &crate::provider::nut_representation(&p),
    )
    .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(p)
}

/// The body name before any number: `T-nut M5 pre-assembly`.
pub fn body_base(p: &Params) -> String {
    format!("T-nut {} {}", p.thread, p.nut.name)
}

/// The MISUMI part number: `HNTT5-5`.
pub fn part(p: &Params) -> String {
    format!("{}-{}", p.nut.part, p.thread.trim_start_matches('M'))
}

/// What the bill of materials says a nut is.
pub fn description(p: &Params) -> String {
    format!(
        "MISUMI {} {} insertion nut for a 6 mm slot",
        part(p),
        p.nut.name
    )
}

/// Where the nut's own frame goes for a point on the rail's face, so that the tapped
/// hole is on the point, the shoulders are under the lip and the nut's X is along the
/// rail.
pub fn seat(p: &Params, face: &Placement) -> Placement {
    let (x_dir, y_dir) = if p.along_y {
        (face.y_dir, opposite(face.x_dir))
    } else {
        (face.x_dir, face.y_dir)
    };
    Placement {
        point: face.along(-p.slot.lip),
        x_dir,
        y_dir,
        ..*face
    }
}

/// Build the nut on every point. Returns a summary of what was made.
pub fn make_t_nut<R: Runner>(r: &mut R, p: &Params) -> Result<Value, String> {
    let spots = place::resolve_all(r, &p.place)?;
    let faces: Vec<Placement> = if spots.is_empty() {
        vec![Placement::on_origin()]
    } else {
        spots.iter().map(|spot| spot.place).collect()
    };
    let seats: Vec<Placement> = faces.iter().map(|face| seat(p, face)).collect();
    let base = body_base(p);
    let description = description(p);
    let def = Definition {
        plugin: PLUGIN,
        base: &base,
        description: &description,
        bom_type: "purchased",
        site: built_at(p),
    };
    let mut library = component::Library::new(
        r,
        component::Budget {
            creation_calls: 128,
            placements: seats.len(),
        },
    )?;
    let handle = library.ensure(
        &def,
        &format!("MISUMI {}", part(p)),
        &crate::provider::nut_representation(p),
        |library, name, site| library.geometry(name, |mut r| build_nut(&mut r, p, site, name)),
    )?;
    let placed = library.place_all(&handle, &seats, FOLDER)?;
    let report = library.finish()?;
    let mut reply = summary(p, &faces, &placed);
    reply["library"] = report;
    Ok(reply)
}

/// One nut named `body_name`, standing on `site`, whose axes are the world's: the
/// tapped hole runs down along Z through the site, the shoulders are level with it and
/// the length is along X. The bar and its ball are made on the origin and moved to the
/// site before the hole is tapped: see [`plugin_kit::component`].
pub fn build_nut<R: Runner>(
    r: &mut R,
    p: &Params,
    site: &Placement,
    body_name: &str,
) -> Result<(), String> {
    let nut = &p.nut;
    let [x, y, z] = [0.0, 0.0, 0.0];
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": [x - nut.tap_at, y, z],
                "x_dir": [0.0, 1.0, 0.0],
                "y_dir": [0.0, 0.0, 1.0],
            },
            "name": name::part(body_name, "Section"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    draw_loop(r, &nut.section())?;
    r.run("sketch.finish", json!({}))?;
    r.run(
        "solid.extrude",
        json!({
            "sketch": sketch,
            "profiles": [{"point": nut.material_point()}],
            "distance": nut.length,
            "operation": "new",
            "body_name": body_name,
            "name": name::part(body_name, "Bar"),
        }),
    )?;
    if let Some(ball) = nut.ball {
        r.run(
            "solid.cylinder",
            json!({
                "radius": ball.recess,
                "height": ball.recess_depth,
                "base": [x + ball.from_tap, y, z - nut.body],
                "axis": "Z",
                "operation": "cut",
                "targets": [body_name],
                "name": name::part(body_name, "Ball recess"),
            }),
        )?;
        r.run(
            "solid.sphere",
            json!({
                "radius": ball.radius,
                "center": [x + ball.from_tap, y, z + ball.radius - ball.stands_out - nut.body],
                "operation": "join",
                "targets": [body_name],
                "name": name::part(body_name, "Ball"),
            }),
        )?;
    }
    if site.point != [x, y, z] {
        r.run(
            "solid.move",
            json!({
                "bodies": [body_name],
                "translate": site.point,
                "name": name::part(body_name, "Move to site"),
            }),
        )?;
    }
    let [x, y, z] = site.point;
    r.run(
        "solid.hole",
        json!({
            "position": [x, y, z + nut.boss_height],
            "direction": [0.0, 0.0, -1.0],
            "depth": nut.height(),
            "thread": p.thread,
            "modeled": p.modeled,
            "targets": [body_name],
            "name": name::part(body_name, "Thread"),
        }),
    )?;
    Ok(())
}

fn summary(p: &Params, faces: &[Placement], placed: &Placed) -> Value {
    let reply = json!({
        "type": p.nut.name,
        "part": part(p),
        "thread": p.thread,
        "modeled": p.modeled,
        "slot": p.slot.name,
        "along": if p.along_y { "Sketch Y" } else { "Sketch X" },
        "length": p.nut.length,
        "width": (p.nut.width() * 1000.0).round() / 1000.0,
        "height": (p.nut.height() * 1000.0).round() / 1000.0,
        "under_face": p.slot.lip,
    });
    component::with_placements(reply, faces, Some(placed))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rec {
        calls: Vec<(String, Value)>,
    }
    impl Runner for Rec {
        fn site(&mut self, _: [f64; 2]) -> plugin_kit::Placement {
            plugin_kit::Placement::on_origin()
        }

        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            if let Some(reply) = component::stand_in(command) {
                if command == "occurrence.move" {
                    self.calls.push((command.to_string(), args));
                }
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
        let p = parse(&v).unwrap();
        let mut rec = Rec { calls: vec![] };
        let summary = make_t_nut(&mut rec, &p).unwrap();
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
    fn an_empty_form_is_an_m5_pre_assembly_nut_in_a_t_slot() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.nut, NUTS[0]);
        assert_eq!(p.thread, "M5");
        assert_eq!(p.slot, T5);
        assert!(!p.along_y);
        assert!(p.modeled);
        assert_eq!(body_base(&p), "T-nut M5 pre-assembly");
        assert_eq!(part(&p), "HNTT5-5");
        // An empty choice is the default too.
        assert_eq!(
            parse(&json!({"type": "", "size": "", "along": ""})).unwrap(),
            p
        );
    }

    #[test]
    fn a_wrong_choice_is_refused_on_its_field() {
        for (key, value) in [
            ("type", json!("Hammer")),
            ("size", json!("M6")),
            ("slot", json!("U-slot")),
            ("along", json!("Sketch Z")),
            ("thread_modeled", json!("yes")),
        ] {
            let err = parse(&json!({ key: value })).unwrap_err();
            assert_eq!(err.field, key);
        }
    }

    #[test]
    fn the_manifest_lists_this_entry_and_every_command_sent() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let entry = &manifest["runtime"]["entries"][1];
        assert_eq!(entry["name"], "make-t-nut");
        let fields = &entry["input_schema"]["properties"];
        assert_eq!(
            fields["type"]["enum"],
            json!(["Pre-assembly", "Post-assembly"])
        );
        assert_eq!(fields["type"]["default"], DEFAULT_TYPE);
        assert_eq!(fields["size"]["enum"], json!(THREADS));
        assert_eq!(fields["size"]["default"], DEFAULT_SIZE);
        assert_eq!(fields["slot"]["enum"], json!(["T-slot", "V-slot"]));
        assert_eq!(fields["along"]["enum"], json!(["Sketch X", "Sketch Y"]));
        assert_eq!(fields["along"]["default"], DEFAULT_ALONG);
        assert_eq!(fields["thread_modeled"]["default"], true);
        let listed = manifest["runtime"]["commands"].as_array().unwrap();
        for kind in ["Pre-assembly", "Post-assembly"] {
            let (_, calls) = commands(json!({ "type": kind }));
            for (command, _) in &calls {
                assert!(listed.contains(&json!(command)), "{command}");
            }
        }
    }

    #[test]
    fn a_pre_assembly_nut_is_a_section_extruded_and_tapped() {
        let (summary, calls) = commands(json!({}));
        let sketch = all(&calls, "sketch.create");
        assert_eq!(sketch.len(), 1);
        assert_eq!(sketch[0]["name"], "T-nut M5 pre-assembly: Section");
        // The hole is on the site the nut is built on.
        let hole = all(&calls, "solid.hole");
        assert_eq!(hole.len(), 1);
        let (x, y) = (
            hole[0]["position"][0].as_f64().unwrap(),
            hole[0]["position"][1].as_f64().unwrap(),
        );
        assert_eq!(hole[0]["position"], json!([x, y, 0.8]));
        // The section is drawn across the slot, half the nut's length back from the hole.
        assert_eq!(sketch[0]["plane"]["origin"], json!([x - 5.0, y, 0.0]));
        assert_eq!(sketch[0]["plane"]["x_dir"], json!([0.0, 1.0, 0.0]));
        assert_eq!(sketch[0]["plane"]["y_dir"], json!([0.0, 0.0, 1.0]));
        // Two boss roundings, two wing roundings and two bottom roundings.
        assert_eq!(all(&calls, "sketch.arc.three_point").len(), 6);
        let extrude = all(&calls, "solid.extrude");
        assert_eq!(extrude.len(), 1);
        assert_eq!(extrude[0]["distance"], 10.0);
        assert_eq!(extrude[0]["body_name"], "T-nut M5 pre-assembly");
        assert_eq!(hole[0]["thread"], "M5");
        assert_eq!(hole[0]["modeled"], true);
        assert!((hole[0]["depth"].as_f64().unwrap() - 4.1).abs() < 1e-9);
        assert!(all(&calls, "solid.sphere").is_empty());
        assert_eq!(summary["part"], "HNTT5-5");
        assert_eq!(summary["component"]["component"], "T-nut M5 pre-assembly");
        assert_eq!(
            summary["placed"],
            "on the origin, because nothing was selected"
        );
    }

    #[test]
    fn a_post_assembly_nut_has_its_ball_in_a_recess() {
        let (summary, calls) = commands(json!({"type": "Post-assembly", "thread_modeled": false}));
        let hole = all(&calls, "solid.hole")[0];
        let (x, y) = (
            hole["position"][0].as_f64().unwrap(),
            hole["position"][1].as_f64().unwrap(),
        );
        assert_eq!(
            all(&calls, "sketch.create")[0]["plane"]["origin"],
            json!([x - 6.0, y, 0.0])
        );
        assert_eq!(all(&calls, "solid.extrude")[0]["distance"], 17.0);
        assert_eq!(all(&calls, "solid.hole")[0]["modeled"], false);
        let recess = all(&calls, "solid.cylinder");
        assert_eq!(recess.len(), 1);
        assert_eq!(recess[0]["operation"], "cut");
        assert_eq!(recess[0]["radius"], 2.1);
        assert_eq!(recess[0]["base"], json!([x + 7.5, y, -3.2]));
        let ball = all(&calls, "solid.sphere");
        assert_eq!(ball.len(), 1);
        assert_eq!(ball[0]["operation"], "join");
        assert_eq!(ball[0]["radius"], 2.0);
        assert_eq!(ball[0]["center"], json!([x + 7.5, y, -2.0]));
        assert_eq!(summary["part"], "HNTP5-5");
        assert_eq!(summary["length"], 17.0);
    }

    #[test]
    fn the_nut_sits_a_lip_under_the_face_and_turns_with_the_rail() {
        let face = Placement::on_origin();
        let p = parse(&json!({})).unwrap();
        let seat_x = seat(&p, &face);
        assert_eq!(seat_x.point, [0.0, 0.0, -2.0]);
        assert_eq!(seat_x.x_dir, [1.0, 0.0, 0.0]);
        let p = parse(&json!({"slot": "V-slot", "along": "Sketch Y"})).unwrap();
        let seat_y = seat(&p, &face);
        assert_eq!(seat_y.point, [0.0, 0.0, -1.8]);
        assert_eq!(seat_y.x_dir, [0.0, 1.0, 0.0]);
        assert_eq!(seat_y.y_dir, [-1.0, 0.0, 0.0]);
        assert_eq!(seat_y.normal, face.normal);
    }
}
