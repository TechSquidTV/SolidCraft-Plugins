//! A component definition and the exact occurrence path being edited.

use crate::Runner;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Activation {
    #[serde(rename = "active")]
    pub component: u64,
    // Required: a missing path is an unsupported host, not implicit activation.
    #[serde(rename = "active_path")]
    path: Vec<u64>,
}

impl Activation {
    pub fn read(host: &mut impl Runner) -> Result<Self, String> {
        serde_json::from_value(host.run("component.list", json!({}))?).map_err(|error| {
            format!("cannot inspect active component and occurrence path: {error}")
        })
    }

    pub fn root() -> Self {
        Self {
            component: 0,
            path: Vec::new(),
        }
    }

    /// `component.create` with parent 0 and activate=true selects this root occurrence.
    pub fn created(component: u64, occurrence: u64) -> Self {
        Self {
            component,
            path: vec![occurrence],
        }
    }

    pub fn restore(&self, host: &mut impl Runner) -> Result<(), String> {
        let mut args = json!({"component": self.component});
        if let Some(occurrence) = self.path.last() {
            args["occurrence"] = json!(occurrence);
            args["path"] = json!(self.path);
        }
        host.run("component.activate", args)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_context_is_not_treated_as_implicit_activation() {
        assert!(serde_json::from_value::<Activation>(json!({"active": 7})).is_err());
        let implicit: Activation =
            serde_json::from_value(json!({"active": 7, "active_path": []})).unwrap();
        let explicit: Activation =
            serde_json::from_value(json!({"active": 7, "active_path": [9, 12]})).unwrap();
        let other_parent: Activation =
            serde_json::from_value(json!({"active": 7, "active_path": [10, 12]})).unwrap();
        assert_ne!(implicit, explicit);
        assert_ne!(explicit, other_parent);
    }
}
