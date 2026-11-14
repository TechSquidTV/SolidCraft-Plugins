//! Nominal head sizes for the metric screws the dialog can fill in on its own.
//!
//! Sources, one per table. The tests at the end of this file repeat each table from
//! its source, so a typing slip in either copy fails.
//!
//! | Table | Standard | Source |
//! |---|---|---|
//! | `SOCKET` | DIN 912 / ISO 4762 | ARAMFIX data sheet "DIN 912", <https://www.fasteners.eu/standards/DIN/912/> |
//! | `BUTTON` | ISO 7380-1:2011 | the standard, Table 1 |
//! | `COUNTERSUNK` | ISO 10642:2004 | the standard, Table 1 |
//! | `COUNTERSUNK_FLAT` | ISO 2009:2011, ISO 7046-1:2011 | the standards, Table 1 |
//! | `HEX` | ISO 4017 | <https://www.fasteners.eu/standards/ISO/4017/> |
//! | `PAN` | ISO 1580:2011 | the standard, Table 1 |
//! | `CHEESE` | ISO 1207:2011 | the standard, Table 1; also the ARAMFIX data sheet "DIN 84" |
//! | `PAN_CROSS` | ISO 7045:2011 | the standard, Table 1 |
//!
//! The older DIN standards are not all the same as their ISO successors. DIN 7985,
//! DIN 965 and DIN 963 differ from ISO 7045, ISO 7046 and ISO 2009 in head size and
//! recess, and fasteners.eu prints the DIN table under both numbers, so those three
//! come from the ISO documents. The head tops (`Crown`) and the thread lengths `b`
//! come from the same tables.
//!
//! Each row is the nominal maximum from the standard, in millimetres: the size a model
//! uses when the person leaves the head diameter and height empty. A hex head's diameter
//! is the width across the flats.

use super::metric::same_mm;

/// Which standard head to look up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadKind {
    /// ISO 4762 socket head, the current form of DIN 912. Diameter is the head diameter.
    Socket,
    /// ISO 7380-1 button head. Diameter is the head diameter.
    Button,
    /// ISO 10642 hexagon socket countersunk head. Diameter is the head diameter.
    Countersunk,
    /// ISO 2009 slotted and ISO 7046-1 cross-recessed countersunk flat head. Smaller than
    /// the ISO 10642 head, with a cylindrical land above the 90° cone.
    CountersunkFlat,
    /// ISO 4017 hex head. Diameter is the width across the flats.
    Hex,
    /// ISO 1580 pan head. Diameter is the head diameter.
    Pan,
    /// DIN 84 / ISO 1207 cheese head. Diameter is the head diameter.
    Cheese,
    /// ISO 7045 cross-recessed pan head. As wide as the slotted ISO 1580 pan and
    /// taller, so the Phillips recess has a floor under it.
    PanCross,
}

impl HeadKind {
    /// The standard this head is sized from, as the supported-standards table names it.
    pub fn standard(self) -> &'static str {
        match self {
            HeadKind::Socket => "ISO 4762",
            HeadKind::Button => "ISO 7380-1",
            HeadKind::Countersunk => "ISO 10642",
            HeadKind::CountersunkFlat => "ISO 2009 / ISO 7046",
            HeadKind::Hex => "ISO 4017",
            HeadKind::Pan => "ISO 1580",
            HeadKind::Cheese => "ISO 1207",
            HeadKind::PanCross => "ISO 7045",
        }
    }
}

/// A head taken from a standard, before any override.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StandardHead {
    pub diameter: f64,
    pub height: f64,
    /// `diameter` is the width across the flats, not the corner diameter.
    pub across_flats: bool,
}

/// DIN 912 / ISO 4762: `dk` max and `k` max.
/// `(nominal diameter, head diameter or across-flats, head height)`, as in every table here.
const SOCKET: &[(f64, f64, f64)] = &[
    (1.6, 3.0, 1.6),
    (2.0, 3.8, 2.0),
    (2.5, 4.5, 2.5),
    (3.0, 5.5, 3.0),
    (4.0, 7.0, 4.0),
    (5.0, 8.5, 5.0),
    (6.0, 10.0, 6.0),
    (8.0, 13.0, 8.0),
    (10.0, 16.0, 10.0),
    (12.0, 18.0, 12.0),
    (14.0, 21.0, 14.0),
    (16.0, 24.0, 16.0),
    (20.0, 30.0, 20.0),
    (24.0, 36.0, 24.0),
    (30.0, 45.0, 30.0),
    (36.0, 54.0, 36.0),
    (42.0, 63.0, 42.0),
    (48.0, 72.0, 48.0),
    (56.0, 84.0, 56.0),
    (64.0, 96.0, 64.0),
];

/// ISO 7380-1:2011 Table 1: `dk` max and `k` max.
const BUTTON: &[(f64, f64, f64)] = &[
    (3.0, 5.7, 1.65),
    (4.0, 7.6, 2.2),
    (5.0, 9.5, 2.75),
    (6.0, 10.5, 3.3),
    (8.0, 14.0, 4.4),
    (10.0, 17.5, 5.5),
    (12.0, 21.0, 6.6),
    (16.0, 28.0, 8.8),
];

/// ISO 10642:2004 Table 1: the theoretical `dk` max and `k` max. That diameter is the sharp edge of
/// the 90° cone, `d + 2k`, so the head sits flush in a 90° countersink.
const COUNTERSUNK: &[(f64, f64, f64)] = &[
    (3.0, 6.72, 1.86),
    (4.0, 8.96, 2.48),
    (5.0, 11.20, 3.10),
    (6.0, 13.44, 3.72),
    (8.0, 17.92, 4.96),
    (10.0, 22.40, 6.20),
    (12.0, 26.88, 7.44),
    (16.0, 33.60, 8.80),
    (20.0, 40.32, 10.16),
];

/// ISO 2009:2011 and ISO 7046-1:2011, Table 1, which agree: the actual `dk` nominal =
/// max and `k` nominal = max. The 90° cone reaches `dk` below the top, and the rest
/// of `k` is a cylindrical land, which `countersink` works out; carried on to a sharp
/// edge the cone would be the standards' theoretical `dk`, `d + 2k`. M3.5 is the
/// non-preferred size.
const COUNTERSUNK_FLAT: &[(f64, f64, f64)] = &[
    (1.6, 3.00, 1.00),
    (2.0, 3.80, 1.20),
    (2.5, 4.70, 1.50),
    (3.0, 5.50, 1.65),
    (3.5, 7.30, 2.35),
    (4.0, 8.40, 2.70),
    (5.0, 9.30, 2.70),
    (6.0, 11.30, 3.30),
    (8.0, 15.80, 4.65),
    (10.0, 18.30, 5.00),
];

/// ISO 4017: width across the flats `s` (nominal = max) and head height `k` nominal,
/// preferred and non-preferred sizes.
const HEX: &[(f64, f64, f64)] = &[
    (1.6, 3.2, 1.1),
    (2.0, 4.0, 1.4),
    (2.5, 5.0, 1.7),
    (3.0, 5.5, 2.0),
    (3.5, 6.0, 2.4),
    (4.0, 7.0, 2.8),
    (5.0, 8.0, 3.5),
    (6.0, 10.0, 4.0),
    (8.0, 13.0, 5.3),
    (10.0, 16.0, 6.4),
    (12.0, 18.0, 7.5),
    (14.0, 21.0, 8.8),
    (16.0, 24.0, 10.0),
    (18.0, 27.0, 11.5),
    (20.0, 30.0, 12.5),
    (22.0, 34.0, 14.0),
    (24.0, 36.0, 15.0),
    (27.0, 41.0, 17.0),
    (30.0, 46.0, 18.7),
    (33.0, 50.0, 21.0),
    (36.0, 55.0, 22.5),
    (39.0, 60.0, 25.0),
    (42.0, 65.0, 26.0),
    (45.0, 70.0, 28.0),
    (48.0, 75.0, 30.0),
    (52.0, 80.0, 33.0),
    (56.0, 85.0, 35.0),
    (60.0, 90.0, 38.0),
    (64.0, 95.0, 40.0),
];

/// ISO 1580:2011, the slotted pan head: `dk` and `k` nominal = max. M3.5 is the
/// non-preferred size.
const PAN: &[(f64, f64, f64)] = &[
    (1.6, 3.2, 1.0),
    (2.0, 4.0, 1.3),
    (2.5, 5.0, 1.5),
    (3.0, 5.6, 1.8),
    (3.5, 7.0, 2.1),
    (4.0, 8.0, 2.4),
    (5.0, 9.5, 3.0),
    (6.0, 12.0, 3.6),
    (8.0, 16.0, 4.8),
    (10.0, 20.0, 6.0),
];

/// DIN 84 / ISO 1207: `dk` max and `k` max. M3.5 is the non-preferred size.
const CHEESE: &[(f64, f64, f64)] = &[
    (1.6, 3.0, 1.1),
    (2.0, 3.8, 1.4),
    (2.5, 4.5, 1.8),
    (3.0, 5.5, 2.0),
    (3.5, 6.0, 2.4),
    (4.0, 7.0, 2.6),
    (5.0, 8.5, 3.3),
    (6.0, 10.0, 3.9),
    (8.0, 13.0, 5.0),
    (10.0, 16.0, 6.0),
];

/// ISO 7045:2011, Table 1: `dk` and `k` nominal = max. M3.5 is the non-preferred size.
const PAN_CROSS: &[(f64, f64, f64)] = &[
    (1.6, 3.20, 1.30),
    (2.0, 4.00, 1.60),
    (2.5, 5.00, 2.10),
    (3.0, 5.60, 2.40),
    (3.5, 7.00, 2.60),
    (4.0, 8.00, 3.10),
    (5.0, 9.50, 3.70),
    (6.0, 12.00, 4.60),
    (8.0, 16.00, 6.00),
    (10.0, 20.00, 7.50),
];

pub fn standard_head(kind: HeadKind, nominal: f64) -> Option<StandardHead> {
    let table = match kind {
        HeadKind::Socket => SOCKET,
        HeadKind::Button => BUTTON,
        HeadKind::Countersunk => COUNTERSUNK,
        HeadKind::CountersunkFlat => COUNTERSUNK_FLAT,
        HeadKind::Hex => HEX,
        HeadKind::Pan => PAN,
        HeadKind::Cheese => CHEESE,
        HeadKind::PanCross => PAN_CROSS,
    };
    let (_, diameter, height) = table.iter().find(|(size, _, _)| same_mm(*size, nominal))?;
    Some(StandardHead {
        diameter: *diameter,
        height: *height,
        across_flats: kind == HeadKind::Hex,
    })
}

/// The top of a round head. Dimensions are millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Crown {
    /// A plain cylinder.
    Flat,
    /// ISO 7380-1 button: a spherical side of radius `radius` from the bearing face up
    /// to a flat top `flat` across.
    Button { radius: f64, flat: f64 },
    /// ISO 7045 pan: a cylindrical side, then a spherical top of `radius`.
    Dome { radius: f64 },
    /// ISO 1580 pan: a flat top with its edge rounded to `radius`.
    Rounded { radius: f64 },
}

/// ISO 7380-1:2011 Table 1. `(nominal, rf, dL)`: `rf` is the middle of its max and
/// min, and `dL` is the reference diameter of the flat top.
const BUTTON_CROWN: &[(f64, f64, f64)] = &[
    (3.0, 3.5, 2.6),
    (4.0, 4.4, 3.8),
    (5.0, 5.5, 5.0),
    (6.0, 5.9, 6.0),
    (8.0, 7.7, 7.7),
    (10.0, 9.5, 10.0),
    (12.0, 10.85, 12.0),
    (16.0, 14.9, 16.0),
];

/// ISO 7045:2011, Table 1: `rf` approx. `(nominal, rf)`.
const PAN_CROSS_CROWN: &[(f64, f64)] = &[
    (1.6, 2.5),
    (2.0, 3.2),
    (2.5, 4.0),
    (3.0, 5.0),
    (3.5, 6.0),
    (4.0, 6.5),
    (5.0, 8.0),
    (6.0, 10.0),
    (8.0, 13.0),
    (10.0, 16.0),
];

/// ISO 1580:2011, Table 1: `rf` ref. `(nominal, rf)`.
const PAN_CROWN: &[(f64, f64)] = &[
    (1.6, 0.5),
    (2.0, 0.6),
    (2.5, 0.8),
    (3.0, 0.9),
    (3.5, 1.0),
    (4.0, 1.2),
    (5.0, 1.5),
    (6.0, 1.8),
    (8.0, 2.4),
    (10.0, 3.0),
];

/// The top of the standard head. Heads with a plain top, and sizes outside the
/// table, are `Flat`.
pub fn crown(kind: HeadKind, nominal: f64) -> Crown {
    let pick = |table: &[(f64, f64)]| {
        table
            .iter()
            .find(|(size, _)| same_mm(*size, nominal))
            .map(|(_, radius)| *radius)
    };
    match kind {
        HeadKind::Button => BUTTON_CROWN
            .iter()
            .find(|(size, _, _)| same_mm(*size, nominal))
            .map_or(Crown::Flat, |(_, radius, flat)| Crown::Button {
                radius: *radius,
                flat: *flat,
            }),
        HeadKind::PanCross => {
            pick(PAN_CROSS_CROWN).map_or(Crown::Flat, |radius| Crown::Dome { radius })
        }
        HeadKind::Pan => pick(PAN_CROWN).map_or(Crown::Flat, |radius| Crown::Rounded { radius }),
        _ => Crown::Flat,
    }
}

/// One piece of a head's half profile, in the plane through the axis: x is the radius
/// and y is the height above the bearing face.
#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Line(Vec<[f64; 2]>),
    Arc {
        start: [f64; 2],
        through: [f64; 2],
        end: [f64; 2],
    },
}

/// The closed half profile of a crowned head, to revolve about the axis. It starts
/// and ends on the axis at the bearing face. `None` when the head is a plain cylinder,
/// or when the crown does not fit this diameter and height.
pub fn profile(crown: Crown, diameter: f64, height: f64) -> Option<Vec<Segment>> {
    let r = diameter / 2.0;
    match crown {
        Crown::Flat => None,
        Crown::Button { radius, flat } => {
            let top = flat / 2.0;
            let (start, end) = ([r, 0.0], [top, height]);
            let chord = ((r - top).powi(2) + height.powi(2)).sqrt();
            if !(top > 0.0 && top < r && radius > chord / 2.0) {
                return None;
            }
            // The centre is on the axis side of the chord and below it, so the side
            // leans in from the bearing face.
            let mid = [(start[0] + end[0]) / 2.0, (start[1] + end[1]) / 2.0];
            let away = (radius * radius - chord * chord / 4.0).sqrt();
            let normal = [-height / chord, -(r - top) / chord];
            let centre = [mid[0] + normal[0] * away, mid[1] + normal[1] * away];
            let through = [
                centre[0] - normal[0] * radius,
                centre[1] - normal[1] * radius,
            ];
            Some(vec![
                Segment::Line(vec![[0.0, 0.0], start]),
                Segment::Arc {
                    start,
                    through,
                    end,
                },
                Segment::Line(vec![end, [0.0, height], [0.0, 0.0]]),
            ])
        }
        Crown::Dome { radius } => {
            if radius < r {
                return None;
            }
            let rise = radius - (radius * radius - r * r).sqrt();
            let side = height - rise;
            if side <= 0.0 {
                return None;
            }
            let half = (r / radius).asin() / 2.0;
            let through = [radius * half.sin(), height - radius + radius * half.cos()];
            Some(vec![
                Segment::Line(vec![[0.0, 0.0], [r, 0.0], [r, side]]),
                Segment::Arc {
                    start: [r, side],
                    through,
                    end: [0.0, height],
                },
                Segment::Line(vec![[0.0, height], [0.0, 0.0]]),
            ])
        }
        Crown::Rounded { radius } => {
            if !(radius > 0.0 && radius < r && radius < height) {
                return None;
            }
            let k = radius * std::f64::consts::FRAC_1_SQRT_2;
            Some(vec![
                Segment::Line(vec![[0.0, 0.0], [r, 0.0], [r, height - radius]]),
                Segment::Arc {
                    start: [r, height - radius],
                    through: [r - radius + k, height - radius + k],
                    end: [r - radius, height],
                },
                Segment::Line(vec![[r - radius, height], [0.0, height], [0.0, 0.0]]),
            ])
        }
    }
}

/// Standard thread length `b` for a screw long enough to have a plain shank.
///
/// - Socket, button and hexagon socket countersunk heads: `2d + 12`, the `b` row of
///   DIN 912 / ISO 4762, ISO 7380-1 and ISO 10642.
/// - Pan, cheese and flat countersunk heads: 25 mm up to M3 and 38 mm above, the `b`
///   row of ISO 7045, ISO 1580, ISO 1207, ISO 2009 and ISO 7046-1.
/// - A hex head (ISO 4017) is threaded to the head.
pub fn thread_length(kind: HeadKind, nominal: f64) -> Option<f64> {
    standard_head(kind, nominal)?;
    match kind {
        HeadKind::Socket | HeadKind::Button | HeadKind::Countersunk => {
            Some((2.0 * nominal + 12.0).floor())
        }
        HeadKind::PanCross | HeadKind::CountersunkFlat | HeadKind::Pan | HeadKind::Cheese => {
            Some(if nominal <= 3.0 { 25.0 } else { 38.0 })
        }
        HeadKind::Hex => None,
    }
}

/// ISO 10642:2004 Table 1: the shortest length that has a plain shank, for each size.
/// Shorter commercial lengths are in the shaded area, threaded to the head.
const COUNTERSUNK_SHANK_FROM: &[(f64, f64)] = &[
    (3.0, 30.0),
    (4.0, 30.0),
    (5.0, 35.0),
    (6.0, 40.0),
    (8.0, 50.0),
    (10.0, 55.0),
    (12.0, 65.0),
    (14.0, 70.0),
    (16.0, 80.0),
    (20.0, 100.0),
];

/// Whether a standard screw of this `length` has a plain shank above its thread, or is
/// threaded to the head. Each standard draws that boundary in its length table.
///
/// - DIN 912 / ISO 4762 and ISO 7380-1 shade the lengths threaded to the head. A screw
///   is clear of that area once it is longer than `b` plus eight and a half pitches,
///   which reproduces the boundary of both tables for every size.
/// - ISO 10642 shades them too, further down, because its length includes the head:
///   the first shanked length of each size is listed.
/// - ISO 7045:2011 threads pan heads to the head up to 25 mm for M3 and smaller and up
///   to 40 mm above.
/// - ISO 1580 and ISO 1207 thread slotted pan and cheese heads to the head up to 30 mm
///   for M3 and smaller and up to 40 mm above.
/// - ISO 2009 and ISO 7046-1 thread flat countersunk heads to the head up to 30 mm
///   for M3 and smaller and up to 45 mm above.
pub fn has_plain_shank(kind: HeadKind, nominal: f64, pitch: f64, length: f64) -> bool {
    let Some(b) = thread_length(kind, nominal) else {
        return false;
    };
    let small = nominal <= 3.0;
    match kind {
        HeadKind::Socket | HeadKind::Button => length > b + 8.5 * pitch,
        HeadKind::Countersunk => COUNTERSUNK_SHANK_FROM
            .iter()
            .find(|(size, _)| same_mm(*size, nominal))
            .is_some_and(|(_, from)| length >= *from),
        HeadKind::PanCross => length > if small { 25.0 } else { 40.0 },
        HeadKind::Pan | HeadKind::Cheese => length > if small { 30.0 } else { 40.0 },
        HeadKind::CountersunkFlat => length > if small { 30.0 } else { 45.0 },
        HeadKind::Hex => false,
    }
}

/// A countersunk head as a cone and the cylindrical land above it: `(cone height, land)`.
///
/// The cone is 90°, so it reaches `diameter` after `(diameter - major) / 2`. What is
/// left of `height` is the land. A head too low for that, such as a custom one, is a
/// single cone over the whole height, with no land.
pub fn countersink(major: f64, diameter: f64, height: f64) -> (f64, f64) {
    let cone = (diameter - major) / 2.0;
    // A land thinner than a nanometre is rounding, as on the ISO 10642 heads.
    if cone > 0.0 && cone < height - 1e-9 {
        (cone, height - cone)
    } else {
        (height, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_m3_socket_head_is_5_5_by_3() {
        let head = standard_head(HeadKind::Socket, 3.0).unwrap();
        assert_eq!(head.diameter, 5.5);
        assert_eq!(head.height, 3.0);
        assert!(!head.across_flats);
    }

    #[test]
    fn an_m3_hex_head_is_across_the_flats() {
        let head = standard_head(HeadKind::Hex, 3.0).unwrap();
        assert_eq!(head.diameter, 5.5);
        assert!(head.across_flats);
    }

    #[test]
    fn a_fine_pitch_still_uses_the_nominal_head() {
        assert!(standard_head(HeadKind::Button, 3.0).is_some());
        assert!(standard_head(HeadKind::Button, 3.5).is_none());
    }

    /// DIN 912, now DIN EN ISO 4762. Diameter and height are `dk` max and `k` max.
    /// Pitch is the coarse `P` from the same table.
    /// <https://www.fasteners.eu/standards/DIN/912/>
    #[test]
    fn socket_heads_match_din_912() {
        let din_912 = [
            (1.6, 3.0, 1.6, 0.35),
            (2.0, 3.8, 2.0, 0.4),
            (2.5, 4.5, 2.5, 0.45),
            (3.0, 5.5, 3.0, 0.5),
            (4.0, 7.0, 4.0, 0.7),
            (5.0, 8.5, 5.0, 0.8),
            (6.0, 10.0, 6.0, 1.0),
            (8.0, 13.0, 8.0, 1.25),
            (10.0, 16.0, 10.0, 1.5),
            (12.0, 18.0, 12.0, 1.75),
            (14.0, 21.0, 14.0, 2.0),
            (16.0, 24.0, 16.0, 2.0),
            (20.0, 30.0, 20.0, 2.5),
            (24.0, 36.0, 24.0, 3.0),
            (30.0, 45.0, 30.0, 3.5),
            (36.0, 54.0, 36.0, 4.0),
            (42.0, 63.0, 42.0, 4.5),
            (48.0, 72.0, 48.0, 5.0),
            (56.0, 84.0, 56.0, 5.5),
            (64.0, 96.0, 64.0, 6.0),
        ];
        assert_eq!(SOCKET.len(), din_912.len());
        for (nominal, diameter, height, pitch) in din_912 {
            let head = standard_head(HeadKind::Socket, nominal).unwrap();
            assert_eq!(head.diameter, diameter, "M{nominal} head diameter");
            assert_eq!(head.height, height, "M{nominal} head height");
            assert!(!head.across_flats, "M{nominal}");
            assert_eq!(
                crate::metric::coarse_pitch(nominal),
                Some(pitch),
                "M{nominal} pitch"
            );
        }
    }

    /// ISO 10642:2004 Table 1, `dk` theoretical max and `k` max.
    #[test]
    fn countersunk_heads_match_iso_10642_and_are_ninety_degree_cones() {
        let iso_10642 = [
            (3.0, 6.72, 1.86),
            (4.0, 8.96, 2.48),
            (5.0, 11.20, 3.10),
            (6.0, 13.44, 3.72),
            (8.0, 17.92, 4.96),
            (10.0, 22.40, 6.20),
            (12.0, 26.88, 7.44),
            (16.0, 33.60, 8.80),
            (20.0, 40.32, 10.16),
        ];
        assert_eq!(COUNTERSUNK.len(), iso_10642.len());
        for (nominal, diameter, height) in iso_10642 {
            let head = standard_head(HeadKind::Countersunk, nominal).unwrap();
            assert_eq!(
                (head.diameter, head.height),
                (diameter, height),
                "M{nominal}"
            );
            // 45° a side: the radius grows by the height.
            assert!(
                (diameter - nominal - 2.0 * height).abs() < 1e-9,
                "M{nominal}"
            );
        }
    }

    /// ISO 1207, <https://www.fasteners.eu/standards/ISO/1207/>, and the DIN 84 data
    /// sheet (DIN EN ISO 1207): `dk` and `k` nominal = max.
    #[test]
    fn cheese_heads_match_iso_1207() {
        let iso_1207 = [
            (1.6, 3.0, 1.1),
            (2.0, 3.8, 1.4),
            (2.5, 4.5, 1.8),
            (3.0, 5.5, 2.0),
            (3.5, 6.0, 2.4),
            (4.0, 7.0, 2.6),
            (5.0, 8.5, 3.3),
            (6.0, 10.0, 3.9),
            (8.0, 13.0, 5.0),
            (10.0, 16.0, 6.0),
        ];
        assert_eq!(CHEESE.len(), iso_1207.len());
        for (nominal, diameter, height) in iso_1207 {
            let head = standard_head(HeadKind::Cheese, nominal).unwrap();
            assert_eq!(
                (head.diameter, head.height),
                (diameter, height),
                "M{nominal}"
            );
            assert!(!head.across_flats);
        }
        assert!(standard_head(HeadKind::Cheese, 12.0).is_none());
    }

    /// ISO 7045:2011, Table 1: `dk` and `k` nominal = max.
    #[test]
    fn cross_recessed_pan_heads_match_iso_7045() {
        let iso_7045 = [
            (1.6, 3.2, 1.3),
            (2.0, 4.0, 1.6),
            (2.5, 5.0, 2.1),
            (3.0, 5.6, 2.4),
            (3.5, 7.0, 2.6),
            (4.0, 8.0, 3.1),
            (5.0, 9.5, 3.7),
            (6.0, 12.0, 4.6),
            (8.0, 16.0, 6.0),
            (10.0, 20.0, 7.5),
        ];
        assert_eq!(PAN_CROSS.len(), iso_7045.len());
        for (nominal, diameter, height) in iso_7045 {
            let head = standard_head(HeadKind::PanCross, nominal).unwrap();
            assert_eq!(
                (head.diameter, head.height),
                (diameter, height),
                "M{nominal}"
            );
        }
        // As wide as the slotted pan of the same thread, and taller.
        let slotted = standard_head(HeadKind::Pan, 3.0).unwrap();
        let cross = standard_head(HeadKind::PanCross, 3.0).unwrap();
        assert_eq!(cross.diameter, slotted.diameter);
        assert!(cross.height > slotted.height);
        // `rf` from the same table.
        let rf = [2.5, 3.2, 4.0, 5.0, 6.0, 6.5, 8.0, 10.0, 13.0, 16.0];
        for ((nominal, _, _), radius) in iso_7045.into_iter().zip(rf) {
            assert_eq!(crown(HeadKind::PanCross, nominal), Crown::Dome { radius });
        }
    }

    /// ISO 2009:2011 and ISO 7046-1:2011, Table 1, which agree: theoretical `dk` max,
    /// actual `dk` nominal = max and `k` nominal = max.
    #[test]
    fn flat_countersunk_heads_match_iso_2009_and_iso_7046() {
        let table = [
            (1.6, 3.6, 3.0, 1.0),
            (2.0, 4.4, 3.8, 1.2),
            (2.5, 5.5, 4.7, 1.5),
            (3.0, 6.3, 5.5, 1.65),
            (3.5, 8.2, 7.3, 2.35),
            (4.0, 9.4, 8.4, 2.7),
            (5.0, 10.4, 9.3, 2.7),
            (6.0, 12.6, 11.3, 3.3),
            (8.0, 17.3, 15.8, 4.65),
            (10.0, 20.0, 18.3, 5.0),
        ];
        assert_eq!(COUNTERSUNK_FLAT.len(), table.len());
        for (nominal, theoretical, diameter, height) in table {
            let head = standard_head(HeadKind::CountersunkFlat, nominal).unwrap();
            assert_eq!(
                (head.diameter, head.height),
                (diameter, height),
                "M{nominal}"
            );
            // The cone is 90°: carried to a sharp edge it is the theoretical diameter.
            assert!(
                (nominal + 2.0 * height - theoretical).abs() < 1e-9,
                "M{nominal}"
            );
            // It reaches the actual diameter below the top, so every size has a land.
            let (cone, land) = countersink(nominal, diameter, height);
            assert!(
                land > 0.0 && (cone + land - height).abs() < 1e-12,
                "M{nominal}"
            );
            assert!(
                (cone - (diameter - nominal) / 2.0).abs() < 1e-12,
                "M{nominal}"
            );
        }
        // The standards stop at M10.
        assert!(standard_head(HeadKind::CountersunkFlat, 12.0).is_none());
    }

    #[test]
    fn an_iso_10642_head_is_all_cone_and_a_low_custom_head_too() {
        let (cone, land) = countersink(3.0, 6.72, 1.86);
        assert_eq!((cone, land), (1.86, 0.0));
        assert_eq!(countersink(3.0, 8.0, 1.0), (1.0, 0.0));
        let (cone, land) = countersink(3.0, 5.6, 1.65);
        assert!(
            (cone - 1.3).abs() < 1e-9 && (land - 0.35).abs() < 1e-9,
            "{cone} {land}"
        );
    }

    /// ISO 1580, <https://www.fasteners.eu/standards/ISO/1580/> (the same table as
    /// `/DIN/85/`): `dk` and `k` nominal = max.
    #[test]
    fn slotted_pan_heads_match_iso_1580() {
        let iso_1580 = [
            (1.6, 3.2, 1.0),
            (2.0, 4.0, 1.3),
            (2.5, 5.0, 1.5),
            (3.0, 5.6, 1.8),
            (3.5, 7.0, 2.1),
            (4.0, 8.0, 2.4),
            (5.0, 9.5, 3.0),
            (6.0, 12.0, 3.6),
            (8.0, 16.0, 4.8),
            (10.0, 20.0, 6.0),
        ];
        assert_eq!(PAN.len(), iso_1580.len());
        for (nominal, diameter, height) in iso_1580 {
            let head = standard_head(HeadKind::Pan, nominal).unwrap();
            assert_eq!(
                (head.diameter, head.height),
                (diameter, height),
                "M{nominal}"
            );
        }
    }

    /// ISO 7380-1:2011 Table 1, `dk` max and `k` max.
    #[test]
    fn button_heads_match_iso_7380_1() {
        let iso_7380 = [
            (3.0, 5.7, 1.65),
            (4.0, 7.6, 2.2),
            (5.0, 9.5, 2.75),
            (6.0, 10.5, 3.3),
            (8.0, 14.0, 4.4),
            (10.0, 17.5, 5.5),
            (12.0, 21.0, 6.6),
            (16.0, 28.0, 8.8),
        ];
        assert_eq!(BUTTON.len(), iso_7380.len());
        for (nominal, diameter, height) in iso_7380 {
            let head = standard_head(HeadKind::Button, nominal).unwrap();
            assert_eq!(
                (head.diameter, head.height),
                (diameter, height),
                "M{nominal}"
            );
        }
    }

    #[test]
    fn a_button_profile_leans_in_from_the_bearing_face_to_its_flat() {
        let Crown::Button { radius, flat } = crown(HeadKind::Button, 3.0) else {
            panic!()
        };
        assert_eq!((radius, flat), (3.5, 2.6));
        let profile = profile(crown(HeadKind::Button, 3.0), 5.7, 1.65).unwrap();
        let Segment::Arc {
            start,
            through,
            end,
        } = profile[1].clone()
        else {
            panic!()
        };
        assert_eq!(start, [2.85, 0.0]);
        assert_eq!(end, [1.3, 1.65]);
        // The middle of the arc bulges outside the straight chord, and stays inside dk.
        assert!(
            through[0] > (2.85 + 1.3) / 2.0 && through[0] < 2.85,
            "{through:?}"
        );
        assert!(through[1] > 0.0 && through[1] < 1.65);
        // All three points are `rf` from one centre.
        let centre = [-0.3388794552318144, -1.4425837306723108];
        for p in [start, through, end] {
            let d = ((p[0] - centre[0]).powi(2) + (p[1] - centre[1]).powi(2)).sqrt();
            assert!((d - 3.5).abs() < 1e-9, "{d}");
        }
        assert_eq!(profile[0], Segment::Line(vec![[0.0, 0.0], [2.85, 0.0]]));
    }

    #[test]
    fn an_iso_7045_dome_keeps_its_cylindrical_side() {
        // M3: rf 5 over a 5.6 mm head rises 0.86, which leaves a 1.54 mm side.
        let profile = profile(crown(HeadKind::PanCross, 3.0), 5.6, 2.4).unwrap();
        let Segment::Line(side) = profile[0].clone() else {
            panic!()
        };
        assert!((side[2][1] - 1.542).abs() < 0.005, "{side:?}");
        let Segment::Arc { through, end, .. } = profile[1].clone() else {
            panic!()
        };
        assert_eq!(end, [0.0, 2.4]);
        let d = (through[0].powi(2) + (through[1] - (2.4 - 5.0)).powi(2)).sqrt();
        assert!((d - 5.0).abs() < 1e-9);
    }

    #[test]
    fn an_iso_1580_pan_rounds_its_top_edge() {
        assert_eq!(crown(HeadKind::Pan, 3.0), Crown::Rounded { radius: 0.9 });
        let profile = profile(Crown::Rounded { radius: 0.9 }, 5.6, 1.8).unwrap();
        let Segment::Arc { start, end, .. } = profile[1].clone() else {
            panic!()
        };
        assert_eq!(start, [2.8, 0.9]);
        assert!((end[0] - 1.9).abs() < 1e-12 && end[1] == 1.8);
        // A crown that does not fit, and a plain head, have no profile.
        assert!(super::profile(Crown::Rounded { radius: 2.0 }, 5.6, 1.8).is_none());
        assert!(super::profile(Crown::Flat, 5.5, 3.0).is_none());
        assert_eq!(crown(HeadKind::Socket, 3.0), Crown::Flat);
        assert_eq!(crown(HeadKind::Button, 2.0), Crown::Flat);
    }

    /// `b` from the DIN 912 data sheet, ISO 7380-1:2011 and ISO 10642:2004 (`2d + 12`),
    /// and from ISO 7045:2011, ISO 1580:2011, ISO 1207:2011, ISO 2009:2011 and
    /// ISO 7046-1:2011 (25 mm to M3, 38 mm above).
    #[test]
    fn thread_lengths_match_the_b_rows() {
        let din_912 = [
            (1.6, 15.0),
            (2.0, 16.0),
            (2.5, 17.0),
            (3.0, 18.0),
            (4.0, 20.0),
            (5.0, 22.0),
            (6.0, 24.0),
            (8.0, 28.0),
            (10.0, 32.0),
            (12.0, 36.0),
            (14.0, 40.0),
            (16.0, 44.0),
            (20.0, 52.0),
            (24.0, 60.0),
            (30.0, 72.0),
            (36.0, 84.0),
            (42.0, 96.0),
            (48.0, 108.0),
            (56.0, 124.0),
            (64.0, 140.0),
        ];
        for (d, b) in din_912 {
            assert_eq!(thread_length(HeadKind::Socket, d), Some(b), "M{d}");
        }
        for (d, b) in [
            (3.0, 18.0),
            (4.0, 20.0),
            (5.0, 22.0),
            (6.0, 24.0),
            (8.0, 28.0),
            (10.0, 32.0),
            (12.0, 36.0),
            (16.0, 44.0),
        ] {
            assert_eq!(thread_length(HeadKind::Button, d), Some(b), "M{d}");
            assert_eq!(thread_length(HeadKind::Countersunk, d), Some(b), "M{d}");
        }
        assert_eq!(thread_length(HeadKind::Countersunk, 20.0), Some(52.0));
        let small_and_large = [
            HeadKind::PanCross,
            HeadKind::Pan,
            HeadKind::Cheese,
            HeadKind::CountersunkFlat,
        ];
        for kind in small_and_large {
            for d in [1.6, 2.0, 2.5, 3.0] {
                assert_eq!(thread_length(kind, d), Some(25.0), "{kind:?} M{d}");
            }
            for d in [3.5, 4.0, 5.0, 6.0, 8.0, 10.0] {
                assert_eq!(thread_length(kind, d), Some(38.0), "{kind:?} M{d}");
            }
            assert_eq!(thread_length(kind, 12.0), None, "{kind:?}");
        }
        assert_eq!(thread_length(HeadKind::Hex, 6.0), None);
    }

    /// The DIN 912 data sheet marks each length as "full thread" or gives `ls` and
    /// `lg`, and ISO 7380-1:2011 shades the same lengths. These are the last length
    /// threaded to the head and the first shanked one for each size.
    #[test]
    fn socket_and_button_shanks_start_where_din_912_and_iso_7380_do() {
        let din_912 = [
            (2.0, 0.4, 16.0, 20.0),
            (2.5, 0.45, 20.0, 25.0),
            (3.0, 0.5, 20.0, 25.0),
            (4.0, 0.7, 25.0, 30.0),
            (5.0, 0.8, 25.0, 30.0),
            (6.0, 1.0, 30.0, 35.0),
            (8.0, 1.25, 35.0, 40.0),
            (10.0, 1.5, 40.0, 45.0),
            (12.0, 1.75, 50.0, 55.0),
            (14.0, 2.0, 55.0, 60.0),
            (16.0, 2.0, 60.0, 65.0),
            (20.0, 2.5, 70.0, 80.0),
            (24.0, 3.0, 80.0, 90.0),
            (30.0, 3.5, 100.0, 110.0),
            (36.0, 4.0, 110.0, 120.0),
            (42.0, 4.5, 130.0, 140.0),
            (48.0, 5.0, 150.0, 160.0),
            (56.0, 5.5, 160.0, 180.0),
            (64.0, 6.0, 180.0, 200.0),
        ];
        for (d, pitch, full, shanked) in din_912 {
            assert!(
                !has_plain_shank(HeadKind::Socket, d, pitch, full),
                "M{d} x {full}"
            );
            assert!(
                has_plain_shank(HeadKind::Socket, d, pitch, shanked),
                "M{d} x {shanked}"
            );
        }
        let iso_7380 = [
            (3.0, 0.5, 20.0, 25.0),
            (4.0, 0.7, 25.0, 30.0),
            (5.0, 0.8, 25.0, 30.0),
            (6.0, 1.0, 30.0, 35.0),
            (8.0, 1.25, 35.0, 40.0),
            (10.0, 1.5, 40.0, 45.0),
            (12.0, 1.75, 50.0, 55.0),
            (16.0, 2.0, 60.0, 65.0),
        ];
        for (d, pitch, full, shanked) in iso_7380 {
            assert!(
                !has_plain_shank(HeadKind::Button, d, pitch, full),
                "M{d} x {full}"
            );
            assert!(
                has_plain_shank(HeadKind::Button, d, pitch, shanked),
                "M{d} x {shanked}"
            );
        }
    }

    /// ISO 10642:2004 Table 1: the last length in the shaded area and the first with
    /// `ls` and `lg`, for each size.
    #[test]
    fn countersunk_socket_shanks_start_where_iso_10642_does() {
        let iso_10642 = [
            (3.0, 0.5, 25.0, 30.0),
            (4.0, 0.7, 25.0, 30.0),
            (5.0, 0.8, 30.0, 35.0),
            (6.0, 1.0, 35.0, 40.0),
            (8.0, 1.25, 45.0, 50.0),
            (10.0, 1.5, 50.0, 55.0),
            (12.0, 1.75, 60.0, 65.0),
            (16.0, 2.0, 70.0, 80.0),
            (20.0, 2.5, 90.0, 100.0),
        ];
        for (d, pitch, full, shanked) in iso_10642 {
            assert!(
                !has_plain_shank(HeadKind::Countersunk, d, pitch, full),
                "M{d} x {full}"
            );
            assert!(
                has_plain_shank(HeadKind::Countersunk, d, pitch, shanked),
                "M{d} x {shanked}"
            );
        }
    }

    /// The discontinuous line in Table 1 of ISO 7045:2011, ISO 1580:2011,
    /// ISO 1207:2011, ISO 2009:2011 and ISO 7046-1:2011: lengths above it are threaded
    /// to the head. `(last length above the line, first below)` for M3 and smaller,
    /// then for the larger sizes.
    #[test]
    fn slotted_and_cross_recessed_shanks_start_below_the_discontinuous_line() {
        let lines = [
            (HeadKind::PanCross, (25.0, 30.0), (40.0, 45.0)),
            (HeadKind::Pan, (30.0, 35.0), (40.0, 45.0)),
            (HeadKind::Cheese, (30.0, 35.0), (40.0, 45.0)),
            (HeadKind::CountersunkFlat, (30.0, 35.0), (45.0, 50.0)),
        ];
        for (kind, small, large) in lines {
            for d in [1.6, 2.0, 2.5, 3.0] {
                assert!(!has_plain_shank(kind, d, 0.5, small.0), "{kind:?} M{d}");
                assert!(has_plain_shank(kind, d, 0.5, small.1), "{kind:?} M{d}");
            }
            for d in [3.5, 4.0, 5.0, 6.0, 8.0, 10.0] {
                assert!(!has_plain_shank(kind, d, 1.0, large.0), "{kind:?} M{d}");
                assert!(has_plain_shank(kind, d, 1.0, large.1), "{kind:?} M{d}");
            }
        }
        assert!(!has_plain_shank(HeadKind::Hex, 6.0, 1.0, 100.0));
    }
    /// ISO 4017, <https://www.fasteners.eu/standards/ISO/4017/>: `s` nominal = max and
    /// `k` nominal, the preferred sizes and then the non-preferred ones.
    #[test]
    fn hex_heads_match_iso_4017() {
        let preferred = [
            (1.6, 3.2, 1.1),
            (2.0, 4.0, 1.4),
            (2.5, 5.0, 1.7),
            (3.0, 5.5, 2.0),
            (4.0, 7.0, 2.8),
            (5.0, 8.0, 3.5),
            (6.0, 10.0, 4.0),
            (8.0, 13.0, 5.3),
            (10.0, 16.0, 6.4),
            (12.0, 18.0, 7.5),
            (16.0, 24.0, 10.0),
            (20.0, 30.0, 12.5),
            (24.0, 36.0, 15.0),
            (30.0, 46.0, 18.7),
            (36.0, 55.0, 22.5),
            (42.0, 65.0, 26.0),
            (48.0, 75.0, 30.0),
            (56.0, 85.0, 35.0),
            (64.0, 95.0, 40.0),
        ];
        let non_preferred = [
            (3.5, 6.0, 2.4),
            (14.0, 21.0, 8.8),
            (18.0, 27.0, 11.5),
            (22.0, 34.0, 14.0),
            (27.0, 41.0, 17.0),
            (33.0, 50.0, 21.0),
            (39.0, 60.0, 25.0),
            (45.0, 70.0, 28.0),
            (52.0, 80.0, 33.0),
            (60.0, 90.0, 38.0),
        ];
        assert_eq!(HEX.len(), preferred.len() + non_preferred.len());
        for (nominal, across_flats, height) in preferred.into_iter().chain(non_preferred) {
            let head = standard_head(HeadKind::Hex, nominal).unwrap();
            assert_eq!(
                (head.diameter, head.height),
                (across_flats, height),
                "M{nominal}"
            );
            assert!(head.across_flats);
        }
    }
}
