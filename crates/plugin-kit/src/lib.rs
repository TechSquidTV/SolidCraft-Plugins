//! What every TechSquid SolidCraft plugin shares, whatever it builds.
//!
//! The host command runner and the wasm guest (`guest!`), the dialog fields and their
//! errors, placement on a selected or named sketch point, parts placed as components,
//! and the names given to bodies and features.

pub mod component;
pub mod content;
pub mod create;
pub mod host;
pub mod input;
pub mod name;
pub mod place;
pub mod revolve;
pub mod timeline;

pub use host::Runner;
pub use input::FieldError;
pub use place::{Placement, Source};

/// Catalog dispatch has no runner argument and therefore cannot change a design.
/// This also keeps protocol errors consistent across every guest.
pub fn query_catalog(
    catalog: &impl parts::Catalog,
    request: &serde_json::Value,
) -> Result<serde_json::Value, parts::CatalogError> {
    catalog.query(request)?.json()
}
