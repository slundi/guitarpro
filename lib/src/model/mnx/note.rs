use std::str::FromStr;

use crate::model::mnx::{error::MnxError, MnxId, Orientation};

pub enum AccidentalEnclosureSymbol {
    /// The accidental is enclosed in square brackets.
    Brackets,
    /// The accidental is enclosed in parentheses.
    Parentheses,
}

// pub struct AccidentalDisplay {
//     pub enclosure: Option,
//     /// Whether this accidental display value (either shown or hidden) was set intentionally, as opposed to
//     /// automatically determined.
//     ///
//     /// This allows for the encoding of intent. Some consuming software may choose to distinguish between accidentals
//     /// that are intentionally displayed and accidentals that are algorithmically displayed.
//     ///
//     /// If this value isn't provided, it's assumed to be false.
//     pub force: bool,
//     /// Whether the accidental is to be displayed. (REQUIRED)
//     pub show: bool,
// }

pub enum TieTargetType {
    /// The tie ends on a note in the event that directly follows the start note's event.
    ///
    /// This is by far the most common type of tie.
    NextNote,
    /// The tie is part of an arpeggio notated as consecutive ties
    Arpeggio,
    /// The tie ends on a note whose event does not directly follow the start note's event, such as an alternate ending, repeat or jump (e.g., D.S. al Coda).
    CrossJump,
    /// The tie ends on a note in a different sequence.
    CrossVoice,
}

impl FromStr for TieTargetType {
    type Err = MnxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nextNone" => Ok(TieTargetType::NextNote),
            "arpeggio" => Ok(TieTargetType::Arpeggio),
            "crossJump" => Ok(TieTargetType::CrossJump),
            "crossVoice" => Ok(TieTargetType::CrossVoice),
            _ => Err(MnxError::InvalidTieTarget(s.to_string())),
        }
    }
}

/// The tie object represents a single tie between two notes. A tie is only encoded on the *first* note — i.e., the
/// note that starts the tie.
///
/// In the common case of simple ties, use "target" to specify the tie's end note.
///
/// For *laissez vibrer* ties - which do not have a particular destination note - omit "target" and use "lv" instead.
pub struct Tie {
    /// Specifies that this tie is a laissez vibrer tie — which does not have a particular destination note.
    ///
    /// If this attribute is omitted, the value is assumed to be false.
    pub lv: bool,
    /// Specifies whether the tie curves upward or downward. If not provided, the consuming software should determine
    /// the value automatically according to its own logic.
    pub side: Option<Orientation>,
    /// The ID of a note that ends this tie.
    ///
    /// The start note and end note must be in the same part (i.e., it's not allowed for a tie to start in one part and
    /// end in a different part).
    ///
    /// The end note must have the same sounded pitch as the start note — but the two notes are allowed to use
    /// different note spellings, as long as those spellings are enharmonically equivalent.
    ///
    /// If "target" is omitted, "lv" is required with a value of true.
    pub target: MnxId,
    /// Specifies the relationship of the tie's start note to its target (end) note.
    ///
    /// Generally, most ties are "nextNote", meaning the tie connects notes that are in consecutive events. The other
    /// supported values — "arpeggio", "crossVoice" and "crossJump" — describe less frequent situations of music
    /// notation.
    ///
    /// Many simpler notation applications make the assumption that ties are always in consecutive events. This
    /// attribute exists as a hint for these applications, so that they can include or exclude ties accordingly. For
    /// example, a simple notation-display application may decide to ignore all ties that don't have targetType
    /// "nextNote".
    ///
    /// If this attribute isn't provided, behavior is undefined. In this case, consuming applications may decide how
    /// (or whether) to parse the tie, according to their own algorithms. For example, a simple algorithm might be:
    /// only honor this tie if the "target" note lives in the subsequent event.
    ///
    /// It is an error to use this attribute if "lv" is true.
    pub target_type: Option<TieTargetType>,
}
