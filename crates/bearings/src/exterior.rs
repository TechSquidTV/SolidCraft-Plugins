//! Evidence for the exterior section, independent of the product's purchasing identity.
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    Measured,
    Estimated,
}

impl Basis {
    pub fn label(self) -> &'static str {
        match self {
            Self::Measured => "Measured",
            Self::Estimated => "Estimated",
        }
    }
}

#[derive(Serialize)]
pub struct Provenance<S> {
    pub basis: Basis,
    pub method: &'static str,
    pub sources: S,
}
