//! A design can exist and be counted before a purchasing choice has been made.
use serde::{Deserialize, Serialize};

/// User-supplied purchasing information. Reference drawings never populate this.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Purchasing {
    pub manufacturer: Option<String>,
    pub supplier: Option<String>,
    pub supplier_part_number: Option<String>,
    pub url: Option<String>,
}

/// Geometry and optional BOM assignment are separate records. Including both in
/// a component identity prevents different purchasing choices sharing BOM metadata.
/// Reference evidence belongs outside this identity; it is not a purchase choice.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Specification<G> {
    pub geometry: G,
    pub part_number: Option<String>,
    pub purchasing: Purchasing,
}

impl<G: Serialize> Specification<G> {
    /// Stable shape/specification identity, unaffected by source or BOM assignment.
    pub fn geometry_identity(&self) -> Result<crate::CanonicalIdentity, crate::CatalogError> {
        crate::identity::canonical(&self.geometry)
    }
}

impl Purchasing {
    /// No purchasing identity exists until the user supplies purchasing information.
    pub fn identity(&self) -> Result<Option<crate::CanonicalIdentity>, crate::CatalogError> {
        if *self == Self::default() {
            Ok(None)
        } else {
            crate::identity::canonical(self).map(Some)
        }
    }
}
