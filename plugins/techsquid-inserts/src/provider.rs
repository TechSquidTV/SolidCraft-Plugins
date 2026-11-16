//! Discovery of the same rows used by the heat-set insertion workflow.

use parts::catalog::{
    self, CatalogInfo, Dimension, Frame, Insertion, PartDescription, PartSummary, Provenance,
    ResolvedPart, Route, Source,
};
use parts::{Catalog, CatalogError, Product, ProductKind, Representation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    catalog::{rows, Row},
    insert,
};

pub struct Inserts;

/// Increment when catalog choices, dimensions, or resolution rules change.
pub const REVISION: &str = "1";
/// Increment when the procedural knurl, thread, or default appearance changes.
const GEOMETRY_REVISION: &str = "1";
const GUIDELINES: &str =
    "https://www.cnckitchen.com/s/CNCKitchen_Heat-Set-Insert-Dimensions-and-Design-Guidelines.pdf";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Specification {
    pub thread: String,
    pub length_mm: f64,
    pub diameter_mm: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    #[serde(default)]
    detail: parts::detail::ThreadDetail,
}

/// These three existing generator rows are absent from the source dimension table.
/// Keep them discoverable as custom models without asserting a supplier product.
fn documented(row: &Row) -> bool {
    !matches!(row.id, "m1.6-2.5" | "m6-6.8-short" | "m8-6.8-short")
}

fn row(id: &str) -> Result<&'static Row, CatalogError> {
    rows()
        .find(|row| row.id == id)
        .ok_or_else(|| CatalogError::new("part", format!("unknown insert {id}")))
}

fn summary(row: &Row) -> PartSummary {
    PartSummary {
        id: row.id.into(),
        name: format!("Heat-set insert {}", row.size),
        family: "heat-set-inserts".into(),
        kind: if documented(row) {
            ProductKind::SupplierProduct
        } else {
            ProductKind::Custom
        },
        designation: row.size.into(),
    }
}

fn dimensions(row: &Row) -> Vec<Dimension> {
    vec![
        Dimension::mm("length", row.length),
        Dimension::mm("outside_diameter", row.diameter),
        Dimension::mm("recommended_hole_diameter", row.hole),
        Dimension::mm("minimum_wall", row.wall),
    ]
}

fn configuration_schema(row: &Row) -> Value {
    let details = if crate::catalog::metric(row.thread) {
        vec!["cosmetic", "modeled"]
    } else {
        vec!["cosmetic"]
    };
    json!({"type": "object", "additionalProperties": false, "properties": {
        "detail": {"type": "string", "enum": details, "default": "cosmetic"}
    }})
}

/// Resolve physical identity from the workflow's validated parameters. Hole cutting
/// and placement are deliberately absent; overrides become a custom product.
pub fn representation(p: &insert::Params) -> Representation<Specification> {
    // Params.size is set only by the existing table lookup in insert::parse.
    let row = crate::catalog::get(p.size).expect("validated insert row");
    let supplied = !p.custom
        && documented(row)
        && p.thread == row.thread
        && p.length == row.length
        && p.diameter == row.diameter;
    Representation {
        product: Product {
            catalog: if supplied {
                "cnc-kitchen"
            } else {
                "techsquid-custom"
            }
            .into(),
            part: row.id.into(),
            kind: if supplied {
                ProductKind::SupplierProduct
            } else {
                ProductKind::Custom
            },
            configuration: Specification {
                thread: p.thread.clone(),
                length_mm: p.length,
                diameter_mm: p.diameter,
            },
            members: vec![],
        },
        revision: GEOMETRY_REVISION.into(),
        detail: if p.modeled { "modeled" } else { "cosmetic" }.into(),
        dependencies: vec![],
    }
}

pub fn validate_selection(value: &Value, p: &insert::Params) -> Result<(), CatalogError> {
    catalog::validate_selection(value, REVISION, &representation(p))
}

pub fn part_number(p: &insert::Params) -> Result<String, String> {
    let product = representation(p).product;
    if product.kind == ProductKind::Custom {
        plugin_kit::component::model_number(&crate::body_base(p), &product)
    } else {
        Ok(format!("CNC Kitchen insert {}", p.size))
    }
}

impl Catalog for Inserts {
    type Configuration = Specification;

    fn info(&self) -> CatalogInfo {
        CatalogInfo {
            id: "techsquid-inserts".into(),
            name: "Heat-set inserts".into(),
            revision: REVISION.into(),
        }
    }

    fn summaries(&self) -> Vec<PartSummary> {
        rows().map(summary).collect()
    }

    fn describe(&self, id: &str) -> Result<PartDescription, CatalogError> {
        let row = row(id)?;
        let schema = configuration_schema(row);
        Ok(PartDescription {
            part: summary(row),
            dimensions: dimensions(row),
            configuration_schema: schema.clone(),
            sources: if documented(row) {
                vec![Source {
                    title: "CNC Kitchen dimension guidelines".into(),
                    url: GUIDELINES.into(),
                }]
            } else {
                vec![]
            },
            details: schema["properties"]["detail"]["enum"]
                .as_array()
                .expect("detail choices")
                .iter()
                .map(|v| v.as_str().expect("detail choice").into())
                .collect(),
            provenance: Provenance {
                supplier: documented(row).then(|| "CNC Kitchen".into()),
                ..Provenance::default()
            },
            attachments: vec![Frame::axial("top", 0.0), Frame::axial("bottom", -row.length)],
            insertion: Route { plugin: crate::PLUGIN.into(), entry: "make-insert".into() },
            notes: vec!["Knurls and threads are procedural representations; no supplier stock availability is asserted.".into()],
        })
    }

    fn resolve(
        &self,
        id: &str,
        value: &Value,
    ) -> Result<ResolvedPart<Specification>, CatalogError> {
        let row = row(id)?;
        let configuration: Configuration = serde_json::from_value(value.clone())
            .map_err(|e| CatalogError::new("configuration", e.to_string()))?;
        let modeled = configuration.detail.modeled();
        if modeled && !crate::catalog::metric(row.thread) {
            return Err(CatalogError::new(
                "detail",
                "this host supports modeled insert threads only for metric sizes",
            ));
        }
        let params = json!({"size": row.size, "create": "Insert only", "thread_modeled": modeled});
        let parsed = insert::parse(&params).map_err(|e| CatalogError::new(e.field, e.reason))?;
        let representation = representation(&parsed);
        let insertion = Insertion::resolved(
            crate::PLUGIN,
            "make-insert",
            params,
            REVISION,
            &representation,
        )?;
        Ok(ResolvedPart {
            representation,
            description: self.describe(id)?,
            members: vec![],
            insertion,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_is_sorted_bounded_and_pins_the_catalog_revision() {
        let mut ids = Vec::new();
        let mut offset = 0;
        loop {
            let page = Inserts
                .query(&json!({"version": 1, "request": {
                    "operation": "list", "offset": offset, "limit": 7
                }}))
                .unwrap()
                .json()
                .unwrap();
            let items = page["result"]["items"].as_array().unwrap();
            assert!(items.len() <= 7);
            ids.extend(
                items
                    .iter()
                    .map(|part| part["id"].as_str().unwrap().to_string()),
            );
            match page["result"]["next_offset"].as_u64() {
                Some(next) => offset = next,
                None => break,
            }
        }
        assert_eq!(ids.len(), rows().count());
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        let matches = Inserts
            .query(&json!({"version": 1, "request": {
                "operation": "list", "query": "Voron"
            }}))
            .unwrap()
            .json()
            .unwrap();
        assert_eq!(matches["result"]["total"], 1);
        assert_eq!(matches["result"]["items"][0]["id"], "m3-4-voron");
        let error = Inserts
            .query(&json!({"version": 1, "revision": "stale", "request": {
                "operation": "resolve", "part": "m3-5.7"
            }}))
            .unwrap_err();
        assert_eq!(error.field, "revision");
    }

    #[test]
    fn packaged_query_schema_matches_the_shared_contract() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let entries = manifest["runtime"]["entries"].as_array().unwrap();
        let catalog = entries
            .iter()
            .find(|entry| entry["name"] == "catalog")
            .unwrap();
        assert_eq!(catalog["effect"], "read");
        assert_eq!(catalog["input_schema"], catalog::entry_schema());
        assert!(!catalog.as_object().unwrap().contains_key("output_schema"));
    }

    #[test]
    fn every_catalog_row_round_trips_through_the_real_workflow() {
        for item in Inserts.summaries() {
            let resolved = Inserts.resolve(&item.id, &json!({})).unwrap();
            let p = insert::parse(&resolved.insertion.params).unwrap();
            assert_eq!(resolved.representation, representation(&p));
            assert_eq!(resolved.representation.product.kind, item.kind);
        }
    }

    #[test]
    fn published_dimensions_and_ids_are_independent_of_display_formatting() {
        let part = Inserts
            .resolve("m3-5.7", &json!({"detail": "modeled"}))
            .unwrap();
        let spec = &part.representation.product.configuration;
        assert_eq!((spec.diameter_mm, spec.length_mm), (4.6, 5.7));
        assert_eq!(part.representation.product.catalog, "cnc-kitchen");
        assert_eq!(part.representation.product.part, "m3-5.7");
        assert_eq!(part.representation.detail, "modeled");
    }

    #[test]
    fn unsupported_details_and_unknown_fields_are_not_silently_replaced() {
        assert!(Inserts
            .resolve("no-2-56-3.2", &json!({"detail": "modeled"}))
            .is_err());
        assert!(Inserts.resolve("m3-5.7", &json!({"length": 6})).is_err());
        assert!(Inserts
            .resolve("m3-5.7", &json!({"detail": "high"}))
            .is_err());
        assert!(Inserts.resolve("M3 × 5.7", &json!({})).is_err());
    }

    #[test]
    fn stale_or_modified_resolved_inputs_are_rejected_before_insertion() {
        let original = Inserts
            .resolve("m3-5.7", &json!({}))
            .unwrap()
            .insertion
            .params;
        for (key, value) in [
            ("custom_length", json!(6)),
            ("size", json!("M4 × 8.1")),
            ("thread_modeled", json!(true)),
        ] {
            let mut changed = original.clone();
            changed[key] = value;
            assert!(insert::parse(&changed).is_err(), "{changed}");
        }
        let mut stale = original;
        stale["catalog_selection"]["catalog_revision"] = json!("old");
        assert!(insert::parse(&stale).is_err());
    }

    #[test]
    fn mounting_options_are_not_part_of_purchasing_identity() {
        let resolved = Inserts.resolve("m3-5.7", &json!({})).unwrap();
        let mut value = resolved.insertion.params;
        value["create"] = json!("Hole and insert");
        value["hole_adjust"] = json!(0.1);
        let p = insert::parse(&value).unwrap();
        assert_eq!(representation(&p), resolved.representation);
    }

    #[test]
    fn custom_dimensions_and_unsourced_rows_do_not_claim_supplier_identity() {
        for value in [
            json!({"size": "M3 × 5.7", "custom_length": 7}),
            json!({"size": "M3 × 5.7", "custom_length": 5.7}),
            json!({"size": "M3 × 5.7", "custom_thread": "M3"}),
            json!({"size": "M1.6 × 2.5"}),
        ] {
            let p = insert::parse(&value).unwrap();
            let identity = representation(&p).product;
            assert_eq!(identity.kind, ProductKind::Custom);
            assert_eq!(identity.catalog, "techsquid-custom");
        }
    }
}
