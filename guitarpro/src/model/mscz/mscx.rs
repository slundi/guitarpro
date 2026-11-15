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
    /// `<Channel>/<program value="…"/>` of the first channel (GM program,
    /// 0-based).
    pub program: Option<u8>,
    /// `<StringData>` block, if the instrument declares one.
    pub string_data: Option<StringData>,
    /// `<clef>`s by the part's staff they're for (0-based; the `staff`
    /// attribute counts from 1): the clef of a staff without a
    /// `<defaultClef>`.
    pub clefs: Vec<(usize, String)>,
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
    /// Whether the `<endRepeat>` count didn't fit `end_repeat`: 0 (kept as
    /// 1), one too large (kept as 255), or not a number (kept as 2).
    pub end_repeat_lossy: bool,
    /// `<Clef>` in the first voice before any Chord/Rest (its
    /// `<concertClefType>`, e.g. `F8vb`): the clef at the measure's start.
    pub start_clef: Option<String>,
    /// `<Clef>` types in the first voice before any Chord/Rest followed by
    /// another there: they hold for no beat, `start_clef` the last.
    pub superseded_clefs: Vec<String>,
    /// `<Clef>` in the first voice after its last Chord/Rest — where
    /// MuseScore puts a clef change taking effect at the next measure.
    pub end_clef: Option<String>,
    /// `<Clef>` types in the first voice between two Chords/Rests, in
    /// order: changes in mid-measure, which the optimized model can only
    /// hold from the next measure on.
    pub mid_clefs: Vec<String>,
    /// `<Jump>`s and `<Marker>`s, in file order. They're system-wide: only
    /// the first staff's count.
    pub marks: Vec<MscxMeasureMark>,
    /// Voices declared inside the measure, in file order.
    pub voices: Vec<MscxVoice>,
}

/// A navigation mark directly under a `<Measure>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MscxMeasureMark {
    Jump(MscxJump),
    Marker(MscxMarker),
}

/// A `<Jump>` (D.C., D.S. ...): where it jumps to, where playback then
/// stops and where it continues, by marker label (`start`/`end` for the
/// score's ends).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MscxJump {
    /// `<text>`, its markup's text joined.
    pub text: String,
    pub jump_to: String,
    pub play_until: String,
    pub continue_at: String,
}

/// A `<Marker>` (segno, coda, fine, to coda): `label` is what jumps refer
/// to it by.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MscxMarker {
    pub label: String,
    /// `<text>`, its markup's text joined (`<sym>segno</sym>` reads `segno`).
    pub text: String,
}

/// A `<TimeSig>` element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MscxTimeSig {
    pub numerator: u8,
    pub denominator: u8,
}

/// A `<KeySig>` element. MuseScore stores signatures as a signed number of
/// fifths, matching MusicXML. A transposing staff's written key
/// (`<actualKey>`) is not read: it follows from the concert key and the
/// instrument's transposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MscxKeySig {
    /// The concert (sounding) key: `<concertKey>`, or `<accidental>` in
    /// files without one.
    pub fifths: i8,
    /// `<mode>`; `None` when absent or unknown (MuseScore then shows the
    /// major key).
    pub mode: Option<MscxKeyMode>,
}

/// A key signature's `<mode>`: MuseScore names modes as MusicXML does
/// (see [`MscxKeyMode::name`] and [`MscxKeyMode::from_name`]).
pub use crate::model::optimized::metadata::Mode as MscxKeyMode;

/// A `<voice>` block under a `<Measure>`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MscxVoice {
    /// Beats in file order (Chord or Rest events).
    pub beats: Vec<MscxBeat>,
    /// `<Spanner type="Volta">` start and end elements (voltas are
    /// system-wide: only the first staff's count).
    pub voltas: Vec<MscxVoltaMark>,
    /// The `<BarLine>` after the voice's last beat: the measure's end
    /// barline (MuseScore writes none for a regular one).
    pub end_barline: Option<MscxBarLine>,
    /// Breath marks before the voice's first beat. Like
    /// [`MscxBeat::breaths`], they follow the previous beat in this voice:
    /// the last one of an earlier measure.
    pub leading_breaths: Vec<String>,
    /// Hairpin end elements before the voice's first beat, ending on the
    /// previous beat in this voice (see [`MscxBeat::hairpin_ends`]).
    pub leading_hairpin_ends: u32,
    /// Segment elements after the voice's last beat, with no beat to
    /// belong to.
    pub trailing: MscxSegment,
    /// Segment elements right before a voice-level `<location>`: they sit
    /// at the gap's start, where this voice has no beat (the next beat is
    /// after the gap), so none holds them.
    pub before_gaps: Vec<MscxSegment>,
}

/// A volta's start (with its `<Volta>`) or end element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MscxVoltaMark {
    /// `None` for an end element (`<prev>`).
    pub start: Option<MscxVolta>,
    /// Beats in the voice before the element: an end before any beat
    /// closes the volta at the previous measure's end.
    pub beats_before: usize,
    /// `<measures>` of its `<next>`/`<prev>` location: how many measures
    /// away the other end is (negative for an end).
    pub measures: Option<i32>,
    /// `<fractions>` of that location as `(numerator, denominator)`: a
    /// negative one on an end after a measure's beats counts back from that
    /// measure's end, not the next one's start.
    pub fractions: Option<(i32, i32)>,
}

/// A `<Volta>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MscxVolta {
    /// `<endings>`: the passes it's played on.
    pub endings: Vec<u8>,
    /// `<beginText>` (`1.`).
    pub begin_text: Option<String>,
    /// `<endHookType>`: 0 (or none) for an open bracket, 1 for a hook down.
    pub end_hook_type: Option<u8>,
}

/// A `<BarLine>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MscxBarLine {
    /// `<subtype>` (`normal`, `double`, `end`, `dashed`...).
    pub subtype: String,
    /// `<spanFrom>`/`<spanTo>`: a "tick" or "short" barline's span preset.
    pub span_from: Option<i8>,
    pub span_to: Option<i8>,
    /// `false` for `<visible>0</visible>`.
    pub visible: bool,
}

/// A single rhythmic event (Chord or Rest) inside a voice.
#[derive(Debug, Clone, PartialEq)]
pub struct MscxBeat {
    pub duration: MscxDuration,
    pub kind: MscxBeatKind,
    /// The chord's `<Spanner type="Slur">` elements, in file order.
    pub slurs: Vec<MscxSlurEnd>,
    /// A voice-level `<location>` right before the beat: a gap MuseScore
    /// skips, as `(numerator, denominator)` of a whole note.
    pub gap_before: Option<(i32, i32)>,
    /// A grace chord's kind (`<acciaccatura/>`, `<grace16after/>`...).
    pub grace: Option<MscxGrace>,
    /// The chord's articulations, ornaments, tremolo and chord lines, in
    /// file order.
    pub marks: Vec<MscxChordMark>,
    /// Segment elements in the voice right before the beat.
    pub segment: MscxSegment,
    // And right after it:
    /// `<Breath>` symbols (`breathMarkComma`, `caesura`...).
    pub breaths: Vec<String>,
    /// Hairpin end elements: a hairpin's end element sits at its end tick,
    /// right after the last chord it covers.
    pub hairpin_ends: u32,
}

impl MscxBeat {
    /// A beat with nothing attached.
    pub fn new(duration: MscxDuration, kind: MscxBeatKind) -> Self {
        Self {
            duration,
            kind,
            slurs: Vec::new(),
            gap_before: None,
            grace: None,
            marks: Vec::new(),
            segment: MscxSegment::default(),
            breaths: Vec::new(),
            hairpin_ends: 0,
        }
    }
}

/// The segment elements MuseScore writes in a voice before a beat.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MscxSegment {
    /// The last `<Dynamic>`.
    pub dynamic: Option<MscxDynamic>,
    /// The last `<Harmony>`.
    pub harmony: Option<MscxHarmony>,
    /// Earlier `<Dynamic>`s before the same beat, replaced by the last.
    pub superseded_dynamics: u32,
    /// Earlier `<Harmony>`s before the same beat, replaced by the last.
    pub superseded_harmonies: u32,
    /// `<Fermata>` subtypes.
    pub fermatas: Vec<String>,
    /// `<Spanner type="HairPin">` starts' `<HairPin><subtype>`s, in file
    /// order: 0/1 the crescendo/diminuendo hairpins, 2/3 their
    /// "cresc."/"dim." lines.
    pub hairpin_starts: Vec<u8>,
}

/// A mark inside a `<Chord>`, by its `<subtype>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MscxChordMark {
    /// An `<Articulation>` symbol (MuseScore 3 writes ornaments and
    /// fermatas as these too).
    Articulation(String),
    /// An `<Ornament>` symbol.
    Ornament(String),
    /// A single-chord `<Tremolo>` (`r16`...).
    Tremolo(String),
    /// A `<ChordLine>` (fall, doit, plop, scoop by number).
    ChordLine(String),
}

/// What makes a `<Chord>` a grace chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MscxGrace {
    /// `<acciaccatura/>`: slashed.
    Acciaccatura,
    /// `<appoggiatura/>`.
    Appoggiatura,
    /// `<grace4/>`, `<grace16/>`, `<grace32/>`.
    Grace4,
    Grace16,
    Grace32,
    /// `<grace8after/>`...: grace notes after the main note.
    Grace8After,
    Grace16After,
    Grace32After,
}

impl MscxGrace {
    /// The element name.
    pub fn tag(self) -> &'static str {
        match self {
            MscxGrace::Acciaccatura => "acciaccatura",
            MscxGrace::Appoggiatura => "appoggiatura",
            MscxGrace::Grace4 => "grace4",
            MscxGrace::Grace16 => "grace16",
            MscxGrace::Grace32 => "grace32",
            MscxGrace::Grace8After => "grace8after",
            MscxGrace::Grace16After => "grace16after",
            MscxGrace::Grace32After => "grace32after",
        }
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        Some(match tag {
            "acciaccatura" => MscxGrace::Acciaccatura,
            "appoggiatura" => MscxGrace::Appoggiatura,
            "grace4" => MscxGrace::Grace4,
            "grace16" => MscxGrace::Grace16,
            "grace32" => MscxGrace::Grace32,
            "grace8after" => MscxGrace::Grace8After,
            "grace16after" => MscxGrace::Grace16After,
            "grace32after" => MscxGrace::Grace32After,
            _ => return None,
        })
    }

    /// Whether it follows its main chord.
    pub fn is_after(self) -> bool {
        matches!(
            self,
            MscxGrace::Grace8After | MscxGrace::Grace16After | MscxGrace::Grace32After
        )
    }
}

/// A `<Dynamic>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MscxDynamic {
    /// `<subtype>`: `mf`, `sfz`... or `other-dynamics` (none = that).
    pub subtype: Option<String>,
    /// `<text>`, its markup's text joined (an `other-dynamics` marking),
    /// with dynamic glyphs (`<sym>dynamicPiano</sym>`) as their letters.
    pub text: Option<String>,
    pub velocity: Option<u8>,
    /// `<voiceAssignment>currentVoiceOnly</voiceAssignment>`: it applies
    /// to its own voice only, not (as by default) to all of them.
    pub voice_only: bool,
}

/// The SMuFL glyphs for the single dynamic letters.
const DYNAMIC_GLYPHS: [(char, &str); 7] = [
    ('p', "dynamicPiano"),
    ('m', "dynamicMezzo"),
    ('f', "dynamicForte"),
    ('r', "dynamicRinforzando"),
    ('s', "dynamicSforzando"),
    ('z', "dynamicZ"),
    ('n', "dynamicNiente"),
];

/// The letters a SMuFL dynamic glyph (`dynamicPiano`, `dynamicMF`,
/// `dynamicSforzato`...) shows; `None` for another glyph.
pub fn dynamic_glyph_letters(glyph: &str) -> Option<String> {
    if let Some(&(letter, _)) = DYNAMIC_GLYPHS.iter().find(|&&(_, name)| name == glyph) {
        return Some(letter.to_string());
    }
    let rest = glyph.strip_prefix("dynamic")?;
    let named = match rest {
        "FortePiano" => "fp",
        "Forzando" => "fz",
        "Sforzando1" => "sf",
        "SforzandoPiano" => "sfp",
        "SforzandoPianissimo" => "sfpp",
        "Sforzato" => "sfz",
        "SforzatoPiano" => "sfzp",
        "SforzatoFF" => "sffz",
        "Rinforzando1" => "rf",
        "Rinforzando2" => "rfz",
        // Letter combinations: `dynamicPPP`, `dynamicMF`...
        _ if !rest.is_empty() && rest.chars().all(|c| "PMFRSZN".contains(c)) => {
            return Some(rest.to_lowercase());
        }
        _ => return None,
    };
    Some(named.to_string())
}

/// The SMuFL glyph for a single dynamic letter.
pub fn dynamic_letter_glyph(letter: char) -> Option<&'static str> {
    DYNAMIC_GLYPHS
        .iter()
        .find(|&&(l, _)| l == letter)
        .map(|&(_, glyph)| glyph)
}

/// A `<Harmony>` (chord symbol).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MscxHarmony {
    /// `<harmonyType>`: 0 (or none) for a standard chord symbol, 1 Roman
    /// numeral, 2 Nashville number.
    pub harmony_type: Option<u8>,
    /// `<root>`/`<base>`: tonal pitch classes (C = 14, G = 15, F = 13).
    /// No root for "N.C.".
    pub root: Option<i32>,
    pub base: Option<i32>,
    /// `<name>`: the text after the root (`m7`).
    pub name: String,
}

/// One end of a slur, written as a `<Spanner type="Slur">` inside its
/// chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MscxSlurEnd {
    /// `true` for a start (with `<next>`), `false` for an end (`<prev>`).
    pub start: bool,
    /// Where the other end's chord is, relative to this one.
    pub location: MscxLocation,
}

/// `numerator/denominator` in lowest terms, the sign on the numerator.
/// `denominator` must not be zero.
pub(crate) fn reduce_fraction(numerator: i128, denominator: i128) -> (i128, i128) {
    fn gcd(a: i128, b: i128) -> i128 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
    let divisor = gcd(numerator.abs(), denominator.abs()).max(1);
    let divisor = if denominator < 0 { -divisor } else { divisor };
    (numerator / divisor, denominator / divisor)
}

/// A relative `<location>`: whole measures, then a fraction of a whole
/// note, and voices (all may be negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MscxLocation {
    pub measures: i32,
    /// `<fractions>` as `(numerator, denominator)`; `None` without one.
    pub fractions: Option<(i32, i32)>,
    pub voices: i32,
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
    /// Range roughly [-1, 33]; can be omitted. A transposing staff's
    /// `<tpc2>` (the written note's class) is not read: it follows from the
    /// instrument's transposition.
    pub tpc: Option<i8>,
    /// `<head>` — notehead group (e.g. `cross`, `diamond`).
    pub head: Option<String>,
    /// `<string>` — 0-based string number for tab notes (top string = 0).
    pub string: Option<u8>,
    /// `<fret>` — fret number for tab notes.
    pub fret: Option<u8>,
    /// `<Spanner type="Tie">` presence — set when this note starts a tie.
    pub tie_start: bool,
    /// This note ends a tie initiated by a previous note.
    pub tie_end: bool,
}
