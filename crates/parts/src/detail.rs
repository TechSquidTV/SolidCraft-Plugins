//! Thread representation is independent of the purchasing configuration.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThreadDetail {
    #[default]
    Cosmetic,
    Modeled,
}

impl ThreadDetail {
    pub const CHOICES: [&'static str; 2] = ["cosmetic", "modeled"];
    pub fn modeled(self) -> bool {
        self == Self::Modeled
    }
    pub fn label(modeled: bool) -> &'static str {
        if modeled {
            "modeled"
        } else {
            "cosmetic"
        }
    }
}
