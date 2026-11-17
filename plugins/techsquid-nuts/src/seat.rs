//! A pocketed nut follows the bolt-hole end at the pocket floor. The pocket wall
//! supplies its turn, so moving or rotating the source sketch carries the hex too.

use fasteners::seat::{near, same, unique, Body, Faces, Seat};
use plugin_kit::{Placement, Runner};

use crate::nut::Pocket;

/// Take persistent names from the actual faces. Require a complete, centered
/// pocket; a missing or ambiguous seat must not silently leave a loose nut.
pub fn find_all<R: Runner>(
    r: &mut R,
    bodies: &[String],
    places: &[Placement],
    pocket: &Pocket,
    major: f64,
) -> Result<Vec<Seat>, String> {
    let radius = fasteners::clearance::medium(major)
        .ok_or("sketch_point: the pocket has no bolt clearance size")?
        / 2.0;
    let faces = Faces::read_all(r, bodies, places, true)?;
    places
        .iter()
        .map(|place| find(&faces, place, pocket, radius))
        .collect()
}

fn find(
    bodies: &[(Body, Faces)],
    place: &Placement,
    pocket: &Pocket,
    radius: f64,
) -> Result<Seat, String> {
    let wall_center = std::array::from_fn(|i| {
        place.point[i] + place.normal[i] * pocket.depth / 2.0
            - place.y_dir[i] * pocket.across_flats / 2.0
    });
    let mut found = None;
    for (body, faces) in bodies {
        let floor = unique(faces.faces.iter().filter(|f| {
            f.plane_normal.is_some_and(|n| same(n, place.normal)) && same(f.centroid, place.point)
        }));
        let wall = unique(faces.faces.iter().filter(|f| {
            f.plane_normal.is_some_and(|n| same(n, place.y_dir)) && same(f.centroid, wall_center)
        }));
        let bore = unique(faces.faces.iter().filter(|f| {
            f.cylinder
                .as_ref()
                .is_some_and(|c| near(c.radius, radius) && c.on_axis(place))
        }));
        if let (Some(floor), Some(wall), Some(bore)) = (floor, wall, bore) {
            if found.is_some() {
                return Err("sketch_point: more than one part has this nut seat".into());
            }
            found = Some(Seat::Hole {
                body: body.clone(),
                end: floor.name.clone(),
                bore: bore.name.clone(),
                turn: Some(wall.name.clone()),
            });
        }
    }
    found.ok_or_else(|| {
            "sketch_point: the nut needs a complete pocket floor, wall and bolt hole to stay attached; move the point clear of edges or overlapping cuts".into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
            assert!(find(
                &[(
                    Body {
                        definition: "Plate".into(),
                        path: vec![]
                    },
                    missing
                )],
                &Placement::on_origin(),
                &pocket(),
                3.3
            )
            .is_err());
            let mut ambiguous = faces();
            ambiguous.faces.push(faces().faces.remove(index));
            assert!(find(
                &[(
                    Body {
                        definition: "Plate".into(),
                        path: vec![]
                    },
                    ambiguous
                )],
                &Placement::on_origin(),
                &pocket(),
                3.3
            )
            .is_err());
        }
        assert!(find(
            &[
                (
                    Body {
                        definition: "A".into(),
                        path: vec![]
                    },
                    faces()
                ),
                (
                    Body {
                        definition: "B".into(),
                        path: vec![]
                    },
                    faces()
                )
            ],
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
        assert!(find(
            &[(
                Body {
                    definition: "Plate".into(),
                    path: vec![]
                },
                wrong_axis
            )],
            &Placement::on_origin(),
            &pocket(),
            3.3
        )
        .is_err());
        assert!(find(
            &[(
                Body {
                    definition: "Plate".into(),
                    path: vec![]
                },
                faces()
            )],
            &Placement::on_origin(),
            &pocket(),
            4.5
        )
        .is_err());
    }
}
