//! Where a part sits.
//!
//! The point is a sketch point, or the centre of a circle in a sketch. The sketch plane is
//! the surface. Its normal points to the outside. For a screw, the head stands on that side
//! and the shank runs the other way, and the top of the shank, where the head begins, is the
//! point itself.
//!
//! With no sketch and no selection, the point is the origin and the normal is +Z, so the
//! head stands on the XY plane.

use serde_json::Value;

use crate::{FieldError, Runner};

/// How the point was chosen. The reply says so, because the origin is also what a
/// forgotten selection gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Origin,
    Selection,
    Named,
}

impl Source {
    pub fn words(self) -> &'static str {
        match self {
            Source::Origin => "on the origin, because nothing was selected",
            Source::Selection => "on the selected sketch point",
            Source::Named => "on the named sketch point",
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

/// Names from the dialog. Both empty means the current selection, or the origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceInput {
    pub sketch: Option<String>,
    pub point: Option<String>,
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

pub fn parse_input(v: &Value) -> Result<PlaceInput, FieldError> {
    let sketch = optional_text(v, "sketch")?;
    let point = optional_text(v, "sketch_point")?;
    match (&sketch, &point) {
        (Some(_), None) => Err(FieldError::new(
            "sketch_point",
            "name a point or a circle in the sketch, such as c1.center",
        )),
        (None, Some(_)) => Err(FieldError::new(
            "sketch",
            "name the sketch that holds the point",
        )),
        _ => Ok(PlaceInput { sketch, point }),
    }
}

/// One placed point, with the sketch and the point id a host command can name.
#[derive(Clone, Debug, PartialEq)]
pub struct Spot {
    pub sketch: String,
    /// The point id itself: a circle resolves to its centre.
    pub point: String,
    pub place: Placement,
}

/// Every point the dialog names or the selection holds, in order, each once. A named
/// `sketch_point` may list several ids with commas. Nothing named and nothing selected
/// is an empty list, which the caller may put on the origin.
pub fn resolve_all<R: Runner>(r: &mut R, input: &PlaceInput) -> Result<Vec<Spot>, String> {
    let (wanted, source) = match (&input.sketch, &input.point) {
        (Some(sketch), Some(points)) => (
            points
                .split(',')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(|id| (sketch.clone(), id.to_string()))
                .collect::<Vec<_>>(),
            Source::Named,
        ),
        (None, None) => (selected_sketch_points(r)?, Source::Selection),
        _ => return Err("sketch_point: name the sketch and the point, or leave both empty".into()),
    };
    let mut inspected: Vec<(String, Value)> = Vec::new();
    let mut spots: Vec<Spot> = Vec::new();
    for (sketch, id) in wanted {
        if !inspected.iter().any(|(name, _)| *name == sketch) {
            let inspect = r
                .run("sketch.inspect", serde_json::json!({"sketch": sketch}))
                .map_err(|err| format!("sketch: {err}"))?;
            inspected.push((sketch.clone(), inspect));
        }
        let inspect = &inspected
            .iter()
            .find(|(name, _)| *name == sketch)
            .expect("inspected above")
            .1;
        let point = curve_center(inspect, &id).unwrap_or(&id).to_string();
        if spots
            .iter()
            .any(|spot| spot.sketch == sketch && spot.point == point)
        {
            continue;
        }
        let place = from_inspect(inspect, &sketch, &id, source)?;
        spots.push(Spot {
            sketch,
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

/// The error when a cut into a part is asked for and no point is named or selected.
/// `cut` is what would be made and `alone` is the Create choice that needs no part.
pub fn nothing_selected(cut: &str, alone: &str) -> String {
    format!(
        "sketch_point: select the sketch points or circles where the {cut} go, then run this again; or set Create to {alone}"
    )
}

/// Where each fastener goes: every named or selected point, or the origin alone when
/// nothing is named and nothing is selected.
pub fn resolve_each<R: Runner>(r: &mut R, input: &PlaceInput) -> Result<Vec<Placement>, String> {
    let spots = resolve_all(r, input)?;
    if spots.is_empty() {
        return Ok(vec![Placement::on_origin()]);
    }
    Ok(spots.into_iter().map(|spot| spot.place).collect())
}

fn selected_sketch_points<R: Runner>(r: &mut R) -> Result<Vec<(String, String)>, String> {
    let selected = r
        .run("selection.get", serde_json::json!({"detail": true}))
        .map_err(|err| format!("sketch_point: {err}"))?;
    let items = selected
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| "sketch_point: the selection could not be read".to_string())?;
    let found: Vec<(String, String)> = items.iter().filter_map(sketch_item).collect();
    if found.is_empty() && !items.is_empty() {
        return Err("sketch_point: select a sketch point or a sketch circle; a face, an edge or a body cannot place it".into());
    }
    Ok(found)
}

fn optional_text(v: &Value, key: &str) -> Result<Option<String>, FieldError> {
    Ok(crate::input::optional_text(v, key, "must be a string")?.map(str::to_string))
}

fn sketch_item(item: &Value) -> Option<(String, String)> {
    let kind = item.get("kind").and_then(Value::as_str).unwrap_or("");
    let entity = item.get("entity").and_then(Value::as_str).unwrap_or("");
    let sketch = item.get("sketch_name").and_then(Value::as_str)?;
    let id = item.get("id").and_then(Value::as_str)?;
    let point = kind.contains("sketch_point") || entity == "point";
    let round = entity == "circle" || entity == "arc";
    if point || round {
        Some((sketch.to_string(), id.to_string()))
    } else {
        None
    }
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
        .ok_or_else(|| format!("sketch_point: no point or circle {point_id} in sketch {sketch}"))?;
    let point = xyz(world).ok_or_else(|| format!("sketch_point: {point_id} has no position"))?;
    let plane = inspect
        .get("plane")
        .ok_or_else(|| format!("sketch: {sketch} has no plane"))?;
    let x_dir = xyz(plane.get("x_dir").unwrap_or(&Value::Null))
        .ok_or_else(|| format!("sketch: {sketch} has no plane"))?;
    let y_dir = xyz(plane.get("y_dir").unwrap_or(&Value::Null))
        .ok_or_else(|| format!("sketch: {sketch} has no plane"))?;
    let x_dir = normalize(x_dir).ok_or_else(|| format!("sketch: {sketch} has no plane"))?;
    let y_dir = normalize(y_dir).ok_or_else(|| format!("sketch: {sketch} has no plane"))?;
    let normal =
        normalize(cross(x_dir, y_dir)).ok_or_else(|| format!("sketch: {sketch} has no plane"))?;
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
            "plane": {
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
        let place = from_inspect(&inspect, "Holes", "c1", Source::Named).unwrap();
        assert_eq!(place.point, [12.0, 10.0, 8.0]);
        assert_eq!(place.normal, [0.0, 0.0, 1.0]);
        assert_eq!(place.tip(12.0), [12.0, 10.0, -4.0]);
    }

    #[test]
    fn a_vertical_sketch_points_its_normal_outward() {
        // XZ plane: sketch x is +X, sketch y is -Z, so the normal is +Y.
        let inspect = json!({
            "plane": {
                "origin": [0.0, 0.0, 0.0],
                "x_dir": [1.0, 0.0, 0.0],
                "y_dir": [0.0, 0.0, -1.0]
            },
            "points": [{"id": "p1", "world": [10.0, 0.0, 6.0]}]
        });
        let place = from_inspect(&inspect, "Side", "p1", Source::Named).unwrap();
        assert_eq!(place.normal, [0.0, 1.0, 0.0]);
        assert_eq!(place.tip(5.0), [10.0, -5.0, 6.0]);
    }

    struct Rec {
        calls: Vec<String>,
        selection: Value,
        inspect: Value,
    }

    impl Runner for Rec {
        fn run(&mut self, command: &str, _args: Value) -> Result<Value, String> {
            self.calls.push(command.to_string());
            Ok(match command {
                "selection.get" => self.selection.clone(),
                "sketch.inspect" => self.inspect.clone(),
                _ => Value::Null,
            })
        }
    }

    /// The first placement, as a single fastener takes it.
    fn resolve(r: &mut Rec, input: &PlaceInput) -> Result<Placement, String> {
        resolve_each(r, input).map(|places| places[0])
    }

    fn holes_inspect() -> Value {
        json!({
            "plane": {
                "origin": [0.0, 0.0, 8.0],
                "x_dir": [1.0, 0.0, 0.0],
                "y_dir": [0.0, 1.0, 0.0]
            },
            "points": [{"id": "c1.center", "world": [12.0, 10.0, 8.0]}],
            "curves": [{"id": "c1", "center": "c1.center"}]
        })
    }

    #[test]
    fn an_empty_selection_sits_on_the_origin() {
        let mut rec = Rec {
            calls: vec![],
            selection: json!({"count": 0, "items": []}),
            inspect: Value::Null,
        };
        let place = resolve(
            &mut rec,
            &PlaceInput {
                sketch: None,
                point: None,
            },
        )
        .unwrap();
        assert_eq!(place, Placement::on_origin());
        assert_eq!(rec.calls, ["selection.get"]);
    }

    #[test]
    fn a_selected_circle_is_read_from_its_sketch() {
        let mut rec = Rec {
            calls: vec![],
            selection: json!({
                "count": 1,
                "items": [{
                    "kind": "external_sketch_curve",
                    "entity": "circle",
                    "id": "c1",
                    "sketch_name": "Holes"
                }]
            }),
            inspect: holes_inspect(),
        };
        let place = resolve(
            &mut rec,
            &PlaceInput {
                sketch: None,
                point: None,
            },
        )
        .unwrap();
        assert_eq!(place.point, [12.0, 10.0, 8.0]);
        assert_eq!(place.normal, [0.0, 0.0, 1.0]);
        assert_eq!(place.source, Source::Selection);
        assert_eq!(rec.calls, ["selection.get", "sketch.inspect"]);
    }

    #[test]
    fn every_selected_circle_is_a_spot_and_the_sketch_is_read_once() {
        let mut rec = Rec {
            calls: vec![],
            selection: json!({
                "count": 3,
                "items": [
                    {"kind": "sketch_curve", "entity": "circle", "id": "c1", "sketch_name": "Holes"},
                    {"kind": "sketch_point", "entity": "point", "id": "c2.center", "sketch_name": "Holes"},
                    {"kind": "sketch_point", "entity": "point", "id": "c1.center", "sketch_name": "Holes"}
                ]
            }),
            inspect: json!({
                "plane": {"origin": [0.0, 0.0, 8.0], "x_dir": [1.0, 0.0, 0.0], "y_dir": [0.0, 1.0, 0.0]},
                "points": [
                    {"id": "c1.center", "world": [12.0, 10.0, 8.0]},
                    {"id": "c2.center", "world": [30.0, 10.0, 8.0]}
                ],
                "curves": [{"id": "c1", "center": "c1.center"}, {"id": "c2", "center": "c2.center"}]
            }),
        };
        let none = PlaceInput {
            sketch: None,
            point: None,
        };
        let spots = resolve_all(&mut rec, &none).unwrap();
        // The circle and its own centre are one spot.
        assert_eq!(spots.len(), 2);
        assert_eq!(spots[0].point, "c1.center");
        assert_eq!(spots[0].sketch, "Holes");
        assert_eq!(spots[1].place.point, [30.0, 10.0, 8.0]);
        assert_eq!(rec.calls, ["selection.get", "sketch.inspect"]);

        let named = PlaceInput {
            sketch: Some("Holes".into()),
            point: Some("c2, c1".into()),
        };
        let spots = resolve_all(&mut rec, &named).unwrap();
        assert_eq!(spots[0].point, "c2.center");
        assert_eq!(spots[1].place.source, Source::Named);

        rec.selection = json!({"count": 0, "items": []});
        assert!(resolve_all(&mut rec, &none).unwrap().is_empty());
    }

    #[test]
    fn a_selected_body_is_refused() {
        let mut rec = Rec {
            calls: vec![],
            selection: json!({
                "count": 1,
                "items": [{"kind": "body", "entity": "body", "id": "Plate"}]
            }),
            inspect: Value::Null,
        };
        let err = resolve(
            &mut rec,
            &PlaceInput {
                sketch: None,
                point: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, "sketch_point: select a sketch point or a sketch circle; a face, an edge or a body cannot place it");
    }

    #[test]
    fn one_name_without_the_other_is_refused() {
        assert_eq!(
            parse_input(&json!({"sketch": "Holes"})).unwrap_err().field,
            "sketch_point"
        );
        assert_eq!(
            parse_input(&json!({"sketch_point": "c1.center"}))
                .unwrap_err()
                .field,
            "sketch"
        );
        assert_eq!(
            parse_input(&json!({})).unwrap(),
            PlaceInput {
                sketch: None,
                point: None
            }
        );
    }
}
