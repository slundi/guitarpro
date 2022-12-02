
/// An enumeration of different triplet feels.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TripletFeel { None, Eighth, Sixteenth }
pub(crate) fn get_triplet_feel(value: i8) -> TripletFeel {
    match value {
        0 => TripletFeel::None,
        1 => TripletFeel::Eighth,
        2 => TripletFeel::Sixteenth,
        _ => panic!("Invalid triplet feel"),
    }
}
pub(crate) fn from_triplet_feel(value: &TripletFeel) -> u8 {
    match value {
        TripletFeel::None       => 0,
        TripletFeel::Eighth     => 1,
        TripletFeel::Sixteenth  => 2,
    }
}

/// An enumeration of available clefs
#[allow(dead_code)]
#[derive(Debug,Clone)]
pub enum MeasureClef { Treble, Bass, Tenor, Alto }
/// A line break directive: `NONE: no line break`, `BREAK: break line`, `Protect the line from breaking`.
#[derive(Debug,Clone)]
pub enum LineBreak { None, Break, Protect }
pub(crate) fn get_line_break(value: u8) -> LineBreak {
    match value {
        1 => LineBreak::Break,
        2 => LineBreak::Protect,
        _ => LineBreak::None,
    }
}
pub(crate) fn from_line_break(value: &LineBreak) -> u8 {
    match value {
        LineBreak::None    => 0,
        LineBreak::Break   => 1,
        LineBreak::Protect => 2,
    }
}

/// An enumeration of all supported slide types.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum SlideType {
    IntoFromAbove, //-2
    IntoFromBelow, //-1
    None, //0
    ShiftSlideTo,
    LegatoSlideTo,
    OutDownwards,
    OutUpWards
}
pub(crate) fn get_slide_type(value: i8) -> SlideType {
    match value {
        -2 => SlideType::IntoFromAbove,
        -1 => SlideType::IntoFromBelow,
        0  => SlideType::None,
        1  => SlideType::ShiftSlideTo,
        2  => SlideType::LegatoSlideTo,
        3  => SlideType::OutDownwards,
        4  => SlideType::OutUpWards,
        _ => panic!("Invalid slide type"),
    }
}
pub(crate) fn from_slide_type(value: &SlideType) -> i8 {
    match value {
        SlideType::IntoFromAbove  => -2,
        SlideType::IntoFromBelow  => -1,
        SlideType::None           => 0,
        SlideType::ShiftSlideTo   => 1,
        SlideType::LegatoSlideTo  => 2,
        SlideType::OutDownwards   => 3,
        SlideType::OutUpWards     => 4,
    }
}

/// An enumeration of all supported slide types.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum NoteType {
    Rest, //0
    Normal, Tie, Dead,
    Unknown(u8),
}
pub(crate) fn get_note_type(value: u8) -> NoteType {
    match value {
        0 => NoteType::Rest,
        1 => NoteType::Normal,
        2 => NoteType::Tie,
        3 => NoteType::Dead,
        _ => NoteType::Unknown(value), //panic!("Cannot read note type"),
    }
}
pub(crate) fn from_note_type(value: &NoteType) -> u8 {
    match value {
        NoteType::Rest             => 0,
        NoteType::Normal           => 1,
        NoteType::Tie              => 2,
        NoteType::Dead             => 3,
        NoteType::Unknown(value)   => *value, //panic!("Cannot read note type"),
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BeatStatus {Empty, Normal, Rest}
pub(crate) fn get_beat_status(value: u8) -> BeatStatus {
    match value {
        0 => BeatStatus::Empty,
        1 => BeatStatus::Normal,
        2 => BeatStatus::Rest,
        _ => BeatStatus::Normal, //panic!("Cannot get beat status"),
    }
}
pub(crate) fn from_beat_status(value: &BeatStatus) -> u8 {
    match value {
        BeatStatus::Empty => 0,
        BeatStatus::Normal => 1,
        BeatStatus::Rest => 2,
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TupletBracket {None, Start, End}

/// Octave signs
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Octave { None, Ottava, Quindicesima, OttavaBassa, QuindicesimaBassa }
pub(crate) fn get_octave(value: u8) -> Octave {
    match value {
        0 => Octave::None,
        1 => Octave::Ottava,
        2 => Octave::Quindicesima,
        3 => Octave::OttavaBassa,
        4 => Octave::QuindicesimaBassa,
        _ => panic!("Cannot get octave value"),
    }
}
pub(crate) fn from_octave(value: &Octave) -> u8 {
    match value {
        Octave::None               => 0,
        Octave::Ottava             => 1,
        Octave::Quindicesima       => 2,
        Octave::OttavaBassa        => 3,
        Octave::QuindicesimaBassa  => 4,
    }
}

/// All beat stroke directions
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BeatStrokeDirection { None, Up, Down }
pub(crate) fn get_beat_stroke_direction(value: i8) -> BeatStrokeDirection {
    match value {
        0 => BeatStrokeDirection::None,
        1 => BeatStrokeDirection::Up,
        2 => BeatStrokeDirection::Down,
        _ => panic!("Cannot read beat stroke direction"),
    }
}
pub(crate) fn from_beat_stroke_direction(value: &BeatStrokeDirection) -> i8 {
    match value {
        BeatStrokeDirection::None => 0,
        BeatStrokeDirection::Up   => 1,
        BeatStrokeDirection::Down => 2,
    }
}
/// Characteristic of articulation
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum SlapEffect { None, Tapping, Slapping, Popping }
pub(crate) fn get_slap_effect(value: u8) -> SlapEffect {
    match value {
        0 => SlapEffect::None,
        1 => SlapEffect::Tapping,
        2 => SlapEffect::Slapping,
        3 => SlapEffect::Popping,
        _ => panic!("Cannot read slap effect for the beat effects"),
    }
}
pub(crate) fn from_slap_effect(value: &SlapEffect) -> u8 {
    match value {
        SlapEffect::None     => 0,
        SlapEffect::Tapping  => 1,
        SlapEffect::Slapping => 2,
        SlapEffect::Popping  => 3,
    }
}


/// Voice directions indicating the direction of beams
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum VoiceDirection { None, Up, Down }

/// Type of the chord.
#[derive(Debug,Clone,PartialEq,Eq)]
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
        0  => ChordType::Major,
        1  => ChordType::Seventh,
        2  => ChordType::MajorSeventh,
        3  => ChordType::Sixth,
        4  => ChordType::Minor,
        5  => ChordType::MinorSeventh,
        6  => ChordType::MinorMajor,
        7  => ChordType::MinorSixth,
        8  => ChordType::SuspendedSecond,
        9  => ChordType::SuspendedFourth,
        10 => ChordType::SeventhSuspendedSecond,
        11 => ChordType::SeventhSuspendedFourth,
        12 => ChordType::Diminished,
        13 => ChordType::Augmented,
        14 => ChordType::Power,
        _  => ChordType::Unknown(value), //panic!("Cannot read chord type (new format)"),
    }
}
pub(crate) fn from_chord_type(value: &ChordType) -> u8 {
    match value {
        ChordType::Major                    => 0,
        ChordType::Seventh                  => 1,
        ChordType::MajorSeventh             => 2,
        ChordType::Sixth                    => 3,
        ChordType::Minor                    => 4,
        ChordType::MinorSeventh             => 5,
        ChordType::MinorMajor               => 6,
        ChordType::MinorSixth               => 7,
        ChordType::SuspendedSecond          => 8,
        ChordType::SuspendedFourth          => 9,
        ChordType::SeventhSuspendedSecond   => 10,
        ChordType::SeventhSuspendedFourth   => 11,
        ChordType::Diminished               => 12,
        ChordType::Augmented                => 13,
        ChordType::Power                    => 14,
        ChordType::Unknown(value) => *value,
    }
}

/// Tonality of the chord
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ChordAlteration {
    /// Perfect.
    Perfect,
    /// Diminished.
    Diminished,
    /// Augmented.
    Augmented,
}
pub(crate) fn get_chord_alteration(value: u8) -> ChordAlteration {
    match value {
        0 => ChordAlteration::Perfect,
        1 => ChordAlteration::Diminished,
        2 => ChordAlteration::Augmented,
        _ => panic!("Cannot read chord fifth (new format)"),
    }
}
pub(crate) fn from_chord_alteration(value: &ChordAlteration) -> u8 {
    match value {
        ChordAlteration::Perfect    => 0,
        ChordAlteration::Diminished => 1,
        ChordAlteration::Augmented  => 2,
    }
}

/// Extension type of the chord
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ChordExtension {
    None,
    /// Ninth chord.
    Ninth,
    /// Eleventh chord.
    Eleventh,
    /// Thirteenth chord.
    Thirteenth,
    Unknown(u8)
}
pub(crate) fn get_chord_extension(value: u8) -> ChordExtension {
    match value {
        0 => ChordExtension::None,
        1 => ChordExtension::Ninth,
        2 => ChordExtension::Eleventh,
        3 => ChordExtension::Thirteenth,
        _ => ChordExtension::Unknown(value), //panic!("Cannot read chord type (new format)"),
    }
}
pub(crate) fn from_chord_extension(value: &ChordExtension) -> u8 {
    match value {
        ChordExtension::None              => 0,
        ChordExtension::Ninth             => 1,
        ChordExtension::Eleventh          => 2,
        ChordExtension::Thirteenth        => 3,
        ChordExtension::Unknown(value) => *value, //panic!("Cannot read chord type (new format)"),
    }
}

/// Left and right hand fingering used in tabs and chord diagram editor.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Fingering {
    /// Open or muted.
    Open, //-1?
    /// Thumb.
    Thumb,
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
        0  => Fingering::Thumb,
        1  => Fingering::Index,
        2  => Fingering::Middle,
        3  => Fingering::Annular,
        4  => Fingering::Little,
        _  => Fingering::Unknown(value), //panic!("Cannot get fingering! How can you have more than 5 fingers per hand?!?"),
    }
}
pub(crate) fn from_fingering(value: &Fingering) -> i8 {
    match value {
        Fingering::Open           => -1,
        Fingering::Thumb          => 0,
        Fingering::Index          => 1,
        Fingering::Middle         => 2,
        Fingering::Annular        => 3,
        Fingering::Little         => 4,
        Fingering::Unknown(value) => *value, //panic!("Cannot get fingering! How can you have more than 5 fingers per hand?!?"),
    }
}

/// All Bend presets
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BendType {
    /// No Preset.
    None,

    //Bends
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

    //Tremolo Bar
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
    ReleaseDown
}
pub(crate) fn get_bend_type(value: i8) -> BendType {
    match value {
        0 => BendType::None,
        1 => BendType::Bend,
        2 => BendType::BendRelease,
        3 => BendType::BendReleaseBend,
        4 => BendType::Prebend,
        5 => BendType::PrebendRelease,
        6 => BendType::Dip,
        7 => BendType::Dive,
        8 => BendType::ReleaseUp,
        9 => BendType::InvertedDip,
        10 => BendType::Return,
        11 => BendType::ReleaseDown,
        _ => panic!("Cannot read bend type"),
    }
}
pub(crate) fn from_bend_type(value: &BendType) -> i8 {
    match value {
        BendType::None             => 0,
        BendType::Bend             => 1,
        BendType::BendRelease      => 2,
        BendType::BendReleaseBend  => 3,
        BendType::Prebend          => 4,
        BendType::PrebendRelease   => 5,
        BendType::Dip              => 6,
        BendType::Dive             => 7,
        BendType::ReleaseUp        => 8,
        BendType::InvertedDip      => 9,
        BendType::Return          => 10,
        BendType::ReleaseDown     => 11,
    }
}

/// All transition types for grace notes.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum GraceEffectTransition {
    ///No transition
    None,
    ///Slide from the grace note to the real one.
    Slide,
    ///Perform a bend from the grace note to the real one.
    Bend,
    ///Perform a hammer on.
    Hammer
}
pub(crate) fn get_grace_effect_transition(value: i8) -> GraceEffectTransition {
    match value {
        0 => GraceEffectTransition::None,
        1 => GraceEffectTransition::Slide,
        2 => GraceEffectTransition::Bend,
        3 => GraceEffectTransition::Hammer,
        _ => panic!("Cannot get transition for the grace effect"),
    }
}
pub(crate) fn from_grace_effect_transition(value: &GraceEffectTransition) -> i8 {
    match value {
        GraceEffectTransition::None   => 0,
        GraceEffectTransition::Slide  => 1,
        GraceEffectTransition::Bend   => 2,
        GraceEffectTransition::Hammer => 3,
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum HarmonicType {
    Natural, //1
    Artificial,
    Tapped,
    Pinch,
    Semi, //5
}
pub(crate) fn from_harmonic_type(value: &HarmonicType) -> i8 {
    match value {
        HarmonicType::Natural       => 1,
        HarmonicType::Artificial    => 2,
        HarmonicType::Tapped        => 3,
        HarmonicType::Pinch         => 4,
        HarmonicType::Semi          => 5,
    }
}

/// Values of auto-accentuation on the beat found in track RSE settings
#[derive(Debug,Clone)]
pub enum Accentuation { None, VerySoft, Soft, Medium, Strong, VeryStrong }
pub(crate) fn get_accentuation(value: u8) -> Accentuation {
    match value {
        0 => Accentuation::None,
        1 => Accentuation::VerySoft,
        2 => Accentuation::Soft,
        3 => Accentuation::Medium,
        4 => Accentuation::Strong,
        5 => Accentuation::VeryStrong,
        _ => panic!("Cannot get accentuation"),
    }
}
pub(crate) fn from_accentuation(value: &Accentuation) -> u8 {
    match value {
        Accentuation::None         => 0,
        Accentuation::VerySoft     => 1,
        Accentuation::Soft         => 2,
        Accentuation::Medium       => 3,
        Accentuation::Strong       => 4,
        Accentuation::VeryStrong   => 5,
    }
}

/// A navigation sign like *Coda* (𝄌: U+1D10C) or *Segno* (𝄋 or 𝄉: U+1D10B or U+1D109).
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum DirectionSign {
    Coda, DoubleCoda,
    Segno, SegnoSegno,
    Fine,
    DaCapo, DaCapoAlCoda, DaCapoAlDoubleCoda, DaCapoAlFine,
    DaSegno, DaSegnoAlCoda, DaSegnoAlDoubleCoda, DaSegnoAlFine, DaSegnoSegno, DaSegnoSegnoAlCoda, DaSegnoSegnoAlDoubleCoda, DaSegnoSegnoAlFine,
    DaCoda, DaDoubleCoda,
}
