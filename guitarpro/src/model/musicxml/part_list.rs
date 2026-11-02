use serde::{Deserialize, Serialize};

/// A virtual instrument assigned to a part.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreInstrument {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "instrument-name")]
    pub instrument_name: String,
    #[serde(
        rename = "instrument-abbreviation",
        skip_serializing_if = "Option::is_none"
    )]
    pub instrument_abbreviation: Option<String>,
    #[serde(rename = "instrument-sound", skip_serializing_if = "Option::is_none")]
    pub instrument_sound: Option<String>,
    /// `"yes"` if this is a solo instrument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solo: Option<()>,
    /// `"yes"` if this is an ensemble instrument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensemble: Option<String>,
}

/// MIDI device assignment for a score instrument.
#[derive(Debug, Serialize, Deserialize)]
pub struct MidiDevice {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@port", skip_serializing_if = "Option::is_none")]
    pub port: Option<u8>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// MIDI playback parameters for a score instrument.
#[derive(Debug, Serialize, Deserialize)]
pub struct MidiInstrument {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "midi-channel", skip_serializing_if = "Option::is_none")]
    pub midi_channel: Option<u8>,
    #[serde(rename = "midi-name", skip_serializing_if = "Option::is_none")]
    pub midi_name: Option<String>,
    #[serde(rename = "midi-bank", skip_serializing_if = "Option::is_none")]
    pub midi_bank: Option<u16>,
    #[serde(rename = "midi-program", skip_serializing_if = "Option::is_none")]
    pub midi_program: Option<u8>,
    #[serde(rename = "midi-unpitched", skip_serializing_if = "Option::is_none")]
    pub midi_unpitched: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pan: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f64>,
}

/// A single part (instrument) in the score.
///
/// Maps to `<score-part>` inside `<part-list>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScorePart {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identification: Option<super::identification::Identification>,
    #[serde(rename = "part-name", skip_serializing_if = "Option::is_none")]
    pub part_name: Option<PartName>,
    #[serde(rename = "part-name-display", skip_serializing_if = "Option::is_none")]
    pub part_name_display: Option<NameDisplay>,
    #[serde(rename = "part-abbreviation", skip_serializing_if = "Option::is_none")]
    pub part_abbreviation: Option<PartName>,
    #[serde(
        rename = "part-abbreviation-display",
        skip_serializing_if = "Option::is_none"
    )]
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
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "@justify", skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Controls how a name is displayed (overrides the plain text name).
#[derive(Debug, Serialize, Deserialize)]
pub struct NameDisplay {
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "@font-family", skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight", skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(rename = "@font-style", skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An accidental glyph used inside a formatted name.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccidentalText {
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(rename = "group-name", skip_serializing_if = "Option::is_none")]
    pub group_name: Option<PartName>,
    #[serde(rename = "group-name-display", skip_serializing_if = "Option::is_none")]
    pub group_name_display: Option<NameDisplay>,
    #[serde(rename = "group-abbreviation", skip_serializing_if = "Option::is_none")]
    pub group_abbreviation: Option<PartName>,
    #[serde(
        rename = "group-abbreviation-display",
        skip_serializing_if = "Option::is_none"
    )]
    pub group_abbreviation_display: Option<NameDisplay>,
    /// `"none"`, `"brace"`, `"line"`, `"bracket"`, `"square"`.
    #[serde(rename = "group-symbol", skip_serializing_if = "Option::is_none")]
    pub group_symbol: Option<GroupSymbol>,
    #[serde(rename = "group-barline", skip_serializing_if = "Option::is_none")]
    pub group_barline: Option<GroupBarline>,
    #[serde(rename = "group-time", skip_serializing_if = "Option::is_none")]
    pub group_time: Option<()>,
}

/// Symbol drawn to the left of a part group.
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupSymbol {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Whether barlines are shared across a part group.
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupBarline {
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// `"yes"`, `"no"`, or `"Mensurstrich"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// One item inside `<part-list>`: either a `<score-part>` or a `<part-group>`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(clippy::large_enum_variant)]
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
