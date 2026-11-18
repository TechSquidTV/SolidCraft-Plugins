//! What every bearing in this crate is built from: two steel rings and, between them,
//! a flat closure face on each side. Radial bearings and CSK clutches differ only in
//! the ring sections they pass in and in what they add afterwards (balls, keyways).

use plugin_kit::{
    name::part,
    revolve::{self, Section},
    Placement, Runner,
};
use serde_json::json;

/// Display colours; illustrative, not a material assignment.
pub const STEEL: [u8; 3] = [160, 160, 160];
pub const RUBBER: [u8; 3] = [64, 64, 64];

/// Mounting envelope in millimetres: what a shaft and a housing are designed against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Envelope {
    pub bore: f64,
    pub diameter: f64,
    pub width: f64,
}

impl std::fmt::Display for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} × {} × {}", self.bore, self.diameter, self.width)
    }
}

/// Revolve the inner ring, then the outer ring, about the frame's normal.
/// Returns the two body names, for the caller to cut and colour.
pub fn rings(
    r: &mut impl Runner,
    frame: &Placement,
    base: &str,
    sections: [Section<'_>; 2],
) -> Result<Vec<String>, String> {
    let mut bodies = Vec::new();
    for (label, section) in ["Inner ring", "Outer ring"].into_iter().zip(sections) {
        let name = format!("{base} {label}");
        let feature = part(base, label);
        revolve::build(
            r,
            frame,
            &part(&feature, "Section"),
            section,
            json!({"name": feature, "body_name": name}),
        )?;
        bodies.push(name);
    }
    Ok(bodies)
}

/// One closure face: a flat annulus between the ring shoulders at height `z`.
/// `label` is the whole face name, such as `Seal front`. Returns the body name.
pub fn face(
    r: &mut impl Runner,
    frame: &Placement,
    base: &str,
    label: &str,
    z: f64,
    [inner, outer]: [f64; 2],
) -> Result<String, String> {
    let name = format!("{base} {label}");
    let feature = part(base, label);
    revolve::build(
        r,
        frame,
        &part(&feature, "Section"),
        Section::Surface(&[[inner, z], [outer, z]]),
        json!({"name": feature, "body_name": name}),
    )?;
    Ok(name)
}

pub fn colour(r: &mut impl Runner, bodies: &[String], colour: [u8; 3]) -> Result<(), String> {
    r.run(
        "appearance.assign",
        json!({"bodies": bodies, "color": colour}),
    )?;
    Ok(())
}
