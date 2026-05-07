use crate::error::GpResult;

/// All Bend presets
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BendType {
    /// No Preset.
    None,

    // Bends
    /// A simple bend.
    Bend,
    /// A bend and release afterwards.
    BendRelease,
    /// A bend, then release and rebend.
    BendReleaseBend,
    /// Prebend.
    Prebend,
    /// Prebend and then release.
    PrebendRelease,

    // Tremolo Bar
    /// Dip the bar down and then back up.
    Dip,
    /// Dive the bar.
    Dive,
    /// Release the bar up.
    ReleaseUp,
    /// Dip the bar up and then back down.
    InvertedDip,
    /// Return the bar.
    Return,
    /// Release the bar down.
    ReleaseDown,
}
pub(crate) fn get_bend_type(value: i8) -> GpResult<BendType> {
    match value {
        0 => Ok(BendType::None),
        1 => Ok(BendType::Bend),
        2 => Ok(BendType::BendRelease),
        3 => Ok(BendType::BendReleaseBend),
        4 => Ok(BendType::Prebend),
        5 => Ok(BendType::PrebendRelease),
        6 => Ok(BendType::Dip),
        7 => Ok(BendType::Dive),
        8 => Ok(BendType::ReleaseUp),
        9 => Ok(BendType::InvertedDip),
        10 => Ok(BendType::Return),
        11 => Ok(BendType::ReleaseDown),
        _ => Ok(BendType::None),
    }
}
pub(crate) fn from_bend_type(value: &BendType) -> i8 {
    match value {
        BendType::None => 0,
        BendType::Bend => 1,
        BendType::BendRelease => 2,
        BendType::BendReleaseBend => 3,
        BendType::Prebend => 4,
        BendType::PrebendRelease => 5,
        BendType::Dip => 6,
        BendType::Dive => 7,
        BendType::ReleaseUp => 8,
        BendType::InvertedDip => 9,
        BendType::Return => 10,
        BendType::ReleaseDown => 11,
    }
}

/// All transition types for grace notes.
#[repr(i8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraceEffectTransition {
    /// No transition
    None = 0,
    /// Slide from the grace note to the real one.
    Slide,
    /// Perform a bend from the grace note to the real one.
    Bend,
    /// Perform a hammer on.
    Hammer,
}
pub(crate) fn get_grace_effect_transition(value: i8) -> GpResult<GraceEffectTransition> {
    match value {
        0 => Ok(GraceEffectTransition::None),
        1 => Ok(GraceEffectTransition::Slide),
        2 => Ok(GraceEffectTransition::Bend),
        3 => Ok(GraceEffectTransition::Hammer),
        _ => Ok(GraceEffectTransition::None),
    }
}
pub(crate) fn from_grace_effect_transition(value: &GraceEffectTransition) -> i8 {
    match value {
        GraceEffectTransition::None => 0,
        GraceEffectTransition::Slide => 1,
        GraceEffectTransition::Bend => 2,
        GraceEffectTransition::Hammer => 3,
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarmonicType {
    Natural = 1,
    Artificial,
    Tapped,
    Pinch,
    Semi,
}
pub(crate) fn from_harmonic_type(value: &HarmonicType) -> i8 {
    match value {
        HarmonicType::Natural => 1,
        HarmonicType::Artificial => 2,
        HarmonicType::Tapped => 3,
        HarmonicType::Pinch => 4,
        HarmonicType::Semi => 5,
    }
}
