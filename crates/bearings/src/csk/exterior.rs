//! Exterior sections measured from supplied STEP files, with explicit estimation.
//! Hidden raceways and internals are omitted. See docs/sources/csk.md.
use super::{Csk, Geometry};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Chamfer {
    pub radial: f64,
    pub axial: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Section {
    pub inner_ring_radius: f64,
    pub outer_ring_radius: f64,
    pub inner_chamfer: Chamfer,
    pub outer_chamfer: Chamfer,
    pub closure_recess: f64,
}

#[derive(Deserialize)]
struct Reference {
    file: String,
    sha256: String,
    designation: Csk,
    bore: f64,
    diameter: f64,
    width: f64,
    section: Section,
}

#[derive(Serialize)]
pub struct Evidence<'a> {
    pub file: &'a str,
    pub sha256: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    Measured,
    Estimated,
}

#[derive(Serialize)]
pub struct Exterior {
    pub section: Section,
    pub basis: Basis,
    pub method: &'static str,
    pub sources: Vec<Evidence<'static>>,
    pub adapted_for_keyways: bool,
}

fn references() -> &'static [Reference] {
    static REFERENCES: OnceLock<Vec<Reference>> = OnceLock::new();
    REFERENCES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../docs/sources/csk-files.json"))
            .expect("checked CSK source ledger")
    })
}

/// Interpolate normalized radial sections between measured bore sizes. Outside
/// the measured range, preserve the nearest reference's radial proportions.
/// Axial chamfer lengths are interpolated in mm and limited for thin custom parts.
/// Deep custom keyways can widen the rings; no slot may break into a closure.
pub fn resolve(g: &Geometry) -> Result<Exterior, String> {
    let refs = references();
    let exact = refs.iter().find(|r| {
        Some(r.designation) == g.designation
            && [g.bore, g.diameter, g.width] == [r.bore, r.diameter, r.width]
    });
    let low = refs
        .iter()
        .rev()
        .find(|r| r.bore <= g.bore)
        .unwrap_or(&refs[0]);
    let high = refs
        .iter()
        .find(|r| r.bore >= g.bore)
        .unwrap_or(&refs[refs.len() - 1]);
    let (low, high) = exact.map_or((low, high), |r| (r, r));
    let t = if high.bore == low.bore {
        0.0
    } else {
        (g.bore - low.bore) / (high.bore - low.bore)
    };
    let mix = |a: f64, b: f64| a + (b - a) * t;
    let span = (g.diameter - g.bore) / 2.0;
    let span_a = (low.diameter - low.bore) / 2.0;
    let span_b = (high.diameter - high.bore) / 2.0;
    let a = &low.section;
    let b = &high.section;
    let radial = |x: f64, y: f64| span * mix(x / span_a, y / span_b);
    let mut section = if exact.is_some() {
        a.clone()
    } else {
        Section {
            inner_ring_radius: g.bore / 2.0
                + radial(
                    a.inner_ring_radius - low.bore / 2.0,
                    b.inner_ring_radius - high.bore / 2.0,
                ),
            outer_ring_radius: g.bore / 2.0
                + radial(
                    a.outer_ring_radius - low.bore / 2.0,
                    b.outer_ring_radius - high.bore / 2.0,
                ),
            inner_chamfer: Chamfer {
                radial: radial(a.inner_chamfer.radial, b.inner_chamfer.radial),
                axial: mix(a.inner_chamfer.axial, b.inner_chamfer.axial),
            },
            outer_chamfer: Chamfer {
                radial: radial(a.outer_chamfer.radial, b.outer_chamfer.radial),
                axial: mix(a.outer_chamfer.axial, b.outer_chamfer.axial),
            },
            closure_recess: mix(a.closure_recess, b.closure_recess),
        }
    };
    let original = section.clone();
    let web = span * 0.02;
    if let Some(key) = g.inner_keyway {
        section.inner_ring_radius = section
            .inner_ring_radius
            .max((g.bore / 2.0 + key.depth).hypot(key.width / 2.0) + web);
    }
    if let Some(key) = g.outer_keyway {
        section.outer_ring_radius = section
            .outer_ring_radius
            .min(g.diameter / 2.0 - key.depth - web);
    }
    let adapted_for_keyways = section != original;
    if section.inner_ring_radius + web >= section.outer_ring_radius {
        return Err(
            "keyways must leave space for separate rings and a closure between them".into(),
        );
    }
    section.inner_chamfer.radial = section
        .inner_chamfer
        .radial
        .min((section.inner_ring_radius - g.bore / 2.0) * 0.45);
    section.outer_chamfer.radial = section
        .outer_chamfer
        .radial
        .min((g.diameter / 2.0 - section.outer_ring_radius) * 0.45);
    section.inner_chamfer.axial = section.inner_chamfer.axial.min(g.width * 0.25);
    section.outer_chamfer.axial = section.outer_chamfer.axial.min(g.width * 0.25);
    let adapted = section != original;
    let evidence = |r: &'static Reference| Evidence {
        file: &r.file,
        sha256: &r.sha256,
    };
    Ok(Exterior {
        section,
        basis: if exact.is_some() && !adapted {
            Basis::Measured
        } else {
            Basis::Estimated
        },
        method: if exact.is_some() && !adapted {
            "Exterior section measured from the supplied STEP reference"
        } else {
            "Normalized radial interpolation by bore; nearest proportions outside 8–40 mm; chamfers limited for thin parts; rings widened for keyways"
        },
        sources: if low.bore == high.bore {
            vec![evidence(low)]
        } else {
            vec![evidence(low), evidence(high)]
        },
        adapted_for_keyways,
    })
}

/// Six-vertex ring profile with separately measured axial and radial chamfers.
pub fn ring(g: &Geometry, section: &Section, outer: bool) -> [[f64; 2]; 6] {
    let h = g.width / 2.0;
    let (edge, shoulder, sign, chamfer) = if outer {
        (
            g.diameter / 2.0,
            section.outer_ring_radius,
            -1.0,
            section.outer_chamfer,
        )
    } else {
        (
            g.bore / 2.0,
            section.inner_ring_radius,
            1.0,
            section.inner_chamfer,
        )
    };
    [
        [shoulder, -h],
        [edge + sign * chamfer.radial, -h],
        [edge, -h + chamfer.axial],
        [edge, h - chamfer.axial],
        [edge + sign * chamfer.radial, h],
        [shoulder, h],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ledger_is_ordered_and_matches_the_catalog() {
        let refs = references();
        assert!(refs.windows(2).all(|r| r[0].bore <= r[1].bore));
        for r in refs {
            let g = r.designation.geometry();
            assert_eq!([g.bore, g.diameter, g.width], [r.bore, r.diameter, r.width]);
            let exterior = resolve(&g).unwrap();
            assert!(matches!(exterior.basis, Basis::Measured));
            assert_eq!(exterior.section, r.section);
            assert_eq!(r.sha256.len(), 64);
        }
    }
    #[test]
    fn all_presets_have_separate_rings_and_sensible_chamfers() {
        for p in super::super::ALL {
            let g = p.geometry();
            let s = resolve(&g).unwrap().section;
            assert!(g.bore / 2.0 + s.inner_chamfer.radial < s.inner_ring_radius);
            assert!(s.inner_ring_radius < s.outer_ring_radius);
            assert!(s.outer_ring_radius < g.diameter / 2.0 - s.outer_chamfer.radial);
            assert!(s.inner_chamfer.axial * 2.0 < g.width);
            assert!(s.outer_chamfer.axial * 2.0 < g.width);
        }
    }
    #[test]
    fn inference_and_keyway_adaptation_are_explicit() {
        let mut g = Csk::Csk20P.geometry();
        assert!(matches!(resolve(&g).unwrap().basis, Basis::Estimated));
        g.inner_keyway.as_mut().unwrap().depth = 7.0;
        assert!(resolve(&g).unwrap().adapted_for_keyways);
        g.width = 0.5;
        assert_eq!(resolve(&g).unwrap().section.inner_chamfer.axial, 0.125);
    }
}
