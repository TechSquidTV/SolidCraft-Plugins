//! Hexagon socket, the Allen drive.
//!
//! Each product standard gives the socket for its own head: the width across the flats
//! `s` and the least depth `t`. The cut is the nominal `s` and that least depth, with a
//! flat floor. The drill point under a real socket and the rounding at the mouth are
//! not cut.
//!
//! A socket head cap screw follows DIN 912 (ISO 4762), a button head ISO 7380-1, a
//! countersunk head ISO 10642 and a set screw ISO 4026.
//!
//! Sources, all checked value for value:
//! - `CAP`: ARAMFIX data sheet "DIN 912", <https://www.fasteners.eu/standards/DIN/912/>
//! - `BUTTON`: ISO 7380-1:2011, Table 1
//! - `COUNTERSUNK`: ISO 10642:2004, Table 1
//! - `CUP_SET`: ISO 4029:2003, Table 1 and Figure 1
//! - `SET`: ISO 4026:2003, Table 1. <https://www.fasteners.eu/standards/iso/4026/>
//!   (DIN 913) agrees for all thirteen sizes, and adds M1.4, M1.8, M14, M18 and M22,
//!   which ISO 4026:2003 does not list and which are not here.

use crate::input::millimetres;
use crate::metric::same_mm;
use crate::FieldError;

/// A socket ready to cut. Dimensions are millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Socket {
    /// Nominal width across the flats `s`, which is the key size.
    pub across_flats: f64,
    /// Depth from the outer face.
    pub depth: f64,
}

impl Socket {
    /// Width across the corners, `s × 2 / √3`.
    pub fn across_corners(self) -> f64 {
        self.across_flats * 2.0 / 3.0_f64.sqrt()
    }

    /// The six corners, the first on the x axis and the rest counter-clockwise. The
    /// caller closes the outline, so the first corner is not repeated.
    pub fn corners(self) -> [[f64; 2]; 6] {
        let r = self.across_corners() / 2.0;
        let mut out = [[0.0; 2]; 6];
        for (i, corner) in out.iter_mut().enumerate() {
            let angle = (i as f64) * std::f64::consts::FRAC_PI_3;
            *corner = [r * angle.cos(), r * angle.sin()];
        }
        out
    }
}

/// Which product standard sizes the socket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// DIN 912 / ISO 4762 socket head cap screw.
    Cap,
    /// ISO 7380-1 button head.
    Button,
    /// ISO 10642 countersunk head.
    Countersunk,
    /// ISO 4026 set screw.
    Set,
}

/// DIN 912 / ISO 4762. `(nominal, s, t min)`.
const CAP: &[(f64, f64, f64)] = &[
    (1.6, 1.5, 0.7),
    (2.0, 1.5, 1.0),
    (2.5, 2.0, 1.1),
    (3.0, 2.5, 1.3),
    (4.0, 3.0, 2.0),
    (5.0, 4.0, 2.5),
    (6.0, 5.0, 3.0),
    (8.0, 6.0, 4.0),
    (10.0, 8.0, 5.0),
    (12.0, 10.0, 6.0),
    (14.0, 12.0, 7.0),
    (16.0, 14.0, 8.0),
    (20.0, 17.0, 10.0),
    (24.0, 19.0, 12.0),
    (30.0, 22.0, 15.5),
    (36.0, 27.0, 19.0),
    (42.0, 32.0, 24.0),
    (48.0, 36.0, 28.0),
    (56.0, 41.0, 34.0),
    (64.0, 46.0, 38.0),
];

/// ISO 7380-1. `(nominal, s, t min)`.
const BUTTON: &[(f64, f64, f64)] = &[
    (3.0, 2.0, 1.04),
    (4.0, 2.5, 1.3),
    (5.0, 3.0, 1.56),
    (6.0, 4.0, 2.08),
    (8.0, 5.0, 2.6),
    (10.0, 6.0, 3.12),
    (12.0, 8.0, 4.16),
    (16.0, 10.0, 5.2),
];

/// ISO 10642. `(nominal, s, t min)`.
const COUNTERSUNK: &[(f64, f64, f64)] = &[
    (3.0, 2.0, 1.1),
    (4.0, 2.5, 1.5),
    (5.0, 3.0, 1.9),
    (6.0, 4.0, 2.2),
    (8.0, 5.0, 3.0),
    (10.0, 6.0, 3.6),
    (12.0, 8.0, 4.3),
    (14.0, 10.0, 4.5),
    (16.0, 10.0, 4.8),
    (20.0, 12.0, 5.6),
];

/// ISO 4026:2003 Table 1. `(nominal, s, t min short, t min long, shaded up to, dp max)`.
/// The standard gives two socket depths: the shallower one for the short lengths it
/// shades in the table, up to the length in the fifth column, and the deeper one for
/// every longer screw. `dp` is the flat left on the point.
const SET: &[SetRow] = &[
    (1.6, 0.7, 0.7, 1.5, 2.0, 0.8),
    (2.0, 0.9, 0.8, 1.7, 3.0, 1.0),
    (2.5, 1.3, 1.2, 2.0, 3.0, 1.5),
    (3.0, 1.5, 1.2, 2.0, 3.0, 2.0),
    (4.0, 2.0, 1.5, 2.5, 4.0, 2.5),
    (5.0, 2.5, 2.0, 3.0, 5.0, 3.5),
    (6.0, 3.0, 2.0, 3.5, 6.0, 4.0),
    (8.0, 4.0, 3.0, 5.0, 8.0, 5.5),
    (10.0, 5.0, 4.0, 6.0, 10.0, 7.0),
    (12.0, 6.0, 4.8, 8.0, 12.0, 8.5),
    (16.0, 8.0, 6.4, 10.0, 16.0, 12.0),
    (20.0, 10.0, 8.0, 12.0, 20.0, 15.0),
    (24.0, 12.0, 10.0, 15.0, 25.0, 18.0),
];

/// The standard socket for this head and nominal diameter. A fine pitch uses the same
/// nominal. A set screw gets the shallower of its two depths; `set_screw` picks by length.
pub fn standard(kind: Kind, nominal: f64) -> Option<Socket> {
    let table = match kind {
        Kind::Cap => CAP,
        Kind::Button => BUTTON,
        Kind::Countersunk => COUNTERSUNK,
        Kind::Set => {
            return set_row(nominal).map(|row| Socket {
                across_flats: row.1,
                depth: row.2,
            })
        }
    };
    table
        .iter()
        .find(|(size, _, _)| same_mm(*size, nominal))
        .map(|(_, across_flats, depth)| Socket {
            across_flats: *across_flats,
            depth: *depth,
        })
}

type SetRow = (f64, f64, f64, f64, f64, f64);

fn set_row(nominal: f64) -> Option<&'static SetRow> {
    SET.iter().find(|row| same_mm(row.0, nominal))
}

/// ISO 4029:2003 Table 1, the cup point set screw.
/// `(nominal, s, t min short, t min long, shaded up to, dz max)`. The sockets are those
/// of ISO 4026, but the standard shades different lengths, and `dz` is the mouth of
/// the cup.
const CUP_SET: &[SetRow] = &[
    (1.6, 0.7, 0.7, 1.5, 2.0, 0.8),
    (2.0, 0.9, 0.8, 1.7, 2.5, 1.0),
    (2.5, 1.3, 1.2, 2.0, 3.0, 1.2),
    (3.0, 1.5, 1.2, 2.0, 4.0, 1.4),
    (4.0, 2.0, 1.5, 2.5, 5.0, 2.0),
    (5.0, 2.5, 2.0, 3.0, 5.0, 2.5),
    (6.0, 3.0, 2.0, 3.5, 6.0, 3.0),
    (8.0, 4.0, 3.0, 5.0, 8.0, 5.0),
    (10.0, 5.0, 4.0, 6.0, 10.0, 6.0),
    (12.0, 6.0, 4.8, 8.0, 12.0, 8.0),
    (16.0, 8.0, 6.4, 10.0, 16.0, 10.0),
    (20.0, 10.0, 8.0, 12.0, 20.0, 14.0),
    (24.0, 12.0, 10.0, 15.0, 25.0, 16.0),
];

fn cup_row(nominal: f64) -> Option<&'static SetRow> {
    CUP_SET.iter().find(|row| same_mm(row.0, nominal))
}

/// The ISO 4029 socket for a cup point set screw of this length.
pub fn cup_set_screw(nominal: f64, length: f64) -> Option<Socket> {
    cup_row(nominal).map(|&(_, across_flats, short, long, shaded, _)| Socket {
        across_flats,
        depth: if length <= shaded { short } else { long },
    })
}

/// Whether ISO 4026, or ISO 4029 for a cup point, shades this length for this size. A
/// shaded, short screw has the shallower socket and a 120° chamfer round its socket
/// end, where a longer one has 90°.
pub fn is_short(nominal: f64, length: f64, cup: bool) -> bool {
    let row = if cup { cup_row(nominal) } else { None }.or_else(|| set_row(nominal));
    row.is_some_and(|row| length <= row.4)
}

/// ISO 4029 `dz` max: the mouth of the cup, which is also where the 45° chamfer round
/// the outside of the point ends, so the rim between them is sharp.
pub fn cup_point(nominal: f64) -> Option<f64> {
    cup_row(nominal).map(|row| row.5)
}

/// The ISO 4026 socket for a set screw of this length: the shallower depth for a
/// length in the shaded area of Table 1, and the deeper one below it.
pub fn set_screw(nominal: f64, length: f64) -> Option<Socket> {
    set_row(nominal).map(|&(_, across_flats, short, long, shaded, _)| Socket {
        across_flats,
        depth: if length <= shaded { short } else { long },
    })
}

/// ISO 4026 `dp` max: the diameter of the flat on a flat-point set screw. The end is
/// chamfered at 45° from the thread down to it.
pub fn flat_point(nominal: f64) -> Option<f64> {
    set_row(nominal).map(|row| row.5)
}

/// A typed socket. A bare key size keeps the product-standard depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Request {
    Key(f64),
    Sized(Socket),
}

const BAD_SIZE: &str =
    "must be a key size in millimetres, such as 2.5, or a key size and a depth, such as 2.5x1.3";

/// `2.5`, or the key size and the depth: `2.5x1.3`, with `x`, `X` or `×` between them.
pub fn parse_size(text: &str) -> Result<Request, FieldError> {
    let bad = || FieldError::new("hex", BAD_SIZE);
    let mut parts = text.split(['x', 'X', '×']);
    let across_flats = parts.next().and_then(millimetres).ok_or_else(bad)?;
    let depth = match parts.next() {
        None => None,
        Some(text) => Some(millimetres(text).ok_or_else(bad)?),
    };
    if parts.next().is_some() {
        return Err(bad());
    }
    Ok(match depth {
        Some(depth) => Request::Sized(Socket {
            across_flats,
            depth,
        }),
        None => Request::Key(across_flats),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `s` nominal and `t` min from the DIN 912 data sheet, M1.6 to M64.
    #[test]
    fn cap_sockets_match_din_912() {
        let s = [
            1.5, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 14.0, 17.0, 19.0, 22.0, 27.0,
            32.0, 36.0, 41.0, 46.0,
        ];
        let t = [
            0.7, 1.0, 1.1, 1.3, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0, 15.5, 19.0,
            24.0, 28.0, 34.0, 38.0,
        ];
        let d = [
            1.6, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 20.0, 24.0, 30.0, 36.0,
            42.0, 48.0, 56.0, 64.0,
        ];
        assert_eq!(CAP.len(), d.len());
        for i in 0..d.len() {
            let socket = standard(Kind::Cap, d[i]).unwrap();
            assert_eq!(
                (socket.across_flats, socket.depth),
                (s[i], t[i]),
                "M{}",
                d[i]
            );
        }
    }

    #[test]
    fn each_head_has_its_own_m3_socket() {
        assert_eq!(standard(Kind::Cap, 3.0).unwrap().across_flats, 2.5);
        assert_eq!(standard(Kind::Button, 3.0).unwrap().across_flats, 2.0);
        assert_eq!(standard(Kind::Countersunk, 3.0).unwrap().across_flats, 2.0);
        assert_eq!(standard(Kind::Set, 3.0).unwrap().across_flats, 1.5);
        assert!(standard(Kind::Button, 2.0).is_none());
    }

    #[test]
    fn every_socket_is_narrower_than_its_thread_and_gets_wider_with_it() {
        for table in [CAP, BUTTON, COUNTERSUNK] {
            let mut last = 0.0;
            for (nominal, s, t) in table {
                assert!(*s < *nominal && *t > 0.0, "M{nominal}");
                assert!(*s >= last, "M{nominal}");
                last = *s;
            }
        }
        let mut last = 0.0;
        for (nominal, s, short, long, _, dp) in SET {
            assert!(s < nominal && dp < nominal && short < long, "M{nominal}");
            assert!(*s >= last, "M{nominal}");
            last = *s;
        }
    }

    #[test]
    fn the_corners_are_a_hexagon_across_the_flats() {
        let socket = Socket {
            across_flats: 2.5,
            depth: 1.3,
        };
        let corners = socket.corners();
        assert!((corners[0][0] - 2.5 / 3.0_f64.sqrt()).abs() < 1e-12);
        assert!(corners[0][1].abs() < 1e-12);
        // Two neighbours share a flat, and its distance from the centre is s / 2.
        let mid = [
            (corners[1][0] + corners[2][0]) / 2.0,
            (corners[1][1] + corners[2][1]) / 2.0,
        ];
        assert!((mid[0].hypot(mid[1]) - 1.25).abs() < 1e-12);
    }

    #[test]
    fn a_size_reads_the_key_and_an_optional_depth() {
        assert_eq!(parse_size("2.5").unwrap(), Request::Key(2.5));
        assert_eq!(
            parse_size(" 2.5 x 1.3 ").unwrap(),
            Request::Sized(Socket {
                across_flats: 2.5,
                depth: 1.3
            })
        );
        assert_eq!(parse_size("T10").unwrap_err().field, "hex");
        assert_eq!(parse_size("2.5x").unwrap_err().field, "hex");
        assert_eq!(parse_size("2x1x1").unwrap_err().field, "hex");
    }

    /// ISO 4026:2003 Table 1: `dp` max, `s` nominal, `t` min for the lengths in the
    /// shaded area and `t` min for the lengths below it, and the last shaded length.
    /// <https://www.fasteners.eu/standards/iso/4026/> gives the same `s`, `t` and `dp`.
    #[test]
    fn set_screws_match_iso_4026() {
        let iso_4026 = [
            (1.6, 0.8, 0.7, 0.7, 1.5, 2.0),
            (2.0, 1.0, 0.9, 0.8, 1.7, 3.0),
            (2.5, 1.5, 1.3, 1.2, 2.0, 3.0),
            (3.0, 2.0, 1.5, 1.2, 2.0, 3.0),
            (4.0, 2.5, 2.0, 1.5, 2.5, 4.0),
            (5.0, 3.5, 2.5, 2.0, 3.0, 5.0),
            (6.0, 4.0, 3.0, 2.0, 3.5, 6.0),
            (8.0, 5.5, 4.0, 3.0, 5.0, 8.0),
            (10.0, 7.0, 5.0, 4.0, 6.0, 10.0),
            (12.0, 8.5, 6.0, 4.8, 8.0, 12.0),
            (16.0, 12.0, 8.0, 6.4, 10.0, 16.0),
            (20.0, 15.0, 10.0, 8.0, 12.0, 20.0),
            (24.0, 18.0, 12.0, 10.0, 15.0, 25.0),
        ];
        assert_eq!(SET.len(), iso_4026.len());
        for (d, dp, s, short, long, shaded) in iso_4026 {
            assert_eq!(flat_point(d), Some(dp), "M{d}");
            let at_line = set_screw(d, shaded).unwrap();
            assert_eq!((at_line.across_flats, at_line.depth), (s, short), "M{d}");
            let below = set_screw(d, shaded + 1.0).unwrap();
            assert_eq!((below.across_flats, below.depth), (s, long), "M{d}");
            // Without a length, the depth every length can hold.
            assert_eq!(standard(Kind::Set, d).unwrap().depth, short, "M{d}");
        }
        // The DIN 913 sizes that ISO 4026:2003 leaves out.
        for d in [1.4, 1.8, 14.0, 18.0, 22.0] {
            assert!(standard(Kind::Set, d).is_none(), "M{d}");
            assert!(flat_point(d).is_none(), "M{d}");
        }
    }

    /// ISO 4029:2003 Table 1: `dz` max, `s` nominal, both `t` min and the last shaded
    /// length.
    #[test]
    fn cup_point_set_screws_match_iso_4029() {
        let iso_4029 = [
            (1.6, 0.8, 0.7, 0.7, 1.5, 2.0),
            (2.0, 1.0, 0.9, 0.8, 1.7, 2.5),
            (2.5, 1.2, 1.3, 1.2, 2.0, 3.0),
            (3.0, 1.4, 1.5, 1.2, 2.0, 4.0),
            (4.0, 2.0, 2.0, 1.5, 2.5, 5.0),
            (5.0, 2.5, 2.5, 2.0, 3.0, 5.0),
            (6.0, 3.0, 3.0, 2.0, 3.5, 6.0),
            (8.0, 5.0, 4.0, 3.0, 5.0, 8.0),
            (10.0, 6.0, 5.0, 4.0, 6.0, 10.0),
            (12.0, 8.0, 6.0, 4.8, 8.0, 12.0),
            (16.0, 10.0, 8.0, 6.4, 10.0, 16.0),
            (20.0, 14.0, 10.0, 8.0, 12.0, 20.0),
            (24.0, 16.0, 12.0, 10.0, 15.0, 25.0),
        ];
        assert_eq!(CUP_SET.len(), iso_4029.len());
        for (d, dz, s, short, long, shaded) in iso_4029 {
            assert_eq!(cup_point(d), Some(dz), "M{d}");
            let at_line = cup_set_screw(d, shaded).unwrap();
            assert_eq!((at_line.across_flats, at_line.depth), (s, short), "M{d}");
            let below = cup_set_screw(d, shaded + 1.0).unwrap();
            assert_eq!((below.across_flats, below.depth), (s, long), "M{d}");
            // The same key as the flat point set screw of that thread.
            assert_eq!(standard(Kind::Set, d).unwrap().across_flats, s, "M{d}");
        }
        // ISO 4029 shades M3 x 4, which ISO 4026 does not.
        assert!(is_short(3.0, 4.0, true) && !is_short(3.0, 4.0, false));
        assert!(is_short(3.0, 3.0, false) && !is_short(3.0, 5.0, true));
        assert!(!is_short(7.0, 2.0, false));
        assert_eq!(cup_set_screw(3.0, 4.0).unwrap().depth, 1.2);
        assert_eq!(set_screw(3.0, 4.0).unwrap().depth, 2.0);
        // The DIN 916 sizes that ISO 4029:2003 leaves out.
        for d in [1.4, 1.8, 14.0, 18.0, 22.0] {
            assert!(cup_point(d).is_none(), "M{d}");
        }
    }
}
