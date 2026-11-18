//! Radial bearings selected by nominal envelope and closure. Supplier identity and
//! the evidence for a modeled exterior are independent. See docs/bearing-coverage.md.
//! Open B626 includes measured raceways and balls. Closed bearings omit hidden internals.
//! CSK reference presets and customizable design specifications live in `csk`.

use plugin_kit::{name::part, revolve::Section, Placement, Runner};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub mod build;
pub mod csk;
pub mod exterior;
mod profile;
mod selection;

pub use build::Envelope;
pub use selection::{Closure, EnvelopeFilter, EnvelopeMatch, Size};

/// Stable catalog namespace kept separate from the supplier's display name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Supplier {
    pub catalog: &'static str,
    pub name: &'static str,
}

pub const OPENBUILDS: Supplier = Supplier {
    catalog: "openbuilds",
    name: "OpenBuilds",
};
pub const MISUMI: Supplier = Supplier {
    catalog: "misumi",
    name: "MISUMI",
};

/// Modeled visible ring geometry, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct RingSection {
    pub inner_ring_radius: f64,
    pub outer_ring_radius: f64,
    /// How far each closure face sits inside the width; `None` for an open bearing.
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

/// Everything that tells one product from another.
struct Product {
    supplier: Option<Supplier>,
    /// Supplier number or a local catalog key for a design specification.
    part: &'static str,
    designation: &'static str,
    size: Size,
    closure: Closure,
    section: SectionSource,
}

enum SectionSource {
    Measured(RingSection),
    FromBearing(Bearing),
}

/// A catalog key doubles as its display designation unless a row gives a (key, designation)
/// pair. A measured section lists the two shoulder radii, face recess and edge radius.
/// An estimated section names its measured donor. Serialized identities belong to saved designs.
macro_rules! products {
    (@names ($part:literal, $designation:literal)) => { ($part, $designation) };
    (@names $part:literal) => { ($part, $part) };
    (@section [$inner:literal, $outer:literal, $recess:expr, $edge:literal]) => {
        SectionSource::Measured(RingSection {
            inner_ring_radius: $inner, outer_ring_radius: $outer,
            closure_recess: $recess, edge_radius: $edge,
        })
    };
    (@section $donor:ident) => { SectionSource::FromBearing(Bearing::$donor) };
    ($( $variant:ident, $id:literal, $supplier:expr, $names:tt,
        $size:ident, $closure:ident, $section:tt; )*) => {
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
        pub enum Bearing { $( #[serde(rename = $id)] $variant, )* }
        pub const ALL: [Bearing; [$(Bearing::$variant,)*].len()] = [$(Bearing::$variant,)*];
        impl Bearing {
            fn product(self) -> Product {
                match self { $(Self::$variant => {
                    let (part, designation) = products!(@names $names);
                    Product { supplier: $supplier, part, designation, size: Size::$size,
                        closure: Closure::$closure, section: products!(@section $section) }
                },)* }
            }
        }
    }
}

// Exact supplier STEP sections: docs/sources/{openbuilds,misumi}-measurements.json.
products! {
    OpenBuilds6252rs, "openbuilds-625-2rs", Some(OPENBUILDS), ("30", "625 2RS"), S625, RubberSeals, [4.3, 6.8, Some(0.3), 0.0];
    OpenBuilds105zz, "openbuilds-105zz", Some(OPENBUILDS), ("800", "105ZZ"), S105, MetalShields, [3.1, 4.6, Some(0.3), 0.0];
    MisumiB626, "misumi-b626", Some(MISUMI), "B626", S626, Open, [5.06, 7.55, None, 0.3];
    MisumiB626zz, "misumi-b626zz", Some(MISUMI), "B626ZZ", S626, MetalShields, [4.75, 8.15, Some(0.5), 0.3];
    MisumiB626ddu, "misumi-b626ddu", Some(MISUMI), "B626DDU", S626, RubberSeals, [4.215, 8.1775, Some(0.11), 0.3];
    MisumiB608zz, "misumi-b608zz", Some(MISUMI), "B608ZZ", S608, MetalShields, [6.1, 9.65, Some(0.6), 0.3];
    MisumiB6000zz, "misumi-b6000zz", Some(MISUMI), "B6000ZZ", S6000, MetalShields, [6.95, 11.1, Some(0.33), 0.3];
    MisumiB6001zz, "misumi-b6001zz", Some(MISUMI), "B6001ZZ", S6001, MetalShields, [8.15, 12.45, Some(0.435), 0.3];
    MisumiB6002zz, "misumi-b6002zz", Some(MISUMI), "B6002ZZ", S6002, MetalShields, [9.65, 13.95, Some(0.435), 0.3];
    MisumiB6003zz, "misumi-b6003zz", Some(MISUMI), "B6003ZZ", S6003, MetalShields, [10.7, 15.4, Some(0.435), 0.3];
    MisumiB6004zz, "misumi-b6004zz", Some(MISUMI), "B6004ZZ", S6004, MetalShields, [13.2, 18.4, Some(0.435), 0.6];
    MisumiB6200zz, "misumi-b6200zz", Some(MISUMI), "B6200ZZ", S6200, MetalShields, [8.15, 12.45, Some(0.435), 0.6];
    MisumiB6201zz, "misumi-b6201zz", Some(MISUMI), "B6201ZZ", S6201, MetalShields, [8.7, 13.4, Some(0.435), 0.6];
    MisumiB6202zz, "misumi-b6202zz", Some(MISUMI), "B6202ZZ", S6202, MetalShields, [9.85, 15.45, Some(0.34), 0.6];
    MisumiB6203zz, "misumi-b6203zz", Some(MISUMI), "B6203ZZ", S6203, MetalShields, [11.5, 17.6, Some(0.34), 0.6];
    MisumiB6204zz, "misumi-b6204zz", Some(MISUMI), "B6204ZZ", S6204, MetalShields, [14.0, 20.8, Some(0.53), 1.0];

    // Catalog identities and same-size exterior donors: docs/sources/misumi.md.
    MisumiB625zz, "misumi-b625zz", Some(MISUMI), "B625ZZ", S625, MetalShields, OpenBuilds6252rs;
    Sealed105, "105-sealed", None, ("105-sealed", "105 sealed"), S105, RubberSeals, OpenBuilds105zz;
    MisumiB608ddu, "misumi-b608ddu", Some(MISUMI), "B608DDU", S608, RubberSeals, MisumiB608zz;
    MisumiB6000ddu, "misumi-b6000ddu", Some(MISUMI), "B6000DDU", S6000, RubberSeals, MisumiB6000zz;
    MisumiB6001ddu, "misumi-b6001ddu", Some(MISUMI), "B6001DDU", S6001, RubberSeals, MisumiB6001zz;
    MisumiB6002ddu, "misumi-b6002ddu", Some(MISUMI), "B6002DDU", S6002, RubberSeals, MisumiB6002zz;
    MisumiB6003ddu, "misumi-b6003ddu", Some(MISUMI), "B6003DDU", S6003, RubberSeals, MisumiB6003zz;
    MisumiB6004ddu, "misumi-b6004ddu", Some(MISUMI), "B6004DDU", S6004, RubberSeals, MisumiB6004zz;
    MisumiB6200ddu, "misumi-b6200ddu", Some(MISUMI), "B6200DDU", S6200, RubberSeals, MisumiB6200zz;
    MisumiB6201ddu, "misumi-b6201ddu", Some(MISUMI), "B6201DDU", S6201, RubberSeals, MisumiB6201zz;
    MisumiB6202ddu, "misumi-b6202ddu", Some(MISUMI), "B6202DDU", S6202, RubberSeals, MisumiB6202zz;
    MisumiB6203ddu, "misumi-b6203ddu", Some(MISUMI), "B6203DDU", S6203, RubberSeals, MisumiB6203zz;
    MisumiB6204ddu, "misumi-b6204ddu", Some(MISUMI), "B6204DDU", S6204, RubberSeals, MisumiB6204zz;
}

impl Bearing {
    pub fn representation(self) -> parts::Representation<Self> {
        parts::Representation {
            product: parts::Product {
                catalog: self
                    .supplier()
                    .map_or("techsquid-specifications", |s| s.catalog)
                    .into(),
                part: self.catalog_part().into(),
                kind: self.kind(),
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
        let number = self
            .supplier()
            .map(|s| format!("{} {}", s.name, self.catalog_part()));
        library.ensure(
            &plugin_kit::component::Definition {
                plugin: creator,
                base: &name,
                description: self.designation(),
                bom_type: if number.is_some() { "purchased" } else { "normal" },
                site: plugin_kit::component::Site::Origin,
            },
            number.as_deref().unwrap_or(""),
            &self.representation(),
            |library, name, frame| library.geometry(name, |mut r| {
                self.build(&mut r, name, frame)?;
                if let SectionSource::FromBearing(donor) = self.product().section {
                    let provenance = exterior::Provenance {
                        basis: exterior::Basis::Estimated,
                        method: "Copied the measured exterior of the same size with the other closure",
                        sources: [donor],
                    };
                    r.run("component.metadata.set", json!({"component":name,"values":{
                        "org.techsquid.bearing.exterior": {"version":1,"section":self.section(),"provenance":provenance}
                    }}))?;
                }
                Ok(())
            }),
        )
    }

    pub fn designation(self) -> &'static str {
        self.product().designation
    }

    pub fn supplier(self) -> Option<Supplier> {
        self.product().supplier
    }

    /// Exact supplier part number; the underlying bearing manufacturer may differ.
    pub fn sku(self) -> Option<&'static str> {
        self.supplier().map(|_| self.catalog_part())
    }

    pub fn catalog_part(self) -> &'static str {
        self.product().part
    }

    pub fn kind(self) -> parts::ProductKind {
        if self.supplier().is_some() {
            parts::ProductKind::SupplierProduct
        } else {
            parts::ProductKind::DesignSpecification
        }
    }

    pub fn basis(self) -> exterior::Basis {
        match self.product().section {
            SectionSource::Measured(_) => exterior::Basis::Measured,
            SectionSource::FromBearing(_) => exterior::Basis::Estimated,
        }
    }

    pub fn size(self) -> Size {
        self.product().size
    }

    pub fn closure(self) -> Closure {
        self.product().closure
    }

    pub fn dimensions(self) -> Envelope {
        self.size().envelope()
    }

    pub fn section(self) -> RingSection {
        match self.product().section {
            SectionSource::Measured(section) => section,
            SectionSource::FromBearing(donor) => donor.section(),
        }
    }

    /// OpenBuilds publishes one build page. MISUMI lists each closure of its deep groove
    /// bearings as one family, and the part number selects the size within it.
    pub fn product_url(self) -> Option<String> {
        if self.supplier()? == OPENBUILDS {
            return Some(
                "https://builds.openbuilds.com/builds/openbuilds-sphinx-1050-20-x-40.7196/".into(),
            );
        }
        let family = match self.closure() {
            Closure::Open => "110302273540",
            Closure::RubberSeals => "110302273630",
            Closure::MetalShields => "110300107560",
        };
        Some(format!(
            "https://us.misumi-ec.com/vona2/detail/{family}/?HissuCode={}",
            self.catalog_part()
        ))
    }

    fn open_race(self) -> Option<OpenRace> {
        (self == Self::MisumiB626).then_some(OpenRace {
            pitch_radius: 6.25,
            groove_radius: 1.95,
            ball_radius: 1.975,
            ball_count: 8,
        })
    }

    /// The axle is +Z and the origin is the bearing's centre plane. A bearing is
    /// one component with its own BOM identity.
    pub fn build(self, r: &mut impl Runner, base: &str, frame: &Placement) -> Result<(), String> {
        let d = self.dimensions();
        let half = d.width / 2.0;
        let section = self.section();
        let square =
            |inner: f64, outer: f64| [[inner, -half], [outer, -half], [outer, half], [inner, half]];
        let polygons = [
            square(d.bore / 2.0, section.inner_ring_radius),
            square(section.outer_ring_radius, d.diameter / 2.0),
        ];
        let curved = [
            profile::ring(self, &section, false),
            profile::ring(self, &section, true),
        ];
        let ring = |outer: usize| {
            if section.edge_radius == 0.0 {
                Section::Solid(&polygons[outer])
            } else {
                Section::CurvedSolid(&curved[outer])
            }
        };
        let mut steel = build::rings(r, frame, base, [ring(0), ring(1)])?;
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
                    "name":part(base, &format!("Ball {}", index + 1)), "body_name":name}),
                )?;
                steel.push(name);
            }
        }
        build::colour(r, &steel, build::STEEL)?;
        let (label, colour) = match self.closure() {
            Closure::Open => return Ok(()),
            Closure::RubberSeals => ("Seal", build::RUBBER),
            Closure::MetalShields => ("Shield", build::STEEL),
        };
        let recess = section
            .closure_recess
            .ok_or("closure: closed bearing needs a face recess")?;
        let shoulders = [section.inner_ring_radius, section.outer_ring_radius];
        let mut closures = Vec::new();
        for (side, sign) in [("front", 1.0), ("back", -1.0)] {
            let label = format!("{label} {side}");
            let z = sign * (half - recess);
            closures.push(build::face(r, frame, base, &label, z, shoulders)?);
        }
        build::colour(r, &closures, colour)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_features_use_the_shared_namespace_separate_from_bodies() {
        #[derive(Default)]
        struct Host(Vec<(String, serde_json::Value)>);
        impl Runner for Host {
            fn run(
                &mut self,
                command: &str,
                args: serde_json::Value,
            ) -> Result<serde_json::Value, String> {
                self.0.push((command.into(), args));
                Ok(json!({"sketch": self.0.len()}))
            }
        }
        for bearing in ALL {
            let mut host = Host::default();
            bearing
                .build(&mut host, "Part", &Placement::on_origin())
                .unwrap();
            let mut names = std::collections::BTreeSet::new();
            for (command, args) in host.0 {
                if let Some(name) = args["name"].as_str() {
                    assert!(name.starts_with("Part: "), "{command}: {name}");
                    assert!(names.insert(name.to_string()), "duplicate {name}");
                }
                if let Some(body) = args["body_name"].as_str() {
                    assert!(body.starts_with("Part "));
                    assert!(!body.starts_with("Part: "));
                }
            }
        }
    }

    #[test]
    fn exterior_dimensions_match_source_and_mini_correction() {
        // Independently transcribed from the supplier STEP and correction ledger.
        for (bearing, expected) in [
            (Bearing::OpenBuilds6252rs, [5.0, 16.0, 5.0, 4.3, 6.8, 0.3]),
            (Bearing::OpenBuilds105zz, [5.0, 10.0, 4.0, 3.1, 4.6, 0.3]),
        ] {
            let d = bearing.dimensions();
            let section = bearing.section();
            assert_eq!(
                [
                    d.bore,
                    d.diameter,
                    d.width,
                    section.inner_ring_radius,
                    section.outer_ring_radius,
                    section.closure_recess.unwrap()
                ],
                expected
            );
        }
    }

    #[test]
    fn misumi_product_urls_match_source_ledger() {
        let sources: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../docs/sources/misumi-files.json")).unwrap();
        for source in sources {
            let sku = source["file"]
                .as_str()
                .unwrap()
                .strip_suffix(".stp")
                .unwrap();
            let bearing = ALL.into_iter().find(|b| b.sku() == Some(sku)).unwrap();
            assert_eq!(
                bearing.product_url().as_deref(),
                source["product_url"].as_str()
            );
        }
    }

    #[test]
    fn supplier_variants_do_not_conflate_closure_or_identity() {
        assert_eq!(Bearing::OpenBuilds6252rs.sku(), Some("30"));
        assert_eq!(Bearing::OpenBuilds6252rs.closure(), Closure::RubberSeals);
        assert_eq!(Bearing::OpenBuilds105zz.sku(), Some("800"));
        assert_eq!(Bearing::OpenBuilds105zz.closure(), Closure::MetalShields);
        assert!(serde_json::from_str::<Bearing>("\"openbuilds-5x10x4.2\"").is_err());
    }

    #[test]
    fn estimated_catalog_envelopes_and_donors() {
        // MISUMI 2014 pp. 1003–1004 (DDU) and 2019 p. 1028 (B625ZZ).
        // 105 sealed is a design specification using the OpenBuilds 105ZZ envelope.
        use Bearing::*;
        let expected = [
            (
                MisumiB625zz,
                Some("B625ZZ"),
                [5., 16., 5.],
                OpenBuilds6252rs,
            ),
            (Sealed105, None, [5., 10., 4.], OpenBuilds105zz),
            (MisumiB608ddu, Some("B608DDU"), [8., 22., 7.], MisumiB608zz),
            (
                MisumiB6000ddu,
                Some("B6000DDU"),
                [10., 26., 8.],
                MisumiB6000zz,
            ),
            (
                MisumiB6001ddu,
                Some("B6001DDU"),
                [12., 28., 8.],
                MisumiB6001zz,
            ),
            (
                MisumiB6002ddu,
                Some("B6002DDU"),
                [15., 32., 9.],
                MisumiB6002zz,
            ),
            (
                MisumiB6003ddu,
                Some("B6003DDU"),
                [17., 35., 10.],
                MisumiB6003zz,
            ),
            (
                MisumiB6004ddu,
                Some("B6004DDU"),
                [20., 42., 12.],
                MisumiB6004zz,
            ),
            (
                MisumiB6200ddu,
                Some("B6200DDU"),
                [10., 30., 9.],
                MisumiB6200zz,
            ),
            (
                MisumiB6201ddu,
                Some("B6201DDU"),
                [12., 32., 10.],
                MisumiB6201zz,
            ),
            (
                MisumiB6202ddu,
                Some("B6202DDU"),
                [15., 35., 11.],
                MisumiB6202zz,
            ),
            (
                MisumiB6203ddu,
                Some("B6203DDU"),
                [17., 40., 12.],
                MisumiB6203zz,
            ),
            (
                MisumiB6204ddu,
                Some("B6204DDU"),
                [20., 47., 14.],
                MisumiB6204zz,
            ),
        ];
        assert_eq!(
            ALL.iter()
                .filter(|b| b.basis() == exterior::Basis::Estimated)
                .count(),
            expected.len()
        );
        for (b, sku, dims, donor) in expected {
            let d = b.dimensions();
            assert_eq!([d.bore, d.diameter, d.width], dims);
            assert_eq!(b.sku(), sku);
            assert_eq!(b.supplier(), sku.map(|_| MISUMI));
            assert!(
                matches!(b.product().section,SectionSource::FromBearing(source) if source==donor)
            );
            assert_eq!(donor.basis(), exterior::Basis::Measured);
            assert_eq!(b.section(), donor.section());
            assert_ne!(b.closure(), donor.closure());
        }
    }

    #[test]
    fn misumi_sections_match_supplied_source_revisions() {
        // Independently transcribed from docs/sources/misumi-measurements.json:
        // bore, OD, width, shoulder radii and edge radius from each exact STEP.
        for (bearing, dims, recess, closure) in [
            (
                Bearing::MisumiB626,
                [6.0, 19.0, 6.0, 5.06, 7.55, 0.3],
                None,
                Closure::Open,
            ),
            (
                Bearing::MisumiB626zz,
                [6.0, 19.0, 6.0, 4.75, 8.15, 0.3],
                Some(0.5),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB626ddu,
                [6.0, 19.0, 6.0, 4.215, 8.1775, 0.3],
                Some(0.11),
                Closure::RubberSeals,
            ),
            (
                Bearing::MisumiB608zz,
                [8.0, 22.0, 7.0, 6.1, 9.65, 0.3],
                Some(0.6),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6000zz,
                [10.0, 26.0, 8.0, 6.95, 11.1, 0.3],
                Some(0.33),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6001zz,
                [12.0, 28.0, 8.0, 8.15, 12.45, 0.3],
                Some(0.435),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6002zz,
                [15.0, 32.0, 9.0, 9.65, 13.95, 0.3],
                Some(0.435),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6003zz,
                [17.0, 35.0, 10.0, 10.7, 15.4, 0.3],
                Some(0.435),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6004zz,
                [20.0, 42.0, 12.0, 13.2, 18.4, 0.6],
                Some(0.435),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6200zz,
                [10.0, 30.0, 9.0, 8.15, 12.45, 0.6],
                Some(0.435),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6201zz,
                [12.0, 32.0, 10.0, 8.7, 13.4, 0.6],
                Some(0.435),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6202zz,
                [15.0, 35.0, 11.0, 9.85, 15.45, 0.6],
                Some(0.34),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6203zz,
                [17.0, 40.0, 12.0, 11.5, 17.6, 0.6],
                Some(0.34),
                Closure::MetalShields,
            ),
            (
                Bearing::MisumiB6204zz,
                [20.0, 47.0, 14.0, 14.0, 20.8, 1.0],
                Some(0.53),
                Closure::MetalShields,
            ),
        ] {
            let d = bearing.dimensions();
            let section = bearing.section();
            assert_eq!(
                [
                    d.bore,
                    d.diameter,
                    d.width,
                    section.inner_ring_radius,
                    section.outer_ring_radius,
                    section.edge_radius
                ],
                dims
            );
            assert_eq!(section.closure_recess, recess);
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
