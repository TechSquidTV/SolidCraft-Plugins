//! Current native contents of a reusable definition, independently of product ID.
//!
//! Build an index once per invocation. It owns a consistent read of the design and
//! caches each inspected feature and child definition. Discard it after a mutation.
//! No import payloads, incomplete sketch wires, external body history, or pending
//! child moves are certified. A failed check means "unverifiable", never "equal".

use std::collections::{BTreeMap, BTreeSet};

use parts::identity::canonical_json;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::Runner;

/// Only this self-referential record is excluded from a component's metadata.
pub const METADATA_KEY: &str = "org.techsquid.parts";
const SNAPSHOT_VERSION: u16 = 2;
const MAX_DEPTH: usize = 8;
const MAX_COMPONENTS: usize = 32;
/// Leave room within the host's 256 calls for creation and placement. The caller
/// must include these reads in the complete operation's preflight budget.
const MAX_READS: usize = 128;

pub const COMMANDS: [&str; 12] = [
    "appearance.list",
    "component.list",
    "component.metadata",
    "component.properties",
    "document.inspect",
    "joint.list",
    "material.list",
    "model.body_features",
    "parameters.list",
    "parameters.users",
    "sketch.inspect",
    "timeline.inspect",
];

/// Document-local digest of the inspected contents, not a purchasing identity.
/// Keep the canonical product/representation identity separately in the record.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    version: u16,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Component {
    pub id: u64,
    pub name: String,
    pub bodies: Vec<String>,
    features: Vec<String>,
    sketches: Vec<String>,
    construction: Vec<String>,
    children: Vec<Child>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Child {
    id: u64,
    component: u64,
    parent: u64,
    grounded: bool,
    transform: [[f64; 4]; 4],
}

#[derive(Deserialize)]
struct ComponentList {
    active: u64,
    components: Vec<Component>,
    pending_moves: BTreeSet<u64>,
}

#[derive(Deserialize)]
struct Document {
    metadata: Value,
    timeline: Vec<TimelineRow>,
}

/// Only semantic parameter state belongs in a snapshot. Comments, favourites,
/// display order and unrelated dependents do not change a part.
#[derive(Clone, Deserialize, Serialize)]
struct Parameter {
    name: String,
    expression: String,
    unit: String,
    value: Option<f64>,
    text: Option<String>,
    error: Option<String>,
    uses: Vec<String>,
}

#[derive(Deserialize)]
struct Parameters {
    parameters: Vec<Parameter>,
}

#[derive(Deserialize)]
struct ParameterUsers {
    drives: Vec<FeatureId>,
}

#[derive(Deserialize)]
struct TimelineRow {
    id: u64,
    name: String,
}

#[derive(Clone, Deserialize)]
struct Feature {
    id: u64,
    #[serde(rename = "type")]
    kind: String,
    definition: Value,
    inputs: Vec<Value>,
    omitted: Vec<Omitted>,
    suppressed: bool,
    rolled_back: bool,
    error: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Omitted {
    field: String,
}

#[derive(Deserialize)]
struct BodyHistories {
    bodies: Vec<BodyHistory>,
}

#[derive(Deserialize)]
struct BodyHistory {
    body: String,
    features: Vec<FeatureId>,
}

#[derive(Deserialize)]
struct FeatureId {
    id: u64,
}

#[derive(Deserialize)]
struct Materials {
    materials: Vec<Value>,
}

#[derive(Deserialize)]
struct Joints {
    joints: Vec<Joint>,
}

#[derive(Deserialize)]
struct Joint {
    a: u64,
    b: u64,
}

#[derive(Deserialize)]
struct Properties {
    bodies: Vec<BodyMaterial>,
}

#[derive(Deserialize)]
struct BodyMaterial {
    name: String,
    material: String,
    density_g_cm3: Option<f64>,
}

#[derive(Serialize)]
struct FeatureContent {
    id: u64,
    kind: String,
    definition: Value,
    inputs: Vec<Value>,
    sketch: Option<Value>,
}

#[derive(Serialize)]
struct BodyContent {
    name: String,
    history: Vec<u64>,
    material: String,
    density_g_cm3: Option<f64>,
    material_definition: Option<Value>,
    appearance: Value,
}

#[derive(Serialize)]
struct ChildContent {
    occurrence: Child,
    contents: Snapshot,
}

#[derive(Serialize)]
struct Contents {
    features: Vec<FeatureContent>,
    bodies: Vec<BodyContent>,
    children: Vec<ChildContent>,
    metadata: Value,
    parameters: Vec<Parameter>,
    appearance: Value,
}

/// A read-only view. Instantiate a new one after any design change.
pub struct Index {
    pub document_metadata: Value,
    pub active: u64,
    pub components: Vec<Component>,
    timeline: Vec<TimelineRow>,
    params: BTreeMap<String, Parameter>,
    parameter_drives: BTreeMap<String, BTreeSet<u64>>,
    histories: BTreeMap<String, Vec<u64>>,
    materials: Vec<Value>,
    appearances: Value,
    joints: Vec<Joint>,
    pending_moves: BTreeSet<u64>,
    features: BTreeMap<u64, Feature>,
    metadata: BTreeMap<u64, Value>,
    snapshots: BTreeMap<u64, Snapshot>,
    visiting: BTreeSet<u64>,
    reads: usize,
}

fn decode<T: DeserializeOwned>(value: Value, context: &str) -> Result<T, String> {
    serde_json::from_value(value).map_err(|e| format!("cannot inspect {context}: {e}"))
}

impl Index {
    pub fn read(r: &mut impl Runner) -> Result<Self, String> {
        let listed: ComponentList = decode(r.run("component.list", json!({}))?, "components")?;
        let doc: Document = decode(
            r.run(
                "document.inspect",
                json!({"measure": false, "bodies": false}),
            )?,
            "timeline",
        )?;
        let histories: BodyHistories =
            decode(r.run("model.body_features", json!({}))?, "body histories")?;
        let materials: Materials = decode(r.run("material.list", json!({}))?, "materials")?;
        let appearances = r.run("appearance.list", json!({}))?;
        let joints: Joints = decode(r.run("joint.list", json!({}))?, "joints")?;
        let params: Parameters = decode(r.run("parameters.list", json!({}))?, "parameters")?;
        Ok(Self {
            document_metadata: doc.metadata,
            active: listed.active,
            components: listed.components,
            timeline: doc.timeline,
            params: params
                .parameters
                .into_iter()
                .map(|p| (p.name.clone(), p))
                .collect(),
            parameter_drives: BTreeMap::new(),
            histories: histories
                .bodies
                .into_iter()
                .map(|b| (b.body, b.features.into_iter().map(|f| f.id).collect()))
                .collect(),
            materials: materials.materials,
            appearances,
            joints: joints.joints,
            pending_moves: listed.pending_moves,
            features: BTreeMap::new(),
            metadata: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            visiting: BTreeSet::new(),
            reads: 7,
        })
    }

    pub fn reads(&self) -> usize {
        self.reads
    }

    fn run(&mut self, r: &mut impl Runner, command: &str, args: Value) -> Result<Value, String> {
        if self.reads >= MAX_READS {
            return Err(
                "part inspection exceeds its 128-call budget; reduce this insertion batch".into(),
            );
        }
        self.reads += 1;
        r.run(command, args)
    }

    pub fn metadata(&mut self, r: &mut impl Runner, component: u64) -> Result<Value, String> {
        if let Some(value) = self.metadata.get(&component) {
            return Ok(value.clone());
        }
        let reply = self.run(r, "component.metadata", json!({"component": component}))?;
        let value = reply
            .get("metadata")
            .filter(|v| v.is_object())
            .ok_or("component metadata was not returned")?
            .clone();
        self.metadata.insert(component, value.clone());
        Ok(value)
    }

    fn feature(&mut self, r: &mut impl Runner, id: u64) -> Result<Feature, String> {
        if let Some(feature) = self.features.get(&id) {
            return Ok(feature.clone());
        }
        let reply = self.run(r, "timeline.inspect", json!({"feature": id}))?;
        let feature: Feature = decode(reply, "feature")?;
        if feature.id != id {
            return Err("host returned a different feature".into());
        }
        self.features.insert(id, feature.clone());
        Ok(feature)
    }

    pub fn snapshot(&mut self, r: &mut impl Runner, component: u64) -> Result<Snapshot, String> {
        if let Some(snapshot) = self.snapshots.get(&component) {
            return Ok(snapshot.clone());
        }
        if component == 0 {
            return Err("the design root is not a reusable part".into());
        }
        if self.visiting.contains(&component) {
            return Err("cycle in component children".into());
        }
        if self.visiting.len() >= MAX_DEPTH
            || self.snapshots.len() + self.visiting.len() >= MAX_COMPONENTS
        {
            return Err(
                "part hierarchy exceeds inspection limits (8 levels, 32 definitions)".into(),
            );
        }
        let node = self
            .components
            .iter()
            .find(|c| c.id == component)
            .ok_or("part component is missing")?
            .clone();
        self.visiting.insert(component);
        let contents = self.contents(r, &node);
        self.visiting.remove(&component);
        let contents = contents?;
        let bytes = canonical_json(&contents).map_err(|e| e.to_string())?;
        let snapshot = Snapshot {
            version: SNAPSHOT_VERSION,
            sha256: format!("{:x}", Sha256::digest(bytes.as_bytes())),
        };
        self.snapshots.insert(component, snapshot.clone());
        Ok(snapshot)
    }

    fn contents(&mut self, r: &mut impl Runner, node: &Component) -> Result<Contents, String> {
        let names: Vec<_> = node
            .features
            .iter()
            .chain(&node.sketches)
            .chain(&node.construction)
            .collect();
        let candidates: Vec<_> = self
            .timeline
            .iter()
            .filter(|row| names.contains(&&row.name))
            .map(|row| row.id)
            .collect();
        let mut owned = Vec::new();
        for id in candidates {
            let feature = self.feature(r, id)?;
            // Native features omit `component` for the root (document format's
            // explicit zero default). A malformed nonzero field is not accepted.
            let owner = match feature.definition.get("component") {
                None => 0,
                Some(value) => value
                    .as_u64()
                    .ok_or("feature ownership is not inspectable")?,
            };
            if owner == node.id {
                owned.push(feature);
            }
        }
        if owned.len() != names.len() {
            return Err("component feature ownership is incomplete".into());
        }
        let ids: BTreeSet<_> = owned.iter().map(|f| f.id).collect();
        let mut features = Vec::new();
        for f in owned {
            if f.suppressed || f.rolled_back || f.error.is_some() {
                return Err(format!(
                    "feature {} is suppressed, rolled back, or failed",
                    f.id
                ));
            }
            if f.omitted.iter().any(|o| o.field != "sketch") {
                return Err(format!(
                    "feature {} contains an uninspectable payload",
                    f.id
                ));
            }
            let sketch = if f.kind == "sketch" {
                let value = self.run(r, "sketch.inspect", json!({"sketch": f.id}))?;
                Some(sketch_contents(value)?)
            } else {
                None
            };
            features.push(FeatureContent {
                id: f.id,
                kind: f.kind,
                definition: f.definition,
                inputs: f.inputs,
                sketch,
            });
        }
        let properties: Properties = decode(
            self.run(
                r,
                "component.properties",
                json!({"component": node.id, "descendants": false}),
            )?,
            "body materials",
        )?;
        if properties.bodies.len() != node.bodies.len() {
            return Err("component body/material inventory disagrees".into());
        }
        let mut bodies = Vec::new();
        for name in &node.bodies {
            let history = self
                .histories
                .get(name)
                .ok_or("body history is missing")?
                .clone();
            if history.is_empty() || history.iter().any(|id| !ids.contains(id)) {
                return Err(format!("{name} has geometry from outside its definition"));
            }
            let material = properties
                .bodies
                .iter()
                .find(|b| &b.name == name)
                .ok_or_else(|| format!("material for {name} is not inspectable"))?;
            // Resolved appearance includes physical material defaults and face looks.
            let appearance = self
                .run(r, "appearance.list", json!({"body": name}))?
                .get("body")
                .filter(|v| v.is_object())
                .ok_or("resolved body appearance is missing")?
                .clone();
            let mut appearance = appearance;
            appearance
                .as_object_mut()
                .ok_or("body appearance is not an object")?
                .remove("name");
            bodies.push(BodyContent {
                name: name.clone(),
                history,
                material: material.material.clone(),
                density_g_cm3: material.density_g_cm3,
                material_definition: self
                    .materials
                    .iter()
                    .find(|v| v["name"].as_str() == Some(material.material.as_str()))
                    .cloned(),
                appearance,
            });
        }
        let mut children = Vec::new();
        for child in &node.children {
            if child.parent != node.id {
                return Err("child ownership is inconsistent".into());
            }
            if self.pending_moves.contains(&child.id) {
                return Err("an assembly child has an uncaptured move".into());
            }
            if self
                .joints
                .iter()
                .any(|j| j.a == child.id || j.b == child.id)
            {
                return Err(
                    "an assembly child has a joint; this fixed assembly cannot be certified".into(),
                );
            }
            children.push(ChildContent {
                occurrence: child.clone(),
                contents: self.snapshot(r, child.component)?,
            });
        }
        if bodies.is_empty() && children.is_empty() {
            return Err("part has no solid bodies or children".into());
        }
        let mut metadata = self.metadata(r, node.id)?;
        metadata
            .as_object_mut()
            .ok_or("metadata is not an object")?
            .remove(METADATA_KEY);
        let appearance = self
            .appearances
            .get("components")
            .and_then(Value::as_array)
            .ok_or("component appearances are not inspectable")?
            .iter()
            .filter(|a| a["component"].as_u64() == Some(node.id))
            .cloned()
            .collect::<Vec<_>>();
        let parameters = self.referenced_parameters(r, &features)?;
        Ok(Contents {
            features,
            bodies,
            children,
            metadata,
            parameters,
            appearance: json!(appearance),
        })
    }
    /// Candidate tokens only reduce host queries; they never establish a dependency.
    /// The native dependency walker verifies feature IDs, including placement and
    /// sketch expressions, without duplicating the host's expression grammar here.
    fn referenced_parameters(
        &mut self,
        r: &mut impl Runner,
        features: &[FeatureContent],
    ) -> Result<Vec<Parameter>, String> {
        let mut candidates = BTreeSet::new();
        for feature in features {
            identifier_candidates(&feature.definition, &mut candidates);
            for input in &feature.inputs {
                identifier_candidates(&input["expression"], &mut candidates);
            }
            if let Some(sketch) = &feature.sketch {
                identifier_candidates(sketch, &mut candidates);
            }
        }
        let owned: BTreeSet<_> = features.iter().map(|f| f.id).collect();
        let mut pending = Vec::new();
        for name in candidates {
            if !self.params.contains_key(&name) {
                continue;
            }
            if !self.parameter_drives.contains_key(&name) {
                let users: ParameterUsers = decode(
                    self.run(r, "parameters.users", json!({"name": name}))?,
                    "parameter dependencies",
                )?;
                self.parameter_drives.insert(
                    name.clone(),
                    users.drives.into_iter().map(|f| f.id).collect(),
                );
            }
            if !self.parameter_drives[&name].is_disjoint(&owned) {
                pending.push(name);
            }
        }
        let mut used = BTreeMap::new();
        while let Some(name) = pending.pop() {
            if used.contains_key(&name) {
                continue;
            }
            let param = self
                .params
                .get(&name)
                .ok_or_else(|| format!("parameter {name} is missing"))?;
            if param.error.is_some() {
                return Err(format!("parameter {name} has an error"));
            }
            pending.extend(param.uses.iter().cloned());
            used.insert(name, param.clone());
        }
        Ok(used.into_values().collect())
    }
}

/// The host reports wire endpoints/counts but not all interior vertices. Generated
/// library profiles use ordinary curves; never certify an incomplete wire record.
fn sketch_contents(mut value: Value) -> Result<Value, String> {
    let fields = value
        .as_object_mut()
        .ok_or("sketch contents are not an object")?;
    if !fields
        .get("wires")
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty)
    {
        return Err("sketch has an incompletely inspectable wire".into());
    }
    for key in ["plane", "points", "curves", "constraints", "links"] {
        if !fields.contains_key(key) {
            return Err(format!("sketch {key} are missing"));
        }
    }
    for key in [
        "id",
        "name",
        "visible",
        "profiles",
        "profiles_3d",
        "dof",
        "status",
        "fully_constrained",
        "param_warnings",
    ] {
        fields.remove(key);
    }
    let points = fields
        .get_mut("points")
        .and_then(Value::as_array_mut)
        .ok_or("sketch points are missing")?;
    for point in points {
        // World coordinates follow the occurrence. The plane and local `at` are
        // intrinsic to the definition and remain in the snapshot.
        point
            .as_object_mut()
            .ok_or("sketch point is not an object")?
            .remove("world");
    }
    Ok(value)
}

fn identifier_candidates(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::String(s) => found.extend(
            s.split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|s| !s.is_empty())
                .map(str::to_string),
        ),
        Value::Array(a) => a.iter().for_each(|v| identifier_candidates(v, found)),
        Value::Object(o) => o.values().for_each(|v| identifier_candidates(v, found)),
        _ => {}
    }
}
