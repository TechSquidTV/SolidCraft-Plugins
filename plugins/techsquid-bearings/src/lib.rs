pub mod csk;
pub mod provider;

use bearings::{Bearing, Closure, Envelope, EnvelopeFilter, EnvelopeMatch, Size};
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
    let size = input::choice(v, "size", "608")?;
    let closure_choice = input::choice(v, "closure", "Shielded")?;
    let closure = Closure::ALL
        .into_iter()
        .find(|c| c.label() == closure_choice)
        .ok_or_else(|| FieldError::new("closure", "choose Shielded, Sealed or Open"))?;
    let bearing = if size == "By dimensions" {
        by_dimensions(v, closure)?
    } else {
        let size = Size::ALL
            .iter()
            .find(|s| s.label() == size)
            .ok_or_else(|| FieldError::new("size", "choose a base size or By dimensions"))?;
        bearings::ALL
            .into_iter()
            .find(|b| b.size() == *size && b.closure() == closure)
            .ok_or_else(|| FieldError::new("closure", "Open is available only for size 626"))?
    };
    parts::catalog::validate_selection(v, provider::REVISION, &bearing.representation())
        .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(Params {
        bearing,
        place: place::parse_input(v)?,
    })
}

fn by_dimensions(v: &Value, closure: Closure) -> Result<Bearing, FieldError> {
    let candidates = bearings::ALL.into_iter().filter(|b| b.closure() == closure);
    let (field, selection) = match_dimensions(
        |key| input::optional_length(v, key),
        candidates,
        |b| (b.size().label(), b.dimensions()),
    )?;
    match selection {
        EnvelopeMatch::One(bearing) => Ok(bearing),
        EnvelopeMatch::None if closure == Closure::Open => Err(FieldError::new(
            "closure",
            "Open is available only for size 626 (6 × 19 × 6 mm)",
        )),
        EnvelopeMatch::None => Err(FieldError::new(
            field,
            "no bearing matches these nominal dimensions and closure",
        )),
        EnvelopeMatch::Ambiguous(reason) => Err(FieldError::new(field, reason)),
    }
}

/// Read the three search fields only when the user selects By dimensions.
fn match_dimensions<T: Copy>(
    read_length: impl Fn(&str) -> Result<Option<f64>, FieldError>,
    candidates: impl Iterator<Item = T>,
    describe: impl Fn(T) -> (&'static str, Envelope),
) -> Result<(&'static str, EnvelopeMatch<T>), FieldError> {
    let length = |key| {
        let value = read_length(key)?;
        if let Some(x) = value {
            input::check_length(key, x)?;
        }
        Ok(value)
    };
    let filter = EnvelopeFilter {
        bore: length("match_bore")?,
        diameter: length("match_diameter")?,
        width: length("match_width")?,
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

plugin_kit::guest! {
    manifest: include_str!("../plugin.json"),
    entries: [{
        entry: "make-bearing", noun: "bearing",
        parse: parse, make: make,
        describe: |p: &Params| format!("Bearing {}", p.bearing.designation()),
    }, {
        entry: "make-csk", noun: "CSK clutch",
        parse: csk::parse, make: csk::make,
        describe: |p: &csk::Params| format!("CSK clutch {}", p.geometry.designation.map_or("Custom", bearings::csk::Csk::designation)),
    }],
}

#[cfg(test)]
mod tests {
    use super::*;
    fn error(params: serde_json::Value) -> FieldError {
        parse(&params).err().expect("invalid bearing selection")
    }

    #[test]
    fn defaults_and_legacy_values() {
        for input in [
            json!({}),
            json!({"size":"","closure":""}),
            json!({"size":"608","match_width":4}),
        ] {
            assert_eq!(parse(&input).unwrap().bearing, Bearing::MisumiB608zz);
        }
        for old in ["625 2RS", "105ZZ", "B626", "B6203ZZ", "5 x 10 x 4.2"] {
            assert_eq!(error(json!({"size":old})).field, "size");
        }
        assert_eq!(error(json!({"size":"625","closure":"ZZ"})).field, "closure");
        assert_eq!(
            error(json!({"size":"608","closure":"Open"})).field,
            "closure"
        );
    }

    #[test]
    fn all_selections_resolve_by_size_and_by_dimensions() {
        for b in bearings::ALL {
            let d = b.dimensions();
            for params in [
                json!({"size":b.size().label(),"closure":b.closure().label()}),
                json!({"size":"By dimensions","closure":b.closure().label(),
                    "match_bore":d.bore,"match_diameter":d.diameter,"match_width":d.width}),
            ] {
                assert_eq!(parse(&params).unwrap().bearing, b);
            }
        }
    }

    #[test]
    fn partial_dimensions_filter_the_selected_closure_and_explain_ambiguity() {
        assert_eq!(
            parse(&json!({"size":"By dimensions","match_width":7}))
                .unwrap()
                .bearing,
            Bearing::MisumiB608zz
        );
        assert_eq!(
            parse(&json!({"size":"By dimensions","match_bore":6,"closure":"Open"}))
                .unwrap()
                .bearing,
            Bearing::MisumiB626
        );
        let ambiguous = error(json!({"size":"By dimensions","match_bore":17}));
        assert_eq!(ambiguous.field, "match_bore");
        for hint in [
            "6003 (17 × 35 × 10 mm)",
            "6203 (17 × 40 × 12 mm)",
            "outside diameter",
        ] {
            assert!(ambiguous.reason.contains(hint));
        }
        assert_eq!(
            error(json!({"size":"By dimensions","match_bore":17,"closure":"Open"})).field,
            "closure"
        );
        assert!(
            error(json!({"size":"By dimensions","match_bore":17,"match_width":14}))
                .reason
                .contains("no bearing")
        );
        for value in [json!(-1), json!(0), json!("17"), json!(1001)] {
            assert_eq!(
                error(json!({"size":"By dimensions","match_width":value})).field,
                "match_width"
            );
        }
        assert_eq!(
            error(json!({"size":"By dimensions","closure":"Open"})).field,
            "match_bore"
        );
    }

    #[test]
    fn manifest_identifies_the_plugin() {
        let m: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        assert_eq!(m["id"], PLUGIN);
    }
}
