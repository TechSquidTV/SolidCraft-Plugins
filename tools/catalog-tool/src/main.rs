//! Update only catalog-owned manifest fields; workflow fields stay hand-authored.

use std::{env, fs, path::Path, process::ExitCode};

use serde_json::{json, Value};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    let write = match args.as_slice() {
        [option] if option == "--write" => true,
        [option] if option == "--check" => false,
        _ => return Err("usage: cargo run -p catalog-tool -- --check|--write".into()),
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let strings = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let rail_sizes = extrusions::profile::SIZES
        .iter()
        .map(|s| s.0.to_string())
        .collect();
    for (plugin, fields) in [
        (
            "techsquid-inserts",
            vec![(
                "make-insert",
                "size",
                techsquid_inserts::catalog::labels()
                    .map(str::to_string)
                    .collect(),
            )],
        ),
        (
            "techsquid-bearings",
            vec![
                (
                    "make-bearing",
                    "size",
                    bearings::ALL
                        .into_iter()
                        .map(|b| b.designation().to_string())
                        .collect(),
                ),
                (
                    "make-csk",
                    "size",
                    bearings::csk::ALL
                        .into_iter()
                        .map(|c| c.designation().to_string())
                        .collect(),
                ),
            ],
        ),
        (
            "techsquid-v-wheels",
            vec![(
                "make-v-wheel",
                "size",
                wheels::ALL
                    .into_iter()
                    .map(|w| w.name().to_string())
                    .collect(),
            )],
        ),
        (
            "techsquid-nuts",
            vec![("make-nut", "size", techsquid_nuts::provider::labels())],
        ),
        (
            "techsquid-screws",
            vec![
                (
                    "make-screw",
                    "size",
                    strings(&techsquid_screws::provider::COMMON_SIZES),
                ),
                (
                    "make-screw",
                    "head",
                    techsquid_screws::screw::Head::ALL
                        .into_iter()
                        .map(|h| h.label().to_string())
                        .collect(),
                ),
            ],
        ),
        (
            "techsquid-extrusions",
            vec![
                ("make-extrusion", "size", rail_sizes),
                ("make-t-nut", "size", strings(&extrusions::tnut::THREADS)),
            ],
        ),
    ] {
        update(
            &root.join(format!("plugins/{plugin}/plugin.json")),
            fields,
            write,
        )?;
    }
    let mut bearings_table = String::from(
        "| Designation | Supplier | Part number | Bore × OD × width (mm) | Closure / mounting | Exterior |\n|---|---|---|---|---|---|\n",
    );
    for bearing in bearings::ALL {
        let d = bearing.dimensions();
        let closure = match bearing.closure() {
            bearings::Closure::Open => "Open, visible balls",
            bearings::Closure::RubberSeals => "Rubber seals, both sides",
            bearings::Closure::MetalShields => "Metal shields, both sides",
        };
        bearings_table.push_str(&format!(
            "| {} | {} | {} | {} × {} × {} | {closure} | Measured |\n",
            bearing.designation(),
            bearing.supplier().name,
            bearing.sku(),
            d.bore,
            d.diameter,
            d.width
        ));
    }
    for csk in bearings::csk::ALL {
        let d = csk.dimensions();
        let basis = exterior_basis(csk);
        bearings_table.push_str(&format!(
            "| {} | Unassigned | — | {} × {} × {} | {} ([reference]({})) | {basis} |\n",
            csk.designation(),
            d.bore,
            d.diameter,
            d.width,
            csk.mounting_description(),
            csk.source().url
        ));
    }
    let mut wheels_table = String::from("| Part | OpenBuilds SKU | OD × bare width (mm) | Bearings | Internal shim, OD × bore × width (mm) |\n|---|---|---|---|---|\n");
    for wheel in wheels::ALL {
        let (diameter, width) = wheel.dimensions();
        let shim = wheel.shim();
        wheels_table.push_str(&format!(
            "| {} | {} | {diameter} × {width} | 2 × {} | {} × {} × {} |\n",
            wheel.name(),
            wheel.sku().unwrap_or("Unconfirmed"),
            wheel.bearing().designation(),
            shim.diameter(),
            shim.bore(),
            shim.thickness()
        ));
    }
    update_table(
        &root.join("plugins/techsquid-bearings/README.md"),
        &bearings_table,
        write,
    )?;
    for (plugin, table) in [
        ("techsquid-bearings", bearings_table),
        ("techsquid-v-wheels", wheels_table),
    ] {
        update_table(
            &root.join(format!("plugins/{plugin}/REFERENCE.md")),
            &table,
            write,
        )?;
    }
    Ok(())
}

fn exterior_basis(csk: bearings::csk::Csk) -> &'static str {
    match bearings::csk::exterior::resolve(&csk.geometry())
        .expect("valid CSK preset")
        .basis
    {
        bearings::csk::exterior::Basis::Measured => "Measured",
        bearings::csk::exterior::Basis::Estimated => "Estimated",
    }
}

fn update_table(path: &Path, table: &str, write: bool) -> Result<(), String> {
    const START: &str = "<!-- catalog-table:start -->";
    const END: &str = "<!-- catalog-table:end -->";
    let original = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (before, rest) = original
        .split_once(START)
        .ok_or("catalog table has no start marker")?;
    let (_, after) = rest
        .split_once(END)
        .ok_or("catalog table has no end marker")?;
    let updated = format!("{before}{START}\n\n{table}\n{END}{after}");
    if updated != original {
        if !write {
            return Err(
                "catalog reference table is stale; run cargo run -p catalog-tool -- --write".into(),
            );
        }
        fs::write(path, updated).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn update(path: &Path, fields: Vec<(&str, &str, Vec<String>)>, write: bool) -> Result<(), String> {
    let original = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut manifest: Value = serde_json::from_str(&original).map_err(|e| e.to_string())?;
    let entries = manifest["runtime"]["entries"]
        .as_array_mut()
        .ok_or("plugin manifest has no entry list")?;
    for (entry, field, choices) in fields {
        let workflow = entries
            .iter_mut()
            .find(|e| e["name"] == entry)
            .ok_or("manifest has no insertion workflow")?;
        let property = workflow["input_schema"]["properties"]
            .get_mut(field)
            .ok_or_else(|| format!("{entry} has no {field} field"))?;
        if entry == "make-csk" && field == "size" {
            property
                .as_object_mut()
                .ok_or("missing CSK size property")?
                .remove("enum");
            let mut options: Vec<_> = bearings::csk::ALL
                .into_iter()
                .map(|csk| {
                    let d = csk.dimensions();
                    let mut title = format!(
                        "{} — {} × {} × {} mm",
                        csk.designation(),
                        d.bore,
                        d.diameter,
                        d.width
                    );
                    let (inner, outer) = csk.keyways();
                    for (label, key) in [("inner", inner), ("outer", outer)] {
                        if let Some(key) = key {
                            title.push_str(&format!("; {label} {} × {}", key.width, key.depth));
                        }
                    }
                    json!({"const":csk.designation(), "title":title})
                })
                .collect();
            options
                .push(json!({"const":"Custom", "title":"Custom — enter dimensions and keyways"}));
            property["oneOf"] = json!(options);
        } else if entry == "make-bearing" && field == "size" {
            // The designation alone says little: show who makes it, its size and its closure.
            property
                .as_object_mut()
                .ok_or("missing bearing size property")?
                .remove("enum");
            let options: Vec<_> = bearings::ALL
                .into_iter()
                .map(|bearing| {
                    let d = bearing.dimensions();
                    let closure = match bearing.closure() {
                        bearings::Closure::Open => "open",
                        bearings::Closure::RubberSeals => "sealed",
                        bearings::Closure::MetalShields => "shielded",
                    };
                    let title = format!(
                        "{} — {}, {} × {} × {} mm, {closure}",
                        bearing.designation(),
                        bearing.supplier().name,
                        d.bore,
                        d.diameter,
                        d.width
                    );
                    json!({"const":bearing.designation(), "title":title})
                })
                .collect();
            property["oneOf"] = json!(options);
        } else {
            property["enum"] = json!(choices);
        }
    }
    let mut commands: std::collections::BTreeSet<String> =
        serde_json::from_value(manifest["runtime"]["commands"].clone())
            .map_err(|e| e.to_string())?;
    commands.extend(plugin_kit::component::commands().map(str::to_string));
    manifest["runtime"]["commands"] = json!(commands);
    let current: Value = serde_json::from_str(&original).map_err(|e| e.to_string())?;
    if manifest != current {
        if !write {
            return Err(
                "catalog fields are stale; run cargo run -p catalog-tool -- --write".into(),
            );
        }
        fs::write(
            path,
            serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_keeps_packages_insertion_only() {
        let plugins = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
        for plugin in fs::read_dir(plugins).unwrap() {
            let manifest = plugin.unwrap().path().join("plugin.json");
            if !manifest.is_file() {
                continue;
            }
            let text = fs::read_to_string(&manifest).unwrap();
            let value: Value = serde_json::from_str(&text).unwrap();
            assert!(
                serde_json::to_vec(&value).unwrap().len() <= 16 * 1024,
                "{} exceeds the host's 16 KiB limit after packaging as compact JSON",
                manifest.display()
            );
            let entries = value["runtime"]["entries"].as_array().unwrap();
            assert!(!entries.is_empty());
            for entry in entries {
                assert_eq!(entry["effect"], "edit", "{}", manifest.display());
                assert_ne!(entry["name"], "catalog", "{}", manifest.display());
            }
            // Re-running the generator must not restore a discovery entry.
            update(&manifest, vec![], false).unwrap();
        }
    }
}
