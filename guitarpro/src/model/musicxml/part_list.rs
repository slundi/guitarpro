use serde::{Deserialize, Serialize};

/// A virtual instrument assigned to a part.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreInstrument {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "instrument-name")]
    pub instrument_name: String,
    #[serde(rename = "instrument-abbreviation")]
    pub instrument_abbreviation: Option<String>,
    #[serde(rename = "instrument-sound")]
    pub instrument_sound: Option<String>,
    /// `"yes"` if this is a solo instrument.
    pub solo: Option<()>,
    /// `"yes"` if this is an ensemble instrument.
    pub ensemble: Option<String>,
}

/// MIDI device assignment for a score instrument.
#[derive(Debug, Serialize, Deserialize)]
pub struct MidiDevice {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@port")]
    pub port: Option<u8>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// MIDI playback parameters for a score instrument.
#[derive(Debug, Serialize, Deserialize)]
pub struct MidiInstrument {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "midi-channel")]
    pub midi_channel: Option<u8>,
    #[serde(rename = "midi-name")]
    pub midi_name: Option<String>,
    #[serde(rename = "midi-bank")]
    pub midi_bank: Option<u16>,
    #[serde(rename = "midi-program")]
    pub midi_program: Option<u8>,
    #[serde(rename = "midi-unpitched")]
    pub midi_unpitched: Option<u8>,
    pub volume: Option<f64>,
    pub pan: Option<f64>,
    pub elevation: Option<f64>,
}

/// A single part (instrument) in the score.
///
/// Maps to `<score-part>` inside `<part-list>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScorePart {
    #[serde(rename = "@id")]
    pub id: String,
    pub identification: Option<super::identification::Identification>,
    #[serde(rename = "part-name")]
    pub part_name: Option<PartName>,
    #[serde(rename = "part-name-display")]
    pub part_name_display: Option<NameDisplay>,
    #[serde(rename = "part-abbreviation")]
    pub part_abbreviation: Option<PartName>,
    #[serde(rename = "part-abbreviation-display")]
    pub part_abbreviation_display: Option<NameDisplay>,
    #[serde(rename = "group", default)]
    pub groups: Vec<String>,
    #[serde(rename = "score-instrument", default)]
    pub score_instruments: Vec<ScoreInstrument>,
    #[serde(rename = "player", default)]
    pub players: Vec<Player>,
    #[serde(rename = "midi-device", default)]
    pub midi_devices: Vec<MidiDevice>,
    #[serde(rename = "midi-instrument", default)]
    pub midi_instruments: Vec<MidiInstrument>,
}

/// The display name of a part.
#[derive(Debug, Serialize, Deserialize)]
pub struct PartName {
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "@justify")]
    pub justify: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// Controls how a name is displayed (overrides the plain text name).
#[derive(Debug, Serialize, Deserialize)]
pub struct NameDisplay {
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "$value", default)]
    pub content: Vec<NameDisplayContent>,
}

/// Content inside a name display: plain text or accidental text.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameDisplayContent {
    DisplayText(FormattedText),
    AccidentalText(AccidentalText),
}

/// A text run with optional formatting.
#[derive(Debug, Serialize, Deserialize)]
pub struct FormattedText {
    #[serde(rename = "@font-family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight")]
    pub font_weight: Option<String>,
    #[serde(rename = "@font-style")]
    pub font_style: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An accidental glyph used inside a formatted name.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccidentalText {
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A named human performer associated with a part.
#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "player-name")]
    pub player_name: String,
}

/// Bracket/brace grouping around two or more parts.
///
/// Maps to `<part-group>` inside `<part-list>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PartGroup {
    /// `"start"` or `"stop"`.
    #[serde(rename = "@type")]
    pub group_type: String,
    #[serde(rename = "@number")]
    pub number: Option<String>,
    #[serde(rename = "group-name")]
    pub group_name: Option<PartName>,
    #[serde(rename = "group-name-display")]
    pub group_name_display: Option<NameDisplay>,
    #[serde(rename = "group-abbreviation")]
    pub group_abbreviation: Option<PartName>,
    #[serde(rename = "group-abbreviation-display")]
    pub group_abbreviation_display: Option<NameDisplay>,
    /// `"none"`, `"brace"`, `"line"`, `"bracket"`, `"square"`.
    #[serde(rename = "group-symbol")]
    pub group_symbol: Option<GroupSymbol>,
    #[serde(rename = "group-barline")]
    pub group_barline: Option<GroupBarline>,
    #[serde(rename = "group-time")]
    pub group_time: Option<()>,
}

/// Symbol drawn to the left of a part group.
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupSymbol {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Whether barlines are shared across a part group.
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupBarline {
    #[serde(rename = "@color")]
    pub color: Option<String>,
    /// `"yes"`, `"no"`, or `"Mensurstrich"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// One item inside `<part-list>`: either a `<score-part>` or a `<part-group>`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PartListItem {
    ScorePart(ScorePart),
    PartGroup(PartGroup),
}

/// The ordered list of all parts and their groupings.
///
/// Maps to the `<part-list>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct PartList {
    #[serde(rename = "$value")]
    pub items: Vec<PartListItem>,
}
