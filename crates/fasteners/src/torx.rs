//! ISO 10664 hexalobular internal drive, the recess called Torx.
//!
//! Table 1 gives the nominal across-tips `A` and across-valleys `B`. Annex A draws the
//! contour for a CAD system: the lobe radius is `re ≈ 0.1 A`, the valley radius is
//! `ri ≈ 0.175 A`, and those arcs meet on a common tangent. The contour here keeps
//! Table 1 `A` and `B` and `re = 0.1 A`, and solves `ri` so the arcs stay tangent.
//! That `ri` stays near `0.175 A` because Table 1 `B` is about `0.72 A`, which is the
//! annex ratio.
//!
//! ISO 10664 leaves the penetration depth to the product standard. The depth used here
//! is the ISO 14579 minimum `t` for the socket-head size that takes that socket.
//! Sockets 15, 40 and 100 are not in that table; their depth is `0.4 A`, inside the
//! band of those minima.
//!
//! Sources: ISO 10664:1999 for Table 1 (`A` and `B`) and annex A (`B ≈ 0.72 A`,
//! `re ≈ 0.1 A`, `ri ≈ 0.175 A`), and ISO 14579,
//! <https://www.fasteners.eu/standards/ISO/14579/>, for the socket of each thread and
//! its penetration. ISO 10664 also gives a counterbore at the mouth, 0.13 mm up to
//! socket 15 and 0.25 mm above, which is not cut, and leaves the bottom of the socket
//! to the maker.

use crate::metric::same_mm;
use plugin_kit::FieldError;

/// One ISO 10664 socket. Dimensions are millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Socket {
    pub number: u16,
    /// Across opposite lobe tips. Table 1 `A`.
    pub across_tips: f64,
    /// Across opposite valley bottoms. Table 1 `B`.
    pub across_valleys: f64,
    /// ISO 14579 minimum penetration, when a socket-head screw uses this socket.
    penetration: Option<f64>,
}

/// `(socket number, A, B, ISO 14579 t min or none)`.
const SOCKETS: &[Socket] = &[
    Socket {
        number: 6,
        across_tips: 1.75,
        across_valleys: 1.27,
        penetration: Some(0.71),
    },
    Socket {
        number: 8,
        across_tips: 2.40,
        across_valleys: 1.75,
        penetration: Some(0.91),
    },
    Socket {
        number: 10,
        across_tips: 2.80,
        across_valleys: 2.05,
        penetration: Some(1.01),
    },
    Socket {
        number: 15,
        across_tips: 3.35,
        across_valleys: 2.40,
        penetration: None,
    },
    Socket {
        number: 20,
        across_tips: 3.95,
        across_valleys: 2.85,
        penetration: Some(1.42),
    },
    Socket {
        number: 25,
        across_tips: 4.50,
        across_valleys: 3.25,
        penetration: Some(1.65),
    },
    Socket {
        number: 30,
        across_tips: 5.60,
        across_valleys: 4.05,
        penetration: Some(2.02),
    },
    Socket {
        number: 40,
        across_tips: 6.75,
        across_valleys: 4.85,
        penetration: None,
    },
    Socket {
        number: 45,
        across_tips: 7.93,
        across_valleys: 5.64,
        penetration: Some(2.92),
    },
    Socket {
        number: 50,
        across_tips: 8.95,
        across_valleys: 6.45,
        penetration: Some(3.62),
    },
    Socket {
        number: 55,
        across_tips: 11.35,
        across_valleys: 8.05,
        penetration: Some(4.82),
    },
    Socket {
        number: 60,
        across_tips: 13.45,
        across_valleys: 9.60,
        penetration: Some(5.62),
    },
    Socket {
        number: 70,
        across_tips: 15.70,
        across_valleys: 11.20,
        penetration: Some(6.62),
    },
    Socket {
        number: 80,
        across_tips: 17.75,
        across_valleys: 12.80,
        penetration: Some(7.5),
    },
    Socket {
        number: 90,
        across_tips: 20.20,
        across_valleys: 14.40,
        penetration: Some(8.69),
    },
    Socket {
        number: 100,
        across_tips: 22.40,
        across_valleys: 16.00,
        penetration: None,
    },
];

/// ISO 14579 socket for a nominal thread diameter. `(nominal, socket number)`.
const FOR_THREAD: &[(f64, u16)] = &[
    (2.0, 6),
    (2.5, 8),
    (3.0, 10),
    (4.0, 20),
    (5.0, 25),
    (6.0, 30),
    (8.0, 45),
    (10.0, 50),
    (12.0, 55),
    (14.0, 60),
    (16.0, 70),
    (18.0, 80),
    (20.0, 90),
];

/// Depth ratio for a socket ISO 14579 does not list. `t min / A` on that table is about 0.36 to 0.43.
const UNTABULATED_DEPTH: f64 = 0.4;

const BAD_SOCKET: &str = "must be an ISO 10664 socket: T6, T8, T10, T15, T20, T25, T30, T40, T45, T50, T55, T60, T70, T80, T90 or T100";

impl Socket {
    /// How deep the recess goes. ISO 14579 `t` min, or `0.4 A` when that table has no row.
    pub fn penetration(self) -> f64 {
        self.penetration
            .unwrap_or(UNTABULATED_DEPTH * self.across_tips)
    }
}

pub fn sockets() -> &'static [Socket] {
    SOCKETS
}

pub fn socket(number: u16) -> Option<&'static Socket> {
    SOCKETS.iter().find(|socket| socket.number == number)
}

/// The ISO 14579 socket for this nominal diameter. A fine pitch uses the same nominal.
pub fn for_nominal(nominal: f64) -> Option<u16> {
    FOR_THREAD
        .iter()
        .find(|(size, _)| same_mm(*size, nominal))
        .map(|(_, number)| *number)
}

/// `T10`, `10`, or the designation `ISO 10664 - 20`.
pub fn parse_socket(text: &str) -> Result<u16, FieldError> {
    let compact: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();
    let rest = compact
        .strip_prefix("iso10664-")
        .or_else(|| compact.strip_prefix("iso10664"))
        .unwrap_or(compact.as_str());
    let rest = rest.strip_prefix('t').unwrap_or(rest);
    let number = rest
        .parse::<u16>()
        .ok()
        .filter(|number| socket(*number).is_some())
        .ok_or_else(|| FieldError::new("torx", BAD_SOCKET))?;
    Ok(number)
}

/// A recess ready to cut. `depth` is how far it reaches into the screw.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Recess {
    pub number: u16,
    pub depth: f64,
}

/// One arc of the recess, in the sketch plane. The profile centre is the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arc {
    pub start: [f64; 2],
    pub through: [f64; 2],
    pub end: [f64; 2],
}

/// Twelve arcs, lobe then valley, counter-clockwise from the +X lobe.
#[derive(Clone, Debug, PartialEq)]
pub struct Contour {
    pub socket: Socket,
    /// Lobe radius. Annex A `re`, exactly `0.1 A`.
    pub re: f64,
    /// Valley radius solved so the valley diameter is Table 1 `B`.
    pub ri: f64,
    pub arcs: Vec<Arc>,
}

pub fn contour(number: u16) -> Option<Contour> {
    let socket = *socket(number)?;
    Some(contour_of(socket))
}

fn contour_of(socket: Socket) -> Contour {
    let a = socket.across_tips;
    let b = socket.across_valleys;
    let re = 0.1 * a;
    let ri = valley_radius(a, b);
    let lobe_center = a / 2.0 - re;
    let valley_center = b / 2.0 + ri;
    // Six lobes. The valley between two lobes sits 30° off each of them.
    let mut tangents = [[[0.0; 2]; 2]; 6];
    for (i, slot) in tangents.iter_mut().enumerate() {
        let lobe = i as f64 * std::f64::consts::FRAC_PI_3;
        let center = polar(lobe_center, lobe);
        *slot = [
            tangent(
                center,
                polar(valley_center, lobe - std::f64::consts::FRAC_PI_6),
                re,
            ),
            tangent(
                center,
                polar(valley_center, lobe + std::f64::consts::FRAC_PI_6),
                re,
            ),
        ];
    }
    let mut arcs = Vec::with_capacity(12);
    for i in 0..6 {
        let lobe = i as f64 * std::f64::consts::FRAC_PI_3;
        let next = (i + 1) % 6;
        arcs.push(Arc {
            start: tangents[i][0],
            through: polar(a / 2.0, lobe),
            end: tangents[i][1],
        });
        arcs.push(Arc {
            start: tangents[i][1],
            through: polar(b / 2.0, lobe + std::f64::consts::FRAC_PI_6),
            end: tangents[next][0],
        });
    }
    Contour {
        socket,
        re,
        ri,
        arcs,
    }
}

/// `ri` such that a valley circle of that radius is tangent to both neighbouring lobe circles
/// and its inner point is at `B / 2`.
fn valley_radius(a: f64, b: f64) -> f64 {
    let re = 0.1 * a;
    let lobe_center = a / 2.0 - re;
    let valley = b / 2.0;
    let cos30 = 3.0_f64.sqrt() / 2.0;
    // Law of cosines at the origin, with the centres 30° apart and `re + ri` between them.
    let num =
        lobe_center * lobe_center + valley * valley - 2.0 * lobe_center * cos30 * valley - re * re;
    let den = 2.0 * re - 2.0 * valley + 2.0 * lobe_center * cos30;
    num / den
}

fn polar(radius: f64, angle: f64) -> [f64; 2] {
    [radius * angle.cos(), radius * angle.sin()]
}

/// Point on the lobe circle, toward the valley centre, one lobe radius out.
fn tangent(lobe_center: [f64; 2], valley_center: [f64; 2], re: f64) -> [f64; 2] {
    let dx = valley_center[0] - lobe_center[0];
    let dy = valley_center[1] - lobe_center[1];
    let dist = (dx * dx + dy * dy).sqrt();
    let scale = re / dist;
    [lobe_center[0] + dx * scale, lobe_center[1] + dy * scale]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ISO 10664:1999 Table 1: socket number, `A` and `B`.
    #[test]
    fn table_1_is_the_sixteen_sockets() {
        let expected = [
            (6, 1.75, 1.27),
            (8, 2.40, 1.75),
            (10, 2.80, 2.05),
            (15, 3.35, 2.40),
            (20, 3.95, 2.85),
            (25, 4.50, 3.25),
            (30, 5.60, 4.05),
            (40, 6.75, 4.85),
            (45, 7.93, 5.64),
            (50, 8.95, 6.45),
            (55, 11.35, 8.05),
            (60, 13.45, 9.60),
            (70, 15.70, 11.20),
            (80, 17.75, 12.80),
            (90, 20.20, 14.40),
            (100, 22.40, 16.00),
        ];
        assert_eq!(SOCKETS.len(), expected.len());
        for (socket, (number, a, b)) in SOCKETS.iter().zip(expected) {
            assert_eq!(socket.number, number);
            assert_eq!(socket.across_tips, a);
            assert_eq!(socket.across_valleys, b);
            let ratio = b / a;
            assert!((0.70..=0.74).contains(&ratio), "T{number} B/A is {ratio}");
        }
    }

    #[test]
    fn an_m3_is_t10_and_an_m8_is_t45() {
        assert_eq!(for_nominal(3.0), Some(10));
        assert_eq!(for_nominal(8.0), Some(45));
        assert_eq!(for_nominal(14.0), Some(60));
        assert_eq!(for_nominal(20.0), Some(90));
        assert_eq!(for_nominal(1.6), None);
        assert_eq!(for_nominal(24.0), None);
    }

    #[test]
    fn penetration_follows_iso_14579_and_the_untabulated_ratio() {
        assert_eq!(socket(10).unwrap().penetration(), 1.01);
        assert_eq!(socket(45).unwrap().penetration(), 2.92);
        assert_eq!(socket(90).unwrap().penetration(), 8.69);
        for number in [15, 40, 100] {
            let socket = socket(number).unwrap();
            assert!(socket.penetration.is_none(), "T{number}");
            assert_eq!(socket.penetration(), 0.4 * socket.across_tips);
        }
    }

    #[test]
    fn a_socket_name_accepts_the_designation() {
        assert_eq!(parse_socket("T10").unwrap(), 10);
        assert_eq!(parse_socket("t20").unwrap(), 20);
        assert_eq!(parse_socket("15").unwrap(), 15);
        assert_eq!(parse_socket("ISO 10664 - 20").unwrap(), 20);
        assert_eq!(parse_socket("T7").unwrap_err().field, "torx");
        assert_eq!(parse_socket("T").unwrap_err().field, "torx");
    }

    /// Annex A: `re = 0.1 A`, `ri = 0.175 A`, centres 30° apart and externally tangent.
    /// The valley diameter that falls out is `0.72 A`.
    #[test]
    fn annex_a_ratios_put_the_valley_at_0_72_a() {
        let a = 10.0;
        let re = 0.1 * a;
        let ri = 0.175 * a;
        let lobe = a / 2.0 - re;
        let dist = re + ri;
        let cos30 = 3.0_f64.sqrt() / 2.0;
        let disc = (2.0 * lobe * cos30).powi(2) - 4.0 * (lobe * lobe - dist * dist);
        let center = (2.0 * lobe * cos30 + disc.sqrt()) / 2.0;
        let valley = 2.0 * (center - ri);
        assert!((valley / a - 0.72).abs() < 0.001, "{valley}");
    }

    #[test]
    fn every_contour_hits_a_and_b_and_meets_on_a_tangent() {
        for socket in SOCKETS {
            let contour = contour(socket.number).unwrap();
            assert_eq!(contour.arcs.len(), 12, "T{}", socket.number);
            assert_eq!(contour.re, 0.1 * socket.across_tips);
            let ratio = contour.ri / socket.across_tips;
            assert!(
                (0.16..=0.20).contains(&ratio),
                "T{} ri/A is {ratio}",
                socket.number
            );
            let lobe = &contour.arcs[0];
            assert!(
                (lobe.through[0] - socket.across_tips / 2.0).abs() < 1e-9
                    && lobe.through[1].abs() < 1e-9,
                "T{} tip",
                socket.number
            );
            let valley = &contour.arcs[1];
            let valley_r = (valley.through[0].powi(2) + valley.through[1].powi(2)).sqrt();
            assert!(
                (valley_r - socket.across_valleys / 2.0).abs() < 1e-9,
                "T{} valley",
                socket.number
            );
            assert_eq!(contour.arcs.last().unwrap().end, contour.arcs[0].start);
            for pair in contour.arcs.windows(2) {
                assert_eq!(pair[0].end, pair[1].start, "T{}", socket.number);
            }
            for arc in &contour.arcs {
                assert!(on_minor_arc(arc), "T{} arc folds back", socket.number);
            }
        }
    }

    fn on_minor_arc(arc: &Arc) -> bool {
        // The circle through the three points, and the through-point on the shorter turn.
        let (center, radius) = circle(arc.start, arc.through, arc.end);
        let sweep = turn(center, arc.start, arc.through) + turn(center, arc.through, arc.end);
        let direct = turn(center, arc.start, arc.end);
        radius.is_finite() && (sweep - direct).abs() < 1e-6 && direct < std::f64::consts::PI
    }

    fn circle(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> ([f64; 2], f64) {
        let d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
        let a2 = a[0] * a[0] + a[1] * a[1];
        let b2 = b[0] * b[0] + b[1] * b[1];
        let c2 = c[0] * c[0] + c[1] * c[1];
        let ux = (a2 * (b[1] - c[1]) + b2 * (c[1] - a[1]) + c2 * (a[1] - b[1])) / d;
        let uy = (a2 * (c[0] - b[0]) + b2 * (a[0] - c[0]) + c2 * (b[0] - a[0])) / d;
        let center = [ux, uy];
        let radius = ((a[0] - ux).powi(2) + (a[1] - uy).powi(2)).sqrt();
        (center, radius)
    }

    fn turn(center: [f64; 2], from: [f64; 2], to: [f64; 2]) -> f64 {
        let ax = from[0] - center[0];
        let ay = from[1] - center[1];
        let bx = to[0] - center[0];
        let by = to[1] - center[1];
        let dot = ax * bx + ay * by;
        let cross = ax * by - ay * bx;
        cross.atan2(dot).abs()
    }
    /// ISO 14579, <https://www.fasteners.eu/standards/ISO/14579/>: the socket for each
    /// thread, its `A` ref and its `t` min. The page rounds T45 to 7.95; ISO 10664
    /// gives 7.93.
    #[test]
    fn sockets_and_depths_match_iso_14579() {
        let iso_14579 = [
            (2.0, 6, 1.75, 0.71),
            (2.5, 8, 2.4, 0.91),
            (3.0, 10, 2.8, 1.01),
            (4.0, 20, 3.95, 1.42),
            (5.0, 25, 4.5, 1.65),
            (6.0, 30, 5.6, 2.02),
            (8.0, 45, 7.95, 2.92),
            (10.0, 50, 8.95, 3.62),
            (12.0, 55, 11.35, 4.82),
            (14.0, 60, 13.45, 5.62),
            (16.0, 70, 15.7, 6.62),
            (18.0, 80, 17.75, 7.5),
            (20.0, 90, 20.2, 8.69),
        ];
        assert_eq!(FOR_THREAD.len(), iso_14579.len());
        for (d, number, a, t) in iso_14579 {
            assert_eq!(for_nominal(d), Some(number), "M{d}");
            let socket = socket(number).unwrap();
            assert!((socket.across_tips - a).abs() < 0.021, "T{number}");
            assert_eq!(socket.penetration(), t, "T{number}");
        }
    }
}
