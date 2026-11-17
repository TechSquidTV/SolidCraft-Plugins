//! Native test driver for the real inspector against a saved design. Each read
//! opens the same immutable fixture, so no production command transport is needed.

use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

use plugin_kit::{content::Index, Runner};
use serde_json::{json, Value};

struct Host {
    cli: String,
    design: String,
    scratch: PathBuf,
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

impl Runner for Host {
    fn run(&mut self, command: &str, params: Value) -> Result<Value, String> {
        let script = self.scratch.join("request.json");
        let result = self.scratch.join("result.json");
        fs::write(
            &script,
            json!([{ "command": command, "params": params }]).to_string(),
        )
        .map_err(|e| e.to_string())?;
        let output = Command::new(&self.cli)
            .arg("run")
            .arg(&script)
            .arg("--in")
            .arg(&self.design)
            .args(["--quiet", "--results", "--json-out"])
            .arg(&result)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "{command}: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let values: Vec<Value> =
            serde_json::from_slice(&fs::read(result).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        values
            .into_iter()
            .next()
            .ok_or_else(|| format!("{command} returned no result"))
    }
}

fn inspect() -> Result<Value, String> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [cli, design, component] = args.as_slice() else {
        return Err("usage: inspect_content CLI DESIGN COMPONENT_ID".into());
    };
    let component = component.parse::<u64>().map_err(|e| e.to_string())?;
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let scratch = env::temp_dir().join(format!("parts-content-{}-{time}", std::process::id()));
    fs::create_dir(&scratch).map_err(|e| e.to_string())?;
    let mut host = Host {
        cli: cli.clone(),
        design: design.clone(),
        scratch,
    };
    let mut index = Index::read(&mut host)?;
    let snapshot = index.snapshot(&mut host, component)?;
    let reads = index.reads();
    // A second use in the same invocation must use the verified cache.
    let repeated = index.snapshot(&mut host, component)?;
    if repeated != snapshot || index.reads() != reads {
        return Err("a cached definition was inspected more than once".into());
    }
    Ok(json!({"snapshot": snapshot, "reads": reads}))
}

fn main() -> ExitCode {
    match inspect() {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
