use serde::{Deserialize, Serialize};

use super::note::{DynamicMark, FormattedText, Level, OtherPlacement, WavyLine};

/// A word/rehearsal/tempo text block.
#[derive(Debug, Serialize, Deserialize)]
pub struct Words {
    #[serde(rename = "@justify", skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    #[serde(rename = "@valign", skip_serializing_if = "Option::is_none")]
    pub valign: Option<String>,
    #[serde(rename = "@font-family", skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight", skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(rename = "@font-style", skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(rename = "@letter-spacing", skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A rehearsal mark (letter, number, or text in a box/circle).
#[derive(Debug, Serialize, Deserialize)]
pub struct Rehearsal {
    #[serde(rename = "@justify", skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    #[serde(rename = "@enclosure", skip_serializing_if = "Option::is_none")]
    pub enclosure: Option<String>,
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight", skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Dynamic marking (`<p>`, `<ff>`, `<sfz>`, etc.) inside a `<direction>`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dynamics {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@halign", skip_serializing_if = "Option::is_none")]
    pub halign: Option<String>,
    #[serde(rename = "@valign", skip_serializing_if = "Option::is_none")]
    pub valign: Option<String>,
    #[serde(rename = "@enclosure", skip_serializing_if = "Option::is_none")]
    pub enclosure: Option<String>,
    #[serde(rename = "$value", default)]
    pub marks: Vec<DynamicMark>,
}

/// A hairpin crescendo or decrescendo.
#[derive(Debug, Serialize, Deserialize)]
pub struct Wedge {
    /// `"crescendo"`, `"diminuendo"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub wedge_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@spread", skip_serializing_if = "Option::is_none")]
    pub spread: Option<f64>,
    #[serde(rename = "@niente", skip_serializing_if = "Option::is_none")]
    pub niente: Option<String>,
    #[serde(rename = "@line-type", skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A dashed or solid line bracket.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bracket {
    /// `"start"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub bracket_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    /// `"up"`, `"down"`, `"both"`, `"none"`.
    #[serde(rename = "@line-end")]
    pub line_end: String,
    #[serde(rename = "@end-length", skip_serializing_if = "Option::is_none")]
    pub end_length: Option<f64>,
    #[serde(rename = "@line-type", skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A pedal mark (piano sustain pedal).
#[derive(Debug, Serialize, Deserialize)]
pub struct Pedal {
    /// `"start"`, `"stop"`, `"sostenuto"`, `"change"`, `"continue"`, `"discontinue"`, `"resume"`.
    #[serde(rename = "@type")]
    pub pedal_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    /// `"yes"` to use a *Ped.* glyph, `"no"` for a bracket line.
    #[serde(rename = "@line", skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    #[serde(rename = "@sign", skip_serializing_if = "Option::is_none")]
    pub sign: Option<String>,
    #[serde(rename = "@abbreviated", skip_serializing_if = "Option::is_none")]
    pub abbreviated: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// An octave shift (8va, 15ma, etc.).
#[derive(Debug, Serialize, Deserialize)]
pub struct OctaveShift {
    /// `"up"`, `"down"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub shift_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    /// `8` for 8va/8vb, `15` for 15ma/15mb, `22` for 22ma/22mb.
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<u8>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@dash-length", skip_serializing_if = "Option::is_none")]
    pub dash_length: Option<f64>,
    #[serde(rename = "@space-length", skip_serializing_if = "Option::is_none")]
    pub space_length: Option<f64>,
}

/// A dashed or dotted line above or below the staff.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dashes {
    /// `"start"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub dashes_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@dash-length", skip_serializing_if = "Option::is_none")]
    pub dash_length: Option<f64>,
    #[serde(rename = "@space-length", skip_serializing_if = "Option::is_none")]
    pub space_length: Option<f64>,
}

/// Metronome marking.
#[derive(Debug, Serialize, Deserialize)]
pub struct Metronome {
    #[serde(rename = "@parentheses", skip_serializing_if = "Option::is_none")]
    pub parentheses: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@justify", skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    // Simple BPM-style metronome
    #[serde(rename = "beat-unit", skip_serializing_if = "Option::is_none")]
    pub beat_unit: Option<String>,
    #[serde(rename = "beat-unit-dot", default)]
    pub beat_unit_dots: Vec<()>,
    #[serde(rename = "beat-unit-tied", skip_serializing_if = "Option::is_none")]
    pub beat_unit_tied: Option<BeatUnitTied>,
    /// Beats per minute.
    #[serde(rename = "per-minute", skip_serializing_if = "Option::is_none")]
    pub per_minute: Option<PerMinute>,
    // Metric modulation style
    #[serde(rename = "metronome-arrows", skip_serializing_if = "Option::is_none")]
    pub metronome_arrows: Option<()>,
    #[serde(rename = "metronome-note", default)]
    pub metronome_notes: Vec<MetronomeNote>,
    #[serde(rename = "metronome-relation", skip_serializing_if = "Option::is_none")]
    pub metronome_relation: Option<String>,
}

/// BPM value in a metronome marking.
#[derive(Debug, Serialize, Deserialize)]
pub struct PerMinute {
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A tied beat-unit in a complex metronome marking.
#[derive(Debug, Serialize, Deserialize)]
pub struct BeatUnitTied {
    #[serde(rename = "beat-unit")]
    pub beat_unit: String,
    #[serde(rename = "beat-unit-dot", default)]
    pub beat_unit_dots: Vec<()>,
}

/// A note value in a metric modulation metronome.
#[derive(Debug, Serialize, Deserialize)]
pub struct MetronomeNote {
    #[serde(rename = "metronome-type")]
    pub metronome_type: String,
    #[serde(rename = "metronome-dot", default)]
    pub metronome_dots: Vec<()>,
    #[serde(rename = "metronome-beam", default)]
    pub metronome_beams: Vec<String>,
    #[serde(rename = "metronome-tied", skip_serializing_if = "Option::is_none")]
    pub metronome_tied: Option<()>,
    #[serde(rename = "metronome-tuplet", skip_serializing_if = "Option::is_none")]
    pub metronome_tuplet: Option<MetronomeTuplet>,
}

/// Tuplet specification inside a metronome note.
#[derive(Debug, Serialize, Deserialize)]
pub struct MetronomeTuplet {
    #[serde(rename = "@type")]
    pub tuplet_type: String,
    #[serde(rename = "@bracket", skip_serializing_if = "Option::is_none")]
    pub bracket: Option<String>,
    #[serde(rename = "@show-number", skip_serializing_if = "Option::is_none")]
    pub show_number: Option<String>,
    #[serde(rename = "actual-notes")]
    pub actual_notes: u8,
    #[serde(rename = "normal-notes")]
    pub normal_notes: u8,
    #[serde(rename = "normal-type", skip_serializing_if = "Option::is_none")]
    pub normal_type: Option<String>,
    #[serde(rename = "normal-dot", default)]
    pub normal_dots: Vec<()>,
}

/// Sound/playback level adjustment.
#[derive(Debug, Serialize, Deserialize)]
pub struct Sound {
    #[serde(rename = "@tempo", skip_serializing_if = "Option::is_none")]
    pub tempo: Option<f64>,
    #[serde(rename = "@dynamics", skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<f64>,
    #[serde(rename = "@dacapo", skip_serializing_if = "Option::is_none")]
    pub dacapo: Option<String>,
    #[serde(rename = "@segno", skip_serializing_if = "Option::is_none")]
    pub segno: Option<String>,
    #[serde(rename = "@dalsegno", skip_serializing_if = "Option::is_none")]
    pub dalsegno: Option<String>,
    #[serde(rename = "@coda", skip_serializing_if = "Option::is_none")]
    pub coda: Option<String>,
    #[serde(rename = "@tocoda", skip_serializing_if = "Option::is_none")]
    pub tocoda: Option<String>,
    #[serde(rename = "@divisions", skip_serializing_if = "Option::is_none")]
    pub divisions: Option<f64>,
    #[serde(rename = "@forward-repeat", skip_serializing_if = "Option::is_none")]
    pub forward_repeat: Option<String>,
    #[serde(rename = "@fine", skip_serializing_if = "Option::is_none")]
    pub fine: Option<String>,
    #[serde(rename = "@time-only", skip_serializing_if = "Option::is_none")]
    pub time_only: Option<String>,
    #[serde(rename = "@pizzicato", skip_serializing_if = "Option::is_none")]
    pub pizzicato: Option<String>,
    #[serde(rename = "@pan", skip_serializing_if = "Option::is_none")]
    pub pan: Option<f64>,
    #[serde(rename = "@elevation", skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f64>,
    #[serde(rename = "@damper-pedal", skip_serializing_if = "Option::is_none")]
    pub damper_pedal: Option<String>,
    #[serde(rename = "@soft-pedal", skip_serializing_if = "Option::is_none")]
    pub soft_pedal: Option<String>,
    #[serde(rename = "@sostenuto-pedal", skip_serializing_if = "Option::is_none")]
    pub sostenuto_pedal: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "midi-device", default)]
    pub midi_devices: Vec<super::part_list::MidiDevice>,
    #[serde(rename = "midi-instrument", default)]
    pub midi_instruments: Vec<super::part_list::MidiInstrument>,
    #[serde(rename = "play", default)]
    pub plays: Vec<super::note::Play>,
    #[serde(rename = "swing", skip_serializing_if = "Option::is_none")]
    pub swing: Option<Swing>,
    #[serde(rename = "offset", skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,
}

/// Swing style (straight, jazz, etc.).
#[derive(Debug, Serialize, Deserialize)]
pub struct Swing {
    /// `"yes"` for straight (no swing) or `"no"` for swung.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub straight: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<u8>,
    #[serde(rename = "swing-type", skip_serializing_if = "Option::is_none")]
    pub swing_type: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A time offset in divisions for a direction or note.
#[derive(Debug, Serialize, Deserialize)]
pub struct Offset {
    #[serde(rename = "@sound", skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(rename = "$text")]
    pub value: f64,
}

/// A segno or coda symbol.
#[derive(Debug, Serialize, Deserialize)]
pub struct Segno {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
}

/// A string-instrument scordatura (retuning) direction.
#[derive(Debug, Serialize, Deserialize)]
pub struct Scordatura {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "accord", default)]
    pub accords: Vec<Accord>,
}

/// One string's retuning in a scordatura.
#[derive(Debug, Serialize, Deserialize)]
pub struct Accord {
    #[serde(rename = "@string")]
    pub string: u8,
    #[serde(rename = "tuning-step")]
    pub tuning_step: String,
    #[serde(rename = "tuning-alter", skip_serializing_if = "Option::is_none")]
    pub tuning_alter: Option<f64>,
    #[serde(rename = "tuning-octave")]
    pub tuning_octave: i8,
}

/// Image direction (for embedded images in the score).
#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "@source")]
    pub source: String,
    #[serde(rename = "@type")]
    pub image_type: String,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@halign", skip_serializing_if = "Option::is_none")]
    pub halign: Option<String>,
    #[serde(rename = "@valign", skip_serializing_if = "Option::is_none")]
    pub valign: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Principal voice / cue voice direction.
#[derive(Debug, Serialize, Deserialize)]
pub struct PrincipalVoice {
    #[serde(rename = "@type")]
    pub voice_type: String,
    #[serde(rename = "@symbol")]
    pub symbol: String,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Percussion sticking direction.
#[derive(Debug, Serialize, Deserialize)]
pub struct Sticking {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Harp pedal diagram direction.
#[derive(Debug, Serialize, Deserialize)]
pub struct HarpPedals {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "pedal-tuning", default)]
    pub pedal_tunings: Vec<PedalTuning>,
}

/// One harp pedal setting.
#[derive(Debug, Serialize, Deserialize)]
pub struct PedalTuning {
    #[serde(rename = "pedal-step")]
    pub pedal_step: String,
    #[serde(rename = "pedal-alter")]
    pub pedal_alter: f64,
}

/// A damp marking (string dampening).
#[derive(Debug, Serialize, Deserialize)]
pub struct Damp {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// One content item inside a `<direction-type>`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DirectionType {
    Rehearsal(Rehearsal),
    Segno(Segno),
    Coda(Segno),
    Words(Words),
    Symbol(FormattedText),
    Wedge(Wedge),
    Dynamics(Dynamics),
    Dashes(Dashes),
    Bracket(Bracket),
    Pedal(Pedal),
    Metronome(Metronome),
    OctaveShift(OctaveShift),
    HarpPedals(HarpPedals),
    Damp(Damp),
    DampAll(Damp),
    Eyeglasses(()),
    StringMute(OtherPlacement),
    Scordatura(Scordatura),
    Image(Image),
    PrincipalVoice(PrincipalVoice),
    Percussion(Vec<PercussionContent>),
    AccordionRegistration(AccordionRegistration),
    Sticking(Sticking),
    #[serde(rename = "other-direction")]
    OtherDirection(OtherPlacement),
}

/// Container for multiple direction types (the `<direction-type>` wrapper element).
///
/// A single `<direction>` may contain multiple `<direction-type>` elements.
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectionTypeWrapper {
    #[serde(rename = "$value")]
    pub content: DirectionType,
}

/// Percussion notation direction.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PercussionContent {
    Glass(OtherPlacement),
    Metal(OtherPlacement),
    Wood(OtherPlacement),
    Pitched(OtherPlacement),
    Membrane(OtherPlacement),
    Effect(OtherPlacement),
    Timpani(OtherPlacement),
    Beater(OtherPlacement),
    Stick(OtherPlacement),
    StickLocation(String),
    OtherPercussion(OtherPlacement),
}

/// Accordion registration (high, middle, low reed combinations).
#[derive(Debug, Serialize, Deserialize)]
pub struct AccordionRegistration {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "accordion-high", skip_serializing_if = "Option::is_none")]
    pub accordion_high: Option<()>,
    #[serde(rename = "accordion-middle", skip_serializing_if = "Option::is_none")]
    pub accordion_middle: Option<u8>,
    #[serde(rename = "accordion-low", skip_serializing_if = "Option::is_none")]
    pub accordion_low: Option<()>,
}

/// A musical direction (text, dynamic, metronome, pedal, etc.) above or below the staff.
///
/// Maps to the `<direction>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Direction {
    /// `"up"` (above staff) or `"down"` (below staff).
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    /// Which staff this direction belongs to (multi-staff parts).
    #[serde(rename = "@directive", skip_serializing_if = "Option::is_none")]
    pub directive: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "direction-type")]
    pub direction_types: Vec<DirectionTypeWrapper>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnote: Option<FormattedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<Sound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listening: Option<Listening>,
    // Wavy-line is occasionally a direct child of direction (rare; normally in notations)
    #[serde(rename = "wavy-line", skip_serializing_if = "Option::is_none")]
    pub wavy_line: Option<WavyLine>,
    #[serde(rename = "dashes", skip_serializing_if = "Option::is_none")]
    pub dashes: Option<Dashes>,
    #[serde(rename = "bracket", skip_serializing_if = "Option::is_none")]
    pub bracket: Option<Bracket>,
    #[serde(rename = "pedal", skip_serializing_if = "Option::is_none")]
    pub pedal: Option<Pedal>,
}

/// Listening/synchronization block inside a direction.
#[derive(Debug, Serialize, Deserialize)]
pub struct Listening {
    #[serde(rename = "$value", default)]
    pub content: Vec<super::note::ListenContent>,
}
