use serde::{Deserialize, Serialize};

/// Scaling factors between written distances and tenths.
///
/// `millimeters` is the real-world size of `tenths` staff-space units.
/// All other distances in `<defaults>` are expressed in tenths.
#[derive(Debug, Serialize, Deserialize)]
pub struct Scaling {
    pub millimeters: f64,
    pub tenths: f64,
}

/// Size of a page in tenths.
#[derive(Debug, Serialize, Deserialize)]
pub struct PageSize {
    #[serde(rename = "page-height")]
    pub page_height: f64,
    #[serde(rename = "page-width")]
    pub page_width: f64,
}

/// Margins for a page side. `type` is `"both"`, `"even"`, or `"odd"`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PageMargins {
    #[serde(rename = "@type")]
    pub margin_type: Option<String>,
    #[serde(rename = "left-margin")]
    pub left_margin: f64,
    #[serde(rename = "right-margin")]
    pub right_margin: f64,
    #[serde(rename = "top-margin")]
    pub top_margin: f64,
    #[serde(rename = "bottom-margin")]
    pub bottom_margin: f64,
}

/// Page layout: size and margins.
#[derive(Debug, Serialize, Deserialize)]
pub struct PageLayout {
    #[serde(flatten)]
    pub size: Option<PageSize>,
    #[serde(rename = "page-margins", default)]
    pub page_margins: Vec<PageMargins>,
}

/// Margins for a system (one row of measures across the page).
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMargins {
    #[serde(rename = "left-margin")]
    pub left_margin: f64,
    #[serde(rename = "right-margin")]
    pub right_margin: f64,
}

/// Vertical distance defaults for systems.
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemLayout {
    #[serde(rename = "system-margins")]
    pub system_margins: Option<SystemMargins>,
    #[serde(rename = "system-distance")]
    pub system_distance: Option<f64>,
    #[serde(rename = "top-system-distance")]
    pub top_system_distance: Option<f64>,
    #[serde(rename = "system-dividers")]
    pub system_dividers: Option<SystemDividers>,
}

/// Whether system dividers are shown.
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemDividers {
    #[serde(rename = "left-divider")]
    pub left_divider: Option<EmptyPrintStyleAlign>,
    #[serde(rename = "right-divider")]
    pub right_divider: Option<EmptyPrintStyleAlign>,
}

/// An empty element carrying only print-style and alignment attributes.
#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyPrintStyleAlign {
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
}

/// Default distance between staves within a part.
#[derive(Debug, Serialize, Deserialize)]
pub struct StaffLayout {
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "staff-distance")]
    pub staff_distance: Option<f64>,
}

/// A named font (family, style, size).
#[derive(Debug, Serialize, Deserialize)]
pub struct Font {
    #[serde(rename = "@font-family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font-style")]
    pub font_style: Option<String>,
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight")]
    pub font_weight: Option<String>,
}

/// A named lyric font. The `name` attribute matches `<lyric name="">`.
#[derive(Debug, Serialize, Deserialize)]
pub struct LyricFont {
    #[serde(rename = "@number")]
    pub number: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(flatten)]
    pub font: Font,
}

/// A named lyric language setting.
#[derive(Debug, Serialize, Deserialize)]
pub struct LyricLanguage {
    #[serde(rename = "@number")]
    pub number: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@xml:lang")]
    pub lang: Option<String>,
}

/// Visual appearance defaults: line widths, note sizes, distances.
#[derive(Debug, Serialize, Deserialize)]
pub struct Appearance {
    #[serde(rename = "line-width", default)]
    pub line_widths: Vec<LineWidth>,
    #[serde(rename = "note-size", default)]
    pub note_sizes: Vec<NoteSize>,
    #[serde(rename = "distance", default)]
    pub distances: Vec<Distance>,
    #[serde(rename = "glyph", default)]
    pub glyphs: Vec<Glyph>,
    #[serde(rename = "other-appearance", default)]
    pub other: Vec<OtherAppearance>,
}

/// Width of a specific line type (e.g. `"beam"`, `"staff"`, `"stem"`).
#[derive(Debug, Serialize, Deserialize)]
pub struct LineWidth {
    #[serde(rename = "@type")]
    pub line_type: String,
    #[serde(rename = "$text")]
    pub value: f64,
}

/// Size of a note head type expressed as a percentage of the normal note head size.
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteSize {
    #[serde(rename = "@type")]
    pub note_type: String,
    #[serde(rename = "$text")]
    pub value: f64,
}

/// A named distance value (e.g. `"hyphen"`, `"beam"`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Distance {
    #[serde(rename = "@type")]
    pub distance_type: String,
    #[serde(rename = "$text")]
    pub value: f64,
}

/// A SMuFL glyph override.
#[derive(Debug, Serialize, Deserialize)]
pub struct Glyph {
    #[serde(rename = "@type")]
    pub glyph_type: String,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An application-specific appearance setting.
#[derive(Debug, Serialize, Deserialize)]
pub struct OtherAppearance {
    #[serde(rename = "@type")]
    pub appearance_type: String,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Score-wide rendering defaults: scaling, page layout, fonts, and appearance.
///
/// Maps to the `<defaults>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Defaults {
    pub scaling: Option<Scaling>,
    #[serde(rename = "concert-score")]
    pub concert_score: Option<()>,
    #[serde(rename = "page-layout")]
    pub page_layout: Option<PageLayout>,
    #[serde(rename = "system-layout")]
    pub system_layout: Option<SystemLayout>,
    #[serde(rename = "staff-layout", default)]
    pub staff_layouts: Vec<StaffLayout>,
    pub appearance: Option<Appearance>,
    #[serde(rename = "music-font")]
    pub music_font: Option<Font>,
    #[serde(rename = "word-font")]
    pub word_font: Option<Font>,
    #[serde(rename = "lyric-font", default)]
    pub lyric_fonts: Vec<LyricFont>,
    #[serde(rename = "lyric-language", default)]
    pub lyric_languages: Vec<LyricLanguage>,
}
