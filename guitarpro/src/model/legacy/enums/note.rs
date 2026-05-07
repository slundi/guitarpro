use crate::error::{GpError, GpResult};

/// An enumeration of all supported slide types.
#[repr(i8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideType {
    IntoFromAbove = -2,
    IntoFromBelow = -1,
    None = 0,
    ShiftSlideTo,
    LegatoSlideTo,
    OutDownwards,
    OutUpWards,
}
pub(crate) fn get_slide_type(value: i8) -> GpResult<SlideType> {
    match value {
        -2 => Ok(SlideType::IntoFromAbove),
        -1 => Ok(SlideType::IntoFromBelow),
        0 => Ok(SlideType::None),
        1 => Ok(SlideType::ShiftSlideTo),
        2 => Ok(SlideType::LegatoSlideTo),
        3 => Ok(SlideType::OutDownwards),
        4 => Ok(SlideType::OutUpWards),
        _ => Err(GpError::InvalidValue {
            context: "slide type",
            value: value as i64,
        }),
    }
}
pub(crate) fn from_slide_type(value: &SlideType) -> i8 {
    match value {
        SlideType::IntoFromAbove => -2,
        SlideType::IntoFromBelow => -1,
        SlideType::None => 0,
        SlideType::ShiftSlideTo => 1,
        SlideType::LegatoSlideTo => 2,
        SlideType::OutDownwards => 3,
        SlideType::OutUpWards => 4,
    }
}

/// An enumeration of all supported note types.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteType {
    Rest,
    Normal,
    Tie,
    Dead,
    Unknown(u8),
}
pub(crate) fn get_note_type(value: u8) -> NoteType {
    match value {
        0 => NoteType::Rest,
        1 => NoteType::Normal,
        2 => NoteType::Tie,
        3 => NoteType::Dead,
        _ => NoteType::Unknown(value),
    }
}
pub(crate) fn from_note_type(value: &NoteType) -> u8 {
    match value {
        NoteType::Rest => 0,
        NoteType::Normal => 1,
        NoteType::Tie => 2,
        NoteType::Dead => 3,
        NoteType::Unknown(value) => *value,
    }
}

/// Left and right hand fingering used in tabs and chord diagram editor.
#[repr(i8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fingering {
    /// Open or muted.
    Open = -1,
    /// Thumb.
    Thumb = 0,
    /// Index finger.
    Index,
    /// Middle finger.
    Middle,
    /// Annular finger.
    Annular,
    /// Little finger.
    Little,

    Unknown(i8),
}
pub(crate) fn get_fingering(value: i8) -> Fingering {
    match value {
        -1 => Fingering::Open,
        0 => Fingering::Thumb,
        1 => Fingering::Index,
        2 => Fingering::Middle,
        3 => Fingering::Annular,
        4 => Fingering::Little,
        _ => Fingering::Unknown(value),
    }
}
pub(crate) fn from_fingering(value: &Fingering) -> i8 {
    match value {
        Fingering::Open => -1,
        Fingering::Thumb => 0,
        Fingering::Index => 1,
        Fingering::Middle => 2,
        Fingering::Annular => 3,
        Fingering::Little => 4,
        Fingering::Unknown(value) => *value,
    }
}

/// Values of auto-accentuation on the beat found in track RSE settings
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Accentuation {
    None,
    VerySoft,
    Soft,
    Medium,
    Strong,
    VeryStrong,
}
pub(crate) fn get_accentuation(value: u8) -> GpResult<Accentuation> {
    match value {
        0 => Ok(Accentuation::None),
        1 => Ok(Accentuation::VerySoft),
        2 => Ok(Accentuation::Soft),
        3 => Ok(Accentuation::Medium),
        4 => Ok(Accentuation::Strong),
        5 => Ok(Accentuation::VeryStrong),
        _ => Err(GpError::InvalidValue {
            context: "accentuation",
            value: value as i64,
        }),
    }
}
pub(crate) fn from_accentuation(value: &Accentuation) -> u8 {
    match value {
        Accentuation::None => 0,
        Accentuation::VerySoft => 1,
        Accentuation::Soft => 2,
        Accentuation::Medium => 3,
        Accentuation::Strong => 4,
        Accentuation::VeryStrong => 5,
    }
}
