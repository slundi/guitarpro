use serde::{Deserialize, Serialize};

use super::{
    attributes::Attributes,
    barline::Barline,
    direction::{Direction, Sound},
    harmony::Harmony,
    note::Note,
};

/// Move the time cursor backwards in the current measure (to notate additional voices).
///
/// Maps to the `<backup>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Backup {
    /// Duration to move backward, in divisions.
    pub duration: u32,
}

/// Move the time cursor forward in the current measure (to skip over time).
///
/// Maps to the `<forward>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Forward {
    /// Duration to move forward, in divisions.
    pub duration: u32,
    pub voice: Option<String>,
    pub staff: Option<u8>,
}

/// Page/system layout break hint.
#[derive(Debug, Serialize, Deserialize)]
pub struct Print {
    /// `"yes"` to force a new system/page here.
    #[serde(rename = "@new-system")]
    pub new_system: Option<String>,
    #[serde(rename = "@new-page")]
    pub new_page: Option<String>,
    #[serde(rename = "@blank-page")]
    pub blank_page: Option<u32>,
    #[serde(rename = "@page-number")]
    pub page_number: Option<String>,
    #[serde(rename = "@staff-spacing")]
    pub staff_spacing: Option<f64>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "system-layout")]
    pub system_layout: Option<super::defaults::SystemLayout>,
    #[serde(rename = "staff-layout", default)]
    pub staff_layouts: Vec<super::defaults::StaffLayout>,
    #[serde(rename = "measure-layout")]
    pub measure_layout: Option<MeasureLayout>,
    #[serde(rename = "measure-numbering")]
    pub measure_numbering: Option<MeasureNumbering>,
    #[serde(rename = "part-name-display")]
    pub part_name_display: Option<super::part_list::NameDisplay>,
    #[serde(rename = "part-abbreviation-display")]
    pub part_abbreviation_display: Option<super::part_list::NameDisplay>,
}

/// Layout distance for a measure.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeasureLayout {
    #[serde(rename = "measure-distance")]
    pub measure_distance: Option<f64>,
}

/// Where and how measure numbers are displayed.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeasureNumbering {
    #[serde(rename = "@system")]
    pub system: Option<String>,
    #[serde(rename = "@staff")]
    pub staff: Option<u8>,
    #[serde(rename = "@multiple-rest-always")]
    pub multiple_rest_always: Option<String>,
    #[serde(rename = "@multiple-rest-range")]
    pub multiple_rest_range: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    /// `"none"`, `"measure"`, `"system"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// A figured bass indication.
#[derive(Debug, Serialize, Deserialize)]
pub struct FiguredBass {
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "@parentheses")]
    pub parentheses: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "figure", default)]
    pub figures: Vec<Figure>,
    pub duration: Option<u32>,
    pub footnote: Option<super::note::FormattedText>,
    pub level: Option<super::note::Level>,
}

/// One figure in a figured-bass notation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Figure {
    #[serde(rename = "prefix")]
    pub prefix: Option<StyleText>,
    #[serde(rename = "figure-number")]
    pub figure_number: Option<StyleText>,
    pub suffix: Option<StyleText>,
    pub extend: Option<super::note::Extend>,
    pub footnote: Option<super::note::FormattedText>,
    pub level: Option<super::note::Level>,
}

/// A styled text element used in figured-bass figures.
#[derive(Debug, Serialize, Deserialize)]
pub struct StyleText {
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A grouping or category marker (e.g. for audio analysis software).
#[derive(Debug, Serialize, Deserialize)]
pub struct Grouping {
    #[serde(rename = "@type")]
    pub grouping_type: String,
    #[serde(rename = "@number")]
    pub number: Option<String>,
    #[serde(rename = "@member-of")]
    pub member_of: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "feature", default)]
    pub features: Vec<Feature>,
}

/// One feature annotation inside a `<grouping>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    #[serde(rename = "@type")]
    pub feature_type: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A hyperlink inside a measure (rare; used in interactive scores).
#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    #[serde(rename = "@xlink:href")]
    pub href: Option<String>,
    #[serde(rename = "@xlink:type")]
    pub link_type: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@element")]
    pub element: Option<String>,
    #[serde(rename = "@position")]
    pub position: Option<u32>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
}

/// A named anchor point inside a measure (for hyperlinks).
#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@element")]
    pub element: Option<String>,
    #[serde(rename = "@position")]
    pub position: Option<u32>,
}

/// One musical event inside a measure.
///
/// The ordering of elements in a measure is significant: `<backup>` and `<forward>`
/// elements advance or retract the time cursor, allowing multiple voices to be encoded
/// sequentially within the same `<measure>`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MusicData {
    Note(Note),
    Backup(Backup),
    Forward(Forward),
    Direction(Direction),
    Attributes(Attributes),
    Harmony(Harmony),
    FiguredBass(FiguredBass),
    Print(Print),
    Sound(Sound),
    Listening(super::direction::Listening),
    Barline(Barline),
    Grouping(Grouping),
    Link(Link),
    Bookmark(Bookmark),
}

/// One measure of music for a single part.
///
/// Maps to the `<measure>` element inside `<part>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Measure {
    /// Measure number (typically `"1"`, `"2"`, …; may be non-numeric for pickup measures).
    #[serde(rename = "@number")]
    pub number: String,
    /// `"yes"` for an implicit measure (e.g. a pickup bar).
    #[serde(rename = "@implicit")]
    pub implicit: Option<String>,
    /// `"yes"` for a non-controlling measure (for multi-part measures).
    #[serde(rename = "@non-controlling")]
    pub non_controlling: Option<String>,
    /// Rendered width of the measure in tenths.
    #[serde(rename = "@width")]
    pub width: Option<f64>,
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// All musical events inside this measure, in time-cursor order.
    #[serde(rename = "$value", default)]
    pub music_data: Vec<MusicData>,
}
