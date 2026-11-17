//! Score-level metadata: title, artist, tempo, key/time signatures, chord symbols,
//! work identification, credits, and encoding provenance.

use serde::{Deserialize, Serialize};

use crate::model::optimized::note::{Pitch, PitchStep};

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

impl TimeSignature {
    /// A measure's length in ticks at `quarter` ticks a quarter: multiplied
    /// out before dividing, so a measure a beat isn't a whole number of
    /// ticks of (as 6/8 at 3 a quarter) keeps its length. `None` for a
    /// zero denominator, or a resolution too large for a measure's ticks
    /// to fit a u64.
    pub fn measure_ticks(self, quarter: u64) -> Option<u64> {
        quarter
            .checked_mul(4 * u64::from(self.numerator))?
            .checked_div(u64::from(self.denominator))
    }
}

/// A key: its tonic and mode. `root` is the mode's own tonic (D for D
/// dorian, which has no sharps or flats), in octave 4.
///
/// Serialized as `tonic`: scores saved before it was the mode's own tonic
/// have a `root`, the major tonic of the signature for the church modes
/// (C for D dorian), and are read as such.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct KeySignature {
    pub root: Pitch,
    pub mode: Mode,
}

/// [`KeySignature`] in a binary format (as `.msor`'s Postcard), which
/// writes every field in order and so can't skip one: just its tonic.
/// Being positional, it can't tell an older save's `root` (for the church
/// modes the major tonic) from it. A binary score from before is refused
/// by its format version (see
/// [`SCORE_FORMAT_VERSION`](crate::model::optimized::global::SCORE_FORMAT_VERSION)),
/// so this only reads keys in the current form; a key read on its own from
/// an older one would be taken as it stands (D dorian saved as C dorian).
/// A self-describing format (JSON) is migrated.
#[derive(Serialize, Deserialize)]
struct BinaryKey {
    tonic: Pitch,
    mode: Mode,
}

impl Serialize for KeySignature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            SerializedKey::from(*self).serialize(serializer)
        } else {
            BinaryKey {
                tonic: self.root,
                mode: self.mode,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for KeySignature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            KeySignature::try_from(SerializedKey::deserialize(deserializer)?)
                .map_err(serde::de::Error::custom)
        } else {
            let BinaryKey { tonic, mode } = BinaryKey::deserialize(deserializer)?;
            Ok(KeySignature { root: tonic, mode })
        }
    }
}

/// [`KeySignature`] as serialized in a human-readable format, by either of
/// its tonics.
#[derive(Serialize, Deserialize, Clone, Copy)]
struct SerializedKey {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tonic: Option<Pitch>,
    /// The tonic as scores saved before `tonic` give it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    root: Option<Pitch>,
    mode: Mode,
}

impl From<KeySignature> for SerializedKey {
    fn from(key: KeySignature) -> Self {
        SerializedKey {
            tonic: Some(key.root),
            root: None,
            mode: key.mode,
        }
    }
}

impl TryFrom<SerializedKey> for KeySignature {
    type Error = &'static str;

    fn try_from(key: SerializedKey) -> Result<Self, Self::Error> {
        match (key.tonic, key.root) {
            (Some(tonic), _) => Ok(KeySignature {
                root: tonic,
                mode: key.mode,
            }),
            // Major and minor keys had their own tonic as `root` already.
            (None, Some(root)) if matches!(key.mode, Mode::Major | Mode::Minor) => {
                Ok(KeySignature {
                    root,
                    mode: key.mode,
                })
            }
            (None, Some(major)) => Ok(KeySignature::from_fifths(
                normalized_fifths(major.fifths()) as i8,
                key.mode,
            )),
            (None, None) => Err("key signature without a tonic"),
        }
    }
}

impl KeySignature {
    /// The key of a signature with `fifths` (sharps positive, flats
    /// negative) in `mode`. Beyond seven sharps or flats, its enharmonic
    /// key within them ([`normalized_fifths`]).
    pub fn from_fifths(fifths: i8, mode: Mode) -> Self {
        let fifths = normalized_fifths(fifths.into());
        // Within seven sharps or flats, the tonic is at most a sharp/flat.
        let root = Pitch::from_fifths(fifths + mode.tonic_fifths()).unwrap_or(Pitch {
            step: PitchStep::C,
            alter: 0,
            octave: 4,
        });
        KeySignature { root, mode }
    }

    /// The signature's signed fifth count, enharmonically brought within
    /// seven sharps or flats (G♯ major → A♭ major, four flats). Inverse of
    /// [`KeySignature::from_fifths`].
    pub fn fifths(&self) -> i8 {
        normalized_fifths(self.root.fifths() - self.mode.tonic_fifths()) as i8
    }
}

/// A signed fifth count (key signature) enharmonically brought within
/// seven sharps or flats: eight sharps are four flats.
pub fn normalized_fifths(mut fifths: i32) -> i32 {
    while fifths > 7 {
        fifths -= 12;
    }
    while fifths < -7 {
        fifths += 12;
    }
    fifths
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

impl Mode {
    /// The mode's name, as MusicXML's and MuseScore's `<mode>` write it.
    pub fn name(self) -> &'static str {
        match self {
            Mode::Major => "major",
            Mode::Minor => "minor",
            Mode::Dorian => "dorian",
            Mode::Phrygian => "phrygian",
            Mode::Lydian => "lydian",
            Mode::Mixolydian => "mixolydian",
            Mode::Locrian => "locrian",
        }
    }

    /// Inverse of [`Mode::name`]; `ionian` and `aeolian` read as major and
    /// minor, anything else (`none`, `unknown`...) as no mode.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "major" | "ionian" => Mode::Major,
            "minor" | "aeolian" => Mode::Minor,
            "dorian" => Mode::Dorian,
            "phrygian" => Mode::Phrygian,
            "lydian" => Mode::Lydian,
            "mixolydian" => Mode::Mixolydian,
            "locrian" => Mode::Locrian,
            _ => return None,
        })
    }

    /// Where the mode's tonic lies on the line of fifths, relative to the
    /// major tonic of its key signature (A minor: three fifths above C).
    fn tonic_fifths(self) -> i32 {
        match self {
            Mode::Lydian => -1,
            Mode::Major => 0,
            Mode::Mixolydian => 1,
            Mode::Dorian => 2,
            Mode::Minor => 3,
            Mode::Phrygian => 4,
            Mode::Locrian => 5,
        }
    }
}

/// An added, altered, or omitted chord degree (e.g. `add9`, `♭5`, `omit3`).
#[derive(Serialize, Deserialize, Debug)]
pub struct ChordDegree {
    /// Scale degree number 1–13.
    pub value: u8,
    /// Semitone alteration (negative = flat, positive = sharp).
    pub alter: f32,
    pub kind: DegreeKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum DegreeKind {
    /// Add a degree not implied by the chord kind (e.g. `add9`).
    Add,
    /// Modify an implied degree (e.g. `♭5`).
    Alter,
    /// Remove an implied degree (e.g. `omit3`).
    Subtract,
}

/// One note in a fretboard chord diagram.
#[derive(Serialize, Deserialize, Debug)]
pub struct FrameNote {
    /// String number, 1 = highest-pitched string.
    pub string: u8,
    pub fret: u8,
    /// Fingering digit 1–5 (index–pinky), or 0 for thumb.
    pub fingering: Option<u8>,
    pub barre: Option<BarreKind>,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum BarreKind {
    Start,
    Stop,
}

/// A fretboard chord diagram (guitar chord box).
#[derive(Serialize, Deserialize, Debug)]
pub struct ChordFrame {
    /// Total number of strings shown.
    pub strings: u8,
    /// Number of frets shown in the diagram.
    pub frets: u8,
    /// First fret displayed (default 1).
    pub first_fret: Option<u8>,
    pub notes: Vec<FrameNote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChordSymbol {
    /// `None` for "no chord" (N.C.).
    pub root: Option<Pitch>,
    pub kind: String,        // e.g. "maj7", "m", "7", "sus4"
    pub bass: Option<Pitch>, // slash chord bass note
    /// Inversion: 0 = root position, 1 = first inversion, etc.
    pub inversion: Option<u8>,
    /// Added, altered, or omitted scale degrees.
    pub degrees: Vec<ChordDegree>,
    /// Optional fretboard chord diagram.
    pub frame: Option<ChordFrame>,
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
