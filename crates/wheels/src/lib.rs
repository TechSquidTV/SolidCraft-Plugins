//! Fixed OpenBuilds wheel profiles measured from supplied STEP references.
//! Dimensions, source hashes, corrections and limitations: docs/sources/openbuilds.md.

use bearings::Bearing;
use parts::catalog::{Frame, Member};
use plugin_kit::{
    revolve::{self, Section},
    Placement, Runner,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Wheel {
    Solid,
    XtremeSolid,
    Delrin,
    DelrinMini,
    XtremeMini,
}

pub const ALL: [Wheel; 5] = [
    Wheel::Solid,
    Wheel::XtremeSolid,
    Wheel::Delrin,
    Wheel::DelrinMini,
    Wheel::XtremeMini,
];

enum CorePart {
    Wheel(Wheel),
    Bearing(Bearing),
    Shim(Shim),
}

struct CoreMember {
    part: CorePart,
    name: &'static str,
    offset: f64,
}

impl CoreMember {
    fn describe(&self) -> Result<Member, String> {
        let frame = Frame::axial(self.name, self.offset);
        match self.part {
            CorePart::Wheel(wheel) => Member::new(&wheel.representation(false)?, frame),
            CorePart::Bearing(bearing) => Member::new(&bearing.representation(), frame),
            CorePart::Shim(shim) => Member::new(&shim.representation(), frame),
        }
        .map_err(|e| e.to_string())
    }
    fn ensure<R: Runner>(
        &self,
        library: &mut plugin_kit::component::Library<'_, R>,
        creator: &str,
    ) -> Result<plugin_kit::component::Handle, String> {
        match self.part {
            CorePart::Wheel(wheel) => wheel.ensure(library, creator, false),
            CorePart::Bearing(bearing) => bearing.ensure(library, creator),
            CorePart::Shim(shim) => shim.ensure(library, creator),
        }
    }
}

/// Original polygon construction using the measured [radius, axial] section
/// vertices of the supplier STEP. Exact nominal CAD, without a guessed press fit.
const SOLID: [[f64; 2]; 10] = [
    [8.0, -5.1],
    [9.8, -5.1],
    [11.95, -2.95],
    [11.95, 2.95],
    [9.8, 5.1],
    [8.0, 5.1],
    [8.0, 0.5],
    [7.0, 0.5],
    [7.0, -0.5],
    [8.0, -0.5],
];
const MINI: [[f64; 2]; 10] = [
    [5.0, -4.4],
    [6.105, -4.4],
    [7.615, -2.89],
    [7.615, 2.89],
    [6.105, 4.4],
    [5.0, 4.4],
    [5.0, 0.5],
    [4.32, 0.5],
    [4.32, -0.5],
    [5.0, -0.5],
];
const GROOVED: [[f64; 2]; 16] = [
    [8.0, -5.115],
    [9.805, -5.115],
    [11.95, -2.97],
    [11.95, -2.42],
    [9.805, -0.275],
    [9.375, -0.275],
    [9.375, 0.275],
    [9.805, 0.275],
    [11.95, 2.42],
    [11.95, 2.97],
    [9.805, 5.115],
    [8.0, 5.115],
    [8.0, 0.5],
    [6.95, 0.5],
    [6.95, -0.5],
    [8.0, -0.5],
];

impl Wheel {
    fn recipe(self) -> [CoreMember; 4] {
        [
            CoreMember {
                part: CorePart::Wheel(self),
                name: "wheel",
                offset: 0.0,
            },
            CoreMember {
                part: CorePart::Bearing(self.bearing()),
                name: "bearing-negative",
                offset: -self.bearing_offset(),
            },
            CoreMember {
                part: CorePart::Bearing(self.bearing()),
                name: "bearing-positive",
                offset: self.bearing_offset(),
            },
            CoreMember {
                part: CorePart::Shim(self.shim()),
                name: "internal-shim",
                offset: 0.0,
            },
        ]
    }

    pub fn members(self) -> Result<Vec<Member>, String> {
        self.recipe().iter().map(CoreMember::describe).collect()
    }

    /// Only independently identified supplier numbers are published. The slug
    /// remains a library identifier and must never masquerade as a supplier SKU.
    pub fn sku(self) -> Option<&'static str> {
        match self {
            Self::XtremeSolid => Some("55"),
            Self::XtremeMini => Some("874"),
            _ => None,
        }
    }

    pub fn dimensions(self) -> (f64, f64) {
        let diameter = 2.0 * self.section().iter().map(|p| p[0]).fold(0.0, f64::max);
        let half = self
            .section()
            .iter()
            .map(|p| p[1].abs())
            .fold(0.0, f64::max);
        (diameter, 2.0 * half)
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::Solid => "solid-v-wheel",
            Self::XtremeSolid => "xtreme-solid-v-wheel",
            Self::Delrin => "delrin-v-wheel",
            Self::DelrinMini => "delrin-mini-v-wheel",
            Self::XtremeMini => "xtreme-mini-v-wheel",
        }
    }

    pub fn representation(self, assembled: bool) -> Result<parts::Representation<Self>, String> {
        let mut representation = parts::Representation {
            product: parts::Product {
                catalog: "openbuilds".into(),
                part: self.id().into(),
                kind: parts::ProductKind::SupplierProduct,
                configuration: self,
                members: vec![],
            },
            revision: "1".into(),
            detail: "exterior".into(),
            dependencies: vec![],
        };
        if assembled {
            let members = self.members()?;
            representation.product.catalog = "techsquid-assemblies".into();
            representation.product.kind = parts::ProductKind::Assembly;
            representation.product.members = members.iter().map(|m| m.product.clone()).collect();
            representation.dependencies = members.into_iter().map(|m| m.representation).collect();
        }
        Ok(representation)
    }

    pub fn ensure<R: Runner>(
        self,
        library: &mut plugin_kit::component::Library<'_, R>,
        creator: &str,
        assembled: bool,
    ) -> Result<plugin_kit::component::Handle, String> {
        let name = if assembled {
            format!("{} assembly", self.name())
        } else {
            self.name().into()
        };
        let number = if assembled {
            format!("Assembly {}", self.id())
        } else {
            self.sku()
                .map(|sku| format!("OpenBuilds {sku}"))
                .unwrap_or_else(|| self.name().into())
        };
        library.ensure(
            &plugin_kit::component::Definition {
                plugin: creator,
                base: &name,
                description: &name,
                // A core made from separate purchases is a phantom, not a purchased kit.
                bom_type: if assembled { "phantom" } else { "purchased" },
                site: plugin_kit::component::Site::Origin,
            },
            &number,
            &self.representation(assembled)?,
            |library, name, frame| {
                if !assembled {
                    return library.geometry(name, |mut r| self.build(&mut r, name, frame));
                }
                let parent = library.active_component();
                for member in self.recipe() {
                    let handle = member.ensure(library, creator)?;
                    library.place(
                        &handle,
                        &Placement {
                            point: frame.along(member.offset),
                            ..*frame
                        },
                        parent,
                    )?;
                }
                Ok(())
            },
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Solid => "Solid V Wheel",
            Self::XtremeSolid => "Xtreme Solid V Wheel",
            Self::Delrin => "Delrin V Wheel",
            Self::DelrinMini => "Delrin Mini V Wheel",
            Self::XtremeMini => "Xtreme Mini V Wheel",
        }
    }

    pub fn section(self) -> &'static [[f64; 2]] {
        match self {
            Self::Solid | Self::XtremeSolid => &SOLID,
            Self::Delrin => &GROOVED,
            Self::DelrinMini | Self::XtremeMini => &MINI,
        }
    }

    pub fn bearing(self) -> Bearing {
        match self {
            Self::Solid | Self::XtremeSolid | Self::Delrin => Bearing::OpenBuilds6252rs,
            Self::DelrinMini | Self::XtremeMini => Bearing::OpenBuilds105zz,
        }
    }

    pub fn shim(self) -> Shim {
        match self {
            Self::Solid | Self::XtremeSolid | Self::Delrin => Shim::FullSize,
            Self::DelrinMini | Self::XtremeMini => Shim::Mini,
        }
    }

    /// Two equal bearings seated on opposite sides of one internal shim.
    pub fn bearing_offset(self) -> f64 {
        (self.bearing().dimensions().width + self.shim().thickness()) / 2.0
    }

    pub fn build(self, r: &mut impl Runner, name: &str, frame: &Placement) -> Result<(), String> {
        revolve::build(
            r,
            frame,
            &format!("{name} section"),
            Section::Solid(self.section()),
            json!({"name": name, "body_name": name}),
        )?;
        let (color, opacity) = match self {
            Self::XtremeSolid | Self::XtremeMini => ([220, 225, 230], 0.35),
            Self::Solid | Self::Delrin | Self::DelrinMini => ([40, 40, 40], 1.0),
        };
        // Visual choices describe black Delrin or clear polycarbonate. They are
        // not measured optical properties or an inferred bulk material density.
        r.run(
            "appearance.assign",
            json!({"bodies": [name], "color": color, "opacity": opacity}),
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Shim {
    FullSize,
    Mini,
}

impl Shim {
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

    pub fn ensure<R: Runner>(
        self,
        library: &mut plugin_kit::component::Library<'_, R>,
        creator: &str,
    ) -> Result<plugin_kit::component::Handle, String> {
        let name = format!("Precision shim {} x 5 x 1", self.diameter());
        library.ensure(
            &plugin_kit::component::Definition {
                plugin: creator,
                base: &name,
                description: "OpenBuilds internal wheel shim",
                bom_type: "purchased",
                site: plugin_kit::component::Site::Origin,
            },
            &format!("OpenBuilds {}", self.sku()),
            &self.representation(),
            |library, name, frame| library.geometry(name, |mut r| self.build(&mut r, name, frame)),
        )
    }

    pub fn diameter(self) -> f64 {
        match self {
            Self::FullSize => 10.0,
            Self::Mini => 8.0,
        }
    }
    pub fn bore(self) -> f64 {
        5.0
    }
    pub fn thickness(self) -> f64 {
        1.0
    }
    pub fn sku(self) -> &'static str {
        match self {
            Self::FullSize => "05",
            Self::Mini => "170",
        }
    }
    pub fn build(self, r: &mut impl Runner, name: &str, frame: &Placement) -> Result<(), String> {
        let (inner, outer, half) = (
            self.bore() / 2.0,
            self.diameter() / 2.0,
            self.thickness() / 2.0,
        );
        revolve::build(
            r,
            frame,
            &format!("{name} section"),
            Section::Solid(&[[inner, -half], [outer, -half], [outer, half], [inner, half]]),
            json!({"name": name, "body_name": name}),
        )?;
        r.run(
            "appearance.assign",
            json!({"bodies": [name], "color": [160,160,160]}),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn volume(section: &[[f64; 2]]) -> f64 {
        let mut moment = 0.0;
        for (a, b) in section.iter().zip(section.iter().cycle().skip(1)) {
            moment += (a[0] + b[0]) * (a[0] * b[1] - b[0] * a[1]);
        }
        (PI * moment / 3.0).abs()
    }

    #[test]
    fn shared_shapes_keep_distinct_products() {
        assert_eq!(Wheel::Solid.section(), Wheel::XtremeSolid.section());
        assert_eq!(Wheel::DelrinMini.section(), Wheel::XtremeMini.section());
        assert_ne!(Wheel::Solid, Wheel::XtremeSolid);
        assert_ne!(Wheel::Solid.section(), Wheel::Delrin.section());
    }

    #[test]
    fn assembly_identity_pins_products_and_representations_separately() {
        let assembly = Wheel::Solid.representation(true).unwrap();
        assert_eq!(assembly.product.members.len(), 4);
        assert_eq!(assembly.dependencies.len(), 4);
        assert_eq!(assembly.product.members[1], assembly.product.members[2]);
        assert_eq!(assembly.dependencies[1], assembly.dependencies[2]);
        let mut revised = assembly.clone();
        let mut bearing = Bearing::OpenBuilds6252rs.representation();
        bearing.revision = "2".into();
        revised.dependencies[1] = bearing.identity().unwrap();
        revised.dependencies[2] = bearing.identity().unwrap();
        assert_eq!(assembly.product.identity(), revised.product.identity());
        assert_ne!(assembly.identity(), revised.identity());
        revised.product.members[1] = Bearing::OpenBuilds105zz
            .representation()
            .product
            .identity()
            .unwrap();
        assert_ne!(assembly.product.identity(), revised.product.identity());
    }

    #[test]
    fn assembly_stack_matches_reference_and_mini_correction() {
        assert_eq!(Wheel::Solid.bearing_offset(), 3.0);
        assert_eq!(Wheel::DelrinMini.bearing_offset(), 2.5);
        assert_eq!(
            (
                Shim::FullSize.diameter(),
                Shim::FullSize.bore(),
                Shim::FullSize.thickness()
            ),
            (10.0, 5.0, 1.0)
        );
        assert_eq!(
            (
                Shim::Mini.diameter(),
                Shim::Mini.bore(),
                Shim::Mini.thickness()
            ),
            (8.0, 5.0, 1.0)
        );
        assert!((volume(Wheel::Solid.section()) - 2246.0293078).abs() < 0.0001);
    }

    #[test]
    fn sections_match_independently_measured_reference_vertices() {
        // Axial/radial vertex inventory from supplier STEP, independent of polygon
        // winding. Host tests additionally compare imported body volumes and bounds.
        for (wheel, mut expected) in [
            (
                Wheel::Solid,
                vec![
                    [-5.1, 8.0],
                    [-5.1, 9.8],
                    [-2.95, 11.95],
                    [-0.5, 7.0],
                    [-0.5, 8.0],
                    [0.5, 7.0],
                    [0.5, 8.0],
                    [2.95, 11.95],
                    [5.1, 8.0],
                    [5.1, 9.8],
                ],
            ),
            (
                Wheel::DelrinMini,
                vec![
                    [-4.4, 5.0],
                    [-4.4, 6.105],
                    [-2.89, 7.615],
                    [-0.5, 4.32],
                    [-0.5, 5.0],
                    [0.5, 4.32],
                    [0.5, 5.0],
                    [2.89, 7.615],
                    [4.4, 5.0],
                    [4.4, 6.105],
                ],
            ),
            (
                Wheel::Delrin,
                vec![
                    [-5.115, 8.0],
                    [-5.115, 9.805],
                    [-2.97, 11.95],
                    [-2.42, 11.95],
                    [-0.5, 6.95],
                    [-0.5, 8.0],
                    [-0.275, 9.375],
                    [-0.275, 9.805],
                    [0.275, 9.375],
                    [0.275, 9.805],
                    [0.5, 6.95],
                    [0.5, 8.0],
                    [2.42, 11.95],
                    [2.97, 11.95],
                    [5.115, 8.0],
                    [5.115, 9.805],
                ],
            ),
        ] {
            let mut actual: Vec<_> = wheel.section().iter().map(|p| [p[1], p[0]]).collect();
            actual.sort_by(|a, b| a.partial_cmp(b).unwrap());
            expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(actual, expected);
        }
    }
}
