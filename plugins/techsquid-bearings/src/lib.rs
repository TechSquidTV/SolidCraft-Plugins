pub mod csk;
pub mod provider;

use bearings::{Bearing, Closure, Envelope, EnvelopeFilter, EnvelopeMatch, Size};
use plugin_kit::{
    component::{self, Library},
    input, place, FieldError, Runner,
};
use serde_json::{json, Value};

pub const PLUGIN: &str = "techsquid-bearings";
const MANIFEST: &str = include_str!("../plugin.json");

pub struct Params {
    pub bearing: Bearing,
    place: place::PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, MANIFEST, "make-bearing")?;
    let size = input::choice(v, "size", "608")?;
    let closure = closure(v, "closure")?;
    let size = Size::ALL
        .iter()
        .find(|s| s.label() == size)
        .ok_or_else(|| FieldError::new("size", "choose a listed bearing size"))?;
    let bearing = bearings::ALL
        .into_iter()
        .find(|b| b.size() == *size && b.closure() == closure)
        .ok_or_else(|| FieldError::new("closure", "Open is available only for size 626"))?;
    finish(v, bearing)
}

fn closure(v: &Value, field: &str) -> Result<Closure, FieldError> {
    let choice = input::choice(v, field, "Shielded")?;
    Closure::ALL
        .into_iter()
        .find(|c| c.label() == choice)
        .ok_or_else(|| FieldError::new(field, "choose Shielded, Sealed or Open"))
}

fn finish(v: &Value, bearing: Bearing) -> Result<Params, FieldError> {
    parts::catalog::validate_selection(v, provider::REVISION, &bearing.representation())
        .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(Params {
        bearing,
        place: place::parse_input(v)?,
    })
}

pub fn by_dimensions(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, MANIFEST, "make-bearing-by-dimensions")?;
    let closure_field = "narrow_by_closure";
    let closure = closure(v, closure_field)?;
    let candidates = bearings::ALL.into_iter().filter(|b| b.closure() == closure);
    let (field, selection) =
        match_dimensions(v, candidates, |b| (b.size().label(), b.dimensions()))?;
    let bearing = match selection {
        EnvelopeMatch::One(bearing) => bearing,
        EnvelopeMatch::None if closure == Closure::Open => {
            return Err(FieldError::new(
                closure_field,
                "Open is available only for size 626 (6 × 19 × 6 mm)",
            ));
        }
        EnvelopeMatch::None => {
            return Err(FieldError::new(
                field,
                "no bearing matches these nominal dimensions and closure",
            ));
        }
        EnvelopeMatch::Ambiguous(reason) => return Err(FieldError::new(field, reason)),
    };
    finish(v, bearing)
}

fn optional_length(v: &Value, field: &str) -> Result<Option<f64>, FieldError> {
    let value = input::optional_length(v, field)?;
    if let Some(length) = value {
        input::check_length(field, length)?;
    }
    Ok(value)
}

fn match_dimensions<T: Copy>(
    v: &Value,
    candidates: impl Iterator<Item = T>,
    describe: impl Fn(T) -> (&'static str, Envelope),
) -> Result<(&'static str, EnvelopeMatch<T>), FieldError> {
    let filter = EnvelopeFilter {
        bore: optional_length(v, "match_bore")?,
        diameter: optional_length(v, "match_diameter")?,
        width: optional_length(v, "match_width")?,
    };
    let field = if filter.bore.is_some() {
        "match_bore"
    } else if filter.diameter.is_some() {
        "match_diameter"
    } else if filter.width.is_some() {
        "match_width"
    } else {
        return Err(FieldError::new(
            "match_bore",
            "enter at least one nominal dimension in mm",
        ));
    };
    Ok((field, filter.select(candidates, describe)))
}

pub fn make(r: &mut impl Runner, p: &Params) -> Result<Value, String> {
    place_part(r, &p.place, json!({"part":p.bearing}), |library| {
        p.bearing.ensure(library, PLUGIN)
    })
}

fn place_part<R: Runner>(
    r: &mut R,
    placement: &place::PlaceInput,
    mut result: Value,
    ensure: impl FnOnce(&mut Library<'_, R>) -> Result<component::Handle, String>,
) -> Result<Value, String> {
    let places = place::resolve_each(r, placement)?;
    let (placed, report) = component::insert(r, 64, &places, "Bearings", ensure, |_, _| Ok(()))?;
    result["library"] = report;
    Ok(component::with_placements(result, &places, Some(&placed)))
}

#[cfg(target_arch = "wasm32")]
fn describe(p: &Params) -> String {
    format!("Bearing {}", p.bearing.designation())
}

plugin_kit::guest! {
    manifest: MANIFEST,
    entries: [{
        entry: "make-bearing", noun: "bearing",
        parse: parse, make: make,
        describe: describe,
    }, {
        entry: "make-csk", noun: "CSK clutch",
        parse: csk::parse, make: csk::make,
        describe: csk::describe,
    }, {
        entry: "make-bearing-by-dimensions", noun: "bearing",
        parse: by_dimensions, make: make, describe: describe,
    }, {
        entry: "make-csk-by-dimensions", noun: "CSK clutch",
        parse: csk::by_dimensions, make: csk::make, describe: csk::describe,
    }],
}

#[cfg(test)]
mod tests {
    use super::*;
    fn error(params: serde_json::Value) -> FieldError {
        parse(&params).err().expect("invalid bearing selection")
    }

    #[test]
    fn defaults_and_retired_inputs() {
        for input in [json!({}), json!({"size":"","closure":""})] {
            assert_eq!(parse(&input).unwrap().bearing, Bearing::MisumiB608zz);
        }
        for old in [
            "625 2RS",
            "105ZZ",
            "B626",
            "B6203ZZ",
            "5 x 10 x 4.2",
            "By dimensions",
        ] {
            assert_eq!(error(json!({"size":old})).field, "size");
        }
        assert_eq!(error(json!({"size":"625","closure":"ZZ"})).field, "closure");
        assert_eq!(
            error(json!({"size":"608","match_width":4})).field,
            "match_width"
        );
        assert_eq!(
            search_error(json!({"closure":"Open","match_bore":6})).field,
            "closure"
        );
        assert_eq!(
            error(json!({"size":"608","closure":"Open"})).field,
            "closure"
        );
    }

    #[test]
    fn all_selections_resolve_by_size_and_by_dimensions() {
        for b in bearings::ALL {
            let d = b.dimensions();
            let named =
                parse(&json!({"size":b.size().label(),"closure":b.closure().label()})).unwrap();
            let matched = by_dimensions(&json!({"narrow_by_closure":b.closure().label(),
                "match_bore":d.bore,"match_diameter":d.diameter,"match_width":d.width}))
            .unwrap();
            assert_eq!(named.bearing, b);
            assert_eq!(matched.bearing, named.bearing);
        }
    }

    fn search_error(params: Value) -> FieldError {
        by_dimensions(&params)
            .err()
            .expect("invalid dimension search")
    }

    #[test]
    fn partial_dimensions_filter_the_selected_closure_and_explain_ambiguity() {
        assert_eq!(
            by_dimensions(&json!({"match_width":7})).unwrap().bearing,
            Bearing::MisumiB608zz
        );
        assert_eq!(
            by_dimensions(&json!({"match_bore":6,"narrow_by_closure":"Open"}))
                .unwrap()
                .bearing,
            Bearing::MisumiB626
        );
        let ambiguous = search_error(json!({"match_bore":17}));
        assert_eq!(ambiguous.field, "match_bore");
        for hint in [
            "6003 (17 × 35 × 10 mm)",
            "6203 (17 × 40 × 12 mm)",
            "outside diameter",
        ] {
            assert!(ambiguous.reason.contains(hint));
        }
        assert_eq!(
            search_error(json!({"match_bore":17,"narrow_by_closure":"Open"})).field,
            "narrow_by_closure"
        );
        assert!(search_error(json!({"match_bore":17,"match_width":14}))
            .reason
            .contains("no bearing"));
        for value in [json!(-1), json!(0), json!("17"), json!(1001)] {
            assert_eq!(
                search_error(json!({"match_width":value})).field,
                "match_width"
            );
        }
        assert_eq!(
            search_error(json!({"narrow_by_closure":"Open"})).field,
            "match_bore"
        );
    }

    #[test]
    fn manifest_identifies_the_plugin() {
        let m: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(m["id"], PLUGIN);
    }
}
