//! What every TechSquid SolidCraft plugin shares, whatever it builds.
//!
//! The host command runner and the wasm guest (`guest!`), the dialog fields and their
//! errors, placement on a selected or named sketch point, and the names given to bodies
//! and features.

pub mod host;
pub mod input;
pub mod name;
pub mod place;

pub use host::Runner;
pub use input::FieldError;
pub use place::{Placement, Source};
