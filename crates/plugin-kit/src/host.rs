//! The slice of the SolidCraft host a generator needs.
//!
//! Each plugin implements this on its own wasm guest. The geometry calls `run` and never
//! sees the component imports.

use serde_json::Value;

/// Run one SolidCraft command with JSON arguments and take the JSON reply.
pub trait Runner {
    fn run(&mut self, command: &str, args: Value) -> Result<Value, String>;

    /// Commands already sent by this invocation, including placement discovery.
    fn command_count(&self) -> usize {
        0
    }

    /// The names of the bodies already in the design. A guest asks the host with
    /// `measured_body_names`. A host that cannot say has none.
    fn body_names(&mut self) -> Vec<String> {
        Vec::new()
    }

    /// The frame a new definition is built on, for the site its plugin asks for: see
    /// [`crate::component::site_at`]. A test host that checks what a build sends keeps
    /// it on the origin, so the numbers it checks are the part's own.
    fn site(&mut self, at: [f64; 2]) -> crate::Placement {
        crate::component::site_at(at)
    }
}

/// A builder generic over `Runner` runs on a `&mut dyn Runner` too, as
/// [`crate::component::Library::geometry`] hands it one.
impl Runner for &mut dyn Runner {
    fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
        (**self).run(command, args)
    }

    fn body_names(&mut self) -> Vec<String> {
        (**self).body_names()
    }

    fn command_count(&self) -> usize {
        (**self).command_count()
    }

    fn site(&mut self, at: [f64; 2]) -> crate::Placement {
        (**self).site(at)
    }
}

/// The arguments of the `inspect.measure` call that lists the body names.
pub fn body_names_query() -> Value {
    serde_json::json!({"names_only": true})
}

/// The body names in the reply to `inspect.measure` with `body_names_query`.
pub fn measured_body_names(reply: &Value) -> Vec<String> {
    reply
        .get("bodies")
        .and_then(Value::as_array)
        .map(|bodies| {
            bodies
                .iter()
                .filter_map(|body| body.get("name").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// The id `sketch.create` returned. The host sends a number or a non-empty name.
pub fn sketch_id(reply: &Value) -> Result<Value, String> {
    match reply.get("sketch") {
        Some(id @ Value::Number(_)) => Ok(id.clone()),
        Some(Value::String(id)) if !id.is_empty() => Ok(Value::String(id.clone())),
        _ => Err("sketch.create did not return a sketch id".to_string()),
    }
}

/// The first body a solid feature named. Later cuts use this, because the host may rename it.
pub fn body_name_of(reply: &Value) -> Option<String> {
    reply
        .get("created_bodies")
        .and_then(Value::as_array)
        .and_then(|bodies| bodies.first())
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Adds each of `bodies` to `into` once, keeping the order they came in.
pub fn add_new(into: &mut Vec<String>, bodies: impl IntoIterator<Item = String>) {
    for body in bodies {
        if !into.contains(&body) {
            into.push(body);
        }
    }
}

/// The bodies a feature changed, when the host says. An empty list means a cut found
/// no part to cut.
pub fn changed_bodies(reply: &Value) -> Option<Vec<String>> {
    let changed = reply.get("changed_bodies").and_then(Value::as_array)?;
    Some(
        changed
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
    )
}

/// The bodies a cut into a part changed. A cut that found no part under the point is
/// refused: `cut` is what it was for (`to drill`) and `alone` is the Create choice that
/// needs no part. A host that does not say what changed gives an empty list.
pub fn cut_bodies(reply: &Value, cut: &str, alone: &str) -> Result<Vec<String>, String> {
    match changed_bodies(reply) {
        Some(bodies) if bodies.is_empty() => Err(format!(
            "sketch_point: there is no part under the point {cut}; put the sketch on the part, or set Create to {alone}"
        )),
        Some(bodies) => Ok(bodies),
        None => Ok(Vec::new()),
    }
}

/// JSON from a host command. A reply that is not JSON becomes null, and a failure names the command.
pub fn command_reply<E: std::fmt::Display>(
    command: &str,
    reply: Result<String, E>,
) -> Result<Value, String> {
    match reply {
        Ok(text) => Ok(serde_json::from_str(&text).unwrap_or(Value::Null)),
        Err(err) => Err(format!("{command}: {err}")),
    }
}

/// `"solid.thread: ..."` from a host command that failed: the command id and its reason.
pub fn command_failure(err: &str) -> Option<(&str, &str)> {
    let (command, reason) = err.split_once(": ")?;
    let id = command.contains('.')
        && command
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_');
    id.then_some((command, reason))
}

/// `"hole_depth: ..."` from the generator: a dialog field key and its reason. A command
/// id has a dot, so `command_failure` takes those.
pub fn input_field(err: &str) -> Option<(&str, &str)> {
    let (field, reason) = err.split_once(": ")?;
    let key = !field.is_empty()
        && field
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    key.then_some((field, reason))
}

/// Field keys and the titles the dialog shows for them, read from `plugin.json`, so an
/// error names a field the way the form does and the two cannot drift apart.
pub fn manifest_labels(manifest: &str) -> Vec<(String, String)> {
    let manifest: Value = serde_json::from_str(manifest).unwrap_or(Value::Null);
    let mut labels = Vec::new();
    let entries = manifest["runtime"]["entries"].as_array();
    for entry in entries.into_iter().flatten() {
        let properties = entry["input_schema"]["properties"].as_object();
        for (key, property) in properties.into_iter().flatten() {
            if let Some(title) = property["title"].as_str() {
                labels.push((key.clone(), title.to_string()));
            }
        }
    }
    labels
}

/// The reason with the field's dialog label in front. The host prints the field key,
/// and a person knows the label.
pub fn labelled<K: AsRef<str>, L: AsRef<str>>(
    labels: &[(K, L)],
    field: &str,
    reason: &str,
) -> String {
    match labels.iter().find(|(key, _)| key.as_ref() == field) {
        Some((_, label)) => format!("{}: {reason}", label.as_ref()),
        None => reason.to_string(),
    }
}

/// The wasm guest of a plugin. Every plugin exports the same thing: read the parameters,
/// build, and turn each failure into the host's error for it. Workflows are listed
/// under `entries`; only these insertion workflows are exported.
///
/// - `manifest`: `include_str!("../plugin.json")`, for the dialog labels.
/// - `parse`: `fn(&Value) -> Result<Params, FieldError>`.
/// - `make`: `fn(&mut impl Runner, &Params) -> Result<Value, String>`. An error that
///   starts with a field key and `": "` is that field's, one that starts with a command
///   id is that command's, and anything else is a trap.
/// - `describe`: `fn(&Params) -> String`, the line logged before building.
#[macro_export]
macro_rules! guest {
    (
        manifest: $manifest:expr,
        entries: [$({
            entry: $entry:literal,
            noun: $noun:literal,
            parse: $parse:path,
            make: $make:path,
            describe: $describe:expr $(,)?
        }),+ $(,)?] $(,)?
    ) => {
        #[cfg(target_arch = "wasm32")]
        mod guest {
            wit_bindgen::generate!({ path: "../../wit", world: "command-plugin" });

            #[allow(unused_imports)]
            use super::*;
            use craft::plugin::{
                commands, log,
                types::{CommandFailure, FieldFailure, PluginError},
            };

            #[derive(Default)]
            struct Host { calls: usize }
            impl $crate::Runner for Host {
                fn run(
                    &mut self,
                    command: &str,
                    args: serde_json::Value,
                ) -> Result<serde_json::Value, String> {
                    self.calls += 1;
                    $crate::host::command_reply(
                        command,
                        commands::run(command, &args.to_string()).map_err(words),
                    )
                }

                fn body_names(&mut self) -> Vec<String> {
                    use $crate::Runner as _;
                    self.run("inspect.measure", $crate::host::body_names_query())
                        .map(|reply| $crate::host::measured_body_names(&reply))
                        .unwrap_or_default()
                }

                fn command_count(&self) -> usize { self.calls }
            }

            /// A host error in plain words.
            fn words(e: PluginError) -> String {
                match e {
                    PluginError::PermissionDenied(text)
                    | PluginError::LimitExceeded(text)
                    | PluginError::Trap(text) => text,
                    PluginError::CommandFailed(failure) => failure.reason,
                    PluginError::InvalidManifest(failure)
                    | PluginError::InvalidInput(failure) => {
                        format!("{}: {}", failure.field, failure.reason)
                    }
                    PluginError::Cancelled => "cancelled".into(),
                }
            }

            fn invalid(field: &str, reason: &str) -> PluginError {
                let labels = $crate::host::manifest_labels($manifest);
                PluginError::InvalidInput(FieldFailure {
                    field: field.into(),
                    reason: $crate::host::labelled(&labels, field, reason),
                })
            }

            fn failure(e: String) -> PluginError {
                if let Some((field, reason)) = $crate::host::input_field(&e) {
                    return invalid(field, reason);
                }
                match $crate::host::command_failure(&e) {
                    Some((command, reason)) => PluginError::CommandFailed(CommandFailure {
                        command: command.into(),
                        reason: reason.into(),
                    }),
                    None => PluginError::Trap(e),
                }
            }

            struct Plugin;
            impl exports::craft::plugin::entry::Guest for Plugin {
                fn execute(entry: String, params: String) -> Result<String, PluginError> {
                    // `entry` is only empty if the host sends that, so this call stays in
                    // the component and the `commands` import is linked before the
                    // generator uses it.
                    if entry.is_empty() {
                        let _ = commands::run("inspect.measure", "{}");
                    }
                    let v: serde_json::Value = serde_json::from_str(&params)
                        .map_err(|err| invalid("params", &err.to_string()))?;
                    $(
                        if entry == $entry {
                            let p = $parse(&v).map_err(|err| invalid(&err.field, &err.reason))?;
                            log::log(log::Level::Info, &($describe)(&p));
                            let made = $make(&mut Host::default(), &p).map_err(failure)?;
                            if made["placed"]
                                .as_str()
                                .is_some_and(|at| at.contains("origin"))
                            {
                                log::log(
                                    log::Level::Warn,
                                    concat!(
                                        "nothing was selected, so the ",
                                        $noun,
                                        " is on the origin"
                                    ),
                                );
                            }
                            return Ok(made.to_string());
                        }
                    )+
                    Err(PluginError::InvalidInput(FieldFailure {
                        field: "entry".into(),
                        reason: format!("unknown entry {entry}"),
                    }))
                }
            }

            export!(Plugin);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_sketch_id_is_a_number_or_a_name() {
        assert_eq!(sketch_id(&json!({"sketch": 7})).unwrap(), json!(7));
        assert_eq!(
            sketch_id(&json!({"sketch": "Nut profile"})).unwrap(),
            json!("Nut profile")
        );
        assert!(sketch_id(&json!({"sketch": ""})).is_err());
        assert!(sketch_id(&json!({})).is_err());
    }

    #[test]
    fn the_first_created_body_is_the_name() {
        assert_eq!(
            body_name_of(&json!({"created_bodies": ["Nut", "Other"]})).as_deref(),
            Some("Nut")
        );
        assert!(body_name_of(&json!({})).is_none());
    }

    #[test]
    fn a_taken_body_name_gets_the_next_number() {
        let mut taken = measured_body_names(&json!({
            "bodies": [{"name": "Base", "definition": "Base"}, {"name": "Screw M3 x 12 socket"}, {"name": "Screw M3 x 12 socket (2)"}],
            "body_count": 3,
        }));
        assert_eq!(
            taken,
            ["Base", "Screw M3 x 12 socket", "Screw M3 x 12 socket (2)"]
        );
        assert_eq!(
            crate::name::reserve(&mut taken, "Screw M3 x 12 socket"),
            "Screw M3 x 12 socket (3)"
        );
        assert_eq!(
            crate::name::reserve(&mut taken, "Screw M3 x 12 socket"),
            "Screw M3 x 12 socket (4)"
        );
        assert_eq!(
            crate::name::reserve(&mut taken, "Insert M3 x 5.7"),
            "Insert M3 x 5.7"
        );
        assert_eq!(
            crate::name::reserve(&mut taken, "Insert M3 x 5.7"),
            "Insert M3 x 5.7 (2)"
        );
        assert!(measured_body_names(&json!(null)).is_empty());
        assert_eq!(body_names_query(), json!({"names_only": true}));
    }

    #[test]
    fn a_command_reply_keeps_json_and_names_the_command() {
        let ok: Result<String, String> = Ok(r#"{"sketch": 1}"#.into());
        assert_eq!(
            command_reply("sketch.create", ok).unwrap(),
            json!({"sketch": 1})
        );
        let bad: Result<String, &str> = Err("nope");
        assert_eq!(
            command_reply("solid.hole", bad).unwrap_err(),
            "solid.hole: nope"
        );
        let junk: Result<String, String> = Ok("not json".into());
        assert_eq!(command_reply("solid.hole", junk).unwrap(), Value::Null);
    }

    #[test]
    fn a_placement_error_names_its_field() {
        assert_eq!(
            input_field("sketch: Holes has no plane"),
            Some(("sketch", "Holes has no plane"))
        );
        assert_eq!(
            input_field("sketch_point: select a sketch point or a circle"),
            Some(("sketch_point", "select a sketch point or a circle"))
        );
        assert!(input_field("solid.extrude did not name the nut body").is_none());
    }

    #[test]
    fn labels_come_from_the_manifest_titles() {
        let manifest = r#"{"runtime": {"entries": [{"input_schema": {"properties": {
            "length": {"type": "number", "title": "Length"},
            "sketch_point": {"type": "string", "title": "Sketch point"},
            "untitled": {"type": "string"}
        }}}]}}"#;
        let labels = manifest_labels(manifest);
        assert_eq!(
            labelled(&labels, "length", "must be a number"),
            "Length: must be a number"
        );
        assert_eq!(
            labelled(&labels, "sketch_point", "no p9"),
            "Sketch point: no p9"
        );
        assert_eq!(labelled(&labels, "untitled", "x"), "x");
        assert!(manifest_labels("not json").is_empty());
    }

    #[test]
    fn a_failed_command_is_split_from_its_reason() {
        assert_eq!(
            command_failure("solid.thread: no thread M99 in the table"),
            Some(("solid.thread", "no thread M99 in the table"))
        );
        assert!(command_failure("solid.extrude did not name the nut body").is_none());
        assert!(command_failure("sketch_point: no point p9").is_none());
        assert_eq!(
            input_field("hole_depth: use Blind"),
            Some(("hole_depth", "use Blind"))
        );
        assert!(input_field("solid.hole: no body there").is_none());
        assert!(input_field("the insert body was not named").is_none());
    }

    #[test]
    fn a_reason_leads_with_the_dialog_label() {
        let labels = [("thread_length", "Thread length")];
        assert_eq!(
            labelled(&labels, "thread_length", "must be no longer than the shank"),
            "Thread length: must be no longer than the shank"
        );
        assert_eq!(
            labelled(&labels, "entry", "unknown entry x"),
            "unknown entry x"
        );
    }
}
