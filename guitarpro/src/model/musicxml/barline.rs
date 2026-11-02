use serde::{Deserialize, Serialize};

use super::note::{FormattedText, Level, WavyLine};

/// Visual style of a barline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BarStyle {
    Regular,
    Dotted,
    Dashed,
    Heavy,
    LightLight,
    LightHeavy,
    HeavyLight,
    HeavyHeavy,
    Tick,
    Short,
    None,
}

/// The bar style element (carries a color attribute in addition to the style value).
#[derive(Debug, Serialize, Deserialize)]
pub struct BarStyleColor {
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "$text")]
    pub value: BarStyle,
}

/// A repeat mark (start or end of a repeated section).
#[derive(Debug, Serialize, Deserialize)]
pub struct Repeat {
    /// `"forward"` (start repeat) or `"backward"` (end repeat).
    #[serde(rename = "@direction")]
    pub direction: String,
    /// Number of times to repeat (default 2 if absent).
    #[serde(rename = "@times", skip_serializing_if = "Option::is_none")]
    pub times: Option<u8>,
    /// `"yes"` or `"no"` — whether the repeat uses a winged bracket.
    #[serde(rename = "@winged", skip_serializing_if = "Option::is_none")]
    pub winged: Option<String>,
}

/// A first/second/nth ending bracket.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ending {
    /// `"start"`, `"stop"`, or `"discontinue"`.
    #[serde(rename = "@type")]
    pub ending_type: String,
    /// Comma-separated list of ending numbers (e.g. `"1"`, `"1,2"`, `"3"`).
    #[serde(rename = "@number")]
    pub number: String,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@end-length", skip_serializing_if = "Option::is_none")]
    pub end_length: Option<f64>,
    #[serde(rename = "@text-x", skip_serializing_if = "Option::is_none")]
    pub text_x: Option<f64>,
    #[serde(rename = "@text-y", skip_serializing_if = "Option::is_none")]
    pub text_y: Option<f64>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A fermata on a barline (whole-measure fermata).
#[derive(Debug, Serialize, Deserialize)]
pub struct Fermata {
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub fermata_type: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A segno or coda sign on a barline.
#[derive(Debug, Serialize, Deserialize)]
pub struct BarlineSegno {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
}

/// A barline, with optional repeat, ending, and style information.
///
/// Maps to the `<barline>` element.
///
/// `location` indicates where on the measure the barline appears:
/// - `"right"` (default) — at the end of the measure
/// - `"left"` — at the start of the measure
/// - `"middle"` — mid-measure (e.g. metric modulation)
#[derive(Debug, Serialize, Deserialize)]
pub struct Barline {
    #[serde(rename = "@location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(rename = "@segno", skip_serializing_if = "Option::is_none")]
    pub segno: Option<String>,
    #[serde(rename = "@coda", skip_serializing_if = "Option::is_none")]
    pub coda: Option<String>,
    #[serde(rename = "@divisions", skip_serializing_if = "Option::is_none")]
    pub divisions: Option<f64>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "bar-style", skip_serializing_if = "Option::is_none")]
    pub bar_style: Option<BarStyleColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnote: Option<FormattedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    #[serde(rename = "wavy-line", skip_serializing_if = "Option::is_none")]
    pub wavy_line: Option<WavyLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segno_mark: Option<BarlineSegno>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coda_mark: Option<BarlineSegno>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending: Option<Ending>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Repeat>,
    #[serde(rename = "fermata", default)]
    pub fermatas: Vec<Fermata>,
}
