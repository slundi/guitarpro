use crate::error::{GpError, GpResult};

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeatStatus {
    Empty,
    Normal,
    Rest,
}
pub(crate) fn get_beat_status(value: u8) -> BeatStatus {
    match value {
        0 => BeatStatus::Empty,
        1 => BeatStatus::Normal,
        2 => BeatStatus::Rest,
        _ => BeatStatus::Normal,
    }
}
pub(crate) fn from_beat_status(value: &BeatStatus) -> u8 {
    match value {
        BeatStatus::Empty => 0,
        BeatStatus::Normal => 1,
        BeatStatus::Rest => 2,
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TupletBracket {
    None,
    Start,
    End,
}

/// Octave signs
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Octave {
    None,
    Ottava,
    Quindicesima,
    OttavaBassa,
    QuindicesimaBassa,
}
pub(crate) fn get_octave(value: u8) -> GpResult<Octave> {
    match value {
        0 => Ok(Octave::None),
        1 => Ok(Octave::Ottava),
        2 => Ok(Octave::Quindicesima),
        3 => Ok(Octave::OttavaBassa),
        4 => Ok(Octave::QuindicesimaBassa),
        _ => Err(GpError::InvalidValue {
            context: "octave",
            value: value as i64,
        }),
    }
}
pub(crate) fn from_octave(value: &Octave) -> u8 {
    match value {
        Octave::None => 0,
        Octave::Ottava => 1,
        Octave::Quindicesima => 2,
        Octave::OttavaBassa => 3,
        Octave::QuindicesimaBassa => 4,
    }
}

/// All beat stroke directions
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeatStrokeDirection {
    None,
    Up,
    Down,
}
pub(crate) fn get_beat_stroke_direction(value: i8) -> GpResult<BeatStrokeDirection> {
    match value {
        0 => Ok(BeatStrokeDirection::None),
        1 => Ok(BeatStrokeDirection::Up),
        2 => Ok(BeatStrokeDirection::Down),
        _ => Ok(BeatStrokeDirection::None),
    }
}
pub(crate) fn from_beat_stroke_direction(value: &BeatStrokeDirection) -> i8 {
    match value {
        BeatStrokeDirection::None => 0,
        BeatStrokeDirection::Up => 1,
        BeatStrokeDirection::Down => 2,
    }
}

/// Characteristic of articulation
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlapEffect {
    None,
    Tapping,
    Slapping,
    Popping,
}
pub(crate) fn get_slap_effect(value: u8) -> GpResult<SlapEffect> {
    match value {
        0 => Ok(SlapEffect::None),
        1 => Ok(SlapEffect::Tapping),
        2 => Ok(SlapEffect::Slapping),
        3 => Ok(SlapEffect::Popping),
        _ => Ok(SlapEffect::None),
    }
}
pub(crate) fn from_slap_effect(value: &SlapEffect) -> u8 {
    match value {
        SlapEffect::None => 0,
        SlapEffect::Tapping => 1,
        SlapEffect::Slapping => 2,
        SlapEffect::Popping => 3,
    }
}

/// Voice directions indicating the direction of beams
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceDirection {
    None,
    Up,
    Down,
}
