//! Static forms for the Bearings plugin. The host renders one form per entry.

use bearings::{csk, Closure, Envelope, Size};
use serde_json::{json, Map, Value};

pub(super) fn entries() -> [Value; 4] {
    let radial_sizes = Size::ALL
        .iter()
        .map(|size| size_option(size.label(), size.envelope()))
        .collect::<Vec<_>>();
    let starting_presets = std::iter::once(json!({
        "const": "From scratch", "title": "From scratch — Dust covers by default"
    }))
        .chain(csk::ALL.into_iter().map(|preset| json!({
            "const": preset.designation(),
            "title": format!("{} — {} · {} mm", preset.designation(), preset.closure().label(), preset.dimensions())
        })))
        .collect::<Vec<_>>();

    let radial_closure = json!({
        "title": "Closure", "type": "string", "default": "Shielded",
        "oneOf": Closure::ALL.map(|closure| json!({
            "const": closure.label(),
            "title": if closure == Closure::Open { "Open (626 only)" } else { closure.label() }
        }))
    });
    let csk_closures = csk::Closure::ALL.map(csk::Closure::label);
    let search_keyway = |title| {
        json!({
            "title": title, "type": "string", "default": "Any",
            "oneOf": [
                {"const": "Any"},
                {"const": "Enabled", "title": "Present"},
                {"const": "None", "title": "Absent"}
            ]
        })
    };

    // Required fields lead the host's form in the order given here. Optional
    // fields sort by key, so match_* comes before narrow_by_* and placement.
    let mut radial_search = dimensions("match", "Leave blank to match any value.");
    radial_search.insert("narrow_by_closure".into(), radial_closure.clone());
    let mut csk_search = dimensions("match", "Leave blank to match any value.");
    csk_search.extend([
        (
            "narrow_by_closure".into(),
            choice("Closure", std::iter::once("Any").chain(csk_closures)),
        ),
        (
            "narrow_by_inner_keyway".into(),
            search_keyway("Inner keyway"),
        ),
        (
            "narrow_by_outer_keyway".into(),
            search_keyway("Outer keyway"),
        ),
    ]);
    let mut configure = dimensions(
        "custom",
        "Leave blank to use the preset. From scratch requires all three dimensions.",
    );
    configure.extend([
        (
            "size".into(),
            json!({
                "title": "Starting preset", "type": "string", "default": "CSK12P",
                "oneOf": starting_presets
            }),
        ),
        ("closure".into(), inherited_choice("Closure", csk_closures)),
    ]);
    configure["closure"]["description"] =
        json!("Use the starting preset’s closure or choose an override. Overrides keep the current dimensions. From scratch uses dust covers by default.");
    for (key, title, angle) in [("inner", "Inner", 0), ("outer", "Outer", 180)] {
        let key = format!("{key}_keyway");
        let title = format!("{title} keyway");
        let mut mode = inherited_choice(&title, ["Enabled", "None"]);
        mode["description"] =
            json!("Use the starting preset’s keyway or choose an override. To remove a keyway, choose None and clear its angle, depth and width.");
        configure.insert(key.clone(), mode);
        configure.insert(
            format!("{key}_angle"),
            json!({
                "title": format!("{title} angle"), "type": "number", "unit": "deg",
                "description": format!("Counterclockwise from local +Y viewed from +Z. A new keyway starts at {angle}°.")
            }),
        );
        for axis in ["depth", "width"] {
            configure.insert(
                format!("{key}_{axis}"),
                json!({
                    "title": format!("{title} {axis}"), "type": "number", "unit": "mm",
                    "exclusiveMinimum": 0,
                    "description": "Leave blank to use the preset. A new keyway needs both width and depth."
                }),
            );
        }
    }

    [
        entry(
            "make-bearing",
            "Bearing",
            [
                ("size".into(), size_field("608", radial_sizes)),
                ("closure".into(), radial_closure),
            ]
            .into_iter()
            .collect(),
            &["size", "closure"],
        ),
        entry("make-csk", "CSK clutch", configure, &["size"]),
        entry(
            "make-bearing-by-dimensions",
            "Bearing by dimensions",
            radial_search,
            &[],
        ),
        entry(
            "make-csk-by-dimensions",
            "CSK clutch by dimensions",
            csk_search,
            &[],
        ),
    ]
}

fn entry(name: &str, title: &str, mut fields: Map<String, Value>, required: &[&str]) -> Value {
    fields.insert(
        "placement".into(),
        json!({
            "title": "Placement",
            "description": "Pick sketch points, circles or circle interiors. With none selected, place one at the origin.",
            "type": "array", "maxItems": 32,
            "x-solidcraft-selection": {
                "accept": ["profile", "sketch_point", "sketch_curve"], "version": 1
            }
        }),
    );
    json!({
        "name": name, "effect": "edit",
        "slots": [{"kind": "menu", "name": title}],
        "input_schema": {
            "type": "object", "additionalProperties": true,
            "properties": fields, "required": required,
            "x-solidcraft-icon": format!("icons/{name}.svg")
        }
    })
}

fn dimensions(prefix: &str, tip: &str) -> Map<String, Value> {
    [
        ("bore", "Bore"),
        ("diameter", "Outside diameter"),
        ("width", "Width"),
    ]
    .into_iter()
    .map(|(key, title)| {
        (
            format!("{prefix}_{key}"),
            json!({
                "title": title, "type": "number", "unit": "mm", "description": tip,
                "exclusiveMinimum": 0, "maximum": plugin_kit::input::MAX_LENGTH_MM
            }),
        )
    })
    .collect()
}

fn size_option(designation: &str, envelope: Envelope) -> Value {
    json!({"const": designation, "title": format!("{designation} — {envelope} mm")})
}

fn size_field(default: &str, options: Vec<Value>) -> Value {
    json!({"title": "Size", "type": "string", "default": default, "oneOf": options})
}

fn choice<'a>(title: &str, options: impl IntoIterator<Item = &'a str>) -> Value {
    let options = options.into_iter().collect::<Vec<_>>();
    json!({"title": title, "type": "string", "default": options.first(), "enum": options})
}

fn inherited_choice<'a>(title: &str, overrides: impl IntoIterator<Item = &'a str>) -> Value {
    let options = std::iter::once(json!({"const": "Preset", "title": "Use starting preset"}))
        .chain(overrides.into_iter().map(|name| json!({"const": name})))
        .collect::<Vec<_>>();
    json!({"title": title, "type": "string", "default": "Preset", "oneOf": options})
}
