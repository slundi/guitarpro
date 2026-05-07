use serde::Deserialize;

use super::note::{FormattedText, Level, WavyLine};

/// Visual style of a barline.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Deserialize)]
pub struct BarStyleColor {
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "$text")]
    pub value: BarStyle,
}

/// A repeat mark (start or end of a repeated section).
#[derive(Debug, Deserialize)]
pub struct Repeat {
    /// `"forward"` (start repeat) or `"backward"` (end repeat).
    #[serde(rename = "@direction")]
    pub direction: String,
    /// Number of times to repeat (default 2 if absent).
    #[serde(rename = "@times")]
    pub times: Option<u8>,
    /// `"yes"` or `"no"` — whether the repeat uses a winged bracket.
    #[serde(rename = "@winged")]
    pub winged: Option<String>,
}

/// A first/second/nth ending bracket.
#[derive(Debug, Deserialize)]
pub struct Ending {
    /// `"start"`, `"stop"`, or `"discontinue"`.
    #[serde(rename = "@type")]
    pub ending_type: String,
    /// Comma-separated list of ending numbers (e.g. `"1"`, `"1,2"`, `"3"`).
    #[serde(rename = "@number")]
    pub number: String,
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@end-length")]
    pub end_length: Option<f64>,
    #[serde(rename = "@text-x")]
    pub text_x: Option<f64>,
    #[serde(rename = "@text-y")]
    pub text_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A fermata on a barline (whole-measure fermata).
#[derive(Debug, Deserialize)]
pub struct Fermata {
    #[serde(rename = "@type")]
    pub fermata_type: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A segno or coda sign on a barline.
#[derive(Debug, Deserialize)]
pub struct BarlineSegno {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@smufl")]
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
#[derive(Debug, Deserialize)]
pub struct Barline {
    #[serde(rename = "@location")]
    pub location: Option<String>,
    #[serde(rename = "@segno")]
    pub segno: Option<String>,
    #[serde(rename = "@coda")]
    pub coda: Option<String>,
    #[serde(rename = "@divisions")]
    pub divisions: Option<f64>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "bar-style")]
    pub bar_style: Option<BarStyleColor>,
    pub footnote: Option<FormattedText>,
    pub level: Option<Level>,
    #[serde(rename = "wavy-line")]
    pub wavy_line: Option<WavyLine>,
    pub segno_mark: Option<BarlineSegno>,
    pub coda_mark: Option<BarlineSegno>,
    pub ending: Option<Ending>,
    pub repeat: Option<Repeat>,
    #[serde(rename = "fermata", default)]
    pub fermatas: Vec<Fermata>,
}
