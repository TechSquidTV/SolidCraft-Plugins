//! Fastener geometry shared by the screw, heat-set insert and nut plugins.
//!
//! Size designations, metric sizes, ISO 273 clearance holes, standard heads, ISO 4032
//! nuts, the hex profile, the ISO 10664 Torx recess, the ISO 4757 type H Phillips recess
//! and the slotted drive live here. What any plugin needs, fastener or not, is in
//! `plugin-kit`.

pub mod clearance;
pub mod designation;
pub mod head;
pub mod hex;
pub mod hexsocket;
pub mod metric;
pub mod nut;
pub mod phillips;
pub mod slot;
pub mod torx;
