//! Families with typed configuration, not a Cartesian enumeration of lengths.

use crate::screw::{Drive, Head, Params, Resolved};
use fasteners::{hexsocket, phillips, slot, torx};
use parts::detail::ThreadDetail;
use parts::{Product, ProductKind, Representation};
use serde::{Deserialize, Serialize};
pub const REVISION: &str = "1";
/// Dialog convenience list, not a stock table. Other metric sizes are resolved
/// against the existing head/drive standards and require explicit support there.
pub const COMMON_SIZES: [&str; 12] = [
    "M1.6", "M2", "M2.5", "M3", "M4", "M5", "M6", "M8", "M10", "M12", "M16", "M20",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Specification {
    nominal_length_mm: f64,
    shank_length_mm: f64,
    head: Head,
    drive: Drive,
    resolved: Resolved,
    thread_length_mm: Option<f64>,
    hex: Option<hexsocket::Socket>,
    torx: Option<torx::Recess>,
    slot: Option<slot::Slot>,
    phillips: Option<phillips::Tool>,
}

pub fn representation(p: &Params) -> Representation<Specification> {
    Representation {
        product: Product {
            catalog: if p.custom {
                "techsquid-custom"
            } else {
                "techsquid-standards"
            }
            .into(),
            part: p.head.id().into(),
            kind: if p.custom {
                ProductKind::Custom
            } else {
                ProductKind::StandardConfiguration
            },
            configuration: Specification {
                nominal_length_mm: p.nominal_length,
                shank_length_mm: p.length,
                head: p.head,
                drive: p.drive,
                resolved: p.resolved,
                thread_length_mm: p.thread_length,
                hex: p.hex,
                torx: p.recess,
                slot: p.slot,
                phillips: p.phillips,
            },
            members: vec![],
        },
        revision: "1".into(),
        detail: ThreadDetail::label(p.modeled).into(),
        dependencies: vec![],
    }
}

pub fn part_number(p: &Params) -> Result<String, String> {
    if p.custom {
        return plugin_kit::component::model_number(&representation(p).product);
    }
    Ok(format!(
        "{} M{}x{} x {}",
        p.standard.ok_or("standard screw has no designation")?,
        p.resolved.major.ok_or("screw has no major diameter")?,
        p.resolved.pitch.ok_or("screw has no pitch")?,
        p.nominal_length
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screw::{self, Drive, Head, Params, Tip};

    use parts::catalog::{
        CatalogInfo, Dimension, Frame, Insertion, PartDescription, PartSummary, Provenance,
        ResolvedPart, Route, Source,
    };
    use parts::detail::ThreadDetail;
    use parts::{Catalog, CatalogError, ProductKind};
    use serde::Deserialize;
    use serde_json::{json, Value};

    pub struct Screws;

    #[derive(Deserialize)]
    #[serde(default, deny_unknown_fields)]
    struct Configuration {
        size: String,
        length: f64,
        drive: Option<Drive>,
        tip: Tip,
        detail: ThreadDetail,
    }
    impl Default for Configuration {
        fn default() -> Self {
            Self {
                size: "M3".into(),
                length: 12.0,
                drive: None,
                tip: Tip::Flat,
                detail: ThreadDetail::Cosmetic,
            }
        }
    }

    fn head(id: &str) -> Result<Head, CatalogError> {
        Head::ALL
            .into_iter()
            .find(|h| h.id() == id)
            .ok_or_else(|| CatalogError::new("part", format!("unknown screw family {id}")))
    }
    fn params(head: Head, c: &Configuration) -> Value {
        json!({"size":c.size,"length":c.length,"head":head.label(),"drive":c.drive.map_or("Standard",Drive::label),
            "tip":c.tip.label(),"thread_modeled":c.detail.modeled(),"create":"Screw only"})
    }
    fn summary(head: Head) -> PartSummary {
        PartSummary {
            id: head.id().into(),
            name: head.label().into(),
            family: "screws".into(),
            kind: if head == Head::Rod {
                ProductKind::Custom
            } else {
                ProductKind::StandardConfiguration
            },
            designation: head.default_standard().unwrap_or(head.label()).into(),
        }
    }
    fn description(p: &Params) -> PartDescription {
        let mut part = summary(p.head);
        part.name = crate::body_base(p);
        part.designation = p.designation.clone();
        part.kind = representation(p).product.kind;
        PartDescription { part,
            dimensions:vec![Dimension::mm("length",p.nominal_length),Dimension::mm("major_diameter",p.resolved.major.expect("catalog metric size")),
                Dimension::mm("pitch",p.resolved.pitch.expect("catalog metric pitch"))],
            sources:match p.head {
                Head::Socket=>vec![Source {title:"DIN 912 / ISO 4762 head table".into(),url:"https://www.fasteners.eu/standards/DIN/912/".into()}],
                Head::Hex=>vec![Source {title:"ISO 4017 head table".into(),url:"https://www.fasteners.eu/standards/ISO/4017/".into()}],
                _=>vec![],
            },
            provenance:Provenance {standards:if p.custom {vec![]} else {p.standard.into_iter().map(str::to_string).collect()},..Provenance::default()},
            configuration_schema:json!({"type":"object","additionalProperties":false,"properties":{
                "size":{"type":"string","default":"M3","description":"Metric designation; supported combinations are validated during resolve."},
                "length":{"type":"number","exclusiveMinimum":0,"maximum":1000,"default":12,"unit":"mm"},
                "drive":{"type":"string","enum":["HexSocket","Phillips","Slotted","Torx","None"],"description":"Omit for the family's standard drive."},
                "tip":{"type":"string","enum":["Flat","Point","Cup"],"default":"Flat"},
                "detail":{"type":"string","enum":ThreadDetail::CHOICES,"default":"cosmetic"}}}),
            details:ThreadDetail::CHOICES.into_iter().map(str::to_string).collect(),
            attachments:vec![Frame::axial("under-head-seat",0.0),Frame::axial("tip",-p.length)],
            insertion:Route {plugin:crate::PLUGIN.into(),entry:"make-screw".into()},
            notes:vec!["Standards-based dimensional configuration, not a supplier stock claim. Arbitrary lengths, grades, finishes and commercial availability are not certified.".into(),
                "Head and drive dimensions come from the standards listed in the packaged guide. Typed geometry overrides are custom models.".into()],
        }
    }
    impl Catalog for Screws {
        type Configuration = Specification;
        fn info(&self) -> CatalogInfo {
            CatalogInfo {
                id: crate::PLUGIN.into(),
                name: "Screws".into(),
                revision: REVISION.into(),
            }
        }
        fn summaries(&self) -> Vec<PartSummary> {
            Head::ALL.into_iter().map(summary).collect()
        }
        fn describe(&self, id: &str) -> Result<PartDescription, CatalogError> {
            let p = screw::parse(&params(head(id)?, &Configuration::default()))
                .map_err(|e| CatalogError::new(e.field, e.reason))?;
            Ok(description(&p))
        }
        fn resolve(
            &self,
            id: &str,
            value: &Value,
        ) -> Result<ResolvedPart<Specification>, CatalogError> {
            let c: Configuration = serde_json::from_value(value.clone())
                .map_err(|e| CatalogError::new("configuration", e.to_string()))?;
            let params = params(head(id)?, &c);
            let p = screw::parse(&params).map_err(|e| CatalogError::new(e.field, e.reason))?;
            if p.resolved.major.is_none() || p.resolved.pitch.is_none() {
                return Err(CatalogError::new(
                    "size",
                    "catalog screws require a supported metric designation",
                ));
            }
            let representation = representation(&p);
            let insertion = Insertion::resolved(
                crate::PLUGIN,
                "make-screw",
                params,
                REVISION,
                &representation,
            )?;
            Ok(ResolvedPart {
                representation,
                description: description(&p),
                members: vec![],
                insertion,
            })
        }
    }
    #[test]
    fn all_families_resolve_and_keep_mounting_out_of_identity() {
        for family in Screws.summaries() {
            let resolved = Screws.resolve(&family.id, &json!({})).unwrap();
            assert_eq!(
                representation(&screw::parse(&resolved.insertion.params).unwrap()),
                resolved.representation
            );
            let modeled = Screws
                .resolve(&family.id, &json!({"detail":"modeled"}))
                .unwrap();
            assert_eq!(
                resolved.representation.product,
                modeled.representation.product
            );
            let mut params = resolved.insertion.params;
            params["create"] = json!("Screw and hole");
            params["hole_type"] = json!("Clearance");
            assert_eq!(
                representation(&screw::parse(&params).unwrap()),
                resolved.representation
            );
            params["length"] = json!(20);
            assert!(screw::parse(&params).is_err());
        }
    }
    #[test]
    fn overrides_are_custom_even_when_their_numbers_match() {
        for params in [
            json!({"head_diameter":5.5}),
            json!({"drive_size":"2.5"}),
            json!({"thread_length":12}),
        ] {
            assert_eq!(
                representation(&screw::parse(&params).unwrap()).product.kind,
                ProductKind::Custom
            );
        }
        assert!(Screws.resolve("socket", &json!({"size":"M999"})).is_err());
        assert!(Screws
            .resolve("socket", &json!({"length":12,"head_diameter":5.5}))
            .is_err());
    }

    #[test]
    fn standard_designations_are_searchable_without_enumerating_lengths() {
        let found = Screws
            .query(&json!({"version":1,"request":{"operation":"list","query":"4762"}}))
            .unwrap()
            .json()
            .unwrap();
        assert_eq!(found["result"]["items"][0]["id"], "socket");
        for params in [
            json!({"size":"M8x1"}),
            json!({"tip":"Point"}),
            json!({"tip":"Cup"}),
        ] {
            assert_eq!(
                representation(&screw::parse(&params).unwrap()).product.kind,
                ProductKind::Custom
            );
        }
    }
}
