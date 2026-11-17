//! Representation-based reuse and assembly composition. The document directory is
//! only a lookup accelerator: every candidate's identity and current native contents
//! are checked. Names, creator plugin and stored hashes alone never qualify.

use super::{components, id_of, make_folder, put, Activation, Definition, Placed};
use crate::{
    content::{Index, Snapshot, METADATA_KEY},
    name::{reserve, reserve_part},
    timeline, Placement, Runner,
};
use parts::Representation;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const DIRECTORY_KEY: &str = "org.techsquid.parts.index";
const VERSION: u16 = 1;
const MAX_CALLS: usize = 248;

/// Conservative creation cost for the resolved recipe, verified against host tests.
/// Reusing a verified definition needs only its placement budget. A runtime guard
/// still bounds unexpected host work and makes the outer transaction roll back.
pub struct Budget {
    pub creation_calls: usize,
    pub placements: usize,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u16,
    product: String,
    representation: String,
    creator: String,
    site: [f64; 3],
    snapshot: Snapshot,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Directory {
    version: u16,
    definitions: BTreeMap<String, Vec<u64>>,
}

/// A definition resolved in this invocation. Placements consume its initial
/// occurrence once, then copy the same definition for subsequent uses.
#[derive(Clone, Debug)]
pub struct Handle {
    pub component: u64,
    pub name: String,
    pub reused: bool,
    key: String,
    site: [f64; 3],
}

struct Resolved {
    handle: Handle,
    first: Option<u64>,
}

struct Counted<'a, R> {
    host: &'a mut R,
    calls: usize,
}
impl<R: Runner> Runner for Counted<'_, R> {
    fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
        if self.calls >= MAX_CALLS {
            return Err("parts: insertion exceeds its command budget; use a smaller batch".into());
        }
        self.calls += 1;
        self.host.run(command, args)
    }
    fn command_count(&self) -> usize {
        self.host.command_count()
    }
    fn body_names(&mut self) -> Vec<String> {
        self.host.body_names()
    }
    fn site(&mut self, at: [f64; 2]) -> Placement {
        self.host.site(at)
    }
}

/// One plugin transaction. Call `finish` after every resolved part has been placed.
/// The host's edit-entry transaction owns rollback and undo; errors must propagate.
pub struct Library<'a, R: Runner> {
    host: Counted<'a, R>,
    index: Index,
    directory: Directory,
    resolved: BTreeMap<String, Resolved>,
    visiting: BTreeSet<String>,
    original_active: Activation,
    active: Activation,
    dirty: bool,
    rejected: Vec<Value>,
    budget: Budget,
}

impl<'a, R: Runner> Library<'a, R> {
    pub fn new(host: &'a mut R, budget: Budget) -> Result<Self, String> {
        if budget.placements == 0 || budget.placements > 32 || budget.creation_calls > MAX_CALLS {
            return Err("parts: choose 1–32 placements within the recipe budget".into());
        }
        let calls = host.command_count();
        let mut host = Counted { host, calls };
        let index = Index::read(&mut host)?;
        let directory: Directory = match index.document_metadata.get(DIRECTORY_KEY) {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|e| format!("parts directory is invalid: {e}"))?,
            None => Directory {
                version: VERSION,
                definitions: BTreeMap::new(),
            },
        };
        if directory.version != VERSION {
            return Err("unsupported parts directory version".into());
        }
        let active = index.activation.clone();
        Ok(Self {
            host,
            index,
            directory,
            resolved: BTreeMap::new(),
            visiting: BTreeSet::new(),
            original_active: active.clone(),
            active,
            dirty: false,
            rejected: Vec::new(),
            budget,
        })
    }

    pub fn runner(&mut self) -> &mut dyn Runner {
        &mut self.host
    }

    pub fn active_component(&self) -> u64 {
        self.active.component
    }

    fn activate(&mut self, activation: Activation) -> Result<(), String> {
        if self.active != activation {
            activation.restore(&mut self.host)?;
            self.active = activation;
        }
        Ok(())
    }

    /// Fold only this definition's own geometry rows. Nested definitions retain
    /// their own folders and are composed through `ensure` and `place`.
    pub fn geometry<F>(&mut self, name: &str, build: F) -> Result<(), String>
    where
        F: FnOnce(&mut dyn Runner) -> Result<(), String>,
    {
        timeline::folded(&mut self.host, name, build)
    }

    pub fn ensure<C, F>(
        &mut self,
        def: &Definition<'_>,
        part_number: &str,
        representation: &Representation<C>,
        build: F,
    ) -> Result<Handle, String>
    where
        C: Serialize,
        F: FnOnce(&mut Self, &str, &Placement) -> Result<(), String>,
    {
        let product = representation
            .product
            .identity()
            .map_err(|e| e.to_string())?
            .as_str()
            .to_string();
        let representation = representation
            .identity()
            .map_err(|e| e.to_string())?
            .as_str()
            .to_string();
        // Full canonical text is the in-memory key. The digest only finds
        // persistent candidates and cannot authorize their reuse.
        if let Some(resolved) = self.resolved.get(&representation) {
            return Ok(resolved.handle.clone());
        }
        if self.visiting.contains(&representation) {
            return Err("cycle in part assembly recipe".into());
        }
        if self.visiting.len() >= 8 || self.resolved.len() + self.visiting.len() >= 32 {
            return Err("parts: assembly exceeds 8 levels or 32 definitions".into());
        }
        let digest = format!("{:x}", Sha256::digest(representation.as_bytes()));
        let candidates = self
            .directory
            .definitions
            .get(&digest)
            .cloned()
            .unwrap_or_default();
        for id in candidates {
            let Some(node) = self.index.components.iter().find(|c| c.id == id).cloned() else {
                continue;
            };
            let metadata = self.index.metadata(&mut self.host, id)?;
            let record: Result<Record, _> =
                serde_json::from_value(metadata.get(METADATA_KEY).cloned().unwrap_or(Value::Null));
            let reason = match record {
                Ok(record)
                    if record.version == VERSION
                        && record.representation == representation
                        && record.product == product
                        && record.site == self.host.site(def.site.of(id)).point =>
                {
                    match self.index.snapshot(&mut self.host, id) {
                        Ok(snapshot) if snapshot == record.snapshot => {
                            let handle = Handle {
                                component: id,
                                name: node.name,
                                reused: true,
                                key: representation.clone(),
                                site: record.site,
                            };
                            self.resolved.insert(
                                representation,
                                Resolved {
                                    handle: handle.clone(),
                                    first: None,
                                },
                            );
                            return Ok(handle);
                        }
                        Ok(_) => "native contents were modified".into(),
                        Err(error) => error,
                    }
                }
                _ => "identity or build frame is missing, changed, or unsupported".into(),
            };
            self.rejected
                .push(json!({"component": id, "name": node.name, "reason": reason}));
        }
        let previous = self.active.clone();
        if self.visiting.is_empty()
            && self.host.calls + self.budget.creation_calls + self.budget.placements * 3 + 8
                > MAX_CALLS
        {
            return Err("parts: this creation batch exceeds the command budget; insert fewer points, then reuse the part".into());
        }
        let mut taken: Vec<_> = self
            .index
            .components
            .iter()
            .flat_map(|c| c.names().map(str::to_string))
            .collect();
        let bodies: Vec<_> = self
            .index
            .components
            .iter()
            .flat_map(|c| c.bodies.iter().map(String::as_str))
            .collect();
        let name = reserve_part(&mut taken, def.base, &bodies);
        let made = self.host.run(
            "component.create",
            json!({"name": name, "parent": 0, "activate": true, "grounded": false}),
        )?;
        let component = id_of(&made, "component")?;
        let occurrence = id_of(&made, "occurrence")?;
        self.active = Activation::created(component, occurrence);
        let frame = self.host.site(def.site.of(component));
        self.visiting.insert(representation.clone());
        let result = build(self, &name, &frame);
        self.visiting.remove(&representation);
        result?;
        self.host.run(
            "component.metadata.set",
            json!({"component": component, "part_number": part_number,
            "description": def.description, "bom_type": def.bom_type}),
        )?;
        // A mutation invalidates the old view. Seal finished contents, including
        // child transforms, after setting purchasing properties.
        self.index = Index::read(&mut self.host)?;
        let snapshot = self.index.snapshot(&mut self.host, component)?;
        let record = Record {
            version: VERSION,
            product,
            representation: representation.clone(),
            creator: def.plugin.into(),
            site: frame.point,
            snapshot,
        };
        self.host.run(
            "component.metadata.set",
            json!({"component": component, "values": {METADATA_KEY: record}}),
        )?;
        self.directory.definitions.insert(digest, vec![component]);
        self.dirty = true;
        self.activate(previous)?;
        let handle = Handle {
            component,
            name,
            reused: false,
            key: representation.clone(),
            site: frame.point,
        };
        self.resolved.insert(
            representation,
            Resolved {
                handle: handle.clone(),
                first: Some(occurrence),
            },
        );
        Ok(handle)
    }

    /// Place inside `parent`. During composition the parent's build occurrence is
    /// still at the origin. Shared placement code handles build-site correction.
    pub fn place(
        &mut self,
        handle: &Handle,
        frame: &Placement,
        parent: u64,
    ) -> Result<u64, String> {
        let resolved = self
            .resolved
            .get_mut(&handle.key)
            .ok_or("part handle belongs to a different invocation")?;
        let occurrence = match resolved.first.take() {
            Some(id) => id,
            None => id_of(
                &self.host.run(
                    "occurrence.copy",
                    json!({"component": handle.component, "parent": 0}),
                )?,
                "occurrence",
            )?,
        };
        let active = self.active.clone();
        self.activate(Activation::root())?;
        put(&mut self.host, occurrence, frame, handle.site, parent)?;
        self.activate(active)?;
        Ok(occurrence)
    }

    pub fn place_all(
        &mut self,
        handle: &Handle,
        places: &[Placement],
        folder: &str,
    ) -> Result<Placed, String> {
        if places.is_empty() || places.len() > 32 {
            return Err("parts: choose 1–32 placements".into());
        }
        let listed = self.host.run("component.list", json!({"compact": true}))?;
        let listed = components(&listed);
        // A child belongs to the folder definition, so inserting into a copied
        // folder would multiply purchases. Only a single root placement is safe.
        // Walk the shared naming sequence so a numbered replacement is reused.
        let mut unavailable = Vec::new();
        let folder = loop {
            let name = reserve(&mut unavailable, folder);
            match listed.iter().find(|c| c.name == name) {
                Some(c)
                    if c.id != 0
                        && c.parents == [0]
                        && self
                            .host
                            .run("component.metadata", json!({"component": c.id}))?["bom_type"]
                            == "phantom" =>
                {
                    break c.id;
                }
                Some(_) => continue,
                None => break make_folder(&mut self.host, &name)?,
            }
        };
        let mut occurrences = Vec::new();
        // Place a whole batch in world coordinates with one context switch.
        let active = self.active.clone();
        self.activate(Activation::root())?;
        for place in places {
            occurrences.push(self.place(handle, place, folder)?);
        }
        self.activate(active)?;
        Ok(Placed {
            component: handle.name.clone(),
            reused: handle.reused,
            occurrences,
        })
    }

    /// Complete context restoration and directory writes in the same undo operation.
    pub fn finish(mut self) -> Result<Value, String> {
        if self.resolved.values().any(|r| r.first.is_some()) {
            return Err("an assembly recipe left an unplaced definition".into());
        }
        if self.dirty {
            let value = serde_json::to_value(&self.directory).map_err(|e| e.to_string())?;
            if value.to_string().len() > 60 * 1024 {
                return Err("parts directory exceeds 60 KiB".into());
            }
            self.host.run(
                "document.metadata.set",
                json!({"key": DIRECTORY_KEY, "value": value}),
            )?;
        }
        self.activate(self.original_active.clone())?;
        Ok(json!({"host_calls": self.host.calls, "preserved_modified_definitions": self.rejected}))
    }
}
