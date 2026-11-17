//! Physical identity is independent of the generating plugin and its placements.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::catalog::CatalogError;

/// What is actually known about the item, rather than an assertion of stock status.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductKind {
    SupplierProduct,
    DesignSpecification,
    StandardConfiguration,
    Custom,
    Assembly,
}

/// A provider's concrete configuration stays typed until it crosses the wire.
///
/// The catalog is the identity authority (for example a manufacturer's catalog),
/// not the plugin used to insert it. `part` is a stable product/family key, never a
/// translated display name. Placement and rendering detail do not belong here.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Product<C> {
    pub catalog: String,
    pub part: String,
    pub kind: ProductKind,
    pub configuration: C,
    /// Purchasing composition, separate from child geometry revisions. Empty for
    /// a single product; repeated identities express assembly quantities.
    pub members: Vec<CanonicalIdentity>,
}

/// One CAD representation of a product. The revision covers geometry and default
/// appearance changes. Separate detail modes can retain the same purchasing ID.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Representation<C> {
    pub product: Product<C>,
    pub revision: String,
    pub detail: String,
    /// Ordered resolved child representations. Repetition expresses quantity;
    /// the recipe revision fixes their local placement frames.
    pub dependencies: Vec<CanonicalIdentity>,
}

/// The canonical serialized identity used at the host boundary. Keep and compare
/// these bytes; a shortened hash or human-readable part name is not proof of equality.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct CanonicalIdentity(String);

impl CanonicalIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<C: Serialize> Product<C> {
    pub fn identity(&self) -> Result<CanonicalIdentity, CatalogError> {
        key("catalog", &self.catalog)?;
        key("part", &self.part)?;
        canonical(self)
    }
}

impl<C: Serialize> Representation<C> {
    pub fn identity(&self) -> Result<CanonicalIdentity, CatalogError> {
        self.product.identity()?;
        key("revision", &self.revision)?;
        key("detail", &self.detail)?;
        canonical(self)
    }
}

pub(crate) fn key(field: &str, value: &str) -> Result<(), CatalogError> {
    if value.is_empty()
        || value.len() > 128
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(CatalogError::new(
            field,
            "requires 1–128 bytes without surrounding whitespace or control characters",
        ));
    }
    Ok(())
}

pub(crate) fn canonical(value: &impl Serialize) -> Result<CanonicalIdentity, CatalogError> {
    let text = canonical_json(value).map_err(|e| CatalogError::new("identity", e.to_string()))?;
    if text.len() > 16 * 1024 {
        return Err(CatalogError::new("identity", "exceeds 16 KiB"));
    }
    Ok(CanonicalIdentity(text))
}

/// Deterministic JSON for identities and current-content snapshots. Object key
/// order is not significant; array order and concrete numeric types remain so.
pub fn canonical_json(value: &impl Serialize) -> Result<String, serde_json::Error> {
    // serde_json's default object map is ordered. Rebuild objects explicitly so
    // identity stays stable even if a downstream consumer enables preserve_order.
    fn ordered(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let sorted: std::collections::BTreeMap<_, _> = map.into_iter().collect();
                Value::Object(sorted.into_iter().map(|(k, v)| (k, ordered(v))).collect())
            }
            Value::Array(values) => Value::Array(values.into_iter().map(ordered).collect()),
            other => other,
        }
    }
    serde_json::to_string(&ordered(serde_json::to_value(value)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Serialize)]
    struct Screw {
        diameter_mm: u16,
        length_mm: u16,
        material: &'static str,
    }

    fn screw() -> Product<Screw> {
        Product {
            catalog: "iso".into(),
            part: "4762".into(),
            kind: ProductKind::StandardConfiguration,
            members: vec![],
            configuration: Screw {
                diameter_mm: 3,
                length_mm: 12,
                material: "steel",
            },
        }
    }

    #[test]
    fn thread_detail_changes_geometry_but_not_the_purchased_screw() {
        let cosmetic = Representation {
            product: screw(),
            revision: "1".into(),
            detail: "cosmetic-thread".into(),
            dependencies: vec![],
        };
        let modeled = Representation {
            detail: "modeled-thread".into(),
            ..cosmetic.clone()
        };
        assert_eq!(cosmetic.product.identity(), modeled.product.identity());
        assert_ne!(cosmetic.identity(), modeled.identity());
    }

    #[test]
    fn material_and_source_authority_are_part_of_purchasing_identity() {
        let original = screw();
        let mut other = original.clone();
        other.configuration.material = "stainless-steel";
        assert_ne!(original.identity(), other.identity());
        other = original.clone();
        other.catalog = "supplier-catalog".into();
        assert_ne!(original.identity(), other.identity());
        other = original.clone();
        other.kind = ProductKind::Custom;
        assert_ne!(original.identity(), other.identity());
    }

    #[test]
    fn map_order_does_not_change_identity() {
        use serde_json::json;
        let a = Product {
            catalog: "test".into(),
            part: "part".into(),
            kind: ProductKind::Custom,
            members: vec![],
            configuration: json!({"b": 2, "a": {"z": 3, "y": 4}}),
        };
        let b = Product {
            configuration: json!({"a": {"y": 4, "z": 3}, "b": 2}),
            ..a.clone()
        };
        assert_eq!(a.identity(), b.identity());
    }

    #[test]
    fn invalid_authority_cannot_become_a_reuse_key() {
        let mut part = screw();
        part.catalog = " ".into();
        assert!(part.identity().is_err());
        part.catalog = "catalog\nname".into();
        assert!(part.identity().is_err());
    }
}
