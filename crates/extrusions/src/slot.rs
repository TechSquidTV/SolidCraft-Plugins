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
    /// What runs along the middle of the slot floor.
    pub floor: Floor,
    /// Diameter of the bore through each core.
    pub bore: f64,
    /// Radius of the rail's four outer corners.
    pub corner: f64,
}

/// The shape of a slot floor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Floor {
    /// A plain V groove with sharp edges, as (half width, depth).
    Vee(f64, f64),
    /// The MISUMI 5 series floor: [`MisumiFloor`].
    Misumi(MisumiFloor),
}

/// The MISUMI 5 series slot floor. A groove with flanks at `flank` degrees to the floor
/// runs along the middle, `depth` deep, and both its edges are rounded to `round`. Going
/// anticlockwise round the rail, the second rounded edge runs straight on down into a
/// shallower mark, whose far side rises at `rise` degrees to meet the floor `mark_end`
/// from the middle.
///
/// Source: the end faces of MISUMI's STEP models NFS5-2040, NFS5-2060, NFS5-2080,
/// NFS5-4040 and NFSF5-2020, which all draw a floor at y = 4 as (0.6536, 4), an R0.5
/// arc to (0.4036, 3.933), (0, 3.7), (-0.4036, 3.933), an R0.5 arc to (-0.9036, 3.933),
/// (-1.2189, 3.751), (-2.5, 4). The catalog's Slot Bottom Details gives the 120° and
/// the 0.3.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MisumiFloor {
    pub depth: f64,
    pub flank: f64,
    pub round: f64,
    pub rise: f64,
    pub mark_end: f64,
}

impl MisumiFloor {
    /// How far from the middle each rounding's centre is.
    pub fn round_at(&self) -> f64 {
        let flank = self.flank.to_radians();
        self.round * flank.sin() + (self.depth - self.round * (1.0 - flank.cos())) / flank.tan()
    }

    /// Where a rounding meets a flank, as (distance from the middle, depth below the floor).
    pub fn flank_top(&self) -> (f64, f64) {
        let flank = self.flank.to_radians();
        (
            self.round_at() - self.round * flank.sin(),
            self.round * (1.0 - flank.cos()),
        )
    }

    /// The bottom of the mark, as (distance from the middle, depth below the floor).
    pub fn mark(&self) -> (f64, f64) {
        let (flank, rise) = (self.flank.to_radians().tan(), self.rise.to_radians().tan());
        // The flank carries on down from the far side of the second rounding.
        let (t0, d0) = (
            self.round_at() + self.round * self.flank.to_radians().sin(),
            self.flank_top().1,
        );
        let t = (rise * self.mark_end - d0 + flank * t0) / (flank + rise);
        (t, rise * (self.mark_end - t))
    }
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
/// and HFS5-4040 sections (8 mm core, Ø4.2 bore, 4-R1 corners, 1.5 mm webs), and "Slot
/// Bottom Details" (a 120° groove 0.3 mm deep along the slot floor). MISUMI's STEP
/// models have the same vertices, and the floor is theirs: see [`MisumiFloor`].
pub const T5: Slot = Slot {
    name: "T-slot",
    throat: 3.0,
    mouth: 3.0,
    lip: 2.0,
    cavity: 6.0,
    core: 4.0,
    web: 1.5,
    floor: Floor::Misumi(MisumiFloor {
        depth: 0.3,
        flank: 30.0,
        round: 0.5,
        rise: 11.0,
        mark_end: 2.5,
    }),
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
    floor: Floor::Vee(0.21, 0.21),
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

    /// The catalog's Slot Bottom Details (120° and 0.3 deep) and the floor vertices of
    /// the STEP models.
    #[test]
    fn the_t_slot_floor_is_the_misumi_floor() {
        let Floor::Misumi(floor) = T5.floor else {
            panic!("the T-slot has the MISUMI floor");
        };
        assert_eq!(floor.depth, 0.3);
        assert_eq!(floor.flank * 2.0 + 120.0, 180.0);
        assert_eq!(floor.round, 0.5);
        assert!(close(floor.round_at(), 0.6536));
        let (t, d) = floor.flank_top();
        assert!(close(t, 0.4036) && close(4.0 - d, 3.933));
        let (t, d) = floor.mark();
        assert!(close(t, 1.2189) && close(4.0 - d, 3.751));
        assert_eq!(floor.mark_end, 2.5);
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
