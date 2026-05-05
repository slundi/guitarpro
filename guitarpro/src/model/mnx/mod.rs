pub mod error;
pub mod note;

use std::convert::TryFrom;

use crate::model::mnx::error::{MnxError, MnxIdError};

/// MNX requires that IDs:
///
/// * Are between 1 and 256 characters long (inclusive)
/// * Consist only of ASCII characters (trgex `^[\x21-\x7E]{1,256}$`)
/// * Do not contain spaces or nonprintable characters
// #[serde(try_from = "String")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MnxId(String);

impl MnxId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MnxId {
    type Error = MnxIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(MnxIdError::Empty);
        }
        if value.len() > 256 {
            return Err(MnxIdError::TooLong);
        }
        // Check for ASCII and no spaces
        if !value.is_ascii() || value.chars().any(|c| c.is_whitespace()) {
            return Err(MnxIdError::InvalidCharacters);
        }

        Ok(MnxId(value))
    }
}

// Implement Deref so you can use it like a string easily
impl std::ops::Deref for MnxId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A MIDI pitch — an unsigned number between 0-127, where middle C is 60.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MidiNumber(u8);

impl MidiNumber {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 127;

    pub fn new(value: u8) -> Result<Self, MnxError> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(MnxError::InvalidMidiNumber(value))
        }
    }

    pub fn get(&self) -> u8 {
        self.0
    }
}

/// This contains information about how to interpret the data within an MNX document.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Support {
    /// This answers the question "Does every note with a visible accidental have accidentalDisplay set?"
    ///
    /// It's meant as a heads-up for consuming software that doesn't have its own accidental-display algorithm. For
    /// this type of software, when reading a document with useAccidentalDisplay=false, the software might opt to
    /// display a warning to users, saying "We don't know which notes should have an accidental displayed." Or the
    /// software might opt to reject the document outright due to not containing sufficient information.
    ///
    /// If this value is not provided, it is to be interpreted as false.
    pub use_accidental_display: bool,
    /// This answers the question "Are beams encoded in this document?"
    ///
    /// If the value is true, any MNX event that is not encoded within a beam should be treated as unbeamed.
    ///
    /// If the value is false (or not provided), consuming software is expected to use its own beaming algorithm to
    /// determine beams.
    pub use_beams: bool,
}

/// Holds technical MNX metadata about the document, including the MNX version.
pub struct Metadata {
    /// Information about how to interpret the data in this MNX file, in cases of ambiguity.
    pub support: Option<Support>,
    /// The MNX version number, as an integer.
    ///
    /// MNX uses simple integers for version numbers, as opposed to strings or multi-part version numbers ("3.1"). Each
    /// release of MNX increments the version number.
    ///
    /// Because MNX aims to be backward compatible — i.e., an older MNX version should always be openable by future
    /// software — version numbers are mostly useful for determining which new MNX features are available.
    pub version: u16,
}

/// Represents a symbol's vertical orientation.
///
/// When unspecified: consuming applications are free to use their own algorithms to determine orientation.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(rename_all = "lowercase")]
pub enum Orientation {
    Up,
    Down,
}

/// The global object represents global notation data, which is generally shared by a set of parts within the score.
pub struct Global {
    // pub lyrics: Option<>,
    // /// An array of measure global objects
    // pub measures: Vec<>,
    // /// An object with user-defined keys, where each value is a sound
    // pub sounds: Option<>,
}

// // https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/lyrics-global/
// // Dictionary
// pub struct LyricsGlobal {
//     /// An object mapping lyric line IDs (which are user-defined) to lyric metadata objects. This encodes global
//     /// metadata on a per-lyric-line basis.
//     pub line_metadata: Option<LyricLineMetadata>,
//     /// An array of all lyric line IDs used in this MNX document, in the order in which they should be visually
//     /// displayed (from top to bottom).
//     pub line_order: Option<String>,
// }

pub struct Pitch {
    /// Describes the alteration to a note's pitch, as an integer in the range -3 to 3, inclusive.
    pub alter: Option<i8>,
    /// A number representing an octave in Scientific Pitch Notation. The octave for middle C is 4.
    pub octave: u8,
    /// Allowed values: A, B, C, D, E, F, G
    pub step: char,
}
