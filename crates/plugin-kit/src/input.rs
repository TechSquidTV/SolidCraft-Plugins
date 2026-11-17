//! JSON fields a plugin dialog sends. Lengths are millimetres.

use serde_json::Value;

/// The longest length the plugin accepts, in millimetres.
pub const MAX_LENGTH_MM: f64 = 1000.0;

/// Fixed catalog workflows reject dimension overrides and misspelled inputs.
pub fn only_fields(v: &Value, allowed: &[&str]) -> Result<(), FieldError> {
    let fields = v
        .as_object()
        .ok_or_else(|| FieldError::new("params", "requires an object"))?;
    for key in fields.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(FieldError::new(key, "unsupported field"));
        }
    }
    Ok(())
}

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

/// A dropdown's choice: the trimmed text of `key`, or `default` when the field is absent,
/// null or blank. The caller checks the text is one of its choices.
pub fn choice<'a>(v: &'a Value, key: &str, default: &'a str) -> Result<&'a str, FieldError> {
    Ok(optional_text(v, key, "must be a choice from the list")?.unwrap_or(default))
}

/// A text field that may be left empty: its trimmed text, or `None` when it is absent,
/// null or blank. `wrong` is the error for a value that is not text.
pub fn optional_text<'a>(
    v: &'a Value,
    key: &str,
    wrong: &str,
) -> Result<Option<&'a str>, FieldError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if text.trim().is_empty() => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.trim())),
        Some(_) => Err(FieldError::new(key, wrong)),
    }
}

/// Validate workflow fields against the packaged form, plus the typed catalog
/// precondition. Retired keys are errors, never silently ignored defaults.
pub fn workflow_fields(v: &Value, manifest: &str, entry: &str) -> Result<(), FieldError> {
    let manifest: Value =
        serde_json::from_str(manifest).map_err(|e| FieldError::new("plugin", e.to_string()))?;
    let entries = manifest["runtime"]["entries"]
        .as_array()
        .ok_or_else(|| FieldError::new("plugin", "manifest has no entries"))?;
    let properties = entries
        .iter()
        .find(|e| e["name"] == entry)
        .and_then(|e| e["input_schema"]["properties"].as_object())
        .ok_or_else(|| FieldError::new("plugin", "workflow has no field declaration"))?;
    let mut allowed: Vec<_> = properties.keys().map(String::as_str).collect();
    allowed.push("catalog_selection");
    only_fields(v, &allowed)
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

/// Read a bool, or `default` when the field is absent. JSON null counts as absent.
pub fn flag(v: &Value, key: &str, default: bool) -> Result<bool, FieldError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(FieldError::new(key, "must be true or false")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_moved_error_changes_only_its_own_field() {
        let err = FieldError::new("size", "x").moved("size", "custom_size");
        assert_eq!(err.field, "custom_size");
        assert_eq!(
            FieldError::new("length", "x").moved("size", "c").field,
            "length"
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
    fn a_choice_is_trimmed_and_a_blank_one_is_the_default() {
        assert_eq!(
            choice(&json!({"k": " Blind "}), "k", "Through"),
            Ok("Blind")
        );
        for v in [json!({}), json!({"k": null}), json!({"k": "  "})] {
            assert_eq!(choice(&v, "k", "Through"), Ok("Through"));
        }
        let err = choice(&json!({"k": 3}), "k", "Through").unwrap_err();
        assert_eq!(err, FieldError::new("k", "must be a choice from the list"));
        assert_eq!(
            optional_text(&json!({"k": " T10 "}), "k", "x"),
            Ok(Some("T10"))
        );
        assert_eq!(optional_text(&json!({"k": ""}), "k", "x"), Ok(None));
        assert_eq!(
            optional_text(&json!({"k": []}), "k", "must be a socket"),
            Err(FieldError::new("k", "must be a socket"))
        );
    }

    #[test]
    fn flag_takes_its_default_when_absent() {
        assert!(flag(&json!({}), "modeled", true).unwrap());
        assert!(!flag(&json!({}), "modeled", false).unwrap());
        assert!(flag(&json!({"modeled": null}), "modeled", true).unwrap());
        assert!(!flag(&json!({"modeled": false}), "modeled", true).unwrap());
        assert!(flag(&json!({"modeled": true}), "modeled", false).unwrap());
        assert_eq!(
            flag(&json!({"modeled": "yes"}), "modeled", true)
                .unwrap_err()
                .field,
            "modeled"
        );
    }
}
