//! What is cut into a rail so that rails screw together with no bracket: a tap in the
//! bores of an end, and screw holes across the rail. Sizes are millimetres.
//!
//! A screw goes in through a slot of one rail, across its core, and into the tapped end
//! of the next. Its head is wider than the slot, so an access hole through the lips lets
//! it down onto the slot floor, which it bears on.
//!
//! A point of a section is (x, y): x along the rail's width, y along its height.

use crate::profile::{Grid, P};
use crate::slot::{Slot, HALF_CELL};

/// The thread a rail's bore is tapped with. Both rails have a 4.2 mm bore, which is the
/// tap drill of M5.
pub const TAP_THREAD: &str = "M5";

/// How deep an end is tapped. An M5 x 25 screw seated on the slot floor of a 2020 rail
/// reaches 11 mm into the end it holds.
pub const TAP_DEPTH: f64 = 12.0;

/// The hole an M5 screw passes through: ISO 273, medium fit.
pub const CLEARANCE: f64 = 5.5;

/// The hole that lets the head of an M5 socket head screw through the lips: the 8.5 mm
/// head of ISO 4762 and 0.5 mm round it, as the screw plugin recesses a head.
pub const ACCESS: f64 = 9.5;

/// The side of a rail a screw goes in from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    XPlus,
    XMinus,
    YPlus,
    YMinus,
}

/// The faces, in the order the dialog lists them.
pub const FACES: [Face; 4] = [Face::YPlus, Face::YMinus, Face::XPlus, Face::XMinus];

impl Face {
    /// What a name and the dialog call it: `+Y`.
    pub fn name(&self) -> &'static str {
        match self {
            Face::XPlus => "+X",
            Face::XMinus => "-X",
            Face::YPlus => "+Y",
            Face::YMinus => "-Y",
        }
    }

    pub fn of(name: &str) -> Option<Face> {
        FACES.into_iter().find(|face| face.name() == name)
    }

    /// The direction the face looks, out of the rail.
    pub fn outward(&self) -> P {
        match self {
            Face::XPlus => [1.0, 0.0],
            Face::XMinus => [-1.0, 0.0],
            Face::YPlus => [0.0, 1.0],
            Face::YMinus => [0.0, -1.0],
        }
    }

    fn along_x(&self) -> bool {
        matches!(self, Face::XPlus | Face::XMinus)
    }

    /// How far the face is from the middle of the rail.
    pub fn half(&self, grid: &Grid) -> f64 {
        if self.along_x() {
            grid.width() / 2.0
        } else {
            grid.height() / 2.0
        }
    }

    /// The middle of every slot on the face, as points on the face.
    pub fn slots(&self, grid: &Grid) -> Vec<P> {
        let half = self.half(grid);
        let out = self.outward();
        let mut across: Vec<f64> = grid
            .centres()
            .iter()
            .map(|c| if self.along_x() { c[1] } else { c[0] })
            .collect();
        across.sort_by(f64::total_cmp);
        across.dedup();
        across
            .into_iter()
            .map(|t| {
                if self.along_x() {
                    [out[0] * half, t]
                } else {
                    [t, out[1] * half]
                }
            })
            .collect()
    }
}

/// One screw hole across a rail, from a slot of `face`: two cylinders that both run in
/// from the face along its inward normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrossHole {
    /// Where the hole starts, in the middle of a slot on the face.
    pub at: P,
    /// The direction it runs, into the rail.
    pub inward: P,
    /// The access hole, from the face down to the slot floor: (diameter, depth).
    pub access: (f64, f64),
    /// The clearance hole, from the slot floor out through the far face: (diameter,
    /// where it starts below the face, how long it is).
    pub clearance: (f64, f64, f64),
}

impl CrossHole {
    /// How far below the face a screw head sits: on the slot floor.
    pub fn seat(slot: &Slot) -> f64 {
        HALF_CELL - slot.core
    }
}

/// The screw holes one position along a rail gets: one through each slot of `face`.
pub fn cross_holes(slot: &Slot, grid: &Grid, face: Face) -> Vec<CrossHole> {
    let out = face.outward();
    let seat = CrossHole::seat(slot);
    face.slots(grid)
        .into_iter()
        .map(|at| CrossHole {
            at,
            inward: [-out[0], -out[1]],
            access: (ACCESS, seat),
            clearance: (CLEARANCE, seat, 2.0 * face.half(grid) - seat),
        })
        .collect()
}

/// The closest a screw hole may be to an end: its access hole and a millimetre of metal.
pub fn hole_margin() -> f64 {
    ACCESS / 2.0 + 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::{T5, V5};

    /// ISO 273 medium for M5 is 5.5, ISO 4762 gives an M5 head 8.5 across, and the bore of
    /// both rails is the 4.2 mm tap drill of M5 (5 less its 0.8 pitch).
    #[test]
    fn the_holes_are_for_an_m5_socket_head_screw() {
        assert_eq!(CLEARANCE, 5.5);
        assert_eq!(ACCESS, 8.5 + 2.0 * 0.5);
        assert_eq!(TAP_THREAD, "M5");
        assert!((T5.bore - (5.0 - 0.8)).abs() < 1e-9);
        assert_eq!(V5.bore, T5.bore);
        // The screw passes the slot, and its head does not.
        for slot in [T5, V5] {
            assert!(CLEARANCE < 2.0 * slot.throat, "{}", slot.name);
            assert!(ACCESS > 2.0 * slot.throat, "{}", slot.name);
        }
    }

    #[test]
    fn a_screw_head_sits_on_the_slot_floor() {
        assert_eq!(CrossHole::seat(&T5), 6.0);
        assert!((CrossHole::seat(&V5) - 6.1).abs() < 1e-9);
        // An M5 x 25 through a 2020 then holds 11 mm of the next rail's end.
        assert_eq!(25.0 - (20.0 - CrossHole::seat(&T5)), 11.0);
        let reach = 25.0 - (2.0 * HALF_CELL - CrossHole::seat(&T5));
        assert!(TAP_DEPTH > reach);
    }

    #[test]
    fn a_2020_has_one_slot_a_face_and_a_2040_two_on_its_wide_faces() {
        let square = Grid { nx: 1, ny: 1 };
        for face in FACES {
            assert_eq!(face.slots(&square).len(), 1);
            assert_eq!(Face::of(face.name()), Some(face));
        }
        assert_eq!(Face::YPlus.slots(&square), [[0.0, 10.0]]);
        assert_eq!(Face::XMinus.slots(&square), [[-10.0, 0.0]]);
        let tall = Grid { nx: 1, ny: 2 };
        assert_eq!(Face::XPlus.slots(&tall), [[10.0, -10.0], [10.0, 10.0]]);
        assert_eq!(Face::YMinus.slots(&tall), [[0.0, -20.0]]);
        assert_eq!(Face::of("top"), None);
    }

    #[test]
    fn a_hole_runs_from_the_face_through_the_whole_rail() {
        let holes = cross_holes(&T5, &Grid { nx: 1, ny: 1 }, Face::YMinus);
        assert_eq!(holes.len(), 1);
        let hole = holes[0];
        assert_eq!(hole.at, [0.0, -10.0]);
        assert_eq!(hole.inward, [0.0, 1.0]);
        assert_eq!(hole.access, (9.5, 6.0));
        assert_eq!(hole.clearance, (5.5, 6.0, 14.0));
        // Across the 40 mm of a 2040 the clearance hole is 20 mm longer.
        let long = cross_holes(&T5, &Grid { nx: 1, ny: 2 }, Face::YPlus)[0];
        assert_eq!(long.clearance, (5.5, 6.0, 34.0));
    }
}
