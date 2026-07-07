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

use serde::{Deserialize, Serialize};

/// Work (opus/title) metadata.
///
/// Maps to the `<work>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Work {
    #[serde(rename = "work-number")]
    pub work_number: Option<String>,
    #[serde(rename = "work-title")]
    pub work_title: Option<String>,
    pub opus: Option<Opus>,
}

/// A reference to an external MusicXML opus file.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "measure", default)]
    pub measures: Vec<measure::Measure>,
}

/// One part's worth of music within a single `<measure>` in `score-timewise` format.
///
/// Maps to the `<part>` element inside `<measure>` inside `<score-timewise>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TimewisePart {
    /// References a `<score-part id="…">` in the `<part-list>`.
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "$value", default)]
    pub music_data: Vec<measure::MusicData>,
}

/// One measure across all parts in `score-timewise` format.
///
/// Maps to the `<measure>` element inside `<score-timewise>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TimewiseMeasure {
    #[serde(rename = "@number")]
    pub number: String,
    #[serde(rename = "@implicit")]
    pub implicit: Option<String>,
    #[serde(rename = "@non-controlling")]
    pub non_controlling: Option<String>,
    #[serde(rename = "@width")]
    pub width: Option<f64>,
    #[serde(rename = "@text")]
    pub text: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "part", default)]
    pub parts: Vec<TimewisePart>,
}

/// The root element of a MusicXML `score-timewise` document.
///
/// In `score-timewise` format the music is organized first by measure, then by part —
/// the inverse of `score-partwise`.  It is rarely produced in practice but is valid
/// MusicXML 3.0 / 3.1 / 4.0.
///
/// ```xml
/// <score-timewise version="4.0">
///   <part-list> … </part-list>
///   <measure number="1">
///     <part id="P1"> … </part>
///   </measure>
/// </score-timewise>
/// ```
///
/// The MusicXML specification ships an official XSLT stylesheet to convert between
/// `score-partwise` and `score-timewise`; the musical content of every element is
/// otherwise identical.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "score-timewise")]
pub struct ScoreTimewise {
    /// MusicXML version string, e.g. `"4.0"`, `"3.1"`, `"3.0"`.
    #[serde(rename = "@version")]
    pub version: Option<String>,

    // --- header (identical to score-partwise) ---
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
    #[serde(rename = "measure", default)]
    pub measures: Vec<TimewiseMeasure>,
}

impl ScoreTimewise {
    /// Transpose this `score-timewise` document into the equivalent
    /// [`ScorePartwise`].
    ///
    /// `score-timewise` organizes music measure-first then part; `score-partwise`
    /// organizes part-first then measure. Both encode identical musical content —
    /// the MusicXML specification ships an official XSLT stylesheet to transpose
    /// between them. This is a pure restructuring of the same `musicxml` types.
    ///
    /// The header, defaults and part-list are moved across unchanged. The
    /// per-measure `<part>` fragments are regrouped into per-part `<measure>`
    /// sequences in part-list order (the partwise convention), preserving
    /// document order within each part. A part listed in `<part-list>` but absent
    /// from every measure yields an empty measure list; `<part>` fragments
    /// referencing an id not in the part-list are dropped.
    pub fn into_partwise(self) -> ScorePartwise {
        use std::collections::HashMap;

        let ScoreTimewise {
            version,
            work,
            movement_number,
            movement_title,
            identification,
            defaults,
            credits,
            part_list,
            measures,
        } = self;

        // Output part order follows the <part-list> (partwise convention).
        let part_ids: Vec<String> = part_list
            .items
            .iter()
            .filter_map(|item| match item {
                part_list::PartListItem::ScorePart(sp) => Some(sp.id.clone()),
                _ => None,
            })
            .collect();

        // Accumulate each part's measures in document order.
        let mut per_part: HashMap<String, Vec<measure::Measure>> = HashMap::new();
        for tw_measure in measures {
            let TimewiseMeasure {
                number,
                implicit,
                non_controlling,
                width,
                text,
                id,
                parts,
            } = tw_measure;
            for tw_part in parts {
                let measure = measure::Measure {
                    number: number.clone(),
                    implicit: implicit.clone(),
                    non_controlling: non_controlling.clone(),
                    width,
                    text: text.clone(),
                    id: id.clone(),
                    music_data: tw_part.music_data,
                };
                per_part.entry(tw_part.id).or_default().push(measure);
            }
        }

        let parts = part_ids
            .into_iter()
            .map(|part_id| {
                let measures = per_part.remove(&part_id).unwrap_or_default();
                Part {
                    id: part_id,
                    measures,
                }
            })
            .collect();

        ScorePartwise {
            version,
            work,
            movement_number,
            movement_title,
            identification,
            defaults,
            credits,
            part_list,
            parts,
        }
    }
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
#[derive(Debug, Serialize, Deserialize)]
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
