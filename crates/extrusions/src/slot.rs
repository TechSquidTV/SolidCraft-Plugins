//! The slot shapes of the 20 mm series. Sizes are millimetres.
//!
//! Both rails share the 20 mm cell and the 4.2 mm bore, so one grid serves both. The
//! slots differ: a V-slot is 6.25 mm at the throat under a 90° chamfer, and its cavity
//! is 11 mm wide under a 1.8 mm lip, where the T-slot is 6 mm, 12 mm and 2 mm.

/// Half a cell. A cell is 20 mm square.
pub const HALF_CELL: f64 = 10.0;

/// One slot shape, and the core and webs that go with it. Every size but `web` is
/// measured from the cell centre or from the outer face, as the field says.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// What a name and the dialog call it: `T-slot`, `V-slot`.
    pub name: &'static str,
    /// Half the width of the slot where it is narrowest.
    pub throat: f64,
    /// Half the width of the slot at the outer face. Wider than `throat` on a V-slot,
    /// where a 45° chamfer joins the two.
    pub mouth: f64,
    /// Thickness of the lip, from the outer face to the cavity under it. A wall between
    /// two cells is this thick too.
    pub lip: f64,
    /// Half the width of the cavity under the lip.
    pub cavity: f64,
    /// Half the side of the square core. The slot floor is a core face.
    pub core: f64,
    /// Thickness of the web from a core corner to the rail's corner or wall.
    pub web: f64,
    /// A V groove along the middle of the slot floor, as (half width, depth). Zero for none.
    pub groove: (f64, f64),
    /// Diameter of the bore through each core.
    pub bore: f64,
    /// Radius of the rail's four outer corners.
    pub corner: f64,
}

impl Slot {
    /// How far a web edge sits from the diagonal it runs along, along x or y.
    pub fn web_offset(&self) -> f64 {
        self.web / std::f64::consts::SQRT_2
    }

    /// Where a web edge leaves a core face, from the cell centre.
    pub fn web_root(&self) -> f64 {
        self.core - self.web_offset()
    }

    /// The inside face of a wall between two cells, from the cell centre.
    pub fn wall(&self) -> f64 {
        HALF_CELL - self.lip
    }

    /// The far face of the leg that closes a slot cavity beside a neighbouring cell,
    /// from the cell centre. The leg is as thick as the lip.
    pub fn leg(&self) -> f64 {
        self.cavity + self.lip
    }
}

/// MISUMI 5 series T-slot (HFS5, NFS5).
///
/// Source: MISUMI catalog 2019 US p. 2686 to 2687, "Slot Details (Common to All Series)"
/// (6 mm slot, 2 mm lip, 12 mm cavity, 4 mm from the lip to the floor) and the HFS5-2020
/// and HFS5-4040 sections (8 mm core, Ø4.2 bore, 4-R1 corners, 1.5 mm webs). The HFS and
/// CAF rails also have a 0.3 mm groove along the slot floor, which the NFS rails do not
/// have and which is left out here.
pub const T5: Slot = Slot {
    name: "T-slot",
    throat: 3.0,
    mouth: 3.0,
    lip: 2.0,
    cavity: 6.0,
    core: 4.0,
    web: 1.5,
    groove: (0.0, 0.0),
    bore: 4.2,
    corner: 1.0,
};

/// OpenBuilds V-Slot.
///
/// Source: OpenBuilds "V-Slot 20x20 Linear Rail" drawing (6.25 throat, 9.16 mouth, 11
/// cavity, 1.80 lip, 4.30 from the lip to the floor, 1.50 web, Ø4.20 bore) and the
/// vertices of `V-Slot 20x20.dxf` (7.8 mm core, R0.5 corners, a 0.21 mm groove along
/// each slot floor).
pub const V5: Slot = Slot {
    name: "V-slot",
    throat: 3.125,
    mouth: 4.58,
    lip: 1.8,
    cavity: 5.5,
    core: 3.9,
    web: 1.5,
    groove: (0.21, 0.21),
    bore: 4.2,
    corner: 0.5,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    /// The MISUMI slot detail, repeated from the catalog page.
    #[test]
    fn the_t_slot_is_the_misumi_5_series_slot() {
        assert_eq!(T5.throat * 2.0, 6.0);
        assert_eq!(T5.mouth, T5.throat);
        assert_eq!(T5.lip, 2.0);
        assert_eq!(T5.cavity * 2.0, 12.0);
        // 4 mm from under the lip to the floor.
        assert_eq!(HALF_CELL - T5.lip - T5.core, 4.0);
        assert_eq!(T5.core * 2.0, 8.0);
        assert_eq!(T5.bore, 4.2);
        assert_eq!(T5.corner, 1.0);
        assert_eq!(T5.web, 1.5);
    }

    /// The OpenBuilds drawing, and the DXF vertices the drawing does not dimension.
    #[test]
    fn the_v_slot_is_the_openbuilds_slot() {
        assert_eq!(V5.throat * 2.0, 6.25);
        assert_eq!(V5.mouth * 2.0, 9.16);
        assert_eq!(V5.cavity * 2.0, 11.0);
        assert_eq!(V5.lip, 1.8);
        assert!(close(HALF_CELL - V5.lip - V5.core, 4.3));
        assert_eq!(V5.web, 1.5);
        assert_eq!(V5.bore, 4.2);
        // DXF: the web leaves the core at (2.8393, 3.9) and the corner arcs are R0.5.
        assert!(close(V5.web_root(), 2.8393));
        assert_eq!(V5.corner, 0.5);
        // DXF, 20x40: the closed cavity runs (6.2393, 2.7) to (8.2, 2.7) from its centre.
        assert!(close(V5.wall(), 8.2));
        assert!(close(HALF_CELL - V5.leg(), 2.7));
        assert!(close(V5.leg() - V5.web_offset(), 6.2393));
    }

    #[test]
    fn the_two_rails_share_the_cell_and_the_bore() {
        assert_eq!(T5.bore, V5.bore);
        assert_eq!(T5.web, V5.web);
    }
}
