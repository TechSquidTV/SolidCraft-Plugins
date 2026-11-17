//! Formsprag CSK mounting dimensions and keyways. See docs/sources/csk.md.
use super::Dimensions;
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

/// Nominal rectangular keyway: depth is measured radially from the bore/OD apex.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Keyway {
    pub width: f64,
    pub depth: f64,
}

macro_rules! products {
    ($( $variant:ident, $label:literal, $size:ident, $mount:ident, $sealed:literal; )*) => {
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
        pub enum Csk { $( #[serde(rename = $label)] $variant, )* }
        pub const ALL: [Csk; 42] = [$(Csk::$variant,)*];
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
    Csk12, "CSK12", S12, Plain, false;
    Csk15, "CSK15", S15, Plain, false;
    Csk17, "CSK17", S17, Plain, false;
    Csk20, "CSK20", S20, Plain, false;
    Csk25, "CSK25", S25, Plain, false;
    Csk30, "CSK30", S30, Plain, false;
    Csk35, "CSK35", S35, Plain, false;
    Csk40, "CSK40", S40, Plain, false;
    Csk82Rs, "CSK8-2RS", S8, Plain, true;
    Csk122Rs, "CSK12-2RS", S12, Plain, true;
    Csk152Rs, "CSK15-2RS", S15, Plain, true;
    Csk172Rs, "CSK17-2RS", S17, Plain, true;
    Csk202Rs, "CSK20-2RS", S20, Plain, true;
    Csk252Rs, "CSK25-2RS", S25, Plain, true;
    Csk302Rs, "CSK30-2RS", S30, Plain, true;
    Csk352Rs, "CSK35-2RS", S35, Plain, true;
    Csk402Rs, "CSK40-2RS", S40, Plain, true;
    Csk12P, "CSK12P", S12, Inner, false;
    Csk15P, "CSK15P", S15, Inner, false;
    Csk17P, "CSK17P", S17, Inner, false;
    Csk20P, "CSK20P", S20, Inner, false;
    Csk25P, "CSK25P", S25, Inner, false;
    Csk30P, "CSK30P", S30, Inner, false;
    Csk35P, "CSK35P", S35, Inner, false;
    Csk40P, "CSK40P", S40, Inner, false;
    Csk15Pp, "CSK15PP", S15, Both, false;
    Csk17Pp, "CSK17PP", S17, Both, false;
    Csk20Pp, "CSK20PP", S20, Both, false;
    Csk25Pp, "CSK25PP", S25, Both, false;
    Csk30Pp, "CSK30PP", S30, Both, false;
    Csk35Pp, "CSK35PP", S35, Both, false;
    Csk40Pp, "CSK40PP", S40, Both, false;
    Csk12P2Rs, "CSK12P-2RS", S12, Inner, true;
    Csk15P2Rs, "CSK15P-2RS", S15, Inner, true;
    Csk17P2Rs, "CSK17P-2RS", S17, Inner, true;
    Csk20P2Rs, "CSK20P-2RS", S20, Inner, true;
    Csk25P2Rs, "CSK25P-2RS", S25, Inner, true;
    Csk30P2Rs, "CSK30P-2RS", S30, Inner, true;
    Csk35P2Rs, "CSK35P-2RS", S35, Inner, true;
    Csk40P2Rs, "CSK40P-2RS", S40, Inner, true;
    Csk30Pp2Rs, "CSK30PP-2RS", S30, Both, true;
}

impl Csk {
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
            Size::S8 => unreachable!("no keyed size 8 is cataloged"),
            Size::S12 => (4.0, 1.8, 0.0, 0.0),
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
            }),
            matches!(mount, Mounting::Both).then_some(Keyway {
                width: ow,
                depth: od,
            }),
        )
    }
    pub fn geometry(self) -> Geometry {
        let (inner_keyway, outer_keyway) = self.keyways();
        let d = self.dimensions();
        Geometry {
            designation: self,
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
    pub designation: Csk,
    pub bore: f64,
    pub diameter: f64,
    pub width: f64,
    pub inner_keyway: Option<Keyway>,
    pub outer_keyway: Option<Keyway>,
}

impl Geometry {
    pub fn customized(&self) -> bool {
        *self != self.designation.geometry()
    }

    pub fn dimensions(&self) -> Dimensions {
        Dimensions {
            bore: self.bore,
            diameter: self.diameter,
            width: self.width,
            section: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let reference = self.designation.dimensions();
        if [self.bore, self.diameter, self.width]
            != [reference.bore, reference.diameter, reference.width]
        {
            return Err("mounting envelope must match the selected CSK designation".into());
        }
        let d = self.dimensions();
        let (inner, outer) = self.designation.keyways();
        if self.inner_keyway.is_some() != inner.is_some()
            || self.outer_keyway.is_some() != outer.is_some()
        {
            return Err("keyways must match the selected P/PP designation".into());
        }
        for key in [self.inner_keyway, self.outer_keyway].into_iter().flatten() {
            if !key.width.is_finite()
                || !key.depth.is_finite()
                || key.width <= 0.0
                || key.depth <= 0.0
            {
                return Err("keyway dimensions must be finite and positive".into());
            }
        }
        if let Some(key) = self.inner_keyway {
            if key.width >= d.bore
                || (d.bore / 2.0 + key.depth).hypot(key.width / 2.0) >= d.diameter / 2.0
            {
                return Err(
                    "inner keyway must stay inside the outside diameter and narrower than the bore"
                        .into(),
                );
            }
        }
        if let Some(key) = self.outer_keyway {
            let floor = d.diameter / 2.0 - key.depth;
            if floor <= d.bore / 2.0 || floor.hypot(key.width / 2.0) >= d.diameter / 2.0 {
                return Err("outer keyway must leave material around the bore and intersect the outside diameter".into());
            }
            if self
                .inner_keyway
                .is_some_and(|inner| d.bore / 2.0 + inner.depth >= floor)
            {
                return Err("inner and outer keyways must not meet".into());
            }
        }
        Ok(())
    }

    pub fn build(&self, r: &mut impl Runner, base: &str, frame: &Placement) -> Result<(), String> {
        self.validate()?;
        let d = self.dimensions();
        let h = d.width / 2.0;
        let name = format!("{base} Mounting envelope");
        let feature = part(base, "Mounting envelope");
        revolve::build(
            r,
            frame,
            &part(&feature, "Section"),
            Section::Solid(&[
                [d.bore / 2.0, -h],
                [d.diameter / 2.0, -h],
                [d.diameter / 2.0, h],
                [d.bore / 2.0, h],
            ]),
            json!({"name":feature,"body_name":name}),
        )?;
        self.cut_keyways(r, base, &name, frame)?;
        r.run(
            "appearance.assign",
            json!({"bodies":[name],"color":[160,160,160]}),
        )?;
        Ok(())
    }
    /// Both slots face local +Y, extend through the full width, and rotate with the placement.
    fn cut_keyways(
        &self,
        r: &mut impl Runner,
        base: &str,
        body: &str,
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
            let sketch = r.run("sketch.create", json!({"name":part(&feature,"Section"),"plane":{"origin":origin,"x_dir":frame.x_dir,"y_dir":frame.y_dir}}))?;
            let sketch = sketch_id(&sketch)?;
            let (low, high) = if is_outer {
                (d.diameter / 2.0 - key.depth, d.diameter / 2.0 + 1.0)
            } else {
                (0.0, d.bore / 2.0 + key.depth)
            };
            let w = key.width / 2.0;
            r.run("sketch.line",json!({"sketch":sketch,"closed":true,"points":[[-w,low],[w,low],[w,high],[-w,high]]}))?;
            r.run("sketch.finish", json!({}))?;
            r.run("solid.extrude",json!({"name":feature,"sketch":sketch,"distance":d.width,"operation":"cut","targets":[body]}))?;
        }
        Ok(())
    }
}

/// Source-neutral design identity. Reference evidence is stored separately.
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
        revision: "1".into(),
        detail: "mounting-geometry".into(),
        dependencies: vec![],
    }
}

pub fn ensure<R: Runner>(
    geometry: &Geometry,
    library: &mut plugin_kit::component::Library<'_, R>,
    creator: &str,
) -> Result<plugin_kit::component::Handle, String> {
    geometry.validate()?;
    let designation = geometry.designation.designation();
    let name = format!(
        "CSK clutch {designation}{}",
        if geometry.customized() { " custom" } else { "" }
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
                    "version":2, "geometry":geometry,
                    "geometry_identity":representation(geometry).product.identity().map_err(|e| e.to_string())?,
                    "reference": {
                        "title":"Formsprag CSK catalog dimensions",
                        "sources":[
                            {"title":"Formsprag P-1352-FC 3/18", "url":"https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-1352-fc.pdf"},
                            {"title":"Formsprag MCC-P-956-FC-EN-US 11/25, CSKP/PP", "url":"https://www.formsprag.com/-/media/Project/Altramotion/shared/files/Literature/brand/formsprag-clutch/catalogs/p-956-fc-sections/mcc-p-956-fc-en-us_csk_p_csk_pp.pdf"}
                        ],
                        "baseline":geometry.designation.geometry(),
                        "customized":geometry.customized()
                    }
                }
            }}))?;
            Ok(())
        })
    })
}
