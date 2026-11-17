//! Explicit, bounded queries. Catalog implementations need no host or selection.

use std::fmt;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

use crate::identity::{key, CanonicalIdentity, ProductKind, Representation};

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_PAGE_SIZE: usize = 64;
pub const MAX_QUERY_BYTES: usize = 256;
pub const MAX_REQUEST_BYTES: usize = 16 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogError {
    pub field: String,
    pub reason: String,
}

impl CatalogError {
    pub fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

impl std::error::Error for CatalogError {}

/// Each operation has its own fields; resolve cannot accidentally accept placement
/// or manufacturing options. Configuration is decoded into the provider's type.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    List {
        #[serde(default)]
        query: String,
        #[serde(default)]
        offset: usize,
        #[serde(default = "default_limit")]
        limit: usize,
    },
    Describe {
        part: String,
    },
    Resolve {
        part: String,
        #[serde(default = "empty_configuration")]
        configuration: Value,
    },
}

const fn default_limit() -> usize {
    32
}
fn empty_configuration() -> Value {
    json!({})
}

/// Version is mandatory; revision is mandatory for resolving a previously browsed
/// catalog. Discovery can omit revision to find the current value.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CatalogRequest {
    pub version: u16,
    #[serde(default)]
    pub revision: Option<String>,
    pub request: Query,
}

impl CatalogRequest {
    pub fn parse(value: &Value) -> Result<Self, CatalogError> {
        if serde_json::to_vec(value).map_or(true, |bytes| bytes.len() > MAX_REQUEST_BYTES) {
            return Err(CatalogError::new("catalog", "request exceeds 16 KiB"));
        }
        if value.get("revision").is_some_and(Value::is_null) {
            return Err(CatalogError::new(
                "revision",
                "omit it for discovery, or supply a revision string",
            ));
        }
        let request: Self = serde_json::from_value(value.clone())
            .map_err(|e| CatalogError::new("catalog", e.to_string()))?;
        if request.version != PROTOCOL_VERSION {
            return Err(CatalogError::new(
                "version",
                format!("supported catalog protocol is {PROTOCOL_VERSION}"),
            ));
        }
        if let Some(revision) = &request.revision {
            key("revision", revision)?;
        }
        match &request.request {
            Query::List {
                query,
                offset,
                limit,
            } => {
                if query.len() > MAX_QUERY_BYTES || query.chars().any(char::is_control) {
                    return Err(CatalogError::new(
                        "query",
                        "use at most 256 bytes without control characters",
                    ));
                }
                if !(1..=MAX_PAGE_SIZE).contains(limit) {
                    return Err(CatalogError::new("limit", "choose 1–64 items"));
                }
                if *offset > 1_000_000 {
                    return Err(CatalogError::new(
                        "offset",
                        "exceeds the catalog pagination limit",
                    ));
                }
            }
            Query::Describe { part } => key("part", part)?,
            Query::Resolve {
                part,
                configuration,
            } => {
                key("part", part)?;
                if request.revision.is_none() {
                    return Err(CatalogError::new(
                        "revision",
                        "resolve requires the catalog revision returned by list or describe",
                    ));
                }
                if !configuration.is_object() {
                    return Err(CatalogError::new("configuration", "requires an object"));
                }
            }
        }
        Ok(request)
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CatalogInfo {
    pub id: String,
    pub name: String,
    pub revision: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PartSummary {
    pub id: String,
    pub name: String,
    pub family: String,
    pub kind: ProductKind,
    pub designation: String,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Mm,
    Degrees,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Dimension {
    pub name: String,
    pub value: f64,
    pub unit: Unit,
}

impl Dimension {
    pub fn mm(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            unit: Unit::Mm,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Source {
    pub title: String,
    pub url: String,
}

/// Attribution is separate from an engineering designation. An unknown bearing
/// manufacturer must not be inferred from its supplier or its size code.
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Provenance {
    pub supplier: Option<String>,
    pub supplier_part_number: Option<String>,
    pub manufacturer: Option<String>,
    pub standards: Vec<String>,
}

/// A right-handed local frame in millimetres, independent of occurrence placement.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Frame {
    pub name: String,
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub z_axis: [f64; 3],
}

impl Frame {
    pub fn axial(name: impl Into<String>, z: f64) -> Self {
        Self {
            name: name.into(),
            origin: [0.0, 0.0, z],
            x_axis: [1.0, 0.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Route {
    pub plugin: String,
    pub entry: String,
}

/// One occurrence in a resolved recipe. Repeated identities encode quantity;
/// each occurrence retains its own named local frame.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Member {
    pub product: CanonicalIdentity,
    pub representation: CanonicalIdentity,
    pub frame: Frame,
}

impl Member {
    pub fn new<C: Serialize>(part: &Representation<C>, frame: Frame) -> Result<Self, CatalogError> {
        Ok(Self {
            product: part.product.identity()?,
            representation: part.identity()?,
            frame,
        })
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PartDescription {
    pub part: PartSummary,
    pub dimensions: Vec<Dimension>,
    pub sources: Vec<Source>,
    /// The provider's typed configuration schema, not the insertion workflow form.
    pub configuration_schema: Value,
    pub details: Vec<String>,
    pub provenance: Provenance,
    pub attachments: Vec<Frame>,
    pub insertion: Route,
    pub notes: Vec<String>,
}

/// An explicit route to an existing insertion workflow. It is data, not authority
/// to execute it. The workflow must resolve/validate these inputs again.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Insertion {
    pub plugin: String,
    pub entry: String,
    pub params: Value,
}

/// An API-only precondition returned by resolution. The insertion workflow still
/// parses its own inputs and compares their resolved representation with this one.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Selection<C> {
    pub version: u16,
    pub catalog_revision: String,
    pub representation: Representation<C>,
}

impl Insertion {
    pub fn resolved<C: Clone + Serialize>(
        plugin: &str,
        entry: &str,
        mut params: Value,
        catalog_revision: &str,
        representation: &Representation<C>,
    ) -> Result<Self, CatalogError> {
        representation.identity()?;
        key("catalog_revision", catalog_revision)?;
        let fields = params.as_object_mut().ok_or_else(|| {
            CatalogError::new("insertion", "workflow parameters must be an object")
        })?;
        if fields.contains_key("catalog_selection") {
            return Err(CatalogError::new(
                "insertion",
                "workflow must not supply its own catalog selection",
            ));
        }
        let selection = Selection {
            version: PROTOCOL_VERSION,
            catalog_revision: catalog_revision.into(),
            representation: representation.clone(),
        };
        fields.insert(
            "catalog_selection".into(),
            serde_json::to_value(selection)
                .map_err(|e| CatalogError::new("insertion", e.to_string()))?,
        );
        Ok(Self {
            plugin: plugin.into(),
            entry: entry.into(),
            params,
        })
    }
}

/// Ordinary dialog runs resolve against the current catalog. Runs made from a
/// catalog response additionally pin its revision and the selected representation.
pub fn validate_selection<C: Serialize + DeserializeOwned>(
    params: &Value,
    revision: &str,
    representation: &Representation<C>,
) -> Result<(), CatalogError> {
    let Some(value) = params.get("catalog_selection") else {
        return Ok(());
    };
    let selected: Selection<C> = serde_json::from_value(value.clone())
        .map_err(|e| CatalogError::new("catalog_selection", e.to_string()))?;
    if selected.version != PROTOCOL_VERSION || selected.catalog_revision != revision {
        return Err(CatalogError::new(
            "catalog_selection",
            "catalog changed; resolve the selection again",
        ));
    }
    if selected.representation.identity()? != representation.identity()? {
        return Err(CatalogError::new(
            "catalog_selection",
            "workflow inputs no longer match the resolved part",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ResolvedPart<C> {
    pub representation: Representation<C>,
    /// The fully configured description, not necessarily the family's defaults.
    pub description: PartDescription,
    pub members: Vec<Member>,
    pub insertion: Insertion,
}

impl<C: Serialize> ResolvedPart<C> {
    fn validate(&self) -> Result<(), CatalogError> {
        self.representation.identity()?;
        let products: Vec<_> = self.members.iter().map(|m| m.product.clone()).collect();
        let representations: Vec<_> = self
            .members
            .iter()
            .map(|m| m.representation.clone())
            .collect();
        if products != self.representation.product.members
            || representations != self.representation.dependencies
        {
            return Err(CatalogError::new(
                "catalog",
                "resolved members disagree with the part identity",
            ));
        }
        if self.description.insertion.plugin != self.insertion.plugin
            || self.description.insertion.entry != self.insertion.entry
        {
            return Err(CatalogError::new(
                "catalog",
                "resolved insertion route disagrees with its description",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum CatalogResult<C> {
    List {
        items: Vec<PartSummary>,
        total: usize,
        next_offset: Option<usize>,
    },
    Describe {
        part: Box<PartDescription>,
    },
    Resolve {
        part: Box<ResolvedPart<C>>,
    },
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CatalogResponse<C> {
    pub version: u16,
    pub catalog: CatalogInfo,
    pub result: CatalogResult<C>,
}

impl<C: Serialize> CatalogResponse<C> {
    pub fn json(&self) -> Result<Value, CatalogError> {
        let bytes =
            serde_json::to_vec(self).map_err(|e| CatalogError::new("catalog", e.to_string()))?;
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(CatalogError::new(
                "catalog",
                "response exceeds 512 KiB; request fewer items",
            ));
        }
        serde_json::from_slice(&bytes).map_err(|e| CatalogError::new("catalog", e.to_string()))
    }
}

pub trait Catalog {
    type Configuration: Serialize;

    fn info(&self) -> CatalogInfo;
    fn summaries(&self) -> Vec<PartSummary>;
    fn describe(&self, part: &str) -> Result<PartDescription, CatalogError>;
    fn resolve(
        &self,
        part: &str,
        configuration: &Value,
    ) -> Result<ResolvedPart<Self::Configuration>, CatalogError>;

    fn query(&self, value: &Value) -> Result<CatalogResponse<Self::Configuration>, CatalogError> {
        let request = CatalogRequest::parse(value)?;
        let catalog = self.info();
        key("catalog.id", &catalog.id)?;
        key("catalog.revision", &catalog.revision)?;
        if request
            .revision
            .as_ref()
            .is_some_and(|r| r != &catalog.revision)
        {
            return Err(CatalogError::new(
                "revision",
                "catalog changed; list or describe it again",
            ));
        }
        let result = match request.request {
            Query::List {
                query,
                offset,
                limit,
            } => {
                let query = query.trim().to_lowercase();
                let mut items = self.summaries();
                items.sort_by(|a, b| a.id.cmp(&b.id));
                for item in &items {
                    key("part", &item.id)?;
                }
                if items.windows(2).any(|pair| pair[0].id == pair[1].id) {
                    return Err(CatalogError::new("catalog", "duplicate product IDs"));
                }
                items.retain(|p| {
                    [&p.id, &p.name, &p.family, &p.designation]
                        .iter()
                        .any(|text| text.to_lowercase().contains(&query))
                });
                let total = items.len();
                let end = offset.saturating_add(limit).min(total);
                CatalogResult::List {
                    items: items.into_iter().skip(offset).take(limit).collect(),
                    total,
                    next_offset: (end < total).then_some(end),
                }
            }
            Query::Describe { part } => CatalogResult::Describe {
                part: Box::new(self.describe(&part)?),
            },
            Query::Resolve {
                part,
                configuration,
            } => {
                let resolved = self.resolve(&part, &configuration)?;
                resolved.validate()?;
                CatalogResult::Resolve {
                    part: Box::new(resolved),
                }
            }
        };
        Ok(CatalogResponse {
            version: PROTOCOL_VERSION,
            catalog,
            result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_versions_fields_and_unbounded_queries() {
        for value in [
            json!({"version": 2, "request": {"operation": "list"}}),
            json!({"version": 1, "revision": null, "request": {"operation": "list"}}),
            json!({"version": 1, "request": {"operation": "list", "limit": 0}}),
            json!({"version": 1, "request": {"operation": "list", "limit": 65}}),
            json!({"version": 1, "request": {"operation": "list", "query": "x".repeat(257)}}),
            json!({"version": 1, "request": {"operation": "describe", "part": "x", "point": [0,0,0]}}),
            json!({"version": 1, "request": {"operation": "resolve", "part": "x"}}),
            json!({"version": 1, "revision": "1", "request": {"operation": "resolve", "part": "x", "configuration": []}}),
            json!({"version": 1, "request": {"operation": "list"}, "target": "Plate"}),
        ] {
            assert!(CatalogRequest::parse(&value).is_err(), "{value}");
        }
    }

    #[test]
    fn accepts_discovery_without_a_revision_but_pins_resolution() {
        assert!(
            CatalogRequest::parse(&json!({"version": 1, "request": {"operation": "list"}})).is_ok()
        );
        assert!(CatalogRequest::parse(&json!({"version": 1, "revision": "1", "request": {"operation": "resolve", "part": "x"}})).is_ok());
    }
}
