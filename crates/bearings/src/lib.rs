//! Supplier products, not arbitrary bore/diameter/width combinations.
//!
//! Sources and measured CAD revisions: docs/sources/openbuilds.md. External rings
//! are solid envelopes; closures are annular faces at the measured recess. Hidden
//! raceways, seal thickness, balls and cages are deliberately not inferred.

use plugin_kit::{
    revolve::{self, Section},
    Placement, Runner,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Bearing {
    #[serde(rename = "openbuilds-625-2rs")]
    OpenBuilds6252rs,
    #[serde(rename = "openbuilds-105zz")]
    OpenBuilds105zz,
}

pub const ALL: [Bearing; 2] = [Bearing::OpenBuilds6252rs, Bearing::OpenBuilds105zz];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Closure {
    RubberSeals,
    MetalShields,
}

/// Exterior section measurements in millimetres, not a manufacturing drawing.
pub struct Dimensions {
    pub bore: f64,
    pub diameter: f64,
    pub width: f64,
    pub inner_ring_radius: f64,
    pub outer_ring_radius: f64,
    pub closure_recess: f64,
}

impl Bearing {
    pub fn representation(self) -> parts::Representation<Self> {
        parts::Representation {
            product: parts::Product {
                catalog: "openbuilds".into(),
                part: self.sku().into(),
                kind: parts::ProductKind::SupplierProduct,
                configuration: self,
                members: vec![],
            },
            revision: "1".into(),
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
        let number = format!("OpenBuilds {}", self.sku());
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
        }
    }

    /// OpenBuilds supplier SKU. The original bearing manufacturer is not identified.
    pub fn sku(self) -> &'static str {
        match self {
            Self::OpenBuilds6252rs => "30",
            Self::OpenBuilds105zz => "800",
        }
    }

    pub fn closure(self) -> Closure {
        match self {
            Self::OpenBuilds6252rs => Closure::RubberSeals,
            Self::OpenBuilds105zz => Closure::MetalShields,
        }
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
                closure_recess: 0.3,
            },
            Self::OpenBuilds105zz => Dimensions {
                bore: 5.0,
                diameter: 10.0,
                width: 4.0,
                inner_ring_radius: 3.1,
                outer_ring_radius: 4.6,
                closure_recess: 0.3,
            },
        }
    }

    /// The axle is +Z and the origin is the bearing's centre plane. A bearing is
    /// one purchased component; its four bodies are not separately purchased parts.
    pub fn build(self, r: &mut impl Runner, base: &str, frame: &Placement) -> Result<(), String> {
        let d = self.dimensions();
        let half = d.width / 2.0;
        let mut rings = Vec::new();
        for (label, inner, outer) in [
            ("Inner ring", d.bore / 2.0, d.inner_ring_radius),
            ("Outer ring", d.outer_ring_radius, d.diameter / 2.0),
        ] {
            let name = format!("{base} {label}");
            revolve::build(
                r,
                frame,
                &format!("{name} section"),
                Section::Solid(&[[inner, -half], [outer, -half], [outer, half], [inner, half]]),
                json!({"name": name, "body_name": name}),
            )?;
            rings.push(name);
        }
        r.run(
            "appearance.assign",
            json!({"bodies": rings, "color": [160,160,160]}),
        )?;
        let mut closures = Vec::new();
        let label = match self.closure() {
            Closure::RubberSeals => "Seal",
            Closure::MetalShields => "Shield",
        };
        for (side, sign) in [("front", 1.0), ("back", -1.0)] {
            let name = format!("{base} {label} {side}");
            let z = sign * (half - d.closure_recess);
            revolve::build(
                r,
                frame,
                &format!("{name} section"),
                Section::Surface(&[[d.inner_ring_radius, z], [d.outer_ring_radius, z]]),
                json!({"name": name, "body_name": name}),
            )?;
            closures.push(name);
        }
        let color = match self.closure() {
            Closure::RubberSeals => [64, 64, 64],
            Closure::MetalShields => [160, 160, 160],
        };
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
                    d.closure_recess
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
}
