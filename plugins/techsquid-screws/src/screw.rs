//! Screw parameters. Sizes are millimetres.
//!
//! The top of the shank sits on a sketch point. The sketch normal points outward: the
//! head stands on that side and the shank runs the other way. The shank is a cylinder of
//! the thread's major diameter, because `solid.thread` refuses a thicker shaft. A point
//! shortens that cylinder and leaves the last diameter for a cone. A cup is a recess in
//! the end. This module resolves the size. The crate root builds the solid.

use fasteners::clearance;
use fasteners::designation;
use fasteners::head::{self, Crown, HeadKind};
use fasteners::hexsocket::{self, Socket};
use fasteners::metric;
use fasteners::phillips::{self, Tool};
use fasteners::slot::{self, Slot};
use fasteners::torx::{self, Recess};
pub use plugin_kit::create::Create;
use plugin_kit::create::Words;
use plugin_kit::input::{self, FieldError};
use plugin_kit::place::{self, PlaceInput};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum Head {
    Socket,
    Button,
    Countersunk,
    Hex,
    Pan,
    Cheese,
    Set,
    Rod,
}

impl Head {
    /// The default M3 family designation used by catalog discovery. Actual
    /// resolution follows the chosen drive, tip and dimensions through the same
    /// standard lookup as the insertion workflow.
    pub fn default_standard(self) -> Option<&'static str> {
        standard_of(self, standard_drive(self, Some(3.0)), Tip::Flat, false)
    }
    pub const ALL: [Self; 8] = [
        Self::Socket,
        Self::Button,
        Self::Countersunk,
        Self::Hex,
        Self::Pan,
        Self::Cheese,
        Self::Set,
        Self::Rod,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::Socket => "socket",
            Self::Button => "button",
            Self::Countersunk => "countersunk",
            Self::Hex => "hex",
            Self::Pan => "pan",
            Self::Cheese => "cheese",
            Self::Set => "set",
            Self::Rod => "rod",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Head::Socket => "Socket head",
            Head::Button => "Button head",
            Head::Countersunk => "Countersunk",
            Head::Hex => "Hex head",
            Head::Pan => "Pan head",
            Head::Cheese => "Cheese head",
            Head::Set => "Set screw",
            Head::Rod => "Threaded rod",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum Drive {
    HexSocket,
    Phillips,
    Slotted,
    Torx,
    None,
}

impl Drive {
    pub fn label(self) -> &'static str {
        match self {
            Drive::HexSocket => "Hex socket",
            Drive::Phillips => "Phillips",
            Drive::Slotted => "Slotted",
            Drive::Torx => "Torx",
            Drive::None => "None",
        }
    }
}

/// The end of the shank. Flat is the square end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum Tip {
    Flat,
    Point,
    Cup,
}

impl Tip {
    pub fn label(self) -> &'static str {
        match self {
            Tip::Flat => "Flat",
            Tip::Point => "Point",
            Tip::Cup => "Cup",
        }
    }
}

/// Tip after the major diameter is known. A point's length is that diameter.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum ResolvedTip {
    Flat,
    /// ISO 4026 flat point of a set screw: the end is chamfered at 45° down to a flat
    /// `diameter` across.
    FlatPoint {
        diameter: f64,
    },
    Point {
        length: f64,
    },
    /// `diameter` is the mouth on the end face. `depth` is how far the 120° cone reaches.
    Cup {
        diameter: f64,
        depth: f64,
    },
}

/// Head size after a standard lookup or an override. Ready for the generator.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum ResolvedHead {
    None,
    Round { diameter: f64, height: f64 },
    Hex { across_flats: f64, height: f64 },
    Countersunk { diameter: f64, height: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resolved {
    pub major: Option<f64>,
    pub pitch: Option<f64>,
    pub head: ResolvedHead,
    /// The top of a round head. Flat when the head size was typed.
    pub crown: Crown,
    pub tip: ResolvedTip,
}

/// What a run makes. A hole needs a part under the point, so the default is the screw
/// alone, which also stands on the origin with nothing selected.
pub const CREATE: Words = Words {
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

pub const CLEARANCE: &str = "Clearance";
pub const RECESSED: &str = "Clearance, head recessed";
pub const TAPPED: &str = "Tapped";

/// Radial gap between a head and its counterbore. ISO counterbores for socket head
/// screws leave about this much: 6.5 mm for the 5.5 mm head of an M3.
pub const HEAD_GAP: f64 = 0.5;

/// The hole for this screw in the part under it.
#[derive(Clone, Debug, PartialEq)]
pub enum HoleKind {
    /// An ISO 273 clearance hole. The head sits on the surface.
    Clearance { size: clearance::Hole },
    /// A clearance hole with a counterbore the head sits in, flush with the surface.
    Counterbore {
        size: clearance::Hole,
        diameter: f64,
        depth: f64,
    },
    /// A clearance hole with a 90° countersink `diameter` across at the surface.
    Countersink {
        size: clearance::Hole,
        diameter: f64,
    },
    /// A hole tapped for the screw's own thread.
    Tapped { thread: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hole {
    pub kind: HoleKind,
    /// Depth from the surface: as far as the screw reaches.
    pub depth: f64,
    /// How far the screw sits below the surface, so a recessed head ends flush.
    pub sink: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub custom: bool,
    pub create: Create,
    /// The hole to drill in the part. Empty when only the screw is made.
    pub hole: Option<Hole>,
    /// The standard the head is sized from. Empty for a typed head size.
    pub standard: Option<&'static str>,
    /// Thread size as `solid.thread` writes it: `M3`, `M8x1`, `1/2-10 ACME`.
    pub designation: String,
    /// The field the size came from, `size` or `custom_size`, for errors about it.
    pub size_field: &'static str,
    /// The length as typed and as the standard lists it: under the head, overall for
    /// a countersunk head, a set screw and a threaded rod.
    pub nominal_length: f64,
    /// Length of the shank under the head. The nominal length less a countersunk head.
    pub length: f64,
    pub head: Head,
    pub drive: Drive,
    pub tip: Tip,
    /// Override for the standard head. One of the two is enough, except a countersunk head.
    pub head_diameter: Option<f64>,
    pub head_height: Option<f64>,
    /// Threaded length from where the thread starts. On a point, that is the shoulder
    /// where the cone meets the cylinder. `None` means the whole cylindrical face.
    pub thread_length: Option<f64>,
    /// `solid.thread`'s `modeled`. The dialog calls this `thread_modeled`.
    pub modeled: bool,
    /// Where the top of the shank sits. Both empty uses the selection, or the origin.
    pub place: PlaceInput,
    /// ISO 10664 recess. Set when `drive` is Torx.
    pub recess: Option<Recess>,
    /// Straight slot. Set when `drive` is Slotted.
    pub slot: Option<Slot>,
    /// ISO 4757 type H recess. Set when `drive` is Phillips.
    pub phillips: Option<Tool>,
    /// Hexagon socket. Set when `drive` is Hex socket.
    pub hex: Option<Socket>,
    pub resolved: Resolved,
}

pub fn parse(v: &Value) -> Result<Params, FieldError> {
    input::workflow_fields(v, include_str!("../plugin.json"), "make-screw")?;
    let size_field = designation::size_field(v);
    let at_size = |err: FieldError| err.moved("size", size_field);
    let designation = designation::designation(v, "M3")?;
    let nominal_length = input::length_or(v, "length", 12.0)?;
    let head = head(v)?;
    let asked_drive = drive(v)?;
    let tip = tip(v)?;
    let head_diameter = input::optional_length(v, "head_diameter")?;
    let head_height = input::optional_length(v, "head_height")?;
    let thread_length = input::optional_length(v, "thread_length")?;
    let modeled = input::flag(v, "thread_modeled", true)?;
    let place = place::parse_input(v)?;
    let create = CREATE.parse(v)?;
    // The drive comes first: a head is sized from the standard of its drive.
    let major = metric::parse_metric(&designation)
        .map_err(at_size)?
        .map(|size| size.major);
    let drive = asked_drive.unwrap_or_else(|| standard_drive(head, major));
    let resolved = match resolve(&designation, head, drive, tip, head_diameter, head_height) {
        Ok(resolved) => resolved,
        // A hole alone needs the head only to recess it. Without one it is as deep as
        // the length.
        Err(_) if !create.part() && hole_type(v)? != RECESSED => Resolved {
            major,
            pitch: metric::parse_metric(&designation)
                .map_err(at_size)?
                .map(|size| size.pitch),
            head: ResolvedHead::None,
            crown: Crown::Flat,
            tip: ResolvedTip::Flat,
        },
        Err(err) => return Err(at_size(err)),
    };
    // ISO 10642, ISO 2009 and ISO 7046-1 measure a countersunk screw overall, head and
    // all. The shank under the head is what is left.
    let length = match resolved.head {
        ResolvedHead::Countersunk { height, .. } => {
            let under = nominal_length - height;
            if under <= 0.0 {
                return Err(FieldError::new(
                    "length",
                    format!(
                        "a countersunk screw's length includes its {} mm head, so it has to be longer than that",
                        metric::number(height)
                    ),
                ));
            }
            under
        }
        _ => nominal_length,
    };
    // An empty Thread length is the standard one: `b` on a screw long enough to have
    // a plain shank, and the whole shank on a shorter one.
    let thread_length = thread_length
        .or_else(|| standard_thread_length(head, drive, &resolved, length, nominal_length));
    // A hole alone has no drive, so a drive that would not fit the screw is no matter.
    let (recess, slot, phillips, hex) = if create.part() {
        let text = input::optional_text(
            v,
            "drive_size",
            "must be text, such as T10, PH2, 2.5 or 0.8x0.7",
        )?;
        if drive == Drive::None && text.is_some() {
            return Err(FieldError::new(
                "drive_size",
                "this screw has no drive to size; choose a Drive, or leave this empty",
            ));
        }
        let to_box = |err: FieldError| {
            if matches!(err.field.as_str(), "torx" | "slot" | "phillips" | "hex") {
                FieldError::new("drive_size", err.reason)
            } else {
                err
            }
        };
        let requested = if drive == Drive::Torx {
            text.map(torx::parse_socket).transpose().map_err(to_box)?
        } else {
            None
        };
        let requested_slot = if drive == Drive::Slotted {
            text.map(slot::parse_size).transpose().map_err(to_box)?
        } else {
            None
        };
        let requested_phillips = if drive == Drive::Phillips {
            text.map(phillips::parse_size).transpose().map_err(to_box)?
        } else {
            None
        };
        let requested_hex = if drive == Drive::HexSocket {
            text.map(hexsocket::parse_size)
                .transpose()
                .map_err(to_box)?
        } else {
            None
        };
        (
            plan_recess(drive, head, requested, &resolved, length).map_err(to_box)?,
            plan_slot(drive, head, requested_slot, &resolved, length).map_err(to_box)?,
            plan_phillips(drive, head, requested_phillips, &resolved, length).map_err(to_box)?,
            plan_hex(drive, head, requested_hex, &resolved, length).map_err(to_box)?,
        )
    } else {
        (None, None, None, None)
    };
    let hole = if create.cut() {
        Some(plan_hole(v, &designation, &resolved, length)?)
    } else {
        None
    };
    let standard = standard_of(
        head,
        drive,
        tip,
        head_diameter.is_some() || head_height.is_some(),
    );
    let mut p = Params {
        custom: standard.is_none()
            || tip == Tip::Point
            || (tip == Tip::Cup && (head, drive) != (Head::Set, Drive::HexSocket))
            || resolved.major.is_none_or(|major| {
                metric::parse_metric(&format!("M{}", metric::number(major)))
                    .ok()
                    .flatten()
                    .is_none_or(|coarse| Some(coarse.pitch) != resolved.pitch)
            })
            || v.get("thread_length").is_some_and(|v| !v.is_null())
            || v.get("drive_size")
                .is_some_and(|v| v.as_str().is_some_and(|s| !s.trim().is_empty())),
        create,
        hole,
        standard,
        designation,
        size_field,
        nominal_length,
        length,
        head,
        drive,
        tip,
        head_diameter,
        head_height,
        thread_length,
        modeled,
        place,
        recess,
        slot,
        phillips,
        hex,
        resolved,
    };
    // Up to the full length means all of it: the cylinder above a point, and between
    // the chamfers on a set screw, is the most a thread can cover.
    if let Some(thread) = p.thread_length {
        let room = p.thread_room();
        if thread > room && thread <= p.length + 1e-9 && room > 0.0 {
            p.thread_length = Some(room);
        }
    }
    validate(&p)?;
    parts::catalog::validate_selection(
        v,
        crate::provider::REVISION,
        &crate::provider::representation(&p),
    )
    .map_err(|e| FieldError::new(e.field, e.reason))?;
    Ok(p)
}

pub fn validate(p: &Params) -> Result<(), FieldError> {
    designation::check_designation(&p.designation)
        .map_err(|err| err.moved("size", p.size_field))?;
    input::check_length("length", p.nominal_length)?;
    if !p.create.part() {
        return Ok(());
    }
    if let Some(pitch) = p.resolved.pitch {
        if let ResolvedTip::Point { length: point } = p.resolved.tip {
            if point > 0.0 && p.length - point < pitch {
                let least = point + pitch;
                return Err(FieldError::new(
                    "length",
                    format!(
                        "a point on this size is {point} mm of the length, so the screw has to be at least {least} mm"
                    ),
                ));
            }
        }
        let ends = p.point_chamfer().unwrap_or(0.0) + p.socket_end_chamfer_length().unwrap_or(0.0);
        if ends > 0.0 && p.length - ends < pitch {
            let least = metric::number(ends + pitch);
            return Err(FieldError::new(
                "length",
                format!(
                    "the chamfers on the ends of this set screw take {} mm, so it has to be at least {least} mm",
                    metric::number(ends)
                ),
            ));
        }
    }
    if let Some(thread) = p.thread_length {
        input::check_length("thread_length", thread)?;
        if thread > p.thread_room() + 1e-9 {
            return Err(FieldError::new(
                "thread_length",
                "must be no longer than the shank",
            ));
        }
    }
    Ok(())
}

fn resolve(
    designation: &str,
    head: Head,
    drive: Drive,
    tip: Tip,
    diameter: Option<f64>,
    height: Option<f64>,
) -> Result<Resolved, FieldError> {
    let size = metric::parse_metric(designation)?;
    let (major, pitch) = match size {
        Some(size) => (Some(size.major), Some(size.pitch)),
        None => (None, None),
    };
    Ok(Resolved {
        major,
        pitch,
        head: resolve_head(head, drive, major, diameter, height)?,
        crown: match (kind_of(head, drive), major) {
            // A typed head size has no standard top to go with it.
            (Some(kind), Some(major)) if diameter.is_none() && height.is_none() => {
                head::crown(kind, major)
            }
            _ => Crown::Flat,
        },
        tip: resolve_tip(tip, head, major),
    })
}

fn resolve_tip(tip: Tip, head: Head, major: Option<f64>) -> ResolvedTip {
    let Some(major) = major else {
        return match tip {
            Tip::Flat => ResolvedTip::Flat,
            Tip::Point => ResolvedTip::Point { length: 0.0 },
            Tip::Cup => ResolvedTip::Cup {
                diameter: 0.0,
                depth: 0.0,
            },
        };
    };
    match tip {
        // A set screw's flat end is the ISO 4026 flat point, where the standard lists it.
        Tip::Flat if head == Head::Set => match hexsocket::flat_point(major) {
            Some(diameter) => ResolvedTip::FlatPoint { diameter },
            None => ResolvedTip::Flat,
        },
        Tip::Flat => ResolvedTip::Flat,
        // One major diameter, the long end of an ISO 1478 Type C gimlet.
        Tip::Point => ResolvedTip::Point { length: major },
        Tip::Cup => {
            let diameter = cup_diameter(major);
            ResolvedTip::Cup {
                diameter,
                depth: cup_depth(diameter),
            }
        }
    }
}

/// The mouth of the cup: ISO 4029 `dz` max, or half the diameter for a size that
/// standard does not list. A fine pitch uses the same mouth.
fn cup_diameter(major: f64) -> f64 {
    hexsocket::cup_point(major).unwrap_or(major / 2.0)
}

/// Depth of a 120° cup. The side sits 60° off the axis, so depth = (dz / 2) / tan(60°).
fn cup_depth(diameter: f64) -> f64 {
    diameter / (2.0 * 3.0_f64.sqrt())
}

fn resolve_head(
    head: Head,
    drive: Drive,
    major: Option<f64>,
    diameter: Option<f64>,
    height: Option<f64>,
) -> Result<ResolvedHead, FieldError> {
    if let Some(noun) = headless(head) {
        if diameter.is_some() || height.is_some() {
            let field = if diameter.is_some() {
                "head_diameter"
            } else {
                "head_height"
            };
            return Err(FieldError::new(
                field,
                format!("{noun} has no head; leave the head diameter and height empty"),
            ));
        }
        return Ok(ResolvedHead::None);
    }

    let (diameter, height) = match (diameter, height) {
        (Some(diameter), Some(height)) => {
            checked_override(head, major, diameter, height)?;
            (diameter, height)
        }
        (None, None) => {
            let standard = lookup_standard(head, drive, major).ok_or_else(missing_standard)?;
            (standard.diameter, standard.height)
        }
        (Some(diameter), None) => {
            if head == Head::Countersunk {
                return Err(both_dimensions("head_height"));
            }
            let standard =
                lookup_standard(head, drive, major).ok_or_else(|| no_standard("head_height"))?;
            input::check_length("head_diameter", diameter)?;
            if let Some(major) = major {
                clears_the_thread(head, diameter, major)?;
            }
            (diameter, standard.height)
        }
        (None, Some(height)) => {
            if head == Head::Countersunk {
                return Err(both_dimensions("head_diameter"));
            }
            let standard =
                lookup_standard(head, drive, major).ok_or_else(|| no_standard("head_diameter"))?;
            input::check_length("head_height", height)?;
            (standard.diameter, height)
        }
    };
    Ok(finish(head, diameter, height))
}

fn checked_override(
    head: Head,
    major: Option<f64>,
    diameter: f64,
    height: f64,
) -> Result<(), FieldError> {
    input::check_length("head_diameter", diameter)?;
    input::check_length("head_height", height)?;
    if let Some(major) = major {
        clears_the_thread(head, diameter, major)?;
    }
    Ok(())
}

fn lookup_standard(head: Head, drive: Drive, major: Option<f64>) -> Option<head::StandardHead> {
    let major = major?;
    let kind = kind_of(head, drive)?;
    head::standard_head(kind, major)
}

const BOTH_DIMENSIONS: &str = "set the head diameter and the head height, or leave both empty";
const NO_STANDARD: &str = "this size has no standard head here; set the head diameter and height";

fn both_dimensions(field: &str) -> FieldError {
    FieldError::new(field, BOTH_DIMENSIONS)
}

fn no_standard(field: &str) -> FieldError {
    FieldError::new(field, NO_STANDARD)
}

fn missing_standard() -> FieldError {
    no_standard("head_diameter")
}

fn clears_the_thread(head: Head, diameter: f64, major: f64) -> Result<(), FieldError> {
    if diameter > major {
        return Ok(());
    }
    let reason = if head == Head::Hex {
        "must be wider than the thread"
    } else {
        "must be larger than the thread"
    };
    Err(FieldError::new("head_diameter", reason))
}

fn headless(head: Head) -> Option<&'static str> {
    match head {
        Head::Set => Some("a set screw"),
        Head::Rod => Some("a threaded rod"),
        _ => None,
    }
}

/// The standard that sizes this head, which follows the drive. A pan head with a
/// Phillips recess is the ISO 7045 head, and any other pan is the slotted ISO 1580
/// (ISO 1580) head. A countersunk head with a slot or a Phillips recess is the flat
/// ISO 2009 / ISO 7046-1 head, and any other is the hexagon socket ISO 10642 head.
fn kind_of(head: Head, drive: Drive) -> Option<HeadKind> {
    Some(match head {
        Head::Pan if drive == Drive::Phillips => HeadKind::PanCross,
        Head::Countersunk if matches!(drive, Drive::Phillips | Drive::Slotted) => {
            HeadKind::CountersunkFlat
        }
        Head::Socket => HeadKind::Socket,
        Head::Button => HeadKind::Button,
        Head::Countersunk => HeadKind::Countersunk,
        Head::Hex => HeadKind::Hex,
        Head::Pan => HeadKind::Pan,
        Head::Cheese => HeadKind::Cheese,
        Head::Set | Head::Rod => return None,
    })
}

fn finish(head: Head, diameter: f64, height: f64) -> ResolvedHead {
    match head {
        Head::Hex => ResolvedHead::Hex {
            across_flats: diameter,
            height,
        },
        Head::Countersunk => ResolvedHead::Countersunk { diameter, height },
        Head::Socket | Head::Button | Head::Pan | Head::Cheese => {
            ResolvedHead::Round { diameter, height }
        }
        Head::Set | Head::Rod => ResolvedHead::None,
    }
}

/// Material left outside the lobe tips, and under the recess.
const TORX_WALL: f64 = 0.2;
const TORX_FLOOR: f64 = 0.3;
const TORX_MIN_DEPTH: f64 = 0.4;

/// Material left beside the slot, and under it. The ISO 1580 M1.6 slot is 0.35 mm
/// deep, so a slot is not held to the Torx minimum depth.
const SLOT_WALL: f64 = 0.2;
const SLOT_FLOOR: f64 = 0.3;

/// The recess for a Torx drive. A Torx size on another drive is refused.
fn plan_recess(
    drive: Drive,
    head: Head,
    requested: Option<u16>,
    resolved: &Resolved,
    length: f64,
) -> Result<Option<torx::Recess>, FieldError> {
    if drive != Drive::Torx {
        if requested.is_some() {
            return Err(FieldError::new(
                "torx",
                "leave this empty, or set Drive to Torx",
            ));
        }
        return Ok(None);
    }
    if head == Head::Rod {
        return Err(FieldError::new(
            "drive",
            "a threaded rod has no drive; set Drive to None",
        ));
    }
    let socket = match requested {
        Some(number) => torx::socket(number)
            .ok_or_else(|| FieldError::new("torx", "must be a socket such as T10"))?,
        None => {
            let major = resolved.major.ok_or_else(|| {
                FieldError::new(
                    "torx",
                    "this size has no standard Torx socket; set one, such as T10",
                )
            })?;
            let number = torx::for_nominal(major).ok_or_else(|| {
                FieldError::new(
                    "torx",
                    "this size has no standard Torx socket; set one, such as T10",
                )
            })?;
            let Some(socket) = torx::socket(number) else {
                return Err(FieldError::new("torx", "must be a socket such as T10"));
            };
            socket
        }
    };
    let room = recess_room(resolved, length);
    let depth = socket.penetration().min(room - TORX_FLOOR);
    if depth < TORX_MIN_DEPTH {
        return Err(FieldError::new(
            "torx",
            format!(
                "a T{} recess is {} mm deep and this screw leaves less than {TORX_MIN_DEPTH} mm for it",
                socket.number,
                socket.penetration()
            ),
        ));
    }
    if let Some(span) = span_at(resolved, depth) {
        if span < socket.across_tips + 2.0 * TORX_WALL {
            let why = if requested.is_none() {
                format!(
                    "T{} is {} mm across the lobes and does not fit in this screw; set a smaller Torx",
                    socket.number, socket.across_tips
                )
            } else {
                format!(
                    "T{} is {} mm across the lobes and does not fit in this screw",
                    socket.number, socket.across_tips
                )
            };
            return Err(FieldError::new("torx", why));
        }
    }
    Ok(Some(torx::Recess {
        number: socket.number,
        depth,
    }))
}

/// The slot for a slotted drive. A slot size on another drive is refused.
fn plan_slot(
    drive: Drive,
    head: Head,
    requested: Option<Slot>,
    resolved: &Resolved,
    length: f64,
) -> Result<Option<Slot>, FieldError> {
    if drive != Drive::Slotted {
        if requested.is_some() {
            return Err(FieldError::new(
                "slot",
                "leave this empty, or set Drive to Slotted",
            ));
        }
        return Ok(None);
    }
    if head == Head::Rod {
        return Err(FieldError::new(
            "drive",
            "a threaded rod has no drive; set Drive to None",
        ));
    }
    let slot = match requested {
        Some(slot) => slot,
        None => {
            let major = resolved.major.ok_or_else(|| {
                FieldError::new(
                    "slot",
                    "this size has no standard slot; set one, such as 0.8x0.7",
                )
            })?;
            let found = match head {
                Head::Set => slot::for_set(major),
                Head::Cheese => slot::for_cheese(major),
                Head::Countersunk => slot::for_countersunk(major),
                _ => slot::for_headed(major),
            };
            found.ok_or_else(|| {
                FieldError::new(
                    "slot",
                    "this size has no standard slot; set one, such as 0.8x0.7",
                )
            })?
        }
    };
    let room = recess_room(resolved, length);
    if room < slot.depth + SLOT_FLOOR {
        return Err(FieldError::new(
            "slot",
            format!(
                "a {} mm slot is {} mm deep and this screw leaves less than {SLOT_FLOOR} mm under it",
                slot.width, slot.depth
            ),
        ));
    }
    if let Some(span) = span_at(resolved, slot.depth) {
        if span < slot.width + 2.0 * SLOT_WALL {
            return Err(FieldError::new(
                "slot",
                format!("a {} mm slot does not fit in this screw", slot.width),
            ));
        }
    }
    Ok(Some(slot))
}

/// Material left outside the wing tips, and under the point.
///
/// The floor is 0.1 mm. ISO 7046-1 gauges an M3 recess at 1.7 mm in a countersunk
/// head 1.65 mm tall, so the point of that standard screw is in the shank. An ISO 7045 pan
/// head is 2.4 mm tall for a 1.7 mm recess, so it has room to spare.
const PHILLIPS_WALL: f64 = 0.2;
const PHILLIPS_FLOOR: f64 = 0.1;

fn missing_phillips(head: Head) -> FieldError {
    let example = if head == Head::Set {
        "PH1x0.8"
    } else {
        "PH1x1.7"
    };
    FieldError::new(
        "phillips",
        format!("this size has no standard Phillips recess; set one, such as {example}"),
    )
}

/// The recess from the product standard. A set screw has no such row.
fn standard_phillips(head: Head, major: f64) -> Option<phillips::Standard> {
    if head == Head::Countersunk {
        phillips::countersunk(major)
    } else if head == Head::Set || head == Head::Cheese {
        // DIN 84 is slotted only. A cross-recessed cheese head needs a Drive size.
        None
    } else {
        phillips::headed(major)
    }
}

/// The type H recess for a Phillips drive. A Phillips size on another drive is refused.
///
/// A flat-top head takes the ISO 7045 penetration. A countersunk head takes the
/// ISO 7046 penetration. The opening is checked on the outer face, and the wing
/// width `g` is checked again where the 28° point begins, because a countersunk
/// cone narrows faster than the wings.
fn plan_phillips(
    drive: Drive,
    head: Head,
    requested: Option<phillips::Request>,
    resolved: &Resolved,
    length: f64,
) -> Result<Option<Tool>, FieldError> {
    if drive != Drive::Phillips {
        if requested.is_some() {
            return Err(FieldError::new(
                "phillips",
                "leave this empty, or set Drive to Phillips",
            ));
        }
        return Ok(None);
    }
    if head == Head::Rod {
        return Err(FieldError::new(
            "drive",
            "a threaded rod has no drive; set Drive to None",
        ));
    }
    let standard = resolved
        .major
        .and_then(|major| standard_phillips(head, major));
    // The opening is the standard's `m` for the standard recess. A typed number keeps
    // the standard penetration, and a typed depth is taken as the penetration.
    let (number, depth, opening) = match requested {
        Some(phillips::Request::Sized { number, depth }) => {
            (number, depth, phillips::opening_at(number, depth))
        }
        Some(phillips::Request::Number(number)) => {
            let row = standard.ok_or_else(|| missing_phillips(head))?;
            let opening = if number == row.number {
                Some(row.opening)
            } else {
                phillips::opening_at(number, row.nominal_depth)
            };
            (number, row.depth, opening)
        }
        None => {
            let row = standard.ok_or_else(|| missing_phillips(head))?;
            (row.number, row.depth, Some(row.opening))
        }
    };
    let bad_size = || FieldError::new("phillips", "must be a Type H size, such as PH2 or PH2x2.1");
    let (width, bottom) = phillips::tool(number).ok_or_else(bad_size)?;
    let opening = opening.ok_or_else(bad_size)?;
    let h1 = phillips::point_height(bottom);
    if depth <= h1 || opening <= bottom {
        return Err(FieldError::new(
            "phillips",
            format!("PH{number} needs to be deeper than {h1:.2} mm"),
        ));
    }
    let tool = Tool {
        number,
        width,
        bottom,
        depth,
        opening,
    };
    // Too wide for the face is the plainer fault, so it is told first.
    if let Some(span) = span_at(resolved, 0.0) {
        if span < tool.opening_diameter() + 2.0 * PHILLIPS_WALL {
            return Err(FieldError::new(
                "phillips",
                format!(
                    "PH{number} opens to {:.2} mm and does not fit in this screw",
                    tool.opening_diameter()
                ),
            ));
        }
    }
    // The recess ends at the tip of its point, below the gauged penetration. In a low
    // head that tip runs on into the shank, which is sound while the recess is
    // narrower than the root of the thread where it crosses.
    let reach = tool.reach();
    let too_deep = || {
        FieldError::new(
            "phillips",
            format!(
                "a PH{number} recess reaches {reach:.2} mm into this screw and leaves less than {PHILLIPS_FLOOR} mm under it"
            ),
        )
    };
    let room = recess_room(resolved, length);
    if room < reach + PHILLIPS_FLOOR {
        let into_shank = match (resolved.head, resolved.major, resolved.pitch) {
            (ResolvedHead::None, _, _) | (_, None, _) | (_, _, None) => None,
            // ISO 68-1: the root of an external thread is 1.2269 pitches under the major.
            (_, Some(major), Some(pitch)) => Some((reach - room, major - 1.2269 * pitch)),
        };
        match into_shank {
            Some((below, root))
                if 2.0 * tool.radius_at(below.max(0.0)) + 2.0 * PHILLIPS_WALL <= root => {}
            _ => return Err(too_deep()),
        }
    }
    // Where the wings meet the point their diameter is `g`. On a countersunk head
    // that station is narrower than the face.
    if let Some(span) = span_at(resolved, tool.wing_depth()) {
        if span < bottom + 2.0 * PHILLIPS_WALL {
            return Err(FieldError::new(
                "phillips",
                format!("PH{number} does not fit in this screw"),
            ));
        }
    }
    Ok(Some(tool))
}

impl Params {
    /// The cylinder a thread can cover: the shank less a point, and less the chamfers
    /// on the ends of a set screw, each by how far it runs along the shank.
    pub fn thread_room(&self) -> f64 {
        let point = match self.resolved.tip {
            ResolvedTip::Point { length } => length,
            _ => 0.0,
        };
        self.length
            - point
            - self.point_chamfer().unwrap_or(0.0)
            - self.socket_end_chamfer_length().unwrap_or(0.0)
    }

    /// How far the chamfer on a set screw's point runs up the shank from the tip: it is
    /// 45°, so as far as it narrows the end. An ISO 4026 flat point is chamfered down
    /// to its flat, and an ISO 4029 cup point down to the mouth of its cup, which
    /// leaves a sharp rim. Nothing on any other tip, or on a cup the standard does not
    /// size.
    pub fn point_chamfer(&self) -> Option<f64> {
        let major = self.resolved.major?;
        let end = match self.resolved.tip {
            ResolvedTip::FlatPoint { diameter } => diameter,
            ResolvedTip::Cup { diameter, .. }
                if self.head == Head::Set && hexsocket::cup_point(major) == Some(diameter) =>
            {
                diameter
            }
            _ => return None,
        };
        (end < major).then_some((major - end) / 2.0)
    }

    /// The chamfer round the socket end of a hexagon socket set screw, as how far it
    /// narrows the end face on each side. ISO 4026 and ISO 4029 draw it down to about
    /// the root of the thread, which lies 0.6134 pitches under the major diameter.
    pub fn socket_end_chamfer(&self) -> Option<f64> {
        match (self.head, self.drive, self.resolved.pitch) {
            (Head::Set, Drive::HexSocket, Some(pitch)) => Some(0.6134 * pitch),
            _ => None,
        }
    }

    /// The included angle of that chamfer: 120° on the short lengths the standard
    /// shades in its table, and 90° on every longer screw.
    pub fn socket_end_angle_deg(&self) -> f64 {
        let cup = matches!(self.resolved.tip, ResolvedTip::Cup { .. });
        match self.resolved.major {
            Some(major) if hexsocket::is_short(major, self.length, cup) => 120.0,
            _ => 90.0,
        }
    }

    /// How far the socket end chamfer runs down the shank: as far as it narrows the
    /// end at 90°, and less at 120°.
    pub fn socket_end_chamfer_length(&self) -> Option<f64> {
        let half = (self.socket_end_angle_deg() / 2.0).to_radians();
        self.socket_end_chamfer().map(|radial| radial / half.tan())
    }
}

/// The standard that sizes this screw. A set screw is named by its drive.
fn standard_of(head: Head, drive: Drive, tip: Tip, custom_head: bool) -> Option<&'static str> {
    if custom_head {
        return None;
    }
    if (head, drive, tip) == (Head::Set, Drive::HexSocket, Tip::Cup) {
        return Some("ISO 4029");
    }
    match (head, drive) {
        (Head::Set, Drive::HexSocket) => Some("ISO 4026"),
        (Head::Set, Drive::Slotted) => Some("ISO 4766"),
        (Head::Countersunk, Drive::Phillips) => Some("ISO 7046-1"),
        (Head::Countersunk, Drive::Slotted) => Some("ISO 2009"),
        _ => kind_of(head, drive).map(HeadKind::standard),
    }
}

/// The hole for this screw. Every kind is as deep as the screw reaches.
fn plan_hole(
    v: &Value,
    designation: &str,
    resolved: &Resolved,
    length: f64,
) -> Result<Hole, FieldError> {
    let name = hole_type(v)?;
    let major = resolved
        .major
        .ok_or_else(|| FieldError::new("hole_type", "a hole needs a metric size, such as M3"))?;
    // ISO 273 lists clearance holes by nominal size, whatever the pitch. A tapped hole
    // needs none.
    let size = || {
        clearance::Hole::for_major(major).ok_or_else(|| {
            FieldError::new(
                designation::size_field(v),
                format!(
                    "ISO 273 lists no clearance hole for M{}; use a size from M1 to M64, or the Tapped hole",
                    metric::number(major)
                ),
            )
        })
    };
    match name {
        CLEARANCE => Ok(Hole {
            kind: HoleKind::Clearance { size: size()? },
            depth: length,
            sink: 0.0,
        }),
        TAPPED => Ok(Hole {
            kind: HoleKind::Tapped {
                thread: designation.to_string(),
            },
            depth: length,
            sink: 0.0,
        }),
        RECESSED => {
            let size = size()?;
            let (kind, sink) = match resolved.head {
                ResolvedHead::None => return Err(FieldError::new(
                    "hole_type",
                    "a set screw or a threaded rod has no head to recess; use Clearance or Tapped",
                )),
                ResolvedHead::Round { diameter, height } => (
                    HoleKind::Counterbore {
                        size,
                        diameter: diameter + 2.0 * HEAD_GAP,
                        depth: height,
                    },
                    height,
                ),
                // The counterbore clears the corners, not only the flats.
                ResolvedHead::Hex {
                    across_flats,
                    height,
                } => (
                    HoleKind::Counterbore {
                        size,
                        diameter: across_flats * 2.0 / 3.0_f64.sqrt() + 2.0 * HEAD_GAP,
                        depth: height,
                    },
                    height,
                ),
                // The 90° cone is widened by the land, so the cone of the head meets it
                // and the top of the head ends flush.
                ResolvedHead::Countersunk { diameter, height } => {
                    let (_, land) = head::countersink(major, diameter, height);
                    (
                        HoleKind::Countersink {
                            size,
                            diameter: diameter + 2.0 * land,
                        },
                        height,
                    )
                }
            };
            Ok(Hole {
                kind,
                depth: length + sink,
                sink,
            })
        }
        other => Err(FieldError::new(
            "hole_type",
            format!("must be {CLEARANCE}, {RECESSED} or {TAPPED}, not {other}"),
        )),
    }
}

/// The Hole type field, Clearance when it is empty.
fn hole_type(v: &Value) -> Result<&str, FieldError> {
    input::choice(v, "hole_type", CLEARANCE)
}

/// The standard thread length `b`, when the standard gives a screw this long a plain
/// shank. `None` threads it to the head.
///
/// `length` is the shank under the head, `nominal` the length the standard's table is
/// read at, which includes a countersunk head.
fn standard_thread_length(
    head: Head,
    drive: Drive,
    resolved: &Resolved,
    length: f64,
    nominal: f64,
) -> Option<f64> {
    let kind = kind_of(head, drive)?;
    let (major, pitch) = (resolved.major?, resolved.pitch?);
    let b = head::thread_length(kind, major)?;
    // A point is not threaded, so `b` has to fit on the cylinder above it.
    let cylinder = match resolved.tip {
        ResolvedTip::Point { length: point } => length - point,
        _ => length,
    };
    (head::has_plain_shank(kind, major, pitch, nominal) && b < cylinder).then_some(b)
}

/// The drive a head usually has, when its standard lists one for this size: a hex
/// socket in a socket, button or countersunk head and in a set screw, a Phillips recess
/// in a pan head, and in a countersunk head below M3, where ISO 10642 has no socket but
/// ISO 7046-1 has a recess. A hex head and a threaded rod have none.
fn standard_drive(head: Head, major: Option<f64>) -> Drive {
    let Some(major) = major else {
        return Drive::None;
    };
    match head {
        Head::Countersunk => {
            if standard_hex(head, major).is_some() {
                Drive::HexSocket
            } else if standard_phillips(head, major).is_some() {
                Drive::Phillips
            } else {
                Drive::None
            }
        }
        Head::Socket | Head::Button | Head::Set => {
            if standard_hex(head, major).is_some() {
                Drive::HexSocket
            } else {
                Drive::None
            }
        }
        Head::Pan => {
            if phillips::for_headed(major).is_some() {
                Drive::Phillips
            } else {
                Drive::None
            }
        }
        Head::Cheese => {
            if slot::for_cheese(major).is_some() {
                Drive::Slotted
            } else {
                Drive::None
            }
        }
        Head::Hex | Head::Rod => Drive::None,
    }
}

/// Material left beside the socket corners, and under the socket.
const HEX_WALL: f64 = 0.2;
const HEX_FLOOR: f64 = 0.3;

/// The socket from the product standard for this head. A pan head and a hex head have none.
fn standard_hex(head: Head, major: f64) -> Option<Socket> {
    standard_hex_for(head, major, None, false)
}

/// As `standard_hex`, with the length a set screw's socket depth depends on.
fn standard_hex_for(head: Head, major: f64, length: Option<f64>, cup: bool) -> Option<Socket> {
    if let (Head::Set, Some(length)) = (head, length) {
        // ISO 4029 for a cup point, where it lists the size, and ISO 4026 otherwise.
        let cup_socket = if cup {
            hexsocket::cup_set_screw(major, length)
        } else {
            None
        };
        return cup_socket.or_else(|| hexsocket::set_screw(major, length));
    }
    let kind = match head {
        Head::Socket => hexsocket::Kind::Cap,
        Head::Button => hexsocket::Kind::Button,
        Head::Countersunk => hexsocket::Kind::Countersunk,
        Head::Set => hexsocket::Kind::Set,
        Head::Hex | Head::Pan | Head::Cheese | Head::Rod => return None,
    };
    hexsocket::standard(kind, major)
}

fn missing_hex(head: Head) -> FieldError {
    let why = match head {
        Head::Pan => "a pan head has no standard hex socket",
        Head::Cheese => "a cheese head has no standard hex socket",
        Head::Hex => "a hex head has no standard hex socket",
        _ => "this size has no standard hex socket",
    };
    FieldError::new(
        "hex",
        format!("{why}; set a key size and a depth, such as 2.5x1.3, or choose another Drive"),
    )
}

/// The hexagon socket for a Hex socket drive. A key size on another drive is refused.
fn plan_hex(
    drive: Drive,
    head: Head,
    requested: Option<hexsocket::Request>,
    resolved: &Resolved,
    length: f64,
) -> Result<Option<Socket>, FieldError> {
    if drive != Drive::HexSocket {
        if requested.is_some() {
            return Err(FieldError::new(
                "hex",
                "leave this empty, or set Drive to Hex socket",
            ));
        }
        return Ok(None);
    }
    if head == Head::Rod {
        return Err(FieldError::new(
            "drive",
            "a threaded rod has no drive; set Drive to None",
        ));
    }
    let standard = resolved.major.and_then(|major| {
        let cup = matches!(resolved.tip, ResolvedTip::Cup { .. });
        standard_hex_for(head, major, Some(length), cup)
    });
    let socket = match requested {
        Some(hexsocket::Request::Sized(socket)) => socket,
        Some(hexsocket::Request::Key(across_flats)) => Socket {
            across_flats,
            depth: standard.ok_or_else(|| missing_hex(head))?.depth,
        },
        None => standard.ok_or_else(|| missing_hex(head))?,
    };
    let room = recess_room(resolved, length);
    if room < socket.depth + HEX_FLOOR {
        return Err(FieldError::new(
            "hex",
            format!(
                "a {} mm hex socket is {} mm deep and this screw leaves less than {HEX_FLOOR} mm under it",
                socket.across_flats, socket.depth
            ),
        ));
    }
    if let Some(span) = span_at(resolved, socket.depth) {
        if span < socket.across_corners() + 2.0 * HEX_WALL {
            return Err(FieldError::new(
                "hex",
                format!(
                    "a {} mm hex socket is {:.2} mm across the corners and does not fit in this screw",
                    socket.across_flats,
                    socket.across_corners()
                ),
            ));
        }
    }
    Ok(Some(socket))
}

/// How much of the screw the recess can use: the head, or the cylinder of a set screw.
fn recess_room(resolved: &Resolved, length: f64) -> f64 {
    match resolved.head {
        ResolvedHead::None => {
            let point = match resolved.tip {
                ResolvedTip::Point { length } => length,
                _ => 0.0,
            };
            let cup = match resolved.tip {
                ResolvedTip::Cup { depth, .. } => depth,
                _ => 0.0,
            };
            length - point - cup
        }
        ResolvedHead::Round { height, .. }
        | ResolvedHead::Hex { height, .. }
        | ResolvedHead::Countersunk { height, .. } => height,
    }
}

/// The width the lobe tips have to fit in, at the bottom of the recess.
fn span_at(resolved: &Resolved, depth: f64) -> Option<f64> {
    match resolved.head {
        ResolvedHead::None => resolved.major,
        ResolvedHead::Round { diameter, .. } => Some(diameter),
        ResolvedHead::Hex { across_flats, .. } => Some(across_flats),
        ResolvedHead::Countersunk { diameter, height } => {
            let major = resolved.major?;
            // Full width through the land, then down the cone to the shank.
            let (cone, land) = head::countersink(major, diameter, height);
            if depth <= land {
                Some(diameter)
            } else {
                let z = (height - depth).max(0.0);
                Some(major + (diameter - major) * (z / cone))
            }
        }
    }
}

fn head(v: &Value) -> Result<Head, FieldError> {
    let name = input::choice(v, "head", "Socket head")?;
    Head::ALL
        .into_iter()
        .find(|head| head.label() == name)
        .ok_or_else(|| FieldError::new("head", format!("must be a head from the list, not {name}")))
}

/// The drive that was asked for. `None` is Standard: the usual drive for the head.
fn drive(v: &Value) -> Result<Option<Drive>, FieldError> {
    let name = input::choice(v, "drive", "Standard")?;
    match name {
        "Standard" => Ok(None),
        "Hex socket" => Ok(Some(Drive::HexSocket)),
        "Phillips" => Ok(Some(Drive::Phillips)),
        "Slotted" => Ok(Some(Drive::Slotted)),
        "Torx" => Ok(Some(Drive::Torx)),
        "None" => Ok(Some(Drive::None)),
        other => Err(FieldError::new(
            "drive",
            format!("must be a drive from the list, not {other}"),
        )),
    }
}

fn tip(v: &Value) -> Result<Tip, FieldError> {
    let name = input::choice(v, "tip", "Flat")?;
    match name {
        "Flat" => Ok(Tip::Flat),
        "Point" => Ok(Tip::Point),
        "Cup" => Ok(Tip::Cup),
        other => Err(FieldError::new(
            "tip",
            format!("must be a tip from the list, not {other}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_are_an_m3_socket_screw() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.designation, "M3");
        assert_eq!(p.length, 12.0);
        assert_eq!(p.head, Head::Socket);
        assert_eq!(p.drive, Drive::HexSocket);
        assert!(p.recess.is_none());
        assert!(p.phillips.is_none());
        assert_eq!(p.tip, Tip::Flat);
        assert_eq!(p.resolved.tip, ResolvedTip::Flat);
        assert_eq!(p.head_diameter, None);
        assert_eq!(p.head_height, None);
        assert_eq!(p.thread_length, None);
        assert!(p.modeled);
        assert!(!parse(&json!({"thread_modeled": false})).unwrap().modeled);
        assert_eq!(p.resolved.major, Some(3.0));
        assert_eq!(p.resolved.pitch, Some(0.5));
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Round {
                diameter: 5.5,
                height: 3.0
            }
        );
    }

    #[test]
    fn an_m8_hex_screw_uses_the_standard_across_flats() {
        let p = parse(&json!({
            "size": "M8x1",
            "length": 30,
            "head": "Hex head",
            "drive": "None",
            "thread_length": 16,
            "thread_modeled": true,
        }))
        .unwrap();
        assert_eq!(p.designation, "M8x1");
        assert_eq!(p.length, 30.0);
        assert_eq!(p.head, Head::Hex);
        assert_eq!(p.drive, Drive::None);
        assert_eq!(p.thread_length, Some(16.0));
        assert!(p.modeled);
        assert_eq!(p.resolved.major, Some(8.0));
        assert_eq!(p.resolved.pitch, Some(1.0));
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Hex {
                across_flats: 13.0,
                height: 5.3
            }
        );
    }

    #[test]
    fn a_fine_pitch_uses_the_nominal_head() {
        let p = parse(&json!({"size": "M3x0.35"})).unwrap();
        assert_eq!(p.resolved.pitch, Some(0.35));
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Round {
                diameter: 5.5,
                height: 3.0
            }
        );
    }

    #[test]
    fn standard_heads_for_an_m3() {
        let button = parse(&json!({"head": "Button head"})).unwrap();
        assert_eq!(
            button.resolved.head,
            ResolvedHead::Round {
                diameter: 5.7,
                height: 1.65
            }
        );
        let countersunk = parse(&json!({"head": "Countersunk"})).unwrap();
        assert_eq!(
            countersunk.resolved.head,
            ResolvedHead::Countersunk {
                diameter: 6.72,
                height: 1.86
            }
        );
        // The slotted ISO 1580 pan. With a Phillips recess it is the ISO 7045 head.
        let pan = parse(&json!({"head": "Pan head", "drive": "Slotted"})).unwrap();
        assert_eq!(
            pan.resolved.head,
            ResolvedHead::Round {
                diameter: 5.6,
                height: 1.8
            }
        );
    }

    #[test]
    fn an_untabulated_size_takes_both_head_overrides() {
        // A bare M3.5 takes its ISO 261 coarse pitch, and still has no standard head.
        assert_eq!(
            parse(&json!({"size": "M3.5"})).unwrap_err().field,
            "head_diameter"
        );
        assert_eq!(
            parse(&json!({"size": "M3.5x0.6"})).unwrap_err().field,
            "head_diameter"
        );
        let p = parse(&json!({
            "size": "M3.5x0.6",
            "head_diameter": 7,
            "head_height": 2.5,
        }))
        .unwrap();
        assert_eq!(p.resolved.major, Some(3.5));
        assert_eq!(p.resolved.pitch, Some(0.6));
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Round {
                diameter: 7.0,
                height: 2.5
            }
        );
    }

    #[test]
    fn overrides_replace_the_standard_head() {
        let p = parse(&json!({"head_diameter": 9, "head_height": 4})).unwrap();
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Round {
                diameter: 9.0,
                height: 4.0
            }
        );
    }

    #[test]
    fn one_head_dimension_keeps_the_other_from_the_standard() {
        let wider = parse(&json!({"head_diameter": 8})).unwrap();
        assert_eq!(
            wider.resolved.head,
            ResolvedHead::Round {
                diameter: 8.0,
                height: 3.0
            }
        );
        let taller = parse(&json!({"head_height": 4})).unwrap();
        assert_eq!(
            taller.resolved.head,
            ResolvedHead::Round {
                diameter: 5.5,
                height: 4.0
            }
        );
        let hex = parse(&json!({"head": "Hex head", "head_diameter": 10})).unwrap();
        assert_eq!(
            hex.resolved.head,
            ResolvedHead::Hex {
                across_flats: 10.0,
                height: 2.0
            }
        );
    }

    #[test]
    fn a_countersunk_head_takes_both_dimensions_or_neither() {
        let err = parse(&json!({"head": "Countersunk", "head_diameter": 8})).unwrap_err();
        assert_eq!(err.field, "head_height");
        assert_eq!(
            err.reason,
            "set the head diameter and the head height, or leave both empty"
        );
        let err = parse(&json!({"head": "Countersunk", "head_height": 2})).unwrap_err();
        assert_eq!(err.field, "head_diameter");
    }

    #[test]
    fn one_dimension_on_an_unknown_head_asks_for_both() {
        let err = parse(&json!({
            "size": "M12",
            "head": "Pan head",
            "head_height": 4,
        }))
        .unwrap_err();
        assert_eq!(err.field, "head_diameter");
        assert_eq!(
            err.reason,
            "this size has no standard head here; set the head diameter and height"
        );
    }

    #[test]
    fn the_head_has_to_clear_the_thread() {
        let err = parse(&json!({"head_diameter": 3, "head_height": 2})).unwrap_err();
        assert_eq!(err.field, "head_diameter");
        assert_eq!(err.reason, "must be larger than the thread");
        let err = parse(&json!({"head_diameter": 3})).unwrap_err();
        assert_eq!(err.reason, "must be larger than the thread");
        let err = parse(&json!({
            "head": "Hex head",
            "head_diameter": 3,
            "head_height": 2,
        }))
        .unwrap_err();
        assert_eq!(err.reason, "must be wider than the thread");
    }

    #[test]
    fn a_non_metric_size_needs_a_head_size() {
        assert_eq!(
            parse(&json!({"size": "1/2-10 ACME"})).unwrap_err().field,
            "head_diameter"
        );
        let err = parse(&json!({"size": "1/2-10 ACME", "head_diameter": 20})).unwrap_err();
        assert_eq!(err.field, "head_height");
        let p = parse(&json!({
            "size": "1/2-10 ACME",
            "head_diameter": 11,
            "head_height": 4,
        }))
        .unwrap();
        assert_eq!(p.resolved.major, None);
        assert_eq!(p.resolved.pitch, None);
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Round {
                diameter: 11.0,
                height: 4.0
            }
        );
    }

    #[test]
    fn a_set_screw_rejects_a_head_size() {
        let err = parse(&json!({"head": "Set screw", "head_diameter": 5})).unwrap_err();
        assert_eq!(err.field, "head_diameter");
        assert!(err.reason.contains("set screw"));
        let p = parse(&json!({"head": "Set screw"})).unwrap();
        assert_eq!(p.head, Head::Set);
        assert_eq!(p.resolved.head, ResolvedHead::None);
        assert_eq!(p.resolved.major, Some(3.0));
    }

    #[test]
    fn a_threaded_rod_rejects_a_head_height() {
        let err = parse(&json!({"head": "Threaded rod", "head_height": 4})).unwrap_err();
        assert_eq!(err.field, "head_height");
        assert!(err.reason.contains("threaded rod"));
    }

    #[test]
    fn rejects_a_thread_longer_than_the_shank() {
        let err = parse(&json!({"length": 10, "thread_length": 12})).unwrap_err();
        assert_eq!(err.field, "thread_length");
    }

    #[test]
    fn rejects_an_unknown_head() {
        let err = parse(&json!({"head": "wing"})).unwrap_err();
        assert_eq!(err.field, "head");
    }

    #[test]
    fn a_point_is_one_diameter_of_the_length() {
        let p = parse(&json!({"tip": "Point"})).unwrap();
        assert_eq!(p.tip, Tip::Point);
        assert_eq!(p.resolved.tip, ResolvedTip::Point { length: 3.0 });
    }

    #[test]
    fn an_m3_cup_is_1_4_across() {
        let p = parse(&json!({"head": "Set screw", "tip": "Cup", "length": 10})).unwrap();
        let ResolvedTip::Cup { diameter, depth } = p.resolved.tip else {
            panic!("cup");
        };
        assert_eq!(diameter, 1.4);
        assert!((depth - 1.4 / (2.0 * 3.0_f64.sqrt())).abs() < 1e-12);
    }

    #[test]
    fn a_fine_pitch_uses_the_coarse_cup() {
        let p = parse(&json!({"size": "M8x1", "tip": "Cup"})).unwrap();
        let ResolvedTip::Cup { diameter, .. } = p.resolved.tip else {
            panic!("cup");
        };
        assert_eq!(diameter, 5.0);
    }

    #[test]
    fn an_untabulated_cup_is_half_the_major() {
        // ISO 4029 lists M1.6 to M24, without M14.
        for (size, length, mouth) in [("M30", 60, 15.0), ("M14", 40, 7.0)] {
            let p = parse(&json!({"size": size, "tip": "Cup", "length": length})).unwrap();
            let ResolvedTip::Cup { diameter, .. } = p.resolved.tip else {
                panic!("cup");
            };
            assert_eq!(diameter, mouth, "{size}");
        }
    }

    #[test]
    fn a_short_point_is_refused() {
        let err = parse(&json!({"tip": "Point", "length": 3})).unwrap_err();
        assert_eq!(err.field, "length");
        assert!(err.reason.contains("3 mm"));
        assert!(err.reason.contains("3.5 mm"));
        let p = parse(&json!({"tip": "Point", "length": 4})).unwrap();
        assert_eq!(p.resolved.tip, ResolvedTip::Point { length: 3.0 });
    }

    #[test]
    fn a_thread_up_to_the_full_length_covers_the_cylinder_above_a_point() {
        // The manifest says to type the full length to thread it all.
        let full = parse(&json!({"tip": "Point", "thread_length": 12})).unwrap();
        assert_eq!(full.thread_length, Some(9.0));
        let past = parse(&json!({"tip": "Point", "thread_length": 10})).unwrap();
        assert_eq!(past.thread_length, Some(9.0));
        let err = parse(&json!({"tip": "Point", "thread_length": 12.5})).unwrap_err();
        assert_eq!(err.field, "thread_length");
        let p = parse(&json!({"tip": "Point", "thread_length": 9})).unwrap();
        assert_eq!(p.thread_length, Some(9.0));
    }

    #[test]
    fn rejects_an_unknown_tip() {
        let err = parse(&json!({"tip": "dog"})).unwrap_err();
        assert_eq!(err.field, "tip");
    }

    #[test]
    fn rejects_a_blank_designation() {
        let err = parse(&json!({"size": "  "})).unwrap_err();
        assert_eq!(err.field, "size");
    }

    #[test]
    fn an_m3_torx_is_t10_at_the_iso_14579_depth() {
        let p = parse(&json!({"drive": "Torx"})).unwrap();
        assert_eq!(p.drive, Drive::Torx);
        assert_eq!(
            p.recess,
            Some(torx::Recess {
                number: 10,
                depth: 1.01
            })
        );
        let named = parse(&json!({"drive": "Torx", "drive_size": "T20"})).unwrap();
        assert_eq!(named.recess.unwrap().number, 20);
        assert_eq!(named.recess.unwrap().depth, 1.42);
    }

    #[test]
    fn every_tabulated_head_takes_its_standard_torx() {
        for head in [
            "Socket head",
            "Button head",
            "Countersunk",
            "Hex head",
            "Pan head",
        ] {
            for major in [
                2.0_f64, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0,
            ] {
                let designation = if major.fract() == 0.0 {
                    format!("M{}", major as u32)
                } else {
                    format!("M{major}")
                };
                if parse(&json!({"size": &designation, "head": head})).is_err() {
                    continue;
                }
                let p = match parse(&json!({"size": &designation, "head": head, "drive": "Torx"})) {
                    Ok(p) => p,
                    // Below M3 a countersunk head is the ISO 7046-1 Phillips one; a Torx
                    // countersunk head is sized from ISO 10642, which starts at M3.
                    Err(err) if head == "Countersunk" && major < 3.0 => {
                        assert_eq!(err.field, "head_diameter", "{designation}");
                        continue;
                    }
                    Err(err) => panic!("{designation} {head}: {} {}", err.field, err.reason),
                };
                assert!(p.recess.is_some(), "{designation} {head}");
            }
        }
    }

    #[test]
    fn a_size_without_a_standard_socket_asks_for_one() {
        let err = parse(&json!({"size": "M24", "drive": "Torx"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("T10"));
        let p = parse(&json!({"size": "M24", "drive": "Torx", "drive_size": "T60"})).unwrap();
        assert_eq!(p.recess.unwrap().number, 60);
    }

    #[test]
    fn a_torx_number_on_another_drive_is_refused() {
        let err = parse(&json!({"drive_size": "T10"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("key"));
    }

    #[test]
    fn a_threaded_rod_refuses_a_torx_drive() {
        let err = parse(&json!({"head": "Threaded rod", "drive": "Torx"})).unwrap_err();
        assert_eq!(err.field, "drive");
    }

    #[test]
    fn an_m3_set_screw_needs_a_smaller_torx_than_t10() {
        let err = parse(&json!({"head": "Set screw", "drive": "Torx"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("smaller"));
        let p = parse(&json!({"head": "Set screw", "drive": "Torx", "drive_size": "T6"})).unwrap();
        assert_eq!(
            p.recess,
            Some(torx::Recess {
                number: 6,
                depth: 0.71
            })
        );
    }

    #[test]
    fn a_short_head_refuses_the_recess() {
        let err = parse(&json!({"drive": "Torx", "head_height": 0.5})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("1.01"));
    }

    #[test]
    fn rejects_an_unknown_torx_socket() {
        let err = parse(&json!({"drive": "Torx", "drive_size": "T7"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
    }

    #[test]
    fn an_m3_pan_slot_is_the_iso_1580_slot() {
        let p = parse(&json!({"head": "Pan head", "drive": "Slotted"})).unwrap();
        assert_eq!(
            p.slot,
            Some(Slot {
                width: 0.8,
                depth: 0.7
            })
        );
        let socket = parse(&json!({"drive": "Slotted"})).unwrap();
        assert_eq!(socket.slot, p.slot);
    }

    #[test]
    fn an_m3_set_screw_slot_is_the_iso_4766_slot() {
        let p = parse(&json!({"head": "Set screw", "drive": "Slotted"})).unwrap();
        assert_eq!(
            p.slot,
            Some(Slot {
                width: 0.4,
                depth: 0.8
            })
        );
    }

    #[test]
    fn a_fine_pitch_uses_the_coarse_slot() {
        let p = parse(&json!({"size": "M8x1", "head": "Pan head", "drive": "Slotted"})).unwrap();
        assert_eq!(
            p.slot,
            Some(Slot {
                width: 2.0,
                depth: 1.9
            })
        );
    }

    #[test]
    fn a_non_preferred_size_uses_its_slot() {
        let p = parse(&json!({
            "size": "M3.5x0.6",
            "head": "Pan head",
            "drive": "Slotted",
            "head_diameter": 7.0,
            "head_height": 2.1,
        }))
        .unwrap();
        assert_eq!(
            p.slot,
            Some(Slot {
                width: 1.0,
                depth: 0.8
            })
        );
    }

    #[test]
    fn a_size_without_a_standard_slot_asks_for_one() {
        let err = parse(&json!({"size": "M12", "drive": "Slotted"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("0.8x0.7"));
        let p =
            parse(&json!({"size": "M12", "drive": "Slotted", "drive_size": "1.6x1.4"})).unwrap();
        assert_eq!(
            p.slot,
            Some(Slot {
                width: 1.6,
                depth: 1.4
            })
        );
    }

    #[test]
    fn a_retired_slot_field_is_rejected() {
        let err = parse(&json!({"slot": "0.8x0.7"})).unwrap_err();
        assert_eq!(err.field, "slot");
        assert!(err.reason.contains("unsupported"));
    }

    #[test]
    fn a_threaded_rod_refuses_a_slotted_drive() {
        let err = parse(&json!({"head": "Threaded rod", "drive": "Slotted"})).unwrap_err();
        assert_eq!(err.field, "drive");
    }

    #[test]
    fn a_short_head_refuses_the_slot() {
        let err = parse(&json!({"head": "Pan head", "drive": "Slotted", "head_height": 0.5}))
            .unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("0.7"));
    }

    #[test]
    fn a_wide_slot_is_refused() {
        let err = parse(&json!({"head": "Pan head", "drive": "Slotted", "drive_size": "5.5x0.5"}))
            .unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("does not fit"));
    }

    #[test]
    fn an_m3_phillips_is_recess_1_at_the_iso_7045_depth() {
        let pan = parse(&json!({"head": "Pan head", "drive": "Phillips"})).unwrap();
        assert_eq!(
            pan.phillips,
            Some(Tool {
                number: 1,
                width: 0.97,
                bottom: 1.27,
                depth: 1.4,
                opening: 3.0,
            })
        );
        let socket = parse(&json!({"drive": "Phillips"})).unwrap();
        assert_eq!(socket.phillips, pan.phillips);
        let countersunk = parse(&json!({"head": "Countersunk", "drive": "Phillips"})).unwrap();
        // ISO 7046-1: the same recess number, gauged deeper and cut wider.
        assert_eq!(countersunk.phillips.unwrap().number, 1);
        assert_eq!(countersunk.phillips.unwrap().depth, 1.7);
        assert_eq!(countersunk.phillips.unwrap().opening, 3.2);
    }

    #[test]
    fn an_m2_pan_and_countersunk_take_their_own_phillips_rows() {
        let pan = parse(&json!({"size": "M2", "head": "Pan head", "drive": "Phillips"})).unwrap();
        // ISO 7045 and ISO 7046-1 both give an M2 recess 0, gauged at 0.9 mm.
        assert_eq!(pan.phillips.unwrap().number, 0);
        assert_eq!(pan.phillips.unwrap().depth, 0.9);
        let countersunk = parse(&json!({
            "size": "M2",
            "head": "Countersunk",
            "drive": "Phillips",
            "head_diameter": 3.8,
            "head_height": 1.2,
        }))
        .unwrap();
        assert_eq!(countersunk.phillips.unwrap().number, 0);
        assert_eq!(countersunk.phillips.unwrap().depth, 0.9);
    }

    #[test]
    fn an_m4_countersunk_phillips_is_deeper_than_its_pan() {
        let pan = parse(&json!({"size": "M4", "head": "Pan head", "drive": "Phillips"})).unwrap();
        assert_eq!(pan.phillips.unwrap().number, 2);
        assert_eq!(pan.phillips.unwrap().depth, 1.9);
        let countersunk =
            parse(&json!({"size": "M4", "head": "Countersunk", "drive": "Phillips"})).unwrap();
        assert_eq!(countersunk.phillips.unwrap().number, 2);
        assert_eq!(countersunk.phillips.unwrap().depth, 2.1);
    }

    #[test]
    fn a_phillips_number_keeps_the_standard_depth() {
        let p =
            parse(&json!({"head": "Pan head", "drive": "Phillips", "drive_size": "PH2"})).unwrap();
        assert_eq!(p.phillips.unwrap().number, 2);
        assert_eq!(p.phillips.unwrap().depth, 1.4);
    }

    #[test]
    fn a_fine_pitch_uses_the_coarse_phillips() {
        let p = parse(&json!({"size": "M8x1", "head": "Pan head", "drive": "Phillips"})).unwrap();
        assert_eq!(p.phillips.unwrap().number, 4);
        assert_eq!(p.phillips.unwrap().depth, 4.0);
    }

    #[test]
    fn a_non_preferred_size_uses_its_phillips_row() {
        let p = parse(&json!({
            "size": "M3.5x0.6",
            "head": "Pan head",
            "drive": "Phillips",
            "head_diameter": 7.0,
            "head_height": 2.1,
        }))
        .unwrap();
        assert_eq!(p.phillips.unwrap().number, 2);
        assert_eq!(p.phillips.unwrap().depth, 1.4);
    }

    #[test]
    fn a_size_without_a_standard_phillips_asks_for_one() {
        let err = parse(&json!({"size": "M12", "drive": "Phillips"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("PH1x1.7"));
        let p =
            parse(&json!({"size": "M12", "drive": "Phillips", "drive_size": "PH3x3.0"})).unwrap();
        assert_eq!(p.phillips.unwrap().number, 3);
        assert_eq!(p.phillips.unwrap().depth, 3.0);
    }

    #[test]
    fn a_set_screw_asks_for_a_phillips_size() {
        let err = parse(&json!({"head": "Set screw", "drive": "Phillips"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("PH1x0.8"));
        let p = parse(&json!({"head": "Set screw", "drive": "Phillips", "drive_size": "PH1x0.8"}))
            .unwrap();
        assert_eq!(p.phillips.unwrap().number, 1);
        assert_eq!(p.phillips.unwrap().depth, 0.8);
    }

    #[test]
    fn a_phillips_size_on_another_drive_is_refused() {
        let err = parse(&json!({"drive_size": "PH2"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("key"));
    }

    #[test]
    fn a_threaded_rod_refuses_a_phillips_drive() {
        let err = parse(&json!({"head": "Threaded rod", "drive": "Phillips"})).unwrap_err();
        assert_eq!(err.field, "drive");
    }

    #[test]
    fn a_short_head_refuses_the_phillips() {
        let err = parse(&json!({"head": "Pan head", "drive": "Phillips", "head_height": 1.0}))
            .unwrap_err();
        assert_eq!(err.field, "drive_size");
        // The recess reaches 2.07 mm, and 1.07 mm under a 1 mm head it is still wider
        // than the root of the thread.
        assert!(err.reason.contains("2.07"), "{}", err.reason);
        assert!(err.reason.contains("0.1"));
        // A 1.65 mm button head is low too, but there the point alone crosses into
        // the shank, well inside the thread root.
        let button = parse(&json!({"head": "Button head", "drive": "Phillips"})).unwrap();
        assert!(button.phillips.unwrap().reach() > 1.65);
        // A set screw has only its own length.
        let set = parse(
            &json!({"head": "Set screw", "drive": "Phillips", "length": 2,
            "drive_size": "PH1x1.7"}),
        )
        .unwrap_err();
        assert_eq!(set.field, "drive_size");
    }

    #[test]
    fn a_shallow_phillips_is_refused() {
        let err = parse(&json!({"head": "Pan head", "drive": "Phillips", "drive_size": "PH1x0.2"}))
            .unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("deeper"));
    }

    #[test]
    fn a_wide_phillips_is_refused_on_a_pan() {
        let err = parse(&json!({"head": "Pan head", "drive": "Phillips", "drive_size": "PH4x2.2"}))
            .unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("does not fit"));
    }

    #[test]
    fn a_phillips_that_clears_the_countersunk_face_can_still_miss_the_cone() {
        // A custom 90° cone on an M2, 2 mm to 10 mm over 4 mm. At a 2.9 mm penetration
        // the PH4 opening clears the 10 mm face, but the cone is narrower than `g`
        // where the point begins.
        let err = parse(&json!({
            "size": "M2",
            "head": "Countersunk",
            "head_diameter": 10,
            "head_height": 4,
            "drive": "Phillips",
            "drive_size": "PH4x2.9"
        }))
        .unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("does not fit"));
        let opening = phillips::opening_at(4, 2.9).unwrap();
        assert!(opening + 0.4 < 10.0, "{opening}");
        // The standard ISO 10642 cone is 90°, and the same recess sized for it fits.
        assert!(parse(&json!({"size": "M8", "head": "Countersunk", "drive": "Phillips"})).is_ok());
    }

    #[test]
    fn a_cheese_head_is_din_84_with_its_own_slot() {
        let p = parse(&json!({"head": "Cheese head"})).unwrap();
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Round {
                diameter: 5.5,
                height: 2.0
            }
        );
        // Standard on a cheese head is the DIN 84 slot: 0.8 wide, 0.85 deep on an M3.
        assert_eq!(p.drive, Drive::Slotted);
        assert_eq!(
            p.slot.unwrap(),
            Slot {
                width: 0.8,
                depth: 0.85
            }
        );
        // The same slot on a pan head is the shallower ISO 1580 one.
        let pan = parse(&json!({"head": "Pan head", "drive": "Slotted"})).unwrap();
        assert_eq!(pan.slot.unwrap().depth, 0.7);
        // Every DIN 84 size builds with its slot.
        for size in [
            "M1.6", "M2", "M2.5", "M3", "M3.5x0.6", "M4", "M5", "M6", "M8", "M10",
        ] {
            let p = parse(&json!({"size": size, "head": "Cheese head", "length": 20}));
            assert!(p.as_ref().is_ok_and(|p| p.slot.is_some()), "{size}: {p:?}");
        }
        // DIN 84 stops at M10, and has no cross recess.
        let big = parse(&json!({"size": "M12", "head": "Cheese head"})).unwrap_err();
        assert_eq!(big.field, "head_diameter");
        let cross = parse(&json!({"head": "Cheese head", "drive": "Phillips"})).unwrap_err();
        assert_eq!(cross.field, "drive_size");
        assert!(parse(&json!({"head": "Cheese head", "drive": "Torx"})).is_ok());
    }

    #[test]
    fn the_standard_drive_follows_the_head() {
        let drive = |head: &str| parse(&json!({"head": head})).unwrap();
        assert_eq!(drive("Socket head").drive, Drive::HexSocket);
        assert_eq!(drive("Button head").drive, Drive::HexSocket);
        assert_eq!(drive("Countersunk").drive, Drive::HexSocket);
        assert_eq!(drive("Set screw").drive, Drive::HexSocket);
        assert_eq!(drive("Pan head").drive, Drive::Phillips);
        assert_eq!(drive("Pan head").phillips.unwrap().number, 1);
        assert_eq!(drive("Hex head").drive, Drive::None);
        assert_eq!(drive("Threaded rod").drive, Drive::None);
        // A size the head's standard does not list gets no drive, and no error.
        let big = parse(&json!({"size": "M12", "head": "Pan head",
            "head_diameter": 24, "head_height": 7}))
        .unwrap();
        assert_eq!(big.drive, Drive::None);
    }

    #[test]
    fn each_head_takes_the_socket_of_its_own_standard() {
        let hex = |head: &str| parse(&json!({"head": head})).unwrap().hex.unwrap();
        assert_eq!(
            (hex("Socket head").across_flats, hex("Socket head").depth),
            (2.5, 1.3)
        );
        assert_eq!(
            (hex("Button head").across_flats, hex("Button head").depth),
            (2.0, 1.04)
        );
        assert_eq!(
            (hex("Countersunk").across_flats, hex("Countersunk").depth),
            (2.0, 1.1)
        );
        assert_eq!(
            (hex("Set screw").across_flats, hex("Set screw").depth),
            (1.5, 2.0)
        );
        // ISO 4026 shades M3 x 3: that one keeps the shallower socket.
        let short = parse(&json!({"head": "Set screw", "length": 3})).unwrap();
        assert_eq!(short.hex.unwrap().depth, 1.2);
        let longer = parse(&json!({"head": "Set screw", "length": 4})).unwrap();
        assert_eq!(longer.hex.unwrap().depth, 2.0);
        assert!(parse(&json!({"drive": "Torx"})).unwrap().hex.is_none());
    }

    #[test]
    fn every_standard_head_holds_its_standard_hex_socket() {
        let sizes = [
            1.6, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 20.0, 24.0, 30.0, 36.0,
            42.0, 48.0, 56.0, 64.0,
        ];
        let mut cut = 0;
        for head in ["Socket head", "Button head", "Countersunk", "Set screw"] {
            for size in sizes {
                let v = json!({"size": format!("M{size}"), "head": head,
                    "drive": "Hex socket", "length": 100});
                match parse(&v) {
                    Ok(p) => {
                        assert!(p.hex.is_some(), "{head} M{size}");
                        cut += 1;
                    }
                    // No standard head or no standard socket for this size.
                    Err(err) => assert!(
                        err.reason.contains("no standard"),
                        "{head} M{size}: {}: {}",
                        err.field,
                        err.reason
                    ),
                }
            }
        }
        assert_eq!(cut, 20 + 8 + 9 + 13);
    }

    #[test]
    fn a_hex_socket_that_does_not_fit_is_refused() {
        // Explicit Hex socket on a head with no standard socket asks for a size.
        let err = parse(&json!({"head": "Pan head", "drive": "Hex socket"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        assert!(err.reason.contains("pan head"), "{}", err.reason);
        let p = parse(&json!({"head": "Pan head", "drive": "Hex socket", "drive_size": "2x1"}))
            .unwrap();
        assert_eq!(p.hex.unwrap().across_flats, 2.0);
        let wide = parse(&json!({"drive_size": "5"})).unwrap_err();
        assert_eq!(wide.field, "drive_size");
        assert!(wide.reason.contains("does not fit"), "{}", wide.reason);
        let deep = parse(&json!({"drive_size": "2.5x2.9"})).unwrap_err();
        assert_eq!(deep.field, "drive_size");
        let short = parse(&json!({"head": "Set screw", "length": 1.2})).unwrap_err();
        assert_eq!(short.field, "drive_size");
        let rod = parse(&json!({"head": "Threaded rod", "drive": "Hex socket"})).unwrap_err();
        assert_eq!(rod.field, "drive");
    }

    #[test]
    fn drive_size_sizes_whichever_drive_is_chosen() {
        let torx = parse(&json!({"drive": "Torx", "drive_size": "T8"})).unwrap();
        assert_eq!(torx.recess.unwrap().number, 8);
        let ph = parse(&json!({"head": "Pan head", "drive": "Phillips", "drive_size": "PH1x1.5"}))
            .unwrap();
        assert_eq!(ph.phillips.unwrap().depth, 1.5);
        let slot = parse(&json!({"head": "Pan head", "drive": "Slotted", "drive_size": "0.9x0.6"}))
            .unwrap();
        assert_eq!(slot.slot.unwrap().width, 0.9);
        // The key size alone keeps the standard depth.
        let hex = parse(&json!({"drive_size": "2"})).unwrap();
        assert_eq!(
            hex.hex.unwrap(),
            Socket {
                across_flats: 2.0,
                depth: 1.3
            }
        );
        // Standard on a pan head is Phillips, so the box reads a Phillips size.
        let pan = parse(&json!({"head": "Pan head", "drive_size": "PH2"})).unwrap();
        assert_eq!(pan.phillips.unwrap().number, 2);
        // A wrong kind of size is reported against the box, in that drive's words.
        let err = parse(&json!({"drive": "Torx", "drive_size": "PH2"})).unwrap_err();
        assert_eq!(err.field, "drive_size");
        let none = parse(&json!({"drive": "None", "drive_size": "T10"})).unwrap_err();
        assert_eq!(none.field, "drive_size");
        let hex_head = parse(&json!({"head": "Hex head", "drive_size": "T10"})).unwrap_err();
        assert_eq!(hex_head.field, "drive_size");
        // The single drive field selects the same sized recess.
        let old = parse(&json!({"drive": "Torx", "drive_size": "T8"})).unwrap();
        assert_eq!(old.recess.unwrap().number, 8);
    }

    #[test]
    fn a_pan_head_is_sized_from_the_standard_of_its_drive() {
        // Phillips, which is also Standard on a pan head, is ISO 7045: 5.6 by 2.4 on an M3.
        for v in [
            json!({"head": "Pan head"}),
            json!({"head": "Pan head", "drive": "Phillips"}),
        ] {
            let p = parse(&v).unwrap();
            assert_eq!(
                p.resolved.head,
                ResolvedHead::Round {
                    diameter: 5.6,
                    height: 2.4
                }
            );
            assert_eq!(p.phillips.unwrap().depth, 1.4);
        }
        // Slotted, or no drive, is the lower ISO 1580 pan.
        for drive in ["Slotted", "None", "Torx"] {
            let p = parse(&json!({"head": "Pan head", "drive": drive})).unwrap();
            assert_eq!(
                p.resolved.head,
                ResolvedHead::Round {
                    diameter: 5.6,
                    height: 1.8
                },
                "{drive}"
            );
        }
        // Every ISO 7045 size holds its standard recess.
        for size in [
            "M1.6", "M2", "M2.5", "M3", "M3.5x0.6", "M4", "M5", "M6", "M8", "M10",
        ] {
            let p = parse(&json!({"size": size, "head": "Pan head", "length": 20})).unwrap();
            let ResolvedHead::Round { height, .. } = p.resolved.head else {
                panic!("{size}")
            };
            let tool = p.phillips.unwrap();
            assert!(
                height - tool.depth >= 0.5,
                "{size}: {height} - {}",
                tool.depth
            );
        }
        // A custom height still wins.
        let p = parse(&json!({"head": "Pan head", "head_height": 3})).unwrap();
        assert_eq!(
            p.resolved.head,
            ResolvedHead::Round {
                diameter: 5.6,
                height: 3.0
            }
        );
    }

    #[test]
    fn a_countersunk_head_is_sized_from_the_standard_of_its_drive() {
        // Hex socket, which is Standard here, Torx and None: ISO 10642.
        for v in [
            json!({"head": "Countersunk"}),
            json!({"head": "Countersunk", "drive": "Torx"}),
            json!({"head": "Countersunk", "drive": "None"}),
        ] {
            let p = parse(&v).unwrap();
            assert_eq!(
                p.resolved.head,
                ResolvedHead::Countersunk {
                    diameter: 6.72,
                    height: 1.86
                },
                "{v}"
            );
        }
        // Phillips is ISO 7046-1 and Slotted is ISO 2009: the same lower, narrower head.
        let flat = ResolvedHead::Countersunk {
            diameter: 5.5,
            height: 1.65,
        };
        let cross = parse(&json!({"head": "Countersunk", "drive": "Phillips"})).unwrap();
        assert_eq!(cross.resolved.head, flat);
        assert_eq!(cross.phillips.unwrap().depth, 1.7);
        let slotted = parse(&json!({"head": "Countersunk", "drive": "Slotted"})).unwrap();
        assert_eq!(slotted.resolved.head, flat);
        // The ISO 2009 slot, not the deeper pan-head one.
        assert_eq!(
            slotted.slot.unwrap(),
            Slot {
                width: 0.8,
                depth: 0.6
            }
        );
    }

    #[test]
    fn every_iso_7046_and_iso_2009_size_builds_with_its_standard_drive() {
        for size in [
            "M1.6", "M2", "M2.5", "M3", "M3.5x0.6", "M4", "M5", "M6", "M8", "M10",
        ] {
            let p = parse(&json!({"size": size, "head": "Countersunk",
                "drive": "Phillips", "length": 30}));
            assert!(
                p.as_ref().is_ok_and(|p| p.phillips.is_some()),
                "{size}: {p:?}"
            );
        }
        for size in [
            "M1.6", "M2", "M2.5", "M3", "M3.5x0.6", "M4", "M5", "M6", "M8", "M10",
        ] {
            let p = parse(&json!({"size": size, "head": "Countersunk",
                "drive": "Slotted", "length": 30}));
            assert!(p.as_ref().is_ok_and(|p| p.slot.is_some()), "{size}: {p:?}");
        }
    }

    #[test]
    fn a_long_screw_takes_the_standard_thread_length() {
        // DIN 912 M3: b is 18. Up to 20 mm it is threaded to the head, from 25 mm not.
        let len = |v: Value| parse(&v).unwrap().thread_length;
        assert_eq!(len(json!({"length": 12})), None);
        assert_eq!(len(json!({"length": 20})), None);
        assert_eq!(len(json!({"length": 25})), Some(18.0));
        assert_eq!(len(json!({"size": "M6", "length": 30})), None);
        assert_eq!(len(json!({"size": "M6", "length": 35})), Some(24.0));
        // A fine pitch has a shorter run-out, so the shank starts sooner.
        assert_eq!(len(json!({"size": "M8x1", "length": 37})), Some(28.0));
        assert_eq!(len(json!({"size": "M8", "length": 37})), None);
        // Each head follows the line its own standard draws.
        let at = |head: &str, drive: &str, size: &str, length: f64| {
            len(json!({"head": head, "drive": drive, "size": size, "length": length}))
        };
        // ISO 7045: to the head up to 25 mm for M3, up to 40 mm for M4.
        assert_eq!(at("Pan head", "Phillips", "M3", 25.0), None);
        assert_eq!(at("Pan head", "Phillips", "M3", 30.0), Some(25.0));
        assert_eq!(at("Pan head", "Phillips", "M4", 40.0), None);
        assert_eq!(at("Pan head", "Phillips", "M4", 45.0), Some(38.0));
        // ISO 1580 and ISO 1207: up to 30 mm for M3, up to 40 mm for M4.
        for head in ["Pan head", "Cheese head"] {
            assert_eq!(at(head, "Slotted", "M3", 30.0), None, "{head}");
            assert_eq!(at(head, "Slotted", "M3", 35.0), Some(25.0), "{head}");
            assert_eq!(at(head, "Slotted", "M4", 40.0), None, "{head}");
            assert_eq!(at(head, "Slotted", "M4", 45.0), Some(38.0), "{head}");
        }
        // ISO 2009 and ISO 7046-1: up to 30 mm for M3, up to 45 mm for M4.
        for drive in ["Slotted", "Phillips"] {
            assert_eq!(at("Countersunk", drive, "M3", 30.0), None, "{drive}");
            assert_eq!(at("Countersunk", drive, "M3", 35.0), Some(25.0), "{drive}");
            assert_eq!(at("Countersunk", drive, "M4", 45.0), None, "{drive}");
            assert_eq!(at("Countersunk", drive, "M4", 50.0), Some(38.0), "{drive}");
        }
        // ISO 10642: an M3 is shaded up to 25 mm, an M8 up to 45 mm.
        assert_eq!(at("Countersunk", "Hex socket", "M3", 25.0), None);
        assert_eq!(at("Countersunk", "Hex socket", "M3", 30.0), Some(18.0));
        assert_eq!(at("Countersunk", "Hex socket", "M8", 45.0), None);
        assert_eq!(at("Countersunk", "Hex socket", "M8", 50.0), Some(28.0));
        // A hex head, a set screw and a rod are threaded all the way.
        for head in ["Hex head", "Set screw", "Threaded rod"] {
            assert_eq!(len(json!({"head": head, "length": 60})), None, "{head}");
        }
        // What is typed wins, including the whole length.
        assert_eq!(len(json!({"length": 40, "thread_length": 40})), Some(40.0));
        assert_eq!(len(json!({"length": 40, "thread_length": 10})), Some(10.0));
        // A point is outside the thread: 25 mm with a 3 mm point leaves 22 of cylinder,
        // which still holds the 18 mm of thread.
        assert_eq!(len(json!({"length": 20, "tip": "Point"})), None);
        assert_eq!(len(json!({"length": 25, "tip": "Point"})), Some(18.0));
    }

    #[test]
    fn a_standard_head_has_its_standard_top_and_a_typed_one_is_flat() {
        let crown = |v: Value| parse(&v).unwrap().resolved.crown;
        assert_eq!(crown(json!({})), Crown::Flat);
        assert_eq!(
            crown(json!({"head": "Button head"})),
            Crown::Button {
                radius: 3.5,
                flat: 2.6
            }
        );
        assert_eq!(
            crown(json!({"head": "Pan head"})),
            Crown::Dome { radius: 5.0 }
        );
        assert_eq!(
            crown(json!({"head": "Pan head", "drive": "Slotted"})),
            Crown::Rounded { radius: 0.9 }
        );
        assert_eq!(crown(json!({"head": "Cheese head"})), Crown::Flat);
        assert_eq!(
            crown(json!({"head": "Button head", "head_height": 2})),
            Crown::Flat
        );
    }

    #[test]
    fn the_reply_names_the_standard_of_the_head_and_drive() {
        let standard = |v: Value| parse(&v).unwrap().standard;
        assert_eq!(standard(json!({})), Some("ISO 4762"));
        assert_eq!(standard(json!({"head": "Button head"})), Some("ISO 7380-1"));
        assert_eq!(standard(json!({"head": "Countersunk"})), Some("ISO 10642"));
        assert_eq!(
            standard(json!({"head": "Countersunk", "drive": "Phillips"})),
            Some("ISO 7046-1")
        );
        assert_eq!(standard(json!({"head": "Pan head"})), Some("ISO 7045"));
        assert_eq!(
            standard(json!({"head": "Countersunk", "drive": "Slotted"})),
            Some("ISO 2009")
        );
        assert_eq!(
            standard(json!({"head": "Pan head", "drive": "Slotted"})),
            Some("ISO 1580")
        );
        assert_eq!(standard(json!({"head": "Cheese head"})), Some("ISO 1207"));
        assert_eq!(standard(json!({"head": "Hex head"})), Some("ISO 4017"));
        assert_eq!(standard(json!({"head": "Set screw"})), Some("ISO 4026"));
        assert_eq!(
            standard(json!({"head": "Set screw", "drive": "Slotted"})),
            Some("ISO 4766")
        );
        assert_eq!(standard(json!({"head": "Threaded rod"})), None);
        assert_eq!(standard(json!({"head_height": 4})), None);
    }

    #[test]
    fn a_hole_is_planned_only_when_create_asks_for_one() {
        let p = parse(&json!({})).unwrap();
        assert_eq!(p.create, Create::Part);
        assert_eq!(p.hole, None);
        let p = parse(&json!({"create": "Screw and hole"})).unwrap();
        assert_eq!(
            p.hole,
            Some(Hole {
                kind: HoleKind::Clearance {
                    size: clearance::Hole::Size("M3".into())
                },
                depth: 12.0,
                sink: 0.0
            })
        );
        // A fine pitch still takes the clearance hole of its nominal size, and taps its own.
        let fine = json!({"size": "M8x1", "length": 20, "create": "Hole only"});
        assert_eq!(
            parse(&fine).unwrap().hole.unwrap().kind,
            HoleKind::Clearance {
                size: clearance::Hole::Size("M8".into())
            }
        );
        let mut tapped = fine.clone();
        tapped["hole_type"] = json!(TAPPED);
        assert_eq!(
            parse(&tapped).unwrap().hole.unwrap().kind,
            HoleKind::Tapped {
                thread: "M8x1".into()
            }
        );
        let err = parse(&json!({"create": "Hole only", "hole_type": "Square"})).unwrap_err();
        assert_eq!(err.field, "hole_type");
        let err = parse(&json!({"create": "Sideways"})).unwrap_err();
        assert_eq!(err.field, "create");
    }

    /// ISO 4029:2003 Table 1 and Figure 1: `dz` max, the 120° cup and the 45° chamfer.
    #[test]
    fn a_cup_point_set_screw_follows_iso_4029() {
        let iso_4029 = [
            (1.6, 0.8),
            (2.0, 1.0),
            (2.5, 1.2),
            (3.0, 1.4),
            (4.0, 2.0),
            (5.0, 2.5),
            (6.0, 3.0),
            (8.0, 5.0),
            (10.0, 6.0),
            (12.0, 8.0),
            (16.0, 10.0),
            (20.0, 14.0),
            (24.0, 16.0),
        ];
        for (nominal, mouth) in iso_4029 {
            assert_eq!(cup_diameter(nominal), mouth, "M{nominal}");
            // 120° included: the side is 60° off the axis.
            let depth = cup_depth(mouth);
            assert!(
                (depth * 3.0_f64.sqrt() - mouth / 2.0).abs() < 1e-12,
                "M{nominal}"
            );
        }
        let cup = parse(&json!({"head": "Set screw", "tip": "Cup", "length": 6})).unwrap();
        assert_eq!(cup.standard, Some("ISO 4029"));
        // The outside is chamfered at 45° from 3 mm down to the 1.4 mm mouth.
        assert!((cup.point_chamfer().unwrap() - 0.8).abs() < 1e-12);
        assert!((cup.socket_end_chamfer().unwrap() - 0.3067).abs() < 1e-9);
        // ISO 4029 shades M3 x 4, so that length keeps the shallower socket. The flat
        // point of ISO 4026 is already at the deeper one there.
        let short = parse(&json!({"head": "Set screw", "tip": "Cup", "length": 4})).unwrap();
        assert_eq!(short.hex.unwrap().depth, 1.2);
        let flat = parse(&json!({"head": "Set screw", "length": 4})).unwrap();
        assert_eq!(flat.hex.unwrap().depth, 2.0);
        assert_eq!(flat.standard, Some("ISO 4026"));
        assert_eq!(cup.hex.unwrap().depth, 2.0);
        // A cup on a headed screw is not that standard's, and is not chamfered.
        let headed = parse(&json!({"tip": "Cup"})).unwrap();
        assert_eq!(headed.point_chamfer(), None);
        assert_eq!(headed.standard, Some("ISO 4762"));
        // Nor is a cup on a size the standard does not list.
        let odd = parse(
            &json!({"custom_size": "M14", "head": "Set screw", "tip": "Cup",
            "drive": "None", "length": 30}),
        )
        .unwrap();
        assert_eq!(odd.point_chamfer(), None);
    }

    #[test]
    fn a_set_screw_has_the_iso_4026_flat_point_and_end_chamfers() {
        let set = parse(&json!({"head": "Set screw", "length": 6})).unwrap();
        // M3: dp is 2, so the 45° chamfer runs 0.5 mm up the shank.
        assert_eq!(set.resolved.tip, ResolvedTip::FlatPoint { diameter: 2.0 });
        assert_eq!(set.point_chamfer(), Some(0.5));
        // The socket end is chamfered to the thread root: 0.6134 of the 0.5 pitch.
        assert!((set.socket_end_chamfer().unwrap() - 0.3067).abs() < 1e-9);
        // A slotted set screw has the point and no socket end.
        let slotted =
            parse(&json!({"head": "Set screw", "drive": "Slotted", "length": 6})).unwrap();
        assert_eq!(slotted.point_chamfer(), Some(0.5));
        assert_eq!(slotted.socket_end_chamfer(), None);
        // A headed screw and a rod have neither.
        for v in [json!({}), json!({"head": "Threaded rod"})] {
            let p = parse(&v).unwrap();
            assert_eq!(p.point_chamfer(), None, "{v}");
        }
        assert_eq!(parse(&json!({})).unwrap().socket_end_chamfer(), None);
        // A size ISO 4026 does not list keeps a square end.
        let odd =
            parse(&json!({"custom_size": "M7x1", "head": "Set screw", "drive": "None"})).unwrap();
        assert_eq!(odd.resolved.tip, ResolvedTip::Flat);
    }

    #[test]
    fn a_countersunk_length_includes_the_head() {
        // ISO 10642 M3: k = 1.86, so an M3 x 12 has 10.14 mm under the head.
        let p = parse(&json!({"head": "Countersunk", "length": 12})).unwrap();
        let ResolvedHead::Countersunk { height, .. } = p.resolved.head else {
            panic!("not countersunk");
        };
        assert_eq!(p.nominal_length, 12.0);
        assert!((p.length + height - 12.0).abs() < 1e-9);
        assert!(p.length < 12.0);
        // The recessed hole sinks the head flush, so it is as deep as the overall length.
        let hole = parse(&json!({
            "head": "Countersunk", "length": 12, "create": "Screw and hole",
            "hole_type": RECESSED,
        }))
        .unwrap()
        .hole
        .unwrap();
        assert!((hole.depth - 12.0).abs() < 1e-9);
        // Other heads keep the length under the head.
        let socket = parse(&json!({"length": 12})).unwrap();
        assert_eq!((socket.length, socket.nominal_length), (12.0, 12.0));
    }

    #[test]
    fn a_countersunk_screw_has_to_be_longer_than_its_head() {
        let err = parse(&json!({"head": "Countersunk", "length": 1.5})).unwrap_err();
        assert_eq!(err.field, "length");
        assert!(err.reason.contains("1.86 mm head"), "{}", err.reason);
    }

    #[test]
    fn the_countersunk_plain_shank_follows_the_overall_length() {
        // ISO 10642 M3 has a plain shank from 30 mm overall.
        let short = parse(&json!({"head": "Countersunk", "length": 25})).unwrap();
        assert_eq!(short.thread_length, None);
        let long = parse(&json!({"head": "Countersunk", "length": 30})).unwrap();
        assert!(long.thread_length.is_some());
    }

    #[test]
    fn a_hole_alone_does_not_need_a_standard_head() {
        let p = parse(&json!({
            "size": "M12", "head": "Pan head", "create": "Hole only",
            "hole_type": CLEARANCE,
        }))
        .unwrap();
        assert_eq!(p.hole.unwrap().depth, 12.0);
        // Nor a drive that fits.
        parse(&json!({
            "head": "Socket head", "drive": "Torx", "drive_size": "T100",
            "create": "Hole only",
        }))
        .unwrap();
        // A recessed hole still needs the head it recesses.
        let err = parse(&json!({
            "size": "M12", "head": "Pan head", "create": "Hole only",
            "hole_type": RECESSED,
        }))
        .unwrap_err();
        assert_eq!(err.field, "head_diameter");
    }

    #[test]
    fn a_small_countersunk_screw_takes_the_phillips_head_by_default() {
        let p = parse(&json!({"size": "M2", "head": "Countersunk"})).unwrap();
        assert_eq!(p.drive, Drive::Phillips);
        assert_eq!(p.standard, Some("ISO 7046-1"));
        assert!(p.phillips.is_some());
        let m3 = parse(&json!({"size": "M3", "head": "Countersunk"})).unwrap();
        assert_eq!(m3.drive, Drive::HexSocket);
    }

    #[test]
    fn a_bad_custom_size_is_reported_on_custom_size() {
        let err = parse(&json!({"size": "M3", "custom_size": "M7.5"})).unwrap_err();
        assert_eq!(err.field, "custom_size", "{}", err.reason);
        let err = parse(&json!({"size": "M3", "custom_size": "Mx"})).unwrap_err();
        assert_eq!(err.field, "custom_size", "{}", err.reason);
    }

    #[test]
    fn a_hole_for_a_size_the_host_has_no_clearance_for_goes_by_diameter() {
        let p = parse(&json!({
            "custom_size": "M3.5x0.6", "length": 10, "create": "Screw and hole",
            "head_diameter": 6, "head_height": 2.4,
        }))
        .unwrap();
        assert_eq!(
            p.hole.unwrap().kind,
            HoleKind::Clearance {
                size: clearance::Hole::Diameter(3.9)
            }
        );
        let err = parse(&json!({
            "custom_size": "M66x4", "length": 100, "create": "Hole only",
        }))
        .unwrap_err();
        assert_eq!(err.field, "custom_size", "{}", err.reason);
    }

    #[test]
    fn a_set_screw_shorter_than_its_chamfers_is_refused() {
        let err = parse(&json!({"head": "Set screw", "drive": "None", "length": 0.4})).unwrap_err();
        assert_eq!(err.field, "length");
        assert!(err.reason.contains("chamfers"), "{}", err.reason);
    }

    #[test]
    fn a_full_length_thread_on_a_set_screw_stays_between_its_chamfers() {
        let p = parse(&json!({"head": "Set screw", "length": 6, "thread_length": 6})).unwrap();
        let room = p.thread_room();
        assert!(room < 6.0);
        assert_eq!(p.thread_length, Some(room));
    }

    #[test]
    fn a_short_set_screw_is_threaded_up_to_its_120_degree_chamfer() {
        // M3 x 3 flat point: the 45° point chamfer takes (3 - 2) / 2 = 0.5 mm of the
        // length, and the 120° socket end chamfer 0.3067 / tan 60° = 0.17707 mm, not
        // the 0.3067 mm it narrows the end by.
        let short = parse(&json!({"head": "Set screw", "length": 3, "thread_length": 3})).unwrap();
        assert_eq!(short.socket_end_angle_deg(), 120.0);
        let room = short.thread_room();
        assert!((room - (3.0 - 0.5 - 0.17707)).abs() < 1e-5, "{room}");
        assert_eq!(short.thread_length, Some(room));
        // At 90° the chamfer runs as far down the shank as it narrows the end.
        let longer = parse(&json!({"head": "Set screw", "length": 4})).unwrap();
        assert!((longer.thread_room() - (4.0 - 0.5 - 0.3067)).abs() < 1e-9);
        // The shortest length is counted the same way: with a socket shallow enough to
        // fit, 0.5 + 0.17707 mm of chamfers leave one 0.5 mm pitch in 1.2 mm.
        let shallow = |length: f64| {
            parse(&json!({"head": "Set screw", "drive_size": "1.5x0.2", "length": length}))
        };
        assert!((shallow(1.2).unwrap().thread_room() - 0.52293).abs() < 1e-5);
        let err = shallow(1.1).unwrap_err();
        assert_eq!(err.field, "length");
        assert!(err.reason.contains("take 0.6771 mm"), "{}", err.reason);
    }

    #[test]
    fn a_short_set_screw_has_a_120_degree_socket_end() {
        // ISO 4026 shades M3 x 3. The chamfer narrows the end by the same 0.3067 mm a
        // side, and at 120° it runs 0.3067 / tan 60° down the shank.
        let short = parse(&json!({"head": "Set screw", "length": 3})).unwrap();
        assert_eq!(short.socket_end_angle_deg(), 120.0);
        assert!((short.socket_end_chamfer().unwrap() - 0.3067).abs() < 1e-9);
        assert!((short.socket_end_chamfer_length().unwrap() - 0.17707).abs() < 1e-5);
        let longer = parse(&json!({"head": "Set screw", "length": 4})).unwrap();
        assert_eq!(longer.socket_end_angle_deg(), 90.0);
        assert!((longer.socket_end_chamfer_length().unwrap() - 0.3067).abs() < 1e-9);
        // ISO 4029 shades M3 x 4 for a cup point.
        let cup = parse(&json!({"head": "Set screw", "tip": "Cup", "length": 4})).unwrap();
        assert_eq!(cup.socket_end_angle_deg(), 120.0);
        // A headed screw has no socket end chamfer at all.
        assert_eq!(
            parse(&json!({"length": 3}))
                .unwrap()
                .socket_end_chamfer_length(),
            None
        );
    }
}
