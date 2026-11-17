//! Purchasing assignments use the host's actual part-number grouping rules.
use super::{Definition, Handle, Index, Library, Record, METADATA_KEY};
use crate::{Placement, Runner};
use parts::{purchasing::Details, Representation};
use serde::Serialize;
use serde_json::{json, Value};

const PURCHASE_KEY: &str = "org.techsquid.purchasing";

impl<R: Runner> Library<'_, R> {
    /// Assign an optional purchase to any design factory. Reject BOM-number
    /// conflicts before edits, then reuse geometry independently of its listing URL.
    pub fn ensure_assigned<C, F>(
        &mut self,
        def: &Definition<'_>,
        design: Representation<C>,
        details: &Details,
        build: F,
    ) -> Result<Handle, String>
    where
        C: Serialize,
        F: FnOnce(&mut Self, &str, &Placement) -> Result<(), String>,
    {
        details.validate().map_err(|e| e.to_string())?;
        let representation = details.assignment.representation(design);
        let product = representation
            .product
            .identity()
            .map_err(|e| e.to_string())?;
        let number = details.assignment.part_number.as_deref().unwrap_or("");
        if !number.is_empty() {
            // Check every definition, including currently excluded/unused parts:
            // changing visibility or placement must not reveal a latent collision.
            let components: Vec<_> = self
                .index
                .components
                .iter()
                .filter(|c| c.id != 0)
                .map(|c| (c.id, c.name.clone()))
                .collect();
            for (id, name) in components {
                let metadata = self.index.metadata(&mut self.host, id)?;
                if metadata.get("part_number").and_then(Value::as_str) != Some(number) {
                    continue;
                }
                let record = metadata
                    .get(METADATA_KEY)
                    .cloned()
                    .and_then(|v| serde_json::from_value::<Record>(v).ok());
                let matches = match record {
                    Some(record)
                        if record.version == super::VERSION
                            && record.product == product.as_str() =>
                    {
                        self.index.snapshot(&mut self.host, id)? == record.snapshot
                    }
                    _ => false,
                };
                if !matches {
                    return Err(format!("part_number: {number} is already used by incompatible or edited component {name}; choose a unique BOM number or leave it blank"));
                }
            }
        }
        let def = Definition {
            plugin: def.plugin,
            base: def.base,
            description: def.description,
            site: def.site,
            bom_type: if details.assignment.purchase.is_some() {
                "purchased"
            } else {
                def.bom_type
            },
        };
        let handle = self.ensure(&def, number, &representation, build)?;
        if details.assignment != Default::default() {
            self.record_purchase(&handle, details)?;
        }
        Ok(handle)
    }

    fn record_purchase(&mut self, handle: &Handle, details: &Details) -> Result<(), String> {
        // ensure has just verified this handle. Change only our annotation and
        // reseal the snapshot so a URL edit never blesses arbitrary modified geometry.
        self.index = Index::read(&mut self.host)?;
        let metadata = self.index.metadata(&mut self.host, handle.component)?;
        let previous = &metadata[PURCHASE_KEY];
        let url = details.url.as_deref().or_else(|| previous["url"].as_str());
        let value = json!({"version":1, "assignment":details.assignment, "url":url,
            "purchasing_identity":details.assignment.purchase.as_ref()
                .map(|p| p.identity()).transpose().map_err(|e| e.to_string())?});
        if *previous == value {
            return Ok(());
        }
        let mut record: Record = serde_json::from_value(metadata[METADATA_KEY].clone())
            .map_err(|e| format!("missing verified definition: {e}"))?;
        self.host.run(
            "component.metadata.set",
            json!({"component":handle.component,"values":{PURCHASE_KEY:value}}),
        )?;
        self.index = Index::read(&mut self.host)?;
        record.snapshot = self.index.snapshot(&mut self.host, handle.component)?;
        self.host.run(
            "component.metadata.set",
            json!({"component":handle.component,"values":{METADATA_KEY:record}}),
        )?;
        // Drop metadata cached before the new seal; later composition may inspect it.
        self.index = Index::read(&mut self.host)?;
        Ok(())
    }
}
