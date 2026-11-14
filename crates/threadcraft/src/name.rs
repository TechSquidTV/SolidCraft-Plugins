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
