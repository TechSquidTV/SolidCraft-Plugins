//! Heat-set insert parameters. Sizes are millimetres.
//!
//! Size picks a catalog row: its diameter, its length and the screw in its bore. Each
//! custom field replaces one of those and leaves the others on the row. The face end
//! sits on a sketch point and the tip runs into the part. Nothing selected puts the
//! face on the origin.

use fasteners::metric;
use plugin_kit::input::{self, FieldError};
use plugin_kit::place::{self, PlaceInput};
use serde_json::Value;

use crate::catalog::{self, Row};
use crate::knurl;

/// What a run makes. The hole is the part most people want, so it is in the default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Create {
    Both,
    HoleOnly,
    InsertOnly,
}

impl Create {
    pub fn label(self) -> &'static str {
        match self {
            Create::Both => "Hole and insert",
            Create::HoleOnly => "Hole only",
            Create::InsertOnly => "Insert only",
        }
    }

    pub fn hole(self) -> bool {
        self != Create::InsertOnly
    }

    pub fn insert(self) -> bool {
        self != Create::HoleOnly
    }
}

pub const STRAIGHT: &str = "Heat-set (straight)";
pub const STEPPED: &str = "Glue-in (stepped)";

/// The hole in the part. Diameters already carry the adjustment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HoleShape {
    /// The plastic hole D2 of a heat-set insert.
    Straight { diameter: f64 },
    /// The resin hole: `d1` to `l1`, `d2` to `l2`, then `d3`.
    Stepped {
        d1: f64,
        d2: f64,
        d3: f64,
        l1: f64,
        l2: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoleCut {
    pub shape: HoleShape,
    /// Depth from the surface. `None` drills through.
    pub depth: Option<f64>,
}

/// The widest change to the hole diameter, either way.
pub const MAX_ADJUST: f64 = 1.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    /// The catalog row the dialog chose.
    pub size: &'static str,
    pub thread: String,
    pub length: f64,
    pub diameter: f64,
    pub modeled: bool,
    /// Basic major diameter, so the knurl root stays outside the thread.
    pub major: f64,
    /// A bore diameter when the host has no name for this thread.
    pub plain_bore: Option<f64>,
    /// The plastic hole and the wall beside it, when the diameter is the row's.
    pub hole: Option<f64>,
    pub wall: Option<f64>,
    pub create: Create,
    /// The hole to drill in the part. Empty when only the insert is made.
    pub cut: Option<HoleCut>,
    pub place: PlaceInput,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    let row = size(v)?;
    let custom_length = input::optional_length(v, "custom_length")?;
    let custom_diameter = input::optional_length(v, "custom_diameter")?;
    let length = custom_length.unwrap_or(row.length);
    let diameter = custom_diameter.unwrap_or(row.diameter);
    input::check_length("custom_length", length)?;
    input::check_length("custom_diameter", diameter)?;
    let bore = bore(v, row)?;
    let modeled = input::flag(
        v,
        if v.get("thread_modeled").is_some() {
            "thread_modeled"
        } else {
            "modeled"
        },
        true,
    )
    .map_err(|err| FieldError::new("thread_modeled", err.reason))?;
    let place = place::parse_input(v)?;
    // SolidCraft models metric threads only. A Unified or pipe thread stays cosmetic,
    // as the dialog says, rather than refusing the run.
    let modeled = modeled && bore.metric;
    let smallest = knurl::min_crest_diameter(bore.major);
    if diameter < smallest {
        return Err(FieldError::new(
            "custom_diameter",
            format!(
                "the insert is {diameter} mm across; its knurl needs at least {smallest:.2} mm to stay outside the {} thread; pick the Size for that thread, or set a larger diameter",
                bore.thread
            ),
        ));
    }
    let create = create(v)?;
    let cut = if create.hole() {
        Some(hole_cut(
            v,
            row,
            diameter,
            length,
            custom_diameter.is_some() || custom_length.is_some(),
        )?)
    } else {
        None
    };
    let listed = custom_diameter.is_none();
    Ok(Params {
        create,
        cut,
        size: row.size,
        thread: bore.thread,
        length,
        diameter,
        modeled,
        major: bore.major,
        plain_bore: bore.plain_bore,
        hole: listed.then_some(row.hole),
        wall: listed.then_some(row.wall),
        place,
    })
}

fn choice<'a>(v: &'a Value, key: &str, default: &'a str) -> Result<&'a str, FieldError> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::String(text)) => Ok(text.as_str()),
        Some(_) => Err(FieldError::new(key, "must be a choice from the list")),
    }
}

fn create(v: &Value) -> Result<Create, FieldError> {
    match choice(v, "create", "Hole and insert")? {
        "Hole and insert" => Ok(Create::Both),
        "Hole only" => Ok(Create::HoleOnly),
        "Insert only" => Ok(Create::InsertOnly),
        other => Err(FieldError::new(
            "create",
            format!("must be Hole and insert, Hole only or Insert only, not {other}"),
        )),
    }
}

/// The hole for this insert. `custom` is true when the diameter or length was typed.
fn hole_cut(
    v: &Value,
    row: &Row,
    diameter: f64,
    length: f64,
    custom: bool,
) -> Result<HoleCut, FieldError> {
    let adjust = match input::optional_length(v, "hole_adjust")? {
        None => 0.0,
        Some(adjust) if adjust.is_finite() && adjust.abs() <= MAX_ADJUST => adjust,
        Some(_) => {
            return Err(FieldError::new(
                "hole_adjust",
                format!("must be between -{MAX_ADJUST} and {MAX_ADJUST} mm"),
            ))
        }
    };
    let depth = match choice(v, "hole_depth", "Blind")? {
        // CNC Kitchen: a blind hole is about 1 mm deeper than the insert.
        "Blind" => Some(length + 1.0),
        "Through" => None,
        other => {
            return Err(FieldError::new(
                "hole_depth",
                format!("must be Blind or Through, not {other}"),
            ))
        }
    };
    let shape = match choice(v, "hole_type", STRAIGHT)? {
        STRAIGHT => {
            // A typed diameter keeps the row's interference with the plastic.
            let hole = diameter - (row.diameter - row.hole) + adjust;
            if !(hole > 0.0 && hole < diameter) {
                return Err(FieldError::new(
                    "hole_adjust",
                    format!(
                        "the hole would be {hole:.2} mm for a {diameter} mm insert; it has to stay narrower than the insert"
                    ),
                ));
            }
            HoleShape::Straight { diameter: hole }
        }
        STEPPED => {
            if custom {
                return Err(FieldError::new(
                    "hole_type",
                    "a glue-in hole uses the catalog size; clear Custom diameter and Custom length, or use Heat-set (straight)",
                ));
            }
            let step = catalog::stepped(row.size).ok_or_else(|| {
                FieldError::new(
                    "hole_type",
                    format!(
                        "{} has no glue-in hole; CNC Kitchen lists them for {}. Pick one of those, or use Heat-set (straight)",
                        row.size,
                        catalog::stepped_sizes()
                    ),
                )
            })?;
            // As for the straight hole: the step the insert presses into stays
            // narrower than the insert.
            let grip = step.d2 + adjust;
            if !(step.d3 + adjust > 0.0 && grip < row.diameter) {
                return Err(FieldError::new(
                    "hole_adjust",
                    format!(
                        "the glue-in hole would be {grip:.2} mm where the insert grips, for a {} mm insert; it has to stay narrower than the insert",
                        row.diameter
                    ),
                ));
            }
            HoleShape::Stepped {
                d1: step.d1 + adjust,
                d2: step.d2 + adjust,
                d3: step.d3 + adjust,
                l1: step.l1,
                l2: step.l2,
            }
        }
        other => {
            return Err(FieldError::new(
                "hole_type",
                format!("must be {STRAIGHT} or {STEPPED}, not {other}"),
            ))
        }
    };
    Ok(HoleCut { shape, depth })
}

fn size(v: &Value) -> Result<&'static Row, FieldError> {
    let text = match v.get("size") {
        None | Some(Value::Null) => catalog::DEFAULT_SIZE,
        Some(Value::String(text)) => text.as_str(),
        Some(_) => return Err(FieldError::new("size", "must be a size from the list")),
    };
    catalog::find_size(text).ok_or_else(|| {
        FieldError::new(
            "size",
            format!("must be a size from the list, such as M3 x 5.7, not {text}"),
        )
    })
}

struct Bore {
    thread: String,
    major: f64,
    plain_bore: Option<f64>,
    metric: bool,
}

impl Bore {
    fn of(row: &Row) -> Self {
        Bore {
            thread: row.thread.to_string(),
            major: row.major,
            plain_bore: row.plain_bore,
            metric: catalog::metric(row.thread),
        }
    }
}

const BAD_THREAD: &str = "use a size such as M3 or M8x1";

/// The row's thread, or `custom_thread` when it is filled.
fn bore(v: &Value, row: &Row) -> Result<Bore, FieldError> {
    let text = match v.get("custom_thread") {
        None | Some(Value::Null) => return Ok(Bore::of(row)),
        Some(Value::String(text)) => text.trim().to_string(),
        Some(_) => return Err(FieldError::new("custom_thread", BAD_THREAD)),
    };
    if text.is_empty() {
        return Ok(Bore::of(row));
    }
    if let Some(found) = catalog::find_thread(&text) {
        return Ok(Bore::of(found));
    }
    match metric::parse_metric(&text) {
        Ok(Some(size)) => Ok(Bore {
            thread: text,
            major: size.major,
            plain_bore: None,
            metric: true,
        }),
        Ok(None) => Err(FieldError::new("custom_thread", BAD_THREAD)),
        Err(err) => Err(FieldError::new("custom_thread", err.reason)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_default_is_an_m3_insert() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.thread, "M3");
        assert_eq!(p.length, 5.7);
        assert_eq!(p.diameter, 4.6);
        assert_eq!(p.major, 3.0);
        assert!(p.modeled);
        assert!(!parse(&json!({"thread_modeled": false})).unwrap().modeled);
        assert_eq!(p.plain_bore, None);
        assert_eq!(p.size, "M3 × 5.7");
        assert_eq!(p.hole, Some(4.0));
        assert_eq!(p.wall, Some(1.6));
    }

    #[test]
    fn a_size_sets_diameter_length_and_thread_together() {
        let p = parse(&json!({"size": "M5 × 9.5"})).unwrap();
        assert_eq!(p.thread, "M5");
        assert_eq!(p.diameter, 7.1);
        assert_eq!(p.length, 9.5);
        assert_eq!(p.major, 5.0);
        // A plain x and loose spacing find the same row.
        assert_eq!(parse(&json!({"size": "m5 x 9.5"})).unwrap(), p);
        let err = parse(&json!({"size": "M7 × 9"})).unwrap_err();
        assert_eq!(err.field, "size");
    }

    #[test]
    fn the_default_drills_the_catalog_hole_one_millimetre_past_the_insert() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.create, Create::Both);
        let cut = p.cut.unwrap();
        assert_eq!(cut.shape, HoleShape::Straight { diameter: 4.0 });
        assert_eq!(cut.depth, Some(6.7));
        let p = parse(&json!({"create": "Insert only"})).unwrap();
        assert_eq!(p.cut, None);
    }

    #[test]
    fn an_adjustment_and_a_through_hole_change_the_cut() {
        let p = parse(&json!({"hole_adjust": 0.1, "hole_depth": "Through"})).unwrap();
        let cut = p.cut.unwrap();
        assert_eq!(cut.depth, None);
        match cut.shape {
            HoleShape::Straight { diameter } => assert!((diameter - 4.1).abs() < 1e-9),
            other => panic!("{other:?}"),
        }
        // A typed diameter keeps the 0.6 mm the row leaves for the plastic.
        let p = parse(&json!({"custom_diameter": 5.0})).unwrap();
        match p.cut.unwrap().shape {
            HoleShape::Straight { diameter } => assert!((diameter - 4.4).abs() < 1e-9),
            other => panic!("{other:?}"),
        }
        let err = parse(&json!({"hole_adjust": 0.7})).unwrap_err();
        assert_eq!(err.field, "hole_adjust");
        let err = parse(&json!({"hole_adjust": 2})).unwrap_err();
        assert_eq!(err.field, "hole_adjust");
    }

    #[test]
    fn a_glue_in_hole_is_the_stepped_row() {
        let p = parse(&json!({"hole_type": STEPPED, "hole_adjust": 0.1})).unwrap();
        let cut = p.cut.unwrap();
        assert_eq!(cut.depth, Some(6.7));
        match cut.shape {
            HoleShape::Stepped { d1, d2, d3, l1, l2 } => {
                assert!((d1 - 4.7).abs() < 1e-9);
                assert!((d2 - 4.5).abs() < 1e-9);
                assert!((d3 - 4.1).abs() < 1e-9);
                assert_eq!((l1, l2), (1.7, 4.2));
            }
            other => panic!("{other:?}"),
        }
        let err = parse(&json!({"hole_type": STEPPED, "size": "M8 × 12.7"})).unwrap_err();
        assert_eq!(err.field, "hole_type");
        assert!(err.reason.contains("M3 × 5.7"), "{}", err.reason);
        let err = parse(&json!({"hole_type": STEPPED, "custom_length": 5})).unwrap_err();
        assert_eq!(err.field, "hole_type");
    }

    #[test]
    fn a_custom_diameter_drops_the_catalog_hole() {
        let p = parse(&json!({"custom_diameter": 5.0})).unwrap();
        assert_eq!(p.hole, None);
        assert_eq!(p.wall, None);
    }

    #[test]
    fn diameter_and_length_are_what_was_typed() {
        let p =
            parse(&json!({"custom_diameter": 5.0, "custom_length": 4.0, "custom_thread": "M3"}))
                .unwrap();
        assert_eq!(p.diameter, 5.0);
        assert_eq!(p.length, 4.0);
        assert_eq!(p.thread, "M3");
        assert_eq!(p.major, 3.0);
    }

    #[test]
    fn a_fine_pitch_uses_that_major() {
        let p = parse(&json!({"custom_thread": "M8x1", "custom_diameter": 10.2})).unwrap();
        assert_eq!(p.thread, "M8x1");
        assert_eq!(p.major, 8.0);
    }

    #[test]
    fn a_number_two_insert_has_a_plain_bore() {
        let p = parse(&json!({"size": "#2-56 × 3.2"})).unwrap();
        assert_eq!(p.thread, "#2-56 UNC");
        assert_eq!(p.plain_bore, Some(0.086 * 25.4 - 25.4 / 56.0));
    }

    #[test]
    fn an_unknown_thread_is_refused() {
        let err = parse(&json!({"custom_thread": "nope"})).unwrap_err();
        assert_eq!(err.field, "custom_thread");
    }

    #[test]
    fn a_modeled_unified_or_pipe_thread_stays_cosmetic() {
        for size in ["1/4-20 × 12.7", "#4-40 × 5.7", "G1/8 × 12.7"] {
            let p = parse(&json!({"size": size, "thread_modeled": true})).unwrap();
            assert!(!p.modeled, "{size}");
        }
        let p = parse(&json!({"size": "1/4-20 × 12.7", "modeled": true})).unwrap();
        assert!(!p.modeled);
        let p = parse(&json!({"size": "M3 × 5.7", "thread_modeled": true})).unwrap();
        assert!(p.modeled);
    }

    #[test]
    fn a_glue_in_hole_stays_narrower_than_the_insert() {
        let glue = |adjust: f64| {
            parse(&json!({"hole_type": STEPPED, "create": "Hole only", "hole_adjust": adjust}))
        };
        let err = glue(1.0).unwrap_err();
        assert_eq!(err.field, "hole_adjust");
        assert!(err.reason.contains("narrower"));
        assert_eq!(glue(0.2).unwrap_err().field, "hole_adjust");
        let Some(HoleCut {
            shape: HoleShape::Stepped { d2, .. },
            ..
        }) = glue(0.1).unwrap().cut
        else {
            panic!("a glue-in hole is stepped");
        };
        assert!((d2 - 4.5).abs() < 1e-9);
        assert!(glue(-1.0).is_ok());
    }

    #[test]
    fn a_custom_diameter_keeps_the_knurl_outside_the_thread() {
        let err = parse(&json!({"create": "Insert only", "custom_diameter": 3.1})).unwrap_err();
        assert_eq!(err.field, "custom_diameter");
        assert!(err.reason.contains("M3"));
        assert!(parse(&json!({"create": "Insert only", "custom_diameter": 3.0})).is_err());
        let p = parse(&json!({"create": "Insert only", "custom_diameter": 3.3})).unwrap();
        let shape = knurl::shape(p.length, p.diameter, p.major);
        assert!(shape.gap_diameter > p.major && shape.root * 2.0 > p.major);
    }

    #[test]
    fn a_diameter_inside_the_thread_is_refused() {
        let err = parse(&json!({"custom_diameter": 2.5})).unwrap_err();
        assert_eq!(err.field, "custom_diameter");
        // A thread typed over a smaller row says what to do about it.
        let err = parse(&json!({"custom_thread": "M5"})).unwrap_err();
        assert_eq!(err.field, "custom_diameter");
        assert!(err.reason.contains("pick the Size"), "{}", err.reason);
    }
}
