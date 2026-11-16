//! Measured MISUMI ring sections. Rounded outside edges and exposed open-bearing
//! raceways use true arcs, not faceted approximations. See docs/sources/misumi.md.

use super::Bearing;
use plugin_kit::revolve::Segment;

pub fn ring(bearing: Bearing, outer: bool) -> Vec<Segment> {
    let d = bearing.dimensions();
    let h = d.width / 2.0;
    let f = d.edge_radius;
    if f == 0.0 {
        return vec![];
    }
    let (edge, shoulder, sign) = if outer {
        (d.diameter / 2.0, d.outer_ring_radius, -1.0)
    } else {
        (d.bore / 2.0, d.inner_ring_radius, 1.0)
    };
    let a = [edge + sign * f, -h];
    let b = [edge + sign * f, h];
    let mut segments = vec![Segment::Line(vec![a, [shoulder, -h]])];
    if let Some(race) = bearing.open_race() {
        // The source's toroidal races have pitch radius 6.25 and section radius
        // 1.95. The same source record positions the visible balls.
        let pitch = race.pitch_radius;
        let radius = race.groove_radius;
        let extent = (radius.powi(2) - (shoulder - pitch).powi(2)).sqrt();
        segments.push(Segment::Line(vec![[shoulder, -h], [shoulder, -extent]]));
        segments.push(Segment::Arc {
            start: [shoulder, -extent],
            through: [pitch - sign * radius, 0.0],
            end: [shoulder, extent],
        });
        segments.push(Segment::Line(vec![[shoulder, extent], [shoulder, h], b]));
    } else {
        segments.push(Segment::Line(vec![[shoulder, -h], [shoulder, h], b]));
    }
    let k = f * std::f64::consts::FRAC_1_SQRT_2;
    segments.extend([
        Segment::Arc {
            start: b,
            through: [edge + sign * (f - k), h - f + k],
            end: [edge, h - f],
        },
        Segment::Line(vec![[edge, h - f], [edge, -h + f]]),
        Segment::Arc {
            start: [edge, -h + f],
            through: [edge + sign * (f - k), -h + f - k],
            end: a,
        },
    ]);
    segments
}
