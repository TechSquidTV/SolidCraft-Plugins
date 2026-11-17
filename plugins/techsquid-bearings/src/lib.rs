pub mod provider;

use bearings::Bearing;
use plugin_kit::{
    component::{self, Library},
    input, place, FieldError, Runner,
};
use serde_json::{json, Value};

pub const PLUGIN: &str = "techsquid-bearings";
pub struct Params {
    pub bearing: Bearing,
    place: place::PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-bearing")?;
    let size = input::choice(v, "size", "625 2RS")?;
    let bearing = bearings::ALL
        .into_iter()
        .find(|b| b.designation() == size)
        .ok_or_else(|| FieldError::new("size", "choose a listed bearing"))?;
    parts::catalog::validate_selection(v, provider::REVISION, &bearing.representation())
        .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(Params {
        bearing,
        place: place::parse_input(v)?,
    })
}

pub fn make(r: &mut impl Runner, p: &Params) -> Result<Value, String> {
    let places = place::resolve_each(r, &p.place)?;
    if places.len() > 32 {
        return Err("sketch_point: choose at most 32 points".into());
    }
    let mut library = Library::new(
        r,
        component::Budget {
            creation_calls: 64,
            placements: places.len(),
        },
    )?;
    let handle = p.bearing.ensure(&mut library, PLUGIN)?;
    let placed = library.place_all(&handle, &places, "Bearings")?;
    let report = library.finish()?;
    Ok(component::with_placements(
        json!({"part":p.bearing, "library":report}),
        &places,
        Some(&placed),
    ))
}

plugin_kit::guest! {
    manifest: include_str!("../plugin.json"),
    entries: [{
        entry: "make-bearing", noun: "bearing",
        parse: parse, make: make,
        describe: |p: &Params| format!("Bearing {}", p.bearing.designation()),
    }],
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_catalog_rejects_unknown_dimensions() {
        assert!(parse(&json!({"size":"625 2RS", "diameter":17})).is_err());
        assert!(parse(&json!({"size":"5 x 10 x 4.2"})).is_err());
    }
    #[test]
    fn manifest_identifies_the_plugin() {
        let m: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(m["id"], PLUGIN);
    }
}
