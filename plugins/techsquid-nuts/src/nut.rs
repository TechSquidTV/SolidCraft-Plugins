//! ISO 4032 nut parameters. Sizes are millimetres.
//!
//! The bearing face sits on a sketch point. The sketch normal points outward and the nut
//! stands on that side. A blank width and height come from the ISO 4032 table. A fine
//! pitch such as `M8x1` keeps the M8 nut and passes that pitch to the bore.

use fasteners::clearance::Hole;
use fasteners::designation;
use fasteners::metric::{self, MetricSize};
use fasteners::nut::{self, NutSize};
pub use plugin_kit::create::Create;
use plugin_kit::create::Words;
use plugin_kit::input::{self, FieldError};
use plugin_kit::place::{self, PlaceInput};
use serde_json::Value;

pub const DEFAULT_DESIGNATION: &str = "M3";

/// What a run makes. A pocket needs a part under the point, so the default is the nut
/// alone, which also stands on the origin with nothing selected.
pub const CREATE: Words = Words {
    part_only: "Nut only",
    both: "Nut and pocket",
    cut_only: "Pocket only",
    default: Create::Part,
    part: "nut",
    cut: "pocket",
    cuts: "pockets",
    make: "cut",
    made: "cut",
};

/// Gap between each flat of the nut and its pocket, when the field is left alone. A
/// printed pocket comes out small, and a nut has to drop in.
pub const DEFAULT_POCKET_ADJUST: f64 = 0.2;
/// The widest gap the dialog takes.
pub const MAX_POCKET_ADJUST: f64 = 1.0;

/// A hexagonal pocket the nut sits in, flush with the surface, over a clearance hole
/// for its bolt.
#[derive(Clone, Debug, PartialEq)]
pub struct Pocket {
    pub across_flats: f64,
    pub depth: f64,
    /// The ISO 273 medium clearance hole for the bolt.
    pub clearance: Hole,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    /// Explicit dimension overrides and non-ISO pitches are custom configurations.
    pub custom: bool,
    pub create: Create,
    /// The pocket to cut in the part. Empty when only the nut is made.
    pub pocket: Option<Pocket>,
    /// What the person typed, such as `M6` or `M8x1`.
    pub designation: String,
    /// What `solid.hole` is given: the designation, which SolidCraft reads with its coarse pitch.
    pub thread: String,
    pub across_flats: f64,
    pub height: f64,
    pub major: f64,
    pub pitch: f64,
    /// `da` max. Empty when the size is not in the table.
    pub countersink: Option<f64>,
    /// `c` max. Empty when the size is not in the table, and the chamfer then runs to the flats.
    pub chamfer_max: Option<f64>,
    /// `solid.hole`'s `modeled`. The dialog calls this `thread_modeled`.
    pub modeled: bool,
    pub place: PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-nut")?;
    let designation = designation::designation(v, DEFAULT_DESIGNATION)?;
    let size_field = designation::size_field(v);
    let across_flats = input::optional_length(v, "custom_width")?;
    let height = input::optional_length(v, "custom_thickness")?;
    let modeled = input::flag(v, "thread_modeled", true)?;
    let place = place::parse_input(v)?;
    let (size, thread, listed) =
        resolve_size(&designation).map_err(|e| e.moved("size", size_field))?;
    let custom = across_flats.is_some()
        || height.is_some()
        || listed.is_none_or(|row| row.pitch != size.pitch);
    let (across_flats, height, countersink, chamfer_max) =
        resolve_body(listed, across_flats, height)?;
    let create = CREATE.parse(v)?;
    let pocket = if create.cut() {
        let adjust = match input::optional_length_or_zero(v, "pocket_adjust")? {
            None => DEFAULT_POCKET_ADJUST,
            Some(adjust) if (0.0..=MAX_POCKET_ADJUST).contains(&adjust) => adjust,
            Some(_) => {
                return Err(FieldError::new(
                    "pocket_adjust",
                    format!("must be between 0 and {MAX_POCKET_ADJUST} mm"),
                ))
            }
        };
        let clearance = Hole::for_major(size.major).ok_or_else(|| {
            FieldError::new(
                size_field,
                format!(
                    "ISO 273 has no clearance hole for M{}, so the pocket has no bolt hole; pick a listed size or use Nut only",
                    metric::number(size.major)
                ),
            )
        })?;
        Some(Pocket {
            across_flats: across_flats + 2.0 * adjust,
            depth: height,
            clearance,
        })
    } else {
        None
    };
    let p = Params {
        custom,
        create,
        pocket,
        designation,
        thread,
        across_flats,
        height,
        major: size.major,
        pitch: size.pitch,
        countersink,
        chamfer_max,
        modeled,
        place,
    };
    validate(&p)?;
    parts::catalog::validate_selection(
        v,
        crate::provider::REVISION,
        &crate::provider::representation(&p),
    )
    .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(p)
}

fn validate(p: &Params) -> Result<(), FieldError> {
    designation::check_designation(&p.designation).map_err(|e| e.moved("size", "custom_size"))?;
    input::check_length("custom_width", p.across_flats)?;
    input::check_length("custom_thickness", p.height)?;
    if p.across_flats <= p.major {
        return Err(FieldError::new(
            "custom_width",
            "must be wider than the thread",
        ));
    }
    let end = nut::end_chamfer(p.across_flats, p.chamfer_max);
    if p.height <= 2.0 * end.axial {
        return Err(FieldError::new(
            "custom_thickness",
            "must be taller than the two end chamfers",
        ));
    }
    // A listed countersink is still cut when the body is overridden. It has to stay
    // inside the flats, and the two ends must not meet.
    if let Some(da) = p.countersink {
        if p.across_flats <= da {
            return Err(FieldError::new(
                "custom_width",
                "must be wider than the countersink",
            ));
        }
        let depth = nut::countersink_depth(p.major, da);
        if p.height <= 2.0 * depth {
            return Err(FieldError::new(
                "custom_thickness",
                "must be taller than the two countersinks",
            ));
        }
    }
    Ok(())
}

fn resolve_size(designation: &str) -> Result<(MetricSize, String, Option<NutSize>), FieldError> {
    match metric::parse_metric(designation) {
        Ok(Some(size)) => Ok((size, designation.to_string(), nut::standard_nut(size.major))),
        Ok(None) => Err(FieldError::new(
            "size",
            "an ISO 4032 nut is metric; use a size such as M6 or M8x1",
        )),
        Err(err) => Err(err),
    }
}

fn resolve_body(
    listed: Option<NutSize>,
    across_flats: Option<f64>,
    height: Option<f64>,
) -> Result<(f64, f64, Option<f64>, Option<f64>), FieldError> {
    let (across_flats, height) = match (across_flats, height, listed) {
        (Some(across_flats), Some(height), _) => (across_flats, height),
        (None, None, Some(nut)) => (nut.across_flats, nut.height),
        (Some(across_flats), None, Some(nut)) => (across_flats, nut.height),
        (None, Some(height), Some(nut)) => (nut.across_flats, height),
        (Some(_), None, None) => {
            return Err(FieldError::new("custom_thickness", BOTH));
        }
        (None, Some(_), None) => {
            return Err(FieldError::new("custom_width", BOTH));
        }
        (None, None, None) => {
            return Err(FieldError::new("custom_width", NO_STANDARD));
        }
    };
    let countersink = listed.map(|nut| nut.countersink);
    let chamfer_max = listed.map(|nut| nut.chamfer_max);
    Ok((across_flats, height, countersink, chamfer_max))
}

const BOTH: &str = "set the width across the flats and the height, or leave both empty";
const NO_STANDARD: &str =
    "this size has no ISO 4032 nut here; set the width across the flats and the height";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_blank_form_is_an_m3_nut_like_the_screw_form() {
        let blank = parse(&json!({})).unwrap();
        let m3 = parse(&json!({"size": "M3"})).unwrap();
        assert_eq!(blank.designation, "M3");
        assert_eq!(blank.thread, m3.thread);
        assert_eq!(blank.across_flats, m3.across_flats);
        assert_eq!(blank.height, m3.height);
    }

    #[test]
    fn an_m6_is_the_iso_4032_m6() {
        let p = parse(&json!({"size": "M6"})).unwrap();
        assert_eq!(p.designation, "M6");
        assert_eq!(p.thread, "M6");
        assert_eq!(p.across_flats, 10.0);
        assert_eq!(p.height, 5.2);
        assert_eq!(p.major, 6.0);
        assert_eq!(p.pitch, 1.0);
        assert_eq!(p.countersink, Some(6.75));
        assert_eq!(p.chamfer_max, Some(0.5));
        assert!(p.modeled);
        assert!(!parse(&json!({"thread_modeled": false})).unwrap().modeled);
    }

    #[test]
    fn a_fine_pitch_keeps_the_m8_nut() {
        let p = parse(&json!({"size": "M8x1"})).unwrap();
        assert_eq!(p.thread, "M8x1");
        assert_eq!(p.across_flats, 13.0);
        assert_eq!(p.height, 6.8);
        assert_eq!(p.pitch, 1.0);
    }

    #[test]
    fn a_bare_m3_5_takes_the_table_pitch() {
        let p = parse(&json!({"size": "M3.5"})).unwrap();
        assert_eq!(p.thread, "M3.5");
        assert_eq!(p.across_flats, 6.0);
        assert_eq!(p.height, 2.8);
        assert_eq!(p.pitch, 0.6);
    }

    #[test]
    fn one_override_keeps_the_other_standard() {
        let wider = parse(&json!({"size": "M6", "custom_width": 11.0})).unwrap();
        assert_eq!(wider.across_flats, 11.0);
        assert_eq!(wider.height, 5.2);
        let taller = parse(&json!({"size": "M6", "custom_thickness": 6.0})).unwrap();
        assert_eq!(taller.across_flats, 10.0);
        assert_eq!(taller.height, 6.0);
    }

    #[test]
    fn an_unlisted_size_needs_both_dimensions() {
        let err = parse(&json!({"size": "M7x1"})).unwrap_err();
        assert_eq!(err.field, "custom_width");
        let p = parse(&json!({
            "size": "M7x1",
            "custom_width": 11.0,
            "custom_thickness": 5.5,
        }))
        .unwrap();
        assert_eq!(p.thread, "M7x1");
        assert_eq!(p.major, 7.0);
        assert!(p.countersink.is_none());
        assert!(p.chamfer_max.is_none());
    }

    #[test]
    fn a_nut_narrower_than_the_thread_is_refused() {
        let err = parse(&json!({"size": "M6", "custom_width": 6.0})).unwrap_err();
        assert_eq!(err.field, "custom_width");
    }

    #[test]
    fn a_nut_shorter_than_its_chamfers_is_refused() {
        let err = parse(&json!({"size": "M6", "custom_thickness": 0.5})).unwrap_err();
        assert_eq!(err.field, "custom_thickness");
    }

    #[test]
    fn a_countersink_must_fit_the_overridden_body() {
        let narrow = parse(&json!({"size": "M64", "custom_width": 65.0})).unwrap_err();
        assert_eq!(narrow.field, "custom_width");
        let short = parse(&json!({"size": "M64", "custom_thickness": 2.5})).unwrap_err();
        assert_eq!(short.field, "custom_thickness");
    }

    #[test]
    fn every_standard_size_parses() {
        for nut in fasteners::nut::sizes() {
            let designation = format!("M{}", metric::number(nut.nominal));
            let p = parse(&json!({"size": designation})).unwrap();
            assert_eq!(p.across_flats, nut.across_flats, "{designation}");
            assert_eq!(p.height, nut.height, "{designation}");
        }
    }

    #[test]
    fn a_custom_size_replaces_the_listed_one() {
        let p = parse(&json!({"size": "M6", "custom_size": "M8x1"})).unwrap();
        assert_eq!(p.thread, "M8x1");
        assert_eq!(p.across_flats, 13.0);
    }

    #[test]
    fn every_size_in_the_dialog_list_is_a_standard_nut() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let sizes =
            &manifest["runtime"]["entries"][0]["input_schema"]["properties"]["size"]["enum"];
        for size in sizes.as_array().unwrap() {
            let size = size.as_str().unwrap();
            let p = parse(&json!({"size": size})).unwrap();
            assert!(p.countersink.is_some(), "{size}");
        }
    }

    #[test]
    fn an_inch_size_is_not_a_metric_nut() {
        let err = parse(&json!({"size": "1/4-20 UNC"})).unwrap_err();
        assert_eq!(err.field, "size");
    }

    #[test]
    fn a_pocket_is_the_nut_plus_a_gap_on_each_flat() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.create, Create::Part);
        assert_eq!(p.pocket, None);
        let p = parse(&json!({"size": "M6", "create": "Nut and pocket"})).unwrap();
        let pocket = p.pocket.unwrap();
        // ISO 4032 M6: 10 across the flats and 5.2 high.
        assert!((pocket.across_flats - 10.4).abs() < 1e-9);
        assert_eq!(pocket.depth, 5.2);
        assert_eq!(pocket.clearance, Hole::Size("M6".into()));
        let tight =
            parse(&json!({"size": "M6", "create": "Pocket only", "pocket_adjust": 0})).unwrap();
        assert_eq!(tight.pocket.unwrap().across_flats, 10.0);
        let fine = parse(&json!({"custom_size": "M8x1", "create": "Pocket only"})).unwrap();
        assert_eq!(fine.pocket.unwrap().clearance, Hole::Size("M8".into()));
        // M3.5 is in ISO 273 but not in the host's clearance table: a plain diameter.
        let odd = parse(&json!({"custom_size": "M3.5", "create": "Nut and pocket"})).unwrap();
        assert_eq!(odd.pocket.unwrap().clearance, Hole::Diameter(3.9));
        // M7.5x1 has no ISO 273 hole, so the pocket is refused before anything is cut.
        let err = parse(&json!({
            "custom_size": "M7.5x1", "create": "Pocket only",
            "custom_width": 12, "custom_thickness": 6,
        }))
        .unwrap_err();
        assert_eq!(err.field, "custom_size");
        let err = parse(&json!({"create": "Pocket only", "pocket_adjust": 2})).unwrap_err();
        assert_eq!(err.field, "pocket_adjust");
        let err = parse(&json!({"create": "Tray"})).unwrap_err();
        assert_eq!(err.field, "create");
    }

    #[test]
    fn a_bad_custom_size_is_reported_on_custom_size() {
        let err = parse(&json!({"size": "M3", "custom_size": "M7.5"})).unwrap_err();
        assert_eq!(err.field, "custom_size");
        let err = parse(&json!({"size": "M3", "custom_size": "1/4-20 UNC"})).unwrap_err();
        assert_eq!(err.field, "custom_size");
        let err = parse(&json!({"size": "1/4-20 UNC"})).unwrap_err();
        assert_eq!(err.field, "size");
    }

    #[test]
    fn a_listed_size_is_a_plain_decimal() {
        // M3.5 is an ISO 4032 size, and ISO 261 gives it a coarse pitch.
        assert_eq!(parse(&json!({"custom_size": "M3.5"})).unwrap().pitch, 0.6);
        assert!(parse(&json!({"custom_size": "M1e1"})).is_err());
        assert!(parse(&json!({"custom_size": "M+3.5"})).is_err());
        assert!(parse(&json!({"custom_size": "M3."})).is_err());
    }
}
