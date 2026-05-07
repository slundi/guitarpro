use crate::error::{GpError, GpResult};

/// An enumeration of different triplet feels.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TripletFeel {
    None,
    Eighth,
    Sixteenth,
}
pub(crate) fn get_triplet_feel(value: i8) -> GpResult<TripletFeel> {
    match value {
        0 => Ok(TripletFeel::None),
        1 => Ok(TripletFeel::Eighth),
        2 => Ok(TripletFeel::Sixteenth),
        _ => Err(GpError::InvalidValue {
            context: "triplet feel",
            value: value as i64,
        }),
    }
}
pub(crate) fn from_triplet_feel(value: &TripletFeel) -> u8 {
    match value {
        TripletFeel::None => 0,
        TripletFeel::Eighth => 1,
        TripletFeel::Sixteenth => 2,
    }
}

/// An enumeration of available clefs
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum MeasureClef {
    Treble,
    Bass,
    Tenor,
    Alto,
}

/// A line break directive: `NONE: no line break`, `BREAK: break line`, `Protect the line from breaking`.
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum LineBreak {
    None,
    Break,
    Protect,
}
pub(crate) fn get_line_break(value: u8) -> LineBreak {
    match value {
        1 => LineBreak::Break,
        2 => LineBreak::Protect,
        _ => LineBreak::None,
    }
}
pub(crate) fn from_line_break(value: &LineBreak) -> u8 {
    match value {
        LineBreak::None => 0,
        LineBreak::Break => 1,
        LineBreak::Protect => 2,
    }
}

/// A navigation sign like *Coda* (𝄌: U+1D10C) or *Segno* (𝄋 or 𝄉: U+1D10B or U+1D109).
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirectionSign {
    Coda,
    DoubleCoda,
    Segno,
    SegnoSegno,
    Fine,
    DaCapo,
    DaCapoAlCoda,
    DaCapoAlDoubleCoda,
    DaCapoAlFine,
    DaSegno,
    DaSegnoAlCoda,
    DaSegnoAlDoubleCoda,
    DaSegnoAlFine,
    DaSegnoSegno,
    DaSegnoSegnoAlCoda,
    DaSegnoSegnoAlDoubleCoda,
    DaSegnoSegnoAlFine,
    DaCoda,
    DaDoubleCoda,
}
