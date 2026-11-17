//! Explicit purchasing assignments, independent of geometry and listing metadata.
use crate::{CanonicalIdentity, CatalogError, Product, Representation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Identifier {
    pub authority: String,
    pub part_number: String,
}

impl Identifier {
    fn validate(&self) -> Result<(), CatalogError> {
        crate::identity::key("authority", &self.authority)?;
        crate::identity::key("part_number", &self.part_number)
    }
}

/// A supplier SKU identifies an offer. Manufacturer information, when known,
/// must identify a product too; a company name alone is not a product identity.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Purchase {
    pub supplier: Identifier,
    pub manufacturer: Option<Identifier>,
}

impl Purchase {
    pub fn identity(&self) -> Result<CanonicalIdentity, CatalogError> {
        self.supplier.validate()?;
        if let Some(manufacturer) = &self.manufacturer {
            manufacturer.validate()?;
        }
        crate::identity::canonical(self)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub part_number: Option<String>,
    pub purchase: Option<Purchase>,
}

/// URL is editable metadata and never participates in product or reuse identity.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Details {
    pub assignment: Assignment,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Configuration<C> {
    Assigned { design: C, assignment: Assignment },
    Unassigned(C),
}

impl Assignment {
    /// Preserve the exact existing identity when no assignment is provided.
    pub fn representation<C>(&self, design: Representation<C>) -> Representation<Configuration<C>> {
        let configuration = if *self == Self::default() {
            Configuration::Unassigned(design.product.configuration)
        } else {
            Configuration::Assigned {
                design: design.product.configuration,
                assignment: self.clone(),
            }
        };
        Representation {
            product: Product {
                catalog: design.product.catalog,
                part: design.product.part,
                kind: design.product.kind,
                configuration,
                members: design.product.members,
            },
            revision: design.revision,
            detail: design.detail,
            dependencies: design.dependencies,
        }
    }
}

impl Details {
    pub fn validate(&self) -> Result<(), CatalogError> {
        if let Some(number) = &self.assignment.part_number {
            crate::identity::key("part_number", number)?;
        }
        if let Some(purchase) = &self.assignment.purchase {
            purchase.identity()?;
        }
        if let Some(url) = &self.url {
            let authority = url
                .strip_prefix("https://")
                .or_else(|| url.strip_prefix("http://"))
                .and_then(|tail| tail.split(['/', '?', '#']).next());
            if self.assignment.purchase.is_none()
                || url.len() > 2048
                || url.chars().any(char::is_whitespace)
                || url.chars().any(char::is_control)
                || authority.is_none_or(|host| host.is_empty() || host.contains('@'))
            {
                return Err(CatalogError::new("purchase_url", "requires a supplier/SKU assignment and an HTTP(S) URL with a host and no credentials or whitespace"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn design() -> Representation<serde_json::Value> {
        Representation {
            product: Product {
                catalog: "example".into(),
                part: "design".into(),
                kind: crate::ProductKind::DesignSpecification,
                configuration: json!({"bore":12}),
                members: vec![],
            },
            revision: "1".into(),
            detail: "mounting".into(),
            dependencies: vec![],
        }
    }
    #[test]
    fn assignments_preserve_unassigned_identities_and_exclude_urls() {
        assert_eq!(
            design().identity(),
            Assignment::default().representation(design()).identity()
        );
        let mut details = Details {
            assignment: Assignment {
                part_number: None,
                purchase: Some(Purchase {
                    supplier: Identifier {
                        authority: "Seller".into(),
                        part_number: "123".into(),
                    },
                    manufacturer: None,
                }),
            },
            url: Some("https://example.com/123".into()),
        };
        let original = details.assignment.representation(design()).identity();
        details.url = Some("https://example.com/123?ref=other".into());
        assert_eq!(
            original,
            details.assignment.representation(design()).identity()
        );
        details
            .assignment
            .purchase
            .as_mut()
            .unwrap()
            .supplier
            .part_number = "456".into();
        assert_ne!(
            original,
            details.assignment.representation(design()).identity()
        );
    }
    #[test]
    fn reject_incomplete_identifiers_and_unassigned_urls() {
        let invalid = Purchase {
            supplier: Identifier {
                authority: "Seller".into(),
                part_number: "".into(),
            },
            manufacturer: None,
        };
        assert!(invalid.identity().is_err());
        assert!(Details {
            url: Some("https://example.com".into()),
            ..Details::default()
        }
        .validate()
        .is_err());
    }
}
