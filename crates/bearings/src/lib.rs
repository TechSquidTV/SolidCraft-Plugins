//! Supplier products, not arbitrary bore/diameter/width combinations.
//!
//! Sources and measured CAD revisions: docs/sources/{openbuilds,misumi}.md. External rings
//! are solid envelopes; closures are annular faces at the measured recess. Hidden
//! seal thickness and cages are deliberately not inferred. Open B626 includes
//! its exposed, sourced raceways and balls; closed bearings omit hidden balls.

use plugin_kit::{
    revolve::{self, Section},
    Placement, Runner,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod profile;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Bearing {
    #[serde(rename = "openbuilds-625-2rs")]
    OpenBuilds6252rs,
    #[serde(rename = "openbuilds-105zz")]
    OpenBuilds105zz,
    #[serde(rename = "misumi-b626")]
    MisumiB626,
    #[serde(rename = "misumi-b626zz")]
    MisumiB626zz,
    #[serde(rename = "misumi-b626ddu")]
    MisumiB626ddu,
    #[serde(rename = "misumi-b608zz")]
    MisumiB608zz,
}

pub const ALL: [Bearing; 6] = [
    Bearing::OpenBuilds6252rs,
    Bearing::OpenBuilds105zz,
    Bearing::MisumiB626,
    Bearing::MisumiB626zz,
    Bearing::MisumiB626ddu,
    Bearing::MisumiB608zz,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Closure {
    Open,
    RubberSeals,
    MetalShields,
}

/// Stable catalog namespace kept separate from the supplier's display name.
pub struct Supplier {
    pub catalog: &'static str,
    pub name: &'static str,
}

/// Exterior section measurements in millimetres, not a manufacturing drawing.
pub struct Dimensions {
    pub bore: f64,
    pub diameter: f64,
    pub width: f64,
    pub inner_ring_radius: f64,
    pub outer_ring_radius: f64,
    pub closure_recess: Option<f64>,
    pub edge_radius: f64,
}

/// Visible geometry from the open B626 STEP, shared by race and ball construction.
struct OpenRace {
    pitch_radius: f64,
    groove_radius: f64,
    ball_radius: f64,
    ball_count: u32,
}

impl Bearing {
    pub fn representation(self) -> parts::Representation<Self> {
        parts::Representation {
            product: parts::Product {
                catalog: self.supplier().catalog.into(),
                part: self.sku().into(),
                kind: parts::ProductKind::SupplierProduct,
                configuration: self,
                members: vec![],
            },
            revision: if self == Self::MisumiB626 { "2" } else { "1" }.into(),
            detail: "exterior".into(),
            dependencies: vec![],
        }
    }

    /// Shared by direct insertion and wheel recipes; the caller is only provenance.
    pub fn ensure<R: Runner>(
        self,
        library: &mut plugin_kit::component::Library<'_, R>,
        creator: &str,
    ) -> Result<plugin_kit::component::Handle, String> {
        let name = format!("Bearing {}", self.designation());
        let number = format!("{} {}", self.supplier().name, self.sku());
        library.ensure(
            &plugin_kit::component::Definition {
                plugin: creator,
                base: &name,
                description: self.designation(),
                bom_type: "purchased",
                site: plugin_kit::component::Site::Origin,
            },
            &number,
            &self.representation(),
            |library, name, frame| library.geometry(name, |mut r| self.build(&mut r, name, frame)),
        )
    }

    pub fn designation(self) -> &'static str {
        match self {
            Self::OpenBuilds6252rs => "625 2RS",
            Self::OpenBuilds105zz => "105ZZ",
            Self::MisumiB626 | Self::MisumiB626zz | Self::MisumiB626ddu | Self::MisumiB608zz => {
                self.sku()
            }
        }
    }

    pub fn supplier(self) -> Supplier {
        match self {
            Self::OpenBuilds6252rs | Self::OpenBuilds105zz => Supplier {
                catalog: "openbuilds",
                name: "OpenBuilds",
            },
            Self::MisumiB626 | Self::MisumiB626zz | Self::MisumiB626ddu | Self::MisumiB608zz => {
                Supplier {
                    catalog: "misumi",
                    name: "MISUMI",
                }
            }
        }
    }

    pub fn product_url(self) -> &'static str {
        match self {
            Self::OpenBuilds6252rs | Self::OpenBuilds105zz => {
                "https://builds.openbuilds.com/builds/openbuilds-sphinx-1050-20-x-40.7196/"
            }
            Self::MisumiB626 => {
                "https://us.misumi-ec.com/vona2/detail/110302273540/?HissuCode=B626"
            }
            Self::MisumiB626zz => {
                "https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B626ZZ"
            }
            Self::MisumiB626ddu => {
                "https://us.misumi-ec.com/vona2/detail/110302273630/?HissuCode=B626DDU"
            }
            Self::MisumiB608zz => {
                "https://us.misumi-ec.com/vona2/detail/110300107560/?HissuCode=B608ZZ"
            }
        }
    }

    /// Exact supplier part number; the underlying bearing manufacturer may differ.
    pub fn sku(self) -> &'static str {
        match self {
            Self::OpenBuilds6252rs => "30",
            Self::OpenBuilds105zz => "800",
            Self::MisumiB626 => "B626",
            Self::MisumiB626zz => "B626ZZ",
            Self::MisumiB626ddu => "B626DDU",
            Self::MisumiB608zz => "B608ZZ",
        }
    }

    pub fn closure(self) -> Closure {
        match self {
            Self::OpenBuilds6252rs => Closure::RubberSeals,
            Self::OpenBuilds105zz => Closure::MetalShields,
            Self::MisumiB626 => Closure::Open,
            Self::MisumiB626zz | Self::MisumiB608zz => Closure::MetalShields,
            Self::MisumiB626ddu => Closure::RubberSeals,
        }
    }

    fn open_race(self) -> Option<OpenRace> {
        (self == Self::MisumiB626).then_some(OpenRace {
            pitch_radius: 6.25,
            groove_radius: 1.95,
            ball_radius: 1.975,
            ball_count: 8,
        })
    }

    pub fn dimensions(self) -> Dimensions {
        // OpenBuilds STEP exterior sections, with the documented 105ZZ width
        // correction (2016-02-27). The 4.2 mm reference is not a separate product.
        match self {
            Self::OpenBuilds6252rs => Dimensions {
                bore: 5.0,
                diameter: 16.0,
                width: 5.0,
                inner_ring_radius: 4.3,
                outer_ring_radius: 6.8,
                closure_recess: Some(0.3),
                edge_radius: 0.0,
            },
            Self::OpenBuilds105zz => Dimensions {
                bore: 5.0,
                diameter: 10.0,
                width: 4.0,
                inner_ring_radius: 3.1,
                outer_ring_radius: 4.6,
                closure_recess: Some(0.3),
                edge_radius: 0.0,
            },
            // Exact supplied MISUMI STEP revisions; 626 variants have different
            // radial boundaries and recesses despite sharing the 6 × 19 × 6 envelope.
            Self::MisumiB626 => Dimensions {
                bore: 6.0,
                diameter: 19.0,
                width: 6.0,
                inner_ring_radius: 5.06,
                outer_ring_radius: 7.55,
                closure_recess: None,
                edge_radius: 0.3,
            },
            Self::MisumiB626zz => Dimensions {
                bore: 6.0,
                diameter: 19.0,
                width: 6.0,
                inner_ring_radius: 4.75,
                outer_ring_radius: 8.15,
                closure_recess: Some(0.5),
                edge_radius: 0.3,
            },
            Self::MisumiB626ddu => Dimensions {
                bore: 6.0,
                diameter: 19.0,
                width: 6.0,
                inner_ring_radius: 4.215,
                outer_ring_radius: 8.1775,
                closure_recess: Some(0.11),
                edge_radius: 0.3,
            },
            Self::MisumiB608zz => Dimensions {
                bore: 8.0,
                diameter: 22.0,
                width: 7.0,
                inner_ring_radius: 6.1,
                outer_ring_radius: 9.65,
                closure_recess: Some(0.6),
                edge_radius: 0.3,
            },
        }
    }

    /// The axle is +Z and the origin is the bearing's centre plane. A bearing is
    /// one purchased component; its bodies are not separately purchased parts.
    pub fn build(self, r: &mut impl Runner, base: &str, frame: &Placement) -> Result<(), String> {
        let d = self.dimensions();
        let half = d.width / 2.0;
        let mut steel = Vec::new();
        for (label, inner, outer, outer_ring) in [
            ("Inner ring", d.bore / 2.0, d.inner_ring_radius, false),
            ("Outer ring", d.outer_ring_radius, d.diameter / 2.0, true),
        ] {
            let name = format!("{base} {label}");
            let polygon = [[inner, -half], [outer, -half], [outer, half], [inner, half]];
            let curved = profile::ring(self, outer_ring);
            revolve::build(
                r,
                frame,
                &format!("{name} section"),
                if d.edge_radius == 0.0 {
                    Section::Solid(&polygon)
                } else {
                    Section::CurvedSolid(&curved)
                },
                json!({"name": name, "body_name": name}),
            )?;
            steel.push(name);
        }
        if let Some(race) = self.open_race() {
            for index in 0..race.ball_count {
                let angle = std::f64::consts::TAU * f64::from(index) / f64::from(race.ball_count);
                let center: [f64; 3] = std::array::from_fn(|i| {
                    frame.point[i]
                        + race.pitch_radius
                            * (angle.cos() * frame.x_dir[i] + angle.sin() * frame.y_dir[i])
                });
                let name = format!("{base} Ball {}", index + 1);
                r.run(
                    "solid.sphere",
                    json!({"radius":race.ball_radius, "center":center,
                    "name":name, "body_name":name}),
                )?;
                steel.push(name);
            }
        }
        r.run(
            "appearance.assign",
            json!({"bodies": steel, "color": [160,160,160]}),
        )?;
        let mut closures = Vec::new();
        let (label, color) = match self.closure() {
            Closure::Open => return Ok(()),
            Closure::RubberSeals => ("Seal", [64, 64, 64]),
            Closure::MetalShields => ("Shield", [160, 160, 160]),
        };
        let recess = d
            .closure_recess
            .ok_or("closed bearing has no sourced recess")?;
        for (side, sign) in [("front", 1.0), ("back", -1.0)] {
            let name = format!("{base} {label} {side}");
            let z = sign * (half - recess);
            revolve::build(
                r,
                frame,
                &format!("{name} section"),
                Section::Surface(&[[d.inner_ring_radius, z], [d.outer_ring_radius, z]]),
                json!({"name": name, "body_name": name}),
            )?;
            closures.push(name);
        }
        r.run(
            "appearance.assign",
            json!({"bodies": closures, "color": color}),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exterior_dimensions_match_source_and_mini_correction() {
        // Independently transcribed from the supplier STEP and correction ledger.
        for (bearing, expected) in [
            (Bearing::OpenBuilds6252rs, [5.0, 16.0, 5.0, 4.3, 6.8, 0.3]),
            (Bearing::OpenBuilds105zz, [5.0, 10.0, 4.0, 3.1, 4.6, 0.3]),
        ] {
            let d = bearing.dimensions();
            assert_eq!(
                [
                    d.bore,
                    d.diameter,
                    d.width,
                    d.inner_ring_radius,
                    d.outer_ring_radius,
                    d.closure_recess.unwrap()
                ],
                expected
            );
        }
    }

    #[test]
    fn supplier_variants_do_not_conflate_closure_or_identity() {
        assert_eq!(Bearing::OpenBuilds6252rs.sku(), "30");
        assert_eq!(Bearing::OpenBuilds6252rs.closure(), Closure::RubberSeals);
        assert_eq!(Bearing::OpenBuilds105zz.sku(), "800");
        assert_eq!(Bearing::OpenBuilds105zz.closure(), Closure::MetalShields);
        assert!(serde_json::from_str::<Bearing>("\"openbuilds-5x10x4.2\"").is_err());
    }

    #[test]
    fn misumi_sections_match_supplied_source_revisions() {
        // Independently transcribed from the source surfaces and vertices.
        for (bearing, dims, recess, closure) in [
            (
                Bearing::MisumiB626,
                [6.0, 19.0, 6.0, 5.06, 7.55],
                None,
                Closure::Open,
            ),
            (
                Bearing::MisumiB626zz,
                [6.0, 19.0, 6.0, 4.75, 8.15],
                Some(0.5),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB626ddu,
                [6.0, 19.0, 6.0, 4.215, 8.1775],
                Some(0.11),
                Closure::RubberSeals,
            ),
            (
                Bearing::MisumiB608zz,
                [8.0, 22.0, 7.0, 6.1, 9.65],
                Some(0.6),
                Closure::MetalShields,
            ),
        ] {
            let d = bearing.dimensions();
            assert_eq!(
                [
                    d.bore,
                    d.diameter,
                    d.width,
                    d.inner_ring_radius,
                    d.outer_ring_radius
                ],
                dims
            );
            assert_eq!(d.closure_recess, recess);
            assert_eq!(d.edge_radius, 0.3);
            assert_eq!(bearing.closure(), closure);
            assert_eq!(bearing.representation().product.catalog, "misumi");
        }
        let products: std::collections::BTreeSet<_> = ALL
            .into_iter()
            .map(|b| {
                b.representation()
                    .product
                    .identity()
                    .unwrap()
                    .as_str()
                    .to_owned()
            })
            .collect();
        assert_eq!(products.len(), ALL.len());
        let race = Bearing::MisumiB626.open_race().unwrap();
        assert_eq!(
            (
                race.pitch_radius,
                race.groove_radius,
                race.ball_radius,
                race.ball_count
            ),
            (6.25, 1.95, 1.975, 8)
        );
        assert!(ALL
            .into_iter()
            .filter(|b| *b != Bearing::MisumiB626)
            .all(|b| b.open_race().is_none()));
    }
}
