use serde::{Deserialize, Serialize};

use super::note::{FormattedText, Level};

/// Root pitch class of a chord symbol.
#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    #[serde(rename = "root-step")]
    pub root_step: RootStep,
    #[serde(rename = "root-alter")]
    pub root_alter: Option<f64>,
}

/// The letter name of a root note.
#[derive(Debug, Serialize, Deserialize)]
pub struct RootStep {
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Numeral-based chord notation (Roman numeral analysis).
#[derive(Debug, Serialize, Deserialize)]
pub struct Numeral {
    #[serde(rename = "numeral-root")]
    pub numeral_root: NumeralRoot,
    #[serde(rename = "numeral-alter")]
    pub numeral_alter: Option<f64>,
    pub numeral_key: Option<NumeralKey>,
}

/// The Roman numeral root value.
#[derive(Debug, Serialize, Deserialize)]
pub struct NumeralRoot {
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// Key context for a Roman numeral chord.
#[derive(Debug, Serialize, Deserialize)]
pub struct NumeralKey {
    #[serde(rename = "numeral-fifths")]
    pub numeral_fifths: i8,
    #[serde(rename = "numeral-mode")]
    pub numeral_mode: String,
}

/// The quality/type of a chord symbol.
///
/// E.g. `"major"`, `"minor"`, `"dominant"`, `"major-seventh"`, `"half-diminished"`, etc.
#[derive(Debug, Serialize, Deserialize)]
pub struct Kind {
    /// `"yes"` to use a symbol (e.g. △ for major-seventh), `"no"` for text.
    #[serde(rename = "@use-symbols")]
    pub use_symbols: Option<String>,
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@stack-degrees")]
    pub stack_degrees: Option<String>,
    #[serde(rename = "@parentheses-degrees")]
    pub parentheses_degrees: Option<String>,
    #[serde(rename = "@bracket-degrees")]
    pub bracket_degrees: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// The bass note of a slash chord (e.g. C/E).
#[derive(Debug, Serialize, Deserialize)]
pub struct Bass {
    #[serde(rename = "@arrangement")]
    pub arrangement: Option<String>,
    #[serde(rename = "bass-separator")]
    pub bass_separator: Option<StyleText>,
    #[serde(rename = "bass-step")]
    pub bass_step: BassStep,
    #[serde(rename = "bass-alter")]
    pub bass_alter: Option<f64>,
}

/// The letter name of a bass note.
#[derive(Debug, Serialize, Deserialize)]
pub struct BassStep {
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A styled text element.
#[derive(Debug, Serialize, Deserialize)]
pub struct StyleText {
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// An added, altered, or omitted degree in a chord (e.g. `add9`, `b5`, `omit3`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Degree {
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "degree-value")]
    pub degree_value: DegreeValue,
    #[serde(rename = "degree-alter")]
    pub degree_alter: DegreeAlter,
    #[serde(rename = "degree-type")]
    pub degree_type: DegreeType,
}

/// The scale degree number (1–13).
#[derive(Debug, Serialize, Deserialize)]
pub struct DegreeValue {
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// The alteration of the degree (semitones).
#[derive(Debug, Serialize, Deserialize)]
pub struct DegreeAlter {
    #[serde(rename = "@plus-minus")]
    pub plus_minus: Option<String>,
    #[serde(rename = "$text")]
    pub value: f64,
}

/// Whether the degree is added, altered, or omitted.
#[derive(Debug, Serialize, Deserialize)]
pub struct DegreeType {
    #[serde(rename = "@text")]
    pub text: Option<String>,
    /// `"add"`, `"alter"`, `"subtract"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// A fret-board diagram (guitar chord frame).
#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@halign")]
    pub halign: Option<String>,
    #[serde(rename = "@valign")]
    pub valign: Option<String>,
    #[serde(rename = "@height")]
    pub height: Option<f64>,
    #[serde(rename = "@width")]
    pub width: Option<f64>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// Number of strings on the diagram.
    #[serde(rename = "frame-strings")]
    pub frame_strings: u8,
    /// Number of frets shown.
    #[serde(rename = "frame-frets")]
    pub frame_frets: u8,
    /// First fret number shown (default 1).
    #[serde(rename = "first-fret")]
    pub first_fret: Option<FirstFret>,
    #[serde(rename = "frame-note", default)]
    pub frame_notes: Vec<FrameNote>,
}

/// The starting fret of a chord diagram.
#[derive(Debug, Serialize, Deserialize)]
pub struct FirstFret {
    #[serde(rename = "@text")]
    pub text: Option<String>,
    /// `"right"` or `"left"`.
    #[serde(rename = "@location")]
    pub location: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// One note in a fret-board diagram.
#[derive(Debug, Serialize, Deserialize)]
pub struct FrameNote {
    /// String number (1 = highest-pitched).
    pub string: u8,
    pub fret: u8,
    pub fingering: Option<super::note::Fingering>,
    pub barre: Option<Barre>,
}

/// A barre chord indicator in a frame diagram.
#[derive(Debug, Serialize, Deserialize)]
pub struct Barre {
    /// `"start"` or `"stop"`.
    #[serde(rename = "@type")]
    pub barre_type: String,
    #[serde(rename = "@color")]
    pub color: Option<String>,
}

/// A chord symbol (harmony annotation).
///
/// Maps to the `<harmony>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Harmony {
    /// `"explicit"`, `"implied"`, or `"alternate"`.
    #[serde(rename = "@type")]
    pub harmony_type: Option<String>,
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "@print-frame")]
    pub print_frame: Option<String>,
    #[serde(rename = "@arrangement")]
    pub arrangement: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    pub footnote: Option<FormattedText>,
    pub level: Option<Level>,
    // Root, numeral, or function (exactly one should be present)
    pub root: Option<Root>,
    pub numeral: Option<Numeral>,
    pub function: Option<StyleText>,
    pub kind: Kind,
    pub inversion: Option<Inversion>,
    pub bass: Option<Bass>,
    #[serde(rename = "degree", default)]
    pub degrees: Vec<Degree>,
    pub frame: Option<Frame>,
    pub offset: Option<super::direction::Offset>,
    pub staff: Option<u8>,
}

/// Inversion number for a chord (0 = root position, 1 = first inversion, etc.).
#[derive(Debug, Serialize, Deserialize)]
pub struct Inversion {
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: u8,
}
