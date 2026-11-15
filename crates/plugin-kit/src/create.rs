//! The Create choice: the part alone, the part and the cut it goes in, or the cut alone.
//!
//! A screw goes in a hole, a nut in a pocket and an insert in its hole, so the three
//! plugins ask the same question with their own words. [`Words`] holds those words and
//! everything that is said with them: the choice's labels, the error when only a cut is
//! asked for and nothing is selected, and the note when a cut was skipped.

use serde_json::Value;

use crate::input::{self, FieldError};
use crate::place;

/// The dialog field.
pub const FIELD: &str = "create";

/// What a run makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Create {
    /// The part alone: `Screw only`.
    Part,
    /// The cut, then the part in it: `Screw and hole`.
    Both,
    /// The cut alone: `Hole only`.
    Cut,
}

impl Create {
    /// Whether the part is made.
    pub fn part(self) -> bool {
        self != Create::Cut
    }

    /// Whether the cut is made.
    pub fn cut(self) -> bool {
        self != Create::Part
    }
}

/// One plugin's words for its Create choice.
pub struct Words {
    /// The labels, as the dialog lists them.
    pub part_only: &'static str,
    pub both: &'static str,
    pub cut_only: &'static str,
    /// The choice the dialog opens on.
    pub default: Create,
    /// The part, as in "a loose screw": `screw`.
    pub part: &'static str,
    /// The cut, one and several: `hole`, `holes`.
    pub cut: &'static str,
    pub cuts: &'static str,
    /// Making the cut, and having made it: `drill`, `drilled`.
    pub make: &'static str,
    pub made: &'static str,
}

impl Words {
    /// The dialog's label for `create`.
    pub fn label(&self, create: Create) -> &'static str {
        match create {
            Create::Part => self.part_only,
            Create::Both => self.both,
            Create::Cut => self.cut_only,
        }
    }

    /// The `create` field, or the default when it is empty.
    pub fn parse(&self, v: &Value) -> Result<Create, FieldError> {
        let text = input::choice(v, FIELD, self.label(self.default))?;
        [Create::Part, Create::Both, Create::Cut]
            .into_iter()
            .find(|&create| self.label(create) == text)
            .ok_or_else(|| {
                FieldError::new(
                    FIELD,
                    format!(
                        "must be {}, {} or {}, not {text}",
                        self.part_only, self.both, self.cut_only
                    ),
                )
            })
    }

    /// With nothing selected there is no part to cut, so the part alone goes on the
    /// origin. Returns whether a cut that was asked for is skipped, or the error when the
    /// cut was all that was asked for.
    pub fn nothing_selected(&self, create: Create) -> Result<bool, String> {
        if !create.part() {
            return Err(place::nothing_selected(
                self.cuts,
                &format!("{} for a loose {} on the origin", self.part_only, self.part),
            ));
        }
        Ok(create.cut())
    }

    /// The reply's note when a cut was skipped because nothing was selected.
    pub fn skipped(&self) -> String {
        format!(
            "no {} was {} because nothing was selected; select the sketch points or circles where the {} go to {} them",
            self.cut, self.made, self.cuts, self.make
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SCREW: Words = Words {
        part_only: "Screw only",
        both: "Screw and hole",
        cut_only: "Hole only",
        default: Create::Part,
        part: "screw",
        cut: "hole",
        cuts: "holes",
        make: "drill",
        made: "drilled",
    };

    #[test]
    fn a_label_is_read_back_and_an_empty_field_is_the_default() {
        for create in [Create::Part, Create::Both, Create::Cut] {
            assert_eq!(
                SCREW.parse(&json!({"create": SCREW.label(create)})),
                Ok(create)
            );
        }
        assert_eq!(SCREW.parse(&json!({})), Ok(Create::Part));
        assert_eq!(SCREW.parse(&json!({"create": null})), Ok(Create::Part));
        let err = SCREW.parse(&json!({"create": "Nut only"})).unwrap_err();
        assert_eq!(err.field, "create");
        assert_eq!(
            err.reason,
            "must be Screw only, Screw and hole or Hole only, not Nut only"
        );
        assert_eq!(
            SCREW.parse(&json!({"create": 3})).unwrap_err().field,
            "create"
        );
    }

    #[test]
    fn what_each_choice_makes() {
        assert!(Create::Part.part() && !Create::Part.cut());
        assert!(Create::Both.part() && Create::Both.cut());
        assert!(!Create::Cut.part() && Create::Cut.cut());
    }

    #[test]
    fn with_nothing_selected_the_part_is_made_and_a_cut_alone_is_refused() {
        assert_eq!(SCREW.nothing_selected(Create::Part), Ok(false));
        assert_eq!(SCREW.nothing_selected(Create::Both), Ok(true));
        assert_eq!(
            SCREW.nothing_selected(Create::Cut),
            Err("sketch_point: select the sketch points or circles where the holes go, then run this again; or set Create to Screw only for a loose screw on the origin".to_string())
        );
        assert_eq!(
            SCREW.skipped(),
            "no hole was drilled because nothing was selected; select the sketch points or circles where the holes go to drill them"
        );
    }
}
