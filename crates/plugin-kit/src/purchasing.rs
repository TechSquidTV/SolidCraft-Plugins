//! Shared workflow parsing for explicit supplier assignments.
use crate::{input, FieldError};
use parts::purchasing::{Assignment, Details, Identifier, Purchase};
use serde_json::Value;

pub fn parse(v: &Value) -> Result<Details, FieldError> {
    let text = |field| -> Result<Option<String>, FieldError> {
        let value = input::optional_text(v, field, "must be text")?;
        if value.is_some_and(|s| s.len() > 128 || s.chars().any(char::is_control)) {
            return Err(FieldError::new(
                field,
                "use at most 128 bytes without control characters",
            ));
        }
        Ok(value.map(str::to_owned))
    };
    let pair = |authority_field, number_field| -> Result<Option<Identifier>, FieldError> {
        match (text(authority_field)?, text(number_field)?) {
            (None, None) => Ok(None),
            (Some(authority), Some(part_number)) => Ok(Some(Identifier {
                authority,
                part_number,
            })),
            _ => Err(FieldError::new(
                authority_field,
                format!("supply both {authority_field} and {number_field}"),
            )),
        }
    };
    let supplier = pair("purchase_supplier", "purchase_sku")?;
    let manufacturer = pair("purchase_manufacturer", "purchase_mpn")?;
    if manufacturer.is_some() && supplier.is_none() {
        return Err(FieldError::new(
            "purchase_supplier",
            "supply a supplier and SKU for this purchasing assignment",
        ));
    }
    let details = Details {
        assignment: Assignment {
            part_number: text("part_number")?,
            purchase: supplier.map(|supplier| Purchase {
                supplier,
                manufacturer,
            }),
        },
        url: input::optional_text(v, "purchase_url", "must be text")?.map(str::to_owned),
    };
    details
        .validate()
        .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(details)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn only_complete_product_identifiers_are_accepted() {
        assert_eq!(
            parse(&json!({"purchase_supplier":" "})).unwrap(),
            Details::default()
        );
        for v in [
            json!({"purchase_supplier":"Seller"}),
            json!({"purchase_sku":"123"}),
            json!({"purchase_manufacturer":"Brand"}),
            json!({"purchase_url":"https://example.com"}),
            json!({"purchase_supplier":"Seller","purchase_sku":"123","purchase_url":"https://"}),
        ] {
            assert!(parse(&v).is_err(), "{v}");
        }
        assert!(parse(&json!({"purchase_supplier":"Seller","purchase_sku":"123",
            "purchase_manufacturer":"Brand","purchase_mpn":"CSK12P","purchase_url":"https://example.com/123"})).is_ok());
    }
}
