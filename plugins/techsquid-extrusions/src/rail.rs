//! What a person sets for a rail. Sizes are millimetres.

use extrusions::profile::{Grid, SIZES};
use extrusions::slot::{Slot, T5, V5};
use plugin_kit::input::FieldError;
use plugin_kit::place::{self, PlaceInput};
use serde_json::Value;

pub const DEFAULT_SLOT: &str = "T-slot";
pub const DEFAULT_SIZE: &str = "2020";
pub const DEFAULT_LENGTH: f64 = 500.0;

/// The longest rail the dialog takes. Stock lengths end at 4 m (MISUMI) and 6 m.
pub const MAX_LENGTH_MM: f64 = 6000.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub slot: Slot,
    /// The size as listed, such as `2040`.
    pub size: String,
    pub grid: Grid,
    pub length: f64,
    pub place: PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    let slot = match text(v, "slot", DEFAULT_SLOT)? {
        "T-slot" => T5,
        "V-slot" => V5,
        other => {
            return Err(FieldError::new(
                "slot",
                format!("must be T-slot or V-slot, not {other}"),
            ))
        }
    };
    let size = text(v, "size", DEFAULT_SIZE)?;
    let grid = Grid::of(size).ok_or_else(|| {
        let listed: Vec<&str> = SIZES.iter().map(|s| s.0).collect();
        FieldError::new(
            "size",
            format!("must be one of {}, not {size}", listed.join(", ")),
        )
    })?;
    if slot == V5 && grid.nx > 1 {
        // OpenBuilds publishes sections for the single-row rails this was checked against.
        return Err(FieldError::new(
            "size",
            format!("a V-slot comes as 2020, 2040, 2060 or 2080 here; pick one of those, or T-slot for a {size}"),
        ));
    }
    let length = match v.get("length") {
        None | Some(Value::Null) => DEFAULT_LENGTH,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(_) => return Err(FieldError::new("length", "must be a number of millimetres")),
    };
    // A name shows a length to 0.01 mm, so two rails with one name are one rail.
    let length = (length * 100.0).round() / 100.0;
    if !(length.is_finite() && length > 0.0 && length <= MAX_LENGTH_MM) {
        return Err(FieldError::new(
            "length",
            format!("must be more than 0 and at most {MAX_LENGTH_MM} mm"),
        ));
    }
    Ok(Params {
        slot,
        size: size.to_string(),
        grid,
        length,
        place: place::parse_input(v)?,
    })
}

/// A choice from a list, or `default` when the field is absent.
pub(crate) fn text<'a>(v: &'a Value, key: &str, default: &'a str) -> Result<&'a str, FieldError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::String(s)) if s.trim().is_empty() => Ok(default),
        Some(Value::String(s)) => Ok(s.trim()),
        Some(_) => Err(FieldError::new(key, "must be one of the listed choices")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_empty_form_is_a_half_metre_2020_t_slot() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.slot, T5);
        assert_eq!(p.size, "2020");
        assert_eq!(p.grid, Grid { nx: 1, ny: 1 });
        assert_eq!(p.length, 500.0);
    }

    #[test]
    fn an_empty_choice_is_the_default() {
        let p = parse(&json!({"slot": "", "size": " "})).unwrap();
        assert_eq!(p.slot, T5);
        assert_eq!(p.size, "2020");
    }

    #[test]
    fn a_v_slot_2080_is_one_cell_wide_and_four_tall() {
        let p = parse(&json!({"slot": "V-slot", "size": "2080", "length": 1000})).unwrap();
        assert_eq!(p.slot, V5);
        assert_eq!(p.grid, Grid { nx: 1, ny: 4 });
        assert_eq!(p.length, 1000.0);
    }

    #[test]
    fn a_v_slot_4040_is_refused_on_the_size_field() {
        let err = parse(&json!({"slot": "V-slot", "size": "4040"})).unwrap_err();
        assert_eq!(err.field, "size");
        assert!(err.reason.contains("T-slot"), "{}", err.reason);
        assert!(parse(&json!({"slot": "T-slot", "size": "4040"})).is_ok());
    }

    #[test]
    fn an_unknown_choice_names_its_field() {
        assert_eq!(parse(&json!({"slot": "U-slot"})).unwrap_err().field, "slot");
        assert_eq!(parse(&json!({"size": "3030"})).unwrap_err().field, "size");
        assert_eq!(parse(&json!({"size": 2020})).unwrap_err().field, "size");
    }

    #[test]
    fn a_length_must_be_above_zero_and_no_more_than_six_metres() {
        for length in [json!(0), json!(-5), json!(6000.5), json!("long")] {
            let err = parse(&json!({"length": length})).unwrap_err();
            assert_eq!(err.field, "length");
        }
        assert_eq!(parse(&json!({"length": 6000})).unwrap().length, 6000.0);
        assert_eq!(parse(&json!({"length": 0.5})).unwrap().length, 0.5);
    }

    #[test]
    fn a_length_is_kept_to_a_hundredth_as_its_name_shows_it() {
        assert_eq!(parse(&json!({"length": 100.004})).unwrap().length, 100.0);
        assert_eq!(parse(&json!({"length": 122.456})).unwrap().length, 122.46);
        // Too short to show is too short to cut.
        let err = parse(&json!({"length": 0.004})).unwrap_err();
        assert_eq!(err.field, "length");
    }

    #[test]
    fn a_named_point_needs_its_sketch() {
        let err = parse(&json!({"sketch_point": "c1"})).unwrap_err();
        assert_eq!(err.field, "sketch");
    }
}
