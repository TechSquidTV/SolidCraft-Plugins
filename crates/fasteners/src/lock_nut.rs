//! Thin hexagon nuts with a polyamide insert, DIN 985.
//! Bossard BN 637, columns d1, s, h max and m:
//! <https://www.tme.eu/Document/89f67e5e26d56fab067c14b4e5fd7c28/BN637.pdf>.
//! The supplier specifies the envelope and wrenching height. The rolled lip and
//! captured insert below are illustrative sections for an installed nut.

use crate::metric::same_mm;
use plugin_kit::name::part;
use plugin_kit::revolve::{self, Section, Segment};
use plugin_kit::{Placement, Runner};
use serde_json::json;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LockNut {
    pub nominal: f64,
    pub across_flats: f64,
    pub height: f64,
    pub hex_height: f64,
}

// Millimetres, in nominal diameter order.
const ROWS: &[(f64, f64, f64, f64)] = &[
    (2.5, 5., 3.8, 2.),
    (3., 5.5, 4., 2.4),
    (4., 7., 5., 2.9),
    (5., 8., 5., 3.2),
    (6., 10., 6., 4.),
    (8., 13., 8., 5.5),
    (10., 17., 10., 6.5),
    (12., 19., 12., 8.),
    (14., 22., 14., 9.5),
    (16., 24., 16., 10.5),
    (18., 27., 18.5, 13.),
    (20., 30., 20., 14.),
    (22., 32., 22., 15.),
    (24., 36., 24., 15.),
    (27., 41., 27., 17.),
    (30., 46., 30., 19.),
];

pub fn sizes() -> impl Iterator<Item = LockNut> {
    ROWS.iter()
        .map(|&(nominal, across_flats, height, hex_height)| LockNut {
            nominal,
            across_flats,
            height,
            hex_height,
        })
}

impl LockNut {
    pub fn for_major(major: f64) -> Option<Self> {
        sizes().find(|row| same_mm(row.nominal, major))
    }

    /// DIN 985:1987 starts at M3. Bossard BN 637 lists M2.5 separately
    /// as article 1143735 with its own sourced dimensions.
    pub fn supplier_article(self) -> Option<&'static str> {
        (self.nominal == 2.5).then_some("1143735")
    }

    /// Scale the wrenching height proportionally when the overall height is edited.
    pub fn with_envelope(mut self, width: Option<f64>, height: Option<f64>) -> Self {
        if let Some(height) = height {
            self.hex_height *= height / self.height;
            self.height = height;
        }
        self.across_flats = width.unwrap_or(self.across_flats);
        self
    }

    /// Join a rolled metal rim to the threaded hex, then add the captive insert
    /// as a second body of that same component. Its smooth, nominal-diameter
    /// bore depicts the installed state without overlapping the mating screw.
    pub fn add_collar(
        self,
        r: &mut impl Runner,
        site: &Placement,
        body: &str,
        base: &str,
    ) -> Result<(), String> {
        let bore = self.nominal / 2.0;
        let outside = self.across_flats / 2.0;
        let wall = outside - bore;
        let pocket = bore + 0.68 * wall;
        let lip = bore + 0.38 * wall;
        let bottom = self.hex_height;
        let top = self.height;
        let rise = top - bottom;
        let seat = top - 0.24 * rise;
        let round = (wall * 0.46).min(rise * 0.7);
        let start = [outside, top - round];
        let end = [outside - round, top];
        let mid = round * std::f64::consts::FRAC_1_SQRT_2;
        let collar = [
            Segment::Arc {
                start,
                through: [end[0] + mid, start[1] + mid],
                end,
            },
            Segment::Line(vec![
                end,
                [lip, top],
                [lip, seat],
                [pocket, seat],
                [pocket, bottom],
                [outside, bottom],
                start,
            ]),
        ];
        let feature = part(base, "Rolled collar");
        revolve::build(
            r,
            site,
            &part(&feature, "Section"),
            Section::CurvedSolid(&collar),
            json!({"operation":"join", "targets":[body], "name":feature}),
        )?;
        let ring = [
            [bore, bottom],
            [pocket, bottom],
            [pocket, seat],
            [bore, seat],
        ];
        let name = format!("{base} Nylon insert");
        let feature = part(base, "Nylon insert");
        revolve::build(
            r,
            site,
            &part(&feature, "Section"),
            Section::Solid(&ring),
            json!({"operation":"new", "name":feature, "body_name":name}),
        )?;
        r.run(
            "appearance.assign",
            json!({"bodies":[name], "color":[53,92,160]}),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_match_bossard_bn637() {
        // Independent transcription in the supplier's column order: d1, h max, m, s.
        let reference = [
            (2.5, 3.8, 2., 5.),
            (3., 4., 2.4, 5.5),
            (4., 5., 2.9, 7.),
            (5., 5., 3.2, 8.),
            (6., 6., 4., 10.),
            (8., 8., 5.5, 13.),
            (10., 10., 6.5, 17.),
            (12., 12., 8., 19.),
            (14., 14., 9.5, 22.),
            (16., 16., 10.5, 24.),
            (18., 18.5, 13., 27.),
            (20., 20., 14., 30.),
            (22., 22., 15., 32.),
            (24., 24., 15., 36.),
            (27., 27., 17., 41.),
            (30., 30., 19., 46.),
        ];
        assert_eq!(sizes().count(), reference.len());
        for (d, h, m, s) in reference {
            let row = LockNut::for_major(d).unwrap();
            assert_eq!((row.height, row.hex_height, row.across_flats), (h, m, s));
        }
    }
}
