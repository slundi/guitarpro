use serde::Deserialize;

use super::note::{DynamicMark, FormattedText, Level, OtherPlacement, WavyLine};

/// A word/rehearsal/tempo text block.
#[derive(Debug, Deserialize)]
pub struct Words {
    #[serde(rename = "@justify")]
    pub justify: Option<String>,
    #[serde(rename = "@valign")]
    pub valign: Option<String>,
    #[serde(rename = "@font-family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight")]
    pub font_weight: Option<String>,
    #[serde(rename = "@font-style")]
    pub font_style: Option<String>,
    #[serde(rename = "@letter-spacing")]
    pub letter_spacing: Option<String>,
    #[serde(rename = "@xml:lang")]
    pub lang: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A rehearsal mark (letter, number, or text in a box/circle).
#[derive(Debug, Deserialize)]
pub struct Rehearsal {
    #[serde(rename = "@justify")]
    pub justify: Option<String>,
    #[serde(rename = "@enclosure")]
    pub enclosure: Option<String>,
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight")]
    pub font_weight: Option<String>,
    #[serde(rename = "@xml:lang")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Dynamic marking (`<p>`, `<ff>`, `<sfz>`, etc.) inside a `<direction>`.
#[derive(Debug, Deserialize)]
pub struct Dynamics {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@halign")]
    pub halign: Option<String>,
    #[serde(rename = "@valign")]
    pub valign: Option<String>,
    #[serde(rename = "@enclosure")]
    pub enclosure: Option<String>,
    #[serde(rename = "$value", default)]
    pub marks: Vec<DynamicMark>,
}

/// A hairpin crescendo or decrescendo.
#[derive(Debug, Deserialize)]
pub struct Wedge {
    /// `"crescendo"`, `"diminuendo"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub wedge_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@spread")]
    pub spread: Option<f64>,
    #[serde(rename = "@niente")]
    pub niente: Option<String>,
    #[serde(rename = "@line-type")]
    pub line_type: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
}

/// A dashed or solid line bracket.
#[derive(Debug, Deserialize)]
pub struct Bracket {
    /// `"start"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub bracket_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    /// `"up"`, `"down"`, `"both"`, `"none"`.
    #[serde(rename = "@line-end")]
    pub line_end: String,
    #[serde(rename = "@end-length")]
    pub end_length: Option<f64>,
    #[serde(rename = "@line-type")]
    pub line_type: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
}

/// A pedal mark (piano sustain pedal).
#[derive(Debug, Deserialize)]
pub struct Pedal {
    /// `"start"`, `"stop"`, `"sostenuto"`, `"change"`, `"continue"`, `"discontinue"`, `"resume"`.
    #[serde(rename = "@type")]
    pub pedal_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    /// `"yes"` to use a *Ped.* glyph, `"no"` for a bracket line.
    #[serde(rename = "@line")]
    pub line: Option<String>,
    #[serde(rename = "@sign")]
    pub sign: Option<String>,
    #[serde(rename = "@abbreviated")]
    pub abbreviated: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
}

/// An octave shift (8va, 15ma, etc.).
#[derive(Debug, Deserialize)]
pub struct OctaveShift {
    /// `"up"`, `"down"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub shift_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    /// `8` for 8va/8vb, `15` for 15ma/15mb, `22` for 22ma/22mb.
    #[serde(rename = "@size")]
    pub size: Option<u8>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@dash-length")]
    pub dash_length: Option<f64>,
    #[serde(rename = "@space-length")]
    pub space_length: Option<f64>,
}

/// A dashed or dotted line above or below the staff.
#[derive(Debug, Deserialize)]
pub struct Dashes {
    /// `"start"`, `"stop"`, `"continue"`.
    #[serde(rename = "@type")]
    pub dashes_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@dash-length")]
    pub dash_length: Option<f64>,
    #[serde(rename = "@space-length")]
    pub space_length: Option<f64>,
}

/// Metronome marking.
#[derive(Debug, Deserialize)]
pub struct Metronome {
    #[serde(rename = "@parentheses")]
    pub parentheses: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@justify")]
    pub justify: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    // Simple BPM-style metronome
    #[serde(rename = "beat-unit")]
    pub beat_unit: Option<String>,
    #[serde(rename = "beat-unit-dot", default)]
    pub beat_unit_dots: Vec<()>,
    #[serde(rename = "beat-unit-tied")]
    pub beat_unit_tied: Option<BeatUnitTied>,
    /// Beats per minute.
    #[serde(rename = "per-minute")]
    pub per_minute: Option<PerMinute>,
    // Metric modulation style
    #[serde(rename = "metronome-arrows")]
    pub metronome_arrows: Option<()>,
    #[serde(rename = "metronome-note", default)]
    pub metronome_notes: Vec<MetronomeNote>,
    #[serde(rename = "metronome-relation")]
    pub metronome_relation: Option<String>,
}

/// BPM value in a metronome marking.
#[derive(Debug, Deserialize)]
pub struct PerMinute {
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A tied beat-unit in a complex metronome marking.
#[derive(Debug, Deserialize)]
pub struct BeatUnitTied {
    #[serde(rename = "beat-unit")]
    pub beat_unit: String,
    #[serde(rename = "beat-unit-dot", default)]
    pub beat_unit_dots: Vec<()>,
}

/// A note value in a metric modulation metronome.
#[derive(Debug, Deserialize)]
pub struct MetronomeNote {
    #[serde(rename = "metronome-type")]
    pub metronome_type: String,
    #[serde(rename = "metronome-dot", default)]
    pub metronome_dots: Vec<()>,
    #[serde(rename = "metronome-beam", default)]
    pub metronome_beams: Vec<String>,
    #[serde(rename = "metronome-tied")]
    pub metronome_tied: Option<()>,
    #[serde(rename = "metronome-tuplet")]
    pub metronome_tuplet: Option<MetronomeTuplet>,
}

/// Tuplet specification inside a metronome note.
#[derive(Debug, Deserialize)]
pub struct MetronomeTuplet {
    #[serde(rename = "@type")]
    pub tuplet_type: String,
    #[serde(rename = "@bracket")]
    pub bracket: Option<String>,
    #[serde(rename = "@show-number")]
    pub show_number: Option<String>,
    #[serde(rename = "actual-notes")]
    pub actual_notes: u8,
    #[serde(rename = "normal-notes")]
    pub normal_notes: u8,
    #[serde(rename = "normal-type")]
    pub normal_type: Option<String>,
    #[serde(rename = "normal-dot", default)]
    pub normal_dots: Vec<()>,
}

/// Sound/playback level adjustment.
#[derive(Debug, Deserialize)]
pub struct Sound {
    #[serde(rename = "@tempo")]
    pub tempo: Option<f64>,
    #[serde(rename = "@dynamics")]
    pub dynamics: Option<f64>,
    #[serde(rename = "@dacapo")]
    pub dacapo: Option<String>,
    #[serde(rename = "@segno")]
    pub segno: Option<String>,
    #[serde(rename = "@dalsegno")]
    pub dalsegno: Option<String>,
    #[serde(rename = "@coda")]
    pub coda: Option<String>,
    #[serde(rename = "@tocoda")]
    pub tocoda: Option<String>,
    #[serde(rename = "@divisions")]
    pub divisions: Option<f64>,
    #[serde(rename = "@forward-repeat")]
    pub forward_repeat: Option<String>,
    #[serde(rename = "@fine")]
    pub fine: Option<String>,
    #[serde(rename = "@time-only")]
    pub time_only: Option<String>,
    #[serde(rename = "@pizzicato")]
    pub pizzicato: Option<String>,
    #[serde(rename = "@pan")]
    pub pan: Option<f64>,
    #[serde(rename = "@elevation")]
    pub elevation: Option<f64>,
    #[serde(rename = "@damper-pedal")]
    pub damper_pedal: Option<String>,
    #[serde(rename = "@soft-pedal")]
    pub soft_pedal: Option<String>,
    #[serde(rename = "@sostenuto-pedal")]
    pub sostenuto_pedal: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "midi-device", default)]
    pub midi_devices: Vec<super::part_list::MidiDevice>,
    #[serde(rename = "midi-instrument", default)]
    pub midi_instruments: Vec<super::part_list::MidiInstrument>,
    #[serde(rename = "play", default)]
    pub plays: Vec<super::note::Play>,
    #[serde(rename = "swing")]
    pub swing: Option<Swing>,
    #[serde(rename = "offset")]
    pub offset: Option<Offset>,
}

/// Swing style (straight, jazz, etc.).
#[derive(Debug, Deserialize)]
pub struct Swing {
    /// `"yes"` for straight (no swing) or `"no"` for swung.
    pub straight: Option<()>,
    pub first: Option<u8>,
    pub second: Option<u8>,
    #[serde(rename = "swing-type")]
    pub swing_type: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
}

/// A time offset in divisions for a direction or note.
#[derive(Debug, Deserialize)]
pub struct Offset {
    #[serde(rename = "@sound")]
    pub sound: Option<String>,
    #[serde(rename = "$text")]
    pub value: f64,
}

/// A segno or coda symbol.
#[derive(Debug, Deserialize)]
pub struct Segno {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
}

/// A string-instrument scordatura (retuning) direction.
#[derive(Debug, Deserialize)]
pub struct Scordatura {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "accord", default)]
    pub accords: Vec<Accord>,
}

/// One string's retuning in a scordatura.
#[derive(Debug, Deserialize)]
pub struct Accord {
    #[serde(rename = "@string")]
    pub string: u8,
    #[serde(rename = "tuning-step")]
    pub tuning_step: String,
    #[serde(rename = "tuning-alter")]
    pub tuning_alter: Option<f64>,
    #[serde(rename = "tuning-octave")]
    pub tuning_octave: i8,
}

/// Image direction (for embedded images in the score).
#[derive(Debug, Deserialize)]
pub struct Image {
    #[serde(rename = "@source")]
    pub source: String,
    #[serde(rename = "@type")]
    pub image_type: String,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@halign")]
    pub halign: Option<String>,
    #[serde(rename = "@valign")]
    pub valign: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
}

/// Principal voice / cue voice direction.
#[derive(Debug, Deserialize)]
pub struct PrincipalVoice {
    #[serde(rename = "@type")]
    pub voice_type: String,
    #[serde(rename = "@symbol")]
    pub symbol: String,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// Percussion sticking direction.
#[derive(Debug, Deserialize)]
pub struct Sticking {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Harp pedal diagram direction.
#[derive(Debug, Deserialize)]
pub struct HarpPedals {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "pedal-tuning", default)]
    pub pedal_tunings: Vec<PedalTuning>,
}

/// One harp pedal setting.
#[derive(Debug, Deserialize)]
pub struct PedalTuning {
    #[serde(rename = "pedal-step")]
    pub pedal_step: String,
    #[serde(rename = "pedal-alter")]
    pub pedal_alter: f64,
}

/// A damp marking (string dampening).
#[derive(Debug, Deserialize)]
pub struct Damp {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
}

/// One content item inside a `<direction-type>`.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct DirectionTypeWrapper {
    #[serde(rename = "$value")]
    pub content: DirectionType,
}

/// Percussion notation direction.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct AccordionRegistration {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "accordion-high")]
    pub accordion_high: Option<()>,
    #[serde(rename = "accordion-middle")]
    pub accordion_middle: Option<u8>,
    #[serde(rename = "accordion-low")]
    pub accordion_low: Option<()>,
}

/// A musical direction (text, dynamic, metronome, pedal, etc.) above or below the staff.
///
/// Maps to the `<direction>` element.
#[derive(Debug, Deserialize)]
pub struct Direction {
    /// `"up"` (above staff) or `"down"` (below staff).
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    /// Which staff this direction belongs to (multi-staff parts).
    #[serde(rename = "@directive")]
    pub directive: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "direction-type")]
    pub direction_types: Vec<DirectionTypeWrapper>,
    pub offset: Option<Offset>,
    pub footnote: Option<FormattedText>,
    pub level: Option<Level>,
    pub voice: Option<String>,
    pub staff: Option<u8>,
    pub sound: Option<Sound>,
    pub listening: Option<Listening>,
    // Wavy-line is occasionally a direct child of direction (rare; normally in notations)
    #[serde(rename = "wavy-line")]
    pub wavy_line: Option<WavyLine>,
    #[serde(rename = "dashes")]
    pub dashes: Option<Dashes>,
    #[serde(rename = "bracket")]
    pub bracket: Option<Bracket>,
    #[serde(rename = "pedal")]
    pub pedal: Option<Pedal>,
}

/// Listening/synchronization block inside a direction.
#[derive(Debug, Deserialize)]
pub struct Listening {
    #[serde(rename = "$value", default)]
    pub content: Vec<super::note::ListenContent>,
}
