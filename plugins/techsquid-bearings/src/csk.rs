//! CSK workflow: reference defaults and optional keyway overrides.
use bearings::csk::{self, Geometry};
use plugin_kit::{input, place, FieldError, Runner};
use serde_json::{json, Value};

pub struct Params {
    pub geometry: Geometry,
    place: place::PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-csk")?;
    let size = input::choice(v, "size", "CSK12P")?;
    let template = csk::ALL
        .into_iter()
        .find(|c| c.designation() == size)
        .ok_or_else(|| FieldError::new("size", "choose a listed CSK designation"))?;
    let mut geometry = template.geometry();
    for (prefix, key) in [
        ("inner", &mut geometry.inner_keyway),
        ("outer", &mut geometry.outer_keyway),
    ] {
        for dimension in ["width", "depth"] {
            let field = format!("{prefix}_keyway_{dimension}");
            if let Some(value) = input::optional_length(v, &field)? {
                input::check_length(&field, value)?;
                let key = key.as_mut().ok_or_else(|| {
                    FieldError::new(
                        &field,
                        "selected designation has no such keyway; choose P or PP",
                    )
                })?;
                match dimension {
                    "width" => key.width = value,
                    _ => key.depth = value,
                }
            }
        }
    }
    geometry
        .validate()
        .map_err(|e| FieldError::new("keyway", e))?;
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
    fn overrides_are_resolved_without_assigning_a_supplier() {
        let default = parse(&json!({"size":"CSK12P"})).unwrap().geometry;
        let customized = parse(&json!({"size":"CSK12P","inner_keyway_depth":1.2}))
            .unwrap()
            .geometry;
        assert_eq!(default.inner_keyway.unwrap().depth, 1.8);
        assert_eq!(customized.inner_keyway.unwrap().depth, 1.2);
        assert_eq!(customized.designation.dimensions().width, 10.0);
        assert!(customized.customized());
        assert_ne!(
            csk::representation(&default).identity().unwrap(),
            csk::representation(&customized).identity().unwrap()
        );
        let explicit_default = parse(&json!({"size":"CSK12P","inner_keyway_depth":1.8}))
            .unwrap()
            .geometry;
        assert_eq!(default, explicit_default);
    }
    #[test]
    fn invalid_overrides_are_rejected_before_host_edits() {
        for value in [
            json!({"size":"CSK12P","inner_keyway_depth":0}),
            json!({"size":"CSK12P","inner_keyway_width":12}),
            json!({"size":"CSK12P","inner_keyway_depth":20}),
            json!({"size":"CSK12P","outer_keyway_depth":1}),
            json!({"size":"CSK12","inner_keyway_depth":1}),
            json!({"size":"CSK15PP","inner_keyway_depth":8,"outer_keyway_depth":3}),
            json!({"size":"CSK12P","bore":13}),
            json!({"size":"CSK12P","part_number":"CL-12"}),
            json!({"size":"CSK12P","purchase_supplier":"Seller"}),
        ] {
            assert!(parse(&value).is_err(), "{value}");
        }
    }
}
