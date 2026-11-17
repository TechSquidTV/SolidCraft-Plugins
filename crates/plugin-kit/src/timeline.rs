//! Timeline folders.
//!
//! A plugin run can leave many rows in the timeline: a sketch and a feature for every
//! pocket, a dozen for one part. [`folded`] puts the rows one piece of work makes into a
//! folder of their own. SolidCraft shows a new folder folded, so the run reads as one row
//! until it is opened.

use serde_json::{json, Value};

use crate::{Placement, Runner};

/// The command [`folded`] sends. A plugin that folds lists it in its manifest.
pub const COMMAND: &str = "timeline.group";

/// Runs `work`, then folds every sketch and feature it made into one folder named `name`.
/// Failure to group propagates to the host transaction, as does a build failure.
pub fn folded<R, T, F>(r: &mut R, name: &str, work: F) -> Result<T, String>
where
    R: Runner,
    F: FnOnce(&mut dyn Runner) -> Result<T, String>,
{
    let mut made = Made {
        host: r,
        rows: Vec::new(),
    };
    let out = work(&mut made)?;
    let rows = made.rows;
    if !rows.is_empty() {
        r.run(COMMAND, json!({"name": name, "features": rows}))?;
    }
    Ok(out)
}

/// A host that passes every command on and keeps the id of each sketch and feature the
/// replies name.
struct Made<'a, R: Runner> {
    host: &'a mut R,
    rows: Vec<u64>,
}

impl<R: Runner> Runner for Made<'_, R> {
    fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
        let reply = self.host.run(command, args)?;
        // A new sketch is a timeline row too; its reply names it `sketch`.
        let key = if command == "sketch.create" {
            "sketch"
        } else {
            "feature"
        };
        if let Some(id) = reply.get(key).and_then(Value::as_u64) {
            if !self.rows.contains(&id) {
                self.rows.push(id);
            }
        }
        Ok(reply)
    }

    fn body_names(&mut self) -> Vec<String> {
        self.host.body_names()
    }

    fn site(&mut self, at: [f64; 2]) -> Placement {
        self.host.site(at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Numbers every sketch and feature, as SolidCraft does.
    struct Timeline(Vec<(String, Value)>, u64, bool);
    impl Runner for Timeline {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            self.0.push((command.to_string(), args));
            self.1 += 1;
            match command {
                "timeline.group" if !self.2 => Err("unknown command timeline.group".into()),
                "sketch.create" => Ok(json!({"sketch": self.1})),
                c if c.starts_with("solid.") => Ok(json!({"feature": self.1})),
                _ => Ok(json!({})),
            }
        }
    }

    fn pocket(r: &mut dyn Runner) -> Result<u8, String> {
        r.run("sketch.create", json!({}))?;
        r.run("sketch.line", json!({}))?;
        r.run("solid.extrude", json!({}))?;
        r.run("browser.visibility", json!({}))?;
        r.run("solid.hole", json!({}))?;
        Ok(7)
    }

    #[test]
    fn the_sketches_and_features_made_are_folded_under_the_name() {
        let mut host = Timeline(Vec::new(), 10, true);
        assert_eq!(folded(&mut host, "Nut pockets M6", pocket), Ok(7));
        let (command, args) = host.0.last().unwrap();
        assert_eq!(command, "timeline.group");
        // The sketch, the extrude and the hole; not the line or the visibility.
        assert_eq!(
            args,
            &json!({"name": "Nut pockets M6", "features": [11, 13, 15]})
        );
    }

    #[test]
    fn a_grouping_failure_propagates_for_transaction_rollback() {
        let mut host = Timeline(Vec::new(), 0, false);
        assert!(folded(&mut host, "Nut pockets M6", pocket).is_err());
    }

    #[test]
    fn nothing_made_or_a_failure_makes_no_folder() {
        let mut host = Timeline(Vec::new(), 0, true);
        folded(&mut host, "Empty", |_| Ok(())).unwrap();
        assert!(host.0.is_empty());
        let failed = folded(&mut host, "Failed", |r| {
            r.run("solid.extrude", json!({}))?;
            Err::<(), _>("no part under the point".to_string())
        });
        assert!(failed.is_err());
        assert!(!host.0.iter().any(|(c, _)| c == "timeline.group"));
    }
}
