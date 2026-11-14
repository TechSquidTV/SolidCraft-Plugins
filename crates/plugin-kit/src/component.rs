//! Parts as components.
//!
//! Each part a plugin places, such as a fastener or a rail, is a component of its own, built once at its own origin
//! and placed on every point. Four M3 x 12 socket screws are one definition,
//! `Screw M3 x 12 socket`, with four placements: the thread is modelled once, the bill of
//! materials counts four, and the part they go into keeps only its holes.
//!
//! Every placement goes under one folder component in the design, such as `Fasteners`
//! ([`Definition::folder`]). It is a phantom,
//! so the bill of materials counts what is in it and has no row for it.
//!
//! A later run reuses a definition only when it would build exactly the same body. A
//! dry run of the build records the commands it would send, and their hash is kept in the
//! component's metadata under [`METADATA_KEY`]. A screw with a modelled thread and one with
//! a cosmetic thread are two definitions with one part number.
//!
//! Inside its component the fastener stands as [`Placement::on_origin`] puts it: the
//! point on the origin, the outer side up +Z. A placement turns that frame onto the
//! sketch: X onto the sketch x, Y onto the sketch y, Z onto the normal, the origin onto the point.

use serde_json::{json, Value};

use crate::host::free_body_name;
use crate::{Placement, Runner};

/// The component metadata key that says which plugin built a definition and how. The
/// name is from when this was the Threadcraft repository; designs made then still hold it.
pub const METADATA_KEY: &str = "dev.threadcraft";

/// The body name a dry run builds with. Every name in the recorded commands starts with
/// it, so the hash does not change with the number a second definition gets.
const DRY_NAME: &str = "fastener";

/// Every command [`place_all`] sends. A plugin's manifest lists them.
pub const COMMANDS: [&str; 8] = [
    "component.list",
    "component.create",
    "component.metadata",
    "component.metadata.set",
    "component.activate",
    "occurrence.copy",
    "occurrence.move",
    "occurrence.reparent",
];

/// What a run places.
pub struct Definition<'a> {
    /// The plugin id, `techsquid-screws`.
    pub plugin: &'a str,
    /// The body name before any number, `Screw M3 x 12 socket`. It is the component's
    /// name and part number too.
    pub base: &'a str,
    /// What the bill of materials says it is, such as its standard. Empty for none.
    pub description: &'a str,
    /// The component every placement goes in, `Fasteners`.
    pub folder: &'a str,
    /// The bill of materials type, `purchased`.
    pub bom_type: &'a str,
}

/// What [`place_all`] did, for the run's reply.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    /// The fastener component's name, and its body's.
    pub component: String,
    /// Whether an existing definition was placed again instead of a new one built.
    pub reused: bool,
    /// The placements made, one for each point, in order.
    pub occurrences: Vec<u64>,
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

/// Place the fastener `build` makes on every placement in `places`. `build` makes one
/// fastener at [`Placement::on_origin`] with the body name it is given, in the active
/// component. The component that was active before is active again afterwards.
pub fn place_all<R, F>(
    r: &mut R,
    def: &Definition,
    places: &[Placement],
    mut build: F,
) -> Result<Placed, String>
where
    R: Runner,
    F: FnMut(&mut dyn Runner, &str) -> Result<(), String>,
{
    let geometry = geometry_key(&mut build)?;
    let list = r.run("component.list", json!({"compact": true}))?;
    let active = list.get("active").cloned().unwrap_or(json!(0));
    let components = components(&list);
    let folder = match folder_id(&components, def.folder) {
        Some(id) => id,
        None => make_folder(r, def.folder)?,
    };
    let found = find_definition(r, def, &components, &geometry)?;
    let reused = found.is_some();
    let mut occurrences = Vec::new();
    let mut remaining = places;
    let (component, name) = match found {
        Some(found) => found,
        None => {
            let mut taken = r.body_names();
            taken.extend(components.iter().map(|c| c.name.clone()));
            let name = free_body_name(&mut taken, def.base);
            let made = r.run(
                "component.create",
                json!({"name": name, "parent": 0, "activate": true, "grounded": false}),
            )?;
            let component = id_of(&made, "component")?;
            let first = id_of(&made, "occurrence")?;
            build(r, &name)?;
            let mut meta = json!({
                "component": component,
                "part_number": def.base,
                "bom_type": def.bom_type,
                "values": {METADATA_KEY: {"plugin": def.plugin, "geometry": geometry}},
            });
            if !def.description.is_empty() {
                meta["description"] = json!(def.description);
            }
            r.run("component.metadata.set", meta)?;
            if let Some((place, rest)) = places.split_first() {
                put(r, first, place, folder)?;
                occurrences.push(first);
                remaining = rest;
            }
            (component, name)
        }
    };
    for place in remaining {
        let copy = r.run(
            "occurrence.copy",
            json!({"component": component, "parent": 0}),
        )?;
        let occurrence = id_of(&copy, "occurrence")?;
        put(r, occurrence, place, folder)?;
        occurrences.push(occurrence);
    }
    r.run("component.activate", json!({"component": active}))?;
    Ok(Placed {
        component: name,
        reused,
        occurrences,
    })
}

/// What a host with an empty design answers to the commands [`place_all`] sends, for a
/// test host that records a build: the folder is component 1 and the fastener component 2,
/// placed by occurrence 3, and a copy is occurrence 4. `None` for any other command.
pub fn stand_in(command: &str) -> Option<Value> {
    Some(match command {
        "component.list" => json!({"active": 0, "components": []}),
        "component.create" => json!({"component": 2, "occurrence": 3}),
        "component.metadata" => json!({"metadata": {}}),
        "occurrence.copy" => json!({"occurrence": 4}),
        "component.metadata.set"
        | "occurrence.move"
        | "occurrence.reparent"
        | "component.activate" => json!({}),
        _ => return None,
    })
}

/// The commands [`place_all`] sends around a new build, before and after it, in an
/// empty design, for one placement and then `copies` more.
pub fn new_definition_commands(copies: usize) -> (Vec<&'static str>, Vec<&'static str>) {
    let before = vec![
        "component.list",
        "component.create",
        "component.metadata.set",
        "component.create",
    ];
    let mut after = vec![
        "component.metadata.set",
        "occurrence.move",
        "occurrence.reparent",
    ];
    for _ in 0..copies {
        after.extend(["occurrence.copy", "occurrence.move", "occurrence.reparent"]);
    }
    after.push("component.activate");
    (before, after)
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

/// A placement made under the design, where its parent's frame is the world, then moved
/// into the folder, which keeps it where it is in the world wherever the folder is.
fn put<R: Runner>(
    r: &mut R,
    occurrence: u64,
    place: &Placement,
    folder: u64,
) -> Result<(), String> {
    r.run(
        "occurrence.move",
        json!({"occurrence": occurrence, "to_matrix": matrix(place)}),
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

/// The folder component placed in the design itself, if there is one.
fn folder_id(components: &[Listed], folder: &str) -> Option<u64> {
    components
        .iter()
        .find(|c| c.id != 0 && c.name == folder && c.parents.contains(&0))
        .map(|c| c.id)
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

/// A definition this plugin built with the same commands, named `base` or `base (n)`.
fn find_definition<R: Runner>(
    r: &mut R,
    def: &Definition,
    components: &[Listed],
    geometry: &str,
) -> Result<Option<(u64, String)>, String> {
    for c in components {
        if c.id == 0 || !numbered(&c.name, def.base) {
            continue;
        }
        let meta = r.run("component.metadata", json!({"component": c.id}))?;
        let ours = &meta["metadata"][METADATA_KEY];
        if ours["plugin"] == json!(def.plugin) && ours["geometry"] == json!(geometry) {
            return Ok(Some((c.id, c.name.clone())));
        }
    }
    Ok(None)
}

/// `name` is `base`, or `base (n)` as [`free_body_name`] numbers it.
fn numbered(name: &str, base: &str) -> bool {
    match name.strip_prefix(base) {
        Some("") => true,
        Some(rest) => rest
            .strip_prefix(" (")
            .and_then(|n| n.strip_suffix(')'))
            .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())),
        None => false,
    }
}

fn id_of(reply: &Value, key: &str) -> Result<u64, String> {
    reply
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("the host did not say which {key} it made"))
}

/// The hash of every command `build` sends for one fastener on the origin, named
/// [`DRY_NAME`]. Two runs with the same hash build the same body.
fn geometry_key<F>(build: &mut F) -> Result<String, String>
where
    F: FnMut(&mut dyn Runner, &str) -> Result<(), String>,
{
    let mut dry = Dry::default();
    build(&mut dry, DRY_NAME)?;
    Ok(format!("{:016x}", fnv1a(dry.text.as_bytes())))
}

/// A host that answers every command as a build expects and keeps what it was sent.
#[derive(Default)]
struct Dry {
    text: String,
    sketches: u64,
}

impl Runner for Dry {
    fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
        self.text.push_str(command);
        self.text.push(' ');
        self.text.push_str(&args.to_string());
        self.text.push('\n');
        self.sketches += 1;
        let body = args
            .get("body_name")
            .and_then(Value::as_str)
            .unwrap_or(DRY_NAME);
        Ok(json!({
            "sketch": self.sketches,
            "feature": self.sketches,
            "created_bodies": [body],
            "changed_bodies": [body],
            "created": {"curves": ["l1"], "points": ["p1"]},
        }))
    }
}

/// FNV-1a, 64 bits: small, and the same on every host.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::place::Source;

    /// A design with the components `list` names, recording every command.
    struct Host {
        calls: Vec<(String, Value)>,
        list: Value,
        metadata: Value,
        next: u64,
    }

    impl Host {
        fn new(list: Value) -> Self {
            Host {
                calls: vec![],
                list,
                metadata: json!({}),
                next: 100,
            }
        }

        fn commands(&self) -> Vec<&str> {
            self.calls.iter().map(|(c, _)| c.as_str()).collect()
        }
    }

    impl Runner for Host {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            self.calls.push((command.to_string(), args.clone()));
            self.next += 1;
            Ok(match command {
                "component.list" => self.list.clone(),
                "component.metadata" => json!({"metadata": self.metadata}),
                "component.create" => json!({"component": self.next, "occurrence": self.next + 50}),
                "occurrence.copy" => json!({"occurrence": self.next + 50}),
                _ => json!({}),
            })
        }

        fn body_names(&mut self) -> Vec<String> {
            vec!["Base".into()]
        }
    }

    fn cylinder(r: &mut dyn Runner, name: &str) -> Result<(), String> {
        r.run(
            "solid.cylinder",
            json!({"height": 3, "body_name": name, "name": format!("{name}: Shank")}),
        )?;
        Ok(())
    }

    fn at(point: [f64; 3]) -> Placement {
        Placement {
            point,
            normal: [0.0, -1.0, 0.0],
            x_dir: [1.0, 0.0, 0.0],
            y_dir: [0.0, 0.0, 1.0],
            source: Source::Named,
        }
    }

    const SCREW: Definition = Definition {
        plugin: "techsquid-screws",
        base: "Screw M3 x 12 socket",
        description: "ISO 4762",
        folder: "Fasteners",
        bom_type: "purchased",
    };

    #[test]
    fn a_new_fastener_is_built_once_and_placed_on_every_point() {
        let mut host = Host::new(json!({"active": 7, "components": [
            {"id": 0, "name": "Design", "occurrences": []},
            {"id": 7, "name": "Base", "occurrences": [{"id": 8, "parent": 0}]},
        ]}));
        let placed = place_all(
            &mut host,
            &SCREW,
            &[at([1.0, 0.0, 2.0]), at([5.0, 0.0, 2.0])],
            cylinder,
        )
        .unwrap();
        assert_eq!(
            host.commands(),
            [
                "component.list",
                "component.create",
                "component.metadata.set",
                "component.create",
                "solid.cylinder",
                "component.metadata.set",
                "occurrence.move",
                "occurrence.reparent",
                "occurrence.copy",
                "occurrence.move",
                "occurrence.reparent",
                "component.activate",
            ]
        );
        let calls = &host.calls;
        // The folder is a phantom in the design itself.
        assert_eq!(
            calls[1].1,
            json!({"name": "Fasteners", "parent": 0, "activate": false, "grounded": false})
        );
        assert_eq!(calls[2].1, json!({"component": 102, "bom_type": "phantom"}));
        // The fastener is built in its own component, free to move.
        assert_eq!(calls[3].1["name"], "Screw M3 x 12 socket");
        assert_eq!(calls[3].1["grounded"], false);
        assert_eq!(calls[4].1["body_name"], "Screw M3 x 12 socket");
        let meta = &calls[5].1;
        assert_eq!(meta["part_number"], "Screw M3 x 12 socket");
        assert_eq!(meta["bom_type"], "purchased");
        assert_eq!(meta["description"], "ISO 4762");
        assert_eq!(meta["values"][METADATA_KEY]["plugin"], "techsquid-screws");
        // Its first placement is the one the component came with.
        assert_eq!(calls[6].1["occurrence"], 154);
        assert_eq!(
            calls[6].1["to_matrix"],
            json!([
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, -1.0, 0.0],
                [0.0, 1.0, 0.0, 2.0]
            ])
        );
        assert_eq!(calls[7].1, json!({"occurrence": 154, "parent": 102}));
        assert_eq!(calls[8].1, json!({"component": 104, "parent": 0}));
        // The part that was active is active again.
        assert_eq!(calls[11].1, json!({"component": 7}));
        assert_eq!(
            placed,
            Placed {
                component: "Screw M3 x 12 socket".into(),
                reused: false,
                occurrences: vec![154, 159],
            }
        );
    }

    #[test]
    fn a_matching_definition_is_placed_again_and_not_rebuilt() {
        let mut probe = Host::new(json!({"active": 0, "components": []}));
        place_all(&mut probe, &SCREW, &[], cylinder).unwrap();
        let stored = probe.calls[5].1["values"].clone();
        let mut host = Host::new(json!({"active": 0, "components": [
            {"id": 0, "name": "Design", "occurrences": []},
            {"id": 3, "name": "Fasteners", "occurrences": [{"id": 4, "parent": 0}]},
            {"id": 5, "name": "Screw M3 x 12 socket", "occurrences": [{"id": 6, "parent": 3}]},
        ]}));
        host.metadata = stored;
        let placed = place_all(&mut host, &SCREW, &[at([0.0; 3])], cylinder).unwrap();
        assert_eq!(
            host.commands(),
            [
                "component.list",
                "component.metadata",
                "occurrence.copy",
                "occurrence.move",
                "occurrence.reparent",
                "component.activate",
            ]
        );
        assert_eq!(host.calls[2].1, json!({"component": 5, "parent": 0}));
        assert_eq!(host.calls[4].1["parent"], 3);
        assert!(placed.reused);
        assert_eq!(placed.component, "Screw M3 x 12 socket");
    }

    #[test]
    fn a_different_build_gets_a_definition_of_its_own() {
        let mut host = Host::new(json!({"active": 0, "components": [
            {"id": 0, "name": "Design", "occurrences": []},
            {"id": 3, "name": "Fasteners", "occurrences": [{"id": 4, "parent": 0}]},
            {"id": 5, "name": "Screw M3 x 12 socket", "occurrences": [{"id": 6, "parent": 3}]},
        ]}));
        host.metadata = json!({METADATA_KEY: {"plugin": "techsquid-screws", "geometry": "other"}});
        let placed = place_all(&mut host, &SCREW, &[at([0.0; 3])], cylinder).unwrap();
        assert!(!placed.reused);
        assert_eq!(placed.component, "Screw M3 x 12 socket (2)");
        assert_eq!(host.calls[2].0, "component.create");
    }

    #[test]
    fn the_key_follows_the_build_and_not_the_name() {
        let mut a = |r: &mut dyn Runner, name: &str| cylinder(r, name);
        let mut b = |r: &mut dyn Runner, name: &str| {
            r.run("solid.cylinder", json!({"height": 4, "body_name": name}))?;
            Ok(())
        };
        assert_eq!(geometry_key(&mut a).unwrap(), geometry_key(&mut a).unwrap());
        assert_ne!(geometry_key(&mut a).unwrap(), geometry_key(&mut b).unwrap());
    }

    #[test]
    fn every_manifest_lists_every_command_sent() {
        for manifest in [
            include_str!("../../../plugins/techsquid-screws/plugin.json"),
            include_str!("../../../plugins/techsquid-inserts/plugin.json"),
            include_str!("../../../plugins/techsquid-nuts/plugin.json"),
        ] {
            let manifest: Value = serde_json::from_str(manifest).unwrap();
            let listed = manifest["runtime"]["commands"].as_array().unwrap();
            for command in COMMANDS {
                assert!(listed.contains(&json!(command)), "{command}");
            }
        }
        let mut host = Host::new(json!({"active": 0, "components": [
            {"id": 3, "name": "Screw M3 x 12 socket", "occurrences": [{"id": 6, "parent": 0}]},
        ]}));
        place_all(&mut host, &SCREW, &[at([0.0; 3]), at([1.0; 3])], cylinder).unwrap();
        for (command, _) in &host.calls {
            assert!(
                command.starts_with("solid.") || COMMANDS.contains(&command.as_str()),
                "{command}"
            );
        }
    }

    #[test]
    fn a_numbered_name_belongs_to_its_base() {
        assert!(numbered("Nut M3", "Nut M3"));
        assert!(numbered("Nut M3 (2)", "Nut M3"));
        assert!(!numbered("Nut M3x0.5", "Nut M3"));
        assert!(!numbered("Nut M3 (x)", "Nut M3"));
        assert!(!numbered("Nut M30", "Nut M3"));
    }
}
