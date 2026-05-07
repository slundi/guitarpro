use crate::error::GpResult;

/// Type of the chord.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordType {
    /// Major chord.
    Major,
    /// Dominant seventh chord.
    Seventh,
    /// Major seventh chord.
    MajorSeventh,
    /// Add sixth chord.
    Sixth,
    /// Minor chord.
    Minor,
    /// Minor seventh chord.
    MinorSeventh,
    /// Minor major seventh chord.
    MinorMajor,
    /// Minor add sixth chord.
    MinorSixth,
    /// Suspended second chord.
    SuspendedSecond,
    /// Suspended fourth chord.
    SuspendedFourth,
    /// Seventh suspended second chord.
    SeventhSuspendedSecond,
    /// Seventh suspended fourth chord.
    SeventhSuspendedFourth,
    /// Diminished chord.
    Diminished,
    /// Augmented chord.
    Augmented,
    /// Power chord.
    Power,

    Unknown(u8),
}
pub(crate) fn get_chord_type(value: u8) -> ChordType {
    match value {
        0 => ChordType::Major,
        1 => ChordType::Seventh,
        2 => ChordType::MajorSeventh,
        3 => ChordType::Sixth,
        4 => ChordType::Minor,
        5 => ChordType::MinorSeventh,
        6 => ChordType::MinorMajor,
        7 => ChordType::MinorSixth,
        8 => ChordType::SuspendedSecond,
        9 => ChordType::SuspendedFourth,
        10 => ChordType::SeventhSuspendedSecond,
        11 => ChordType::SeventhSuspendedFourth,
        12 => ChordType::Diminished,
        13 => ChordType::Augmented,
        14 => ChordType::Power,
        _ => ChordType::Unknown(value),
    }
}
pub(crate) fn from_chord_type(value: &ChordType) -> u8 {
    match value {
        ChordType::Major => 0,
        ChordType::Seventh => 1,
        ChordType::MajorSeventh => 2,
        ChordType::Sixth => 3,
        ChordType::Minor => 4,
        ChordType::MinorSeventh => 5,
        ChordType::MinorMajor => 6,
        ChordType::MinorSixth => 7,
        ChordType::SuspendedSecond => 8,
        ChordType::SuspendedFourth => 9,
        ChordType::SeventhSuspendedSecond => 10,
        ChordType::SeventhSuspendedFourth => 11,
        ChordType::Diminished => 12,
        ChordType::Augmented => 13,
        ChordType::Power => 14,
        ChordType::Unknown(value) => *value,
    }
}

/// Tonality of the chord
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordAlteration {
    /// Perfect.
    Perfect,
    /// Diminished.
    Diminished,
    /// Augmented.
    Augmented,
}
pub(crate) fn get_chord_alteration(value: u8) -> GpResult<ChordAlteration> {
    match value {
        0 => Ok(ChordAlteration::Perfect),
        1 => Ok(ChordAlteration::Diminished),
        2 => Ok(ChordAlteration::Augmented),
        _ => Ok(ChordAlteration::Perfect),
    }
}
pub(crate) fn from_chord_alteration(value: &ChordAlteration) -> u8 {
    match value {
        ChordAlteration::Perfect => 0,
        ChordAlteration::Diminished => 1,
        ChordAlteration::Augmented => 2,
    }
}

/// Extension type of the chord
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordExtension {
    None,
    /// Ninth chord.
    Ninth,
    /// Eleventh chord.
    Eleventh,
    /// Thirteenth chord.
    Thirteenth,
    Unknown(u8),
}
pub(crate) fn get_chord_extension(value: u8) -> ChordExtension {
    match value {
        0 => ChordExtension::None,
        1 => ChordExtension::Ninth,
        2 => ChordExtension::Eleventh,
        3 => ChordExtension::Thirteenth,
        _ => ChordExtension::Unknown(value),
    }
}
pub(crate) fn from_chord_extension(value: &ChordExtension) -> u8 {
    match value {
        ChordExtension::None => 0,
        ChordExtension::Ninth => 1,
        ChordExtension::Eleventh => 2,
        ChordExtension::Thirteenth => 3,
        ChordExtension::Unknown(value) => *value,
    }
}
