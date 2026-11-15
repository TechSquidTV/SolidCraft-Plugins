//! Screw generator for SolidCraft.
//!
//! One entry, `make-screw`. The top of the shank sits on a sketch point, or on the origin
//! when nothing is selected. The head stands on the outer side of that plane and the shank
//! runs the other way. A socket or cheese head is a cylinder, a button or pan head is its
//! half profile revolved, a countersunk head is a 90° cone with the land a flat head has
//! above it, and a hex head is a six-sided sketch with its top chamfered. A countersunk
//! screw's length includes the head, as its standards measure it. A Torx drive cuts an ISO 10664
//! recess into the outer face. A slotted drive cuts a straight slot across that face.
//! A Phillips drive cuts an ISO 4757 type H recess, sunk to the depth for that head.
//! A point is a cone on the end of a shorter shank. A cup is a
//! cone cut into the end face. `solid.thread` starts at the tip of the cylinder, which is
//! the shoulder when the end comes to a point.

pub mod screw;

use fasteners::head;
use fasteners::hex::{self, Hex};
use fasteners::phillips;
use fasteners::torx;
use fasteners::FOLDER;
use plugin_kit::component::{self as fasten, Definition, Placed, Site};
use plugin_kit::host::{body_name_of, changed_bodies, sketch_id};
use plugin_kit::name;
use plugin_kit::place::{self, opposite, Placement};
use plugin_kit::Runner;
use screw::{Head, Hole, HoleKind, Params, ResolvedHead, ResolvedTip};
use serde_json::{json, Value};

/// A designation we cannot size a shank from. Metric sizes carry their diameter.
pub const NO_DIAMETER: &str =
    "this size has no diameter to build the shank; use a metric size such as M3 or M8x1";

/// How far a tip cone overlaps the shank, or starts outside the end face.
/// A face that only touches is a weak boolean. The apex of a point stays put.
const TIP_OVERLAP: f64 = 0.02;

/// How far the slot runs past the head on each end, so both ends break out.
const SLOT_PAST: f64 = 0.5;

/// How far each Phillips wing runs past the wing-tip cone, so the intersect covers it.
const PHILLIPS_PAST: f64 = 0.5;

/// Build a screw on every named or selected point, or one on the origin. Returns a
/// summary of what was made.
pub fn make_screw<R: Runner>(r: &mut R, p: &Params) -> Result<Value, String> {
    if p.resolved.major.is_none() {
        return Err(format!("{}: {NO_DIAMETER}", p.size_field));
    }
    let spots = place::resolve_all(r, &p.place)?;
    let mut drilled: Vec<String> = Vec::new();
    // Screw and hole with nothing selected still makes the screw. Only a run that
    // would make nothing but a hole is refused.
    let skipped = spots.is_empty() && p.create.hole();
    let places: Vec<Placement> = if spots.is_empty() {
        if !p.create.screw() {
            return Err(place::nothing_selected(
                "holes",
                "Screw only for a loose screw on the origin",
            ));
        }
        vec![Placement::on_origin()]
    } else {
        // The holes go first: the screw would otherwise be a body under the point too.
        let sink = match &p.hole {
            Some(hole) => {
                for (sketch, points) in place::by_sketch(&spots) {
                    let reply = r.run("solid.hole", hole_args(p, hole, sketch, &points))?;
                    match changed_bodies(&reply) {
                        Some(bodies) if bodies.is_empty() => {
                            return Err("sketch_point: there is no part under the point to drill; put the sketch on the part, or set Create to Screw only".into());
                        }
                        Some(bodies) => {
                            for body in bodies {
                                if !drilled.contains(&body) {
                                    drilled.push(body);
                                }
                            }
                        }
                        None => {}
                    }
                }
                hole.sink
            }
            None => 0.0,
        };
        // A recessed head sits below the surface, so its top ends flush.
        spots
            .iter()
            .map(|spot| Placement {
                point: spot.place.along(-sink),
                ..spot.place
            })
            .collect()
    };
    let placed = if p.create.screw() {
        let base = body_base(p);
        let def = Definition {
            plugin: PLUGIN,
            base: &base,
            description: p.standard.unwrap_or(""),
            folder: FOLDER,
            bom_type: "purchased",
            site: built_at(p),
        };
        Some(fasten::place_all(r, &def, &places, |mut r, name, site| {
            build_screw(&mut r, p, site, name)
        })?)
    } else {
        None
    };
    Ok(summary(p, &places, &drilled, skipped, placed.as_ref()))
}

/// The body name before any number: what the screw is, its size, then its head, as a
/// parts list writes it: `Screw M3 x 12 socket`, `Set screw M3 x 6`, `Threaded rod M8 x 100`.
pub fn body_base(p: &Params) -> String {
    let size = name::sized(&p.designation, p.nominal_length);
    let head = match p.head {
        Head::Set => return format!("Set screw {size}"),
        Head::Rod => return format!("Threaded rod {size}"),
        Head::Socket => "socket",
        Head::Button => "button",
        Head::Countersunk => "countersunk",
        Head::Hex => "hex",
        Head::Pan => "pan",
        Head::Cheese => "cheese",
    };
    format!("Screw {size} {head}")
}

/// The hole feature's name: its kind and the screw size, `Clearance hole M3`. With Hole
/// only there is no screw body, so this is the only place the size shows.
pub fn hole_name(p: &Params, hole: &Hole) -> String {
    let kind = match hole.kind {
        HoleKind::Clearance { .. } => "Clearance hole",
        HoleKind::Counterbore { .. } => "Counterbore",
        HoleKind::Countersink { .. } => "Countersink",
        HoleKind::Tapped { .. } => "Tapped hole",
    };
    format!("{kind} {}", p.designation)
}

/// One hole feature for every point of `sketch`. No target is named: the host drills
/// the part under each point.
fn hole_args(p: &Params, hole: &Hole, sketch: &str, points: &[&str]) -> Value {
    let mut args = json!({
        "sketch": sketch,
        "points": points,
        "depth": hole.depth,
        "name": hole_name(p, hole),
    });
    match &hole.kind {
        HoleKind::Clearance { size } => {
            size.apply(&mut args);
        }
        HoleKind::Counterbore {
            size,
            diameter,
            depth,
        } => {
            size.apply(&mut args);
            args["type"] = json!("counterbore");
            args["cb_diameter"] = json!(diameter);
            args["cb_depth"] = json!(depth);
        }
        HoleKind::Countersink { size, diameter } => {
            size.apply(&mut args);
            args["type"] = json!("countersink");
            args["cs_diameter"] = json!(diameter);
            args["cs_angle"] = json!(90);
        }
        HoleKind::Tapped { thread } => {
            args["thread"] = json!(thread);
            args["modeled"] = json!(p.modeled);
        }
    }
    args
}

/// The plugin id, which a screw component's metadata names.
pub const PLUGIN: &str = "techsquid-screws";

/// Where a screw is built. One whose thread is cut gets a site of its own, so that no
/// other screw of its size is on its axis: see [`plugin_kit::component`]. One whose
/// thread is only listed is built on the origin.
pub fn built_at(p: &Params) -> Site {
    if p.modeled {
        Site::Compact
    } else {
        Site::Origin
    }
}

/// One screw named `body_name` with the top of its shank on `site`, whose axes are the
/// world's. It is made on the origin and moved to the site before its thread is cut:
/// the kernel refuses some heads away from the origin.
fn build_screw<R: Runner>(
    r: &mut R,
    p: &Params,
    site: &Placement,
    body_name: &str,
) -> Result<(), String> {
    let place = &Placement {
        point: [0.0, 0.0, 0.0],
        ..*site
    };
    let Some(major) = p.resolved.major else {
        return Err(format!("{}: {NO_DIAMETER}", p.size_field));
    };
    let place = *place;
    let length = p.length;
    let axis = place.normal;
    let point_length = match p.resolved.tip {
        ResolvedTip::Point { length } => length,
        _ => 0.0,
    };
    // The point is part of `length`. The cylinder stops at the shoulder.
    let cylinder = length - point_length;
    // Every feature and sketch of this screw starts with its name.
    let part = body_name;
    let named = |feature: &str| name::part(part, feature);
    let shank = r.run(
        "solid.cylinder",
        json!({
            "diameter": major,
            "height": cylinder,
            "base": place.tip(cylinder),
            "axis": axis,
            "operation": "new",
            "name": named("Shank"),
            "body_name": body_name,
        }),
    )?;
    // Join the body the host named, in case it still chose another name.
    // Guessing the requested name could attach the head to another screw.
    let body = body_name_of(&shank);
    // The chamfers go on the plain cylinder, before a cup is cut into its end.
    add_set_screw_chamfers(r, p, &place, body.as_deref(), part, major)?;
    add_tip(r, p, &place, body.as_deref(), part, major)?;
    match p.resolved.head {
        ResolvedHead::None => {}
        ResolvedHead::Round { diameter, height } => {
            let body = shank_body(body.clone())?;
            match head::profile(p.resolved.crown, diameter, height) {
                Some(profile) => revolve_head(r, &place, &body, part, &profile)?,
                None => {
                    r.run(
                        "solid.cylinder",
                        json!({
                            "diameter": diameter,
                            "height": height,
                            "base": place.point,
                            "axis": axis,
                            "operation": "join",
                            "targets": [body],
                            "name": named("Head"),
                        }),
                    )?;
                }
            }
        }
        ResolvedHead::Countersunk { diameter, height } => {
            let body = shank_body(body.clone())?;
            // A 90° cone, then the cylindrical land a flat ISO 2009 / ISO 7046-1 head has
            // above it. An ISO 10642 head is all cone.
            let (cone, land) = fasteners::head::countersink(major, diameter, height);
            r.run(
                "solid.cone",
                json!({
                    "diameter": major,
                    "top_diameter": diameter,
                    "height": cone,
                    "base": place.point,
                    "axis": axis,
                    "operation": "join",
                    "targets": [body.clone()],
                    "name": named("Head"),
                }),
            )?;
            if land > 0.0 {
                r.run(
                    "solid.cylinder",
                    json!({
                        "diameter": diameter,
                        "height": land,
                        "base": place.along(cone),
                        "axis": axis,
                        "operation": "join",
                        "targets": [body],
                        "name": named("Head land"),
                    }),
                )?;
            }
        }
        ResolvedHead::Hex {
            across_flats,
            height,
        } => {
            let body = shank_body(body.clone())?;
            // The nut plugin extrudes the same profile. The sketch id comes back from
            // this call, because another screw may already have used the name.
            hex::extrude(
                r,
                &Hex {
                    origin: place.point,
                    x_dir: place.x_dir,
                    y_dir: place.y_dir,
                    across_flats,
                    height,
                    sketch_name: named("Head profile"),
                    name: named("Head"),
                    operation: "join",
                    body_name: None,
                    target: Some(body.clone()),
                },
            )?;
            chamfer_hex_head(r, &place, &body, part, across_flats, height)?;
        }
    }
    add_torx(r, p, &place, body.as_deref(), part)?;
    add_slot(r, p, &place, body.as_deref(), part)?;
    add_phillips(r, p, &place, body.as_deref(), part)?;
    add_hex(r, p, &place, body.as_deref(), part)?;
    if site.point != place.point {
        r.run(
            "solid.move",
            json!({
                "bodies": [body.as_deref().unwrap_or(body_name)],
                "translate": site.point,
                "name": name::part(part, "Move to site"),
            }),
        )?;
    }
    let place = *site;
    // The pick sits in the first third of the cylindrical face, measured from the tip.
    // A point closes that end, so the third is of the cylinder above the shoulder.
    // With a head and a flat end, that end is the only free one and a sixth of the
    // length is the same place.
    // A chamfer at either end of a set screw is not cylinder, so the sixth is of
    // what lies between them.
    let (tip_chamfer, end_chamfer) = (
        p.point_chamfer().unwrap_or(0.0),
        p.socket_end_chamfer_length().unwrap_or(0.0),
    );
    let along = match p.resolved.tip {
        ResolvedTip::Point { length: point } if point > 0.0 => point + (length - point) / 6.0,
        _ => tip_chamfer + (length - tip_chamfer - end_chamfer) / 6.0,
    };
    let at = place.on_shank(length, along, major / 2.0);
    let mut thread = json!({
        "face": at,
        "from": at,
        "designation": p.designation,
        "modeled": p.modeled,
        "name": named("Thread"),
    });
    if let Some(thread_length) = p.thread_length {
        thread["length"] = json!(thread_length);
    }
    r.run("solid.thread", thread)?;
    Ok(())
}

/// A domed or rounded head: its half profile in the plane through the axis, revolved
/// onto the shank. The sketch's x is the radius and its y runs up the axis.
fn revolve_head<R: Runner>(
    r: &mut R,
    place: &Placement,
    body: &str,
    part: &str,
    profile: &[head::Segment],
) -> Result<(), String> {
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": place.point,
                "x_dir": place.x_dir,
                "y_dir": place.normal,
            },
            "name": name::part(part, "Head profile"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    for segment in profile {
        match segment {
            head::Segment::Line(points) => {
                r.run("sketch.line", json!({"sketch": sketch, "points": points}))?;
            }
            head::Segment::Arc {
                start,
                through,
                end,
            } => {
                r.run(
                    "sketch.arc.three_point",
                    json!({"sketch": sketch, "start": start, "through": through, "end": end}),
                )?;
            }
        }
    }
    r.run("sketch.finish", json!({}))?;
    r.run(
        "solid.revolve",
        json!({
            "sketch": sketch,
            "axis": {"origin": place.point, "dir": place.normal},
            "angle": 360,
            "operation": "join",
            "targets": [body],
            "name": name::part(part, "Head"),
        }),
    )?;
    Ok(())
}

/// The 30° chamfer on the top of a hex head, out to the across-flats circle: the same
/// cut a DIN 934 nut has on each end.
fn chamfer_hex_head<R: Runner>(
    r: &mut R,
    place: &Placement,
    body: &str,
    part: &str,
    across_flats: f64,
    height: f64,
) -> Result<(), String> {
    let end = fasteners::nut::end_chamfer(across_flats, None);
    let name = name::part(part, "Head chamfer");
    revolve_cut(r, place, body, height, &end.triangle, &name)
}

/// Cut a triangle revolved about the screw axis from the end face `along` above the
/// point. The triangle's x is the radius and its y runs from that face into the screw.
/// A revolved cut is the same whichever way the screw points, where a chamfer with
/// two unequal legs depends on which of its faces the host takes first.
fn revolve_cut<R: Runner>(
    r: &mut R,
    place: &Placement,
    body: &str,
    along: f64,
    triangle: &[[f64; 2]; 3],
    name: &str,
) -> Result<(), String> {
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": place.along(along),
                "x_dir": place.x_dir,
                "y_dir": opposite(place.normal),
            },
            "name": format!("{name} profile"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    r.run(
        "sketch.line",
        json!({"sketch": sketch, "closed": true, "points": triangle}),
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

/// The chamfers ISO 4026 and ISO 4029 give a set screw: 45° on the point, down to the
/// flat `dp` or to the mouth of the cup `dz`, and round the socket end, down to the
/// root of the thread, 90° included or 120° on the short lengths the standard shades.
/// The 45° ones are chamfer features on the edge of that end, so they can be edited
/// afterwards. The 120° one has unequal legs and is a revolved cut.
fn add_set_screw_chamfers<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body: Option<&str>,
    part: &str,
    major: f64,
) -> Result<(), String> {
    let chamfer = |r: &mut R, distance: f64, along: f64, feature: &str| -> Result<(), String> {
        let body = shank_body(body.map(str::to_string))?;
        r.run(
            "solid.chamfer",
            json!({
                "edges": [place.on_shank(p.length, along, major / 2.0)],
                "distance": distance,
                "body": body,
                "name": name::part(part, feature),
            }),
        )?;
        Ok(())
    };
    if let Some(distance) = p.point_chamfer() {
        chamfer(r, distance, 0.0, "Point chamfer")?;
    }
    let (Some(radial), Some(length)) = (p.socket_end_chamfer(), p.socket_end_chamfer_length())
    else {
        return Ok(());
    };
    if p.socket_end_angle_deg() == 90.0 {
        return chamfer(r, radial, p.length, "Socket end chamfer");
    }
    // The socket end is the face the screw stands on, at the point itself. The cutter
    // runs a little past the thread so the cut breaks out of the cylinder.
    let body = shank_body(body.map(str::to_string))?;
    let (rim, past) = (major / 2.0, 0.1);
    let triangle = [
        [rim - radial, 0.0],
        [rim + past, 0.0],
        [rim + past, (radial + past) * length / radial],
    ];
    let name = name::part(part, "Socket end chamfer");
    revolve_cut(r, place, &body, 0.0, &triangle, &name)
}

/// The cone for a point or a cup. A flat end adds nothing. A missing body name stops here.
fn add_tip<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body: Option<&str>,
    part: &str,
    major: f64,
) -> Result<(), String> {
    match p.resolved.tip {
        ResolvedTip::Flat => Ok(()),
        ResolvedTip::Point { length: point } if point > 0.0 => {
            let body = shank_body(body.map(str::to_string))?;
            let cylinder = p.length - point;
            // The large end sits `TIP_OVERLAP` inside the cylinder. The apex is the true tip.
            r.run(
                "solid.cone",
                json!({
                    "diameter": major,
                    "top_diameter": 0.0,
                    "height": point + TIP_OVERLAP,
                    "base": place.tip(cylinder - TIP_OVERLAP),
                    "axis": opposite(place.normal),
                    "operation": "join",
                    "targets": [body],
                    "name": name::part(part, "Point"),
                }),
            )?;
            Ok(())
        }
        ResolvedTip::Cup { diameter, depth } if diameter > 0.0 && depth > 0.0 => {
            let body = shank_body(body.map(str::to_string))?;
            let reach = depth + TIP_OVERLAP;
            // The large end sits just outside the face, so it is wider than the mouth.
            // On the face the cut is `diameter` across, and the apex is `depth` inside.
            r.run(
                "solid.cone",
                json!({
                    "diameter": diameter * reach / depth,
                    "top_diameter": 0.0,
                    "height": reach,
                    "base": place.tip(p.length + TIP_OVERLAP),
                    "axis": place.normal,
                    "operation": "cut",
                    "targets": [body],
                    "name": name::part(part, "Cup"),
                }),
            )?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Cut the hexalobular recess from the outer face. The sketch sits just outside that
/// face so the cut breaks the surface, and the depth inside the screw is the penetration.
fn add_torx<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body: Option<&str>,
    part: &str,
) -> Result<(), String> {
    let Some(recess) = p.recess else {
        return Ok(());
    };
    let body = shank_body(body.map(str::to_string))?;
    let contour = torx::contour(recess.number)
        .ok_or_else(|| format!("T{} is not an ISO 10664 socket", recess.number))?;
    let height = head_height(&p.resolved);
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": place.along(height + TIP_OVERLAP),
                "x_dir": place.x_dir,
                "y_dir": place.y_dir,
            },
            "name": name::part(part, "Torx profile"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    for arc in &contour.arcs {
        r.run(
            "sketch.arc.three_point",
            json!({
                "sketch": sketch,
                "start": arc.start,
                "through": arc.through,
                "end": arc.end,
            }),
        )?;
    }
    r.run("sketch.finish", json!({}))?;
    r.run(
        "solid.extrude",
        json!({
            "distance": recess.depth + TIP_OVERLAP,
            "direction": "negative",
            "operation": "cut",
            "sketch": sketch,
            "targets": [body],
            "name": name::part(part, "Torx"),
        }),
    )?;
    Ok(())
}

/// Cut a straight slot across the outer face. The rectangle is longer than the head, so
/// both ends open. Its width is the nominal slot. The sketch sits just outside the face.
fn add_slot<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body: Option<&str>,
    part: &str,
) -> Result<(), String> {
    let Some(slot) = p.slot else {
        return Ok(());
    };
    let body = shank_body(body.map(str::to_string))?;
    let height = head_height(&p.resolved);
    let across = outer_width(&p.resolved)?;
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": place.along(height + TIP_OVERLAP),
                "x_dir": place.x_dir,
                "y_dir": place.y_dir,
            },
            "name": name::part(part, "Slot profile"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    r.run(
        "sketch.rectangle.center",
        json!({
            "sketch": sketch,
            "center": [0.0, 0.0],
            "corner": [across / 2.0 + SLOT_PAST, slot.width / 2.0],
        }),
    )?;
    r.run("sketch.finish", json!({}))?;
    r.run(
        "solid.extrude",
        json!({
            "distance": slot.depth + TIP_OVERLAP,
            "direction": "negative",
            "operation": "cut",
            "sketch": sketch,
            "targets": [body],
            "name": name::part(part, "Slot"),
        }),
    )?;
    Ok(())
}

/// Cut a type H recess from the outer face.
///
/// The tool is two cones: a 28° point, then the 26°30′ wing-tip envelope, which is
/// wider when the penetration is deeper. A straight plus of thickness `b` intersects
/// those cones. The side lean and the blunt point are left out. The intersect is then
/// cut from the screw, and it starts just outside the face so the cut breaks the surface.
fn add_phillips<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body: Option<&str>,
    part: &str,
) -> Result<(), String> {
    let Some(tool) = p.phillips else {
        return Ok(());
    };
    let screw = shank_body(body.map(str::to_string))?;
    let height = head_height(&p.resolved);
    let h1 = tool.point_height();
    let point = r.run(
        "solid.cone",
        json!({
            "diameter": tool.bottom,
            "top_diameter": 0.0,
            "height": h1,
            "base": place.along(height - tool.reach() + h1),
            "axis": opposite(place.normal),
            "operation": "new",
            "name": name::part(part, "Phillips point"),
            "body_name": name::part(part, "Phillips point"),
        }),
    )?;
    let point = feature_body(&point, "solid.cone did not name the Phillips point")?;
    let envelope_height = tool.reach() - h1 + 2.0 * TIP_OVERLAP;
    r.run(
        "solid.cone",
        json!({
            "diameter": 2.0 * tool.radius_at(tool.reach() + TIP_OVERLAP),
            "top_diameter": 2.0 * tool.radius_at(h1 - TIP_OVERLAP),
            "height": envelope_height,
            "base": place.along(height + TIP_OVERLAP),
            "axis": opposite(place.normal),
            "operation": "join",
            "targets": [point.clone()],
            "name": name::part(part, "Phillips envelope"),
        }),
    )?;
    let reach = tool.radius_at(tool.reach() + TIP_OVERLAP) + PHILLIPS_PAST;
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": place.along(height + TIP_OVERLAP),
                "x_dir": place.x_dir,
                "y_dir": place.y_dir,
            },
            "name": name::part(part, "Phillips wings"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    r.run(
        "sketch.line",
        json!({
            "sketch": sketch,
            "points": phillips::plus(tool.width, reach),
            "closed": true,
        }),
    )?;
    r.run("sketch.finish", json!({}))?;
    let wings = r.run(
        "solid.extrude",
        json!({
            "distance": tool.reach() + 2.0 * TIP_OVERLAP,
            "direction": "negative",
            "operation": "new",
            "sketch": sketch,
            "name": name::part(part, "Phillips wings"),
            "body_name": name::part(part, "Phillips wings"),
        }),
    )?;
    let wings = feature_body(&wings, "solid.extrude did not name the Phillips wings")?;
    r.run(
        "solid.combine",
        json!({
            "target": wings.clone(),
            "tools": [point],
            "operation": "intersect",
            "name": name::part(part, "Phillips recess"),
        }),
    )?;
    r.run(
        "solid.combine",
        json!({
            "target": screw,
            "tools": [wings],
            "operation": "cut",
            "name": name::part(part, "Phillips"),
        }),
    )?;
    Ok(())
}

/// Cut the hexagon socket from the outer face. The sketch sits just outside that face so
/// the cut breaks the surface, and the depth inside the screw is the socket depth.
fn add_hex<R: Runner>(
    r: &mut R,
    p: &Params,
    place: &Placement,
    body: Option<&str>,
    part: &str,
) -> Result<(), String> {
    let Some(socket) = p.hex else {
        return Ok(());
    };
    let body = shank_body(body.map(str::to_string))?;
    let height = head_height(&p.resolved);
    let created = r.run(
        "sketch.create",
        json!({
            "plane": {
                "origin": place.along(height + TIP_OVERLAP),
                "x_dir": place.x_dir,
                "y_dir": place.y_dir,
            },
            "name": name::part(part, "Hex socket profile"),
        }),
    )?;
    let sketch = sketch_id(&created)?;
    r.run(
        "sketch.line",
        json!({
            "sketch": sketch,
            "points": socket.corners(),
            "closed": true,
        }),
    )?;
    r.run("sketch.finish", json!({}))?;
    r.run(
        "solid.extrude",
        json!({
            "distance": socket.depth + TIP_OVERLAP,
            "direction": "negative",
            "operation": "cut",
            "sketch": sketch,
            "targets": [body],
            "name": name::part(part, "Hex socket"),
        }),
    )?;
    Ok(())
}

/// How far the outer face is above the point: the head height, or nothing on a set
/// screw or a rod, whose drive is cut into the end of the shank.
fn head_height(resolved: &screw::Resolved) -> f64 {
    match resolved.head {
        ResolvedHead::None => 0.0,
        ResolvedHead::Round { height, .. }
        | ResolvedHead::Hex { height, .. }
        | ResolvedHead::Countersunk { height, .. } => height,
    }
}

fn feature_body(made: &Value, missing: &str) -> Result<String, String> {
    body_name_of(made).ok_or_else(|| missing.to_string())
}

/// The widest reach of the outer face. A round head uses its diameter. A hex head uses
/// the width across the corners, `across flats × 2 / √3`.
fn outer_width(resolved: &screw::Resolved) -> Result<f64, String> {
    match resolved.head {
        ResolvedHead::None => resolved.major.ok_or_else(|| NO_DIAMETER.to_string()),
        ResolvedHead::Round { diameter, .. } | ResolvedHead::Countersunk { diameter, .. } => {
            Ok(diameter)
        }
        ResolvedHead::Hex { across_flats, .. } => Ok(across_flats * 2.0 / 3.0_f64.sqrt()),
    }
}

fn shank_body(body: Option<String>) -> Result<String, String> {
    body.ok_or_else(|| "solid.cylinder did not name the shank body".to_string())
}

/// `skipped` is true when a hole was asked for and nothing was selected to drill.
fn summary(
    p: &Params,
    places: &[Placement],
    drilled: &[String],
    skipped: bool,
    placed: Option<&Placed>,
) -> Value {
    let place = &places[0];
    let hole = p.hole.as_ref().filter(|_| !skipped).map(|hole| {
        let kind = match &hole.kind {
            HoleKind::Clearance { size } => {
                json!({"type": screw::CLEARANCE, "clearance": size.describe()})
            }
            HoleKind::Counterbore {
                size,
                diameter,
                depth,
            } => json!({
                "type": screw::RECESSED,
                "clearance": size.describe(),
                "counterbore_diameter": diameter,
                "counterbore_depth": depth,
            }),
            HoleKind::Countersink { size, diameter } => json!({
                "type": screw::RECESSED,
                "clearance": size.describe(),
                "countersink_diameter": diameter,
            }),
            HoleKind::Tapped { thread } => json!({"type": screw::TAPPED, "thread": thread}),
        };
        let mut hole_json = kind;
        hole_json["depth"] = json!(hole.depth);
        hole_json
    });
    let (diameter, height) = match p.resolved.head {
        ResolvedHead::None => (None, None),
        ResolvedHead::Round { diameter, height }
        | ResolvedHead::Countersunk { diameter, height } => (Some(diameter), Some(height)),
        ResolvedHead::Hex {
            across_flats,
            height,
        } => (Some(across_flats), Some(height)),
    };
    json!({
        "designation": p.designation,
        "length": p.nominal_length,
        "shank_length": p.length,
        "major": p.resolved.major,
        "pitch": p.resolved.pitch,
        "head": p.head.label(),
        "drive": p.drive.label(),
        "torx": p.recess.map(|recess| format!("T{}", recess.number)),
        "torx_depth": p.recess.map(|recess| recess.depth),
        "slot_width": p.slot.map(|slot| slot.width),
        "slot_depth": p.slot.map(|slot| slot.depth),
        "phillips": p.phillips.map(|tool| format!("PH{}", tool.number)),
        "phillips_depth": p.phillips.map(|tool| tool.depth),
        "phillips_opening": p.phillips.map(|tool| tool.opening),
        "phillips_reach": p.phillips.map(|tool| tool.reach()),
        "hex": p.hex.map(|socket| socket.across_flats),
        "hex_depth": p.hex.map(|socket| socket.depth),
        "tip": p.tip.label(),
        "head_diameter": diameter,
        "head_height": height,
        "thread_length": p.thread_length,
        "modeled": p.modeled,
        "standard": p.standard,
        "create": p.create.label(),
        "hole": hole,
        "drilled": drilled,
        "count": places.len(),
        "points": places.iter().map(|place| place.point).collect::<Vec<_>>(),
        "at": place.point,
        "normal": place.normal,
        "placed": place.source.words(),
        "component": placed.map(Placed::json),
        "note": skipped.then_some(NO_HOLE),
    })
}

/// Why a Screw and hole run made no hole.
const NO_HOLE: &str = "no hole was drilled because nothing was selected; select the sketch points or circles where the holes go to drill them";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Rec(Vec<(String, Value)>);
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
                "sketch.create" => json!({"sketch": 7, "name": "Head profile"}),
                "solid.cylinder" | "solid.cone" | "solid.extrude"
                    if args.get("operation").and_then(Value::as_str) == Some("new")
                        && args.get("body_name").is_some() =>
                {
                    json!({"created_bodies": [args["body_name"]]})
                }
                _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
            };
            self.0.push((command.to_string(), args));
            Ok(reply)
        }
    }

    fn commands(v: Value) -> (Value, Vec<(String, Value)>) {
        let p = screw::parse(&v).unwrap();
        let mut rec = Rec(vec![]);
        let summary = make_screw(&mut rec, &p).unwrap();
        (summary, rec.0)
    }

    fn args<'a>(calls: &'a [(String, Value)], name: &str) -> &'a Value {
        &calls.iter().find(|(command, _)| command == name).unwrap().1
    }

    fn named<'a>(calls: &'a [(String, Value)], command: &str, name: &str) -> &'a Value {
        &calls
            .iter()
            .find(|(c, args)| c == command && feature(args) == name)
            .unwrap()
            .1
    }

    /// The feature part of a name, after the screw it belongs to: `Head` in
    /// `Screw M3 x 12 socket: Head`. A hole has no screw in front.
    fn feature(args: &Value) -> &str {
        let name = args["name"].as_str().unwrap_or_default();
        name.split_once(": ").map_or(name, |(_, feature)| feature)
    }

    /// On a site the screw is made on the origin, moved, and threaded where it lands.
    #[test]
    fn on_a_site_the_screw_is_made_on_the_origin_and_threaded_on_the_site() {
        let p = screw::parse(&json!({"size": "M5", "length": 25})).unwrap();
        let mut rec = Rec(vec![]);
        let site = plugin_kit::component::site_at([-105.0, -125.0]);
        build_screw(&mut rec, &p, &site, "Screw M5 x 25 socket").unwrap();
        let names: Vec<&str> = rec.0.iter().map(|(c, _)| c.as_str()).collect();
        let moved = names.iter().position(|c| *c == "solid.move").unwrap();
        // The move is the last thing before the thread.
        assert_eq!(names[moved + 1..], ["solid.thread"]);
        assert_eq!(rec.0[0].1["base"], json!([0.0, 0.0, -25.0]));
        assert_eq!(rec.0[moved].1["translate"], json!([-105.0, -125.0, 0.0]));
        let pick = &rec.0[moved + 1].1["face"];
        assert_eq!(pick[0].as_f64().unwrap(), -105.0 + 2.5);
        assert_eq!(pick[1], -125.0);
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert!(manifest["runtime"]["commands"]
            .as_array()
            .unwrap()
            .contains(&json!("solid.move")));
    }

    #[test]
    fn a_screw_with_a_cut_thread_is_built_on_a_site_of_its_own() {
        let site = |v: Value| built_at(&screw::parse(&v).unwrap());
        assert_eq!(site(json!({"size": "M5", "length": 25})), Site::Compact);
        assert_eq!(
            site(json!({"size": "M5", "length": 25, "thread_modeled": false})),
            Site::Origin
        );
    }

    #[test]
    fn an_m3_socket_screw_is_a_shank_a_head_a_hex_socket_and_a_thread() {
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
                "solid.extrude",
                "solid.thread"
            ]
        );
        // DIN 912: a 2.5 mm key, 1.3 mm deep, cut from just outside the 3 mm head.
        let profile = &calls[3].1;
        assert_eq!(profile["name"], "Screw M3 x 12 socket: Hex socket profile");
        assert_eq!(
            profile["plane"]["origin"],
            json!([0.0, 0.0, 3.0 + TIP_OVERLAP])
        );
        let corners = calls[4].1["points"].as_array().unwrap();
        assert_eq!(corners.len(), 6);
        assert!((corners[0][0].as_f64().unwrap() - 2.5 / 3.0_f64.sqrt()).abs() < 1e-12);
        assert_eq!(calls[4].1["closed"], true);
        let socket = &calls[6].1;
        assert_eq!(socket["name"], "Screw M3 x 12 socket: Hex socket");
        assert_eq!(socket["operation"], "cut");
        assert_eq!(socket["direction"], "negative");
        assert_eq!(socket["distance"], 1.3 + TIP_OVERLAP);
        assert_eq!(socket["targets"], json!(["Screw M3 x 12 socket"]));
        assert_eq!(summary["hex"], 2.5);
        assert_eq!(summary["hex_depth"], 1.3);
        let shank = &calls[1].1;
        assert_eq!(shank["diameter"], 3.0);
        assert_eq!(shank["height"], 12.0);
        assert_eq!(shank["base"], json!([0.0, 0.0, -12.0]));
        assert_eq!(shank["axis"], json!([0.0, 0.0, 1.0]));
        assert_eq!(shank["operation"], "new");
        assert_eq!(shank["body_name"], "Screw M3 x 12 socket");
        let head = &calls[2].1;
        assert_eq!(head["diameter"], 5.5);
        assert_eq!(head["height"], 3.0);
        assert_eq!(head["base"], json!([0.0, 0.0, 0.0]));
        assert_eq!(head["operation"], "join");
        assert_eq!(head["targets"], json!(["Screw M3 x 12 socket"]));
        let thread = &calls[7].1;
        assert_eq!(thread["designation"], "M3");
        assert_eq!(thread["modeled"], true);
        assert_eq!(thread["face"], json!([1.5, 0.0, -10.0]));
        assert_eq!(thread["from"], json!([1.5, 0.0, -10.0]));
        assert!(thread.get("length").is_none());
        assert_eq!(summary["head"], "Socket head");
        assert_eq!(summary["drive"], "Hex socket");
        assert_eq!(summary["tip"], "Flat");
        assert_eq!(summary["head_diameter"], 5.5);
        assert_eq!(summary["pitch"], 0.5);
        assert_eq!(summary["at"], json!([0.0, 0.0, 0.0]));
        assert_eq!(summary["normal"], json!([0.0, 0.0, 1.0]));
    }

    #[test]
    fn a_hex_head_is_a_six_sided_sketch() {
        let (_, calls) = commands(json!({
            "designation": "M8x1",
            "length": 30,
            "head": "Hex head",
            "thread_length": 16,
            "thread_modeled": true,
        }));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
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
                "solid.thread",
            ]
        );
        // The top is chamfered at 30°, out to the across-flats circle, as a nut is.
        let chamfer = named(&calls, "sketch.create", "Head chamfer profile");
        assert_eq!(chamfer["plane"]["origin"], json!([0.0, 0.0, 5.3]));
        assert_eq!(chamfer["plane"]["y_dir"], json!([0.0, 0.0, -1.0]));
        let cut = named(&calls, "solid.revolve", "Head chamfer");
        assert_eq!(cut["operation"], "cut");
        assert_eq!(cut["targets"], json!(["Screw M8x1 x 30 hex"]));
        assert_eq!(
            args(&calls, "sketch.create")["plane"]["origin"],
            json!([0.0, 0.0, 0.0])
        );
        let polygon = args(&calls, "sketch.polygon.circumscribed");
        assert_eq!(polygon["radius"], 6.5);
        assert_eq!(polygon["sides"], 6);
        assert_eq!(args(&calls, "solid.extrude")["distance"], 5.3);
        assert_eq!(args(&calls, "solid.extrude")["sketch"], 7);
        let thread = args(&calls, "solid.thread");
        assert_eq!(thread["designation"], "M8x1");
        assert_eq!(thread["length"], 16.0);
        assert_eq!(thread["modeled"], true);
        assert_eq!(thread["face"], json!([4.0, 0.0, -25.0]));
    }

    #[test]
    fn a_countersunk_head_is_a_truncated_cone() {
        let (_, calls) = commands(json!({"head": "Countersunk"}));
        assert_eq!(calls[2].0, "solid.cone");
        let cone = &calls[2].1;
        assert_eq!(cone["diameter"], 3.0);
        // ISO 10642: 6.72 mm is 3 + 2 × 1.86, the sharp edge of a 90° cone.
        assert_eq!(cone["top_diameter"], 6.72);
        assert_eq!(cone["height"], 1.86);
        assert_eq!(cone["base"], json!([0.0, 0.0, 0.0]));
        assert_eq!(cone["operation"], "join");
    }

    #[test]
    fn an_m3_torx_cuts_a_t10_recess_from_the_top_of_the_head() {
        let (summary, calls) = commands(json!({"drive": "Torx"}));
        let mut names: Vec<&str> = calls.iter().map(|(c, _)| c.as_str()).collect();
        let arcs = names
            .iter()
            .filter(|name| **name == "sketch.arc.three_point")
            .count();
        names.retain(|name| *name != "sketch.arc.three_point");
        assert_eq!(
            names,
            [
                "selection.get",
                "solid.cylinder",
                "solid.cylinder",
                "sketch.create",
                "sketch.finish",
                "solid.extrude",
                "solid.thread",
            ]
        );
        assert_eq!(arcs, 12);
        let sketch = args(&calls, "sketch.create");
        assert_eq!(sketch["name"], "Screw M3 x 12 socket: Torx profile");
        assert_eq!(
            sketch["plane"]["origin"],
            json!([0.0, 0.0, 3.0 + TIP_OVERLAP])
        );
        let arc = args(&calls, "sketch.arc.three_point");
        assert_eq!(arc["sketch"], 7);
        let contour = torx::contour(10).unwrap();
        assert_eq!(arc["through"], json!(contour.arcs[0].through));
        let cut = args(&calls, "solid.extrude");
        assert_eq!(cut["operation"], "cut");
        assert_eq!(cut["direction"], "negative");
        assert_eq!(cut["sketch"], 7);
        assert_eq!(cut["targets"], json!(["Screw M3 x 12 socket"]));
        assert_eq!(cut["name"], "Screw M3 x 12 socket: Torx");
        let depth = summary["torx_depth"].as_f64().unwrap();
        assert_eq!(cut["distance"], json!(depth + TIP_OVERLAP));
        assert_eq!(summary["torx"], "T10");
        assert!((depth - 1.01).abs() < 1e-9);
    }

    #[test]
    fn a_set_screw_torx_is_cut_into_the_end() {
        let (_, calls) =
            commands(json!({"head": "Set screw", "drive": "Torx", "torx": "T6", "length": 10}));
        assert_eq!(
            calls
                .iter()
                .map(|(c, _)| c.as_str())
                .filter(|c| *c != "sketch.arc.three_point")
                .collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "solid.chamfer",
                "sketch.create",
                "sketch.finish",
                "solid.extrude",
                "solid.thread",
            ]
        );
        assert_eq!(
            args(&calls, "sketch.create")["plane"]["origin"],
            json!([0.0, 0.0, TIP_OVERLAP])
        );
        assert_eq!(
            args(&calls, "solid.extrude")["targets"],
            json!(["Set screw M3 x 10"])
        );
    }

    #[test]
    fn an_m3_pan_cuts_the_iso_1580_slot_across_the_head() {
        let (summary, calls) = commands(json!({"head": "Pan head", "drive": "Slotted"}));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "sketch.create",
                "sketch.line",
                "sketch.arc.three_point",
                "sketch.line",
                "sketch.finish",
                "solid.revolve",
                "sketch.create",
                "sketch.rectangle.center",
                "sketch.finish",
                "solid.extrude",
                "solid.thread",
            ]
        );
        // ISO 1580: the head is revolved, with its top edge rounded to rt = 0.9.
        let profile = named(&calls, "sketch.create", "Head profile");
        assert_eq!(profile["plane"]["y_dir"], json!([0.0, 0.0, 1.0]));
        let arc = args(&calls, "sketch.arc.three_point");
        assert_eq!(arc["start"], json!([2.8, 0.9]));
        let head = named(&calls, "solid.revolve", "Head");
        assert_eq!(head["operation"], "join");
        assert_eq!(head["angle"], 360);
        assert_eq!(head["targets"], json!(["Screw M3 x 12 pan"]));
        let sketch = named(&calls, "sketch.create", "Slot profile");
        assert_eq!(sketch["name"], "Screw M3 x 12 pan: Slot profile");
        assert_eq!(
            sketch["plane"]["origin"],
            json!([0.0, 0.0, 1.8 + TIP_OVERLAP])
        );
        let rect = args(&calls, "sketch.rectangle.center");
        assert_eq!(rect["sketch"], 7);
        assert_eq!(rect["center"], json!([0.0, 0.0]));
        assert_eq!(rect["corner"], json!([5.6 / 2.0 + SLOT_PAST, 0.4]));
        let cut = args(&calls, "solid.extrude");
        assert_eq!(cut["operation"], "cut");
        assert_eq!(cut["direction"], "negative");
        assert_eq!(cut["sketch"], 7);
        assert_eq!(cut["targets"], json!(["Screw M3 x 12 pan"]));
        assert_eq!(cut["name"], "Screw M3 x 12 pan: Slot");
        assert_eq!(cut["distance"], json!(0.7 + TIP_OVERLAP));
        assert_eq!(summary["slot_width"], 0.8);
        assert_eq!(summary["slot_depth"], 0.7);
        assert_eq!(summary["torx"], Value::Null);
    }

    #[test]
    fn a_set_screw_slot_is_cut_into_the_end() {
        let (_, calls) = commands(json!({"head": "Set screw", "drive": "Slotted", "length": 10}));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "solid.chamfer",
                "sketch.create",
                "sketch.rectangle.center",
                "sketch.finish",
                "solid.extrude",
                "solid.thread",
            ]
        );
        assert_eq!(
            args(&calls, "sketch.create")["plane"]["origin"],
            json!([0.0, 0.0, TIP_OVERLAP])
        );
        assert_eq!(
            args(&calls, "sketch.rectangle.center")["corner"],
            json!([3.0 / 2.0 + SLOT_PAST, 0.2])
        );
        assert_eq!(
            args(&calls, "solid.extrude")["distance"],
            json!(0.8 + TIP_OVERLAP)
        );
        assert_eq!(
            args(&calls, "solid.extrude")["targets"],
            json!(["Set screw M3 x 10"])
        );
    }

    #[test]
    fn a_hex_slot_opens_past_the_corners() {
        let (_, calls) = commands(json!({"head": "Hex head", "drive": "Slotted"}));
        let across_corners = 5.5 * 2.0 / 3.0_f64.sqrt();
        assert_eq!(
            args(&calls, "sketch.rectangle.center")["corner"],
            json!([across_corners / 2.0 + SLOT_PAST, 0.4])
        );
    }

    #[test]
    fn an_m3_pan_cuts_a_phillips_recess_from_the_outer_face() {
        let (summary, calls) = commands(json!({"head": "Pan head", "drive": "Phillips"}));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "sketch.create",
                "sketch.line",
                "sketch.arc.three_point",
                "sketch.line",
                "sketch.finish",
                "solid.revolve",
                "solid.cone",
                "solid.cone",
                "sketch.create",
                "sketch.line",
                "sketch.finish",
                "solid.extrude",
                "solid.combine",
                "solid.combine",
                "solid.thread",
            ]
        );
        // ISO 7045: the domed head ends on the axis at the full height, and the
        // recess depth is measured from there.
        assert_eq!(
            args(&calls, "sketch.arc.three_point")["end"],
            json!([0.0, 2.4])
        );
        // ISO 7045: an M3 cross-recessed pan head is 2.4 mm tall.
        let height = 2.4;
        let bottom = 1.27;
        let h1 = phillips::point_height(bottom);
        // ISO 7045: the wings are 3 mm across at the face. They narrow at 26°30′ to
        // `g`, and the point ends `h1` below that: 2.07 mm under the face.
        let depth = (3.0 - bottom) / (2.0 * 26.5_f64.to_radians().tan()) + h1;
        assert!((depth - 2.073).abs() < 0.005, "{depth}");
        let point = named(&calls, "solid.cone", "Phillips point");
        assert_eq!(point["diameter"], bottom);
        assert_eq!(point["top_diameter"], 0.0);
        assert_eq!(point["height"], h1);
        assert_eq!(point["base"], json!([0.0, 0.0, height - depth + h1]));
        assert_eq!(point["axis"], json!([0.0, 0.0, -1.0]));
        assert_eq!(point["operation"], "new");
        assert_eq!(point["body_name"], "Screw M3 x 12 pan: Phillips point");
        let envelope = named(&calls, "solid.cone", "Phillips envelope");
        assert_eq!(
            envelope["diameter"],
            2.0 * phillips::radius_at(bottom, depth + TIP_OVERLAP)
        );
        assert_eq!(
            envelope["top_diameter"],
            2.0 * phillips::radius_at(bottom, h1 - TIP_OVERLAP)
        );
        assert_eq!(envelope["height"], depth - h1 + 2.0 * TIP_OVERLAP);
        assert_eq!(envelope["base"], json!([0.0, 0.0, height + TIP_OVERLAP]));
        assert_eq!(envelope["axis"], json!([0.0, 0.0, -1.0]));
        assert_eq!(envelope["operation"], "join");
        assert_eq!(
            envelope["targets"],
            json!(["Screw M3 x 12 pan: Phillips point"])
        );
        let sketch = named(&calls, "sketch.create", "Phillips wings");
        assert_eq!(sketch["name"], "Screw M3 x 12 pan: Phillips wings");
        assert_eq!(
            sketch["plane"]["origin"],
            json!([0.0, 0.0, height + TIP_OVERLAP])
        );
        let reach = phillips::radius_at(bottom, depth + TIP_OVERLAP) + PHILLIPS_PAST;
        let line = &calls
            .iter()
            .find(|(c, args)| c == "sketch.line" && args["closed"] == true)
            .unwrap()
            .1;
        assert_eq!(line["sketch"], 7);
        assert_eq!(line["closed"], true);
        assert_eq!(line["points"], json!(phillips::plus(0.97, reach)));
        let wings = args(&calls, "solid.extrude");
        assert_eq!(wings["operation"], "new");
        assert_eq!(wings["direction"], "negative");
        assert_eq!(wings["distance"], depth + 2.0 * TIP_OVERLAP);
        assert_eq!(wings["sketch"], 7);
        assert_eq!(wings["body_name"], "Screw M3 x 12 pan: Phillips wings");
        let recess = named(&calls, "solid.combine", "Phillips recess");
        assert_eq!(recess["operation"], "intersect");
        assert_eq!(recess["target"], "Screw M3 x 12 pan: Phillips wings");
        assert_eq!(
            recess["tools"],
            json!(["Screw M3 x 12 pan: Phillips point"])
        );
        let cut = named(&calls, "solid.combine", "Phillips");
        assert_eq!(cut["operation"], "cut");
        assert_eq!(cut["target"], "Screw M3 x 12 pan");
        assert_eq!(cut["tools"], json!(["Screw M3 x 12 pan: Phillips wings"]));
        assert_eq!(summary["phillips"], "PH1");
        assert_eq!(summary["phillips_depth"], 1.4);
        assert_eq!(summary["phillips_opening"], 3.0);
        assert_eq!(summary["phillips_reach"], depth);
        assert_eq!(summary["torx"], Value::Null);
        assert_eq!(summary["slot_width"], Value::Null);
    }

    #[test]
    fn a_set_screw_has_no_head() {
        let (summary, calls) =
            commands(json!({"head": "Set screw", "length": 10, "drive": "None"}));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "solid.chamfer",
                "solid.thread"
            ]
        );
        assert_eq!(calls[1].1["height"], 10.0);
        assert_eq!(calls[1].1["base"], json!([0.0, 0.0, -10.0]));
        // ISO 4026 flat point: the tip edge is chamfered 0.5 mm, from 3 mm down to 2.
        let chamfer = &calls[2].1;
        assert_eq!(chamfer["name"], "Set screw M3 x 10: Point chamfer");
        assert_eq!(chamfer["distance"], 0.5);
        assert_eq!(chamfer["edges"], json!([[1.5, 0.0, -10.0]]));
        assert_eq!(chamfer["body"], "Set screw M3 x 10");
        // The thread is picked a sixth of the way up the cylinder above the chamfer.
        let thread = &calls[3].1;
        assert!((thread["face"][2].as_f64().unwrap() - (-10.0 + 0.5 + 9.5 / 6.0)).abs() < 1e-9);
        assert_eq!(summary["head_diameter"], Value::Null);
        assert_eq!(summary["head_height"], Value::Null);
        assert_eq!(summary["tip"], "Flat");
    }

    #[test]
    fn a_point_is_a_cone_inside_the_length() {
        let (summary, calls) = commands(json!({"tip": "Point", "drive": "None"}));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "solid.cone",
                "solid.cylinder",
                "solid.thread"
            ]
        );
        assert_eq!(calls[1].1["height"], 9.0);
        assert_eq!(calls[1].1["base"], json!([0.0, 0.0, -9.0]));
        let cone = &calls[2].1;
        assert_eq!(cone["diameter"], 3.0);
        assert_eq!(cone["top_diameter"], 0.0);
        assert_eq!(cone["height"], 3.0 + TIP_OVERLAP);
        assert_eq!(cone["base"], json!([0.0, 0.0, -(9.0 - TIP_OVERLAP)]));
        assert_eq!(cone["axis"], json!([0.0, 0.0, -1.0]));
        assert_eq!(cone["operation"], "join");
        assert_eq!(cone["targets"], json!(["Screw M3 x 12 socket"]));
        assert_eq!(cone["name"], "Screw M3 x 12 socket: Point");
        assert_eq!(calls[3].1["name"], "Screw M3 x 12 socket: Head");
        assert_eq!(calls[4].1["face"], json!([1.5, 0.0, -7.5]));
        assert!(calls[4].1.get("tip").is_none());
        assert_eq!(summary["tip"], "Point");
    }

    #[test]
    fn a_cup_is_a_cone_cut_into_the_end() {
        let (summary, calls) = commands(json!({
            "head": "Set screw",
            "tip": "Cup",
            "length": 10,
            "drive": "None"
        }));
        assert_eq!(
            calls.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "selection.get",
                "solid.cylinder",
                "solid.chamfer",
                "solid.cone",
                "solid.thread"
            ]
        );
        assert_eq!(calls[1].1["height"], 10.0);
        assert_eq!(calls[1].1["base"], json!([0.0, 0.0, -10.0]));
        // ISO 4029: the outside is chamfered at 45° from 3 mm down to the 1.4 mm mouth,
        // before the cup is cut, so the rim between them is sharp.
        let chamfer = &calls[2].1;
        assert_eq!(chamfer["name"], "Set screw M3 x 10: Point chamfer");
        assert!((chamfer["distance"].as_f64().unwrap() - 0.8).abs() < 1e-12);
        assert_eq!(chamfer["edges"], json!([[1.5, 0.0, -10.0]]));
        let depth = 1.4 / (2.0 * 3.0_f64.sqrt());
        let reach = depth + TIP_OVERLAP;
        let cone = &calls[3].1;
        assert_eq!(cone["diameter"], 1.4 * reach / depth);
        assert_eq!(cone["top_diameter"], 0.0);
        assert_eq!(cone["height"], reach);
        assert_eq!(cone["base"], json!([0.0, 0.0, -(10.0 + TIP_OVERLAP)]));
        assert_eq!(cone["axis"], json!([0.0, 0.0, 1.0]));
        assert_eq!(cone["operation"], "cut");
        assert_eq!(cone["targets"], json!(["Set screw M3 x 10"]));
        assert_eq!(cone["name"], "Set screw M3 x 10: Cup");
        // The thread is picked on the cylinder above the chamfer.
        let along = 0.8 + (10.0 - 0.8) / 6.0;
        assert!((calls[4].1["face"][2].as_f64().unwrap() - (-10.0 + along)).abs() < 1e-9);
        assert_eq!(summary["tip"], "Cup");
        assert_eq!(summary["head_diameter"], Value::Null);
    }

    #[test]
    fn a_named_circle_puts_the_shank_top_on_its_centre() {
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
                    "solid.cylinder"
                        if args.get("operation").and_then(Value::as_str) == Some("new") =>
                    {
                        json!({"created_bodies": ["Screw"]})
                    }
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
                };
                self.0.push((command.to_string(), args));
                Ok(reply)
            }
        }
        let p = screw::parse(&json!({"sketch": "Holes", "sketch_point": "c1", "drive": "None"}))
            .unwrap();
        let mut rec = Named(vec![]);
        let summary = make_screw(&mut rec, &p).unwrap();
        assert_eq!(
            rec.0.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>(),
            [
                "sketch.inspect",
                "solid.cylinder",
                "solid.cylinder",
                "solid.thread"
            ]
        );
        assert_eq!(rec.0[0].1["sketch"], "Holes");
        // The screw is built in its own component, on its origin; its placement puts it
        // on the circle.
        assert_eq!(rec.0[1].1["base"], json!([0.0, 0.0, -12.0]));
        assert_eq!(rec.0[1].1["axis"], json!([0.0, 0.0, 1.0]));
        assert_eq!(rec.0[2].1["base"], json!([0.0, 0.0, 0.0]));
        assert_eq!(rec.0[3].1["face"], json!([1.5, 0.0, -10.0]));
        assert_eq!(summary["at"], json!([12.0, 10.0, 8.0]));
        assert_eq!(summary["normal"], json!([0.0, 0.0, 1.0]));
    }

    #[test]
    fn a_renamed_shank_is_the_body_the_head_joins() {
        struct Renamed(Vec<(String, Value)>);
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
                    "solid.cylinder"
                        if args.get("operation").and_then(Value::as_str) == Some("new") =>
                    {
                        json!({"created_bodies": ["Body1"]})
                    }
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
                };
                self.0.push((command.to_string(), args));
                Ok(reply)
            }
        }
        let p = screw::parse(&json!({"drive": "None"})).unwrap();
        let mut rec = Renamed(vec![]);
        make_screw(&mut rec, &p).unwrap();
        assert_eq!(rec.0[2].1["targets"], json!(["Body1"]));
    }

    #[test]
    fn a_shank_without_a_body_name_builds_no_head() {
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
        let p = screw::parse(&json!({})).unwrap();
        let mut rec = Mute(vec![]);
        let err = make_screw(&mut rec, &p).unwrap_err();
        assert_eq!(err, "solid.cylinder did not name the shank body");
        assert_eq!(rec.0, ["selection.get", "solid.cylinder"]);
    }

    #[test]
    fn a_point_without_a_body_name_builds_no_cone() {
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
        let p = screw::parse(&json!({"tip": "Point", "drive": "None"})).unwrap();
        let mut rec = Mute(vec![]);
        let err = make_screw(&mut rec, &p).unwrap_err();
        assert_eq!(err, "solid.cylinder did not name the shank body");
        assert_eq!(rec.0, ["selection.get", "solid.cylinder"]);
    }

    #[test]
    fn a_renamed_shank_is_the_body_the_point_joins() {
        struct Renamed(Vec<(String, Value)>);
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
                    "solid.cylinder"
                        if args.get("operation").and_then(Value::as_str) == Some("new") =>
                    {
                        json!({"created_bodies": ["Body1"]})
                    }
                    _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
                };
                self.0.push((command.to_string(), args));
                Ok(reply)
            }
        }
        let p = screw::parse(&json!({"tip": "Point", "drive": "None"})).unwrap();
        let mut rec = Renamed(vec![]);
        make_screw(&mut rec, &p).unwrap();
        assert_eq!(rec.0[2].1["targets"], json!(["Body1"]));
        assert_eq!(rec.0[2].1["name"], "Screw M3 x 12 socket: Point");
        assert_eq!(rec.0[3].1["targets"], json!(["Body1"]));
    }

    #[test]
    fn the_manifest_lists_sizes_that_build_and_orders_its_fields() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(manifest["id"], "techsquid-screws");
        let description = manifest["description"].as_str().unwrap();
        assert!(description.chars().count() <= 240, "{description}");
        let entry = &manifest["runtime"]["entries"][0];
        // The menu row, the dialog title and the status line all use this name.
        assert_eq!(entry["slots"], json!([{"kind": "menu", "name": "Screw"}]));
        let props = &entry["input_schema"]["properties"];
        let sizes = props["size"]["enum"].as_array().unwrap();
        assert!(sizes.contains(&props["size"]["default"]));
        for size in sizes {
            let p = screw::parse(&json!({"designation": size})).unwrap();
            assert!(p.resolved.major.is_some(), "{size}");
        }
        // The dialog lists optional fields by key, so a sketch comes before its point.
        assert!(props.get("point").is_none());
        assert!(props.get("sketch_point").is_some());
        assert!(props.get("custom_size").is_some());
        // One box sizes whichever drive is chosen, and it sorts next to Custom size.
        assert!(props.get("drive_size").is_some());
        for old in ["torx", "slot", "phillips", "hex"] {
            assert!(props.get(old).is_none(), "{old}");
        }
        assert_eq!(props["drive"]["default"], "Standard");
        // Every head and drive the dialog offers is one the plugin reads.
        for head in props["head"]["enum"].as_array().unwrap() {
            let p = screw::parse(&json!({"head": head}));
            assert!(p.is_ok(), "{head}: {p:?}");
        }
        for drive in props["drive"]["enum"].as_array().unwrap() {
            let p = screw::parse(&json!({"drive": drive}));
            assert!(p.is_ok(), "{drive}: {p:?}");
        }
    }

    #[test]
    fn a_flat_countersunk_head_is_a_ninety_degree_cone_and_a_land() {
        let (summary, calls) = commands(json!({"head": "Countersunk", "drive": "Slotted"}));
        // ISO 2009 M3: 5.5 across and 1.65 tall. The cone reaches 5.5 at 1.25 mm.
        let cone = named(&calls, "solid.cone", "Head");
        assert_eq!(cone["diameter"], 3.0);
        assert_eq!(cone["top_diameter"], 5.5);
        assert!((cone["height"].as_f64().unwrap() - 1.25).abs() < 1e-9);
        let land = named(&calls, "solid.cylinder", "Head land");
        assert_eq!(land["diameter"], 5.5);
        assert!((land["height"].as_f64().unwrap() - 0.4).abs() < 1e-9);
        assert!((land["base"][2].as_f64().unwrap() - 1.25).abs() < 1e-9);
        assert_eq!(land["operation"], "join");
        // The slot is cut from the top of the land.
        let slot = named(&calls, "sketch.create", "Slot profile");
        assert_eq!(
            slot["plane"]["origin"],
            json!([0.0, 0.0, 1.65 + TIP_OVERLAP])
        );
        assert_eq!(summary["head_diameter"], 5.5);
        assert_eq!(summary["head_height"], 1.65);
        // The hexagon socket head has no land.
        let (_, calls) = commands(json!({"head": "Countersunk"}));
        assert!(calls
            .iter()
            .all(|(_, args)| args["name"] != "Screw M3 x 12 countersunk: Head land"));
    }

    /// Two selected circles on a sketch on the top face of a plate.
    struct OnPlate {
        calls: Vec<(String, Value)>,
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
                "sketch.create" => json!({"sketch": 7}),
                "solid.cylinder" if args["operation"] == "new" => {
                    json!({"created_bodies": [args["body_name"]]})
                }
                "solid.hole" => json!({"changed_bodies": self.changed}),
                _ => plugin_kit::component::stand_in(command).unwrap_or(Value::Null),
            };
            self.calls.push((command.to_string(), args));
            Ok(reply)
        }
    }

    /// Where each placement puts the screw's origin: the last column of its matrix.
    fn placed_at(calls: &[(String, Value)]) -> Vec<[f64; 3]> {
        calls
            .iter()
            .filter(|(c, _)| c == "occurrence.move")
            .map(|(_, args)| {
                let m = &args["to_matrix"];
                [0, 1, 2].map(|i| m[i][3].as_f64().unwrap())
            })
            .collect()
    }

    fn on_plate(v: Value, changed: Value) -> (Result<Value, String>, Vec<(String, Value)>) {
        let p = screw::parse(&v).unwrap();
        let mut rec = OnPlate {
            calls: vec![],
            changed,
        };
        let made = make_screw(&mut rec, &p);
        (made, rec.calls)
    }

    #[test]
    fn two_selected_circles_get_two_screws_and_no_hole_by_default() {
        let (summary, calls) = on_plate(json!({"drive": "None"}), json!(["Plate"]));
        let summary = summary.unwrap();
        assert!(calls.iter().all(|(c, _)| c != "solid.hole"));
        // One screw is built, and placed on both circles.
        let shanks: Vec<&Value> = calls
            .iter()
            .filter(|(c, args)| c == "solid.cylinder" && args["operation"] == "new")
            .map(|(_, args)| &args["base"])
            .collect();
        assert_eq!(shanks, [&json!([0.0, 0.0, -12.0])]);
        assert_eq!(placed_at(&calls), [[10.0, 10.0, 10.0], [30.0, 10.0, 10.0]]);
        assert_eq!(summary["component"]["component"], "Screw M3 x 12 socket");
        assert_eq!(summary["component"]["occurrences"], json!([3, 4]));
        assert_eq!(summary["count"], 2);
        assert_eq!(summary["create"], "Screw only");
        assert_eq!(summary["standard"], "ISO 4762");
        assert_eq!(summary["hole"], Value::Null);
    }

    #[test]
    fn screw_and_hole_drills_one_clearance_feature_before_the_screws() {
        let (summary, calls) = on_plate(
            json!({"drive": "None", "create": "Screw and hole"}),
            json!(["Plate"]),
        );
        let summary = summary.unwrap();
        assert_eq!(calls[2].0, "solid.hole");
        let hole = &calls[2].1;
        assert_eq!(hole["sketch"], "Holes");
        assert_eq!(hole["points"], json!(["c1.center", "c2.center"]));
        assert_eq!(hole["clearance"], "M3");
        assert_eq!(hole["depth"], 12.0);
        assert!(hole.get("targets").is_none() && hole.get("type").is_none());
        assert_eq!(summary["drilled"], json!(["Plate"]));
        assert_eq!(summary["hole"]["clearance"], "M3");
        // The head stays on the surface.
        assert_eq!(placed_at(&calls)[0], [10.0, 10.0, 10.0]);
    }

    #[test]
    fn a_recessed_hole_is_a_counterbore_and_the_screw_sinks_into_it() {
        let (summary, calls) = on_plate(
            json!({"drive": "None", "create": "Screw and hole", "hole_type": screw::RECESSED}),
            json!(["Plate"]),
        );
        let hole = &calls[2].1;
        assert_eq!(hole["type"], "counterbore");
        assert_eq!(hole["cb_diameter"], 6.5);
        assert_eq!(hole["cb_depth"], 3.0);
        assert_eq!(hole["clearance"], "M3");
        assert_eq!(hole["depth"], 15.0);
        // The shank top is 3 mm under the 10 mm surface, so the head ends at 10.
        assert_eq!(placed_at(&calls)[0], [10.0, 10.0, 7.0]);
        assert_eq!(summary.unwrap()["points"][0], json!([10.0, 10.0, 7.0]));
        // A countersunk head gets a 90° countersink of its own diameter.
        let (_, calls) = on_plate(
            json!({"head": "Countersunk", "drive": "None", "create": "Hole only",
                "hole_type": screw::RECESSED}),
            json!(["Plate"]),
        );
        let hole = &calls[2].1;
        assert_eq!(hole["type"], "countersink");
        assert_eq!(hole["cs_diameter"], 6.72);
        assert_eq!(hole["cs_angle"], 90);
        assert_eq!(calls.len(), 3);
    }

    #[test]
    fn a_tapped_hole_takes_the_thread_of_the_screw() {
        let (_, calls) = on_plate(
            json!({"size": "M4", "custom_size": "M8x1", "length": 20, "create": "Hole only",
                "hole_type": screw::TAPPED, "thread_modeled": true}),
            json!(["Plate"]),
        );
        let hole = &calls[2].1;
        assert_eq!(hole["thread"], "M8x1");
        assert_eq!(hole["modeled"], true);
        assert_eq!(hole["depth"], 20.0);
        assert!(hole.get("clearance").is_none());
    }

    #[test]
    fn screw_and_hole_with_nothing_selected_makes_a_loose_screw() {
        let (summary, calls) = commands(json!({"create": "Screw and hole"}));
        // No hole: there is no part to drill.
        assert!(!calls
            .iter()
            .any(|(c, args)| c == "solid.hole" && args.get("sketch").is_some()));
        assert_eq!(summary["hole"], Value::Null);
        assert_eq!(summary["drilled"], json!([]));
        assert_eq!(summary["note"], NO_HOLE);
        assert_eq!(
            summary["placed"],
            "on the origin, because nothing was selected"
        );
        assert_eq!(summary["at"], json!([0.0, 0.0, 0.0]));
        let (summary, _) = commands(json!({}));
        assert_eq!(summary["note"], Value::Null);
    }

    #[test]
    fn a_hole_needs_a_selection_and_a_part() {
        let p = screw::parse(&json!({"create": "Hole only"})).unwrap();
        let mut rec = Rec(vec![]);
        let err = make_screw(&mut rec, &p).unwrap_err();
        assert!(
            err.starts_with("sketch_point: select the sketch points"),
            "{err}"
        );
        assert!(err.contains("Screw only"), "{err}");
        let (made, _) = on_plate(json!({"create": "Hole only"}), json!([]));
        assert!(made
            .unwrap_err()
            .starts_with("sketch_point: there is no part"));
    }

    #[test]
    fn a_set_screw_takes_its_hex_socket_in_the_end() {
        let (summary, calls) = commands(json!({"head": "Set screw", "length": 6}));
        // ISO 4026: a 1.5 mm key in an M3. The end face is the point itself.
        assert_eq!(
            named(&calls, "sketch.create", "Hex socket profile")["plane"]["origin"],
            json!([0.0, 0.0, TIP_OVERLAP])
        );
        // A 6 mm M3 is below the shaded lengths, so its socket is the deeper 2 mm.
        assert_eq!(
            named(&calls, "solid.extrude", "Hex socket")["distance"],
            2.0 + TIP_OVERLAP
        );
        assert_eq!(summary["drive"], "Hex socket");
        assert_eq!(summary["hex"], 1.5);
        assert_eq!(summary["hex_depth"], 2.0);
        // Both ends are chamfered before the socket is cut: the point, then the
        // socket end, which is the face the screw stands on.
        let chamfers: Vec<&Value> = calls
            .iter()
            .filter(|(c, _)| c == "solid.chamfer")
            .map(|(_, args)| args)
            .collect();
        assert_eq!(chamfers.len(), 2);
        assert_eq!(chamfers[0]["edges"], json!([[1.5, 0.0, -6.0]]));
        assert_eq!(chamfers[1]["name"], "Set screw M3 x 6: Socket end chamfer");
        assert_eq!(chamfers[1]["edges"], json!([[1.5, 0.0, 0.0]]));
        assert!((chamfers[1]["distance"].as_f64().unwrap() - 0.3067).abs() < 1e-9);
        let socket = calls
            .iter()
            .position(|(c, a)| c == "solid.extrude" && a["name"] == "Set screw M3 x 6: Hex socket");
        let last_chamfer = calls.iter().rposition(|(c, _)| c == "solid.chamfer");
        assert!(last_chamfer < socket);
    }

    #[test]
    fn a_custom_size_replaces_the_listed_one() {
        let (summary, calls) = commands(json!({
            "designation": "M4",
            "custom_designation": "M8x1",
            "length": 20,
        }));
        assert_eq!(args(&calls, "solid.thread")["designation"], "M8x1");
        assert_eq!(summary["major"], 8.0);
        assert_eq!(
            summary["placed"],
            "on the origin, because nothing was selected"
        );
    }

    #[test]
    fn a_size_without_a_diameter_builds_nothing() {
        let p = screw::parse(&json!({
            "designation": "1/2-10 ACME",
            "head_diameter": 22,
            "head_height": 8,
        }))
        .unwrap();
        let mut rec = Rec(vec![]);
        let err = make_screw(&mut rec, &p).unwrap_err();
        assert_eq!(err, format!("size: {NO_DIAMETER}"));
        assert!(rec.0.is_empty());
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
    fn a_second_screw_asks_for_a_free_body_name() {
        let p = screw::parse(&json!({})).unwrap();
        let mut host = design(&["Base", "Screw M3 x 12 socket"]);
        make_screw(&mut host, &p).unwrap();
        make_screw(&mut host, &p).unwrap();
        assert_eq!(
            host.asked,
            ["Screw M3 x 12 socket (2)", "Screw M3 x 12 socket (3)"]
        );
    }

    #[test]
    fn a_body_name_is_the_kind_the_size_and_the_head() {
        let base = |v: Value| body_base(&screw::parse(&v).unwrap());
        assert_eq!(base(json!({})), "Screw M3 x 12 socket");
        assert_eq!(
            base(json!({"designation": "M5", "length": 20, "head": "Button head"})),
            "Screw M5 x 20 button"
        );
        assert_eq!(
            base(json!({"custom_size": "M8x1", "length": 25, "head": "Hex head"})),
            "Screw M8x1 x 25 hex"
        );
        assert_eq!(
            base(json!({"head": "Countersunk", "length": 10})),
            "Screw M3 x 10 countersunk"
        );
        assert_eq!(
            base(json!({"head": "Set screw", "length": 6})),
            "Set screw M3 x 6"
        );
        assert_eq!(
            base(json!({"designation": "M8", "head": "Threaded rod", "length": 100})),
            "Threaded rod M8 x 100"
        );
    }

    #[test]
    fn every_feature_of_a_screw_starts_with_its_body_name() {
        let (_, calls) = commands(json!({"drive": "Torx"}));
        for (command, args) in &calls {
            if let Some(name) = args.get("name").and_then(Value::as_str) {
                assert!(
                    name.starts_with("Screw M3 x 12 socket: "),
                    "{command} is named {name}"
                );
            }
        }
    }

    #[test]
    fn a_hole_names_its_kind_and_the_screw_size() {
        let named_hole = |v: Value| {
            let p = screw::parse(&v).unwrap();
            hole_name(&p, p.hole.as_ref().unwrap())
        };
        let hole = |kind: &str| json!({"create": "Hole only", "hole_type": kind});
        assert_eq!(named_hole(hole(screw::CLEARANCE)), "Clearance hole M3");
        assert_eq!(named_hole(hole(screw::RECESSED)), "Counterbore M3");
        assert_eq!(named_hole(hole(screw::TAPPED)), "Tapped hole M3");
        let mut sunk = hole(screw::RECESSED);
        sunk["head"] = json!("Countersunk");
        sunk["designation"] = json!("M4");
        assert_eq!(named_hole(sunk), "Countersink M4");
    }
}

plugin_kit::guest! {
    entry: "make-screw",
    noun: "screw",
    manifest: include_str!("../plugin.json"),
    parse: crate::screw::parse,
    make: crate::make_screw,
    describe: |p: &crate::screw::Params| format!("making a {} screw, {} mm", p.designation, p.nominal_length),
    // The dialog sizes every drive in the one Drive size box.
    aliases: &[
        ("hex", "drive_size"),
        ("phillips", "drive_size"),
        ("slot", "drive_size"),
        ("torx", "drive_size"),
    ],
}
