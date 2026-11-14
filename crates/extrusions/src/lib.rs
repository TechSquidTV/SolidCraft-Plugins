//! Slotted aluminium extrusion profiles: the 20 mm T-slot and V-slot rails.
//!
//! A rail is a grid of 20 mm cells. Each cell has a square core with a bore, a web from
//! each core corner, and a slot on every side that faces out. [`slot`] holds the slot
//! shapes with their sources, and [`profile`] builds the cross-section of a rail from a
//! slot shape and a grid.

pub mod profile;
pub mod slot;

pub use profile::{Grid, Profile, Seg};
pub use slot::Slot;
