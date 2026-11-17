//! Fixed-length stock profiles and sourced slot nuts; paths remain model features.

use crate::{rail, tnut};
use extrusions::{
    machining::Face,
    profile::SIZES,
    slot::{Slot, T5, V5},
    tnut::{TNut, NUTS, THREADS},
};
use parts::catalog::{
    CatalogInfo, Dimension, Frame, Insertion, PartDescription, PartSummary, Provenance,
    ResolvedPart, Route,
};
use parts::detail::ThreadDetail;
use parts::{Catalog, CatalogError, Product, ProductKind, Representation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub struct Extrusions;
pub const REVISION: &str = "1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Specification {
    Rail {
        length_mm: f64,
        end_taps: rail::Taps,
        holes: Vec<(Face, f64)>,
    },
    SlotNut {
        thread: String,
    },
}

pub fn rail_representation(p: &rail::Params) -> Representation<Specification> {
    let (catalog, part) = if p.slot == T5 {
        ("misumi-profile", format!("nfs5-{}", p.size))
    } else {
        ("openbuilds-profile", format!("v-slot-{}", p.size))
    };
    Representation {
        product: Product {
            catalog: catalog.into(),
            part,
            kind: ProductKind::Custom,
            configuration: Specification::Rail {
                length_mm: p.length,
                end_taps: p.taps,
                holes: p.holes.clone(),
            },
            members: vec![],
        },
        revision: "1".into(),
        detail: if p.taps == rail::Taps::None {
            "plain"
        } else {
            ThreadDetail::label(p.modeled)
        }
        .into(),
        dependencies: vec![],
    }
}
pub fn nut_representation(p: &tnut::Params) -> Representation<Specification> {
    Representation {
        product: Product {
            catalog: "misumi".into(),
            part: tnut::part(p),
            kind: ProductKind::SupplierProduct,
            configuration: Specification::SlotNut {
                thread: p.thread.clone(),
            },
            members: vec![],
        },
        revision: "1".into(),
        detail: ThreadDetail::label(p.modeled).into(),
        dependencies: vec![],
    }
}

#[derive(Clone, Copy)]
enum Item {
    Rail(Slot, &'static str),
    Nut(TNut, &'static str),
}
impl Item {
    fn id(self) -> String {
        match self {
            Self::Rail(slot, size) => {
                format!("{}-{size}", if slot == T5 { "t-slot" } else { "v-slot" })
            }
            Self::Nut(nut, thread) => format!(
                "{}-{}",
                nut.part.to_lowercase(),
                thread.trim_start_matches('M')
            ),
        }
    }
    fn summary(self) -> PartSummary {
        let (name, family, kind) = match self {
            Self::Rail(slot, size) => (
                format!("{} {size}", slot.name),
                "extrusions",
                ProductKind::Custom,
            ),
            Self::Nut(nut, thread) => (
                format!("MISUMI {}-{}", nut.part, thread.trim_start_matches('M')),
                "slot-nuts",
                ProductKind::SupplierProduct,
            ),
        };
        PartSummary {
            id: self.id(),
            designation: name.clone(),
            name,
            family: family.into(),
            kind,
        }
    }
}
fn items() -> Vec<Item> {
    let mut items: Vec<_> = [T5, V5]
        .into_iter()
        .flat_map(|slot| {
            SIZES
                .into_iter()
                .filter(move |(_, grid)| slot == T5 || grid.nx == 1)
                .map(move |(size, _)| Item::Rail(slot, size))
        })
        .collect();
    items.extend(NUTS.into_iter().flat_map(|nut| {
        THREADS
            .into_iter()
            .map(move |thread| Item::Nut(nut, thread))
    }));
    items
}
fn item(id: &str) -> Result<Item, CatalogError> {
    items()
        .into_iter()
        .find(|i| i.id() == id)
        .ok_or_else(|| CatalogError::new("part", format!("unknown rail or slot nut {id}")))
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RailConfiguration {
    length: f64,
    end_taps: rail::Taps,
    holes: String,
    detail: ThreadDetail,
}
impl Default for RailConfiguration {
    fn default() -> Self {
        Self {
            length: rail::DEFAULT_LENGTH,
            end_taps: rail::Taps::None,
            holes: String::new(),
            detail: ThreadDetail::Cosmetic,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NutConfiguration {
    #[serde(default)]
    detail: ThreadDetail,
}

fn describe(item: Item) -> PartDescription {
    let (dimensions,provenance,configuration_schema,attachments,entry,notes)=match item {
        Item::Rail(slot,size)=>{
            let grid=extrusions::Grid::of(size).expect("catalog grid");
            (vec![Dimension::mm("width",grid.width()),Dimension::mm("height",grid.height())],
             Provenance {supplier:Some(if slot==T5 {"MISUMI"} else {"OpenBuilds"}.into()),..Provenance::default()},
             json!({"type":"object","additionalProperties":false,"properties":{
                "length":{"type":"number","exclusiveMinimum":0,"maximum":rail::MAX_LENGTH_MM,"default":rail::DEFAULT_LENGTH,"unit":"mm"},
                "end_taps":{"type":"string","enum":["None","Start","End","Both"],"default":"None"},
                "holes":{"type":"string","default":"","description":"Face names and distances in mm, e.g. +Y 10 290. Validated against the resolved length."},
                "detail":{"type":"string","enum":ThreadDetail::CHOICES,"default":"cosmetic"}}}),
             vec![Frame::axial("start",0.0)],"make-extrusion",
             vec!["Cut-to-length or machined configuration retaining the stock profile source. It is not an asserted finished supplier SKU, material grade or finish.".into(),
                "T-slot: MISUMI 2019 US catalog pp. 2686–2687 and NFS5 STEP sections. V-slot: OpenBuilds 20x20 drawing and larger source sections. See packaged source/test table.".into()])
        },
        Item::Nut(nut,thread)=>(
            vec![Dimension::mm("length",nut.length),Dimension::mm("width",nut.width()),Dimension::mm("height",nut.height())],
            Provenance {supplier:Some("MISUMI".into()),supplier_part_number:Some(format!("{}-{}",nut.part,thread.trim_start_matches('M'))),..Provenance::default()},
            json!({"type":"object","additionalProperties":false,"properties":{
                "detail":{"type":"string","enum":ThreadDetail::CHOICES,"default":"cosmetic"}}}),
            vec![Frame::axial("shoulder-seat",0.0)],"make-t-nut",
            vec!["Profile and spring-ball dimensions measured from MISUMI HNTT5-5/HNTP5-5 STEP references. Only the verified M5 products are published.".into()]),
    };
    PartDescription {
        part: item.summary(),
        dimensions,
        provenance,
        configuration_schema,
        attachments,
        insertion: Route {
            plugin: crate::PLUGIN.into(),
            entry: entry.into(),
        },
        sources: vec![],
        details: ThreadDetail::CHOICES
            .into_iter()
            .map(str::to_string)
            .collect(),
        notes,
    }
}
impl Catalog for Extrusions {
    type Configuration = Specification;
    fn info(&self) -> CatalogInfo {
        CatalogInfo {
            id: crate::PLUGIN.into(),
            name: "Extrusions".into(),
            revision: REVISION.into(),
        }
    }
    fn summaries(&self) -> Vec<PartSummary> {
        items().into_iter().map(Item::summary).collect()
    }
    fn describe(&self, id: &str) -> Result<PartDescription, CatalogError> {
        Ok(describe(item(id)?))
    }
    fn resolve(
        &self,
        id: &str,
        value: &Value,
    ) -> Result<ResolvedPart<Specification>, CatalogError> {
        let item = item(id)?;
        let mut description = describe(item);
        let (params, representation) = match item {
            Item::Rail(slot, size) => {
                let c: RailConfiguration = serde_json::from_value(value.clone())
                    .map_err(|e| CatalogError::new("configuration", e.to_string()))?;
                let params = json!({"slot":slot.name,"size":size,"length":c.length,"end_taps":c.end_taps.label(),
                    "holes":c.holes,"thread_modeled":c.detail.modeled()});
                let p = rail::parse(&params).map_err(|e| CatalogError::new(e.field, e.reason))?;
                description
                    .dimensions
                    .push(Dimension::mm("length", p.length));
                description.attachments.push(Frame::axial("end", p.length));
                description.part.name = crate::body_base(&p);
                (params, rail_representation(&p))
            }
            Item::Nut(nut, thread) => {
                let c: NutConfiguration = serde_json::from_value(value.clone())
                    .map_err(|e| CatalogError::new("configuration", e.to_string()))?;
                let params = json!({"type":if nut==NUTS[0] {"Pre-assembly"} else {"Post-assembly"},"size":thread,"thread_modeled":c.detail.modeled()});
                let p = tnut::parse(&params).map_err(|e| CatalogError::new(e.field, e.reason))?;
                (params, nut_representation(&p))
            }
        };
        let insertion = Insertion::resolved(
            crate::PLUGIN,
            &description.insertion.entry,
            params,
            REVISION,
            &representation,
        )?;
        Ok(ResolvedPart {
            representation,
            description,
            members: vec![],
            insertion,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_profiles_and_verified_nuts_resolve() {
        assert_eq!(Extrusions.summaries().len(), 11);
        for item in Extrusions.summaries() {
            let part = Extrusions.resolve(&item.id, &json!({})).unwrap();
            let actual = if part.insertion.entry == "make-extrusion" {
                rail_representation(&rail::parse(&part.insertion.params).unwrap())
            } else {
                nut_representation(&tnut::parse(&part.insertion.params).unwrap())
            };
            assert_eq!(actual, part.representation);
        }
        assert!(Extrusions.describe("v-slot-4040").is_err());
        assert!(Extrusions.describe("hntt5-3").is_err());
    }
    #[test]
    fn machining_changes_the_part_but_slot_mounting_does_not() {
        let plain = Extrusions
            .resolve("v-slot-2020", &json!({"length":300}))
            .unwrap();
        let cut = Extrusions
            .resolve(
                "v-slot-2020",
                &json!({"length":300,"end_taps":"Both","holes":"10 290"}),
            )
            .unwrap();
        assert_ne!(plain.representation.product, cut.representation.product);
        let nut = Extrusions.resolve("hntt5-5", &json!({})).unwrap();
        let mut params = nut.insertion.params;
        params["slot"] = json!("V-slot");
        params["along"] = json!("Sketch Y");
        assert_eq!(
            nut_representation(&tnut::parse(&params).unwrap()),
            nut.representation
        );
        let mut params = plain.insertion.params;
        params["length"] = json!(400);
        assert!(rail::parse(&params).is_err());
    }
}
