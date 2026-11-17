//! Names for the bodies, features and sketches the plugins make.
//!
//! A body name starts with what the part is, then its size: `Screw M3 x 12 socket`,
//! `Insert M3 x 5.7`, `Nut M3`. A body list then sorts by kind and size. A feature or
//! sketch of one part starts with that part's name, `Screw M3 x 12 socket: Head`, so a
//! timeline with several screws reads as one run of rows per screw. A hole feature stays
//! in the part after the hardware is gone, so it names its own kind and size:
//! `Clearance hole M3`.
//!
//! Names use a plain `x`, not `×`, because people and agents type them back into fields.

use crate::Runner;
use serde_json::json;

/// Reserve `base`, then `base (2)`, `base (3)`, etc. Match ASCII case-insensitively
/// like the host's sketch lookup. Reserve immediately so a batch cannot reuse a name.
pub fn reserve(taken: &mut Vec<String>, base: &str) -> String {
    let mut name = base.to_string();
    let mut n = 1;
    while taken
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(&name))
    {
        n += 1;
        name = format!("{base} ({n})");
    }
    taken.push(name.clone());
    name
}

/// Seed a run's name reservations from every component: sketches and construction
/// planes share a document-wide namespace, including features outside the active part.
pub fn feature_names<R: Runner>(r: &mut R) -> Result<Vec<String>, String> {
    let listed = r.run("component.list", json!({}))?;
    let components = listed["components"]
        .as_array()
        .ok_or("component.list did not return components for feature names")?;
    let mut names = Vec::new();
    for component in components {
        for key in ["features", "sketches", "construction"] {
            let entries = component[key]
                .as_array()
                .ok_or_else(|| format!("component.list did not return {key} for feature names"))?;
            for entry in entries {
                names.push(
                    entry
                        .as_str()
                        .ok_or("component.list returned an invalid feature name")?
                        .to_string(),
                );
            }
        }
    }
    Ok(names)
}

/// A length as a name shows it: at most two decimals and no trailing zeros, so `12.0` is
/// `12` and `5.70` is `5.7`.
pub fn length(mm: f64) -> String {
    let rounded = (mm * 100.0).round() / 100.0;
    // `-0` would read oddly; a length is never below zero here.
    format!("{}", rounded + 0.0)
}

/// A thread with a length, the way a size is written on a parts list: `M3 x 12`,
/// `M8x1 x 20`, `#4-40 x 5.7`. The spaces keep a fine pitch apart from the length.
pub fn sized(designation: &str, mm: f64) -> String {
    format!("{designation} x {}", length(mm))
}

/// A feature or sketch that belongs to the part named `part`: `Screw M3 x 12 socket: Head`.
pub fn part(part: &str, feature: &str) -> String {
    format!("{part}: {feature}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservations_skip_existing_case_variants_and_batch_names() {
        let mut taken = vec![
            "NUT POCKET M6 PROFILE".into(),
            "Nut pocket M6 profile (3)".into(),
        ];
        assert_eq!(
            reserve(&mut taken, "Nut pocket M6 profile"),
            "Nut pocket M6 profile (2)"
        );
        assert_eq!(
            reserve(&mut taken, "Nut pocket M6 profile"),
            "Nut pocket M6 profile (4)"
        );
    }

    #[test]
    fn lengths_drop_trailing_zeros() {
        assert_eq!(length(12.0), "12");
        assert_eq!(length(5.7), "5.7");
        assert_eq!(length(12.70), "12.7");
        assert_eq!(length(0.125), "0.13");
        assert_eq!(length(2.0 / 3.0), "0.67");
    }

    #[test]
    fn a_size_keeps_the_pitch_apart_from_the_length() {
        assert_eq!(sized("M3", 12.0), "M3 x 12");
        assert_eq!(sized("M8x1", 20.0), "M8x1 x 20");
        assert_eq!(sized("#4-40", 5.7), "#4-40 x 5.7");
    }

    #[test]
    fn a_feature_starts_with_its_part() {
        assert_eq!(part("Nut M3", "Thread"), "Nut M3: Thread");
    }
}
