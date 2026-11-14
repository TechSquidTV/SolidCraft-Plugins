//! CNC Kitchen heat-set inserts. Sizes are millimetres.
//!
//! Source: CNC Kitchen, "Heat-Set Insert Dimensions & Design Guidelines" (PDF),
//! <https://www.cnckitchen.com/s/CNCKitchen_Heat-Set-Insert-Dimensions-and-Design-Guidelines.pdf>,
//! the metric and UN tables (L, D1, D2 the hole, W). `M1.6 × 2.5` and the short M6 and
//! M8 are not in that table.
//! The glue-in holes are the stepped-bore table on
//! <https://www.cnckitchen.com/blog/tips-and-tricks-for-heat-set-inserts>.
//! `rows_match_cnc_kitchen` repeats both.
//!
//! The dialog lists these rows as Size. Each one names a thread, its major diameter,
//! and the plastic hole that row is meant for. `diameter` is D1, `hole` is the plastic
//! hole D2, and `wall` is the minimum wall beside that hole. A blind hole is at least
//! the insert length plus 1 mm. `#2-56` is drilled as a plain bore because SolidCraft's
//! tap catalogue starts at `#4-40`.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub size: &'static str,
    pub thread: &'static str,
    pub length: f64,
    /// Outside diameter, D1.
    pub diameter: f64,
    /// Plastic hole diameter, D2.
    pub hole: f64,
    /// Minimum wall beside the hole.
    pub wall: f64,
    /// Basic major diameter of the thread, for the outside-diameter check.
    pub major: f64,
    /// A bore with no thread name. SolidCraft has no `#2-56` tap.
    pub plain_bore: Option<f64>,
}

pub const DEFAULT_SIZE: &str = "M3 × 5.7";

/// `#2-56` major minus its pitch: 0.086 in, 56 threads per inch.
const BORE_2_56: f64 = 0.086 * 25.4 - 25.4 / 56.0;

const ROWS: &[Row] = &[
    row("M1.6 × 2.5", "M1.6", 2.5, 2.5, 2.2, 1.3, 1.6, None),
    row("M2 × 3", "M2", 3.0, 3.6, 3.2, 1.3, 2.0, None),
    row("M2.5 × 4", "M2.5", 4.0, 4.6, 4.0, 1.6, 2.5, None),
    row("M3 × 5.7", "M3", 5.7, 4.6, 4.0, 1.6, 3.0, None),
    row("M3 × 3 short", "M3", 3.0, 4.6, 4.0, 1.6, 3.0, None),
    row("M3 × 4 Voron", "M3", 4.0, 5.0, 4.4, 1.6, 3.0, None),
    row("M4 × 8.1", "M4", 8.1, 6.3, 5.7, 2.1, 4.0, None),
    row("M4 × 4 short", "M4", 4.0, 6.3, 5.7, 2.1, 4.0, None),
    row("M5 × 9.5", "M5", 9.5, 7.1, 6.5, 2.6, 5.0, None),
    row("M5 × 5.8 short", "M5", 5.8, 7.1, 6.5, 2.6, 5.0, None),
    row("M6 × 12.7", "M6", 12.7, 8.7, 8.1, 3.3, 6.0, None),
    row("M6 × 6.8 short", "M6", 6.8, 8.7, 8.1, 3.3, 6.0, None),
    row("M8 × 12.7", "M8", 12.7, 10.2, 9.7, 4.5, 8.0, None),
    row("M8 × 6.8 short", "M8", 6.8, 10.2, 9.7, 4.5, 8.0, None),
    row("M10 × 12.7", "M10", 12.7, 12.6, 12.0, 6.0, 10.0, None),
    row(
        "#2-56 × 3.2",
        "#2-56 UNC",
        3.2,
        3.6,
        3.2,
        1.3,
        0.086 * 25.4,
        Some(BORE_2_56),
    ),
    row(
        "#4-40 × 5.7",
        "#4-40 UNC",
        5.7,
        4.6,
        4.0,
        1.6,
        0.112 * 25.4,
        None,
    ),
    row(
        "#4-40 × 3.6 short",
        "#4-40 UNC",
        3.6,
        4.6,
        4.0,
        1.6,
        0.112 * 25.4,
        None,
    ),
    row(
        "#6-32 × 7.1",
        "#6-32 UNC",
        7.1,
        5.4,
        4.8,
        1.8,
        0.138 * 25.4,
        None,
    ),
    row(
        "#6-32 × 3.8 short",
        "#6-32 UNC",
        3.8,
        5.4,
        4.8,
        1.8,
        0.138 * 25.4,
        None,
    ),
    row(
        "#8-32 × 8.2",
        "#8-32 UNC",
        8.2,
        6.3,
        5.7,
        2.1,
        0.164 * 25.4,
        None,
    ),
    row(
        "#8-32 × 4.7 short",
        "#8-32 UNC",
        4.7,
        6.3,
        5.7,
        2.1,
        0.164 * 25.4,
        None,
    ),
    row(
        "#10-24 × 9.5",
        "#10-24 UNC",
        9.5,
        7.1,
        6.5,
        2.6,
        0.190 * 25.4,
        None,
    ),
    row(
        "#10-32 × 9.5",
        "#10-32 UNF",
        9.5,
        7.1,
        6.5,
        2.6,
        0.190 * 25.4,
        None,
    ),
    row(
        "1/4-20 × 12.7",
        "1/4-20 UNC",
        12.7,
        8.7,
        8.1,
        3.3,
        6.35,
        None,
    ),
    row(
        "1/4-20 × 6.4 short",
        "1/4-20 UNC",
        6.4,
        8.7,
        8.1,
        3.3,
        6.35,
        None,
    ),
    row(
        "5/16-18 × 12.7",
        "5/16-18 UNC",
        12.7,
        10.2,
        9.7,
        4.5,
        7.9375,
        None,
    ),
    row(
        "3/8-16 × 12.7",
        "3/8-16 UNC",
        12.7,
        12.5,
        12.0,
        6.0,
        9.525,
        None,
    ),
    row("G1/8 × 12.7", "G 1/8", 12.7, 12.5, 12.0, 6.0, 9.728, None),
];

// One argument per field of `Row`, so each catalog entry reads as a table line.
#[allow(clippy::too_many_arguments)]
const fn row(
    size: &'static str,
    thread: &'static str,
    length: f64,
    diameter: f64,
    hole: f64,
    wall: f64,
    major: f64,
    plain_bore: Option<f64>,
) -> Row {
    Row {
        size,
        thread,
        length,
        diameter,
        hole,
        wall,
        major,
        plain_bore,
    }
}

/// A stepped hole for gluing an insert into a resin print: `d1` down to `l1`, `d2` down
/// to `l2`, then `d3` for the rest. CNC Kitchen lists these eight sizes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stepped {
    pub d1: f64,
    pub d2: f64,
    pub d3: f64,
    pub l1: f64,
    pub l2: f64,
}

/// `(size, d1, d2, d3, l1, l2)`.
const STEPPED: &[(&str, f64, f64, f64, f64, f64)] = &[
    ("M2.5 × 4", 4.6, 4.4, 4.0, 1.0, 2.7),
    ("M3 × 3 short", 4.6, 4.4, 4.0, 0.5, 2.3),
    ("M3 × 5.7", 4.6, 4.4, 4.0, 1.7, 4.2),
    ("M4 × 4 short", 6.4, 6.1, 5.6, 1.0, 3.0),
    ("M4 × 8.1", 6.4, 6.1, 5.6, 2.4, 6.1),
    ("M5 × 5.8 short", 7.1, 6.9, 6.4, 1.6, 4.3),
    ("M5 × 9.5", 7.1, 6.9, 6.4, 3.1, 7.4),
    ("1/4-20 × 12.7", 8.8, 8.5, 8.0, 3.9, 9.3),
];

pub fn stepped(size: &str) -> Option<Stepped> {
    STEPPED
        .iter()
        .find(|row| row.0 == size)
        .map(|&(_, d1, d2, d3, l1, l2)| Stepped { d1, d2, d3, l1, l2 })
}

/// The sizes that have a stepped hole, for the message when another is asked for.
pub fn stepped_sizes() -> String {
    STEPPED
        .iter()
        .map(|row| row.0)
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn get(size: &str) -> Option<&'static Row> {
    ROWS.iter().find(|row| row.size == size)
}

/// The row for a size label, however it is spaced or cased, with `x` for `×`.
pub fn find_size(label: &str) -> Option<&'static Row> {
    let want = compact(label).replace('×', "x");
    ROWS.iter()
        .find(|row| compact(row.size).replace('×', "x") == want)
}

/// The row for a screw size such as `M3` or `#4-40`.
pub fn find_thread(name: &str) -> Option<&'static Row> {
    let want = compact(name);
    if want.is_empty() {
        return None;
    }
    ROWS.iter()
        .find(|row| compact(row.thread) == want)
        .or_else(|| {
            ROWS.iter().find(|row| {
                let full = compact(row.thread);
                let Some(rest) = full.strip_prefix(&want) else {
                    return false;
                };
                match rest.chars().next() {
                    Some(c) if c.is_ascii_digit() || c == '.' => false,
                    Some(_) => true,
                    None => false,
                }
            })
        })
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn labels() -> impl Iterator<Item = &'static str> {
    ROWS.iter().map(|row| row.size)
}

pub fn metric(thread: &str) -> bool {
    thread.starts_with(['M', 'm'])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CNC Kitchen's tables, as printed: `(size, L, D1, D2, W)`.
    const CNC_KITCHEN: &[(&str, f64, f64, f64, f64)] = &[
        ("M2 × 3", 3.0, 3.6, 3.2, 1.3),
        ("M2.5 × 4", 4.0, 4.6, 4.0, 1.6),
        ("M3 × 5.7", 5.7, 4.6, 4.0, 1.6),
        ("M3 × 3 short", 3.0, 4.6, 4.0, 1.6),
        ("M3 × 4 Voron", 4.0, 5.0, 4.4, 1.6),
        ("M4 × 8.1", 8.1, 6.3, 5.7, 2.1),
        ("M4 × 4 short", 4.0, 6.3, 5.7, 2.1),
        ("M5 × 9.5", 9.5, 7.1, 6.5, 2.6),
        ("M5 × 5.8 short", 5.8, 7.1, 6.5, 2.6),
        ("M6 × 12.7", 12.7, 8.7, 8.1, 3.3),
        ("M8 × 12.7", 12.7, 10.2, 9.7, 4.5),
        ("M10 × 12.7", 12.7, 12.6, 12.0, 6.0),
        ("#2-56 × 3.2", 3.2, 3.6, 3.2, 1.3),
        ("#4-40 × 5.7", 5.7, 4.6, 4.0, 1.6),
        ("#4-40 × 3.6 short", 3.6, 4.6, 4.0, 1.6),
        ("#6-32 × 7.1", 7.1, 5.4, 4.8, 1.8),
        ("#6-32 × 3.8 short", 3.8, 5.4, 4.8, 1.8),
        ("#8-32 × 8.2", 8.2, 6.3, 5.7, 2.1),
        ("#8-32 × 4.7 short", 4.7, 6.3, 5.7, 2.1),
        ("#10-24 × 9.5", 9.5, 7.1, 6.5, 2.6),
        ("#10-32 × 9.5", 9.5, 7.1, 6.5, 2.6),
        ("1/4-20 × 12.7", 12.7, 8.7, 8.1, 3.3),
        ("1/4-20 × 6.4 short", 6.4, 8.7, 8.1, 3.3),
        ("5/16-18 × 12.7", 12.7, 10.2, 9.7, 4.5),
        ("3/8-16 × 12.7", 12.7, 12.5, 12.0, 6.0),
        ("G1/8 × 12.7", 12.7, 12.5, 12.0, 6.0),
    ];

    /// The stepped-bore table: `(size, D1, D2, D3, L1, L2)`.
    const CNC_KITCHEN_STEPPED: &[(&str, f64, f64, f64, f64, f64)] = &[
        ("M2.5 × 4", 4.6, 4.4, 4.0, 1.0, 2.7),
        ("M3 × 3 short", 4.6, 4.4, 4.0, 0.5, 2.3),
        ("M3 × 5.7", 4.6, 4.4, 4.0, 1.7, 4.2),
        ("M4 × 4 short", 6.4, 6.1, 5.6, 1.0, 3.0),
        ("M4 × 8.1", 6.4, 6.1, 5.6, 2.4, 6.1),
        ("M5 × 5.8 short", 7.1, 6.9, 6.4, 1.6, 4.3),
        ("M5 × 9.5", 7.1, 6.9, 6.4, 3.1, 7.4),
        ("1/4-20 × 12.7", 8.8, 8.5, 8.0, 3.9, 9.3),
    ];

    #[test]
    fn rows_match_cnc_kitchen() {
        for &(size, length, diameter, hole, wall) in CNC_KITCHEN {
            let row = get(size).unwrap_or_else(|| panic!("{size} is not a row"));
            assert_eq!(
                (row.length, row.diameter, row.hole, row.wall),
                (length, diameter, hole, wall),
                "{size}"
            );
        }
        // Every row comes from it but three it does not print: M1.6 × 2.5 and the
        // short M6 and M8, which take their long size's diameter, hole and wall.
        assert_eq!(CNC_KITCHEN.len() + 3, ROWS.len());
        for (short, long) in [
            ("M6 × 6.8 short", "M6 × 12.7"),
            ("M8 × 6.8 short", "M8 × 12.7"),
        ] {
            let (short, long) = (get(short).unwrap(), get(long).unwrap());
            assert_eq!(
                (short.diameter, short.hole, short.wall),
                (long.diameter, long.hole, long.wall)
            );
        }
        assert_eq!(STEPPED, CNC_KITCHEN_STEPPED);
    }

    #[test]
    fn every_label_resolves_and_clears_its_thread() {
        for label in labels() {
            let row = get(label).unwrap();
            assert_eq!(row.size, label);
            assert!(row.diameter > row.major, "{label}");
            assert!(
                row.diameter >= crate::knurl::min_crest_diameter(row.major),
                "{label}"
            );
            assert!(row.hole > 0.0 && row.wall > 0.0, "{label}");
        }
    }

    #[test]
    fn the_dialog_lists_every_row_in_catalog_order() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let size = &manifest["runtime"]["entries"][0]["input_schema"]["properties"]["size"];
        let listed: Vec<&str> = size["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|label| label.as_str().unwrap())
            .collect();
        assert_eq!(listed, labels().collect::<Vec<_>>());
        assert_eq!(size["default"], DEFAULT_SIZE);
        for label in labels() {
            assert_eq!(find_size(label).unwrap().size, label);
        }
        assert_eq!(find_size("m3 x 5.7").unwrap().size, "M3 × 5.7");
    }

    #[test]
    fn every_stepped_hole_is_a_catalog_row_and_narrows_downward() {
        for (size, d1, d2, d3, l1, l2) in STEPPED {
            let row = get(size).unwrap_or_else(|| panic!("{size} is not a row"));
            assert!(d1 > d2 && d2 > d3 && *d3 > row.major, "{size}");
            assert!(l1 < l2 && *l2 < row.length, "{size}");
        }
        assert_eq!(stepped("M3 × 5.7").unwrap().l2, 4.2);
        assert!(stepped("M8 × 12.7").is_none());
    }

    #[test]
    fn a_screw_size_finds_its_row_without_the_length() {
        assert_eq!(find_thread("M3").unwrap().major, 3.0);
        assert_eq!(find_thread("m2").unwrap().thread, "M2");
        assert_eq!(find_thread("#4-40").unwrap().thread, "#4-40 UNC");
        assert_eq!(find_thread("G 1/8").unwrap().thread, "G 1/8");
        assert!(find_thread("M2").unwrap().thread != "M2.5");
        assert!(find_thread("nope").is_none());
    }
}
