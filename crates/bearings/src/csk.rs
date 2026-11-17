//! CSK reference presets and customizable exterior geometry. See docs/sources/csk.md.
use super::Dimensions;
pub mod exterior;
use plugin_kit::{
    host::sketch_id,
    name::part,
    revolve::{self, Section},
    Placement, Runner,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Copy)]
enum Size {
    S8,
    S10,
    S12,
    S15,
    S17,
    S20,
    S25,
    S30,
    S35,
    S40,
}
#[derive(Clone, Copy)]
enum Mounting {
    Plain,
    Inner,
    Both,
}

/// Visible closures; their colour is illustrative, not a material assignment.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Closure {
    DustCovers,
    LipSeals,
    SingleLipSeal,
}

/// Nominal rectangular keyway: depth is measured radially from the bore/OD apex.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Keyway {
    pub width: f64,
    pub depth: f64,
    /// Counterclockwise from local +Y, viewed from +Z; normalized to [0, 360).
    pub angle_degrees: f64,
}

macro_rules! products {
    ($( $variant:ident, $label:literal, $size:ident, $mount:ident, $sealed:literal; )*) => {
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
        pub enum Csk { $( #[serde(rename = $label)] $variant, )* }
        pub const ALL: [Csk; 55] = [$(Csk::$variant,)*];
        impl Csk {
            pub fn designation(self) -> &'static str { match self { $(Self::$variant => $label,)* } }
            fn configuration(self) -> (Size, Mounting, bool) {
                match self { $(Self::$variant => (Size::$size, Mounting::$mount, $sealed),)* }
            }
        }
    }
}
products! {
    Csk8, "CSK8", S8, Plain, false;
    Csk8P, "CSK8P", S8, Inner, false;
    Csk8Pp, "CSK8PP", S8, Both, false;
    Csk82Rs, "CSK8-2RS", S8, Plain, true;
    Csk8P2Rs, "CSK8P-2RS", S8, Inner, true;
    Csk10, "CSK10", S10, Plain, false;
    Csk10P, "CSK10P", S10, Inner, false;
    Csk10Pp, "CSK10PP", S10, Both, false;
    Csk12, "CSK12", S12, Plain, false;
    Csk12P, "CSK12P", S12, Inner, false;
    Csk12Pp, "CSK12PP", S12, Both, false;
    Csk122Rs, "CSK12-2RS", S12, Plain, true;
    Csk12P2Rs, "CSK12P-2RS", S12, Inner, true;
    Csk15, "CSK15", S15, Plain, false;
    Csk15P, "CSK15P", S15, Inner, false;
    Csk15Pp, "CSK15PP", S15, Both, false;
    Csk152Rs, "CSK15-2RS", S15, Plain, true;
    Csk15P2Rs, "CSK15P-2RS", S15, Inner, true;
    Csk15Pp2Rs, "CSK15PP-2RS", S15, Both, true;
    Csk17, "CSK17", S17, Plain, false;
    Csk17P, "CSK17P", S17, Inner, false;
    Csk17Pp, "CSK17PP", S17, Both, false;
    Csk172Rs, "CSK17-2RS", S17, Plain, true;
    Csk17P2Rs, "CSK17P-2RS", S17, Inner, true;
    Csk17Pp2Rs, "CSK17PP-2RS", S17, Both, true;
    Csk20, "CSK20", S20, Plain, false;
    Csk20P, "CSK20P", S20, Inner, false;
    Csk20Pp, "CSK20PP", S20, Both, false;
    Csk202Rs, "CSK20-2RS", S20, Plain, true;
    Csk20P2Rs, "CSK20P-2RS", S20, Inner, true;
    Csk20Pp2Rs, "CSK20PP-2RS", S20, Both, true;
    Csk25, "CSK25", S25, Plain, false;
    Csk25P, "CSK25P", S25, Inner, false;
    Csk25Pp, "CSK25PP", S25, Both, false;
    Csk252Rs, "CSK25-2RS", S25, Plain, true;
    Csk25P2Rs, "CSK25P-2RS", S25, Inner, true;
    Csk25Pp2Rs, "CSK25PP-2RS", S25, Both, true;
    Csk30, "CSK30", S30, Plain, false;
    Csk30P, "CSK30P", S30, Inner, false;
    Csk30Pp, "CSK30PP", S30, Both, false;
    Csk302Rs, "CSK30-2RS", S30, Plain, true;
    Csk30P2Rs, "CSK30P-2RS", S30, Inner, true;
    Csk30Pp2Rs, "CSK30PP-2RS", S30, Both, true;
    Csk35, "CSK35", S35, Plain, false;
    Csk35P, "CSK35P", S35, Inner, false;
    Csk35Pp, "CSK35PP", S35, Both, false;
    Csk352Rs, "CSK35-2RS", S35, Plain, true;
    Csk35P2Rs, "CSK35P-2RS", S35, Inner, true;
    Csk35Pp2Rs, "CSK35PP-2RS", S35, Both, true;
    Csk40, "CSK40", S40, Plain, false;
    Csk40P, "CSK40P", S40, Inner, false;
    Csk40Pp, "CSK40PP", S40, Both, false;
    Csk402Rs, "CSK40-2RS", S40, Plain, true;
    Csk40P2Rs, "CSK40P-2RS", S40, Inner, true;
    Csk40Pp2Rs, "CSK40PP-2RS", S40, Both, true;
}

impl Csk {
    pub fn source(self) -> parts::catalog::Source {
        use Csk::*;
        let (title, url) = match self {
            Csk8P | Csk8Pp | Csk12Pp => ("PEAK CSK dimensions and keyways", "https://www.peakbearings.com/keywaysize.htm"),
            Csk10 | Csk10P | Csk10Pp => ("Yuki CSK dimensions and keyways", "https://www.ywsbearing.com/25x52x15mm-internal-and-external-keyway-without-groove-One-way-bearing-CSK25PP-BB25-CSK6205-CSK25P-CSK25-pd46892679.html"),
            Csk8P2Rs => ("DKC CSK catalog, page 3", "https://dkc-as.dk/wp-content/uploads/2015/05/DKC-FRIL%C3%98BSLEJER-ENVEJSLEJER-FREEWHEEL-BEARINGS-ONEWAY-CLUTCHES-needle-clutch-brg.pdf"),
            Csk15Pp2Rs | Csk17Pp2Rs | Csk20Pp2Rs | Csk25Pp2Rs | Csk35Pp2Rs | Csk40Pp2Rs => ("PTI CSK series catalog", "https://static.s123-cdn-static-d.com/uploads/1952747/normal_5eb6121f4e1cf.pdf"),
            Csk8 | Csk12 | Csk15 | Csk17 | Csk20 | Csk25 | Csk30 | Csk35 | Csk40 | Csk82Rs | Csk122Rs | Csk152Rs | Csk172Rs | Csk202Rs | Csk252Rs | Csk302Rs | Csk352Rs | Csk402Rs => ("Formsprag CSK catalog dimensions", "https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf"),
            _ => ("Formsprag CSK catalog dimensions", "https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf"),
        };
        parts::catalog::Source {
            title: title.into(),
            url: url.into(),
        }
    }

    pub fn mounting_description(self) -> &'static str {
        match self.configuration() {
            (_, Mounting::Plain, false) => "Unkeyed",
            (Size::S8, Mounting::Plain, true) => "Unkeyed; one lip seal",
            (_, Mounting::Plain, true) => "Unkeyed; 2RS",
            (_, Mounting::Inner, false) => "Inner keyway",
            (_, Mounting::Inner, true) => "Inner keyway; 2RS",
            (_, Mounting::Both, false) => "Inner and outer keyways",
            (_, Mounting::Both, true) => "Inner and outer keyways; 2RS",
        }
    }

    pub fn dimensions(self) -> Dimensions {
        let (size, _, sealed) = self.configuration();
        let (bore, diameter, plain_width, sealed_width) = match size {
            Size::S8 => (8.0, 22.0, 9.0, 9.0),
            Size::S10 => (10.0, 30.0, 9.0, 9.0),
            Size::S12 => (12.0, 32.0, 10.0, 14.0),
            Size::S15 => (15.0, 35.0, 11.0, 16.0),
            Size::S17 => (17.0, 40.0, 12.0, 17.0),
            Size::S20 => (20.0, 47.0, 14.0, 19.0),
            Size::S25 => (25.0, 52.0, 15.0, 20.0),
            Size::S30 => (30.0, 62.0, 16.0, 21.0),
            Size::S35 => (35.0, 72.0, 17.0, 22.0),
            Size::S40 => (40.0, 80.0, 22.0, 27.0),
        };
        Dimensions {
            bore,
            diameter,
            width: if sealed { sealed_width } else { plain_width },
            section: None,
        }
    }
    pub fn keyways(self) -> (Option<Keyway>, Option<Keyway>) {
        let (size, mount, _) = self.configuration();
        if matches!(mount, Mounting::Plain) {
            return (None, None);
        }
        let (iw, id, ow, od) = match size {
            Size::S8 => (3.0, 0.5, 2.0, 0.5),
            Size::S10 => (3.0, 0.6, 2.0, 0.5),
            Size::S12 => (
                4.0,
                if matches!(mount, Mounting::Both) {
                    1.2
                } else {
                    1.8
                },
                2.0,
                0.6,
            ),
            Size::S15 => (5.0, 1.2, 2.0, 0.6),
            Size::S17 => (5.0, 1.2, 2.0, 1.0),
            Size::S20 => (6.0, 1.6, 3.0, 1.5),
            Size::S25 | Size::S30 => (8.0, 2.0, 6.0, 2.0),
            Size::S35 => (10.0, 2.4, 8.0, 2.5),
            Size::S40 => (12.0, 3.3, 10.0, 3.0),
        };
        (
            Some(Keyway {
                width: iw,
                depth: id,
                angle_degrees: 0.0,
            }),
            matches!(mount, Mounting::Both).then_some(Keyway {
                width: ow,
                depth: od,
                angle_degrees: 180.0,
            }),
        )
    }
    pub fn geometry(self) -> Geometry {
        let (inner_keyway, outer_keyway) = self.keyways();
        let d = self.dimensions();
        Geometry {
            designation: Some(self),
            closure: match self.configuration() {
                (Size::S8, _, true) => Closure::SingleLipSeal,
                (_, _, true) => Closure::LipSeals,
                _ => Closure::DustCovers,
            },
            bore: d.bore,
            diameter: d.diameter,
            width: d.width,
            inner_keyway,
            outer_keyway,
        }
    }
}

/// Resolved dimensions, including explicit overrides. No supplier identity.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    pub designation: Option<Csk>,
    pub closure: Closure,
    pub bore: f64,
    pub diameter: f64,
    pub width: f64,
    pub inner_keyway: Option<Keyway>,
    pub outer_keyway: Option<Keyway>,
}

impl Geometry {
    pub fn customized(&self) -> bool {
        self.designation
            .is_none_or(|preset| *self != preset.geometry())
    }

    pub fn dimensions(&self) -> Dimensions {
        Dimensions {
            bore: self.bore,
            diameter: self.diameter,
            width: self.width,
            section: None,
        }
    }

    pub fn validate(&self) -> Result<(), Invalid> {
        let d = self.dimensions();
        for (part, name, value) in [
            (Part::Bore, "bore", d.bore),
            (Part::Diameter, "outside diameter", d.diameter),
            (Part::Width, "width", d.width),
        ] {
            if !value.is_finite() || value <= 0.0 || value > 1000.0 {
                return Err(Invalid::new(
                    part,
                    format!("{name} must be finite, positive and at most 1000 mm"),
                ));
            }
        }
        if d.bore >= d.diameter {
            return Err(Invalid::new(
                Part::Bore,
                "bore must be smaller than the outside diameter",
            ));
        }
        for (part, key) in [
            (Part::InnerKeyway, self.inner_keyway),
            (Part::OuterKeyway, self.outer_keyway),
        ] {
            let Some(key) = key else { continue };
            if !key.angle_degrees.is_finite() || !(0.0..360.0).contains(&key.angle_degrees) {
                return Err(Invalid::new(
                    part,
                    "keyway angles must be finite and normalized to [0, 360) degrees",
                ));
            }
            if !key.width.is_finite()
                || !key.depth.is_finite()
                || key.width <= 0.0
                || key.depth <= 0.0
            {
                return Err(Invalid::new(
                    part,
                    "keyway dimensions must be finite and positive",
                ));
            }
        }
        if let Some(key) = self.inner_keyway {
            if key.width >= d.bore
                || (d.bore / 2.0 + key.depth).hypot(key.width / 2.0) >= d.diameter / 2.0
            {
                return Err(Invalid::new(
                    Part::InnerKeyway,
                    "inner keyway must stay inside the outside diameter and narrower than the bore",
                ));
            }
        }
        if let Some(key) = self.outer_keyway {
            let floor = d.diameter / 2.0 - key.depth;
            if floor <= d.bore / 2.0 || floor.hypot(key.width / 2.0) >= d.diameter / 2.0 {
                return Err(Invalid::new(
                    Part::OuterKeyway,
                    "outer keyway must leave material around the bore and intersect the outside diameter",
                ));
            }
            if self
                .inner_keyway
                .is_some_and(|inner| d.bore / 2.0 + inner.depth >= floor)
            {
                return Err(Invalid::new(
                    Part::OuterKeyway,
                    "inner and outer keyways must not meet",
                ));
            }
        }
        // Only keyways leave the rings too little room; without one it is the envelope.
        let crowded = if self.outer_keyway.is_some() {
            Part::OuterKeyway
        } else if self.inner_keyway.is_some() {
            Part::InnerKeyway
        } else {
            Part::Diameter
        };
        exterior::resolve(self).map_err(|reason| Invalid::new(crowded, reason))?;
        Ok(())
    }

    pub fn build(&self, r: &mut impl Runner, base: &str, frame: &Placement) -> Result<(), String> {
        self.validate()?;
        let exterior = exterior::resolve(self)?;
        let section = &exterior.section;
        let mut steel = Vec::new();
        for (label, outer) in [("Inner ring", false), ("Outer ring", true)] {
            let name = format!("{base} {label}");
            let feature = part(base, label);
            revolve::build(
                r,
                frame,
                &part(&feature, "Section"),
                Section::Solid(&exterior::ring(self, section, outer)),
                json!({"name":feature,"body_name":name}),
            )?;
            steel.push(name);
        }
        self.cut_keyways(r, base, frame)?;
        r.run(
            "appearance.assign",
            json!({"bodies":steel,"color":[160,160,160]}),
        )?;
        for (side, sign) in [("front", 1.0), ("back", -1.0)] {
            let sealed = self.closure == Closure::LipSeals
                || self.closure == Closure::SingleLipSeal && sign > 0.0;
            let label = format!("{} {side}", if sealed { "Seal" } else { "Cover" });
            let name = format!("{base} {label}");
            let feature = part(base, &label);
            let z = sign * (self.width / 2.0 - section.closure_recess);
            revolve::build(
                r,
                frame,
                &part(&feature, "Section"),
                Section::Surface(&[
                    [section.inner_ring_radius, z],
                    [section.outer_ring_radius, z],
                ]),
                json!({"name":feature,"body_name":name}),
            )?;
            let color = if sealed {
                [64, 64, 64]
            } else {
                [160, 160, 160]
            };
            r.run("appearance.assign", json!({"bodies":[name],"color":color}))?;
        }
        Ok(())
    }
    /// Slots extend through the full width, with independent clocking in the placement frame.
    fn cut_keyways(
        &self,
        r: &mut impl Runner,
        base: &str,
        frame: &Placement,
    ) -> Result<(), String> {
        let d = self.dimensions();
        let (inner, outer) = (self.inner_keyway, self.outer_keyway);
        for (label, key, is_outer) in [
            ("Inner keyway", inner, false),
            ("Outer keyway", outer, true),
        ] {
            let Some(key) = key else {
                continue;
            };
            let origin: [f64; 3] =
                std::array::from_fn(|i| frame.point[i] - frame.normal[i] * d.width / 2.0);
            let feature = part(base, label);
            let body = format!("{base} {} ring", if is_outer { "Outer" } else { "Inner" });
            let sketch = r.run("sketch.create", json!({"name":part(&feature,"Section"),"plane":{"origin":origin,"x_dir":frame.x_dir,"y_dir":frame.y_dir}}))?;
            let sketch = sketch_id(&sketch)?;
            let (low, high) = if is_outer {
                (d.diameter / 2.0 - key.depth, d.diameter / 2.0 + 1.0)
            } else {
                (0.0, d.bore / 2.0 + key.depth)
            };
            let w = key.width / 2.0;
            let (sin, cos) = key.angle_degrees.to_radians().sin_cos();
            let points = [[-w, low], [w, low], [w, high], [-w, high]]
                .map(|[x, y]| [x * cos - y * sin, x * sin + y * cos]);
            r.run(
                "sketch.line",
                json!({"sketch":sketch,"closed":true,"points":points}),
            )?;
            r.run("sketch.finish", json!({}))?;
            r.run("solid.extrude",json!({"name":feature,"sketch":sketch,"distance":d.width,"operation":"cut","targets":[body]}))?;
        }
        Ok(())
    }
}

/// Source-neutral design identity. Reference evidence is stored separately.
/// The part of a clutch a refused geometry is wrong in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Bore,
    Diameter,
    Width,
    InnerKeyway,
    OuterKeyway,
}

/// Why a geometry cannot be built, and the part of it to change.
#[derive(Clone, Debug, PartialEq)]
pub struct Invalid {
    pub part: Part,
    pub reason: String,
}

impl Invalid {
    fn new(part: Part, reason: impl Into<String>) -> Self {
        Self {
            part,
            reason: reason.into(),
        }
    }
}

impl From<Invalid> for String {
    fn from(invalid: Invalid) -> String {
        invalid.reason
    }
}

pub fn representation(geometry: &Geometry) -> parts::Representation<Geometry> {
    parts::Representation {
        product: parts::Product {
            catalog: "techsquid-specifications".into(),
            part: "csk".into(),
            kind: if geometry.customized() {
                parts::ProductKind::Custom
            } else {
                parts::ProductKind::DesignSpecification
            },
            configuration: geometry.clone(),
            members: vec![],
        },
        revision: "3".into(),
        detail: "exterior".into(),
        dependencies: vec![],
    }
}

pub fn ensure<R: Runner>(
    geometry: &Geometry,
    library: &mut plugin_kit::component::Library<'_, R>,
    creator: &str,
) -> Result<plugin_kit::component::Handle, String> {
    geometry.validate()?;
    let designation = geometry.designation.map_or("Custom", Csk::designation);
    let name = format!(
        "CSK clutch {designation}{}",
        if geometry.designation.is_some() && geometry.customized() {
            " custom"
        } else {
            ""
        }
    );
    library.ensure(&plugin_kit::component::Definition {
        plugin: creator, base: &name, description: designation,
        bom_type: "normal",
        site: plugin_kit::component::Site::Origin,
    }, "", &representation(geometry), |library, name, frame| {
        library.geometry(name, |mut r| {
            geometry.build(&mut r, name, frame)?;
            r.run("component.metadata.set", json!({"component":name,"values": {
                "org.techsquid.specification": {
                    "version":4, "geometry":geometry,
                    "exterior":exterior::resolve(geometry)?,
                    "geometry_identity":representation(geometry).product.identity().map_err(|e| e.to_string())?,
                    "reference": geometry.designation.map(|preset| json!({
                        "source": preset.source(),
                        "baseline": preset.geometry(),
                        "customized": geometry.customized()
                    }))
                }
            }}))?;
            Ok(())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_envelopes_are_rejected_by_the_factory() {
        for invalid in [f64::NAN, f64::INFINITY, -1.0, 0.0, 1001.0] {
            for axis in 0..3 {
                let mut g = Csk::Csk12.geometry();
                match axis {
                    0 => g.bore = invalid,
                    1 => g.diameter = invalid,
                    _ => g.width = invalid,
                }
                assert!(g.validate().is_err());
            }
        }
        let mut g = Csk::Csk12.geometry();
        g.bore = g.diameter;
        assert!(g.validate().is_err());
    }

    #[test]
    fn added_presets_have_independently_sourced_keyways() {
        for (preset, envelope, inner, outer, publisher) in [
            (
                Csk::Csk8Pp,
                [8.0, 22.0, 9.0],
                [3.0, 0.5],
                [2.0, 0.5],
                "PEAK",
            ),
            (
                Csk::Csk10Pp,
                [10.0, 30.0, 9.0],
                [3.0, 0.6],
                [2.0, 0.5],
                "Yuki",
            ),
            (
                Csk::Csk12Pp,
                [12.0, 32.0, 10.0],
                [4.0, 1.2],
                [2.0, 0.6],
                "PEAK",
            ),
            (
                Csk::Csk20Pp2Rs,
                [20.0, 47.0, 19.0],
                [6.0, 1.6],
                [3.0, 1.5],
                "PTI",
            ),
        ] {
            let g = preset.geometry();
            assert_eq!([g.bore, g.diameter, g.width], envelope);
            let i = g.inner_keyway.unwrap();
            let o = g.outer_keyway.unwrap();
            assert_eq!([i.width, i.depth], inner);
            assert_eq!([o.width, o.depth], outer);
            assert!(preset.source().title.starts_with(publisher));
            let identity = representation(&g).product.identity().unwrap();
            assert!(!identity.as_str().contains(publisher));
        }
    }
}
