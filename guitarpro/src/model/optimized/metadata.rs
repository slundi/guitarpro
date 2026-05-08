//! Score-level metadata: title, artist, tempo, key/time signatures, chord symbols,
//! work identification, credits, and encoding provenance.

use serde::{Deserialize, Serialize};

use crate::model::optimized::note::Pitch;

// ---------------------------------------------------------------------------
// Work / opus identification
// ---------------------------------------------------------------------------

/// Work (opus) identification — the title of the larger collection this movement belongs to.
///
/// Maps to MusicXML `<work>`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Work {
    /// Opus number or catalogue number (e.g. `"Op. 27 No. 2"`, `"BWV 1004"`).
    pub number: Option<String>,
    /// Title of the larger work (e.g. `"Moonlight Sonata"`).
    pub title: Option<String>,
    /// Path or URI to an opus/collection file.
    pub opus: Option<String>,
}

// ---------------------------------------------------------------------------
// Identification / provenance
// ---------------------------------------------------------------------------

/// A person or organization who contributed to the score.
#[derive(Serialize, Deserialize, Debug)]
pub struct Creator {
    /// Role of the contributor: `"composer"`, `"lyricist"`, `"arranger"`,
    /// `"transcriber"`, `"editor"`, etc.
    pub role: String,
    pub name: String,
}

/// Encoding provenance: who created the file, with which software, and when.
#[derive(Serialize, Deserialize, Debug)]
pub struct Identification {
    pub creators: Vec<Creator>,
    /// Copyright or rights statement.
    pub rights: Option<String>,
    /// Software that produced the file (e.g. `"Guitar Pro 8"`, `"MuseScore 4"`).
    pub encoding_software: Option<String>,
    /// ISO 8601 date of encoding (e.g. `"2024-03-15"`).
    pub encoding_date: Option<String>,
    /// Origin of the source material (URL, book reference, etc.).
    pub source: Option<String>,
    /// Free-form key/value pairs for application-specific metadata.
    pub miscellaneous: Vec<(String, String)>,
}

// ---------------------------------------------------------------------------
// Page credits
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TextJustify {
    Left,
    Center,
    Right,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TextValign {
    Top,
    Middle,
    Bottom,
}

/// A text block placed on a score page (title, subtitle, composer name, copyright, etc.).
///
/// Maps to MusicXML `<credit>`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Credit {
    /// Semantic role: `"title"`, `"subtitle"`, `"composer"`, `"lyricist"`,
    /// `"arranger"`, `"rights"`, `"part-name"`, etc.
    pub credit_type: Option<String>,
    pub text: String,
    /// Horizontal position in tenths from the left edge of the page.
    pub position_x: Option<f32>,
    /// Vertical position in tenths from the bottom edge of the page.
    pub position_y: Option<f32>,
    pub font_size: Option<f32>,
    pub justify: Option<TextJustify>,
    pub valign: Option<TextValign>,
    /// 1-based page number (defaults to 1).
    pub page: Option<u16>,
}

// ---------------------------------------------------------------------------
// Musical signature types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8, // 4 = quarter note, 8 = eighth note, ...
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct KeySignature {
    pub root: Pitch,
    pub mode: Mode,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChordSymbol {
    pub root: Pitch,
    pub kind: String,        // e.g. "maj7", "m", "7", "sus4"
    pub bass: Option<Pitch>, // slash chord bass note
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Scale {
    pub root: Pitch,
    pub name: String, // e.g. "pentatonic minor", "blues", "major"
}

// ---------------------------------------------------------------------------
// Metadata (top-level score header)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    // --- work / movement ---
    /// Opus/collection this movement belongs to.
    pub work: Option<Work>,
    /// Movement number within the work (e.g. `"II"`, `"3"`).
    pub movement_number: Option<String>,
    /// Movement title (the primary displayed title of this score).
    pub title: String,

    // --- common fields ---
    pub artist: Option<String>,
    pub album: Option<String>,
    pub composer: Option<String>,
    pub year: Option<u16>,
    pub copyright: Option<String>,

    // --- provenance ---
    pub identification: Option<Identification>,
    /// Page-level text blocks (title page, composer name, copyright notice, etc.).
    pub credits: Vec<Credit>,

    // --- musical defaults ---
    pub master_tempo: f32, // BPM
    pub time_signature: TimeSignature,
    pub key_signature: KeySignature,

    // --- derived / computed fields ---
    pub chords: Vec<ChordSymbol>,
    pub scale_hint: Option<Scale>,
}
