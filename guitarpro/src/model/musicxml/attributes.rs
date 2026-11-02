use serde::{Deserialize, Serialize};

/// Traditional key signature: number of sharps (positive) or flats (negative).
#[derive(Debug, Serialize, Deserialize)]
pub struct TraditionalKey {
    /// Semitone alteration of the cancel key (-7 to 7).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel: Option<Cancel>,
    /// Number of sharps or flats: positive = sharps, negative = flats.
    pub fifths: i8,
    /// `"major"`, `"minor"`, `"dorian"`, `"phrygian"`, `"lydian"`,
    /// `"mixolydian"`, `"aeolian"`, `"ionian"`, `"locrian"`, `"none"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// Indicates a key cancellation (the previous key signature is explicitly cancelled).
#[derive(Debug, Serialize, Deserialize)]
pub struct Cancel {
    /// `"left"`, `"right"`, or `"before-barline"`.
    #[serde(rename = "@location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(rename = "$text")]
    pub fifths: i8,
}

/// One accidental in a non-traditional key signature.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyAccidental {
    #[serde(rename = "key-step")]
    pub key_step: String,
    #[serde(rename = "key-alter")]
    pub key_alter: f64,
    #[serde(rename = "key-accidental", skip_serializing_if = "Option::is_none")]
    pub key_accidental: Option<String>,
}

/// Key signature.
///
/// Can be traditional (fifths-based) or explicit (list of key-step/key-alter pairs).
#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    /// Staff number this key applies to (multi-staff parts only).
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    // Traditional key fields (present for traditional key signatures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel: Option<Cancel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifths: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    // Non-traditional key fields
    #[serde(rename = "key-step", default)]
    pub key_steps: Vec<String>,
    #[serde(rename = "key-alter", default)]
    pub key_alters: Vec<f64>,
    #[serde(rename = "key-accidental", default)]
    pub key_accidentals: Vec<String>,
    #[serde(rename = "key-octave", default)]
    pub key_octaves: Vec<KeyOctave>,
}

/// Specifies which octave a key-signature alteration applies to.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyOctave {
    /// Index into the list of key-step/key-alter pairs (1-based).
    #[serde(rename = "@number")]
    pub number: u8,
    #[serde(rename = "@cancel", skip_serializing_if = "Option::is_none")]
    pub cancel: Option<String>,
    #[serde(rename = "$text")]
    pub octave: i8,
}

/// Time signature numerator/denominator beat pair (for interchangeable time).
#[derive(Debug, Serialize, Deserialize)]
pub struct BeatUnit {
    pub beats: String,
    #[serde(rename = "beat-type")]
    pub beat_type: String,
}

/// Time signature.
#[derive(Debug, Serialize, Deserialize)]
pub struct Time {
    /// Staff number this time signature applies to.
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    /// `"common"`, `"cut"`, `"single-number"`, `"note"`, `"dotted-note"`, `"normal"`.
    #[serde(rename = "@symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "@separator", skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    /// Number of beats per measure. For compound time: comma-separated (e.g. `"3+2"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beats: Option<String>,
    /// Note value of one beat (e.g. `"4"` for quarter note).
    #[serde(rename = "beat-type", skip_serializing_if = "Option::is_none")]
    pub beat_type: Option<String>,
    /// Present for senza-misura (unmeasured) passages.
    #[serde(rename = "senza-misura", skip_serializing_if = "Option::is_none")]
    pub senza_misura: Option<String>,
    /// Additional beat groups for interchangeable time signatures.
    #[serde(rename = "interchangeable", skip_serializing_if = "Option::is_none")]
    pub interchangeable: Option<Interchangeable>,
}

/// Alternate time-signature display for interchangeable meters.
#[derive(Debug, Serialize, Deserialize)]
pub struct Interchangeable {
    #[serde(rename = "@symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beats: Option<String>,
    #[serde(rename = "beat-type", skip_serializing_if = "Option::is_none")]
    pub beat_type: Option<String>,
}

/// Staff type override (e.g. for a TAB staff or a percussion staff).
#[derive(Debug, Serialize, Deserialize)]
pub struct StaffDetails {
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@show-frets", skip_serializing_if = "Option::is_none")]
    pub show_frets: Option<String>,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "@print-spacing", skip_serializing_if = "Option::is_none")]
    pub print_spacing: Option<String>,
    /// `"ossia"`, `"cue"`, `"editorial"`, `"regular"`, `"alternate"`.
    #[serde(rename = "staff-type", skip_serializing_if = "Option::is_none")]
    pub staff_type: Option<String>,
    /// Number of lines on this staff (default 5).
    #[serde(rename = "staff-lines", skip_serializing_if = "Option::is_none")]
    pub staff_lines: Option<u8>,
    #[serde(rename = "line-detail", default)]
    pub line_details: Vec<LineDetail>,
    #[serde(rename = "staff-tuning", default)]
    pub staff_tunings: Vec<StaffTuning>,
    #[serde(rename = "capo", skip_serializing_if = "Option::is_none")]
    pub capo: Option<u8>,
    #[serde(rename = "staff-size", skip_serializing_if = "Option::is_none")]
    pub staff_size: Option<f64>,
}

/// Visual property of a single staff line.
#[derive(Debug, Serialize, Deserialize)]
pub struct LineDetail {
    #[serde(rename = "@line")]
    pub line: u8,
    #[serde(rename = "@width", skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@line-type", skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
}

/// Open-string tuning for one line of a TAB staff.
#[derive(Debug, Serialize, Deserialize)]
pub struct StaffTuning {
    /// Staff line number (1 = lowest).
    #[serde(rename = "@line")]
    pub line: u8,
    #[serde(rename = "tuning-step")]
    pub tuning_step: String,
    #[serde(rename = "tuning-alter", skip_serializing_if = "Option::is_none")]
    pub tuning_alter: Option<f64>,
    #[serde(rename = "tuning-octave")]
    pub tuning_octave: i8,
}

/// Clef definition.
#[derive(Debug, Serialize, Deserialize)]
pub struct Clef {
    /// Staff number (multi-staff parts only).
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@additional", skip_serializing_if = "Option::is_none")]
    pub additional: Option<String>,
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(rename = "@after-barline", skip_serializing_if = "Option::is_none")]
    pub after_barline: Option<String>,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    /// `"G"`, `"F"`, `"C"`, `"percussion"`, `"TAB"`, `"jianpu"`, `"none"`.
    pub sign: String,
    /// Staff line the clef sign sits on (counting from the bottom, 1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u8>,
    /// Octave transposition: `-2`, `-1`, `0`, `1`, `2`.
    #[serde(rename = "clef-octave-change", skip_serializing_if = "Option::is_none")]
    pub clef_octave_change: Option<i8>,
}

/// Chromatic transposition interval for a transposing instrument.
#[derive(Debug, Serialize, Deserialize)]
pub struct Transpose {
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    /// Number of diatonic steps to transpose (e.g. `5` for a 5th).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diatonic: Option<i16>,
    /// Exact chromatic semitones to transpose.
    pub chromatic: i16,
    /// Octave adjustment on top of the chromatic interval.
    #[serde(rename = "octave-change", skip_serializing_if = "Option::is_none")]
    pub octave_change: Option<i8>,
    /// Present when written pitch is enharmonically equivalent (e.g. B# = C).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double: Option<()>,
}

/// Directive text (e.g. "Allegro") at the start of a movement.
#[derive(Debug, Serialize, Deserialize)]
pub struct Directive {
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Musical attributes that apply from this point in the measure until changed.
///
/// The first `<attributes>` in a movement typically sets divisions, key, time, and clef.
/// Maps to the `<attributes>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Attributes {
    /// Scaling factor: number of divisions per quarter note.
    ///
    /// All `<duration>` values in the part are expressed as multiples of `1/divisions`
    /// of a quarter note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divisions: Option<u32>,
    #[serde(rename = "key", default)]
    pub keys: Vec<Key>,
    #[serde(rename = "time", default)]
    pub times: Vec<Time>,
    /// Number of staves in the part (defaults to 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staves: Option<u8>,
    /// `"open"`, `"rhythm"`, or a number.
    #[serde(rename = "part-symbol", skip_serializing_if = "Option::is_none")]
    pub part_symbol: Option<PartSymbol>,
    /// Number of instruments (for multi-instrument parts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruments: Option<u8>,
    #[serde(rename = "clef", default)]
    pub clefs: Vec<Clef>,
    #[serde(rename = "staff-details", default)]
    pub staff_details: Vec<StaffDetails>,
    #[serde(rename = "transpose", default)]
    pub transposes: Vec<Transpose>,
    #[serde(rename = "for-part", default)]
    pub for_parts: Vec<ForPart>,
    #[serde(rename = "directive", default)]
    pub directives: Vec<Directive>,
    #[serde(rename = "measure-style", default)]
    pub measure_styles: Vec<MeasureStyle>,
}

/// Bracket or brace drawn to the left of a multi-staff part.
#[derive(Debug, Serialize, Deserialize)]
pub struct PartSymbol {
    #[serde(rename = "@top-staff", skip_serializing_if = "Option::is_none")]
    pub top_staff: Option<u8>,
    #[serde(rename = "@bottom-staff", skip_serializing_if = "Option::is_none")]
    pub bottom_staff: Option<u8>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Transposition information scoped to a specific part in a multi-part measure.
#[derive(Debug, Serialize, Deserialize)]
pub struct ForPart {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "part-clef", skip_serializing_if = "Option::is_none")]
    pub part_clef: Option<Clef>,
    #[serde(rename = "part-transpose", skip_serializing_if = "Option::is_none")]
    pub part_transpose: Option<Transpose>,
}

/// Measure-style overrides: multiple rests, measure repeats, beat repeats, slash notation.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeasureStyle {
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "$value")]
    pub style: MeasureStyleContent,
}

/// The specific style override inside `<measure-style>`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeasureStyleContent {
    MultipleRest(MultipleRest),
    MeasureRepeat(MeasureRepeat),
    BeatRepeat(BeatRepeat),
    Slash(Slash),
}

/// A multi-measure rest.
#[derive(Debug, Serialize, Deserialize)]
pub struct MultipleRest {
    #[serde(rename = "@use-symbols", skip_serializing_if = "Option::is_none")]
    pub use_symbols: Option<String>,
    #[serde(rename = "$text")]
    pub count: u16,
}

/// A measure-repeat symbol.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeasureRepeat {
    /// `"start"` or `"stop"`.
    #[serde(rename = "@type")]
    pub repeat_type: String,
    #[serde(rename = "@slashes", skip_serializing_if = "Option::is_none")]
    pub slashes: Option<u8>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A beat-repeat symbol.
#[derive(Debug, Serialize, Deserialize)]
pub struct BeatRepeat {
    #[serde(rename = "@type")]
    pub repeat_type: String,
    #[serde(rename = "@slashes", skip_serializing_if = "Option::is_none")]
    pub slashes: Option<u8>,
    #[serde(rename = "@use-dots", skip_serializing_if = "Option::is_none")]
    pub use_dots: Option<String>,
    #[serde(rename = "slash-type", skip_serializing_if = "Option::is_none")]
    pub slash_type: Option<String>,
    #[serde(rename = "slash-dot", default)]
    pub slash_dots: Vec<()>,
    #[serde(rename = "except-voice", default)]
    pub except_voices: Vec<u8>,
}

/// Slash notation (rhythmic notation).
#[derive(Debug, Serialize, Deserialize)]
pub struct Slash {
    #[serde(rename = "@type")]
    pub slash_type: String,
    #[serde(rename = "@use-dots", skip_serializing_if = "Option::is_none")]
    pub use_dots: Option<String>,
    #[serde(rename = "@use-stems", skip_serializing_if = "Option::is_none")]
    pub use_stems: Option<String>,
    #[serde(rename = "slash-type", skip_serializing_if = "Option::is_none")]
    pub slash_note_type: Option<String>,
    #[serde(rename = "slash-dot", default)]
    pub slash_dots: Vec<()>,
    #[serde(rename = "except-voice", default)]
    pub except_voices: Vec<u8>,
}
