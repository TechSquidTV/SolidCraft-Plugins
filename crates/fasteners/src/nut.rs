//! ISO 4032 hexagon nuts.
//!
//! Source: <https://www.fasteners.eu/standards/ISO/4032/>. `nuts_match_iso_4032` repeats it.
//!
//! ISO 4032 replaced DIN 934, which differs at some sizes: DIN 934 is 17 / 19 / 22 / 32 mm
//! across the flats at M10 / M12 / M14 / M22 (ISO 4032: 16 / 18 / 21 / 34) and a little
//! lower at most sizes. This table is ISO 4032 only. The width across the flats is the nominal size and
//! the height is the standard maximum. Both ends are chamfered, which is the nut in the
//! DIN 934 drawing. The chamfer is 30° from the end face, the steep side of the drawing's
//! 15° to 30°. It runs out to the across-flats circle, and it stops at `c` max when that
//! circle would make a taller chamfer than the standard allows. The hole is countersunk
//! on both ends at about 120° included, out to `da` max.

use crate::metric::same_mm;

/// 30° from the end face. The drawing allows 15° to 30°.
pub const CHAMFER_FROM_FACE_DEG: f64 = 30.0;

/// Included angle of the countersink at the thread, from the drawing's detail.
pub const COUNTERSINK_INCLUDED_DEG: f64 = 120.0;

/// How far the chamfer cutter reaches past the hex corner, so the cut clears it.
const MARGIN_MM: f64 = 0.05;

/// One row of the ISO 4032 table. Lengths are millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NutSize {
    pub nominal: f64,
    pub pitch: f64,
    /// Width across the flats, nominal and maximum.
    pub across_flats: f64,
    /// Nut height, the standard maximum.
    pub height: f64,
    /// `da` max, the countersink diameter at the thread.
    pub countersink: f64,
    /// `dw` min, the smallest bearing-face diameter.
    pub bearing_min: f64,
    /// `mw` min, the smallest wrenching height.
    pub wrenching_min: f64,
    /// `c` max, the tallest chamfer.
    pub chamfer_max: f64,
    /// `c` min, the shortest chamfer.
    pub chamfer_min: f64,
}

/// The chamfer cut on one end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndChamfer {
    /// Radius of the chamfer circle on the end face.
    pub face_radius: f64,
    /// Axial height of the chamfer at the hex corner.
    pub axial: f64,
    /// Cutter profile, in a plane through the axis. Sketch +x is radial and +y runs into
    /// the nut from that end face. The three corners are the face at `face_radius`, the
    /// face outside the corner, and that outside point at the cutter's full height.
    pub triangle: [[f64; 2]; 3],
}

// nominal, pitch, across flats, height, da max, dw min, mw min, c max, c min.
// Preferred sizes and the second-choice sizes, in order of nominal diameter.
type Row9 = (f64, f64, f64, f64, f64, f64, f64, f64, f64);
const ROWS: &[Row9] = &[
    (1.6, 0.35, 3.2, 1.3, 1.84, 2.4, 0.8, 0.2, 0.1),
    (2.0, 0.4, 4.0, 1.6, 2.3, 3.1, 1.1, 0.2, 0.1),
    (2.5, 0.45, 5.0, 2.0, 2.9, 4.1, 1.4, 0.3, 0.1),
    (3.0, 0.5, 5.5, 2.4, 3.45, 4.6, 1.7, 0.4, 0.15),
    (3.5, 0.6, 6.0, 2.8, 4.0, 5.0, 2.0, 0.4, 0.15),
    (4.0, 0.7, 7.0, 3.2, 4.6, 5.9, 2.3, 0.4, 0.15),
    (5.0, 0.8, 8.0, 4.7, 5.75, 6.9, 3.5, 0.5, 0.15),
    (6.0, 1.0, 10.0, 5.2, 6.75, 8.9, 3.9, 0.5, 0.15),
    (8.0, 1.25, 13.0, 6.8, 8.75, 11.6, 5.2, 0.6, 0.15),
    (10.0, 1.5, 16.0, 8.4, 10.8, 14.6, 6.4, 0.6, 0.15),
    (12.0, 1.75, 18.0, 10.8, 13.0, 16.6, 8.3, 0.6, 0.15),
    (14.0, 2.0, 21.0, 12.8, 15.1, 19.6, 9.7, 0.6, 0.15),
    (16.0, 2.0, 24.0, 14.8, 17.3, 22.5, 11.3, 0.8, 0.2),
    (18.0, 2.5, 27.0, 15.8, 19.5, 24.9, 12.1, 0.8, 0.2),
    (20.0, 2.5, 30.0, 18.0, 21.6, 27.7, 13.5, 0.8, 0.2),
    (22.0, 2.5, 34.0, 19.4, 23.7, 31.4, 14.5, 0.8, 0.2),
    (24.0, 3.0, 36.0, 21.5, 25.9, 33.3, 16.2, 0.8, 0.2),
    (27.0, 3.0, 41.0, 23.8, 29.1, 38.0, 18.0, 0.8, 0.2),
    (30.0, 3.5, 46.0, 25.6, 32.4, 42.8, 19.4, 0.8, 0.2),
    (33.0, 3.5, 50.0, 28.7, 35.6, 46.6, 21.9, 0.8, 0.2),
    (36.0, 4.0, 55.0, 31.0, 38.9, 51.1, 23.5, 0.8, 0.2),
    (39.0, 4.0, 60.0, 33.4, 42.1, 55.9, 25.4, 1.0, 0.3),
    (42.0, 4.5, 65.0, 34.0, 45.4, 60.0, 25.9, 1.0, 0.3),
    (45.0, 4.5, 70.0, 36.0, 48.6, 64.7, 27.5, 1.0, 0.3),
    (48.0, 5.0, 75.0, 38.0, 51.8, 69.5, 29.1, 1.0, 0.3),
    (52.0, 5.0, 80.0, 42.0, 56.2, 74.2, 32.3, 1.0, 0.3),
    (56.0, 5.5, 85.0, 45.0, 60.5, 78.7, 34.7, 1.0, 0.3),
    (60.0, 5.5, 90.0, 48.0, 64.8, 83.4, 37.1, 1.0, 0.3),
    (64.0, 6.0, 95.0, 51.0, 69.1, 88.2, 39.3, 1.0, 0.3),
];

fn row(values: (f64, f64, f64, f64, f64, f64, f64, f64, f64)) -> NutSize {
    let (
        nominal,
        pitch,
        across_flats,
        height,
        countersink,
        bearing_min,
        wrenching_min,
        chamfer_max,
        chamfer_min,
    ) = values;
    NutSize {
        nominal,
        pitch,
        across_flats,
        height,
        countersink,
        bearing_min,
        wrenching_min,
        chamfer_max,
        chamfer_min,
    }
}

pub fn standard_nut(nominal: f64) -> Option<NutSize> {
    ROWS.iter()
        .find(|values| same_mm(values.0, nominal))
        .copied()
        .map(row)
}

pub fn sizes() -> impl Iterator<Item = NutSize> {
    ROWS.iter().copied().map(row)
}

/// Distance from the axis to a hex corner. A flat sits at half the across-flats width,
/// and the corner is 30° of hex rotation past that flat.
pub fn corner_radius(across_flats: f64) -> f64 {
    across_flats / 3f64.sqrt()
}

/// `chamfer_max` is `c` max. `None` chamfers out to the across-flats circle.
pub fn end_chamfer(across_flats: f64, chamfer_max: Option<f64>) -> EndChamfer {
    let corner = corner_radius(across_flats);
    let flats = across_flats / 2.0;
    let tan = CHAMFER_FROM_FACE_DEG.to_radians().tan();
    let to_flats = (corner - flats) * tan;
    let axial = match chamfer_max {
        Some(limit) => to_flats.min(limit),
        None => to_flats,
    };
    let face_radius = corner - axial / tan;
    let outer = corner + MARGIN_MM;
    let cutter = (outer - face_radius) * tan;
    EndChamfer {
        face_radius,
        axial,
        triangle: [[face_radius, 0.0], [outer, 0.0], [outer, cutter]],
    }
}

/// Axial depth of a 120° countersink that opens from the major diameter to `da`.
pub fn countersink_depth(major: f64, da: f64) -> f64 {
    let from_face = (180.0 - COUNTERSINK_INCLUDED_DEG) / 2.0;
    ((da - major) / 2.0) * from_face.to_radians().tan()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metric::coarse_pitch;

    #[test]
    fn an_m6_nut_is_10_across_the_flats_and_5_2_high() {
        let nut = standard_nut(6.0).unwrap();
        assert_eq!(nut.across_flats, 10.0);
        assert_eq!(nut.height, 5.2);
        assert_eq!(nut.pitch, 1.0);
        assert_eq!(nut.countersink, 6.75);
        assert!(standard_nut(7.0).is_none());
    }

    #[test]
    fn an_m3_5_is_a_second_choice_size() {
        let nut = standard_nut(3.5).unwrap();
        assert_eq!(nut.pitch, 0.6);
        assert_eq!(nut.across_flats, 6.0);
        assert_eq!(nut.height, 2.8);
    }

    #[test]
    fn every_listed_pitch_matches_the_coarse_pitch() {
        for nut in sizes() {
            if let Some(pitch) = coarse_pitch(nut.nominal) {
                assert_eq!(nut.pitch, pitch, "M{}", nut.nominal);
            }
        }
        assert_eq!(coarse_pitch(3.5), Some(0.6));
    }

    #[test]
    fn every_chamfer_stays_inside_the_standard() {
        for nut in sizes() {
            let end = end_chamfer(nut.across_flats, Some(nut.chamfer_max));
            assert!(
                end.axial + 1e-9 >= nut.chamfer_min,
                "M{} chamfer {} is under c min {}",
                nut.nominal,
                end.axial,
                nut.chamfer_min
            );
            assert!(
                end.axial <= nut.chamfer_max + 1e-9,
                "M{} chamfer {} is over c max {}",
                nut.nominal,
                end.axial,
                nut.chamfer_max
            );
            let bearing = 2.0 * end.face_radius;
            assert!(
                bearing + 1e-6 >= nut.bearing_min,
                "M{} bearing {bearing} is under dw min {}",
                nut.nominal,
                nut.bearing_min
            );
            let wrenching = nut.height - 2.0 * end.axial;
            assert!(
                wrenching + 1e-6 >= nut.wrenching_min,
                "M{} wrenching {wrenching} is under mw min {}",
                nut.nominal,
                nut.wrenching_min
            );
            assert!(nut.countersink > nut.nominal);
        }
    }

    #[test]
    fn an_m6_chamfer_reaches_the_flats_and_an_m10_stops_at_c_max() {
        let m6 = end_chamfer(10.0, Some(0.5));
        assert!((m6.face_radius - 5.0).abs() < 1e-9);
        assert!(m6.axial < 0.5);
        let m10 = end_chamfer(16.0, Some(0.6));
        assert!((m10.axial - 0.6).abs() < 1e-12);
        assert!(m10.face_radius > 8.0);
    }

    #[test]
    fn a_countersink_is_shallow() {
        let depth = countersink_depth(6.0, 6.75);
        assert!(depth > 0.15 && depth < 0.3, "{depth}");
    }
    /// ISO 4032, <https://www.fasteners.eu/standards/ISO/4032/>, which prints the same
    /// table as `/DIN/934/`: `s` nominal = max, `m` max, `da` max, `dw` min, `mw` min,
    /// `c` max and `c` min, for the preferred and then the non-preferred threads.
    #[test]
    fn nuts_match_iso_4032() {
        #[rustfmt::skip]
        type Row8 = (f64, f64, f64, f64, f64, f64, f64, f64);
        let iso_4032: [Row8; 29] = [
            (1.6, 3.2, 1.3, 1.84, 2.4, 0.8, 0.2, 0.1),
            (2.0, 4.0, 1.6, 2.3, 3.1, 1.1, 0.2, 0.1),
            (2.5, 5.0, 2.0, 2.9, 4.1, 1.4, 0.3, 0.1),
            (3.0, 5.5, 2.4, 3.45, 4.6, 1.7, 0.4, 0.15),
            (4.0, 7.0, 3.2, 4.6, 5.9, 2.3, 0.4, 0.15),
            (5.0, 8.0, 4.7, 5.75, 6.9, 3.5, 0.5, 0.15),
            (6.0, 10.0, 5.2, 6.75, 8.9, 3.9, 0.5, 0.15),
            (8.0, 13.0, 6.8, 8.75, 11.6, 5.2, 0.6, 0.15),
            (10.0, 16.0, 8.4, 10.8, 14.6, 6.4, 0.6, 0.15),
            (12.0, 18.0, 10.8, 13.0, 16.6, 8.3, 0.6, 0.15),
            (16.0, 24.0, 14.8, 17.3, 22.5, 11.3, 0.8, 0.2),
            (20.0, 30.0, 18.0, 21.6, 27.7, 13.5, 0.8, 0.2),
            (24.0, 36.0, 21.5, 25.9, 33.3, 16.2, 0.8, 0.2),
            (30.0, 46.0, 25.6, 32.4, 42.8, 19.4, 0.8, 0.2),
            (36.0, 55.0, 31.0, 38.9, 51.1, 23.5, 0.8, 0.2),
            (42.0, 65.0, 34.0, 45.4, 60.0, 25.9, 1.0, 0.3),
            (48.0, 75.0, 38.0, 51.8, 69.5, 29.1, 1.0, 0.3),
            (56.0, 85.0, 45.0, 60.5, 78.7, 34.7, 1.0, 0.3),
            (64.0, 95.0, 51.0, 69.1, 88.2, 39.3, 1.0, 0.3),
            (3.5, 6.0, 2.8, 4.0, 5.0, 2.0, 0.4, 0.15),
            (14.0, 21.0, 12.8, 15.1, 19.6, 9.7, 0.6, 0.15),
            (18.0, 27.0, 15.8, 19.5, 24.9, 12.1, 0.8, 0.2),
            (22.0, 34.0, 19.4, 23.7, 31.4, 14.5, 0.8, 0.2),
            (27.0, 41.0, 23.8, 29.1, 38.0, 18.0, 0.8, 0.2),
            (33.0, 50.0, 28.7, 35.6, 46.6, 21.9, 0.8, 0.2),
            (39.0, 60.0, 33.4, 42.1, 55.9, 25.4, 1.0, 0.3),
            (45.0, 70.0, 36.0, 48.6, 64.7, 27.5, 1.0, 0.3),
            (52.0, 80.0, 42.0, 56.2, 74.2, 32.3, 1.0, 0.3),
            (60.0, 90.0, 48.0, 64.8, 83.4, 37.1, 1.0, 0.3),
        ];
        assert_eq!(sizes().count(), iso_4032.len());
        for (d, s, m, da, dw, mw, c_max, c_min) in iso_4032 {
            let nut = standard_nut(d).unwrap();
            assert_eq!(
                (
                    nut.across_flats,
                    nut.height,
                    nut.countersink,
                    nut.bearing_min
                ),
                (s, m, da, dw),
                "M{d}"
            );
            assert_eq!(
                (nut.wrenching_min, nut.chamfer_max, nut.chamfer_min),
                (mw, c_max, c_min),
                "M{d}"
            );
        }
    }
}
