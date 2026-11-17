//! CSK workflow: reference defaults, optional keyway overrides and explicit BOM fields.
use bearings::csk::{self, Design};
use parts::specification::Purchasing;
use plugin_kit::{input, place, FieldError, Runner};
use serde_json::{json, Value};

pub struct Params {
    pub design: Design,
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
    let text = |field| -> Result<Option<String>, FieldError> {
        let value = input::optional_text(v, field, "must be text")?;
        if value.is_some_and(|s| s.len() > 512 || s.chars().any(char::is_control)) {
            return Err(FieldError::new(
                field,
                "use at most 512 bytes without control characters",
            ));
        }
        Ok(value.map(str::to_owned))
    };
    let design = Design {
        geometry,
        part_number: text("part_number")?,
        purchasing: Purchasing {
            manufacturer: text("purchase_manufacturer")?,
            supplier: text("purchase_supplier")?,
            supplier_part_number: text("purchase_sku")?,
            url: text("purchase_url")?,
        },
    };
    if design
        .purchasing
        .url
        .as_deref()
        .is_some_and(|s| !(s.starts_with("https://") || s.starts_with("http://")))
    {
        return Err(FieldError::new(
            "purchase_url",
            "use an http:// or https:// listing URL",
        ));
    }
    parts::catalog::validate_selection(v, crate::provider::REVISION, &csk::representation(&design))
        .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(Params {
        design,
        place: place::parse_input(v)?,
    })
}

pub fn make(r: &mut impl Runner, p: &Params) -> Result<Value, String> {
    crate::place_part(r, &p.place, json!({"specification":p.design}), |library| {
        csk::ensure(&p.design, library, crate::PLUGIN)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overrides_are_resolved_without_assigning_a_supplier() {
        let default = parse(&json!({"size":"CSK12P"})).unwrap().design;
        let customized = parse(&json!({"size":"CSK12P","inner_keyway_depth":1.2}))
            .unwrap()
            .design;
        assert_eq!(default.geometry.inner_keyway.unwrap().depth, 1.8);
        assert_eq!(customized.geometry.inner_keyway.unwrap().depth, 1.2);
        assert_eq!(customized.geometry.designation.dimensions().width, 10.0);
        assert!(customized.geometry.customized());
        assert_eq!(customized.purchasing, Purchasing::default());
        assert!(customized.part_number.is_none());
        assert_ne!(
            csk::representation(&default).identity().unwrap(),
            csk::representation(&customized).identity().unwrap()
        );
        let explicit_default =
            parse(&json!({"size":"CSK12P","inner_keyway_depth":1.8,"purchase_supplier":" "}))
                .unwrap()
                .design;
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
        ] {
            assert!(parse(&value).is_err(), "{value}");
        }
    }
    #[test]
    fn purchasing_changes_do_not_change_geometry_but_keep_bom_definitions_distinct() {
        let a = parse(&json!({"size":"CSK12P","inner_keyway_depth":1.2}))
            .unwrap()
            .design;
        let b = parse(&json!({"size":"CSK12P","inner_keyway_depth":1.2,"part_number":"CL-12","purchase_supplier":"My seller","purchase_url":"https://example.com/clutch"})).unwrap().design;
        assert_eq!(a.geometry, b.geometry);
        assert_eq!(
            a.geometry_identity().unwrap(),
            b.geometry_identity().unwrap()
        );
        assert!(a.purchasing.identity().unwrap().is_none());
        assert!(b.purchasing.identity().unwrap().is_some());
        assert_ne!(
            csk::representation(&a).identity().unwrap(),
            csk::representation(&b).identity().unwrap()
        );
    }
}
