pub mod error;
pub mod event;
pub mod global;
pub mod layout;
pub mod lyrics;
pub mod measure;
pub mod note;
pub mod note_value;
pub mod part;
pub mod score;
pub mod sequence;

use std::convert::TryFrom;

use crate::model::mnx::error::{MnxError, MnxIdError};

/// An MNX identifier string.
///
/// MNX requires that IDs:
/// * Are between 1 and 256 characters long (inclusive)
/// * Consist only of printable ASCII characters (regex `^[\x21-\x7E]{1,256}$`)
/// * Do not contain spaces or non-printable characters
///
/// IDs are used to cross-reference elements such as tied notes, slur targets, and beamed
/// events.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/id/>
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
        if !value.is_ascii() || value.chars().any(|c| c.is_whitespace()) {
            return Err(MnxIdError::InvalidCharacters);
        }
        Ok(MnxId(value))
    }
}

impl std::ops::Deref for MnxId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A MIDI pitch number — an integer between 0 and 127, where middle C is 60.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/midi-number/>
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

/// Represents a symbol's vertical orientation.
///
/// When unspecified, consuming applications are free to use their own algorithms to
/// determine orientation.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/orientation/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Up,
    Down,
}

/// A note's sounded pitch in Scientific Pitch Notation.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/pitch/>
#[derive(Debug, Clone, PartialEq)]
pub struct Pitch {
    /// Alteration of the note's pitch as an integer in the range -3 to 3 (inclusive).
    /// Negative values are flat, positive values are sharp.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/alter/>
    pub alter: Option<i8>,
    /// The octave in Scientific Pitch Notation. Middle C (C4) is in octave 4.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/octave/>
    pub octave: u8,
    /// The pitch class letter. Allowed values: A, B, C, D, E, F, G.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/step/>
    pub step: char,
}

/// Information about how to interpret ambiguous data in an MNX document.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/support/>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Support {
    /// Whether every note with a visible accidental has `accidental_display` set.
    ///
    /// When false (or absent), consuming software without its own accidental-display
    /// algorithm may warn the user or reject the document outright.
    pub use_accidental_display: bool,
    /// Whether beams are explicitly encoded in this document.
    ///
    /// When true, any event not inside a beam should be treated as unbeamed. When false
    /// (or absent), consuming software should apply its own beaming algorithm.
    pub use_beams: bool,
}

/// Metadata about the MNX implementation used to produce a document, including its
/// version number.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/mnx/>
#[derive(Debug, Clone)]
pub struct MnxMetadata {
    /// Information about how to interpret the data in this MNX file.
    pub support: Option<Support>,
    /// The MNX version number as a simple integer.
    ///
    /// MNX uses monotonically increasing integers (not multi-part version strings like
    /// "3.1"). Because MNX aims to be backward-compatible, the version number is mainly
    /// useful for determining which features are available.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/version-number/>
    pub version: u16,
}

/// The root MNX document object — the top-level container for an entire MNX file.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/root/>
#[derive(Debug, Clone)]
pub struct MnxDocument {
    /// Metadata about the MNX implementation used (version, support flags).
    pub mnx: MnxMetadata,
    /// Global notation data shared across all parts (time signatures, key signatures,
    /// tempos, barlines, etc.).
    pub global: global::Global,
    /// The musical parts in the composition.
    pub parts: Vec<part::Part>,
    /// System layout configurations, referenced by ID from scores.
    pub layouts: Option<Vec<layout::SystemLayout>>,
    /// Score definitions, each representing a distinct view of the music (e.g., full score,
    /// individual part).
    pub scores: Option<Vec<score::Score>>,
}
