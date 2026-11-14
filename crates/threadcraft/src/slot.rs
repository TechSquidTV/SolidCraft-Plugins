//! A straight slot for a slotted screw.
//!
//! ISO 2380-1 sizes the screwdriver tip. Its nominal thickness `a` is the slot width the
//! product standard calls `n`. The depth is not in that tip standard. A headed screw uses
//! the ISO 1580 pan-head minimum `t`. A set screw uses the smaller ISO 4766 slot, nominal
//! `n` and minimum `t`. The cut is that nominal width and that minimum depth. The tip
//! wedge, the test disc and the torque are not modeled.
//!
//! Sources:
//! - `HEADED`: ISO 1580:2011, Table 1
//! - `CHEESE`: ISO 1207:2011, Table 1; also the ARAMFIX data sheet "DIN 84"
//! - `COUNTERSUNK`: ISO 2009:2011, Table 1
//! - `SET`: ISO 4766, <https://www.fasteners.eu/standards/ISO/4766/>
//! - ISO 2380-1, the screwdriver tip: <https://www.iso.org/standard/80083.html>

use crate::input::millimetres;
use crate::metric::same_mm;
use crate::FieldError;

/// A slot ready to cut. Dimensions are millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// Nominal width `n`. This is the ISO 2380-1 tip thickness `a`.
    pub width: f64,
    /// Minimum penetration `t`.
    pub depth: f64,
}

/// ISO 1580 nominal slot for a headed screw. `(nominal, n, t min)`.
/// M3.5 is the non-preferred size in that table.
const HEADED: &[(f64, f64, f64)] = &[
    (1.6, 0.4, 0.35),
    (2.0, 0.5, 0.50),
    (2.5, 0.6, 0.60),
    (3.0, 0.8, 0.70),
    (3.5, 1.0, 0.80),
    (4.0, 1.2, 1.00),
    (5.0, 1.2, 1.20),
    (6.0, 1.6, 1.40),
    (8.0, 2.0, 1.90),
    (10.0, 2.5, 2.40),
];

/// ISO 4766 slot for a set screw. `(nominal, n, t min)`.
const SET: &[(f64, f64, f64)] = &[
    (1.0, 0.20, 0.40),
    (1.2, 0.20, 0.40),
    (1.4, 0.20, 0.48),
    (1.6, 0.25, 0.56),
    (2.0, 0.25, 0.64),
    (2.5, 0.40, 0.72),
    (3.0, 0.40, 0.80),
    (3.5, 0.50, 0.96),
    (4.0, 0.60, 1.12),
    (5.0, 0.80, 1.28),
    (6.0, 1.00, 1.60),
    (8.0, 1.20, 2.00),
    (10.0, 1.60, 2.40),
    (12.0, 2.00, 2.80),
];

/// DIN 84 / ISO 1207 slot for a cheese head. `(nominal, n, t min)`.
const CHEESE: &[(f64, f64, f64)] = &[
    (1.6, 0.4, 0.45),
    (2.0, 0.5, 0.60),
    (2.5, 0.6, 0.70),
    (3.0, 0.8, 0.85),
    (3.5, 1.0, 1.00),
    (4.0, 1.2, 1.10),
    (5.0, 1.2, 1.30),
    (6.0, 1.6, 1.60),
    (8.0, 2.0, 2.00),
    (10.0, 2.5, 2.40),
];

/// ISO 2009:2011 slot for a countersunk flat head. `(nominal, n, t min)`. It is
/// shallower than the pan head's, because the head is lower.
const COUNTERSUNK: &[(f64, f64, f64)] = &[
    (1.6, 0.40, 0.32),
    (2.0, 0.50, 0.40),
    (2.5, 0.60, 0.50),
    (3.0, 0.80, 0.60),
    (3.5, 1.00, 0.90),
    (4.0, 1.20, 1.00),
    (5.0, 1.20, 1.10),
    (6.0, 1.60, 1.20),
    (8.0, 2.00, 1.80),
    (10.0, 2.50, 2.00),
];

const BAD_SLOT: &str = "must be a width and a depth in millimetres, such as 0.8x0.7";

/// The ISO 1580 slot for this nominal diameter. A fine pitch uses the same nominal.
pub fn for_headed(nominal: f64) -> Option<Slot> {
    row(HEADED, nominal)
}

/// The DIN 84 / ISO 1207 slot for this nominal diameter. It is deeper than the pan
/// head's, because the cheese head is taller.
pub fn for_cheese(nominal: f64) -> Option<Slot> {
    row(CHEESE, nominal)
}

/// The ISO 2009 slot for this nominal diameter.
pub fn for_countersunk(nominal: f64) -> Option<Slot> {
    row(COUNTERSUNK, nominal)
}

/// The ISO 4766 slot for this nominal diameter. A fine pitch uses the same nominal.
pub fn for_set(nominal: f64) -> Option<Slot> {
    row(SET, nominal)
}

fn row(table: &[(f64, f64, f64)], nominal: f64) -> Option<Slot> {
    table
        .iter()
        .find(|(size, _, _)| same_mm(*size, nominal))
        .map(|(_, width, depth)| Slot {
            width: *width,
            depth: *depth,
        })
}

/// `0.8x0.7`, with `x`, `X` or `×` between the width and the depth.
pub fn parse_size(text: &str) -> Result<Slot, FieldError> {
    let mut parts = text.split(['x', 'X', '×']);
    let width = parts.next().and_then(millimetres);
    let depth = parts.next().and_then(millimetres);
    if parts.next().is_some() || width.is_none() || depth.is_none() {
        return Err(FieldError::new("slot", BAD_SLOT));
    }
    Ok(Slot {
        width: width.unwrap(),
        depth: depth.unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headed_slots_match_iso_1580() {
        let table = [
            (1.6, 0.4, 0.35),
            (2.0, 0.5, 0.50),
            (2.5, 0.6, 0.60),
            (3.0, 0.8, 0.70),
            (3.5, 1.0, 0.80),
            (4.0, 1.2, 1.00),
            (5.0, 1.2, 1.20),
            (6.0, 1.6, 1.40),
            (8.0, 2.0, 1.90),
            (10.0, 2.5, 2.40),
        ];
        assert_eq!(HEADED.len(), table.len());
        for (nominal, width, depth) in table {
            assert_eq!(for_headed(nominal), Some(Slot { width, depth }));
        }
        assert!(for_headed(12.0).is_none());
    }

    /// ISO 4766, <https://www.fasteners.eu/standards/ISO/4766/>: `n` nominal and `t` min.
    #[test]
    fn set_slots_match_iso_4766() {
        let table = [
            (1.0, 0.20, 0.40),
            (1.2, 0.20, 0.40),
            (1.4, 0.20, 0.48),
            (1.6, 0.25, 0.56),
            (2.0, 0.25, 0.64),
            (2.5, 0.40, 0.72),
            (3.0, 0.40, 0.80),
            (3.5, 0.50, 0.96),
            (4.0, 0.60, 1.12),
            (5.0, 0.80, 1.28),
            (6.0, 1.00, 1.60),
            (8.0, 1.20, 2.00),
            (10.0, 1.60, 2.40),
            (12.0, 2.00, 2.80),
        ];
        assert_eq!(SET.len(), table.len());
        for (nominal, width, depth) in table {
            assert_eq!(for_set(nominal), Some(Slot { width, depth }));
        }
        assert!(for_set(14.0).is_none());
        assert_ne!(for_set(3.0).unwrap().width, for_headed(3.0).unwrap().width);
    }

    #[test]
    fn a_size_is_width_then_depth() {
        assert_eq!(
            parse_size("0.8x0.7").unwrap(),
            Slot {
                width: 0.8,
                depth: 0.7
            }
        );
        assert_eq!(parse_size(" 1.6 × 1.4 ").unwrap().width, 1.6);
        assert!(parse_size("0.8").is_err());
        assert!(parse_size("0.8x0").is_err());
        assert!(parse_size("0.8x0.7x1").is_err());
    }

    /// ISO 1207:2011, Table 1, and the DIN 84 data sheet: `n` nominal and `t` min.
    #[test]
    fn cheese_slots_match_iso_1207() {
        let iso_1207 = [
            (1.6, 0.4, 0.45),
            (2.0, 0.5, 0.60),
            (2.5, 0.6, 0.70),
            (3.0, 0.8, 0.85),
            (3.5, 1.0, 1.00),
            (4.0, 1.2, 1.10),
            (5.0, 1.2, 1.30),
            (6.0, 1.6, 1.60),
            (8.0, 2.0, 2.00),
            (10.0, 2.5, 2.40),
        ];
        assert_eq!(CHEESE.len(), iso_1207.len());
        for (nominal, width, depth) in iso_1207 {
            assert_eq!(
                for_cheese(nominal),
                Some(Slot { width, depth }),
                "M{nominal}"
            );
        }
        // Same width as the pan head's slot, and deeper.
        assert_eq!(
            for_cheese(3.0).unwrap().width,
            for_headed(3.0).unwrap().width
        );
        assert!(for_cheese(3.0).unwrap().depth > for_headed(3.0).unwrap().depth);
    }

    /// ISO 2009:2011, Table 1: `n` nominal and `t` min.
    #[test]
    fn countersunk_slots_match_iso_2009() {
        let iso_2009 = [
            (1.6, 0.4, 0.32),
            (2.0, 0.5, 0.4),
            (2.5, 0.6, 0.5),
            (3.0, 0.8, 0.6),
            (3.5, 1.0, 0.9),
            (4.0, 1.2, 1.0),
            (5.0, 1.2, 1.1),
            (6.0, 1.6, 1.2),
            (8.0, 2.0, 1.8),
            (10.0, 2.5, 2.0),
        ];
        assert_eq!(COUNTERSUNK.len(), iso_2009.len());
        for (nominal, width, depth) in iso_2009 {
            assert_eq!(
                for_countersunk(nominal),
                Some(Slot { width, depth }),
                "M{nominal}"
            );
        }
        assert!(for_countersunk(12.0).is_none());
    }

    /// ISO 1580:2011, Table 1: `n` nominal and `t` min.
    #[test]
    fn pan_slots_match_iso_1580() {
        let iso_1580 = [
            (1.6, 0.4, 0.35),
            (2.0, 0.5, 0.50),
            (2.5, 0.6, 0.60),
            (3.0, 0.8, 0.70),
            (3.5, 1.0, 0.80),
            (4.0, 1.2, 1.00),
            (5.0, 1.2, 1.20),
            (6.0, 1.6, 1.40),
            (8.0, 2.0, 1.90),
            (10.0, 2.5, 2.40),
        ];
        assert_eq!(HEADED.len(), iso_1580.len());
        for (nominal, width, depth) in iso_1580 {
            assert_eq!(
                for_headed(nominal),
                Some(Slot { width, depth }),
                "M{nominal}"
            );
        }
    }
}
