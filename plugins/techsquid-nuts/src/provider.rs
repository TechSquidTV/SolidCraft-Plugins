//! The ISO table remains the single owner of sizes and dimensions.

use crate::nut;
use fasteners::{metric, nut as iso};
use parts::detail::ThreadDetail;
use parts::{Product, ProductKind, Representation};
use serde::{Deserialize, Serialize};
pub const REVISION: &str = "1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Specification {
    major_mm: f64,
    pitch_mm: f64,
    across_flats_mm: f64,
    height_mm: f64,
    countersink_mm: Option<f64>,
    chamfer_max_mm: Option<f64>,
}

pub fn representation(p: &nut::Params) -> Representation<Specification> {
    Representation {
        product: Product {
            catalog: if p.custom { "techsquid-custom" } else { "iso" }.into(),
            part: if p.custom { "hex-nut" } else { "4032" }.into(),
            kind: if p.custom {
                ProductKind::Custom
            } else {
                ProductKind::StandardConfiguration
            },
            configuration: Specification {
                major_mm: p.major,
                pitch_mm: p.pitch,
                across_flats_mm: p.across_flats,
                height_mm: p.height,
                countersink_mm: p.countersink,
                chamfer_max_mm: p.chamfer_max,
            },
            members: vec![],
        },
        revision: "1".into(),
        detail: ThreadDetail::label(p.modeled).into(),
        dependencies: vec![],
    }
}

pub fn part_number(p: &nut::Params) -> Result<String, String> {
    if p.custom {
        plugin_kit::component::model_number(&representation(p).product)
    } else {
        Ok(format!("ISO 4032 M{}", p.major))
    }
}

pub fn labels() -> Vec<String> {
    iso::sizes()
        .map(|s| format!("M{}", metric::number(s.nominal)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nut;
    use fasteners::{metric, nut as iso};
    use parts::catalog::{
        CatalogInfo, Dimension, Frame, Insertion, PartDescription, PartSummary, Provenance,
        ResolvedPart, Route, Source,
    };
    use parts::detail::ThreadDetail;
    use parts::{Catalog, CatalogError, ProductKind};
    use serde::Deserialize;
    use serde_json::{json, Value};

    pub struct Nuts;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Configuration {
        #[serde(default)]
        detail: ThreadDetail,
    }
    fn size(id: &str) -> Result<iso::NutSize, CatalogError> {
        iso::sizes()
            .find(|s| id == format!("m{}", metric::number(s.nominal)))
            .ok_or_else(|| CatalogError::new("part", format!("unknown ISO 4032 size {id}")))
    }
    fn summary(size: iso::NutSize) -> PartSummary {
        let designation = format!("M{}", metric::number(size.nominal));
        PartSummary {
            id: designation.to_lowercase(),
            name: format!("ISO 4032 nut {designation}"),
            family: "hexagon-nuts".into(),
            kind: ProductKind::StandardConfiguration,
            designation,
        }
    }
    impl Catalog for Nuts {
        type Configuration = Specification;
        fn info(&self) -> CatalogInfo {
            CatalogInfo {
                id: crate::PLUGIN.into(),
                name: "Hexagon nuts".into(),
                revision: REVISION.into(),
            }
        }
        fn summaries(&self) -> Vec<PartSummary> {
            iso::sizes().map(summary).collect()
        }
        fn describe(&self, id: &str) -> Result<PartDescription, CatalogError> {
            let s = size(id)?;
            Ok(PartDescription {
                part: summary(s), dimensions: vec![Dimension::mm("major_diameter",s.nominal), Dimension::mm("pitch",s.pitch),
                    Dimension::mm("across_flats",s.across_flats), Dimension::mm("height",s.height)],
                sources: vec![Source { title:"ISO 4032 dimensions".into(), url:"https://www.fasteners.eu/standards/ISO/4032/".into() }],
                provenance: Provenance { standards:vec!["ISO 4032".into()], ..Provenance::default() },
                configuration_schema:json!({"type":"object","additionalProperties":false,"properties":{
                    "detail":{"type":"string","enum":ThreadDetail::CHOICES,"default":"cosmetic"}}}),
                details:ThreadDetail::CHOICES.into_iter().map(str::to_string).collect(),
                attachments:vec![Frame::axial("bearing-seat",0.0),Frame::axial("top",s.height)],
                insertion:Route { plugin:crate::PLUGIN.into(),entry:"make-nut".into() },
                notes:vec!["Standard dimensional configuration; no supplier stock, grade or finish is asserted. Custom dimensions and unsupported pitches are custom models.".into()],
            })
        }
        fn resolve(
            &self,
            id: &str,
            value: &Value,
        ) -> Result<ResolvedPart<Specification>, CatalogError> {
            let c: Configuration = serde_json::from_value(value.clone())
                .map_err(|e| CatalogError::new("configuration", e.to_string()))?;
            let s = size(id)?;
            let params = json!({"size":summary(s).designation,"thread_modeled":c.detail.modeled(),"create":"Nut only"});
            let p = nut::parse(&params).map_err(|e| CatalogError::new(e.field, e.reason))?;
            let representation = representation(&p);
            let insertion =
                Insertion::resolved(crate::PLUGIN, "make-nut", params, REVISION, &representation)?;
            Ok(ResolvedPart {
                representation,
                description: self.describe(id)?,
                members: vec![],
                insertion,
            })
        }
    }
    #[test]
    fn all_standard_sizes_round_trip_and_detail_keeps_purchasing_identity() {
        for s in Nuts.summaries() {
            let a = Nuts.resolve(&s.id, &json!({})).unwrap();
            let b = Nuts.resolve(&s.id, &json!({"detail":"modeled"})).unwrap();
            assert_eq!(a.representation.product, b.representation.product);
            assert_ne!(
                a.representation.identity().unwrap(),
                b.representation.identity().unwrap()
            );
            assert_eq!(
                representation(&nut::parse(&a.insertion.params).unwrap()),
                a.representation
            );
        }
    }
    #[test]
    fn overrides_and_fine_pitch_do_not_claim_iso_4032() {
        for params in [
            json!({"custom_width":5.5}),
            json!({"custom_thickness":2.4}),
            json!({"size":"M8x1"}),
        ] {
            assert_eq!(
                representation(&nut::parse(&params).unwrap()).product.kind,
                ProductKind::Custom
            );
        }
        let mut params = Nuts.resolve("m3", &json!({})).unwrap().insertion.params;
        params["custom_width"] = json!(5.5);
        assert!(nut::parse(&params).is_err());
    }
}
