//! CSK workflow: select a reference by designation or dimensions, or define custom geometry.
use bearings::csk::{self, Closure, Geometry, Keyway};
use bearings::EnvelopeMatch;
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
            bore: required_length(v, "custom_bore")?,
            diameter: required_length(v, "custom_diameter")?,
            width: required_length(v, "custom_width")?,
            inner_keyway: None,
            outer_keyway: None,
        }
    } else if size == "By dimensions" {
        by_dimensions(v)?.geometry()
    } else {
        csk::ALL
            .into_iter()
            .find(|c| c.designation() == size)
            .ok_or_else(|| {
                FieldError::new(
                    "size",
                    "choose a listed CSK designation, By dimensions or Custom",
                )
            })?
            .geometry()
    };
    geometry.closure = closure_choice(v, "closure", "Preset")?.unwrap_or(geometry.closure);
    for (field, dimension) in [
        ("custom_bore", &mut geometry.bore),
        ("custom_diameter", &mut geometry.diameter),
        ("custom_width", &mut geometry.width),
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
        let enabled = keyway_choice(v, &field, "Preset")?;
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
        // A tiny negative angle rounds up to exactly 360, and -0 prints as a different name.
        let angle = angle.map(|a| if a == 0.0 || a >= 360.0 { 0.0 } else { a });
        *key = if enabled.unwrap_or(key.is_some()) {
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
    geometry.validate().map_err(|e| {
        let field = match e.part {
            csk::Part::Bore => "custom_bore",
            csk::Part::Diameter => "custom_diameter",
            csk::Part::Width => "custom_width",
            csk::Part::InnerKeyway => "inner_keyway",
            csk::Part::OuterKeyway => "outer_keyway",
        };
        FieldError::new(field, e.reason)
    })?;
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

fn by_dimensions(v: &Value) -> Result<csk::Csk, FieldError> {
    let closure = closure_choice(v, "match_closure", "Any")?;
    let inner = keyway_choice(v, "match_inner_keyway", "Any")?;
    let outer = keyway_choice(v, "match_outer_keyway", "Any")?;
    let candidates = csk::ALL.into_iter().filter(|c| {
        let g = c.geometry();
        closure.is_none_or(|value| value == g.closure)
            && inner.is_none_or(|enabled| enabled == g.inner_keyway.is_some())
            && outer.is_none_or(|enabled| enabled == g.outer_keyway.is_some())
    });
    let (field, selection) = crate::match_dimensions(
        |key| {
            let Some(text) =
                input::optional_text(v, key, "pass the dimension as text in millimetres")?
            else {
                return Ok(None);
            };
            text.replace(',', ".")
                .parse::<f64>()
                .map(Some)
                .map_err(|_| FieldError::new(key, "enter a complete dimension in millimetres"))
        },
        candidates,
        |c| (c.designation(), c.dimensions()),
    )?;
    match selection {
        EnvelopeMatch::One(preset) => Ok(preset),
        EnvelopeMatch::None => Err(FieldError::new(field,
            "no CSK preset matches these dimensions, closure and keyways. Choose Custom for an unlisted clutch")),
        EnvelopeMatch::Ambiguous(reason) => Err(FieldError::new(field,
            format!("{reason}. Use Match closure, Match inner keyway or Match outer keyway to filter the listed presets"))),
    }
}

fn closure_choice(v: &Value, field: &str, default: &str) -> Result<Option<Closure>, FieldError> {
    match input::choice(v, field, default)? {
        value if value == default => Ok(None),
        "Dust covers" => Ok(Some(Closure::DustCovers)),
        "Lip seals" => Ok(Some(Closure::LipSeals)),
        "Single lip seal" => Ok(Some(Closure::SingleLipSeal)),
        _ => Err(FieldError::new(field, "choose a listed closure style")),
    }
}

fn keyway_choice(v: &Value, field: &str, default: &str) -> Result<Option<bool>, FieldError> {
    match input::choice(v, field, default)? {
        value if value == default => Ok(None),
        "Enabled" => Ok(Some(true)),
        "None" => Ok(Some(false)),
        _ => Err(FieldError::new(
            field,
            format!("choose {default}, Enabled or None"),
        )),
    }
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
            let explicit = parse(
                &json!({"size":preset.designation(), "custom_bore":geometry.bore,
                "custom_diameter":geometry.diameter, "custom_width":geometry.width}),
            )
            .unwrap()
            .geometry;
            assert_eq!(
                csk::representation(&geometry).identity(),
                csk::representation(&explicit).identity()
            );
        }
    }
    #[test]
    fn every_designation_can_be_selected_by_partial_dimensions_and_mounting() {
        for c in csk::ALL {
            let expected = c.geometry();
            let closure = match expected.closure {
                Closure::DustCovers => "Dust covers",
                Closure::LipSeals => "Lip seals",
                Closure::SingleLipSeal => "Single lip seal",
            };
            let choice = |key: Option<Keyway>| if key.is_some() { "Enabled" } else { "None" };
            let selected = parse(
                &json!({"size":"By dimensions","match_bore":expected.bore.to_string(),
                "match_closure":closure,"match_inner_keyway":choice(expected.inner_keyway),
                "match_outer_keyway":choice(expected.outer_keyway)}),
            )
            .unwrap();
            assert_eq!(selected.geometry, expected, "{}", c.designation());
        }
    }

    #[test]
    fn a_dimension_search_explains_missing_ambiguous_and_unlisted_inputs() {
        for (params, field, reasons) in [
            (json!({}), "match_bore", vec!["enter at least one"]),
            (
                json!({"match_width":"1001"}),
                "match_width",
                vec!["1000 mm"],
            ),
            (
                json!({"match_bore":"15","match_width":"11"}),
                "match_bore",
                vec!["CSK15 (", "CSK15P (", "CSK15PP (", "Match outer keyway"],
            ),
            (
                json!({"match_diameter":"47","match_outer_keyway":"Enabled"}),
                "match_diameter",
                vec!["CSK20PP (", "CSK20PP-2RS (", "width"],
            ),
            (
                json!({"match_bore":"10","match_closure":"Lip seals"}),
                "match_bore",
                vec!["no CSK preset", "Custom"],
            ),
        ] {
            let mut params = params;
            params["size"] = json!("By dimensions");
            let err = parse(&params).err().unwrap();
            assert_eq!(err.field, field);
            for reason in reasons {
                assert!(err.reason.contains(reason), "{}", err.reason);
            }
        }
    }

    #[test]
    fn switching_only_the_size_discards_all_search_filters() {
        let mut fields = json!({"size":"By dimensions", "match_bore":"15,0", "match_width":"11",
            "match_inner_keyway":"None", "match_outer_keyway":"None", "match_closure":"Dust covers"});
        assert_eq!(parse(&fields).unwrap().geometry, csk::Csk::Csk15.geometry());
        for preset in [csk::Csk::Csk15P, csk::Csk::Csk15Pp, csk::Csk::Csk152Rs] {
            fields["size"] = json!(preset.designation());
            assert_eq!(parse(&fields).unwrap().geometry, preset.geometry());
        }
    }

    #[test]
    fn explicit_geometry_edits_apply_after_a_filtered_search() {
        let fields = json!({"size":"By dimensions", "match_bore":"15", "match_width":"11",
            "match_closure":"Dust covers", "match_outer_keyway":"Enabled",
            "closure":"Lip seals", "inner_keyway":"None"});
        let mut expected = csk::Csk::Csk15Pp.geometry();
        expected.closure = Closure::LipSeals;
        expected.inner_keyway = None;
        assert_eq!(parse(&fields).unwrap().geometry, expected);
    }

    #[test]
    fn an_unfinished_dimension_only_blocks_an_active_search() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let schema = &manifest["runtime"]["entries"][1]["input_schema"];
        for field in ["match_bore", "match_diameter", "match_width"] {
            // Native forms validate numeric fields before the plugin can inspect Size.
            assert_eq!(schema["properties"][field]["type"], "string");
            let mut fields = json!({"size":"By dimensions", field:"1e"});
            assert_eq!(parse(&fields).err().unwrap().field, field);
            fields["size"] = json!("CSK15P");
            assert_eq!(
                parse(&fields).unwrap().geometry,
                csk::Csk::Csk15P.geometry()
            );
        }
    }

    #[test]
    fn envelope_and_keyway_presence_are_editable() {
        let g = parse(
            &json!({"size":"CSK8P", "custom_bore":8.1, "custom_diameter":23, "custom_width":10,
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
            &json!({"size":"Custom", "custom_bore":9, "custom_diameter":25, "custom_width":12,
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
    /// A refused geometry is shown against the field to change.
    #[test]
    fn a_refused_geometry_names_a_dialog_field() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let fields = manifest["runtime"]["entries"][1]["input_schema"]["properties"]
            .as_object()
            .unwrap();
        for (v, field) in [
            (json!({"size":"CSK8P", "custom_bore":30}), "custom_bore"),
            (
                json!({"size":"CSK8P", "inner_keyway":"Enabled", "inner_keyway_width":9,
                    "inner_keyway_depth":1}),
                "inner_keyway",
            ),
            (
                json!({"size":"CSK8P", "outer_keyway":"Enabled", "outer_keyway_width":2,
                    "outer_keyway_depth":20}),
                "outer_keyway",
            ),
        ] {
            let err = parse(&v).err().unwrap();
            assert_eq!(err.field, field, "{}", err.reason);
            assert!(fields.contains_key(field));
        }
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
            json!({"size":"Custom","custom_bore":9,"custom_diameter":25}),
            json!({"size":"CSK12P","inner_keyway_depth":0}),
            json!({"size":"CSK12P","inner_keyway_width":12}),
            json!({"size":"CSK12P","inner_keyway_depth":20}),
            json!({"size":"CSK12P","outer_keyway_depth":1}),
            json!({"size":"CSK12P","outer_keyway":"Enabled","outer_keyway_depth":1}),
            json!({"size":"CSK12P","inner_keyway":"None","inner_keyway_depth":1}),
            json!({"size":"CSK12P","inner_keyway":"invalid"}),
            json!({"size":"CSK15PP","inner_keyway_depth":8,"outer_keyway_depth":3}),
            json!({"size":"CSK12P","custom_bore":32}),
            json!({"size":"CSK12P","custom_width":-1}),
            json!({"size":"CSK12P","custom_diameter":1001}),
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
