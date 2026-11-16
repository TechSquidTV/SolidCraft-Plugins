//! Emit actual library build commands for native geometry verification. This
//! recorder only substitutes sketch IDs with names; the host evaluates all CAD.

use plugin_kit::{Placement, Runner};
use serde_json::{json, Value};

#[derive(Default)]
struct Recorder(Vec<Value>);

impl Runner for Recorder {
    fn run(&mut self, command: &str, params: Value) -> Result<Value, String> {
        let reply = match command {
            "sketch.create" => json!({"sketch": params["name"]}),
            _ => json!({}),
        };
        self.0.push(json!({"command": command, "params": params}));
        Ok(reply)
    }
}

fn main() -> Result<(), String> {
    let mut r = Recorder::default();
    for (index, wheel) in wheels::ALL.into_iter().enumerate() {
        r.run(
            "component.create",
            json!({"name": wheel.name(), "activate": true, "parent": 0, "grounded": false}),
        )?;
        wheel.build(
            &mut r,
            wheel.name(),
            &Placement {
                point: [index as f64 * 30.0, 0.0, 0.0],
                ..Placement::on_origin()
            },
        )?;
    }
    for (index, bearing) in bearings::ALL.into_iter().enumerate() {
        r.run(
            "component.create",
            json!({"name": bearing.designation(), "activate": true, "parent": 0, "grounded": false}),
        )?;
        bearing.build(
            &mut r,
            bearing.designation(),
            &Placement {
                point: [index as f64 * 30.0, 35.0, 0.0],
                ..Placement::on_origin()
            },
        )?;
    }
    for (index, shim) in [wheels::Shim::FullSize, wheels::Shim::Mini]
        .into_iter()
        .enumerate()
    {
        let name = format!("Shim {}", shim.sku());
        r.run(
            "component.create",
            json!({"name": name, "activate": true, "parent": 0, "grounded": false}),
        )?;
        shim.build(
            &mut r,
            &name,
            &Placement {
                point: [index as f64 * 30.0, 60.0, 0.0],
                ..Placement::on_origin()
            },
        )?;
    }
    r.run("component.activate", json!({"component": 0}))?;
    r.run("inspect.measure", json!({"summary": true}))?;
    r.run("component.list", json!({}))?;
    r.run("appearance.list", json!({}))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&r.0).map_err(|e| e.to_string())?
    );
    Ok(())
}
