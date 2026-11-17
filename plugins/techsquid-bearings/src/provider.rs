//! Discovery delegates dimensions and identity to the shared bearing factory.

use bearings::Bearing;
use parts::catalog::{
    CatalogInfo, Dimension, Frame, Insertion, PartDescription, PartSummary, Provenance,
    ResolvedPart, Route, Source,
};
use parts::{Catalog, CatalogError, ProductKind};
use serde::Deserialize;
use serde_json::{json, Value};

pub struct Bearings;
pub const REVISION: &str = "5";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {}

fn bearing(id: &str) -> Result<Bearing, CatalogError> {
    bearings::ALL
        .into_iter()
        .find(|b| b.sku() == id)
        .ok_or_else(|| CatalogError::new("part", format!("unknown bearing {id}")))
}

fn summary(bearing: Bearing) -> PartSummary {
    PartSummary {
        id: bearing.sku().into(),
        name: format!("Bearing {}", bearing.designation()),
        family: "radial-ball-bearings".into(),
        kind: ProductKind::SupplierProduct,
        designation: bearing.designation().into(),
    }
}

impl Catalog for Bearings {
    type Configuration = Bearing;
    fn info(&self) -> CatalogInfo {
        CatalogInfo {
            id: crate::PLUGIN.into(),
            name: "Bearings".into(),
            revision: REVISION.into(),
        }
    }
    fn summaries(&self) -> Vec<PartSummary> {
        bearings::ALL.into_iter().map(summary).collect()
    }
    fn describe(&self, id: &str) -> Result<PartDescription, CatalogError> {
        let bearing = bearing(id)?;
        let d = bearing.dimensions();
        let mut dimensions = vec![
            Dimension::mm("bore", d.bore),
            Dimension::mm("outside_diameter", d.diameter),
            Dimension::mm("width", d.width),
            Dimension::mm("edge_radius", d.section.as_ref().unwrap().edge_radius),
        ];
        if let Some(recess) = d.section.as_ref().unwrap().closure_recess {
            dimensions.push(Dimension::mm("closure_recess", recess));
        }
        let mut sources = vec![Source {
            title: format!(
                "{} {} product reference",
                bearing.supplier().name,
                bearing.sku()
            ),
            url: bearing.product_url().into(),
        }];
        if bearing == Bearing::OpenBuilds105zz {
            sources.push(Source {
                title: "OpenBuilds correction of 105ZZ width to 4 mm".into(),
                url: "https://builds.openbuilds.com/threads/first-design-c-beam-router.5939/page-2"
                    .into(),
            });
        }
        Ok(PartDescription {
            part: summary(bearing), dimensions, sources,
            provenance: Provenance { supplier: Some(bearing.supplier().name.into()),
                supplier_part_number: Some(bearing.sku().into()), ..Provenance::default() },
            configuration_schema: json!({"type":"object","additionalProperties":false,"properties":{}}),
            details: vec!["exterior".into()],
            attachments: vec![Frame::axial("axle-centre", 0.0), Frame::axial("negative-seat", -d.width/2.0),
                Frame::axial("positive-seat", d.width/2.0)],
            insertion: Route { plugin: crate::PLUGIN.into(), entry: "make-bearing".into() },
            notes: vec!["Separate inner/outer rings and sourced closure surfaces; open B626 includes sourced exposed raceways and eight visible balls, without a cage. Hidden internals and closure thickness are not inferred; unsuitable for mass prediction.".into(),
                "Supplier product identity does not establish a bearing manufacturer, material grade, clearance or precision class.".into()],
        })
    }
    fn resolve(&self, id: &str, value: &Value) -> Result<ResolvedPart<Bearing>, CatalogError> {
        let _: Configuration = serde_json::from_value(value.clone())
            .map_err(|e| CatalogError::new("configuration", e.to_string()))?;
        let bearing = bearing(id)?;
        let representation = bearing.representation();
        let insertion = Insertion::resolved(
            crate::PLUGIN,
            "make-bearing",
            json!({"size":bearing.designation()}),
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
    fn every_product_round_trips_without_freeform_dimensions() {
        for b in bearings::ALL {
            let resolved = Bearings.resolve(b.sku(), &json!({})).unwrap();
            assert_eq!(crate::parse(&resolved.insertion.params).unwrap().bearing, b);
            assert_eq!(
                resolved
                    .description
                    .provenance
                    .supplier_part_number
                    .as_deref(),
                Some(b.sku())
            );
            assert!(Bearings.resolve(b.sku(), &json!({"width":4.2})).is_err());
            let mut modified = resolved.insertion.params;
            modified["size"] = json!(if b == Bearing::OpenBuilds6252rs {
                "105ZZ"
            } else {
                "625 2RS"
            });
            assert!(crate::parse(&modified).is_err());
        }
    }
}
