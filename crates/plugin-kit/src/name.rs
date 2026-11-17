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

/// Reserve a whole generated part's naming prefix, including its bodies (`part …`)
/// and features (`part: …`). Renaming a component or body leaves its features behind.
pub fn reserve_part(taken: &mut Vec<String>, base: &str, bodies: &[&str]) -> String {
    loop {
        let candidate = reserve(taken, base);
        let derived = |name: &str, separator: u8| {
            name.get(..candidate.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&candidate))
                && name.as_bytes().get(candidate.len()) == Some(&separator)
        };
        // A layout sketch or assembly can start with the product's name too.
        // Only bodies own space-separated suffixes such as "Seal front".
        let occupied = taken.iter().any(|name| derived(name, b':'))
            || bodies.iter().any(|name| derived(name, b' '));
        if !occupied {
            return candidate;
        }
    }
}

/// Seed a run's name reservations from every component: sketches and construction
/// planes share a document-wide namespace, including features outside the active part.
pub fn feature_names<R: Runner>(r: &mut R) -> Result<Vec<String>, String> {
    let mut listed = r.run("component.list", json!({}))?;
    let components: Vec<crate::content::Component> =
        serde_json::from_value(listed["components"].take())
            .map_err(|e| format!("cannot inspect components for feature names: {e}"))?;
    Ok(components
        .iter()
        .flat_map(|c| c.feature_names().map(str::to_string))
        .collect())
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
    fn part_reservations_cover_old_features_and_multibody_names() {
        let mut taken = vec![
            "NUT M6: Hex profile".into(),
            "Nut M6 (2): Thread".into(),
            "Bearing 625 2RS Inner ring".into(),
            "Nut M60: Hex profile".into(),
        ];
        let bodies = ["Bearing 625 2RS Inner ring"];
        assert_eq!(reserve_part(&mut taken, "Nut M6", &bodies), "Nut M6 (3)");
        assert_eq!(reserve_part(&mut taken, "Nut M6", &bodies), "Nut M6 (4)");
        assert_eq!(
            reserve_part(&mut taken, "Bearing 625 2RS", &bodies),
            "Bearing 625 2RS (2)"
        );
        assert_eq!(reserve_part(&mut taken, "Nut M", &bodies), "Nut M");
        taken.extend([
            "Solid V Wheel assembly".into(),
            "Solid V Wheel layout".into(),
        ]);
        assert_eq!(
            reserve_part(&mut taken, "Solid V Wheel", &bodies),
            "Solid V Wheel"
        );
    }

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
