/// An enumeration of different triplet feels.
#[derive(Clone)]
pub enum TripletFeel { NONE, EIGHTH, SIXTEENTH }


/// An enumeration of available clefs
#[derive(Clone)]
pub enum MeasureClef { Treble, Bass, Tenor, Alto }
/// A line break directive: `NONE: no line break`, `BREAK: break line`, `Protect the line from breaking`.
#[derive(Clone)]
pub enum LineBreak { None, Break, Protect }

/// An enumeration of all supported slide types.
#[derive(Clone,PartialEq)]
pub enum SlideType {
    IntoFromAbove, //-2
    IntoFromBelow, //-1
    None, //0
    ShiftSlideTo,
    LegatoSlideTo,
    OutDownwards,
    OutUpWards
}

/// An enumeration of all supported slide types.
#[derive(Clone,PartialEq)]
pub enum NoteType {
    Rest, //0
    Normal, Tie, Dead,
}

#[derive(Clone,PartialEq)]
pub enum BeatStatus {Empty, Normal, Rest}

#[derive(Clone,PartialEq)]
pub enum TupletBracket {None, Start, End}

/// Octave signs
#[derive(Clone,PartialEq)]
pub enum Octave { None, Ottava, Quindicesima, Ottavabassa, Quindicesimabassa }

/// All beat stroke directions
#[derive(Clone,PartialEq)]
pub enum BeatStrokeDirection { None, Up, Down }
/// Characteristic of articulation
#[derive(Clone,PartialEq)]
pub enum SlapEffect { None, Tapping, Slapping, Popping }


/// Voice directions indicating the direction of beams
#[derive(Clone,PartialEq)]
pub enum VoiceDirection { None, Up, Down }


/// Type of the chord.
#[derive(Clone,PartialEq)]
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
}

/// Tonality of the chord
#[derive(Clone,PartialEq)]
pub enum ChordAlteration {
    /// Perfect.
    Perfect,
    /// Diminished.
    Diminished,
    /// Augmented.
    Augmented,
}

/// Extension type of the chord
#[derive(Clone,PartialEq)]
pub enum ChordExtension {
    None,
    /// Ninth chord.
    Ninth,
    /// Eleventh chord.
    Eleventh,
    /// Thirteenth chord.
    Thirteenth
}

/// Left and right hand fingering used in tabs and chord diagram editor.
#[derive(Clone,PartialEq)]
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
}

pub fn get_fingering(value: i8) -> Fingering {
    match value {
        -1 => Fingering::Open,
        0  => Fingering::Thumb,
        1  => Fingering::Index,
        2  => Fingering::Middle,
        3  => Fingering::Annular,
        4  => Fingering::Little,
        _  => panic!("Cannot get fingering! How can you have more than 5 fingers per hand?!?"),
    }
}

/// All Bend presets
#[derive(Clone,PartialEq)]
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

/// All transition types for grace notes.
#[derive(Clone,PartialEq)]
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

#[derive(Clone,PartialEq)]
pub enum HarmonicType {
    Natural, //1
    Artificial,
    Tapped,
    Pinch,
    Semi, //5
}

/// Values of auto-accentuation on the beat found in track RSE settings
#[derive(Clone)]
pub enum Accentuation { None, VerySoft, Soft, Medium, Strong, VeryStrong }

/// A navigation sign like *Coda* (𝄌: U+1D10C) or *Segno* (𝄋 or 𝄉: U+1D10B or U+1D109).
#[derive(Clone)]
pub enum DirectionSign { Coda, Segno, }