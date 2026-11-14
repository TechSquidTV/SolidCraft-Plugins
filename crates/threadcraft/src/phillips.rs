//! ISO 4757 type H cross recess, the Phillips drive.
//!
//! One tool shape serves each recess number. The point is a cone at 28° to the
//! horizontal, and its diameter is the nominal `g` where that cone meets the wings.
//! Above that, the wing tips open at 26°30′ to the screw axis, so a deeper recess is
//! a wider one. Wing thickness is the nominal `b`, with straight walls. The side
//! lean `β`, the blunt point `α`, the corner radius and the step `f` are not cut.
//! Type Z, Pozidriv, is a different recess.
//!
//! The product standards give the width across the wing tips at the face, `m`, and
//! the recess is cut to that width: an M3 pan head opens to 3 mm. The penetration
//! they give is a gauge reading, taken on the wings, so the tip of the point lies
//! deeper than it.
//!
//! ISO 4757 does not give the penetration. A flat-top head uses the ISO 7045 minimum,
//! measured from the flat outer face. A countersunk head uses the ISO 7046-1 minimum,
//! and the wings have to stay inside the cone as that depth is projected down it.
//!
//! Sources:
//! - `HEADED`: ISO 7045:2011, Table 1
//! - `COUNTERSUNK`: ISO 7046-1:2011, Table 1
//! - `TOOLS`, `t1` and the two angles: ISO 4757:1983, Table 1 and section A-A of the
//!   figure in 2.1. The standard notes that these are theoretical values. Its `e`, `f`,
//!   `r`, `α` and `β` shape the wing flanks and the blunt point, which are not cut.

use crate::input::millimetres;
use crate::metric::same_mm;
use crate::FieldError;

/// Wing tips, degrees off the screw axis. ISO 4757 section A-A, 26°30′.
const WING_FROM_AXIS_DEG: f64 = 26.5;
/// The point, degrees off the horizontal. ISO 4757 section A-A, 28°.
const POINT_FROM_LEVEL_DEG: f64 = 28.0;

/// Nominal tools. `(number, b, g)`. Basic sizes, not the tolerance extremes.
const TOOLS: &[(u8, f64, f64)] = &[
    (0, 0.61, 0.81),
    (1, 0.97, 1.27),
    (2, 1.47, 2.29),
    (3, 2.41, 3.81),
    (4, 3.48, 5.08),
];

/// ISO 7045:2011, Table 1, type H, for a pan head and other flat-topped heads.
/// `(nominal, number, penetration min, penetration max, m ref)`. `m` is the width
/// across the wing tips at the face.
const HEADED: &[Row] = &[
    (1.6, 0, 0.70, 0.95, 1.70),
    (2.0, 0, 0.90, 1.20, 1.90),
    (2.5, 1, 1.15, 1.55, 2.70),
    (3.0, 1, 1.40, 1.80, 3.00),
    (3.5, 2, 1.40, 1.90, 3.90),
    (4.0, 2, 1.90, 2.40, 4.40),
    (5.0, 2, 2.40, 2.90, 4.90),
    (6.0, 3, 3.10, 3.60, 6.90),
    (8.0, 4, 4.00, 4.60, 9.00),
    (10.0, 4, 5.20, 5.80, 10.10),
];

/// ISO 7046-1:2011, Table 1, type H, for a countersunk flat head. The same columns.
const COUNTERSUNK: &[Row] = &[
    (1.6, 0, 0.60, 0.90, 1.60),
    (2.0, 0, 0.90, 1.20, 1.90),
    (2.5, 1, 1.40, 1.80, 2.90),
    (3.0, 1, 1.70, 2.10, 3.20),
    (3.5, 2, 1.90, 2.40, 4.40),
    (4.0, 2, 2.10, 2.60, 4.60),
    (5.0, 2, 2.70, 3.20, 5.20),
    (6.0, 3, 3.00, 3.50, 6.80),
    (8.0, 4, 4.00, 4.60, 8.90),
    (10.0, 4, 5.10, 5.70, 10.00),
];

type Row = (f64, u8, f64, f64, f64);

/// Where the wing tips would be at no penetration, for each recess number: the width
/// across them at the face is this plus `2 × tan 26°30′` for each millimetre of
/// penetration. Fitted to `m ref` and the middle of the penetration range in all
/// twenty rows of ISO 7045 and ISO 7046-1, which it reproduces to within 0.08 mm. A
/// standard screw is cut to its row's `m`; this line sizes a recess that is typed.
const SPREAD: &[(u8, f64)] = &[(0, 0.86), (1, 1.34), (2, 2.257), (3, 3.56), (4, 4.64)];

const BAD_SIZE: &str = "must be a Type H size, such as PH2 or PH2x2.1";

/// A type H recess ready to cut. Dimensions are millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tool {
    pub number: u8,
    /// Nominal wing thickness `b`.
    pub width: f64,
    /// Nominal width `g` at the bottom of the wings.
    pub bottom: f64,
    /// The gauged penetration the product standard gives, its minimum.
    pub depth: f64,
    /// Width across the wing tips at the outer face, the standard's `m`.
    pub opening: f64,
}

impl Tool {
    /// Height of the 28° point. This is the ISO 4757 reference `t1`.
    pub fn point_height(self) -> f64 {
        point_height(self.bottom)
    }

    /// Wing-tip radius, `from_point` millimetres up from the point.
    pub fn radius_at(self, from_point: f64) -> f64 {
        radius_at(self.bottom, from_point)
    }

    /// Diameter of the wing tips where they meet the outer face.
    pub fn opening_diameter(self) -> f64 {
        self.opening
    }

    /// How far below the face the wings narrow to `g`, where the point begins.
    pub fn wing_depth(self) -> f64 {
        (self.opening - self.bottom) / (2.0 * WING_FROM_AXIS_DEG.to_radians().tan())
    }

    /// How far below the face the recess ends, at the tip of its point. This is deeper
    /// than the gauged penetration, because the gauge stops on the wings.
    pub fn reach(self) -> f64 {
        self.wing_depth() + self.point_height()
    }
}

/// Height of the 28° point for a bottom width `g`.
pub fn point_height(bottom: f64) -> f64 {
    (bottom / 2.0) * POINT_FROM_LEVEL_DEG.to_radians().tan()
}

/// Wing-tip radius, `from_point` millimetres up from the point.
///
/// Up to the point height the radius follows the 28° cone. Above that it follows
/// the 26°30′ wing tips, so the opening grows with the penetration.
pub fn radius_at(bottom: f64, from_point: f64) -> f64 {
    let h1 = point_height(bottom);
    if from_point <= h1 {
        from_point / POINT_FROM_LEVEL_DEG.to_radians().tan()
    } else {
        bottom / 2.0 + (from_point - h1) * WING_FROM_AXIS_DEG.to_radians().tan()
    }
}

/// Nominal `b` and `g` for a recess number. `None` when the number is not 0 to 4.
pub fn tool(number: u8) -> Option<(f64, f64)> {
    TOOLS
        .iter()
        .find(|(n, _, _)| *n == number)
        .map(|(_, width, bottom)| (*width, *bottom))
}

/// The recess a product standard gives a thread size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standard {
    pub number: u8,
    /// Penetration min.
    pub depth: f64,
    /// The middle of the penetration range, where `m` is taken.
    pub nominal_depth: f64,
    /// `m approx`, the width across the wing tips at the face.
    pub opening: f64,
}

/// The ISO 7045 recess for this nominal diameter.
pub fn headed(nominal: f64) -> Option<Standard> {
    standard(HEADED, nominal)
}

/// The ISO 7046 recess for this nominal diameter.
pub fn countersunk(nominal: f64) -> Option<Standard> {
    standard(COUNTERSUNK, nominal)
}

/// ISO 7045 type H number and minimum penetration for this nominal diameter.
pub fn for_headed(nominal: f64) -> Option<(u8, f64)> {
    headed(nominal).map(|row| (row.number, row.depth))
}

/// ISO 7046 type H number and minimum penetration for this nominal diameter.
pub fn for_countersunk(nominal: f64) -> Option<(u8, f64)> {
    countersunk(nominal).map(|row| (row.number, row.depth))
}

fn standard(table: &[Row], nominal: f64) -> Option<Standard> {
    table
        .iter()
        .find(|row| same_mm(row.0, nominal))
        .map(|&(_, number, min, max, opening)| Standard {
            number,
            depth: min,
            nominal_depth: (min + max) / 2.0,
            opening,
        })
}

/// The width across the wing tips at the face for a recess sunk to `penetration`.
/// `None` when the number is not 0 to 4.
pub fn opening_at(number: u8, penetration: f64) -> Option<f64> {
    SPREAD
        .iter()
        .find(|(n, _)| *n == number)
        .map(|(_, spread)| spread + 2.0 * WING_FROM_AXIS_DEG.to_radians().tan() * penetration)
}

/// A typed recess. A bare number keeps the product-standard penetration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Request {
    Number(u8),
    Sized { number: u8, depth: f64 },
}

/// `PH2`, `ph2`, `2`, `#2`, or the same with a penetration: `PH2x2.1`.
pub fn parse_size(text: &str) -> Result<Request, FieldError> {
    let compact: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();
    let rest = compact.strip_prefix('#').unwrap_or(&compact);
    let rest = rest.strip_prefix("ph").unwrap_or(rest);
    let mut parts = rest.split(['x', '×']);
    let number = parts.next().and_then(|text| text.parse::<u8>().ok());
    let depth = match parts.next() {
        None => None,
        Some(text) => Some(millimetres(text).ok_or_else(|| FieldError::new("phillips", BAD_SIZE))?),
    };
    if parts.next().is_some() {
        return Err(FieldError::new("phillips", BAD_SIZE));
    }
    let Some(number) = number.filter(|number| tool(*number).is_some()) else {
        return Err(FieldError::new("phillips", BAD_SIZE));
    };
    Ok(match depth {
        Some(depth) => Request::Sized { number, depth },
        None => Request::Number(number),
    })
}

/// A straight plus, `width` thick, reaching `reach` from the centre on each arm.
///
/// The first point is the top arm and the rest go clockwise. The outline is closed
/// by the caller, so the first point is not repeated.
pub fn plus(width: f64, reach: f64) -> [[f64; 2]; 12] {
    let w = width / 2.0;
    let l = reach;
    [
        [w, l],
        [w, w],
        [l, w],
        [l, -w],
        [w, -w],
        [w, -l],
        [-w, -l],
        [-w, -w],
        [-l, -w],
        [-l, w],
        [-w, w],
        [-w, l],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ISO 4757:1983 Table 1, recess type H: `b` and `g` for recess numbers 0 to 4.
    #[test]
    fn tools_match_iso_4757() {
        let iso_4757 = [
            (0, 0.61, 0.81),
            (1, 0.97, 1.27),
            (2, 1.47, 2.29),
            (3, 2.41, 3.81),
            (4, 3.48, 5.08),
        ];
        assert_eq!(TOOLS.len(), iso_4757.len());
        for (number, b, g) in iso_4757 {
            assert_eq!(tool(number), Some((b, g)), "No. {number}");
        }
        // Section A-A of the figure in 2.1.
        assert_eq!(WING_FROM_AXIS_DEG, 26.5);
        assert_eq!(POINT_FROM_LEVEL_DEG, 28.0);
    }

    /// ISO 4757:1983 Table 1, `t1` ref.
    #[test]
    fn point_height_matches_the_iso_4757_reference() {
        // Table 1 reference `t1`. Size 0 has no included angle; the 28° cone gives its `t1`.
        let table = [
            (0.81, 0.22),
            (1.27, 0.34),
            (2.29, 0.61),
            (3.81, 1.01),
            (5.08, 1.35),
        ];
        for (bottom, t1) in table {
            assert!(
                (point_height(bottom) - t1).abs() < 0.02,
                "{bottom}: {} vs {t1}",
                point_height(bottom)
            );
        }
    }

    #[test]
    fn an_m3_opening_follows_the_wing_angle() {
        // PH1 with its point 1.7 mm down. The 26°30′ tips land near 2.63 mm.
        let diameter = 2.0 * radius_at(1.27, 1.7);
        assert!((diameter - 2.63).abs() < 0.02, "{diameter}");
        assert!((radius_at(1.27, point_height(1.27)) - 1.27 / 2.0).abs() < 1e-9);
    }

    /// ISO 7045:2011, Table 1: type H number and penetration min.
    #[test]
    fn headed_recesses_match_iso_7045() {
        let table = [
            (1.6, 0, 0.70),
            (2.0, 0, 0.90),
            (2.5, 1, 1.15),
            (3.0, 1, 1.40),
            (3.5, 2, 1.40),
            (4.0, 2, 1.90),
            (5.0, 2, 2.40),
            (6.0, 3, 3.10),
            (8.0, 4, 4.00),
            (10.0, 4, 5.20),
        ];
        assert_eq!(HEADED.len(), table.len());
        for (nominal, number, depth) in table {
            assert_eq!(for_headed(nominal), Some((number, depth)));
        }
    }

    /// ISO 7046-1:2011, Table 1: type H number and penetration min.
    #[test]
    fn countersunk_recesses_match_iso_7046() {
        let table = [
            (1.6, 0, 0.60),
            (2.0, 0, 0.90),
            (2.5, 1, 1.40),
            (3.0, 1, 1.70),
            (3.5, 2, 1.90),
            (4.0, 2, 2.10),
            (5.0, 2, 2.70),
            (6.0, 3, 3.00),
            (8.0, 4, 4.00),
            (10.0, 4, 5.10),
        ];
        assert_eq!(COUNTERSUNK.len(), table.len());
        for (nominal, number, depth) in table {
            assert_eq!(for_countersunk(nominal), Some((number, depth)));
        }
    }

    #[test]
    fn a_pan_and_a_countersunk_head_take_their_own_rows() {
        // Both standards give an M2 recess 0, 0.9 mm, and they part from M2.5 on.
        assert_eq!(for_headed(2.0), Some((0, 0.90)));
        assert_eq!(for_countersunk(2.0), Some((0, 0.90)));
        assert_eq!(for_headed(3.0), Some((1, 1.40)));
        assert_eq!(for_countersunk(3.0), Some((1, 1.70)));
    }

    #[test]
    fn a_size_reads_the_number_and_an_optional_depth() {
        assert_eq!(parse_size("PH2").unwrap(), Request::Number(2));
        assert_eq!(parse_size("ph2").unwrap(), Request::Number(2));
        assert_eq!(parse_size("#2").unwrap(), Request::Number(2));
        assert_eq!(parse_size("2").unwrap(), Request::Number(2));
        assert_eq!(
            parse_size("PH2x2.1").unwrap(),
            Request::Sized {
                number: 2,
                depth: 2.1
            }
        );
        assert_eq!(
            parse_size("2×2.1").unwrap(),
            Request::Sized {
                number: 2,
                depth: 2.1
            }
        );
        assert_eq!(parse_size("PH5").unwrap_err().field, "phillips");
        assert_eq!(parse_size("PH2x").unwrap_err().field, "phillips");
    }

    #[test]
    fn the_plus_is_twelve_points_around_a_cross() {
        let plus = plus(0.97, 2.0);
        assert_eq!(plus[0], [0.485, 2.0]);
        assert_eq!(plus[2], [2.0, 0.485]);
        assert_eq!(plus[5], [0.485, -2.0]);
        assert_eq!(plus[8], [-2.0, -0.485]);
        assert_eq!(plus.len(), 12);
    }

    /// `m ref` and the penetration range from ISO 7045:2011 and ISO 7046-1:2011.
    #[test]
    fn openings_match_iso_7045_and_iso_7046() {
        let iso_7045 = [
            (1.6, 0.7, 0.95, 1.7),
            (2.0, 0.9, 1.2, 1.9),
            (2.5, 1.15, 1.55, 2.7),
            (3.0, 1.4, 1.8, 3.0),
            (3.5, 1.4, 1.9, 3.9),
            (4.0, 1.9, 2.4, 4.4),
            (5.0, 2.4, 2.9, 4.9),
            (6.0, 3.1, 3.6, 6.9),
            (8.0, 4.0, 4.6, 9.0),
            (10.0, 5.2, 5.8, 10.1),
        ];
        let iso_7046 = [
            (1.6, 0.6, 0.9, 1.6),
            (2.0, 0.9, 1.2, 1.9),
            (2.5, 1.4, 1.8, 2.9),
            (3.0, 1.7, 2.1, 3.2),
            (3.5, 1.9, 2.4, 4.4),
            (4.0, 2.1, 2.6, 4.6),
            (5.0, 2.7, 3.2, 5.2),
            (6.0, 3.0, 3.5, 6.8),
            (8.0, 4.0, 4.6, 8.9),
            (10.0, 5.1, 5.7, 10.0),
        ];
        for (table, find) in [
            (iso_7045, headed as fn(f64) -> Option<Standard>),
            (iso_7046, countersunk),
        ] {
            for (d, min, max, m) in table {
                let row = find(d).unwrap();
                assert_eq!((row.depth, row.opening), (min, m), "M{d}");
                assert!(
                    (row.nominal_depth - (min + max) / 2.0).abs() < 1e-12,
                    "M{d}"
                );
                // The fitted wing line gives nearly the same width as the standard.
                let fitted = opening_at(row.number, row.nominal_depth).unwrap();
                assert!((fitted - m).abs() < 0.08, "M{d}: {fitted} vs {m}");
            }
        }
        assert!(opening_at(5, 1.0).is_none());
    }

    #[test]
    fn an_m3_pan_recess_opens_to_3_and_ends_below_its_gauged_depth() {
        let row = headed(3.0).unwrap();
        let (width, bottom) = tool(row.number).unwrap();
        let tool = Tool {
            number: row.number,
            width,
            bottom,
            depth: row.depth,
            opening: row.opening,
        };
        assert_eq!(tool.opening_diameter(), 3.0);
        // The wings reach `g` 1.73 mm down, and the 28 degree point adds 0.34.
        assert!(
            (tool.wing_depth() - 1.735).abs() < 0.005,
            "{}",
            tool.wing_depth()
        );
        assert!((tool.reach() - 2.073).abs() < 0.005, "{}", tool.reach());
        assert!(tool.reach() > tool.depth);
        // Followed up from the point, the wing tips come out at the opening.
        assert!((2.0 * tool.radius_at(tool.reach()) - 3.0).abs() < 1e-9);
    }
}
