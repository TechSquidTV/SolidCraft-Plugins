//! Shared logic for the SolidCraft threadcraft plugins.
//!
//! Each plugin owns its entry and the parameters a person sets. Designations, metric
//! sizes, ISO 273 clearance holes, standard heads, ISO 4032 nuts, the hex profile, the ISO 10664 Torx recess,
//! the ISO 4757 type H Phillips recess, the slotted drive, placement, fasteners as components, and the host
//! command runner live here.

pub mod clearance;
pub mod fasten;
pub mod head;
pub mod hex;
pub mod hexsocket;
pub mod host;
pub mod input;
pub mod metric;
pub mod name;
pub mod nut;
pub mod phillips;
pub mod place;
pub mod slot;
pub mod torx;

pub use host::Runner;
pub use input::FieldError;
pub use place::{Placement, Source};
