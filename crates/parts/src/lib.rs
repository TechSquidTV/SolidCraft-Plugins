//! Product identity and catalog discovery, independent of the CAD host.
//!
//! Providers keep their configuration types and geometry. This crate supplies the
//! common protocol and identity rules without knowing which plugin exposes a part.

pub mod catalog;
pub mod detail;
pub mod identity;

pub use catalog::{Catalog, CatalogError, CatalogRequest, CatalogResponse};
pub use identity::{CanonicalIdentity, Product, ProductKind, Representation};
