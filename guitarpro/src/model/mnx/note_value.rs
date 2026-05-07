/// The base note type for a note value, such as 'quarter'.
///
/// These values correspond to standard musical note durations and their historical
/// subdivisions, from the longest (duplexMaxima) to the shortest (4096th).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/note-value-base/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteValueBase {
    /// Quadruple whole note (historically known as duplex maxima or octuple whole note).
    DuplexMaxima,
    /// Triple whole note (historically known as maxima).
    Maxima,
    /// Long note (longa), equal to four half notes.
    Longa,
    /// Double whole note (breve), equal to two whole notes.
    Breve,
    /// Whole note (semibreve).
    Whole,
    /// Half note (minim).
    Half,
    /// Quarter note (crotchet).
    Quarter,
    /// Eighth note (quaver).
    Eighth,
    /// 16th note (semiquaver).
    Sixteenth,
    /// 32nd note (demisemiquaver).
    ThirtySecond,
    /// 64th note (hemidemisemiquaver).
    SixtyFourth,
    /// 128th note.
    OneHundredTwentyEighth,
    /// 256th note.
    TwoHundredFiftySixth,
    /// 512th note.
    FiveHundredTwelfth,
    /// 1024th note.
    OneThousandTwentyFourth,
    /// 2048th note.
    TwoThousandAndFortyEighth,
    /// 4096th note.
    FourThousandAndNinetySixth,
}

/// A note value, consisting of a base note type and optional augmentation dots.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/note-value/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteValue {
    /// Base note type, such as 'quarter'.
    pub base: NoteValueBase,
    /// Number of augmentation dots. If not specified, this value is assumed to be 0.
    pub dots: Option<u32>,
}

/// An integer multiple of a note value, used to express tuplet ratios.
///
/// For example, "three quarter notes" would be `{ duration: quarter, multiple: 3 }`.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/note-value-quantity/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteValueQuantity {
    /// The base note value to be multiplied.
    pub duration: NoteValue,
    /// The integer multiple.
    pub multiple: u32,
}

/// A rhythmic position within a measure, expressed as a fraction of the measure.
///
/// For example, 'the position a quarter note's duration into a bar' would be encoded as
/// `fraction: [1, 4]`.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/rhythmic-position/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RhythmicPosition {
    /// The rhythmic position as a fraction `[numerator, denominator]`.
    /// For example, `[1, 4]` means "one quarter note into the measure".
    pub fraction: [u32; 2],
    /// Distinguishes grace notes from the target note at the same rhythmic position.
    /// Counts backward from the target note (index 0). When omitted, the position is
    /// interpreted as occurring before all grace notes at that beat.
    pub grace_index: Option<u32>,
}

/// A rhythmic position within a specific measure of the score.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/measure-rhythmic-position/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureRhythmicPosition {
    /// The 0-based index of the measure within the global measures array.
    pub measure: u32,
    /// The rhythmic position within that measure.
    pub position: RhythmicPosition,
}
