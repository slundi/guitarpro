use serde::{Deserialize, Serialize};

use super::note::{FormattedText, Level};

/// Root pitch class of a chord symbol.
#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    #[serde(rename = "root-step")]
    pub root_step: RootStep,
    #[serde(rename = "root-alter", skip_serializing_if = "Option::is_none")]
    pub root_alter: Option<f64>,
}

/// The letter name of a root note.
#[derive(Debug, Serialize, Deserialize)]
pub struct RootStep {
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Numeral-based chord notation (Roman numeral analysis).
#[derive(Debug, Serialize, Deserialize)]
pub struct Numeral {
    #[serde(rename = "numeral-root")]
    pub numeral_root: NumeralRoot,
    #[serde(rename = "numeral-alter", skip_serializing_if = "Option::is_none")]
    pub numeral_alter: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeral_key: Option<NumeralKey>,
}

/// The Roman numeral root value.
#[derive(Debug, Serialize, Deserialize)]
pub struct NumeralRoot {
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "@use-symbols", skip_serializing_if = "Option::is_none")]
    pub use_symbols: Option<String>,
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "@stack-degrees", skip_serializing_if = "Option::is_none")]
    pub stack_degrees: Option<String>,
    #[serde(
        rename = "@parentheses-degrees",
        skip_serializing_if = "Option::is_none"
    )]
    pub parentheses_degrees: Option<String>,
    #[serde(rename = "@bracket-degrees", skip_serializing_if = "Option::is_none")]
    pub bracket_degrees: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// The bass note of a slash chord (e.g. C/E).
#[derive(Debug, Serialize, Deserialize)]
pub struct Bass {
    #[serde(rename = "@arrangement", skip_serializing_if = "Option::is_none")]
    pub arrangement: Option<String>,
    #[serde(rename = "bass-separator", skip_serializing_if = "Option::is_none")]
    pub bass_separator: Option<StyleText>,
    #[serde(rename = "bass-step")]
    pub bass_step: BassStep,
    #[serde(rename = "bass-alter", skip_serializing_if = "Option::is_none")]
    pub bass_alter: Option<f64>,
}

/// The letter name of a bass note.
#[derive(Debug, Serialize, Deserialize)]
pub struct BassStep {
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A styled text element.
#[derive(Debug, Serialize, Deserialize)]
pub struct StyleText {
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// An added, altered, or omitted degree in a chord (e.g. `add9`, `b5`, `omit3`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Degree {
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// The alteration of the degree (semitones).
#[derive(Debug, Serialize, Deserialize)]
pub struct DegreeAlter {
    #[serde(rename = "@plus-minus", skip_serializing_if = "Option::is_none")]
    pub plus_minus: Option<String>,
    #[serde(rename = "$text")]
    pub value: f64,
}

/// Whether the degree is added, altered, or omitted.
#[derive(Debug, Serialize, Deserialize)]
pub struct DegreeType {
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// `"add"`, `"alter"`, `"subtract"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// A fret-board diagram (guitar chord frame).
#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@halign", skip_serializing_if = "Option::is_none")]
    pub halign: Option<String>,
    #[serde(rename = "@valign", skip_serializing_if = "Option::is_none")]
    pub valign: Option<String>,
    #[serde(rename = "@height", skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(rename = "@width", skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Number of strings on the diagram.
    #[serde(rename = "frame-strings")]
    pub frame_strings: u8,
    /// Number of frets shown.
    #[serde(rename = "frame-frets")]
    pub frame_frets: u8,
    /// First fret number shown (default 1).
    #[serde(rename = "first-fret", skip_serializing_if = "Option::is_none")]
    pub first_fret: Option<FirstFret>,
    #[serde(rename = "frame-note", default)]
    pub frame_notes: Vec<FrameNote>,
}

/// The starting fret of a chord diagram.
#[derive(Debug, Serialize, Deserialize)]
pub struct FirstFret {
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// `"right"` or `"left"`.
    #[serde(rename = "@location", skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingering: Option<super::note::Fingering>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barre: Option<Barre>,
}

/// A barre chord indicator in a frame diagram.
#[derive(Debug, Serialize, Deserialize)]
pub struct Barre {
    /// `"start"` or `"stop"`.
    #[serde(rename = "@type")]
    pub barre_type: String,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// A chord symbol (harmony annotation).
///
/// Maps to the `<harmony>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Harmony {
    /// `"explicit"`, `"implied"`, or `"alternate"`.
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub harmony_type: Option<String>,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "@print-frame", skip_serializing_if = "Option::is_none")]
    pub print_frame: Option<String>,
    #[serde(rename = "@arrangement", skip_serializing_if = "Option::is_none")]
    pub arrangement: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnote: Option<FormattedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    // Root, numeral, or function (exactly one should be present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<Root>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeral: Option<Numeral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<StyleText>,
    pub kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inversion: Option<Inversion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bass: Option<Bass>,
    #[serde(rename = "degree", default)]
    pub degrees: Vec<Degree>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<Frame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<super::direction::Offset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff: Option<u8>,
}

/// Inversion number for a chord (0 = root position, 1 = first inversion, etc.).
#[derive(Debug, Serialize, Deserialize)]
pub struct Inversion {
    #[serde(rename = "@text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: u8,
}
