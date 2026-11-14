//! The outside of a CNC Kitchen insert.
//!
//! The top face is a 16-point star. Those points twist down one way, leave a smooth
//! gap, then twist the other way. A short lead-in at the tip is smooth. The crest is
//! the catalog diameter D1. The proportions are taken from the short M3 photographs:
//! the valley is 0.76 of the crest, and a tooth runs at 50° from the axis, measured
//! halfway between the crest and the valley.

/// Points around the star.
pub const TEETH: usize = 16;
/// Valley radius divided by crest radius.
const ROOT_OF_CREST: f64 = 0.76;
/// Angle between a tooth and the insert axis, at the midpoint of the tooth.
const HELIX_FROM_AXIS_DEG: f64 = 50.0;
/// Smooth pilot at the tip, as a fraction of the length.
const LEAD_FRACTION: f64 = 0.18;
/// Smooth band between the two helices, as a fraction of the length.
const GAP_FRACTION: f64 = 0.09;
/// How far the valley stays outside the thread major, when the photographed valley would not.
const ROOT_CLEARANCE: f64 = 0.10;
/// How far the gap stays outside the thread major.
const GAP_CLEARANCE: f64 = 0.05;
/// The valley never comes closer to the crest than this fraction of it.
const SHALLOWEST_ROOT: f64 = 0.97;
/// Further than this, a tooth crosses the next one and the solid folds through itself.
const MAX_TWIST_DEG: f64 = 40.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub crest: f64,
    pub root: f64,
    pub gap_diameter: f64,
    pub lead_diameter: f64,
    pub lead: f64,
    pub gap: f64,
    pub band: f64,
    pub twist_deg: f64,
    pub chamfer: f64,
}

/// The smallest D1 whose valley and smooth gap both stay outside a thread of this
/// `major`. Below it [`shape`] clamps the valley to the crest and the gap core falls
/// inside the thread, so a modeled thread would cut through the insert.
pub fn min_crest_diameter(major: f64) -> f64 {
    (major + 2.0 * ROOT_CLEARANCE) / SHALLOWEST_ROOT
}

/// `crest_diameter` is D1. `major` is the thread's basic major diameter.
pub fn shape(length: f64, crest_diameter: f64, major: f64) -> Shape {
    let crest = crest_diameter / 2.0;
    // The short M3 valley is only ~0.25 mm outside the major. A larger floor would
    // shallow the teeth in the photographs. The floor only lifts a valley that would
    // otherwise cut into the thread.
    let root_floor = major / 2.0 + ROOT_CLEARANCE;
    let mut root = crest * ROOT_OF_CREST;
    if root < root_floor {
        root = root_floor.min(crest * SHALLOWEST_ROOT);
    }
    if root >= crest {
        root = crest * SHALLOWEST_ROOT;
    }
    let tooth = crest - root;
    let mut gap_radius = root - tooth * 0.28;
    let gap_floor = major / 2.0 + GAP_CLEARANCE;
    if gap_radius < gap_floor {
        gap_radius = gap_floor.min(root - 0.02);
    }
    if gap_radius >= root {
        gap_radius = root - 0.02;
    }
    let lead = length * LEAD_FRACTION;
    let gap = length * GAP_FRACTION;
    let band = (length - lead - gap) / 2.0;
    let mid = (crest + root) / 2.0;
    let twist_deg =
        (band / mid * HELIX_FROM_AXIS_DEG.to_radians().tan().to_degrees()).min(MAX_TWIST_DEG);
    let chamfer = (tooth * 0.3).min(lead * 0.35).min(0.2);
    Shape {
        crest,
        root,
        gap_diameter: gap_radius * 2.0,
        lead_diameter: root * 2.0,
        lead,
        gap,
        band,
        twist_deg,
        chamfer,
    }
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

    #[test]
    fn an_m3_star_clears_its_thread() {
        // Short and standard M3 share D1 4.6. The valley stays at 0.76 of the crest.
        for length in [3.0, 5.7] {
            let shape = shape(length, 4.6, 3.0);
            assert!((shape.crest - 2.3).abs() < 1e-12, "{length}");
            assert!((shape.root - 2.3 * 0.76).abs() < 1e-12, "{length}");
            let tooth = shape.crest - shape.root;
            assert!((shape.gap_diameter / 2.0 - (shape.root - tooth * 0.28)).abs() < 1e-12);
            assert!((shape.lead_diameter - shape.root * 2.0).abs() < 1e-12);
            assert!((shape.lead + shape.gap + shape.band * 2.0 - length).abs() < 1e-9);
            let mid = (shape.crest + shape.root) / 2.0;
            let twist = shape.band / mid * 50.0_f64.to_radians().tan().to_degrees();
            assert!((shape.twist_deg - twist.min(40.0)).abs() < 1e-9, "{length}");
        }
        let shape = shape(5.7, 4.6, 3.0);
        let points = star(shape.crest, shape.root);
        assert_eq!(points.len(), 32);
        assert!((points[0][0] - shape.crest).abs() < 1e-9);
        assert!((points[1][0].hypot(points[1][1]) - shape.root).abs() < 1e-9);
        // Sixteen crests, equally spaced.
        assert_eq!(points.iter().step_by(2).count(), 16);
    }

    #[test]
    fn a_crest_close_to_the_thread_keeps_a_root() {
        let shape = shape(4.0, 3.2, 3.0);
        assert!(shape.root < shape.crest);
        assert!(shape.gap_diameter < shape.lead_diameter);
        assert!(shape.root > 1.5);
    }

    #[test]
    fn a_large_insert_lifts_a_valley_that_would_cut_the_thread() {
        // M8 × 12.7: D1 is 10.2 and the major is 8, so 0.76 of the crest is inside the thread.
        let shape = shape(12.7, 10.2, 8.0);
        assert!(shape.root > 4.0);
        assert!(shape.crest - shape.root > 0.5);
        assert!(shape.gap_diameter / 2.0 > 4.0);
        assert!(shape.gap_diameter < shape.lead_diameter);
    }
}
