//! A pocketed nut follows the bolt-hole end at the pocket floor. The pocket wall
//! supplies its turn, so moving or rotating the source sketch carries the hex too.

use plugin_kit::{Placement, Runner};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::nut::Pocket;

pub struct Seat {
    body: String,
    floor: String,
    bore: String,
    wall: String,
}

#[derive(Deserialize)]
struct Faces {
    faces: Vec<Face>,
}

#[derive(Deserialize)]
struct Face {
    name: String,
    centroid: [f64; 3],
    plane_normal: Option<[f64; 3]>,
    cylinder: Option<Cylinder>,
}

#[derive(Deserialize)]
struct Cylinder {
    origin: [f64; 3],
    axis: [f64; 3],
    radius: f64,
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

fn same(a: [f64; 3], b: [f64; 3]) -> bool {
    a.into_iter().zip(b).all(|(a, b)| near(a, b))
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

fn unique<'a>(mut faces: impl Iterator<Item = &'a Face>) -> Option<&'a Face> {
    let first = faces.next()?;
    faces.next().is_none().then_some(first)
}

impl Seat {
    /// Take persistent names from the actual faces. Require a complete, centered
    /// pocket; a missing or ambiguous seat must not silently leave a loose nut.
    pub fn find_all<R: Runner>(
        r: &mut R,
        bodies: &[String],
        places: &[Placement],
        pocket: &Pocket,
        major: f64,
    ) -> Result<Vec<Self>, String> {
        let radius = fasteners::clearance::medium(major)
            .ok_or("sketch_point: the pocket has no bolt clearance size")?
            / 2.0;
        let faces = bodies
            .iter()
            .map(|body| {
                let reply = r.run("model.faces", json!({"body": body}))?;
                let faces: Faces = serde_json::from_value(reply)
                    .map_err(|e| format!("model.faces: cannot read the nut pocket faces: {e}"))?;
                Ok((body.clone(), faces))
            })
            .collect::<Result<Vec<_>, String>>()?;
        places
            .iter()
            .map(|place| Self::find(&faces, place, pocket, radius))
            .collect()
    }

    fn find(
        bodies: &[(String, Faces)],
        place: &Placement,
        pocket: &Pocket,
        radius: f64,
    ) -> Result<Self, String> {
        let wall_center = std::array::from_fn(|i| {
            place.point[i] + place.normal[i] * pocket.depth / 2.0
                - place.y_dir[i] * pocket.across_flats / 2.0
        });
        let mut found = None;
        for (body, faces) in bodies {
            let floor = unique(faces.faces.iter().filter(|f| {
                f.plane_normal.is_some_and(|n| same(n, place.normal))
                    && same(f.centroid, place.point)
            }));
            let wall = unique(faces.faces.iter().filter(|f| {
                f.plane_normal.is_some_and(|n| same(n, place.y_dir))
                    && same(f.centroid, wall_center)
            }));
            let bore = unique(faces.faces.iter().filter(|f| {
                f.cylinder.as_ref().is_some_and(|c| {
                    let delta = std::array::from_fn(|i| c.origin[i] - place.point[i]);
                    let along = dot(delta, place.normal);
                    near(c.radius, radius)
                        && near(dot(c.axis, place.normal).abs(), 1.0)
                        && same(delta, place.normal.map(|n| n * along))
                })
            }));
            if let (Some(floor), Some(wall), Some(bore)) = (floor, wall, bore) {
                if found.is_some() {
                    return Err("sketch_point: more than one part has this nut seat".into());
                }
                found = Some(Self {
                    body: body.clone(),
                    floor: floor.name.clone(),
                    bore: bore.name.clone(),
                    wall: wall.name.clone(),
                });
            }
        }
        found.ok_or_else(|| {
            "sketch_point: the nut needs a complete pocket floor, wall and bolt hole to stay attached; move the point clear of edges or overlapping cuts".into()
        })
    }

    pub fn attach<R: Runner>(
        &self,
        r: &mut R,
        occurrence: u64,
        place: &Placement,
        designation: &str,
    ) -> Result<(), String> {
        let reply = r.run(
            "joint.create",
            json!({
                "type": "rigid", "name": format!("Nut {designation}: Seat {occurrence}"),
                "a": {"body": self.body, "circle": self.bore, "end": self.floor, "x": self.wall},
                "b": {"occurrence": occurrence, "point": place.point,
                      "z": place.normal, "x": place.y_dir},
                "flip": true, "angle": 0,
            }),
        )?;
        let joint = reply
            .get("joint")
            .and_then(Value::as_u64)
            .ok_or("joint.create did not return the nut seat id")?;
        for key in ["errors", "conflicts"] {
            let problems = reply
                .get(key)
                .and_then(Value::as_array)
                .ok_or_else(|| format!("joint.create did not report nut seat {key}"))?;
            if let Some(problem) = problems.iter().find(|p| p["joint"].as_u64() == Some(joint)) {
                return Err(format!("joint.create: nut seat {key}: {problem}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pocket() -> Pocket {
        crate::nut::parse(&json!({"size": "M6", "create": "Nut and pocket"}))
            .unwrap()
            .pocket
            .unwrap()
    }

    fn faces() -> Faces {
        serde_json::from_value(json!({"faces": [
            {"name": "floor", "centroid": [0, 0, 0], "plane_normal": [0, 0, 1]},
            {"name": "wall", "centroid": [0, -5.2, 2.6], "plane_normal": [0, 1, 0]},
            {"name": "bore", "centroid": [0, 0, -2],
             "cylinder": {"origin": [0, 0, -100], "axis": [0, 0, -1], "radius": 3.3}},
        ]}))
        .unwrap()
    }

    #[test]
    fn a_missing_or_ambiguous_face_does_not_leave_a_loose_nut() {
        for index in 0..3 {
            let mut missing = faces();
            missing.faces.remove(index);
            assert!(Seat::find(
                &[("Plate".into(), missing)],
                &Placement::on_origin(),
                &pocket(),
                3.3
            )
            .is_err());
            let mut ambiguous = faces();
            ambiguous.faces.push(faces().faces.remove(index));
            assert!(Seat::find(
                &[("Plate".into(), ambiguous)],
                &Placement::on_origin(),
                &pocket(),
                3.3
            )
            .is_err());
        }
        assert!(Seat::find(
            &[("A".into(), faces()), ("B".into(), faces())],
            &Placement::on_origin(),
            &pocket(),
            3.3
        )
        .is_err());
    }

    #[test]
    fn the_hole_axis_and_size_must_match_the_seat() {
        let mut wrong_axis = faces();
        wrong_axis.faces[2].cylinder.as_mut().unwrap().origin[0] = 1.0;
        assert!(Seat::find(
            &[("Plate".into(), wrong_axis)],
            &Placement::on_origin(),
            &pocket(),
            3.3
        )
        .is_err());
        assert!(Seat::find(
            &[("Plate".into(), faces())],
            &Placement::on_origin(),
            &pocket(),
            4.5
        )
        .is_err());
    }

    #[test]
    fn attachment_reports_its_solve_failure_without_blame_for_unrelated_joints() {
        struct Host(Value);
        impl Runner for Host {
            fn run(&mut self, _: &str, _: Value) -> Result<Value, String> {
                Ok(self.0.clone())
            }
        }
        let seat = Seat::find(
            &[("Plate".into(), faces())],
            &Placement::on_origin(),
            &pocket(),
            3.3,
        )
        .unwrap();
        for key in ["errors", "conflicts"] {
            let mut reply = json!({"joint": 8, "errors": [], "conflicts": []});
            reply[key] = json!([{"joint": 8, "why": "lost floor"}]);
            assert!(seat
                .attach(&mut Host(reply.clone()), 3, &Placement::on_origin(), "M6")
                .unwrap_err()
                .contains("lost floor"));
            reply[key][0]["joint"] = json!(9);
            seat.attach(&mut Host(reply), 3, &Placement::on_origin(), "M6")
                .unwrap();
        }
        assert!(seat
            .attach(&mut Host(json!({})), 3, &Placement::on_origin(), "M6")
            .is_err());
    }
}
