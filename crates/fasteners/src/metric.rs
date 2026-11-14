//! ISO metric designations: `M3`, `M8x1`, `M3.5x0.6`.
//!
//! A bare `M3` or `M3.5` takes the coarse pitch from ISO 261. A size that is not in that
//! list has to name its pitch. Other forms (`1/4-20 UNC`, `G 1/2`) are left for the host.

use plugin_kit::FieldError;

/// A parsed metric size. `major` and the nominal diameter are the same number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetricSize {
    pub major: f64,
    pub pitch: f64,
}

/// Coarse pitches from ISO 261, first, second and third choice: every size SolidCraft's
/// thread table knows by name. `(nominal diameter, pitch)` in millimetres.
const COARSE: &[(f64, f64)] = &[
    (1.0, 0.25),
    (1.1, 0.25),
    (1.2, 0.25),
    (1.4, 0.3),
    (1.6, 0.35),
    (1.8, 0.35),
    (2.0, 0.4),
    (2.2, 0.45),
    (2.5, 0.45),
    (3.0, 0.5),
    (3.5, 0.6),
    (4.0, 0.7),
    (4.5, 0.75),
    (5.0, 0.8),
    (6.0, 1.0),
    (7.0, 1.0),
    (8.0, 1.25),
    (10.0, 1.5),
    (12.0, 1.75),
    (14.0, 2.0),
    (16.0, 2.0),
    (18.0, 2.5),
    (20.0, 2.5),
    (22.0, 2.5),
    (24.0, 3.0),
    (27.0, 3.0),
    (30.0, 3.5),
    (33.0, 3.5),
    (36.0, 4.0),
    (39.0, 4.0),
    (42.0, 4.5),
    (45.0, 4.5),
    (48.0, 5.0),
    (52.0, 5.0),
    (56.0, 5.5),
    (60.0, 5.5),
    (64.0, 6.0),
];

/// `Some` for an `M` designation, `None` when the text is some other thread form.
pub fn parse_metric(designation: &str) -> Result<Option<MetricSize>, FieldError> {
    let Some(rest) = designation.trim().strip_prefix(['M', 'm']) else {
        return Ok(None);
    };
    if rest.is_empty() {
        return Err(FieldError::new(
            "size",
            "must name a size, as in M3 or M8x1",
        ));
    }
    let (major_text, pitch_text) = split_pitch(rest)?;
    let major = millimeters(major_text)
        .ok_or_else(|| FieldError::new("size", "must name a size, as in M3 or M8x1"))?;
    if !(major.is_finite() && major > 0.0 && major <= 100.0) {
        return Err(FieldError::new("size", "must be between 0 and 100 mm"));
    }
    let pitch = match pitch_text {
        Some(text) => {
            let pitch = millimeters(text)
                .ok_or_else(|| FieldError::new("size", "pitch must be a number, as in M8x1"))?;
            if !(pitch.is_finite() && pitch > 0.0 && pitch < major) {
                return Err(FieldError::new(
                    "size",
                    "pitch must be shorter than the diameter",
                ));
            }
            pitch
        }
        None => coarse_pitch(major)
            .ok_or_else(|| FieldError::new("size", "give the pitch, as in M7.5x1"))?,
    };
    Ok(Some(MetricSize { major, pitch }))
}

pub fn coarse_pitch(nominal: f64) -> Option<f64> {
    COARSE
        .iter()
        .find(|(size, _)| same_mm(*size, nominal))
        .map(|(_, pitch)| *pitch)
}

pub fn same_mm(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn split_pitch(rest: &str) -> Result<(&str, Option<&str>), FieldError> {
    let mut parts = rest.split(['x', 'X', '×']);
    let major = parts.next().unwrap_or("");
    let pitch = parts.next();
    if parts.next().is_some() {
        return Err(FieldError::new("size", "must be a size such as M3 or M8x1"));
    }
    Ok((major, pitch))
}

fn millimeters(text: &str) -> Option<f64> {
    if text.is_empty()
        || text.starts_with('.')
        || text.ends_with('.')
        || text.bytes().filter(|c| *c == b'.').count() > 1
        || !text.bytes().all(|c| c.is_ascii_digit() || c == b'.')
    {
        return None;
    }
    text.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m3_is_the_coarse_pitch() {
        let size = parse_metric("M3").unwrap().unwrap();
        assert_eq!(size.major, 3.0);
        assert_eq!(size.pitch, 0.5);
    }

    #[test]
    fn an_explicit_pitch_wins() {
        let size = parse_metric("M8x1").unwrap().unwrap();
        assert_eq!(size.major, 8.0);
        assert_eq!(size.pitch, 1.0);
    }

    #[test]
    fn an_odd_diameter_needs_a_pitch() {
        assert_eq!(parse_metric("M3.5").unwrap().unwrap().pitch, 0.6);
        assert_eq!(parse_metric("M7").unwrap().unwrap().pitch, 1.0);
        assert_eq!(parse_metric("M7.5").unwrap_err().field, "size");
        let size = parse_metric("M3.5x0.6").unwrap().unwrap();
        assert_eq!(size.pitch, 0.6);
    }

    #[test]
    fn other_forms_are_not_metric() {
        assert!(parse_metric("1/4-20 UNC").unwrap().is_none());
    }
}
