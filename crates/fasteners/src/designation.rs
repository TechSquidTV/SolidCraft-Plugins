//! The size a fastener dialog sends.
//!
//! A designation is the size string `solid.thread` already accepts (`M3`, `M8x1`,
//! `1/2-10 ACME`). The plugin does not parse the thread form itself.

use plugin_kit::FieldError;
use serde_json::Value;

/// The field the size is read from: `custom_size` when it is filled, `size` otherwise.
/// An error in the size belongs on that field.
pub fn size_field(v: &Value) -> &'static str {
    match v.get("custom_size") {
        None | Some(Value::Null) => "size",
        Some(Value::String(s)) if s.trim().is_empty() => "size",
        Some(_) => "custom_size",
    }
}

/// Read the size. `custom_size` wins when it is filled, so the dialog can offer the
/// common sizes in a list and still take any other. Then `size`, then `default`.
pub fn designation(v: &Value, default: &str) -> Result<String, FieldError> {
    match v.get("custom_size") {
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
    let text = match v.get("size") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn designation_defaults_and_trims() {
        assert_eq!(designation(&json!({}), "M6").unwrap(), "M6");
        assert_eq!(
            designation(&json!({"size": " M8x1 "}), "M6").unwrap(),
            "M8x1"
        );
    }

    #[test]
    fn a_custom_size_wins_over_the_listed_one() {
        let v = json!({"size": "M4", "custom_size": " M8x1 "});
        assert_eq!(designation(&v, "M6").unwrap(), "M8x1");
        let blank = json!({"size": "M4", "custom_size": "  "});
        assert_eq!(designation(&blank, "M6").unwrap(), "M4");
        assert_eq!(
            designation(&json!({"custom_size": 8}), "M6")
                .unwrap_err()
                .field,
            "custom_size"
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
        assert_eq!(size_field(&json!({"custom_size": "M7"})), "custom_size");
    }

    #[test]
    fn designation_rejects_a_blank_or_a_number() {
        assert_eq!(
            designation(&json!({"size": "  "}), "M6").unwrap_err().field,
            "size"
        );
        assert_eq!(
            designation(&json!({"size": 6}), "M6").unwrap_err().field,
            "size"
        );
    }
}
