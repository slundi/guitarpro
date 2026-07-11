//! MSCX (MuseScore XML) high-level AST.
//!
//! Only the elements that Part 1 of the MSCZ roadmap cares about are strongly
//! typed: envelope version, meta tags, part / staff / instrument summary and
//! per-staff measure counts. Everything else lives in `raw_xml`, which stays
//! the source-of-truth for byte-identical round-trips.
//!
//! As converters land in Part 2, more of this structure will be populated
//! directly from the XML and eventually the raw-XML field can be dropped.

/// The parsed high-level view of a `score.mscx` file.
///
/// `PartialEq` only — the body carries an `f32` tempo, which is not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct Mscx {
    /// The full MSCX XML content, preserved verbatim.
    ///
    /// This is what `write_mscx` reserializes. Structural mutations should
    /// be performed by higher-level converters that regenerate this string.
    pub raw_xml: String,

    /// `<museScore version="X.Y">` attribute, e.g. `"4.10"`.
    pub version: String,

    /// `<programVersion>` element, e.g. `"4.1.1"`.
    pub program_version: Option<String>,

    /// `<programRevision>` element (git commit hash).
    pub program_revision: Option<String>,

    /// `<Division>` — MIDI-style ticks per quarter note.
    pub division: Option<u32>,

    /// All `<metaTag name="…">value</metaTag>` entries, in order.
    pub meta_tags: Vec<MetaTag>,

    /// One entry per top-level `<Part>` element under `<Score>`.
    pub parts: Vec<Part>,

    /// Per-`<Staff id="…">` measure count, extracted from
    /// `<Score>/<Staff id="…">` blocks. Vec index corresponds to the order
    /// staves appeared. Empty when the parser could not find any staff.
    pub measure_counts: Vec<StaffMeasureCount>,

    /// Parsed master `<Staff>` blocks under `<Score>` — the note-carrying
    /// staves aligned with `<Part>` entries. Populated by Part 2's content
    /// parser; may be empty for archives parsed by Part 1 only.
    ///
    /// Vec index and `staff_id` mirror [`measure_counts`](Self::measure_counts).
    pub score_staves: Vec<MscxStaff>,
}

impl Mscx {
    /// Retrieve a meta tag by name (case-sensitive), returning its value.
    pub fn meta(&self, name: &str) -> Option<&str> {
        self.meta_tags
            .iter()
            .find(|tag| tag.name == name)
            .map(|tag| tag.value.as_str())
    }
}

/// One `<metaTag name="…">value</metaTag>` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaTag {
    pub name: String,
    pub value: String,
}

/// A `<Part id="…">` block under `<Score>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    /// `id` attribute (numeric string in MuseScore 4.x).
    pub id: String,
    /// `<trackName>` element on the part, if present.
    pub track_name: Option<String>,
    /// One entry per `<Staff id="…">` under the part.
    pub staves: Vec<Staff>,
    /// The `<Instrument>` block for this part, if present.
    pub instrument: Option<Instrument>,
}

/// A `<Staff id="…">` block under `<Part>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staff {
    /// `id` attribute.
    pub id: String,
    /// `<StaffType group="…">` attribute (e.g. `pitched`, `tablature`).
    pub group: Option<String>,
    /// `<StaffType>/<name>` (e.g. `stdNormal`, `tab6StrCommon`).
    pub type_name: Option<String>,
    /// `<defaultClef>` value (e.g. `G8vb`).
    pub default_clef: Option<String>,
}

/// A `<Part>/<Instrument>` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    /// `id` attribute on the `<Instrument>` element.
    pub id: String,
    /// `<longName>`.
    pub long_name: Option<String>,
    /// `<shortName>`.
    pub short_name: Option<String>,
    /// `<trackName>` inside `<Instrument>` (often empty).
    pub track_name: Option<String>,
    /// `<transposeDiatonic>`.
    pub transpose_diatonic: Option<i8>,
    /// `<transposeChromatic>`.
    pub transpose_chromatic: Option<i8>,
    /// `<instrumentId>` (e.g. `pluck.guitar.electric`).
    pub instrument_id: Option<String>,
    /// `<StringData>` block, if the instrument declares one.
    pub string_data: Option<StringData>,
}

/// A `<StringData>` block on an instrument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringData {
    /// `<frets>` value.
    pub frets: Option<u8>,
    /// One MIDI note per `<string>` child, in file order (low string first).
    pub strings: Vec<u8>,
}

/// Per-staff measure count captured from `<Score>` streaming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffMeasureCount {
    /// `id` attribute on the `<Staff>` element under `<Score>`.
    pub staff_id: String,
    /// Number of `<Measure>` children encountered.
    pub measure_count: u32,
}

// ---------------------------------------------------------------------------
// Score body content
// ---------------------------------------------------------------------------
//
// Part 2 additions. These types capture measure content (time/key/tempo
// changes plus voices with beats and notes) sufficient to drive a
// `LoadedScore` conversion. Elements not covered by these types are counted
// in [`crate::convert::mscz::LossReport`] and preserved via `raw_xml`.

/// A parsed top-level `<Score>/<Staff id="…">` block: the note-carrying
/// staff aligned with a `<Part>` entry.
#[derive(Debug, Clone, PartialEq)]
pub struct MscxStaff {
    /// `id` attribute on the score-level `<Staff>` element.
    pub staff_id: String,
    /// One entry per `<Measure>` child, in order.
    pub measures: Vec<MscxMeasure>,
}

/// A single `<Measure>` under a score-level `<Staff>`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MscxMeasure {
    /// `len` attribute on `<Measure>` (irregular / pickup bars only).
    pub len: Option<String>,
    /// Time signature declared inside this measure (`<TimeSig>` in the
    /// first voice); `None` = inherit from previous measure.
    pub time_sig: Option<MscxTimeSig>,
    /// Key signature declared inside this measure (`<KeySig>` in the first
    /// voice); `None` = inherit from previous measure.
    pub key_sig: Option<MscxKeySig>,
    /// Tempo change declared in this measure (`<Tempo>` in the first voice);
    /// `None` = inherit.
    ///
    /// The value is in beats-per-second (matching MuseScore's convention);
    /// multiply by 60 to get BPM.
    pub tempo_bps: Option<f32>,
    /// `<startRepeat/>` seen on this measure.
    pub start_repeat: bool,
    /// `<endRepeat>N</endRepeat>` seen on this measure. Value is the total
    /// pass count (default 2).
    pub end_repeat: Option<u8>,
    /// Voices declared inside the measure, in file order.
    pub voices: Vec<MscxVoice>,
}

/// A `<TimeSig>` element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MscxTimeSig {
    pub numerator: u8,
    pub denominator: u8,
}

/// A `<KeySig>` element (fifths only — MuseScore stores concert-key
/// signatures as a signed number of fifths, matching MusicXML).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MscxKeySig {
    pub fifths: i8,
}

/// A `<voice>` block under a `<Measure>`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MscxVoice {
    /// Beats in file order (Chord or Rest events).
    pub beats: Vec<MscxBeat>,
}

/// A single rhythmic event (Chord or Rest) inside a voice.
#[derive(Debug, Clone, PartialEq)]
pub struct MscxBeat {
    pub duration: MscxDuration,
    pub kind: MscxBeatKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MscxBeatKind {
    /// A rest — no notes.
    Rest,
    /// A chord (one or more simultaneous notes sharing the beat's duration).
    Chord(Vec<MscxNote>),
}

/// Note duration: base value (`whole`, `half`, `quarter`, `eighth`,
/// `16th`, `32nd`, `64th`, `128th`, `measure`) + augmentation dots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MscxDuration {
    pub kind: MscxDurationKind,
    pub dots: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MscxDurationKind {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    HundredTwentyEighth,
    /// `<durationType>measure</durationType>` — a whole-measure rest whose
    /// duration is inferred from the time signature.
    Measure,
}

/// A `<Note>` under a `<Chord>`.
#[derive(Debug, Clone, PartialEq)]
pub struct MscxNote {
    /// `<pitch>` MIDI value (0–127). MuseScore always writes this.
    pub pitch: Option<u8>,
    /// `<tpc>` — MuseScore tonal pitch class (used for enharmonic spelling).
    /// Range roughly [-1, 33]; can be omitted.
    pub tpc: Option<i8>,
    /// `<string>` — 0-based string number for tab notes (top string = 0).
    pub string: Option<u8>,
    /// `<fret>` — fret number for tab notes.
    pub fret: Option<u8>,
    /// `<Spanner type="Tie">` presence — set when this note starts a tie.
    pub tie_start: bool,
    /// This note ends a tie initiated by a previous note.
    pub tie_end: bool,
}
