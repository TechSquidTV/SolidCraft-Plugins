//! JSON fields the screw plugin reads. Sizes are millimetres.
//!
//! A designation is the size string `solid.thread` already accepts (`M3`, `M8x1`,
//! `1/2-10 ACME`). The plugin does not parse the thread form itself.

use serde_json::Value;

/// The longest length the plugin accepts, in millimetres.
pub const MAX_LENGTH_MM: f64 = 1000.0;

/// A parameter the host should show against one field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError {
    pub field: String,
    pub reason: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Show an error raised against `from` on `to` instead. The size parsers report
    /// `size`; a size typed in Custom size is reported there with this.
    pub fn moved(self, from: &str, to: &str) -> Self {
        if self.field == from {
            Self::new(to, self.reason)
        } else {
            self
        }
    }
}

/// The value of `key`, or of `old`, the name the field had before the plugins shared
/// one set of names. A script written for the old names keeps working.
pub fn field<'a>(v: &'a Value, key: &str, old: &str) -> Option<&'a Value> {
    v.get(key).or_else(|| v.get(old))
}

/// The field the size is read from: `custom_size` when it is filled, `size` otherwise.
/// An error in the size belongs on that field.
pub fn size_field(v: &Value) -> &'static str {
    match field(v, "custom_size", "custom_designation") {
        None | Some(Value::Null) => "size",
        Some(Value::String(s)) if s.trim().is_empty() => "size",
        Some(_) => "custom_size",
    }
}

/// Read the size. `custom_size` wins when it is filled, so the dialog can offer the
/// common sizes in a list and still take any other. Then `size`, then `default`.
pub fn designation(v: &Value, default: &str) -> Result<String, FieldError> {
    match field(v, "custom_size", "custom_designation") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) if s.trim().is_empty() => {}
        Some(Value::String(s)) => {
            let text = s.trim().to_string();
            check_designation(&text).map_err(|e| e.moved("size", "custom_size"))?;
            return Ok(text);
        }
        Some(_) => {
            return Err(FieldError::new(
                "custom_size",
                "must be a size such as \"M8x1\"",
            ));
        }
    }
    let text = match field(v, "size", "designation") {
        None => default.to_string(),
        Some(Value::String(s)) => s.trim().to_string(),
        Some(_) => {
            return Err(FieldError::new("size", "must be a string such as \"M6\""));
        }
    };
    check_designation(&text)?;
    Ok(text)
}

pub fn check_designation(s: &str) -> Result<(), FieldError> {
    if s.is_empty()
        || s.chars().count() > 64
        || s.chars()
            .any(|c| c.is_control() || (c.is_whitespace() && c != ' '))
    {
        return Err(FieldError::new(
            "size",
            "must be 1 to 64 characters, written the way SolidCraft writes the size",
        ));
    }
    Ok(())
}

/// Read a length, or `default` when the field is absent.
pub fn length_or(v: &Value, key: &str, default: f64) -> Result<f64, FieldError> {
    match v.get(key) {
        None => Ok(default),
        Some(Value::Number(n)) => n
            .as_f64()
            .ok_or_else(|| FieldError::new(key, "must be a number")),
        Some(_) => Err(FieldError::new(key, "must be a number of millimetres")),
    }
}

/// Read a length that may be left out. JSON null counts as absent.
pub fn optional_length(v: &Value, key: &str) -> Result<Option<f64>, FieldError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(Some(
            n.as_f64()
                .ok_or_else(|| FieldError::new(key, "must be a number"))?,
        )),
        Some(_) => Err(FieldError::new(key, "must be a number of millimetres")),
    }
}

/// Read a length that may be left out and may be zero, such as a fit allowance.
pub fn optional_length_or_zero(v: &Value, key: &str) -> Result<Option<f64>, FieldError> {
    match optional_length(v, key)? {
        Some(x) if x.is_finite() && x >= 0.0 => Ok(Some(x)),
        Some(_) => Err(FieldError::new(key, "must be zero or more")),
        None => Ok(None),
    }
}

pub fn check_length(key: &str, x: f64) -> Result<(), FieldError> {
    if x.is_finite() && x > 0.0 && x <= MAX_LENGTH_MM {
        Ok(())
    } else {
        Err(FieldError::new(key, "must be between 0 and 1000 mm"))
    }
}

/// A positive length typed as text, up to `MAX_LENGTH_MM`. The drive sizes use this.
pub fn millimetres(text: &str) -> Option<f64> {
    let value = text.trim().parse::<f64>().ok()?;
    (value.is_finite() && value > 0.0 && value <= MAX_LENGTH_MM).then_some(value)
}

/// Read a bool, or false when the field is absent. JSON null counts as absent.
pub fn flag(v: &Value, key: &str) -> Result<bool, FieldError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(FieldError::new(key, "must be true or false")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn designation_defaults_and_trims() {
        assert_eq!(designation(&json!({}), "M6").unwrap(), "M6");
        assert_eq!(
            designation(&json!({"designation": " M8x1 "}), "M6").unwrap(),
            "M8x1"
        );
    }

    #[test]
    fn a_custom_size_wins_over_the_listed_one() {
        let v = json!({"designation": "M4", "custom_designation": " M8x1 "});
        assert_eq!(designation(&v, "M6").unwrap(), "M8x1");
        let blank = json!({"designation": "M4", "custom_designation": "  "});
        assert_eq!(designation(&blank, "M6").unwrap(), "M4");
        assert_eq!(
            designation(&json!({"custom_designation": 8}), "M6")
                .unwrap_err()
                .field,
            "custom_size"
        );
    }

    #[test]
    fn the_shared_names_win_and_the_old_ones_still_read() {
        assert_eq!(designation(&json!({"size": "M5"}), "M6").unwrap(), "M5");
        assert_eq!(
            designation(&json!({"designation": "M4"}), "M6").unwrap(),
            "M4"
        );
        let both = json!({"size": "M5", "designation": "M4"});
        assert_eq!(designation(&both, "M6").unwrap(), "M5");
        assert_eq!(
            designation(&json!({"custom_size": "M8x1"}), "M6").unwrap(),
            "M8x1"
        );
    }

    #[test]
    fn a_bad_custom_size_is_reported_on_custom_size() {
        let v = json!({"size": "M3", "custom_size": "M3\tx"});
        assert_eq!(designation(&v, "M6").unwrap_err().field, "custom_size");
        assert_eq!(size_field(&v), "custom_size");
        assert_eq!(
            size_field(&json!({"size": "M3", "custom_size": " "})),
            "size"
        );
        assert_eq!(
            size_field(&json!({"custom_designation": "M7"})),
            "custom_size"
        );
        let err = FieldError::new("size", "x").moved("size", "custom_size");
        assert_eq!(err.field, "custom_size");
        assert_eq!(
            FieldError::new("length", "x").moved("size", "c").field,
            "length"
        );
    }

    #[test]
    fn designation_rejects_a_blank_or_a_number() {
        assert_eq!(
            designation(&json!({"designation": "  "}), "M6")
                .unwrap_err()
                .field,
            "size"
        );
        assert_eq!(
            designation(&json!({"designation": 6}), "M6")
                .unwrap_err()
                .field,
            "size"
        );
    }

    #[test]
    fn lengths_follow_the_shared_bound() {
        assert_eq!(length_or(&json!({}), "length", 20.0).unwrap(), 20.0);
        assert_eq!(
            optional_length(&json!({"depth": 12}), "depth").unwrap(),
            Some(12.0)
        );
        assert_eq!(optional_length(&json!({}), "depth").unwrap(), None);
        assert_eq!(check_length("length", 0.0).unwrap_err().field, "length");
        assert!(check_length("length", 1000.0).is_ok());
        assert!(check_length("length", 1000.1).is_err());
    }

    #[test]
    fn flag_defaults_to_false() {
        assert!(!flag(&json!({}), "modeled").unwrap());
        assert!(flag(&json!({"modeled": true}), "modeled").unwrap());
        assert_eq!(
            flag(&json!({"modeled": "yes"}), "modeled")
                .unwrap_err()
                .field,
            "modeled"
        );
    }
}
