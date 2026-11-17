//! Fixed products and the same core recipe used to build their assemblies.

use parts::catalog::{
    CatalogInfo, Dimension, Frame, Insertion, PartDescription, PartSummary, Provenance,
    ResolvedPart, Route, Source,
};
use parts::{Catalog, CatalogError, ProductKind};
use serde::Deserialize;
use serde_json::{json, Value};
use wheels::Wheel;

pub struct Wheels;
pub const REVISION: &str = "1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    #[serde(default = "assembled")]
    assembled: bool,
}
fn assembled() -> bool {
    true
}

fn wheel(id: &str) -> Result<Wheel, CatalogError> {
    wheels::ALL
        .into_iter()
        .find(|w| w.id() == id)
        .ok_or_else(|| CatalogError::new("part", format!("unknown wheel {id}")))
}
fn summary(wheel: Wheel) -> PartSummary {
    PartSummary {
        id: wheel.id().into(),
        name: wheel.name().into(),
        family: "v-wheels".into(),
        kind: ProductKind::SupplierProduct,
        designation: wheel.name().into(),
    }
}

impl Catalog for Wheels {
    type Configuration = Wheel;
    fn info(&self) -> CatalogInfo {
        CatalogInfo {
            id: crate::PLUGIN.into(),
            name: "V-wheels".into(),
            revision: REVISION.into(),
        }
    }
    fn summaries(&self) -> Vec<PartSummary> {
        wheels::ALL.into_iter().map(summary).collect()
    }
    fn describe(&self, id: &str) -> Result<PartDescription, CatalogError> {
        let wheel = wheel(id)?;
        let (diameter, width) = wheel.dimensions();
        Ok(PartDescription {
            part: summary(wheel),
            dimensions: vec![Dimension::mm("outside_diameter", diameter), Dimension::mm("wheel_width", width),
                Dimension::mm("bearing_seat_diameter", wheel.bearing().dimensions().diameter)],
            sources: vec![Source { title: "Supplied OpenBuilds STEP reference".into(),
                url: format!("https://grabcad.com/library/{}-1", wheel.id()) },
                Source { title: "OpenBuilds product list".into(),
                    url: "https://builds.openbuilds.com/builds/openbuilds-sphinx-1050-20-x-40.7196/".into() }],
            provenance: Provenance { supplier: Some("OpenBuilds".into()),
                supplier_part_number: wheel.sku().map(str::to_string), ..Provenance::default() },
            configuration_schema: json!({"type":"object","additionalProperties":false,"properties":{
                "assembled":{"type":"boolean","default":true}}}),
            details: vec!["exterior".into()],
            attachments: vec![Frame::axial("axle-centre", 0.0), Frame::axial("negative-bearing-centre", -wheel.bearing_offset()),
                Frame::axial("positive-bearing-centre", wheel.bearing_offset())],
            insertion: Route { plugin: crate::PLUGIN.into(), entry: "make-v-wheel".into() },
            notes: vec!["Nominal STEP profile, not a press-fit simulation. Full-size Solid CAD width is 10.20 mm; the 10.23 mm retail listing is a documented discrepancy.".into(),
                "The optional core is an assembly of separate purchases: one wheel, two bearings and one internal shim. Axle, external spacers and mounting nut are not included.".into()],
        })
    }
    fn resolve(&self, id: &str, value: &Value) -> Result<ResolvedPart<Wheel>, CatalogError> {
        let configuration: Configuration = serde_json::from_value(value.clone())
            .map_err(|e| CatalogError::new("configuration", e.to_string()))?;
        let wheel = wheel(id)?;
        let representation = wheel
            .representation(configuration.assembled)
            .map_err(|e| CatalogError::new("part", e))?;
        let mut description = self.describe(id)?;
        let members = if configuration.assembled {
            description.part.kind = ProductKind::Assembly;
            description.part.name.push_str(" assembly");
            description.provenance = Provenance::default();
            description.dimensions.push(Dimension::mm(
                "assembled_width",
                2.0 * wheel.bearing().dimensions().width + wheel.shim().thickness(),
            ));
            wheel.members().map_err(|e| CatalogError::new("part", e))?
        } else {
            vec![]
        };
        let insertion = Insertion::resolved(
            crate::PLUGIN,
            "make-v-wheel",
            json!({"size":wheel.name(),"assembled":configuration.assembled}),
            REVISION,
            &representation,
        )?;
        Ok(ResolvedPart {
            representation,
            description,
            members,
            insertion,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_product_and_core_round_trips_through_the_same_factory() {
        for wheel in wheels::ALL {
            for assembled in [false, true] {
                let resolved = Wheels
                    .resolve(wheel.id(), &json!({"assembled":assembled}))
                    .unwrap();
                let p = crate::parse(&resolved.insertion.params).unwrap();
                assert_eq!((p.wheel, p.assembled), (wheel, assembled));
                assert_eq!(
                    resolved
                        .members
                        .iter()
                        .map(|m| m.product.clone())
                        .collect::<Vec<_>>(),
                    resolved.representation.product.members
                );
                assert_eq!(
                    resolved
                        .members
                        .iter()
                        .map(|m| m.representation.clone())
                        .collect::<Vec<_>>(),
                    resolved.representation.dependencies
                );
                let mut changed = resolved.insertion.params;
                changed["assembled"] = json!(!assembled);
                assert!(crate::parse(&changed).is_err());
            }
        }
    }
    #[test]
    fn no_unverified_skus_or_unpublished_products() {
        assert_eq!(
            Wheels
                .describe("solid-v-wheel")
                .unwrap()
                .provenance
                .supplier_part_number,
            None
        );
        assert_eq!(
            Wheels
                .describe("xtreme-solid-v-wheel")
                .unwrap()
                .provenance
                .supplier_part_number
                .as_deref(),
            Some("55")
        );
        assert!(Wheels.resolve("metal-v-wheel", &json!({})).is_err());
        assert!(Wheels
            .resolve("solid-v-wheel", &json!({"diameter":24}))
            .is_err());
    }
}
