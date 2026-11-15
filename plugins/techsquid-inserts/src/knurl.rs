//! The outside of a CNC Kitchen insert.
//!
//! Three rings on a shaft, from the face end to the tip:
//!
//! - the upper knurl, a 12-point star across D1 that twists one way;
//! - a smooth groove;
//! - the middle knurl, a little narrower than the upper one, that twists the other way
//!   and ends in a large 45° chamfer;
//! - a short neck;
//! - a smooth pilot about as wide as the plastic hole D2, with a small chamfer at the
//!   tip, so the insert drops into its hole before it is pressed in.
//!
//! The proportions are measured off the section drawing in CNC Kitchen's "Dimensions &
//! Design Guidelines" (the source of the catalog, see `catalog.rs`), and the tooth
//! count, the tooth depth and the hand of each knurl off CNC Kitchen's product photographs.
//! Radii and lengths here are fractions of the crest radius and of the length L.

/// Points around the star.
pub const TEETH: usize = 12;
/// Valley radius divided by crest radius.
const ROOT_OF_CREST: f64 = 0.85;
/// The groove and the core under both knurls, divided by the crest radius.
const GROOVE_OF_CREST: f64 = 0.83;
/// The neck between the middle knurl's chamfer and the pilot.
const NECK_OF_CREST: f64 = 0.79;
/// The pilot, unless that is wider than the plastic hole.
const PILOT_OF_CREST: f64 = 0.875;
/// The middle knurl's teeth are this fraction as tall as the upper knurl's.
const MIDDLE_TOOTH: f64 = 0.8;
/// Lengths along the insert, as fractions of L. The middle knurl and its chamfer take the rest.
const UPPER_FRACTION: f64 = 0.29;
const GROOVE_FRACTION: f64 = 0.11;
const NECK_FRACTION: f64 = 0.10;
const PILOT_FRACTION: f64 = 0.135;
const TIP_CHAMFER_FRACTION: f64 = 0.03;
/// The middle chamfer runs at 45°, unless that would take more than this share of the
/// middle knurl. Then it is steeper.
const MOST_CHAMFER: f64 = 0.6;
/// Angle between a tooth and the insert axis, at the midpoint of the tooth.
const HELIX_FROM_AXIS_DEG: f64 = 50.0;
/// How far the valley stays outside the thread major, when the drawn valley would not.
const ROOT_CLEARANCE: f64 = 0.10;
/// How far the groove, the neck and the pilot stay outside the thread major.
const GAP_CLEARANCE: f64 = 0.05;
/// The valley never comes closer to the crest than this fraction of it.
const SHALLOWEST_ROOT: f64 = 0.97;
/// Further than this, a tooth crosses the next one and the solid folds through itself.
const MAX_TWIST_DEG: f64 = 40.0;
/// Two profile points closer than this are one.
const SAME: f64 = 1e-9;

/// Radii and depths of one insert. A depth is measured from the face end toward the tip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub length: f64,
    /// The upper knurl's points, D1 / 2.
    pub crest: f64,
    /// The middle knurl's points.
    pub middle_crest: f64,
    /// The valley of both knurls.
    pub root: f64,
    /// The groove, and the core under both knurls.
    pub groove: f64,
    pub neck: f64,
    pub pilot: f64,
    /// The upper knurl runs from the face to this depth.
    pub upper: f64,
    /// The middle knurl starts here.
    pub middle_start: f64,
    /// The middle knurl's length, its chamfer included.
    pub middle: f64,
    /// How far along the axis the middle chamfer runs.
    pub chamfer: f64,
    /// The pilot starts here.
    pub pilot_start: f64,
    pub tip_chamfer: f64,
    /// Twist of each knurl, as `solid.sweep` takes it. The two have opposite signs.
    pub upper_twist: f64,
    pub middle_twist: f64,
}

/// The smallest D1 whose valley, groove and neck all stay outside a thread of this
/// `major`. Below it [`shape`] clamps the valley to the crest and the core falls inside
/// the thread, so a modeled thread would cut through the insert.
pub fn min_crest_diameter(major: f64) -> f64 {
    (major + 2.0 * ROOT_CLEARANCE) / SHALLOWEST_ROOT
}

/// `crest_diameter` is D1, `major` the thread's basic major diameter and `hole` the
/// plastic hole D2 this insert is pressed into.
pub fn shape(length: f64, crest_diameter: f64, major: f64, hole: f64) -> Shape {
    let crest = crest_diameter / 2.0;
    // On a large insert the drawn proportions put the core inside the thread. The
    // floors lift only a radius that would.
    let root_floor = major / 2.0 + ROOT_CLEARANCE;
    let gap_floor = major / 2.0 + GAP_CLEARANCE;
    let mut root = crest * ROOT_OF_CREST;
    if root < root_floor {
        root = root_floor.min(crest * SHALLOWEST_ROOT);
    }
    if root >= crest {
        root = crest * SHALLOWEST_ROOT;
    }
    let mut groove = (crest * GROOVE_OF_CREST).max(gap_floor);
    if groove >= root {
        groove = root - 0.02;
    }
    let middle_crest = root + (crest - root) * MIDDLE_TOOTH;
    let neck = (crest * NECK_OF_CREST).max(gap_floor).min(groove);
    // The pilot drops into the hole, so it is no wider than the hole, and no narrower
    // than the neck.
    let pilot = (crest * PILOT_OF_CREST).min(hole / 2.0).max(neck);

    let upper = length * UPPER_FRACTION;
    let middle_start = upper + length * GROOVE_FRACTION;
    let middle = length * (1.0 - UPPER_FRACTION - GROOVE_FRACTION - NECK_FRACTION - PILOT_FRACTION);
    let chamfer = (middle_crest - neck).min(middle * MOST_CHAMFER);
    let pilot_start = middle_start + middle + length * NECK_FRACTION;
    let tip_chamfer = (length * TIP_CHAMFER_FRACTION).min((pilot - major / 2.0) / 2.0);

    // Both knurls share one helix angle. The longer one sets the twist that stays
    // below the fold.
    let mid = (crest + root) / 2.0;
    let per_mm = (HELIX_FROM_AXIS_DEG.to_radians().tan() / mid)
        .to_degrees()
        .min(MAX_TWIST_DEG / upper.max(middle));
    Shape {
        length,
        crest,
        middle_crest,
        root,
        groove,
        neck,
        pilot,
        upper,
        middle_start,
        middle,
        chamfer,
        pilot_start,
        tip_chamfer,
        upper_twist: -per_mm * upper,
        middle_twist: per_mm * middle,
    }
}

impl Shape {
    /// Where the middle knurl and its chamfer end, and the neck starts.
    pub fn middle_end(&self) -> f64 {
        self.middle_start + self.middle
    }

    /// Depth gained per millimetre the chamfer comes in.
    fn chamfer_slope(&self) -> f64 {
        self.chamfer / (self.middle_crest - self.neck)
    }

    /// The smooth body, half of its section, to revolve about the axis: x is the
    /// radius and y the depth from the face. The core runs under both knurls at the
    /// groove radius and follows the middle chamfer down to the neck.
    pub fn core_profile(&self) -> Vec<[f64; 2]> {
        let end = self.middle_end();
        let slope = self.chamfer_slope();
        simplify(&[
            [0.0, 0.0],
            [self.groove, 0.0],
            [self.groove, end - slope * (self.groove - self.neck)],
            [self.neck, end],
            [self.neck, self.pilot_start],
            [self.pilot, self.pilot_start],
            [self.pilot, self.length - self.tip_chamfer],
            [self.pilot - self.tip_chamfer, self.length],
            [0.0, self.length],
        ])
    }

    /// The triangle that cuts the middle chamfer out of the middle knurl, in the same
    /// coordinates as [`Shape::core_profile`]. Its slope runs from the neck out past the
    /// points.
    pub fn chamfer_cut(&self) -> Vec<[f64; 2]> {
        let end = self.middle_end();
        let outside = self.crest + (self.crest - self.root);
        [
            [self.neck, end],
            [outside, end],
            [outside, end - self.chamfer_slope() * (outside - self.neck)],
        ]
        .to_vec()
    }
}

/// Drops a point that repeats the one before it, or that lies on the line between its
/// neighbours, so no line of the sketch has zero length.
fn simplify(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut out: Vec<[f64; 2]> = Vec::new();
    for &p in points {
        if let Some(last) = out.last() {
            if (last[0] - p[0]).abs() < SAME && (last[1] - p[1]).abs() < SAME {
                continue;
            }
        }
        if out.len() >= 2 {
            let [a, b] = [out[out.len() - 2], out[out.len() - 1]];
            let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
            if cross.abs() < SAME {
                out.pop();
            }
        }
        out.push(p);
    }
    out
}

/// Crest, valley, crest, valley, around the face. Sketch coordinates, millimetres.
pub fn star(crest: f64, root: f64) -> Vec<[f64; 2]> {
    (0..TEETH * 2)
        .map(|i| {
            let angle = i as f64 * std::f64::consts::PI / TEETH as f64;
            let radius = if i % 2 == 0 { crest } else { root };
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// The section drawing, read as fractions of the crest radius and of L.
    #[test]
    fn an_m3_insert_has_the_drawn_rings() {
        let s = shape(5.7, 4.6, 3.0, 4.0);
        assert!(close(s.crest, 2.3));
        assert!(close(s.root, 2.3 * 0.85));
        assert!(close(s.groove, 2.3 * 0.83));
        assert!(close(s.neck, 2.3 * 0.79));
        // 0.875 of the crest is 4.025 across, wider than the 4.0 hole, so the pilot is the hole.
        assert!(close(s.pilot, 2.0));
        assert!(s.crest > s.middle_crest && s.middle_crest > s.root);
        assert!(s.root > s.groove && s.groove > s.neck && s.pilot > s.neck);
        assert!(close(s.upper, 5.7 * 0.29));
        assert!(close(s.middle_start, 5.7 * 0.40));
        assert!(close(s.pilot_start, 5.7 * 0.865));
        // A 45° chamfer from the middle knurl's points down to the neck.
        assert!(close(s.chamfer, s.middle_crest - s.neck));
        assert!(s.tip_chamfer > 0.0 && s.tip_chamfer <= 5.7 * 0.03);
        // The knurls twist opposite ways, at one helix angle.
        assert!(s.upper_twist < 0.0 && s.middle_twist > 0.0);
        assert!(close(-s.upper_twist / s.upper, s.middle_twist / s.middle));
        assert!(s.middle_twist <= MAX_TWIST_DEG && -s.upper_twist <= MAX_TWIST_DEG);
    }

    #[test]
    fn the_core_profile_runs_from_face_to_tip() {
        let s = shape(5.7, 4.6, 3.0, 4.0);
        let profile = s.core_profile();
        assert_eq!(profile.len(), 9);
        assert_eq!(profile[0], [0.0, 0.0]);
        assert_eq!(*profile.last().unwrap(), [0.0, 5.7]);
        // The core's slope and the cut's slope are one line, through the middle knurl's points.
        let cut = s.chamfer_cut();
        let on_line = |p: [f64; 2]| {
            let [a, b] = [cut[0], cut[2]];
            ((b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])).abs() < 1e-9
        };
        assert!(on_line(profile[2]) && on_line(profile[3]));
        assert!(on_line([s.middle_crest, s.middle_end() - s.chamfer]));
        // The cut stays clear of the groove and the upper knurl.
        assert!(cut[2][1] > s.middle_start);
    }

    #[test]
    fn a_large_insert_keeps_its_core_outside_the_thread() {
        // M10 × 12.7: 0.79 of the crest is 4.98, inside the 5.0 thread.
        for (length, d1, major, hole) in [(12.7, 12.6, 10.0, 12.0), (12.7, 10.2, 8.0, 9.7)] {
            let s = shape(length, d1, major, hole);
            for radius in [s.root, s.groove, s.neck, s.pilot - s.tip_chamfer] {
                assert!(radius > major / 2.0, "{d1}: {radius}");
            }
            assert!(s.pilot <= hole / 2.0);
        }
    }

    #[test]
    fn a_crest_close_to_the_thread_keeps_a_root_and_a_clean_profile() {
        // The smallest M3 diameter the dialog takes, in a hole narrower than the neck.
        let s = shape(5.7, 3.3, 3.0, 2.7);
        assert!(s.root < s.crest && s.root > 1.5);
        assert!(s.groove < s.root && s.neck > 1.5);
        assert!(close(s.pilot, s.neck));
        assert!(s.tip_chamfer > 0.0);
        let profile = s.core_profile();
        for pair in profile.windows(2) {
            let length = (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]);
            assert!(length > 1e-9, "{profile:?}");
        }
    }

    #[test]
    fn a_short_insert_keeps_a_straight_middle_knurl() {
        for (length, d1, major, hole) in [
            (3.0, 4.6, 3.0, 4.0),
            (6.8, 10.2, 8.0, 9.7),
            (2.5, 2.5, 1.6, 2.2),
        ] {
            let s = shape(length, d1, major, hole);
            assert!(s.chamfer < s.middle, "{length} x {d1}");
            assert!(close(s.pilot_start + length * PILOT_FRACTION, length));
        }
    }

    #[test]
    fn a_star_has_twelve_points() {
        let points = star(2.3, 1.955);
        assert_eq!(points.len(), 24);
        assert!(close(points[0][0], 2.3));
        assert!(close(points[1][0].hypot(points[1][1]), 1.955));
    }
}
