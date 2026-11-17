//! What a person sets for a rail. Sizes are millimetres.

use extrusions::machining::{self, Face, FACES};
use extrusions::profile::{Grid, SIZES};
use extrusions::slot::{Slot, T5, V5};
use plugin_kit::input::{self, FieldError};
use plugin_kit::place::{self, PlaceInput};
use serde_json::Value;

pub const DEFAULT_SLOT: &str = "T-slot";
pub const DEFAULT_SIZE: &str = "2020";
pub const DEFAULT_LENGTH: f64 = 500.0;

/// The longest rail the dialog takes. Stock lengths end at 4 m (MISUMI) and 6 m.
pub const MAX_LENGTH_MM: f64 = 6000.0;

pub const DEFAULT_TAPS: &str = "None";
pub const DEFAULT_HOLE_FACE: &str = "+Y";

/// The most bores of one rail a thread is cut into. A cut thread takes a few seconds,
/// and a run has 30: SolidCraft's limit for a plugin.
pub const MAX_MODELED_TAPS: usize = 4;

/// The most screw holes one rail takes.
pub const MAX_HOLES: usize = 64;

/// Which ends of a rail are tapped. The start is the end on the sketch point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Taps {
    None,
    Start,
    End,
    Both,
}

impl Taps {
    pub fn label(&self) -> &'static str {
        match self {
            Taps::None => "None",
            Taps::Start => "Start",
            Taps::End => "End",
            Taps::Both => "Both",
        }
    }

    pub fn start(&self) -> bool {
        matches!(self, Taps::Start | Taps::Both)
    }

    pub fn end(&self) -> bool {
        matches!(self, Taps::End | Taps::Both)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub slot: Slot,
    /// The size as listed, such as `2040`.
    pub size: String,
    pub grid: Grid,
    pub length: f64,
    /// The ends whose bores are tapped M5.
    pub taps: Taps,
    /// Whether a tap's thread is cut into the bore, 12 mm deep. Otherwise the thread is
    /// listed and the bore stays plain. The dialog calls this `thread_modeled`.
    pub modeled: bool,
    /// The screw holes: the face each screw goes in from and its distance from the
    /// start, face by face and in order along the rail.
    pub holes: Vec<(Face, f64)>,
    /// The face of the holes that name none.
    pub hole_face: Face,
    pub place: PlaceInput,
    pub path: Option<crate::path::Input>,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-extrusion")?;
    let path = crate::path::parse(v)?;
    let slot = match input::choice(v, "slot", DEFAULT_SLOT)? {
        "T-slot" => T5,
        "V-slot" => V5,
        other => {
            return Err(FieldError::new(
                "slot",
                format!("must be T-slot or V-slot, not {other}"),
            ))
        }
    };
    let size = input::choice(v, "size", DEFAULT_SIZE)?;
    let grid = Grid::of(size).ok_or_else(|| {
        let listed: Vec<&str> = SIZES.iter().map(|s| s.0).collect();
        FieldError::new(
            "size",
            format!("must be one of {}, not {size}", listed.join(", ")),
        )
    })?;
    if slot == V5 && grid.nx > 1 {
        // OpenBuilds publishes sections for the single-row rails this was checked against.
        return Err(FieldError::new(
            "size",
            format!("a V-slot comes as 2020, 2040, 2060 or 2080 here; pick one of those, or T-slot for a {size}"),
        ));
    }
    // Path lengths are read from the host before modeling. Apply the shared hole
    // spacing checks here against the maximum; `path::make` checks each actual line.
    let length = match if path.is_some() {
        None
    } else {
        v.get("length")
    } {
        None | Some(Value::Null) => {
            if path.is_some() {
                MAX_LENGTH_MM
            } else {
                DEFAULT_LENGTH
            }
        }
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(_) => return Err(FieldError::new("length", "must be a number of millimetres")),
    };
    // A name shows a length to 0.01 mm, so two rails with one name are one rail.
    let length = (length * 100.0).round() / 100.0;
    if !(length.is_finite() && length > 0.0 && length <= MAX_LENGTH_MM) {
        return Err(FieldError::new(
            "length",
            format!("must be more than 0 and at most {MAX_LENGTH_MM} mm"),
        ));
    }
    let taps = match input::choice(v, "end_taps", DEFAULT_TAPS)? {
        "None" => Taps::None,
        "Start" => Taps::Start,
        "End" => Taps::End,
        "Both" => Taps::Both,
        other => {
            return Err(FieldError::new(
                "end_taps",
                format!("must be None, Start, End or Both, not {other}"),
            ))
        }
    };
    let tapped =
        machining::TAP_DEPTH * (usize::from(taps.start()) + usize::from(taps.end())) as f64;
    if length < tapped {
        return Err(FieldError::new(
            "end_taps",
            format!(
                "a tap is {} mm deep, so this rail must be at least {tapped} mm long",
                machining::TAP_DEPTH
            ),
        ));
    }
    let modeled = input::flag(v, "thread_modeled", true)?;
    let bores = grid.nx * grid.ny * (usize::from(taps.start()) + usize::from(taps.end()));
    if modeled && bores > MAX_MODELED_TAPS {
        return Err(FieldError::new(
            "thread_modeled",
            format!(
                "this rail has {bores} bores to tap, and a thread can be cut into {MAX_MODELED_TAPS} in the time a run has; turn it off to list the threads, or tap one end"
            ),
        ));
    }
    let hole_face = input::choice(v, "hole_face", DEFAULT_HOLE_FACE)?;
    let hole_face = Face::of(hole_face).ok_or_else(|| {
        let listed: Vec<&str> = FACES.iter().map(Face::name).collect();
        FieldError::new(
            "hole_face",
            format!("must be one of {}, not {hole_face}", listed.join(", ")),
        )
    })?;
    let p = Params {
        slot,
        size: size.to_string(),
        grid,
        length,
        taps,
        modeled,
        holes: holes(v, length, hole_face)?,
        hole_face,
        place: if path.is_some() {
            place::parse_input(&serde_json::json!({}))?
        } else {
            place::parse_input(v)?
        },
        path,
    };
    if v.get("catalog_selection").is_some() && p.path.is_some() {
        return Err(FieldError::new("catalog_selection", "catalog rails require a fixed length; sketch-driven rails are independent path features"));
    }
    parts::catalog::validate_selection(
        v,
        crate::provider::REVISION,
        &crate::provider::rail_representation(&p),
    )
    .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(p)
}

/// The screw holes: distances from the start of the rail, as text with commas or spaces
/// between them, or one number. A face name such as `+X` in the text puts the distances
/// after it on that face; the ones before any name are on `face`. They come back face
/// by face in the dialog's order, each face's in order along the rail, each once.
fn holes(v: &Value, length: f64, face: Face) -> Result<Vec<(Face, f64)>, FieldError> {
    let text = match v.get("holes") {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            return Err(FieldError::new(
                "holes",
                "must be distances in millimetres, such as 10, 290",
            ))
        }
    };
    let margin = machining::hole_margin();
    let mut face = face;
    let mut out: Vec<(Face, f64)> = Vec::new();
    for word in text.split([',', ' ', ';', ':']).filter(|w| !w.is_empty()) {
        if let Some(named) = Face::of(word) {
            face = named;
            continue;
        }
        let at: f64 = word.parse().map_err(|_| {
            FieldError::new(
                "holes",
                format!("{word} is not a distance or a face; type them as 10, 290 or +X 10, 290"),
            )
        })?;
        // A name shows a distance to 0.01 mm, as it does a length.
        let at = (at * 100.0).round() / 100.0;
        if !(at.is_finite() && at >= margin && at <= length - margin) {
            return Err(FieldError::new(
                "holes",
                format!(
                    "{word} is too close to an end; a hole goes from {margin} to {} mm along this rail",
                    length - margin
                ),
            ));
        }
        if !out.contains(&(face, at)) {
            out.push((face, at));
        }
    }
    if out.len() > MAX_HOLES {
        return Err(FieldError::new(
            "holes",
            format!("a rail takes at most {MAX_HOLES} holes"),
        ));
    }
    let order = |face: &Face| FACES.iter().position(|f| f == face).unwrap_or(0);
    out.sort_by(|a, b| order(&a.0).cmp(&order(&b.0)).then(a.1.total_cmp(&b.1)));
    // Holes from two faces meet in the core, so their screws would too.
    for (i, a) in out.iter().enumerate() {
        for b in &out[i + 1..] {
            if a.0 != b.0 && (a.1 - b.1).abs() < machining::ACCESS {
                return Err(FieldError::new(
                    "holes",
                    format!(
                        "the hole from {} at {} and the hole from {} at {} would meet inside the rail; keep holes from different faces {} mm apart",
                        a.0.name(),
                        a.1,
                        b.0.name(),
                        b.1,
                        machining::ACCESS
                    ),
                ));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_empty_form_is_a_half_metre_2020_t_slot() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.slot, T5);
        assert_eq!(p.size, "2020");
        assert_eq!(p.grid, Grid { nx: 1, ny: 1 });
        assert_eq!(p.length, 500.0);
    }

    #[test]
    fn an_empty_choice_is_the_default() {
        let p = parse(&json!({"slot": "", "size": " "})).unwrap();
        assert_eq!(p.slot, T5);
        assert_eq!(p.size, "2020");
    }

    #[test]
    fn a_v_slot_2080_is_one_cell_wide_and_four_tall() {
        let p = parse(&json!({"slot": "V-slot", "size": "2080", "length": 1000})).unwrap();
        assert_eq!(p.slot, V5);
        assert_eq!(p.grid, Grid { nx: 1, ny: 4 });
        assert_eq!(p.length, 1000.0);
    }

    #[test]
    fn a_v_slot_4040_is_refused_on_the_size_field() {
        let err = parse(&json!({"slot": "V-slot", "size": "4040"})).unwrap_err();
        assert_eq!(err.field, "size");
        assert!(err.reason.contains("T-slot"), "{}", err.reason);
        assert!(parse(&json!({"slot": "T-slot", "size": "4040"})).is_ok());
    }

    #[test]
    fn an_unknown_choice_names_its_field() {
        assert_eq!(parse(&json!({"slot": "U-slot"})).unwrap_err().field, "slot");
        assert_eq!(parse(&json!({"size": "3030"})).unwrap_err().field, "size");
        assert_eq!(parse(&json!({"size": 2020})).unwrap_err().field, "size");
    }

    #[test]
    fn a_length_must_be_above_zero_and_no_more_than_six_metres() {
        for length in [json!(0), json!(-5), json!(6000.5), json!("long")] {
            let err = parse(&json!({"length": length})).unwrap_err();
            assert_eq!(err.field, "length");
        }
        assert_eq!(parse(&json!({"length": 6000})).unwrap().length, 6000.0);
        assert_eq!(parse(&json!({"length": 0.5})).unwrap().length, 0.5);
    }

    #[test]
    fn a_length_is_kept_to_a_hundredth_as_its_name_shows_it() {
        assert_eq!(parse(&json!({"length": 100.004})).unwrap().length, 100.0);
        assert_eq!(parse(&json!({"length": 122.456})).unwrap().length, 122.46);
        // Too short to show is too short to cut.
        let err = parse(&json!({"length": 0.004})).unwrap_err();
        assert_eq!(err.field, "length");
    }

    #[test]
    fn a_plain_rail_has_no_taps_and_no_holes() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.taps, Taps::None);
        assert!(p.modeled);
        assert!(!parse(&json!({"thread_modeled": false})).unwrap().modeled);
        assert_eq!(
            parse(&json!({"thread_modeled": "yes"})).unwrap_err().field,
            "thread_modeled"
        );
        assert!(p.holes.is_empty());
        assert_eq!(p.hole_face, Face::YPlus);
        let p = parse(&json!({"end_taps": "", "holes": "", "hole_face": ""})).unwrap();
        assert_eq!(p.taps, Taps::None);
        assert!(p.holes.is_empty());
    }

    #[test]
    fn holes_are_distances_from_the_start_in_order() {
        let p = parse(&json!({"length": 300, "holes": "290, 10 30;270, 10", "hole_face": "-Y"}))
            .unwrap();
        let at = |face, at: &[f64]| at.iter().map(|d| (face, *d)).collect::<Vec<_>>();
        assert_eq!(p.holes, at(Face::YMinus, &[10.0, 30.0, 270.0, 290.0]));
        assert_eq!(p.hole_face, Face::YMinus);
        assert_eq!(
            parse(&json!({"holes": 250})).unwrap().holes,
            [(Face::YPlus, 250.0)]
        );
    }

    #[test]
    fn a_face_name_in_the_text_puts_the_holes_after_it_on_that_face() {
        let p =
            parse(&json!({"length": 300, "holes": "40, +X 290 10, -Y: 150", "hole_face": "-X"}))
                .unwrap();
        assert_eq!(
            p.holes,
            [
                (Face::YMinus, 150.0),
                (Face::XPlus, 10.0),
                (Face::XPlus, 290.0),
                (Face::XMinus, 40.0),
            ]
        );
        // Two faces at one place would put two screws through one point of the core.
        for holes in ["+Y 50 +X 50", "+Y 50 -Y 55", "10 +X 19"] {
            let err = parse(&json!({"length": 300, "holes": holes})).unwrap_err();
            assert_eq!(err.field, "holes");
            assert!(err.reason.contains("would meet"), "{}", err.reason);
        }
        assert!(parse(&json!({"length": 300, "holes": "+Y 50 +X 60"})).is_ok());
    }

    #[test]
    fn a_hole_must_be_clear_of_both_ends() {
        for holes in [
            json!("2"),
            json!("298"),
            json!("ten"),
            json!(true),
            json!("-10"),
        ] {
            let err = parse(&json!({"length": 300, "holes": holes})).unwrap_err();
            assert_eq!(err.field, "holes");
        }
        assert_eq!(
            parse(&json!({"hole_face": "top"})).unwrap_err().field,
            "hole_face"
        );
    }

    #[test]
    fn a_tapped_rail_is_long_enough_for_its_taps() {
        let p = parse(&json!({"length": 24, "end_taps": "Both"})).unwrap();
        assert_eq!(p.taps, Taps::Both);
        assert!(p.taps.start() && p.taps.end());
        assert!(parse(&json!({"length": 20, "end_taps": "Start"})).is_ok());
        // A 2040 has two bores an end, a 2080 four: both ends of it are too many to cut.
        assert!(parse(&json!({"size": "2040", "end_taps": "Both"})).is_ok());
        assert!(parse(&json!({"size": "2080", "end_taps": "Start"})).is_ok());
        let err = parse(&json!({"size": "2080", "end_taps": "Both"})).unwrap_err();
        assert_eq!(err.field, "thread_modeled");
        assert!(
            parse(&json!({"size": "2080", "end_taps": "Both", "thread_modeled": false})).is_ok()
        );
        for (length, taps) in [(20, "Both"), (10, "End"), (100, "Left")] {
            let err = parse(&json!({"length": length, "end_taps": taps})).unwrap_err();
            assert_eq!(err.field, "end_taps");
        }
    }

    #[test]
    fn a_named_point_needs_its_sketch() {
        let err = parse(&json!({"sketch_point": "c1"})).unwrap_err();
        assert_eq!(err.field, "sketch");
    }
}
