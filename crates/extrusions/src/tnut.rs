//! The nuts that go in a 6 mm slot. Sizes are millimetres.
//!
//! A slot nut is a bar with one section along its whole length. A boss on top sits in
//! the slot's throat, and the shoulders beside it bear on the underside of the lips.
//! Below the shoulders the flanks slope in, so the nut clears the webs of the rail.
//!
//! A section is drawn across the slot: the first coordinate is the distance from the
//! middle of the slot, and the second is the height above the shoulders.

use crate::profile::{Seg, P};

/// The spring-loaded ball under a nut that holds it in place before it is tightened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ball {
    /// How far along the nut the ball is from the tapped hole.
    pub from_tap: f64,
    /// Radius of the ball.
    pub radius: f64,
    /// How far the ball stands out below the nut.
    pub stands_out: f64,
    /// Radius of the round recess in the bottom that the ball sits in.
    pub recess: f64,
    /// Depth of that recess.
    pub recess_depth: f64,
}

/// One slot nut.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TNut {
    /// What a name and the dialog call it: `pre-assembly`, `post-assembly`.
    pub name: &'static str,
    /// The MISUMI part it is, without the thread: `HNTT5`.
    pub part: &'static str,
    /// Length along the slot.
    pub length: f64,
    /// How far the tapped hole is from the end the nut starts at.
    pub tap_at: f64,
    /// Half the width of the boss that sits in the throat.
    pub boss: f64,
    /// Height of the boss above the shoulders.
    pub boss_height: f64,
    /// Radius of the boss's two top edges.
    pub boss_round: f64,
    /// Half the width across the flat of the shoulders. The nut is wider than this by
    /// `wing_round` on each side.
    pub wing: f64,
    /// Radius of the edge where a shoulder turns down into a flank.
    pub wing_round: f64,
    /// Height of the body, from the shoulders down to the bottom.
    pub body: f64,
    /// Half the width of the flat bottom.
    pub bottom: f64,
    /// Radius of the edge where a flank meets the bottom.
    pub bottom_round: f64,
    pub ball: Option<Ball>,
}

/// MISUMI HNTT5, the 5 series pre-assembly insertion nut. It slides in from the end of
/// the rail.
///
/// Source: MISUMI's STEP model `HNTT5-5.stp`, whose section has the vertices (2.9, 0.6),
/// (2.7, 0.8), (4.8, 0), (4.9542, -0.3273), (2.59, -3.191) and (2.3586, -3.3) with the
/// shoulders at 0, and is 10 long with the hole in the middle. The drawing on MISUMI's
/// product page gives the 10 by 10 and the 5.8.
pub const PRE_ASSEMBLY: TNut = TNut {
    name: "pre-assembly",
    part: "HNTT5",
    length: 10.0,
    tap_at: 5.0,
    boss: 2.9,
    boss_height: 0.8,
    boss_round: 0.2,
    wing: 4.8,
    wing_round: 0.2,
    body: 3.3,
    bottom: 2.3586,
    bottom_round: 0.3,
    ball: None,
};

/// MISUMI HNTP5, the 5 series post-assembly insertion nut. It goes in through the slot,
/// and a spring-loaded ball under it holds it where it is put.
///
/// Source: MISUMI's STEP model `HNTP5-5.stp`, whose section has the vertices (2.9, 0.5),
/// (2.7, 0.7), (3.95, 0), (4.1047, -0.3268), (1.84, -3.0902) and (1.608, -3.2) with the
/// shoulders at 0. It is 17 long with the hole 6 from one end, and 7.5 past the hole an
/// R2 ball stands 0.8 out of a recess 4.2 across and 0.2 deep. The drawing on MISUMI's
/// product page gives the 17, 8.3, 5.8, 3.2 and 3.9.
pub const POST_ASSEMBLY: TNut = TNut {
    name: "post-assembly",
    part: "HNTP5",
    length: 17.0,
    tap_at: 6.0,
    boss: 2.9,
    boss_height: 0.7,
    boss_round: 0.2,
    wing: 3.95,
    wing_round: 0.2,
    body: 3.2,
    bottom: 1.608,
    bottom_round: 0.3,
    ball: Some(Ball {
        from_tap: 7.5,
        radius: 2.0,
        stands_out: 0.8,
        recess: 2.1,
        recess_depth: 0.2,
    }),
};

/// The nuts, in the order the dialog lists them.
pub const NUTS: [TNut; 2] = [PRE_ASSEMBLY, POST_ASSEMBLY];

/// MISUMI 2019 US catalog pp. 2710 and 2716: HNTT5 and HNTP5 in carbon steel
/// are each sold with M3, M4 or M5 taps. Within each family the listed external
/// dimensions are shared across these thread sizes.
pub const THREADS: [&str; 3] = ["M3", "M4", "M5"];

impl TNut {
    /// MISUMI's purchasing code for this nut family and metric thread.
    pub fn part_number(&self, thread: &str) -> String {
        format!("{}-{}", self.part, thread.trim_start_matches('M'))
    }

    /// The widest the nut is, across the slot.
    pub fn width(&self) -> f64 {
        2.0 * (self.wing + self.wing_round)
    }

    /// The height from the bottom to the top of the boss, the ball left out.
    pub fn height(&self) -> f64 {
        self.body + self.boss_height
    }

    /// The direction a flank faces on the right-hand side, as a unit vector. A flank is
    /// the straight line that touches both the wing rounding and the bottom rounding.
    pub fn flank_normal(&self) -> P {
        let (wing, bottom) = (self.wing_centre(), self.bottom_centre());
        let d = [wing[0] - bottom[0], wing[1] - bottom[1]];
        let apart = d[0].hypot(d[1]);
        // The flank is `wing_round` from one centre and `bottom_round` from the other.
        let between = ((self.bottom_round - self.wing_round) / apart).acos();
        let angle = d[1].atan2(d[0]) - between;
        [angle.cos(), angle.sin()]
    }

    fn wing_centre(&self) -> P {
        [self.wing, -self.wing_round]
    }

    fn bottom_centre(&self) -> P {
        [self.bottom, self.bottom_round - self.body]
    }

    /// Where the flank on the right-hand side starts and ends, top first.
    pub fn flank(&self) -> (P, P) {
        let n = self.flank_normal();
        let (wing, bottom) = (self.wing_centre(), self.bottom_centre());
        (
            [
                wing[0] + self.wing_round * n[0],
                wing[1] + self.wing_round * n[1],
            ],
            [
                bottom[0] + self.bottom_round * n[0],
                bottom[1] + self.bottom_round * n[1],
            ],
        )
    }

    /// The section as one closed loop, clockwise from the top of the boss.
    pub fn section(&self) -> Vec<Seg> {
        let top = self.boss_height;
        let boss_centre = [self.boss - self.boss_round, top - self.boss_round];
        let (flank_top, flank_end) = self.flank();
        // The right-hand half, from the middle of the top to the middle of the bottom.
        let right = [
            Seg::Line([0.0, top], [boss_centre[0], top]),
            arc(
                [boss_centre[0], top],
                [self.boss, boss_centre[1]],
                boss_centre,
            ),
            Seg::Line([self.boss, boss_centre[1]], [self.boss, 0.0]),
            Seg::Line([self.boss, 0.0], [self.wing, 0.0]),
            arc([self.wing, 0.0], flank_top, self.wing_centre()),
            Seg::Line(flank_top, flank_end),
            arc(flank_end, [self.bottom, -self.body], self.bottom_centre()),
            Seg::Line([self.bottom, -self.body], [0.0, -self.body]),
        ];
        let mut segs = right.to_vec();
        // The left-hand half is the right-hand half mirrored and run backwards.
        segs.extend(right.iter().rev().map(mirrored));
        segs
    }

    /// A point inside the section, for picking the region to extrude.
    pub fn material_point(&self) -> P {
        [0.0, -self.body / 2.0]
    }
}

/// A clockwise arc of less than half a turn.
fn arc(start: P, end: P, centre: P) -> Seg {
    let radius = (start[0] - centre[0]).hypot(start[1] - centre[1]);
    let mid = [
        start[0] + end[0] - 2.0 * centre[0],
        start[1] + end[1] - 2.0 * centre[1],
    ];
    let size = mid[0].hypot(mid[1]);
    Seg::Arc {
        start,
        end,
        through: [
            centre[0] + radius * mid[0] / size,
            centre[1] + radius * mid[1] / size,
        ],
        centre,
        ccw: false,
    }
}

/// A segment mirrored across the middle of the slot and turned round, so a clockwise
/// loop stays clockwise.
fn mirrored(seg: &Seg) -> Seg {
    let flip = |p: P| [-p[0], p[1]];
    match *seg {
        Seg::Line(a, b) => Seg::Line(flip(b), flip(a)),
        Seg::Arc {
            start,
            end,
            through,
            centre,
            ccw,
        } => Seg::Arc {
            start: flip(end),
            end: flip(start),
            through: flip(through),
            centre: flip(centre),
            ccw,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::loop_area;
    use crate::slot::{Slot, HALF_CELL, T5, V5};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    fn has(segs: &[Seg], x: f64, y: f64) -> bool {
        segs.iter()
            .any(|s| close(s.start()[0], x) && close(s.start()[1], y))
    }

    /// The vertices of the section in `HNTT5-5.stp`, and the drawing's 10 by 10 and 5.8.
    #[test]
    fn the_pre_assembly_nut_is_the_misumi_hntt5() {
        let nut = PRE_ASSEMBLY;
        assert_eq!(nut.length, 10.0);
        assert!(close(nut.width(), 10.0));
        assert_eq!(nut.boss * 2.0, 5.8);
        assert!(close(nut.height(), 4.1));
        assert_eq!(nut.tap_at * 2.0, nut.length);
        let segs = nut.section();
        for (x, y) in [
            (2.7, 0.8),
            (2.9, 0.6),
            (2.9, 0.0),
            (4.8, 0.0),
            (4.9542, -0.3273),
            (2.59, -3.191),
            (2.3586, -3.3),
        ] {
            assert!(has(&segs, x, y), "({x}, {y})");
            assert!(has(&segs, -x, y), "(-{x}, {y})");
        }
        // The model's flank faces (0.7711, -0.6367).
        let n = nut.flank_normal();
        assert!(close(n[0], 0.7711) && close(n[1], -0.6367));
    }

    /// The vertices of the section in `HNTP5-5.stp`, and the drawing's 17, 8.3, 5.8,
    /// 3.2 and 3.9.
    #[test]
    fn the_post_assembly_nut_is_the_misumi_hntp5() {
        let nut = POST_ASSEMBLY;
        assert_eq!(nut.length, 17.0);
        assert!(close(nut.width(), 8.3));
        assert_eq!(nut.boss * 2.0, 5.8);
        assert_eq!(nut.body, 3.2);
        assert!(close(nut.height(), 3.9));
        assert_eq!(nut.tap_at, 6.0);
        let segs = nut.section();
        for (x, y) in [
            (2.7, 0.7),
            (2.9, 0.5),
            (2.9, 0.0),
            (3.95, 0.0),
            (4.1047, -0.3268),
            (1.84, -3.0902),
            (1.608, -3.2),
        ] {
            assert!(has(&segs, x, y), "({x}, {y})");
            assert!(has(&segs, -x, y), "(-{x}, {y})");
        }
        let n = nut.flank_normal();
        assert!(close(n[0], 0.7734) && close(n[1], -0.6339));
        // The ball is 13.5 along, an R2 sphere whose centre is 1 above the recess floor.
        let ball = nut.ball.unwrap();
        assert_eq!(nut.tap_at + ball.from_tap, 13.5);
        assert_eq!(ball.recess * 2.0, 4.2);
        assert_eq!(ball.radius - ball.stands_out - ball.recess_depth, 1.0);
    }

    /// MISUMI 2019 US catalog pp. 2710 and 2716 list the carbon-steel HNTT5
    /// and HNTP5 with M3, M4 and M5, using one envelope per family.
    #[test]
    fn both_misumi_nut_families_offer_three_threads_with_shared_envelopes() {
        assert_eq!(THREADS, ["M3", "M4", "M5"]);
        assert_eq!(
            (
                PRE_ASSEMBLY.length,
                PRE_ASSEMBLY.width(),
                PRE_ASSEMBLY.height()
            ),
            (10.0, 10.0, 4.1)
        );
        assert_eq!((POST_ASSEMBLY.length, POST_ASSEMBLY.width()), (17.0, 8.3));
        assert!(close(POST_ASSEMBLY.height(), 3.9));
        assert_eq!((PRE_ASSEMBLY.tap_at, POST_ASSEMBLY.tap_at), (5.0, 6.0));
    }

    #[test]
    fn a_section_is_one_closed_loop_with_its_through_points_on_the_arcs() {
        for nut in NUTS {
            let segs = nut.section();
            for (i, seg) in segs.iter().enumerate() {
                assert_eq!(
                    seg.end(),
                    segs[(i + 1) % segs.len()].start(),
                    "{}",
                    nut.name
                );
                if let Seg::Arc {
                    start,
                    through,
                    centre,
                    ..
                } = *seg
                {
                    let r = (start[0] - centre[0]).hypot(start[1] - centre[1]);
                    let at = (through[0] - centre[0]).hypot(through[1] - centre[1]);
                    assert!(close(r, at), "{}", nut.name);
                }
            }
            // Less than the box round it, more than the body's flat bottom alone.
            let area = loop_area(&segs);
            assert!(area < nut.width() * nut.height(), "{}", nut.name);
            assert!(area > 2.0 * nut.bottom * nut.height(), "{}", nut.name);
        }
    }

    /// Half the width of a slot's cavity at a height below the lips: the cavity wall,
    /// or lower down the web that runs in to the core.
    fn cavity_at(slot: &Slot, below: f64) -> f64 {
        let y = HALF_CELL - slot.lip - below;
        slot.cavity.min(y - slot.web_offset())
    }

    #[test]
    fn both_nuts_fit_both_slots() {
        for slot in [T5, V5] {
            let depth = HALF_CELL - slot.lip - slot.core;
            for nut in NUTS {
                let what = format!("{} in a {}", nut.name, slot.name);
                assert!(nut.boss < slot.throat, "{what}");
                assert!(nut.boss_height < slot.lip, "{what}");
                assert!(nut.wing > slot.throat, "{what}");
                assert!(nut.body <= depth, "{what}");
                let (top, end) = nut.flank();
                // The widest point, the two ends of the flank, and the bottom corner.
                for (x, y) in [
                    (nut.width() / 2.0, -nut.wing_round),
                    (top[0], top[1]),
                    (end[0], end[1]),
                    (nut.bottom, -nut.body),
                ] {
                    assert!(x < cavity_at(&slot, -y), "{what} at ({x}, {y})");
                }
                if let Some(ball) = nut.ball {
                    assert!(nut.body + ball.stands_out <= depth, "{what}");
                }
            }
        }
    }
}
