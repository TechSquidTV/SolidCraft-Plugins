//! CSK workflow: editable reference presets or fully custom mounting geometry.
use bearings::csk::{self, Closure, Geometry, Keyway};
use plugin_kit::{input, place, FieldError, Runner};
use serde_json::{json, Value};

pub struct Params {
    pub geometry: Geometry,
    place: place::PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-csk")?;
    let size = input::choice(v, "size", "CSK12P")?;
    let mut geometry = if size == "Custom" {
        Geometry {
            designation: None,
            closure: Closure::DustCovers,
            bore: required_length(v, "bore")?,
            diameter: required_length(v, "diameter")?,
            width: required_length(v, "width")?,
            inner_keyway: None,
            outer_keyway: None,
        }
    } else {
        csk::ALL
            .into_iter()
            .find(|c| c.designation() == size)
            .ok_or_else(|| FieldError::new("size", "choose a listed CSK designation or Custom"))?
            .geometry()
    };
    geometry.closure = match input::choice(v, "closure", "Preset")? {
        "Preset" => geometry.closure,
        "Dust covers" => Closure::DustCovers,
        "Lip seals" => Closure::LipSeals,
        "Single lip seal" => Closure::SingleLipSeal,
        _ => return Err(FieldError::new("closure", "choose a listed closure style")),
    };
    for (field, dimension) in [
        ("bore", &mut geometry.bore),
        ("diameter", &mut geometry.diameter),
        ("width", &mut geometry.width),
    ] {
        if let Some(value) = input::optional_length(v, field)? {
            input::check_length(field, value)?;
            *dimension = value;
        }
    }
    for (prefix, key) in [
        ("inner", &mut geometry.inner_keyway),
        ("outer", &mut geometry.outer_keyway),
    ] {
        let field = format!("{prefix}_keyway");
        let width_field = format!("{field}_width");
        let depth_field = format!("{field}_depth");
        let width = input::optional_length(v, &width_field)?;
        let depth = input::optional_length(v, &depth_field)?;
        let angle_field = format!("{field}_angle");
        let angle = match v.get(&angle_field) {
            None | Some(Value::Null) => None,
            Some(value) => Some(
                value
                    .as_f64()
                    .filter(|a| a.is_finite())
                    .ok_or_else(|| {
                        FieldError::new(&angle_field, "must be a finite number of degrees")
                    })?
                    .rem_euclid(360.0),
            ),
        };
        let angle = angle.map(|a| if a == 0.0 { 0.0 } else { a });
        let enabled = match input::choice(v, &field, "Preset")? {
            "Preset" => key.is_some(),
            "Enabled" => true,
            "None" => false,
            _ => return Err(FieldError::new(&field, "choose Preset, Enabled or None")),
        };
        *key = if enabled {
            Some(Keyway {
                angle_degrees: angle
                    .or(key.map(|k| k.angle_degrees))
                    .unwrap_or(if prefix == "inner" { 0.0 } else { 180.0 }),
                width: width.or(key.map(|k| k.width)).ok_or_else(|| {
                    FieldError::new(&width_field, "enter a width for the added keyway")
                })?,
                depth: depth.or(key.map(|k| k.depth)).ok_or_else(|| {
                    FieldError::new(&depth_field, "enter a depth for the added keyway")
                })?,
            })
        } else {
            if width.is_some() || depth.is_some() || angle.is_some() {
                return Err(FieldError::new(
                    &field,
                    "enable this keyway to specify its dimensions or angle",
                ));
            }
            None
        };
    }
    geometry
        .validate()
        .map_err(|e| FieldError::new("geometry", e))?;
    parts::catalog::validate_selection(
        v,
        crate::provider::REVISION,
        &csk::representation(&geometry),
    )
    .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(Params {
        geometry,
        place: place::parse_input(v)?,
    })
}

fn required_length(v: &Value, field: &str) -> Result<f64, FieldError> {
    let value = input::optional_length(v, field)?
        .ok_or_else(|| FieldError::new(field, "required for Custom"))?;
    input::check_length(field, value)?;
    Ok(value)
}

pub fn make(r: &mut impl Runner, p: &Params) -> Result<Value, String> {
    crate::place_part(
        r,
        &p.place,
        json!({"specification":p.geometry}),
        |library| csk::ensure(&p.geometry, library, crate::PLUGIN),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_preset_is_valid_and_explicit_defaults_reuse() {
        for preset in csk::ALL {
            let geometry = parse(&json!({"size":preset.designation()}))
                .unwrap()
                .geometry;
            assert_eq!(geometry, preset.geometry());
            assert!(!geometry.customized());
            let explicit = parse(&json!({"size":preset.designation(), "bore":geometry.bore,
                "diameter":geometry.diameter, "width":geometry.width}))
            .unwrap()
            .geometry;
            assert_eq!(
                csk::representation(&geometry).identity(),
                csk::representation(&explicit).identity()
            );
        }
    }
    #[test]
    fn envelope_and_keyway_presence_are_editable() {
        let g = parse(
            &json!({"size":"CSK8P", "bore":8.1, "diameter":23, "width":10,
            "inner_keyway":"None", "outer_keyway":"Enabled", "outer_keyway_width":2,
            "outer_keyway_depth":0.5}),
        )
        .unwrap()
        .geometry;
        assert_eq!([g.bore, g.diameter, g.width], [8.1, 23.0, 10.0]);
        assert!(g.inner_keyway.is_none());
        assert_eq!(
            g.outer_keyway.unwrap(),
            Keyway {
                width: 2.0,
                depth: 0.5,
                angle_degrees: 180.0,
            }
        );
        assert!(g.customized());
        assert_ne!(
            csk::representation(&g).identity(),
            csk::representation(&csk::Csk::Csk8P.geometry()).identity()
        );
    }
    #[test]
    fn custom_requires_dimensions_and_has_no_reference_designation() {
        let g = parse(
            &json!({"size":"Custom", "bore":9, "diameter":25, "width":12,
            "inner_keyway":"Enabled", "inner_keyway_width":3, "inner_keyway_depth":0.7}),
        )
        .unwrap()
        .geometry;
        assert!(g.designation.is_none());
        assert!(g.customized());
        assert_eq!(g.inner_keyway.unwrap().depth, 0.7);
        assert!(g.outer_keyway.is_none());
        let restored: Geometry = serde_json::from_value(serde_json::to_value(&g).unwrap()).unwrap();
        assert_eq!(g, restored);
    }
    #[test]
    fn closure_overrides_are_custom_geometry_and_do_not_change_width() {
        let g = parse(&json!({"size":"CSK15P", "closure":"Lip seals"}))
            .unwrap()
            .geometry;
        assert_eq!(g.closure, Closure::LipSeals);
        assert_eq!(g.width, 11.0);
        assert!(g.customized());
        assert_ne!(
            csk::representation(&g).identity(),
            csk::representation(&csk::Csk::Csk15P.geometry()).identity()
        );
    }
    #[test]
    fn keyway_angles_normalize_for_reuse() {
        let preset = parse(&json!({"size":"CSK15PP"})).unwrap().geometry;
        let wrapped =
            parse(&json!({"size":"CSK15PP","inner_keyway_angle":-360,"outer_keyway_angle":-180}))
                .unwrap()
                .geometry;
        assert_eq!(preset, wrapped);
        let aligned = parse(&json!({"size":"CSK15PP","outer_keyway_angle":0}))
            .unwrap()
            .geometry;
        assert!(aligned.customized());
        assert_ne!(
            csk::representation(&preset).identity(),
            csk::representation(&aligned).identity()
        );
        assert!(parse(&json!({"size":"CSK15","inner_keyway_angle":90})).is_err());
    }
    #[test]
    fn invalid_geometry_and_purchasing_fields_are_rejected_before_host_edits() {
        for value in [
            json!({"size":"Custom"}),
            json!({"size":"Custom","bore":9,"diameter":25}),
            json!({"size":"CSK12P","inner_keyway_depth":0}),
            json!({"size":"CSK12P","inner_keyway_width":12}),
            json!({"size":"CSK12P","inner_keyway_depth":20}),
            json!({"size":"CSK12P","outer_keyway_depth":1}),
            json!({"size":"CSK12P","outer_keyway":"Enabled","outer_keyway_depth":1}),
            json!({"size":"CSK12P","inner_keyway":"None","inner_keyway_depth":1}),
            json!({"size":"CSK12P","inner_keyway":"invalid"}),
            json!({"size":"CSK15PP","inner_keyway_depth":8,"outer_keyway_depth":3}),
            json!({"size":"CSK12P","bore":32}),
            json!({"size":"CSK12P","width":-1}),
            json!({"size":"CSK12P","diameter":1001}),
            json!({"size":"CSK12P","part_number":"CL-12"}),
            json!({"size":"CSK12P","purchase_supplier":"Seller"}),
            json!({"size":"CSK50"}),
            json!({"size":"CSK12P","closure":"invalid"}),
            json!({"size":"CSK12PP","inner_keyway_depth":7,"outer_keyway_depth":2.5}),
        ] {
            assert!(parse(&value).is_err(), "{value}");
        }
    }
}
