//! Parts as components.
//!
//! Each part a plugin places, such as a fastener or a rail, is a component of its own, built once at its own origin
//! and placed on every point. Four M3 x 12 socket screws are one definition,
//! `Screw M3 x 12 socket`, with four placements: the thread is modelled once, the bill of
//! materials counts four, and the part they go into keeps only its holes.
//!
//! Every placement goes under one folder component in the design, such as `Fasteners`
//! (passed to [`Library::place_all`]). It is a phantom,
//! so the bill of materials counts what is in it and has no row for it.
//!
//! `Library` reuses a definition only after matching its canonical product and
//! representation and verifying its current native contents. A screw with a modeled
//! thread and one with a cosmetic thread are two representations of one purchase.
//!
//! Inside its component the fastener stands as [`Placement::on_origin`] would put it,
//! the outer side up +Z, on its site: the origin, or the point [`Definition::site`]
//! names, which the metadata keeps. A placement turns that frame onto the sketch: X onto
//! the sketch x, Y onto the sketch y, Z onto the normal, the site onto the point.
//!
//! The site is there because of how SolidCraft 0.41 cuts a modeled thread into a hole.
//! It finds the wall by a point, among every body in the design and whatever the hole's
//! targets are, and every component is built where the design's origin is. A bore of the
//! tap drill's size on the same axis there took the thread: an M5 nut built on the origin
//! threaded the 4.2 mm bore of a 2020 rail standing on it, and stayed plain itself. So
//! a part with a tapped hole is built on a site off the origin, off the axes a rail's
//! bores can be on, and no two such definitions share a site. The same goes for a thread
//! cut onto a screw: a second length of one screw size, built on the first one's axis,
//! put its thread on the first. The site comes from the component's id ([`Site::of`]),
//! so it is the definition's alone.
//!
//! The kernel refuses some cuts, such as the chamfer of a hexagon, at positions it
//! accepts at others. So a small part is made on the origin and moved to its site with
//! `solid.move` before its thread is cut. A rail is built on its site as it stands.
//!
//! Every sketch and feature a build makes is folded into one timeline folder named after
//! the part, so a screw is one row of the timeline until it is opened, not a dozen.

use serde_json::{json, Value};

use crate::timeline;
use crate::{Placement, Runner};

mod library;
pub use library::{Budget, Handle, Library};

/// A generated model number for custom geometry, never a supplier SKU. The full
/// product record remains authoritative for reuse; this stable number keeps
/// different custom configurations separate in the host's part-number-based BOM.
/// No display label or raw workflow input may contribute to this number.
pub fn model_number<C: serde::Serialize>(product: &parts::Product<C>) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let identity = product.identity().map_err(|e| e.to_string())?;
    Ok(format!(
        "Custom {} [{:x}]",
        product.part,
        Sha256::digest(identity.as_str().as_bytes())
    ))
}

/// Commands that create and place native definitions.
const PLACEMENT_COMMANDS: [&str; 9] = [
    "component.list",
    "component.create",
    "component.metadata",
    "component.metadata.set",
    "component.activate",
    "occurrence.copy",
    "occurrence.move",
    "occurrence.reparent",
    timeline::COMMAND,
];

/// Commands used by verified definitions, inspection and occurrence placement.
pub fn commands() -> impl Iterator<Item = &'static str> {
    PLACEMENT_COMMANDS
        .into_iter()
        .chain(crate::content::COMMANDS)
        .chain(["document.metadata.set"])
}

/// What a run places.
pub struct Definition<'a> {
    /// The plugin id, `techsquid-screws`.
    pub plugin: &'a str,
    /// The body name before any number, `Screw M3 x 12 socket`. It is the component's
    /// display name. Purchasing numbers are supplied separately.
    pub base: &'a str,
    /// What the bill of materials says it is, such as its standard. Empty for none.
    pub description: &'a str,
    /// The bill of materials type, `purchased`.
    pub bom_type: &'a str,
    /// Where the definition is built: [`Site::Origin`], or a site of its own for a part
    /// with a cut thread.
    pub site: Site,
}

/// What `Library::place_all` did, for the run's reply.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    /// The fastener component's name, and its body's.
    pub component: String,
    /// Whether an existing definition was placed again instead of a new one built.
    pub reused: bool,
    /// The placements made, one for each point, in order.
    pub occurrences: Vec<u64>,
}

/// Adds what every run's reply says about where it placed its part to `reply`, an
/// object: `count`, `points`, `at` and `normal` (of the first placement), `placed` (how
/// it was found) and `component`.
pub fn with_placements(mut reply: Value, places: &[Placement], placed: Option<&Placed>) -> Value {
    if let (Some(fields), Some(first)) = (reply.as_object_mut(), places.first()) {
        let points: Vec<[f64; 3]> = places.iter().map(|place| place.point).collect();
        fields.insert("count".into(), json!(places.len()));
        fields.insert("points".into(), json!(points));
        fields.insert("at".into(), json!(first.point));
        fields.insert("normal".into(), json!(first.normal));
        fields.insert("placed".into(), json!(first.source.words()));
        fields.insert("component".into(), json!(placed.map(Placed::json)));
    }
    reply
}

impl Placed {
    pub fn json(&self) -> Value {
        json!({
            "component": self.component,
            "reused": self.reused,
            "occurrences": self.occurrences,
        })
    }
}

/// Where a definition is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    /// On the origin: a part with no cut thread.
    Origin,
    /// On a site of its own among the small parts, 20 mm from the next: a screw, a nut
    /// or an insert whose thread is cut.
    Compact,
    /// On a site of its own among the rails, 100 mm from the next, because a rail's
    /// bores reach 30 mm from its middle: a rail with a thread cut into an end.
    Apart,
}

/// How many sites a row holds.
const SITE_ROW: u64 = 64;

impl Site {
    /// Where a small part is built: on a site of its own when its thread is cut
    /// (`modeled`), on the origin when the thread is only listed.
    pub fn for_thread(modeled: bool) -> Site {
        if modeled {
            Site::Compact
        } else {
            Site::Origin
        }
    }

    /// The site of the component `component`, as (x, y). Every component of a design
    /// has its own id, so no two definitions share a site. A rail's bores are on
    /// multiples of 10 mm from its middle, and no site is on such a point of a rail
    /// that stands on the origin, or of one built on another site.
    pub fn of(self, component: u64) -> [f64; 2] {
        let (column, row) = ((component % SITE_ROW) as f64, (component / SITE_ROW) as f64);
        match self {
            Site::Origin => [0.0, 0.0],
            // Towards -X and -Y, clear of the rails.
            Site::Compact => [-105.0 - 20.0 * column, -105.0 - 20.0 * row],
            Site::Apart => [1005.0 + 100.0 * column, 1005.0 + 100.0 * row],
        }
    }
}

/// The frame a definition is built on: `at` in the XY plane, the axes the world's.
pub fn site_at(at: [f64; 2]) -> Placement {
    Placement {
        point: [at[0], at[1], 0.0],
        ..Placement::on_origin()
    }
}

/// Inspection fixture for geometry-command unit recorders. This deliberately
/// describes a fixed cylinder, not the recorder's generated CAD. It lets those
/// unit tests inspect the builder commands independently of the native document.
/// Native integration tests are the authority for actual contents and reuse.
pub fn stand_in(command: &str) -> Option<Value> {
    Some(match command {
        "component.list" => json!({"active":0,"pending_moves":[],"components":[{
            "id":2,"name":"Geometry test fixture","bodies":["Fixture body"],"features":["Fixture cylinder"],
            "sketches":[],"construction":[],"children":[],"occurrences":[]
        }]}),
        "document.inspect" => {
            json!({"metadata":{},"timeline":[{"id":9000001,"name":"Fixture cylinder"}],"params":[]})
        }
        "model.body_features" => {
            json!({"bodies":[{"body":"Fixture body","features":[{"id":9000001}]}]})
        }
        "material.list" => json!({"materials":[]}),
        "appearance.list" => json!({"components":[],"body":{}}),
        "joint.list" => json!({"joints":[]}),
        "parameters.list" => json!({"parameters":[]}),
        "component.properties" => {
            json!({"bodies":[{"name":"Fixture body","material":"Default","density_g_cm3":null}]})
        }
        "timeline.inspect" => {
            json!({"id":9000001,"type":"cylinder","definition":{"component":2,"radius":1,"height":1},
            "inputs":[],"omitted":[],"suppressed":false,"rolled_back":false,"error":null})
        }
        "component.create" => json!({"component": 2, "occurrence": 3}),
        "component.metadata" => json!({"metadata": {}}),
        "occurrence.copy" => json!({"occurrence": 4}),
        "timeline.group" => json!({"group": 5}),
        "component.metadata.set"
        | "document.metadata.set"
        | "occurrence.move"
        | "occurrence.reparent"
        | "component.activate" => json!({}),
        _ => return None,
    })
}

/// The rigid move that turns the fastener's own frame onto `place`, as the rows of a
/// 3 x 4 matrix: each row is one world axis of the sketch x, the sketch y, the normal
/// and the point.
pub fn matrix(place: &Placement) -> [[f64; 4]; 3] {
    let row = |i: usize| {
        [
            place.x_dir[i],
            place.y_dir[i],
            place.normal[i],
            place.point[i],
        ]
    };
    [row(0), row(1), row(2)]
}

/// The frame that puts a part built on `site` onto `place`: the placement, moved back
/// by the site along its own axes.
pub fn from_site(place: &Placement, site: [f64; 3]) -> Placement {
    Placement {
        point: std::array::from_fn(|i| {
            place.point[i]
                - site[0] * place.x_dir[i]
                - site[1] * place.y_dir[i]
                - site[2] * place.normal[i]
        }),
        ..*place
    }
}

/// A placement made under the design, where its parent's frame is the world, then moved
/// into the folder, which keeps it where it is in the world wherever the folder is.
fn put<R: Runner>(
    r: &mut R,
    occurrence: u64,
    place: &Placement,
    site: [f64; 3],
    folder: u64,
) -> Result<(), String> {
    r.run(
        "occurrence.move",
        json!({"occurrence": occurrence, "to_matrix": matrix(&from_site(place, site))}),
    )?;
    r.run(
        "occurrence.reparent",
        json!({"occurrence": occurrence, "parent": folder}),
    )?;
    Ok(())
}

/// A component as `component.list` names it.
struct Listed {
    id: u64,
    name: String,
    /// The parents of its placements.
    parents: Vec<u64>,
}

fn components(list: &Value) -> Vec<Listed> {
    list.get("components")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|c| {
            Some(Listed {
                id: c.get("id")?.as_u64()?,
                name: c.get("name")?.as_str()?.to_string(),
                parents: c
                    .get("occurrences")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|o| o.get("parent").and_then(Value::as_u64))
                    .collect(),
            })
        })
        .collect()
}

fn make_folder<R: Runner>(r: &mut R, name: &str) -> Result<u64, String> {
    let made = r.run(
        "component.create",
        json!({"name": name, "parent": 0, "activate": false, "grounded": false}),
    )?;
    let folder = id_of(&made, "component")?;
    r.run(
        "component.metadata.set",
        json!({"component": folder, "bom_type": "phantom"}),
    )?;
    Ok(folder)
}

fn id_of(reply: &Value, key: &str) -> Result<u64, String> {
    reply
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("the host did not say which {key} it made"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_site_is_the_components_own_and_off_every_rail_bore() {
        assert_eq!(Site::Origin.of(7), [0.0, 0.0]);
        assert_eq!(site_at(Site::Origin.of(7)), Placement::on_origin());
        let mut seen = Vec::new();
        for component in [1, 2, 63, 64, 65, 1000, 5000] {
            for site in [Site::Compact, Site::Apart] {
                let at = site.of(component);
                // A rail's bores are on multiples of 10 from its middle.
                assert_eq!(at[0].rem_euclid(10.0), 5.0, "{component}");
                assert_eq!(at[1].rem_euclid(10.0), 5.0, "{component}");
                assert!(!seen.contains(&at), "{component}");
                seen.push(at);
                assert_eq!(site_at(at).point, [at[0], at[1], 0.0]);
                assert_eq!(site_at(at).normal, [0.0, 0.0, 1.0]);
            }
        }
        // Two rails are far enough apart that no bore of one is a bore of the other.
        let (a, b) = (Site::Apart.of(1), Site::Apart.of(2));
        assert!((a[0] - b[0]).abs() > 60.0);
        assert!((Site::Apart.of(1)[1] - Site::Apart.of(65)[1]).abs() > 60.0);
    }

    #[test]
    fn local_site_is_removed_before_world_placement() {
        let place = Placement {
            point: [10.0, 20.0, 30.0],
            x_dir: [0.0, 1.0, 0.0],
            y_dir: [0.0, 0.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            ..Placement::on_origin()
        };
        assert_eq!(
            matrix(&from_site(&place, [5.0, 7.0, 11.0])),
            [
                [0.0, 0.0, 1.0, -1.0],
                [1.0, 0.0, 0.0, 15.0],
                [0.0, 1.0, 0.0, 23.0]
            ]
        );
    }

    #[test]
    fn every_package_authorizes_inspection_and_placement() {
        let plugins = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
        let mut checked = 0;
        for entry in std::fs::read_dir(plugins).unwrap() {
            let folder = entry.unwrap().path();
            if !folder.is_dir() {
                continue;
            }
            let path = folder.join("plugin.json");
            let text = std::fs::read_to_string(&path).unwrap();
            let manifest: Value = serde_json::from_str(&text).unwrap();
            let listed = manifest["runtime"]["commands"].as_array().unwrap();
            for command in commands() {
                assert!(
                    listed.contains(&json!(command)),
                    "{}: {command}",
                    path.display()
                );
            }
            checked += 1;
        }
        assert!(checked >= 6);
    }
}
