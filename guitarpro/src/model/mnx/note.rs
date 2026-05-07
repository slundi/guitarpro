use std::str::FromStr;

use crate::model::mnx::{MnxId, Orientation, Pitch, error::MnxError};

/// The symbol used to visually enclose an accidental.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/accidental-enclosure-symbol/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccidentalEnclosureSymbol {
    /// The accidental is enclosed in square brackets.
    Brackets,
    /// The accidental is enclosed in parentheses.
    Parentheses,
}

/// Describes the symbol that visually encloses an accidental, such as brackets or parentheses.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/accidental-enclosure/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccidentalEnclosure {
    /// The type of enclosure symbol drawn around the accidental.
    pub symbol: AccidentalEnclosureSymbol,
}

/// Information about the displayed accidental for a note.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/accidental-display/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccidentalDisplay {
    /// Describes the symbol that visually encloses the accidental, such as square brackets
    /// or parentheses.
    pub enclosure: Option<AccidentalEnclosure>,
    /// Whether this accidental display value was set intentionally, as opposed to
    /// automatically determined.
    ///
    /// This allows encoding of intent: some consuming software may choose to distinguish
    /// between accidentals that are intentionally displayed and those that are
    /// algorithmically placed.
    ///
    /// If not provided, assumed to be false.
    pub force: Option<bool>,
    /// Whether the accidental is to be displayed.
    pub show: bool,
}

/// Options controlling whether a note is included in playback.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/perform-options/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformOptions {
    /// When true, this note is muted and should be excluded from playback.
    pub mute: Option<bool>,
}

/// An alternate pitch spelling for transposing instruments, enabling both concert-pitch
/// and written (transposed) representations in the same document.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/written/>
#[derive(Debug, Clone, PartialEq)]
pub struct Written {
    /// The written (transposed) pitch for this note.
    pub pitch: Pitch,
    /// Display information for the written pitch's accidental.
    pub accidental_display: Option<AccidentalDisplay>,
}

/// Specifies the relationship of a tie's start note to its target (end) note.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/tie-target-type/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TieTargetType {
    /// The tie ends on a note in the event that directly follows the start note's event.
    ///
    /// This is by far the most common type of tie.
    NextNote,
    /// The tie is part of an arpeggio notated as consecutive ties.
    Arpeggio,
    /// The tie ends on a note whose event does not directly follow the start note's event,
    /// such as when crossing an alternate ending, repeat, or jump (e.g., D.S. al Coda).
    CrossJump,
    /// The tie ends on a note in a different sequence (voice).
    CrossVoice,
}

impl FromStr for TieTargetType {
    type Err = MnxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nextNote" => Ok(TieTargetType::NextNote),
            "arpeggio" => Ok(TieTargetType::Arpeggio),
            "crossJump" => Ok(TieTargetType::CrossJump),
            "crossVoice" => Ok(TieTargetType::CrossVoice),
            _ => Err(MnxError::InvalidTieTarget(s.to_string())),
        }
    }
}

/// Represents a single tie between two notes. A tie is only encoded on the *first* note —
/// i.e., the note that starts the tie.
///
/// In the common case, use `target` to specify the tie's end note.
///
/// For *laissez vibrer* ties — which do not have a particular destination note — omit
/// `target` and set `lv` to true instead.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/tie/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tie {
    /// When true, this is a laissez vibrer tie with no particular destination note.
    ///
    /// If this attribute is omitted, the value is assumed to be false.
    pub lv: bool,
    /// Whether the tie curves upward or downward. If not provided, consuming software
    /// determines this automatically.
    pub side: Option<Orientation>,
    /// The ID of the note that ends this tie.
    ///
    /// The start and end notes must be in the same part. The end note must have the same
    /// sounded pitch as the start note (though enharmonic spellings are allowed).
    ///
    /// If `target` is omitted, `lv` must be true.
    pub target: Option<MnxId>,
    /// The relationship of the tie's start note to its target (end) note.
    ///
    /// Most ties are `NextNote` (consecutive events). The other values — `Arpeggio`,
    /// `CrossVoice`, and `CrossJump` — describe less common situations.
    ///
    /// Simpler applications may use this as a hint to include or exclude ties. For
    /// example, a display-only application might ignore ties that aren't `NextNote`.
    ///
    /// It is an error to use this attribute when `lv` is true.
    pub target_type: Option<TieTargetType>,
}

/// A single note within a chord event.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/note/>
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// The note's sounded pitch.
    pub pitch: Pitch,
    /// Display information for this note's accidental.
    pub accidental_display: Option<AccidentalDisplay>,
    /// Options controlling whether consuming software plays back this note.
    pub perform: Option<PerformOptions>,
    /// The staff index this note belongs to. Used to override cross-staff notation
    /// (e.g., notes that belong to a different staff than the containing sequence).
    pub staff: Option<u8>,
    /// Ties originating from this note. An array is used because a note may tie to
    /// multiple notes (e.g., in ossia staves).
    pub ties: Option<Vec<Tie>>,
    /// Alternate pitch spelling for transposing instruments.
    pub written: Option<Written>,
    /// Unique identifier for this note, referenced by ties and slurs.
    pub id: Option<MnxId>,
}
