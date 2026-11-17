pub mod provider;

use plugin_kit::{
    component::{self},
    input, place, FieldError, Runner,
};
use serde_json::{json, Value};
use wheels::Wheel;

pub const PLUGIN: &str = "techsquid-v-wheels";
pub struct Params {
    pub wheel: Wheel,
    pub assembled: bool,
    place: place::PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-v-wheel")?;
    let size = input::choice(v, "size", "Solid V Wheel")?;
    let wheel = wheels::ALL
        .into_iter()
        .find(|w| w.name() == size)
        .ok_or_else(|| FieldError::new("size", "choose a listed wheel"))?;
    let assembled = input::flag(v, "assembled", true)?;
    let representation = wheel
        .representation(assembled)
        .map_err(|e| FieldError::new("size", e))?;
    parts::catalog::validate_selection(v, provider::REVISION, &representation)
        .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(Params {
        wheel,
        assembled,
        place: place::parse_input(v)?,
    })
}

pub fn make(r: &mut impl Runner, p: &Params) -> Result<Value, String> {
    let places = place::resolve_each(r, &p.place)?;
    let (placed, report) = component::insert(
        r,
        if p.assembled { 160 } else { 40 },
        &places,
        "Wheels",
        |library| p.wheel.ensure(library, PLUGIN, p.assembled),
        |_, _| Ok(()),
    )?;
    Ok(component::with_placements(
        json!({"part":p.wheel,"assembled":p.assembled,"library":report}),
        &places,
        Some(&placed),
    ))
}

plugin_kit::guest! {
    manifest: include_str!("../plugin.json"),
    entries: [{
        entry: "make-v-wheel", noun: "wheel",
        parse: parse, make: make,
        describe: |p: &Params| p.wheel.name().to_string(),
    }],
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_catalog_rejects_unknown_dimensions() {
        assert!(parse(&json!({"diameter":24})).is_err());
        assert!(parse(&json!({"size":"Metal V Wheel"})).is_err());
    }
    #[test]
    fn manifest_identifies_the_plugin() {
        let m: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(m["id"], PLUGIN);
    }
}
