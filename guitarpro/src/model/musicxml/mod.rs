pub mod attributes;
pub mod barline;
pub mod credit;
pub mod defaults;
pub mod direction;
pub mod harmony;
pub mod identification;
pub mod measure;
pub mod note;
pub mod part_list;

use serde::Deserialize;

/// Work (opus/title) metadata.
///
/// Maps to the `<work>` element.
#[derive(Debug, Deserialize)]
pub struct Work {
    #[serde(rename = "work-number")]
    pub work_number: Option<String>,
    #[serde(rename = "work-title")]
    pub work_title: Option<String>,
    pub opus: Option<Opus>,
}

/// A reference to an external MusicXML opus file.
#[derive(Debug, Deserialize)]
pub struct Opus {
    #[serde(rename = "@xlink:href")]
    pub href: Option<String>,
    #[serde(rename = "@xlink:type")]
    pub link_type: Option<String>,
    #[serde(rename = "@xlink:title")]
    pub title: Option<String>,
}

/// A single instrument part: an ordered sequence of measures.
///
/// Maps to the `<part>` element inside `<score-partwise>`.
/// The `id` references a `<score-part>` in the `<part-list>`.
#[derive(Debug, Deserialize)]
pub struct Part {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "measure", default)]
    pub measures: Vec<measure::Measure>,
}

/// The root element of a MusicXML score-partwise document.
///
/// In `score-partwise` format the music is organized first by part, then by measure.
/// This is the format produced by virtually all notation software (Finale, Sibelius,
/// MuseScore, Guitar Pro, etc.) and is the primary target of this model.
///
/// ```xml
/// <score-partwise version="4.0">
///   <part-list> … </part-list>
///   <part id="P1">
///     <measure number="1"> … </measure>
///   </part>
/// </score-partwise>
/// ```
///
/// MusicXML versions 3.0, 3.1, and 4.0 all share this structure; the `version`
/// attribute carries the version string.
#[derive(Debug, Deserialize)]
#[serde(rename = "score-partwise")]
pub struct ScorePartwise {
    /// MusicXML version string, e.g. `"4.0"`, `"3.1"`, `"3.0"`.
    #[serde(rename = "@version")]
    pub version: Option<String>,

    // --- header ---
    pub work: Option<Work>,
    #[serde(rename = "movement-number")]
    pub movement_number: Option<String>,
    #[serde(rename = "movement-title")]
    pub movement_title: Option<String>,
    pub identification: Option<identification::Identification>,
    pub defaults: Option<defaults::Defaults>,
    #[serde(rename = "credit", default)]
    pub credits: Vec<credit::Credit>,

    // --- part list (instrument definitions) ---
    #[serde(rename = "part-list")]
    pub part_list: part_list::PartList,

    // --- musical content ---
    #[serde(rename = "part", default)]
    pub parts: Vec<Part>,
}
