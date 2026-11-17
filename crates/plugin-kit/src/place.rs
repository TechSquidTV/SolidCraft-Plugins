//! Where a part sits.
//!
//! The point is a sketch point, or the centre of a circle in a sketch, picked in the
//! dialog's Placement field. The sketch plane is the surface. Its normal points to the
//! outside. For a screw, the head stands on that side and the shank runs the other way, and
//! the top of the shank, where the head begins, is the point itself.
//!
//! With nothing picked, the point is the origin and the normal is +Z, so the head stands on
//! the XY plane.

use serde_json::{json, Value};

use crate::{FieldError, Runner};

/// The key of the selection field every placing plugin shares. SolidCraft draws it as a
/// pick field (`x-solidcraft-selection`) and sends the picks as selection references.
pub const FIELD: &str = "placement";

/// How the point was chosen. The reply says so, because the origin is also what a
/// forgotten pick gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Origin,
    Picked,
}

impl Source {
    pub fn words(self) -> &'static str {
        match self {
            Source::Origin => "on the origin, because no point was picked",
            Source::Picked => "on the picked sketch point",
        }
    }
}

/// A placed fastener. `normal` is a unit vector. `x_dir` and `y_dir` are the sketch axes,
/// and `x_dir` × `y_dir` is `normal`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub point: [f64; 3],
    pub normal: [f64; 3],
    pub x_dir: [f64; 3],
    pub y_dir: [f64; 3],
    pub source: Source,
}

/// A sketch point or curve from the Placement field, as SolidCraft sends it:
/// `{"type": "external_sketch_point", "sketch": 2, "id": "p3"}`. The sketch is its feature
/// id; a program calling the plugin may name it instead.
#[derive(Clone, Debug, PartialEq)]
pub struct Pick {
    pub sketch: Value,
    pub id: String,
    pub curve: bool,
}

/// The picks from the dialog. None means the origin.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlaceInput {
    pub picks: Vec<Pick>,
}

impl Placement {
    pub fn on_origin() -> Self {
        Self {
            point: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            x_dir: [1.0, 0.0, 0.0],
            y_dir: [0.0, 1.0, 0.0],
            source: Source::Origin,
        }
    }

    /// The tip, `length` from the point against the normal.
    pub fn tip(&self, length: f64) -> [f64; 3] {
        add(self.point, scale(self.normal, -length))
    }

    /// A point `distance` from this point along the normal.
    pub fn along(&self, distance: f64) -> [f64; 3] {
        add(self.point, scale(self.normal, distance))
    }

    /// A point on the shank surface, `along` up from the tip and `radial` out along `x_dir`.
    pub fn on_shank(&self, length: f64, along: f64, radial: f64) -> [f64; 3] {
        add(
            add(self.tip(length), scale(self.normal, along)),
            scale(self.x_dir, radial),
        )
    }
}

/// The Placement picks, for a plugin that places on sketch points and circles.
pub fn parse_input(v: &Value) -> Result<PlaceInput, FieldError> {
    Ok(PlaceInput {
        picks: parse_picks(v, "sketch points or circles")?,
    })
}

/// The Placement picks in pick order. `wanted` says what the plugin takes, for the error
/// when something else is given.
pub fn parse_picks(v: &Value, wanted: &str) -> Result<Vec<Pick>, FieldError> {
    let items = match v.get(FIELD) {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(items)) => items,
        Some(_) => {
            return Err(FieldError::new(
                FIELD,
                format!("must be a list of picked {wanted}"),
            ))
        }
    };
    items.iter().map(|item| pick(item, wanted)).collect()
}

fn pick(item: &Value, wanted: &str) -> Result<Pick, FieldError> {
    let curve = match item.get("type").and_then(Value::as_str) {
        Some("external_sketch_point") => false,
        Some("external_sketch_curve") => true,
        _ => {
            return Err(FieldError::new(
                FIELD,
                format!("pick {wanted}; a face, an edge or a body cannot place it"),
            ))
        }
    };
    let sketch = match item.get("sketch") {
        Some(Value::Number(n)) if n.as_u64().is_some() => Value::Number(n.clone()),
        Some(Value::String(s)) if !s.trim().is_empty() => Value::String(s.trim().to_string()),
        _ => {
            return Err(FieldError::new(
                FIELD,
                "each pick names its sketch, by feature id or by name",
            ))
        }
    };
    let id = item
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| FieldError::new(FIELD, "each pick names a point or a curve id"))?;
    Ok(Pick {
        sketch,
        id: id.to_string(),
        curve,
    })
}

/// One placed point, with the sketch and the point id a host command can name.
#[derive(Clone, Debug, PartialEq)]
pub struct Spot {
    pub sketch: String,
    /// The point id itself: a circle resolves to its centre.
    pub point: String,
    pub place: Placement,
}

/// Every picked point, in pick order, each once. A picked circle or arc is its centre.
/// Nothing picked is an empty list, which the caller may put on the origin.
pub fn resolve_all<R: Runner>(r: &mut R, input: &PlaceInput) -> Result<Vec<Spot>, String> {
    let mut inspected: Vec<(Value, String, Value)> = Vec::new();
    let mut spots: Vec<Spot> = Vec::new();
    for pick in &input.picks {
        if !inspected.iter().any(|(key, _, _)| *key == pick.sketch) {
            let inspect = r
                .run("sketch.inspect", json!({"sketch": pick.sketch}))
                .map_err(|err| format!("{FIELD}: {err}"))?;
            let name = inspect
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{FIELD}: sketch {} has no name", pick.sketch))?
                .to_string();
            inspected.push((pick.sketch.clone(), name, inspect));
        }
        let (_, sketch, inspect) = inspected
            .iter()
            .find(|(key, _, _)| *key == pick.sketch)
            .expect("inspected above");
        let point = if pick.curve {
            curve_center(inspect, &pick.id).ok_or_else(|| {
                format!(
                    "{FIELD}: {} in {sketch} is not a circle or an arc; pick a point or a circle",
                    pick.id
                )
            })?
        } else {
            pick.id.as_str()
        }
        .to_string();
        if spots
            .iter()
            .any(|spot| spot.sketch == *sketch && spot.point == point)
        {
            continue;
        }
        let place = from_inspect(inspect, sketch, &point, Source::Picked)?;
        spots.push(Spot {
            sketch: sketch.clone(),
            point,
            place,
        });
    }
    Ok(spots)
}

/// The spots grouped by sketch, in order, so one hole feature drills every point of a
/// sketch and follows it.
pub fn by_sketch(spots: &[Spot]) -> Vec<(&str, Vec<&str>)> {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for spot in spots {
        match groups.iter_mut().find(|(sketch, _)| *sketch == spot.sketch) {
            Some((_, points)) => points.push(&spot.point),
            None => groups.push((&spot.sketch, vec![&spot.point])),
        }
    }
    groups
}

/// The error when a cut into a part is asked for and no point is picked.
/// `cut` is what would be made and `alone` is the Create choice that needs no part.
pub fn nothing_selected(cut: &str, alone: &str) -> String {
    format!(
        "{FIELD}: pick the sketch points or circles where the {cut} go; or set Create to {alone}"
    )
}

/// Where each fastener goes: every picked point, or the origin alone when nothing is
/// picked.
pub fn resolve_each<R: Runner>(r: &mut R, input: &PlaceInput) -> Result<Vec<Placement>, String> {
    let spots = resolve_all(r, input)?;
    if spots.is_empty() {
        return Ok(vec![Placement::on_origin()]);
    }
    Ok(spots.into_iter().map(|spot| spot.place).collect())
}

fn from_inspect(
    inspect: &Value,
    sketch: &str,
    point_id: &str,
    source: Source,
) -> Result<Placement, String> {
    let point_id = curve_center(inspect, point_id).unwrap_or(point_id);
    let world = inspect
        .get("points")
        .and_then(Value::as_array)
        .and_then(|points| {
            points
                .iter()
                .find(|point| point.get("id").and_then(Value::as_str) == Some(point_id))
        })
        .and_then(|point| point.get("world"))
        .ok_or_else(|| format!("{FIELD}: no point or circle {point_id} in sketch {sketch}"))?;
    let point = xyz(world).ok_or_else(|| format!("{FIELD}: {point_id} has no position"))?;
    let plane = inspect.get("world_plane").ok_or_else(|| {
        "placement: update SolidCraft to a build exposing sketch.inspect world_plane".to_string()
    })?;
    let x_dir = xyz(plane.get("x_dir").unwrap_or(&Value::Null))
        .ok_or_else(|| format!("{FIELD}: {sketch} has no plane"))?;
    let y_dir = xyz(plane.get("y_dir").unwrap_or(&Value::Null))
        .ok_or_else(|| format!("{FIELD}: {sketch} has no plane"))?;
    let x_dir = normalize(x_dir).ok_or_else(|| format!("{FIELD}: {sketch} has no plane"))?;
    let y_dir = normalize(y_dir).ok_or_else(|| format!("{FIELD}: {sketch} has no plane"))?;
    let normal =
        normalize(cross(x_dir, y_dir)).ok_or_else(|| format!("{FIELD}: {sketch} has no plane"))?;
    Ok(Placement {
        point,
        normal,
        x_dir,
        y_dir,
        source,
    })
}

fn curve_center<'a>(inspect: &'a Value, id: &str) -> Option<&'a str> {
    inspect
        .get("curves")
        .and_then(Value::as_array)?
        .iter()
        .find(|curve| curve.get("id").and_then(Value::as_str) == Some(id))
        .and_then(|curve| curve.get("center"))
        .and_then(Value::as_str)
}

fn xyz(v: &Value) -> Option<[f64; 3]> {
    let nums = v.as_array()?;
    if nums.len() != 3 {
        return None;
    }
    Some([nums[0].as_f64()?, nums[1].as_f64()?, nums[2].as_f64()?])
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn opposite(v: [f64; 3]) -> [f64; 3] {
    scale(v, -1.0)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        None
    } else {
        Some(scale(v, 1.0 / len))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_origin_puts_the_tip_below_the_plane() {
        let place = Placement::on_origin();
        assert_eq!(place.tip(12.0), [0.0, 0.0, -12.0]);
        assert_eq!(place.along(5.2), [0.0, 0.0, 5.2]);
        assert_eq!(opposite(place.normal), [0.0, 0.0, -1.0]);
        assert_eq!(place.on_shank(12.0, 2.0, 1.5), [1.5, 0.0, -10.0]);
    }

    #[test]
    fn a_circle_centre_uses_the_sketch_normal() {
        let inspect = json!({
            "world_plane": {
                "origin": [0.0, 0.0, 8.0],
                "x_dir": [1.0, 0.0, 0.0],
                "y_dir": [0.0, 1.0, 0.0]
            },
            "points": [{
                "id": "c1.center",
                "world": [12.0, 10.0, 8.0]
            }],
            "curves": [{
                "id": "c1",
                "type": "circle",
                "center": "c1.center"
            }]
        });
        let place = from_inspect(&inspect, "Holes", "c1", Source::Picked).unwrap();
        assert_eq!(place.point, [12.0, 10.0, 8.0]);
        assert_eq!(place.normal, [0.0, 0.0, 1.0]);
        assert_eq!(place.tip(12.0), [12.0, 10.0, -4.0]);
    }

    #[test]
    fn a_vertical_sketch_points_its_normal_outward() {
        // The saved XY sketch is rotated through its occurrence into XZ.
        // World sketch x is +X, sketch y is -Z, so the normal is +Y.
        let inspect = json!({
            "plane": {
                "origin": [0.0, 0.0, 0.0],
                "x_dir": [1.0, 0.0, 0.0],
                "y_dir": [0.0, 1.0, 0.0]
            },
            "world_plane": {
                "origin": [0.0, 0.0, 0.0],
                "x_dir": [1.0, 0.0, 0.0],
                "y_dir": [0.0, 0.0, -1.0]
            },
            "points": [{"id": "p1", "world": [10.0, 0.0, 6.0]}]
        });
        let place = from_inspect(&inspect, "Side", "p1", Source::Picked).unwrap();
        assert_eq!(place.normal, [0.0, 1.0, 0.0]);
        assert_eq!(place.tip(5.0), [10.0, -5.0, 6.0]);
    }

    #[test]
    fn a_host_without_world_plane_is_refused_instead_of_using_local_axes() {
        let mut inspect = holes_inspect();
        let plane = inspect
            .as_object_mut()
            .unwrap()
            .remove("world_plane")
            .unwrap();
        inspect["plane"] = plane;
        assert!(from_inspect(&inspect, "Holes", "c1", Source::Picked)
            .unwrap_err()
            .contains("update SolidCraft"));
    }

    struct Rec {
        calls: Vec<(String, Value)>,
        inspect: Value,
    }

    impl Runner for Rec {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            self.calls.push((command.to_string(), args));
            Ok(match command {
                "sketch.inspect" => self.inspect.clone(),
                _ => Value::Null,
            })
        }
    }

    fn holes_inspect() -> Value {
        json!({
            "id": 7,
            "name": "Holes",
            "world_plane": {
                "origin": [0.0, 0.0, 8.0],
                "x_dir": [1.0, 0.0, 0.0],
                "y_dir": [0.0, 1.0, 0.0]
            },
            "points": [
                {"id": "c1.center", "world": [12.0, 10.0, 8.0]},
                {"id": "c2.center", "world": [30.0, 10.0, 8.0]},
                {"id": "l1.start", "world": [0.0, 0.0, 8.0]}
            ],
            "curves": [
                {"id": "c1", "center": "c1.center"},
                {"id": "c2", "center": "c2.center"},
                {"id": "l1", "type": "line"}
            ]
        })
    }

    fn rec() -> Rec {
        Rec {
            calls: vec![],
            inspect: holes_inspect(),
        }
    }

    #[test]
    fn nothing_picked_sits_on_the_origin_and_reads_nothing() {
        let mut rec = rec();
        let input = parse_input(&json!({})).unwrap();
        assert_eq!(input, PlaceInput::default());
        let places = resolve_each(&mut rec, &input).unwrap();
        assert_eq!(places, [Placement::on_origin()]);
        assert!(rec.calls.is_empty());
        assert_eq!(parse_input(&json!({"placement": []})).unwrap(), input);
    }

    #[test]
    fn a_picked_circle_is_its_centre_and_the_sketch_is_read_once() {
        let mut rec = rec();
        let input = parse_input(&json!({"placement": [
            {"type": "external_sketch_curve", "sketch": 7, "id": "c1"},
            {"type": "external_sketch_point", "sketch": 7, "id": "c2.center"},
            {"type": "external_sketch_point", "sketch": 7, "id": "c1.center"}
        ]}))
        .unwrap();
        let spots = resolve_all(&mut rec, &input).unwrap();
        // The circle and its own centre are one spot.
        assert_eq!(spots.len(), 2);
        assert_eq!(spots[0].point, "c1.center");
        assert_eq!(spots[0].sketch, "Holes");
        assert_eq!(spots[0].place.point, [12.0, 10.0, 8.0]);
        assert_eq!(spots[0].place.normal, [0.0, 0.0, 1.0]);
        assert_eq!(spots[0].place.source, Source::Picked);
        assert_eq!(spots[1].place.point, [30.0, 10.0, 8.0]);
        assert_eq!(
            rec.calls,
            [("sketch.inspect".to_string(), json!({"sketch": 7}))]
        );
    }

    #[test]
    fn a_program_may_name_the_sketch() {
        let mut rec = rec();
        let input = parse_input(&json!({"placement": [
            {"type": "external_sketch_point", "sketch": "Holes", "id": "c2.center"}
        ]}))
        .unwrap();
        let spots = resolve_all(&mut rec, &input).unwrap();
        assert_eq!(spots[0].sketch, "Holes");
        assert_eq!(rec.calls[0].1, json!({"sketch": "Holes"}));
    }

    #[test]
    fn a_picked_line_is_refused() {
        let mut rec = rec();
        let input = parse_input(&json!({"placement": [
            {"type": "external_sketch_curve", "sketch": 7, "id": "l1"}
        ]}))
        .unwrap();
        let err = resolve_all(&mut rec, &input).unwrap_err();
        assert_eq!(
            err,
            "placement: l1 in Holes is not a circle or an arc; pick a point or a circle"
        );
    }

    #[test]
    fn anything_but_a_sketch_pick_is_refused_against_the_field() {
        for bad in [
            json!({"placement": [{"type": "body", "name": "Plate"}]}),
            json!({"placement": [{"type": "edge", "body": "Plate", "index": 3}]}),
            json!({"placement": [{"type": "external_sketch_point", "id": "p1"}]}),
            json!({"placement": [{"type": "external_sketch_point", "sketch": 2}]}),
            json!({"placement": [{"type": "external_sketch_point", "sketch": -1, "id": "p1"}]}),
            json!({"placement": "Holes"}),
        ] {
            assert_eq!(parse_input(&bad).unwrap_err().field, "placement", "{bad}");
        }
        assert_eq!(
            parse_input(&json!({"placement": [{"type": "body", "name": "Plate"}]}))
                .unwrap_err()
                .reason,
            "pick sketch points or circles; a face, an edge or a body cannot place it"
        );
    }

    /// SolidCraft's dialog draws a selection field only with these keywords, and turns
    /// Run off for any other array. Every placing entry gets the same pick field.
    #[test]
    fn every_package_declares_the_placement_pick_field() {
        let plugins = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
        let mut checked = 0;
        for folder in std::fs::read_dir(plugins).unwrap() {
            let path = folder.unwrap().path().join("plugin.json");
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let manifest: Value = serde_json::from_str(&text).unwrap();
            for entry in manifest["runtime"]["entries"].as_array().unwrap() {
                let properties = entry["input_schema"]["properties"].as_object().unwrap();
                for (key, property) in properties {
                    if property["type"] == "array" {
                        assert_eq!(key, FIELD, "{}", path.display());
                    }
                }
                let field = properties[FIELD].as_object().unwrap();
                let mut keys: Vec<&str> = field.keys().map(String::as_str).collect();
                keys.sort();
                assert_eq!(
                    keys,
                    [
                        "description",
                        "maxItems",
                        "title",
                        "type",
                        "x-solidcraft-selection"
                    ],
                    "{}",
                    path.display()
                );
                assert_eq!(
                    field["x-solidcraft-selection"],
                    json!({"version": 1, "accept": ["sketch_point", "sketch_curve"]})
                );
                assert_eq!(field["maxItems"], json!(32));
                checked += 1;
            }
        }
        assert!(checked >= 8);
    }
}
