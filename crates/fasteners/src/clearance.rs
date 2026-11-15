//! ISO 273 clearance holes, medium series. Sizes are millimetres.
//!
//! Source: ISO 273:1979, Table 1 (fine, medium and coarse series by nominal thread
//! diameter). `solid.hole` takes `clearance: "M6"` for the sizes its own table lists
//! (SolidCraft 0.41: M1.6 to M64, preferred sizes only). Every other ISO 273 size is
//! passed as a plain `diameter`, so a pocket or hole for an M3.5, M7 or M18 still builds.

use serde_json::{json, Value};

/// `(nominal, fine, medium, coarse)`.
const ISO_273: &[(f64, f64, f64, f64)] = &[
    (1.0, 1.1, 1.2, 1.3),
    (1.2, 1.3, 1.4, 1.5),
    (1.4, 1.5, 1.6, 1.8),
    (1.6, 1.7, 1.8, 2.0),
    (1.8, 2.0, 2.1, 2.2),
    (2.0, 2.2, 2.4, 2.6),
    (2.5, 2.7, 2.9, 3.1),
    (3.0, 3.2, 3.4, 3.6),
    (3.5, 3.7, 3.9, 4.2),
    (4.0, 4.3, 4.5, 4.8),
    (4.5, 4.8, 5.0, 5.3),
    (5.0, 5.3, 5.5, 5.8),
    (6.0, 6.4, 6.6, 7.0),
    (7.0, 7.4, 7.6, 8.0),
    (8.0, 8.4, 9.0, 10.0),
    (10.0, 10.5, 11.0, 12.0),
    (12.0, 13.0, 13.5, 14.5),
    (14.0, 15.0, 15.5, 16.5),
    (16.0, 17.0, 17.5, 18.5),
    (18.0, 19.0, 20.0, 21.0),
    (20.0, 21.0, 22.0, 24.0),
    (22.0, 23.0, 24.0, 26.0),
    (24.0, 25.0, 26.0, 28.0),
    (27.0, 28.0, 30.0, 32.0),
    (30.0, 31.0, 33.0, 35.0),
    (33.0, 34.0, 36.0, 38.0),
    (36.0, 37.0, 39.0, 42.0),
    (39.0, 40.0, 42.0, 45.0),
    (42.0, 43.0, 45.0, 48.0),
    (45.0, 46.0, 48.0, 52.0),
    (48.0, 50.0, 52.0, 56.0),
    (52.0, 54.0, 56.0, 62.0),
    (56.0, 58.0, 62.0, 66.0),
    (60.0, 62.0, 66.0, 70.0),
    (64.0, 66.0, 70.0, 74.0),
];

/// The nominal sizes `solid.hole`'s `clearance` field knows in SolidCraft 0.41.
const HOST_SIZES: &[f64] = &[
    1.6, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 20.0, 24.0, 30.0, 36.0, 42.0,
    48.0, 56.0, 60.0, 64.0,
];

fn same(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The ISO 273 medium clearance diameter for a bolt of nominal diameter `major`.
pub fn medium(major: f64) -> Option<f64> {
    ISO_273
        .iter()
        .find(|row| same(row.0, major))
        .map(|row| row.2)
}

/// How a bolt hole is named to `solid.hole`.
#[derive(Clone, Debug, PartialEq)]
pub enum Hole {
    /// `clearance: "M6"`, a size the host's own table has.
    Size(String),
    /// `diameter`, the ISO 273 medium hole for a size the host's table does not have.
    Diameter(f64),
}

impl Hole {
    /// The clearance hole for `major`, or `None` when ISO 273 does not list that size.
    pub fn for_major(major: f64) -> Option<Hole> {
        let diameter = medium(major)?;
        if HOST_SIZES.iter().any(|&d| same(d, major)) {
            Some(Hole::Size(format!("M{}", crate::metric::number(major))))
        } else {
            Some(Hole::Diameter(diameter))
        }
    }

    /// Add this hole's field to a `solid.hole` request.
    pub fn apply(&self, request: &mut Value) {
        match self {
            Hole::Size(size) => request["clearance"] = json!(size),
            Hole::Diameter(d) => request["diameter"] = json!(d),
        }
    }

    /// What a reply shows: the size, or the diameter.
    pub fn describe(&self) -> Value {
        match self {
            Hole::Size(size) => json!(size),
            Hole::Diameter(d) => json!(d),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_sizes_go_by_name() {
        assert_eq!(Hole::for_major(6.0), Some(Hole::Size("M6".into())));
        assert_eq!(Hole::for_major(1.6), Some(Hole::Size("M1.6".into())));
    }

    #[test]
    fn other_iso_273_sizes_go_by_diameter() {
        assert_eq!(Hole::for_major(3.5), Some(Hole::Diameter(3.9)));
        assert_eq!(Hole::for_major(7.0), Some(Hole::Diameter(7.6)));
        assert_eq!(Hole::for_major(18.0), Some(Hole::Diameter(20.0)));
        assert_eq!(Hole::for_major(52.0), Some(Hole::Diameter(56.0)));
    }

    #[test]
    fn a_size_iso_273_lacks_has_no_hole() {
        assert_eq!(Hole::for_major(7.5), None);
        assert_eq!(Hole::for_major(100.0), None);
    }

    #[test]
    fn every_host_size_is_in_iso_273() {
        for &d in HOST_SIZES {
            assert!(medium(d).is_some(), "M{d}");
        }
    }

    #[test]
    fn apply_sets_one_field() {
        let mut a = json!({});
        Hole::Size("M3".into()).apply(&mut a);
        assert_eq!(a, json!({"clearance": "M3"}));
        let mut b = json!({});
        Hole::Diameter(3.9).apply(&mut b);
        assert_eq!(b, json!({"diameter": 3.9}));
    }
}
