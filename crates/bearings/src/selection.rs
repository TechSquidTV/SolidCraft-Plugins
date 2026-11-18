//! Match nominal envelopes across bearing families. Radial sizes and closures follow.
use crate::Envelope;

/// Partial nominal dimensions in mm. Unspecified axes place no restriction.
#[derive(Clone, Copy, Default)]
pub struct EnvelopeFilter {
    pub bore: Option<f64>,
    pub diameter: Option<f64>,
    pub width: Option<f64>,
}

/// A partial envelope may identify several catalog entries, even after filtering by closure.
pub enum EnvelopeMatch<T> {
    None,
    One(T),
    Ambiguous(String),
}

impl EnvelopeFilter {
    fn matches(self, e: Envelope) -> bool {
        self.bore.is_none_or(|x| x == e.bore)
            && self.diameter.is_none_or(|x| x == e.diameter)
            && self.width.is_none_or(|x| x == e.width)
    }

    pub fn select<T: Copy>(
        self,
        candidates: impl Iterator<Item = T>,
        describe: impl Fn(T) -> (&'static str, Envelope),
    ) -> EnvelopeMatch<T> {
        let matches: Vec<_> = candidates
            .filter_map(|item| {
                let (name, envelope) = describe(item);
                self.matches(envelope).then_some((item, name, envelope))
            })
            .collect();
        match matches.as_slice() {
            [] => EnvelopeMatch::None,
            [(item, _, _)] => EnvelopeMatch::One(*item),
            [(_, _, first), others @ ..] => {
                let hint = if others.iter().any(|(_, _, d)| d.bore != first.bore) {
                    "Enter the bore"
                } else if others.iter().any(|(_, _, d)| d.diameter != first.diameter) {
                    "Enter the outside diameter"
                } else if others.iter().any(|(_, _, d)| d.width != first.width) {
                    "Enter the width"
                } else {
                    "Choose a preset"
                };
                let names = matches
                    .iter()
                    .map(|(_, name, d)| format!("{name} ({d} mm)"))
                    .collect::<Vec<_>>()
                    .join(", ");
                EnvelopeMatch::Ambiguous(format!("matches {names}. {hint} to narrow the selection"))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Closure {
    MetalShields,
    RubberSeals,
    Open,
}

impl Closure {
    pub const ALL: [Self; 3] = [Self::MetalShields, Self::RubberSeals, Self::Open];

    pub fn label(self) -> &'static str {
        match self {
            Self::MetalShields => "Shielded",
            Self::RubberSeals => "Sealed",
            Self::Open => "Open",
        }
    }
}

macro_rules! sizes {
    ($( $variant:ident, $number:literal, $bore:literal, $diameter:literal, $width:literal; )*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Size { $( $variant, )* }
        impl Size {
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];
            pub fn label(self) -> &'static str {
                match self { $(Self::$variant => stringify!($number),)* }
            }
            pub fn envelope(self) -> Envelope {
                match self { $(Self::$variant => Envelope { bore: $bore, diameter: $diameter, width: $width },)* }
            }
        }
    };
}

// Nominal mm, verified against docs/sources/{openbuilds,misumi}-measurements.json.
sizes! {
    S105, 105, 5.0, 10.0, 4.0;
    S608, 608, 8.0, 22.0, 7.0;
    S625, 625, 5.0, 16.0, 5.0;
    S626, 626, 6.0, 19.0, 6.0;
    S6000, 6000, 10.0, 26.0, 8.0;
    S6001, 6001, 12.0, 28.0, 8.0;
    S6002, 6002, 15.0, 32.0, 9.0;
    S6003, 6003, 17.0, 35.0, 10.0;
    S6004, 6004, 20.0, 42.0, 12.0;
    S6200, 6200, 10.0, 30.0, 9.0;
    S6201, 6201, 12.0, 32.0, 10.0;
    S6202, 6202, 15.0, 35.0, 11.0;
    S6203, 6203, 17.0, 40.0, 12.0;
    S6204, 6204, 20.0, 47.0, 14.0;
}
