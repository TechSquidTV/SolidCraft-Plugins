//! The cross-section of a rail, built from a slot shape and a grid of cells.
//!
//! The section is centred on the origin. A `2040` is one cell along X and two along Y,
//! so it is 20 mm wide and 40 mm tall, the way the catalogs draw it.
//!
//! The outline walks the four sides anticlockwise and dips into a slot at every cell.
//! Between two cells of a single row the webs and walls close a cavity. In a 2 x 2 rail
//! no web runs to the middle, so the four cavities are one, and each core is open to it:
//! a wedge of the core that faces the middle is left out, which makes the bore a C.

use crate::slot::{Floor, Slot, HALF_CELL};

pub type P = [f64; 2];

const CELL: f64 = 2.0 * HALF_CELL;

/// How many cells a rail has along X and along Y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub nx: usize,
    pub ny: usize,
}

/// The sizes a rail comes in, by the name on a parts list.
///
/// Sources: MISUMI catalog 2019 US p. 2686 to 2687 draws HFS5-2020, HFS5-2040 and
/// HFS5-4040, and MISUMI's STEP models give NFS5-2020, NFS5-2040, NFS5-2060,
/// NFS5-2080 and NFS5-4040. OpenBuilds publishes V-Slot 20x20, 20x40, 20x60 and 20x80.
pub const SIZES: [(&str, Grid); 5] = [
    ("2020", Grid { nx: 1, ny: 1 }),
    ("2040", Grid { nx: 1, ny: 2 }),
    ("2060", Grid { nx: 1, ny: 3 }),
    ("2080", Grid { nx: 1, ny: 4 }),
    ("4040", Grid { nx: 2, ny: 2 }),
];

impl Grid {
    /// The grid of a listed size, such as `2040`.
    pub fn of(size: &str) -> Option<Grid> {
        SIZES.iter().find(|(name, _)| *name == size).map(|s| s.1)
    }

    pub fn width(&self) -> f64 {
        CELL * self.nx as f64
    }

    pub fn height(&self) -> f64 {
        CELL * self.ny as f64
    }

    /// The centre of every cell, row by row from the bottom left.
    pub fn centres(&self) -> Vec<P> {
        let mut out = Vec::new();
        for j in 0..self.ny {
            for i in 0..self.nx {
                out.push([
                    -self.width() / 2.0 + HALF_CELL + CELL * i as f64,
                    -self.height() / 2.0 + HALF_CELL + CELL * j as f64,
                ]);
            }
        }
        out
    }

    /// Whether [`Profile::new`] can build this grid: one row of up to four cells, or 2 x 2.
    pub fn supported(&self) -> bool {
        (self.nx == 1 && (1..=4).contains(&self.ny)) || (self.nx == 2 && self.ny == 2)
    }
}

/// One piece of the outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seg {
    Line(P, P),
    /// A circular arc from `start` to `end` by way of `through`, less than a full turn.
    Arc {
        start: P,
        end: P,
        through: P,
        centre: P,
        /// Whether it runs anticlockwise.
        ccw: bool,
    },
}

impl Seg {
    pub fn start(&self) -> P {
        match self {
            Seg::Line(a, _) => *a,
            Seg::Arc { start, .. } => *start,
        }
    }

    pub fn end(&self) -> P {
        match self {
            Seg::Line(_, b) => *b,
            Seg::Arc { end, .. } => *end,
        }
    }
}

/// A rail's cross-section.
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub slot: Slot,
    pub grid: Grid,
    /// The outline, anticlockwise and closed.
    pub outer: Vec<Seg>,
    /// The closed cavities between cells, each a closed loop.
    pub cavities: Vec<Vec<Seg>>,
    /// The centre of each round bore. Its diameter is `slot.bore`. A 2 x 2 rail has
    /// none: its bores are part of the middle cavity.
    pub bores: Vec<P>,
}

impl Profile {
    /// The section of a rail. `None` when the grid is not one [`Grid::supported`] builds.
    pub fn new(slot: Slot, grid: Grid) -> Option<Profile> {
        if !grid.supported() {
            return None;
        }
        Some(Profile {
            slot,
            grid,
            outer: outline(&slot, &grid),
            cavities: cavities(&slot, &grid),
            bores: if grid.nx == 1 {
                grid.centres()
            } else {
                Vec::new()
            },
        })
    }

    /// The area of the section in mm².
    pub fn area(&self) -> f64 {
        let bore = std::f64::consts::PI * (self.slot.bore / 2.0).powi(2);
        loop_area(&self.outer)
            - self.cavities.iter().map(|c| loop_area(c)).sum::<f64>()
            - bore * self.bores.len() as f64
    }

    /// A point inside the metal, for picking the region to extrude: in the first cell's
    /// core, clear of its bore and its faces, on the side of the rail's corner. The side
    /// that faces the middle of a 2 x 2 rail is open.
    pub fn material_point(&self) -> P {
        let at = (self.slot.bore / 2.0 + self.slot.web_root()) / 2.0;
        let c = self.grid.centres()[0];
        [c[0] - at, c[1] - at]
    }
}

/// One slot in a side's own frame: each point is (distance along the side from the cell
/// centre, depth below the face). The side runs anticlockwise round the rail, and an arc
/// that is anticlockwise here is anticlockwise in the section.
fn slot_segs(s: &Slot) -> Vec<Seg> {
    let floor = HALF_CELL - s.core;
    // Where the web edge meets the cavity wall.
    let shoulder = HALF_CELL - s.cavity - s.web_offset();
    let mut left = vec![[-s.mouth, 0.0]];
    if s.mouth > s.throat {
        left.push([-s.throat, s.mouth - s.throat]);
    }
    left.extend([
        [-s.throat, s.lip],
        [-s.cavity, s.lip],
        [-s.cavity, shoulder],
        [-s.web_root(), floor],
    ]);
    let right: Vec<P> = left.iter().rev().map(|p| [-p[0], p[1]]).collect();
    let mut segs = chain(&left);
    match s.floor {
        Floor::Vee(half, depth) => {
            let mut points = vec![left[left.len() - 1]];
            points.extend([[-half, floor], [0.0, floor + depth], [half, floor]]);
            points.push(right[0]);
            segs.extend(chain(&points));
        }
        Floor::Misumi(m) => {
            let (at, r) = (m.round_at(), m.round);
            let (top, drop) = m.flank_top();
            let (mark, mark_depth) = m.mark();
            // A point on a rounding centred `centre` from the middle, `degrees` round
            // from straight up towards the face, leaning the way the side runs.
            let on = |centre: f64, degrees: f64| -> P {
                let a = degrees.to_radians();
                [centre + r * a.sin(), floor + r - r * a.cos()]
            };
            let arc = |centre: f64, from: f64, to: f64| Seg::Arc {
                start: on(centre, from),
                end: on(centre, to),
                through: on(centre, (from + to) / 2.0),
                centre: [centre, floor + r],
                ccw: true,
            };
            segs.push(Seg::Line(left[left.len() - 1], [-at, floor]));
            segs.push(arc(-at, 0.0, m.flank));
            segs.extend(chain(&[
                [-top, floor + drop],
                [0.0, floor + m.depth],
                [top, floor + drop],
            ]));
            segs.push(arc(at, -m.flank, m.flank));
            segs.extend(chain(&[
                on(at, m.flank),
                [mark, floor + mark_depth],
                [m.mark_end, floor],
                right[0],
            ]));
        }
    }
    segs.extend(chain(&right));
    segs
}

fn outline(s: &Slot, grid: &Grid) -> Vec<Seg> {
    let (w, h) = (grid.width(), grid.height());
    let r = s.corner;
    // Each side: where it starts, the way it runs, the way out, and its cells.
    let sides: [(P, P, P, usize); 4] = [
        ([-w / 2.0, -h / 2.0], [1.0, 0.0], [0.0, -1.0], grid.nx),
        ([w / 2.0, -h / 2.0], [0.0, 1.0], [1.0, 0.0], grid.ny),
        ([w / 2.0, h / 2.0], [-1.0, 0.0], [0.0, 1.0], grid.nx),
        ([-w / 2.0, h / 2.0], [0.0, -1.0], [-1.0, 0.0], grid.ny),
    ];
    let slot = slot_segs(s);
    let mut segs = Vec::new();
    for (start, e, n, cells) in sides {
        let length = CELL * cells as f64;
        let at = |t: f64, d: f64| -> P {
            [
                start[0] + t * e[0] - d * n[0],
                start[1] + t * e[1] - d * n[1],
            ]
        };
        let mut last = at(r, 0.0);
        for cell in 0..cells {
            let centre = HALF_CELL + CELL * cell as f64;
            let place = |p: P| at(centre + p[0], p[1]);
            segs.push(Seg::Line(last, place(slot[0].start())));
            segs.extend(slot.iter().map(|seg| map(seg, &place)));
            last = place(slot[slot.len() - 1].end());
        }
        segs.push(Seg::Line(last, at(length - r, 0.0)));
        // The next side runs to the left of this one.
        let next = [-e[1], e[0]];
        let corner = at(length, 0.0);
        let centre = [
            corner[0] - r * e[0] + r * next[0],
            corner[1] - r * e[1] + r * next[1],
        ];
        let k = r / std::f64::consts::SQRT_2;
        segs.push(Seg::Arc {
            start: at(length - r, 0.0),
            end: [corner[0] + r * next[0], corner[1] + r * next[1]],
            through: [
                centre[0] + k * (e[0] - next[0]),
                centre[1] + k * (e[1] - next[1]),
            ],
            centre,
            ccw: true,
        });
    }
    segs
}

/// How a core of a 2 x 2 rail opens to the middle cavity. Two straight cuts run from the
/// bore's centre line out to the core's two inner faces, and each meets its face this
/// share of the core's half width past the centre line. The wedge between them, 53.13°
/// wide and facing the middle of the rail, is open.
///
/// Source: MISUMI STEP model `NFS5-4040-50.stp`, whose cuts run from (8.6667, 6) to
/// (9.3359, 8.0078) on the bore of the core at (10, 10), and mirrored.
pub const OPEN_CORE_SHARE: f64 = 1.0 / 3.0;

fn cavities(s: &Slot, grid: &Grid) -> Vec<Vec<Seg>> {
    let face = HALF_CELL - s.core;
    let leg = HALF_CELL - s.leg();
    let root = s.web_root();
    let knee = s.leg() - s.web_offset();
    let wall = s.wall();
    if grid.nx == 1 {
        // One cavity between each pair of cells, drawn round the point between them.
        let right = [[root, face], [knee, leg], [wall, leg]];
        return (1..grid.ny)
            .map(|j| {
                let mid = -grid.height() / 2.0 + CELL * j as f64;
                let mut points: Vec<P> = Vec::new();
                // Clockwise: top right, bottom right, bottom left, top left.
                points.extend(right.iter().map(|p| [p[0], p[1]]));
                points.extend(right.iter().rev().map(|p| [p[0], -p[1]]));
                points.extend(right.iter().map(|p| [-p[0], -p[1]]));
                points.extend(right.iter().rev().map(|p| [-p[0], p[1]]));
                let points: Vec<P> = points.iter().map(|p| [p[0], p[1] + mid]).collect();
                polygon(&points)
            })
            .collect();
    }
    // 2 x 2: the part of the middle cavity in the top right cell, about that cell's
    // centre. It runs from the top wall down the web to the core, round the bore the
    // long way, and out along the other web to the right wall.
    let (f, c) = (s.leg(), s.core);
    let r = s.bore / 2.0;
    // Where a cut meets its face, and how far it leans from that face's normal.
    let e = c * OPEN_CORE_SHARE;
    let lean = e.atan2(c).to_degrees();
    let k = r / std::f64::consts::SQRT_2;
    let d = e.hypot(c);
    let (into, far, out) = ([-r * c / d, -r * e / d], [k, k], [-r * e / d, -r * c / d]);
    // A point on the bore at `degrees` anticlockwise from +X.
    let on = |degrees: f64| -> P {
        let a = degrees.to_radians();
        [r * a.cos(), r * a.sin()]
    };
    let arc = |start: P, end: P, through: P| Seg::Arc {
        start,
        end,
        through,
        centre: [0.0, 0.0],
        ccw: false,
    };
    let mut quarter = chain(&[[-f, wall], [-f, knee], [-c, root], [-c, -e], into]);
    // Clockwise from just below -X round to +X +Y, then on to just left of -Y.
    quarter.push(arc(into, far, on((180.0 + lean + 45.0) / 2.0)));
    quarter.push(arc(far, out, on((45.0 - 90.0 - lean) / 2.0)));
    quarter.extend(chain(&[out, [-e, -c], [root, -c], [knee, -f], [wall, -f]]));
    let moved = |p: P| [p[0] + HALF_CELL, p[1] + HALF_CELL];
    // A quarter turn clockwise about the rail's centre gives the next cell's part.
    let turned = |p: P| [p[1], -p[0]];
    let mut part: Vec<Seg> = quarter.iter().map(|seg| map(seg, &moved)).collect();
    let mut loop_: Vec<Seg> = Vec::new();
    for _ in 0..4 {
        if let Some(last) = loop_.last() {
            loop_.push(Seg::Line(last.end(), part[0].start()));
        }
        loop_.extend(part.iter().cloned());
        part = part.iter().map(|seg| map(seg, &turned)).collect();
    }
    loop_.push(Seg::Line(loop_[loop_.len() - 1].end(), loop_[0].start()));
    vec![loop_]
}

/// The lines that join `points` in order, left open.
fn chain(points: &[P]) -> Vec<Seg> {
    points.windows(2).map(|w| Seg::Line(w[0], w[1])).collect()
}

/// The closed loop of lines round `points`.
fn polygon(points: &[P]) -> Vec<Seg> {
    let n = points.len();
    (0..n)
        .map(|i| Seg::Line(points[i], points[(i + 1) % n]))
        .collect()
}

/// `seg` with every point moved by `f`, which must keep the way an arc turns.
fn map(seg: &Seg, f: &dyn Fn(P) -> P) -> Seg {
    match *seg {
        Seg::Line(a, b) => Seg::Line(f(a), f(b)),
        Seg::Arc {
            start,
            end,
            through,
            centre,
            ccw,
        } => Seg::Arc {
            start: f(start),
            end: f(end),
            through: f(through),
            centre: f(centre),
            ccw,
        },
    }
}

/// The area inside a closed loop, whichever way round it is drawn. Each arc adds or
/// takes the sliver between its chord and the circle.
pub fn loop_area(segs: &[Seg]) -> f64 {
    let mut twice = 0.0;
    for seg in segs {
        let (a, b) = (seg.start(), seg.end());
        twice += a[0] * b[1] - b[0] * a[1];
        if let Seg::Arc {
            start,
            end,
            centre,
            ccw,
            ..
        } = *seg
        {
            let angle = |p: P| (p[1] - centre[1]).atan2(p[0] - centre[0]);
            let radius = (start[0] - centre[0]).hypot(start[1] - centre[1]);
            let turn = std::f64::consts::TAU;
            // The angle swept, as a positive number, the way the arc runs.
            let sweep = if ccw {
                (angle(end) - angle(start)).rem_euclid(turn)
            } else {
                (angle(start) - angle(end)).rem_euclid(turn)
            };
            let sliver = radius * radius * (sweep - sweep.sin());
            twice += if ccw { sliver } else { -sliver };
        }
    }
    (twice / 2.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::{T5, V5};

    fn profile(slot: Slot, size: &str) -> Profile {
        Profile::new(slot, Grid::of(size).unwrap()).unwrap()
    }

    fn close(a: f64, b: f64, within: f64) -> bool {
        (a - b).abs() <= within
    }

    fn has(points: &[P], x: f64, y: f64) -> bool {
        points
            .iter()
            .any(|p| close(p[0], x, 1e-3) && close(p[1], y, 1e-3))
    }

    #[test]
    fn the_outline_is_closed() {
        for slot in [T5, V5] {
            for (size, grid) in SIZES {
                let p = Profile::new(slot, grid).unwrap();
                for (i, seg) in p.outer.iter().enumerate() {
                    let next = p.outer[(i + 1) % p.outer.len()].start();
                    let end = seg.end();
                    assert!(
                        close(end[0], next[0], 1e-9) && close(end[1], next[1], 1e-9),
                        "{} {size} at {i}",
                        slot.name
                    );
                }
            }
        }
    }

    #[test]
    fn a_rail_is_as_wide_and_tall_as_its_name() {
        for (size, w, h) in [
            ("2020", 20.0, 20.0),
            ("2040", 20.0, 40.0),
            ("2060", 20.0, 60.0),
            ("2080", 20.0, 80.0),
            ("4040", 40.0, 40.0),
        ] {
            let p = profile(T5, size);
            let xs: Vec<f64> = p.outer.iter().map(|s| s.start()[0]).collect();
            let ys: Vec<f64> = p.outer.iter().map(|s| s.start()[1]).collect();
            let span = |v: &[f64]| {
                v.iter().cloned().fold(f64::MIN, f64::max)
                    - v.iter().cloned().fold(f64::MAX, f64::min)
            };
            assert_eq!(span(&xs), w, "{size}");
            assert_eq!(span(&ys), h, "{size}");
        }
    }

    /// Vertices of the end face of MISUMI's STEP model `NFS5-2040-50.stp`, in the top
    /// right quarter: the top cell's slots, its core and webs, and the closed cavity.
    #[test]
    fn the_t_slot_2040_has_the_misumi_step_vertices() {
        let p = profile(T5, "2040");
        let outer: Vec<P> = p.outer.iter().map(|s| s.start()).collect();
        for (x, y) in [
            (10.0, 7.0),
            (8.0, 7.0),
            (8.0, 4.0),
            (7.0607, 4.0),
            (4.0, 7.0607),
            (3.7, 10.0),
            (4.0, 12.9393),
            (7.0607, 16.0),
            (8.0, 16.0),
            (8.0, 13.0),
            (10.0, 13.0),
            (10.0, 19.0),
            (9.0, 20.0),
            (3.0, 20.0),
            (3.0, 18.0),
            (6.0, 18.0),
            (6.0, 17.0607),
            (2.9393, 14.0),
            (0.0, 13.7),
        ] {
            assert!(has(&outer, x, y), "({x}, {y})");
        }
        // The STEP model rounds the cavity's corners to R0.3; these are the sharp ones.
        let cavity: Vec<P> = p.cavities[0].iter().map(|s| s.start()).collect();
        for (x, y) in [(2.9393, 6.0), (6.9393, 2.0), (8.0, 2.0), (8.0, -2.0)] {
            assert!(has(&cavity, x, y), "({x}, {y})");
        }
        // The STEP cavity is 142.96 mm² with its corners rounded.
        assert!(close(loop_area(&p.cavities[0]), 142.96, 0.08));
    }

    /// MISUMI catalog 2019 US p. 2686 to 2687, "Sectional Area mm²": HFS5-2020 183,
    /// HFS5-2040 324. The catalog is a loose check: its 2040 is 0.5% over MISUMI's own
    /// STEP model and its 4040, at 472, is 1.5% under. The sections here are held to the
    /// models, and to the catalog within 1%.
    #[test]
    fn the_t_slot_areas_are_near_the_misumi_catalog_areas() {
        for (size, area) in [("2020", 183.0), ("2040", 324.0)] {
            let got = profile(T5, size).area();
            assert!(close(got, area, area * 0.01), "{size}: {got}");
        }
    }

    /// The area of the end face of each MISUMI STEP model: `NFS5-2020-50.stp`,
    /// `NFS5-2040-50.stp`, `NFS5-2060-2060.stp`, `NFS5-2080-50.stp` and
    /// `NFS5-4040-50.stp`, summed from
    /// vertices read to four decimals. The 2040 model rounds its cavity's corners to
    /// R0.3, which adds 0.07 mm² of metal.
    #[test]
    fn the_t_slot_areas_are_the_misumi_step_areas() {
        for (size, area, within) in [
            ("2020", 181.264, 0.03),
            ("2040", 322.438, 0.1),
            ("2060", 463.486, 0.03),
            ("2080", 604.595, 0.03),
            ("4040", 479.131, 0.03),
        ] {
            let got = profile(T5, size).area();
            assert!(close(got, area, within), "{size}: {got}");
        }
    }

    /// Vertices of the end face of MISUMI's STEP model `NFS5-2020-50.stp`, whose outline
    /// has 76 edges: the right slot, the corner above it and the top slot. The open
    /// slots of `NFSF5-2020-50.stp`, whose fourth face is closed, are the same.
    #[test]
    fn the_t_slot_2020_has_the_misumi_step_vertices() {
        let p = profile(T5, "2020");
        let outer: Vec<P> = p.outer.iter().map(|s| s.start()).collect();
        for (x, y) in [
            (10.0, -3.0),
            (8.0, -3.0),
            (8.0, -6.0),
            (7.0607, -6.0),
            (4.0, -2.9393),
            (4.0, -0.6536),
            (3.933, -0.4036),
            (3.7, 0.0),
            (3.933, 0.4036),
            (3.933, 0.9036),
            (3.751, 1.2189),
            (4.0, 2.5),
            (4.0, 2.9393),
            (7.0607, 6.0),
            (8.0, 6.0),
            (8.0, 3.0),
            (10.0, 3.0),
            (10.0, 9.0),
            (9.0, 10.0),
            (3.0, 10.0),
            (3.0, 8.0),
            (6.0, 8.0),
            (6.0, 7.0607),
            (2.9393, 4.0),
            (0.6536, 4.0),
            (0.4036, 3.933),
            (0.0, 3.7),
            (-0.4036, 3.933),
            (-0.9036, 3.933),
            (-1.2189, 3.751),
            (-2.5, 4.0),
        ] {
            assert!(has(&outer, x, y), "({x}, {y})");
        }
        // Each side has sixteen lines, the two roundings of its floor and a corner.
        assert_eq!(outer.len(), 4 * 19);
    }

    /// The net area of each OpenBuilds DXF section: the outline less its cavities and bores.
    #[test]
    fn the_v_slot_areas_are_the_openbuilds_dxf_areas() {
        for (size, area) in [
            ("2020", 171.403),
            ("2040", 299.989),
            ("2060", 428.576),
            ("2080", 557.163),
        ] {
            let got = profile(V5, size).area();
            assert!(close(got, area, 0.01), "{size}: {got}");
        }
    }

    /// Vertices of `V-Slot 20x20.dxf`, in the top right quarter.
    #[test]
    fn the_v_slot_outline_has_the_dxf_vertices() {
        let p = profile(V5, "2020");
        let points: Vec<P> = p.outer.iter().map(|s| s.start()).collect();
        for (x, y) in [
            (0.0, 3.69),
            (0.21, 3.9),
            (2.8393, 3.9),
            (5.5, 6.5607),
            (5.5, 8.2),
            (3.125, 8.2),
            (3.125, 8.545),
            (4.58, 10.0),
            (9.5, 10.0),
            (10.0, 9.5),
        ] {
            assert!(has(&points, x, y), "({x}, {y})");
        }
        assert_eq!(points.len(), 68);
    }

    /// The cavity of `V-Slot 20x40.dxf`.
    #[test]
    fn the_v_slot_2040_cavity_has_the_dxf_vertices() {
        let p = profile(V5, "2040");
        assert_eq!(p.cavities.len(), 1);
        let cavity: Vec<P> = p.cavities[0].iter().map(|s| s.start()).collect();
        assert_eq!(cavity.len(), 12);
        for (x, y) in [(2.8393, 6.1), (6.2393, 2.7), (8.2, 2.7), (8.2, -2.7)] {
            assert!(has(&cavity, x, y) && has(&cavity, -x, -y), "({x}, {y})");
        }
        assert!(close(loop_area(&p.cavities[0]), 150.295, 0.01));
    }

    #[test]
    fn a_row_has_a_cavity_between_each_pair_of_cells() {
        assert_eq!(profile(T5, "2020").cavities.len(), 0);
        assert_eq!(profile(T5, "2060").cavities.len(), 2);
        assert_eq!(profile(V5, "2080").cavities.len(), 3);
        assert_eq!(profile(T5, "2080").bores.len(), 4);
    }

    /// The closed cavities of MISUMI's STEP model `NFS5-2060-2060.stp`, 143.029 mm² each,
    /// and its section of 463.49 mm².
    #[test]
    fn the_t_slot_2060_has_the_misumi_step_cavities() {
        let p = profile(T5, "2060");
        assert_eq!(p.cavities.len(), 2);
        let upper: Vec<P> = p.cavities[1].iter().map(|s| s.start()).collect();
        assert_eq!(upper.len(), 12);
        for (x, y) in [
            (2.9393, 4.0),
            (6.9393, 8.0),
            (8.0, 8.0),
            (8.0, 12.0),
            (6.9393, 12.0),
            (2.9393, 16.0),
        ] {
            assert!(has(&upper, x, y) && has(&upper, -x, y), "({x}, {y})");
        }
        for cavity in &p.cavities {
            assert!(close(loop_area(cavity), 143.029, 0.001));
        }
    }

    /// Vertices of the end face of MISUMI's STEP model `NFS5-4040-50.stp`, in the top
    /// right quarter, and its section of 479.13 mm².
    #[test]
    fn the_t_slot_4040_has_the_misumi_step_vertices() {
        let p = profile(T5, "4040");
        let cavity: Vec<P> = p.cavities[0].iter().map(|s| s.start()).collect();
        for (x, y) in [
            (18.0, 2.0),
            (16.9393, 2.0),
            (12.9393, 6.0),
            (8.6667, 6.0),
            (9.3359, 8.0078),
            (8.0078, 9.3359),
            (6.0, 8.6667),
            (6.0, 12.9393),
            (2.0, 16.9393),
            (2.0, 18.0),
        ] {
            assert!(has(&cavity, x, y), "({x}, {y})");
        }
        let outer: Vec<P> = p.outer.iter().map(|s| s.start()).collect();
        for (x, y) in [
            (20.0, 7.0),
            (18.0, 4.0),
            (14.0, 7.0607),
            (13.7, 10.0),
            (17.0607, 16.0),
            (19.0, 20.0),
            (13.0, 18.0),
            (16.0, 17.0607),
            (10.0, 13.7),
            (4.0, 17.0607),
            (7.0, 18.0),
            (7.0, 20.0),
        ] {
            assert!(has(&outer, x, y), "({x}, {y})");
        }
        assert!(close(loop_area(&p.cavities[0]), 711.964, 0.002));
    }

    /// NFS5-4040: the wall between two slots is 2 mm thick, and each bore is open to
    /// the middle.
    #[test]
    fn a_4040_has_one_middle_cavity_that_takes_in_the_four_bores() {
        let p = profile(T5, "4040");
        assert_eq!(p.cavities.len(), 1);
        assert!(p.bores.is_empty());
        let cavity = &p.cavities[0];
        for (i, seg) in cavity.iter().enumerate() {
            let next = cavity[(i + 1) % cavity.len()].start();
            let end = seg.end();
            assert!(
                close(end[0], next[0], 1e-9) && close(end[1], next[1], 1e-9),
                "{i}"
            );
        }
        let arcs = cavity
            .iter()
            .filter(|s| matches!(s, Seg::Arc { .. }))
            .count();
        assert_eq!(arcs, 8);
        let points: Vec<P> = cavity.iter().map(|s| s.start()).collect();
        // The outer faces of each core start 6 mm from the rail's faces.
        assert!(has(&points, 6.0, 10.0 + T5.web_root()));
        assert!(has(&points, 10.0 + T5.web_root(), 6.0));
        // The wall under the top face, between the legs of the two slots.
        assert!(has(&points, 2.0, 18.0) && has(&points, -2.0, 18.0));
        // Every bore arc is 2.1 mm from its cell's centre.
        for seg in cavity {
            if let Seg::Arc {
                through, centre, ..
            } = seg
            {
                assert_eq!(centre[0].abs(), 10.0);
                assert_eq!(centre[1].abs(), 10.0);
                let d = (through[0] - centre[0]).hypot(through[1] - centre[1]);
                assert!(close(d, 2.1, 1e-9));
            }
        }
    }

    #[test]
    fn a_circle_drawn_as_two_arcs_has_its_area() {
        for ccw in [true, false] {
            let (a, b) = ([2.0, 0.0], [-2.0, 0.0]);
            let (up, down) = ([0.0, 2.0], [0.0, -2.0]);
            let arc = |start, end, through| Seg::Arc {
                start,
                end,
                through,
                centre: [0.0, 0.0],
                ccw,
            };
            let segs = if ccw {
                [arc(a, b, up), arc(b, a, down)]
            } else {
                [arc(a, b, down), arc(b, a, up)]
            };
            assert!(close(loop_area(&segs), std::f64::consts::PI * 4.0, 1e-9));
        }
    }

    #[test]
    fn the_material_point_is_in_the_core_and_out_of_the_bore() {
        for slot in [T5, V5] {
            for (_, grid) in SIZES {
                let p = Profile::new(slot, grid).unwrap();
                let c = grid.centres()[0];
                let m = p.material_point();
                let (dx, dy) = (m[0] - c[0], m[1] - c[1]);
                // The first cell is the bottom left one, so its corner side is -X, -Y.
                assert!(dx < 0.0 && dy < 0.0);
                assert!(dx.hypot(dy) > slot.bore / 2.0 + 0.2);
                assert!(-dx < slot.web_root() && -dy < slot.web_root());
            }
        }
    }

    #[test]
    fn an_unlisted_grid_is_not_built() {
        assert!(Grid::of("3030").is_none());
        assert!(Profile::new(T5, Grid { nx: 3, ny: 2 }).is_none());
    }
}
