use serde::{Deserialize, Serialize};

/// The sounded pitch of a note.
#[derive(Debug, Serialize, Deserialize)]
pub struct Pitch {
    /// Pitch class: `A`–`G`.
    pub step: String,
    /// Semitone alteration: `1.0` = sharp, `-1.0` = flat, `0.5` = quarter-sharp, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alter: Option<f64>,
    /// Octave in scientific pitch notation (middle C = C4, octave 4).
    pub octave: i8,
}

/// Indicates this note is a rest.
#[derive(Debug, Serialize, Deserialize)]
pub struct Rest {
    /// When `"yes"`, this rest fills the whole measure regardless of duration.
    #[serde(rename = "@measure", skip_serializing_if = "Option::is_none")]
    pub measure: Option<String>,
    /// The displayed pitch position of the rest symbol (not the sounded pitch).
    #[serde(rename = "display-step", skip_serializing_if = "Option::is_none")]
    pub display_step: Option<String>,
    #[serde(rename = "display-octave", skip_serializing_if = "Option::is_none")]
    pub display_octave: Option<i8>,
}

/// Unpitched note head (for percussion).
#[derive(Debug, Serialize, Deserialize)]
pub struct Unpitched {
    #[serde(rename = "display-step", skip_serializing_if = "Option::is_none")]
    pub display_step: Option<String>,
    #[serde(rename = "display-octave", skip_serializing_if = "Option::is_none")]
    pub display_octave: Option<i8>,
}

/// The written note type value (rhythmic denominator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteTypeValue {
    #[serde(rename = "1024th")]
    N1024th,
    #[serde(rename = "512th")]
    N512th,
    #[serde(rename = "256th")]
    N256th,
    #[serde(rename = "128th")]
    N128th,
    #[serde(rename = "64th")]
    N64th,
    #[serde(rename = "32nd")]
    N32nd,
    #[serde(rename = "16th")]
    N16th,
    Eighth,
    Quarter,
    Half,
    Whole,
    Breve,
    Long,
    Maxima,
}

/// The written note type with optional size override.
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteType {
    /// `"full"`, `"cue"`, `"grace-cue"`, `"large"`.
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(rename = "$text")]
    pub value: NoteTypeValue,
}

/// An augmentation dot.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dot {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
}

/// Displayed accidental symbol.
#[derive(Debug, Serialize, Deserialize)]
pub struct Accidental {
    #[serde(rename = "@cautionary", skip_serializing_if = "Option::is_none")]
    pub cautionary: Option<String>,
    #[serde(rename = "@editorial", skip_serializing_if = "Option::is_none")]
    pub editorial: Option<String>,
    #[serde(rename = "@bracket", skip_serializing_if = "Option::is_none")]
    pub bracket: Option<String>,
    #[serde(rename = "@parentheses", skip_serializing_if = "Option::is_none")]
    pub parentheses: Option<String>,
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
    /// E.g. `"sharp"`, `"natural"`, `"flat"`, `"double-sharp"`, `"flat-flat"`, `"natural-sharp"`, etc.
    #[serde(rename = "$text")]
    pub value: String,
}

/// Time modification for a tuplet note (e.g. 3 notes in the time of 2).
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeModification {
    /// Actual number of notes in the tuplet group.
    #[serde(rename = "actual-notes")]
    pub actual_notes: u8,
    /// Normal number of notes the group takes the time of.
    #[serde(rename = "normal-notes")]
    pub normal_notes: u8,
    #[serde(rename = "normal-type", skip_serializing_if = "Option::is_none")]
    pub normal_type: Option<String>,
    #[serde(rename = "normal-dot", default)]
    pub normal_dots: Vec<()>,
}

/// Direction of a stem.
#[derive(Debug, Serialize, Deserialize)]
pub struct Stem {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@y-position", skip_serializing_if = "Option::is_none")]
    pub y_position: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// `"up"`, `"down"`, `"double"`, `"none"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// Note head shape.
#[derive(Debug, Serialize, Deserialize)]
pub struct Notehead {
    #[serde(rename = "@filled", skip_serializing_if = "Option::is_none")]
    pub filled: Option<String>,
    #[serde(rename = "@parentheses", skip_serializing_if = "Option::is_none")]
    pub parentheses: Option<String>,
    #[serde(rename = "@font-family", skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
    /// E.g. `"normal"`, `"diamond"`, `"x"`, `"circle-x"`, `"square"`, `"slash"`,
    /// `"triangle"`, `"arrow down"`, `"arrow up"`, `"circled"`, `"do"`, `"re"`, etc.
    #[serde(rename = "$text")]
    pub value: String,
}

/// Beam notation for a single beam level.
#[derive(Debug, Serialize, Deserialize)]
pub struct Beam {
    /// Beam level (1 = outer beam, up to 8).
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "@fan", skip_serializing_if = "Option::is_none")]
    pub fan: Option<String>,
    /// `"begin"`, `"continue"`, `"end"`, `"forward hook"`, `"backward hook"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// A tie notation mark on the note (separate from the logical `<tied>` in notations).
#[derive(Debug, Serialize, Deserialize)]
pub struct Tie {
    /// `"start"` or `"stop"`.
    #[serde(rename = "@type")]
    pub tie_type: String,
    #[serde(rename = "@time-only", skip_serializing_if = "Option::is_none")]
    pub time_only: Option<String>,
}

// ---------------------------------------------------------------------------
// Notations
// ---------------------------------------------------------------------------

/// A tie notation marking (inside `<notations>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Tied {
    #[serde(rename = "@type")]
    pub tied_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type", skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@orientation", skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,
}

/// A slur notation marking.
#[derive(Debug, Serialize, Deserialize)]
pub struct Slur {
    #[serde(rename = "@type")]
    pub slur_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type", skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@orientation", skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,
}

/// Tuplet bracket notation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tuplet {
    #[serde(rename = "@type")]
    pub tuplet_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@bracket", skip_serializing_if = "Option::is_none")]
    pub bracket: Option<String>,
    #[serde(rename = "@show-number", skip_serializing_if = "Option::is_none")]
    pub show_number: Option<String>,
    #[serde(rename = "@show-type", skip_serializing_if = "Option::is_none")]
    pub show_type: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "tuplet-actual", skip_serializing_if = "Option::is_none")]
    pub tuplet_actual: Option<TupletPortion>,
    #[serde(rename = "tuplet-normal", skip_serializing_if = "Option::is_none")]
    pub tuplet_normal: Option<TupletPortion>,
}

/// The actual or normal portion of a displayed tuplet ratio.
#[derive(Debug, Serialize, Deserialize)]
pub struct TupletPortion {
    #[serde(rename = "tuplet-number", skip_serializing_if = "Option::is_none")]
    pub tuplet_number: Option<TupletNumber>,
    #[serde(rename = "tuplet-type", skip_serializing_if = "Option::is_none")]
    pub tuplet_type: Option<String>,
    #[serde(rename = "tuplet-dot", default)]
    pub tuplet_dots: Vec<()>,
}

/// The number shown on a tuplet bracket.
#[derive(Debug, Serialize, Deserialize)]
pub struct TupletNumber {
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// A glissando (continuous-line gliss).
#[derive(Debug, Serialize, Deserialize)]
pub struct Glissando {
    #[serde(rename = "@type")]
    pub glissando_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type", skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A slide (solid line from one note to another).
#[derive(Debug, Serialize, Deserialize)]
pub struct Slide {
    #[serde(rename = "@type")]
    pub slide_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type", skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A fermata above or below a note.
#[derive(Debug, Serialize, Deserialize)]
pub struct Fermata {
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub fermata_type: Option<String>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    /// E.g. `"normal"`, `"angled"`, `"square"`, `"double-angled"`, `"double-square"`,
    /// `"double-dot"`, `"half-curve"`, `"curlew"`.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// An arpeggiate notation (rolled chord indicator).
#[derive(Debug, Serialize, Deserialize)]
pub struct Arpeggiate {
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@direction", skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(rename = "@unbroken", skip_serializing_if = "Option::is_none")]
    pub unbroken: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
}

/// Indicates a chord is not arpeggiated.
#[derive(Debug, Serialize, Deserialize)]
pub struct NonArpeggiate {
    #[serde(rename = "@type")]
    pub non_arpeggiate_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
}

/// An accidental in an ornament (e.g. trill accidental).
#[derive(Debug, Serialize, Deserialize)]
pub struct AccidentalMark {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An empty placement element (many ornaments / articulations have no attributes beyond placement).
#[derive(Debug, Serialize, Deserialize)]
pub struct PlacedEmpty {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
}

/// Trill, turn, mordent, and other ornament marks.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ornaments {
    #[serde(rename = "trill-mark", skip_serializing_if = "Option::is_none")]
    pub trill_mark: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<PlacedEmpty>,
    #[serde(rename = "delayed-turn", skip_serializing_if = "Option::is_none")]
    pub delayed_turn: Option<PlacedEmpty>,
    #[serde(rename = "inverted-turn", skip_serializing_if = "Option::is_none")]
    pub inverted_turn: Option<PlacedEmpty>,
    #[serde(
        rename = "delayed-inverted-turn",
        skip_serializing_if = "Option::is_none"
    )]
    pub delayed_inverted_turn: Option<PlacedEmpty>,
    #[serde(rename = "vertical-turn", skip_serializing_if = "Option::is_none")]
    pub vertical_turn: Option<PlacedEmpty>,
    #[serde(
        rename = "inverted-vertical-turn",
        skip_serializing_if = "Option::is_none"
    )]
    pub inverted_vertical_turn: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shake: Option<PlacedEmpty>,
    #[serde(rename = "wavy-line", skip_serializing_if = "Option::is_none")]
    pub wavy_line: Option<WavyLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mordent: Option<Mordent>,
    #[serde(rename = "inverted-mordent", skip_serializing_if = "Option::is_none")]
    pub inverted_mordent: Option<Mordent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schleifer: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<Tremolo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haydn: Option<PlacedEmpty>,
    #[serde(rename = "other-ornament", skip_serializing_if = "Option::is_none")]
    pub other_ornament: Option<OtherPlacement>,
    #[serde(rename = "accidental-mark", default)]
    pub accidental_marks: Vec<AccidentalMark>,
}

/// A wavy line (trill extension or vibrato).
#[derive(Debug, Serialize, Deserialize)]
pub struct WavyLine {
    #[serde(rename = "@type")]
    pub wavy_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
}

/// A mordent ornament with optional long/approach/departure.
#[derive(Debug, Serialize, Deserialize)]
pub struct Mordent {
    #[serde(rename = "@long", skip_serializing_if = "Option::is_none")]
    pub long: Option<String>,
    #[serde(rename = "@approach", skip_serializing_if = "Option::is_none")]
    pub approach: Option<String>,
    #[serde(rename = "@departure", skip_serializing_if = "Option::is_none")]
    pub departure: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
}

/// A tremolo mark (slashed beams on the stem or between two notes).
#[derive(Debug, Serialize, Deserialize)]
pub struct Tremolo {
    /// `"single"`, `"start"`, `"stop"`, `"unmeasured"`.
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub tremolo_type: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
    /// Number of slashes (1–8).
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub marks: Option<u8>,
}

/// A placement element with a text value (for "other" notation types).
#[derive(Debug, Serialize, Deserialize)]
pub struct OtherPlacement {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Guitar/string technical notations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Technical {
    #[serde(rename = "up-bow", skip_serializing_if = "Option::is_none")]
    pub up_bow: Option<PlacedEmpty>,
    #[serde(rename = "down-bow", skip_serializing_if = "Option::is_none")]
    pub down_bow: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harmonic: Option<Harmonic>,
    #[serde(rename = "open-string", skip_serializing_if = "Option::is_none")]
    pub open_string: Option<PlacedEmpty>,
    #[serde(rename = "thumb-position", skip_serializing_if = "Option::is_none")]
    pub thumb_position: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingering: Option<Fingering>,
    #[serde(rename = "pluck", skip_serializing_if = "Option::is_none")]
    pub pluck: Option<PlacedEmpty>,
    #[serde(rename = "double-tongue", skip_serializing_if = "Option::is_none")]
    pub double_tongue: Option<PlacedEmpty>,
    #[serde(rename = "triple-tongue", skip_serializing_if = "Option::is_none")]
    pub triple_tongue: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped: Option<PlacedEmpty>,
    #[serde(rename = "snap-pizzicato", skip_serializing_if = "Option::is_none")]
    pub snap_pizzicato: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fret: Option<Fret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<StringNumber>,
    #[serde(rename = "hammer-on", skip_serializing_if = "Option::is_none")]
    pub hammer_on: Option<HammerPull>,
    #[serde(rename = "pull-off", skip_serializing_if = "Option::is_none")]
    pub pull_off: Option<HammerPull>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bend: Option<Bend>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tap: Option<OtherPlacement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heel: Option<HeelToe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toe: Option<HeelToe>,
    #[serde(rename = "fingernails", skip_serializing_if = "Option::is_none")]
    pub fingernails: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hole: Option<Hole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow: Option<Arrow>,
    #[serde(rename = "handbell", skip_serializing_if = "Option::is_none")]
    pub handbell: Option<OtherPlacement>,
    #[serde(rename = "brass-bend", skip_serializing_if = "Option::is_none")]
    pub brass_bend: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flip: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smear: Option<PlacedEmpty>,
    #[serde(rename = "open", skip_serializing_if = "Option::is_none")]
    pub open: Option<OtherPlacement>,
    #[serde(rename = "half-muted", skip_serializing_if = "Option::is_none")]
    pub half_muted: Option<OtherPlacement>,
    #[serde(rename = "harmon-mute", skip_serializing_if = "Option::is_none")]
    pub harmon_mute: Option<HarmonMute>,
    #[serde(rename = "golpe", skip_serializing_if = "Option::is_none")]
    pub golpe: Option<PlacedEmpty>,
    #[serde(rename = "other-technical", skip_serializing_if = "Option::is_none")]
    pub other_technical: Option<OtherPlacement>,
}

/// Harmonic notation (natural or artificial).
#[derive(Debug, Serialize, Deserialize)]
pub struct Harmonic {
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natural: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artificial: Option<()>,
    #[serde(rename = "base-pitch", skip_serializing_if = "Option::is_none")]
    pub base_pitch: Option<()>,
    #[serde(rename = "touching-pitch", skip_serializing_if = "Option::is_none")]
    pub touching_pitch: Option<()>,
    #[serde(rename = "sounding-pitch", skip_serializing_if = "Option::is_none")]
    pub sounding_pitch: Option<()>,
}

/// Left-hand fingering digit.
#[derive(Debug, Serialize, Deserialize)]
pub struct Fingering {
    #[serde(rename = "@substitution", skip_serializing_if = "Option::is_none")]
    pub substitution: Option<String>,
    #[serde(rename = "@alternate", skip_serializing_if = "Option::is_none")]
    pub alternate: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Fret number (tablature).
#[derive(Debug, Serialize, Deserialize)]
pub struct Fret {
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// String number (tablature, 1 = highest-pitched string).
#[derive(Debug, Serialize, Deserialize)]
pub struct StringNumber {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// Hammer-on or pull-off marking.
#[derive(Debug, Serialize, Deserialize)]
pub struct HammerPull {
    #[serde(rename = "@type")]
    pub technique_type: String,
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<u8>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A guitar bend.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bend {
    #[serde(rename = "@shape", skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    /// Amount of bend in semitones (e.g. `1` = whole step, `0.5` = half step).
    #[serde(rename = "bend-alter")]
    pub bend_alter: f64,
    /// Present when the bend is a pre-bend (bend before striking).
    #[serde(rename = "pre-bend", skip_serializing_if = "Option::is_none")]
    pub pre_bend: Option<()>,
    /// Present when the bend releases back to the original pitch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<BendRelease>,
    #[serde(rename = "with-bar", skip_serializing_if = "Option::is_none")]
    pub with_bar: Option<OtherPlacement>,
}

/// Release point of a bend.
#[derive(Debug, Serialize, Deserialize)]
pub struct BendRelease {
    #[serde(rename = "@offset", skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,
}

/// Organ heel/toe markings.
#[derive(Debug, Serialize, Deserialize)]
pub struct HeelToe {
    #[serde(rename = "@substitution", skip_serializing_if = "Option::is_none")]
    pub substitution: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
}

/// Wind instrument hole notation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Hole {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "hole-type", skip_serializing_if = "Option::is_none")]
    pub hole_type: Option<String>,
    #[serde(rename = "hole-closed", skip_serializing_if = "Option::is_none")]
    pub hole_closed: Option<HoleClosed>,
    #[serde(rename = "hole-shape", skip_serializing_if = "Option::is_none")]
    pub hole_shape: Option<String>,
}

/// Whether a wind hole is open, closed, or half-closed.
#[derive(Debug, Serialize, Deserialize)]
pub struct HoleClosed {
    #[serde(rename = "@location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Arrow direction for string notation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Arrow {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
    #[serde(rename = "arrow-direction", skip_serializing_if = "Option::is_none")]
    pub arrow_direction: Option<String>,
    #[serde(rename = "arrow-style", skip_serializing_if = "Option::is_none")]
    pub arrow_style: Option<String>,
    #[serde(rename = "circular-arrow", skip_serializing_if = "Option::is_none")]
    pub circular_arrow: Option<String>,
}

/// Harmon mute (stem in or out).
#[derive(Debug, Serialize, Deserialize)]
pub struct HarmonMute {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "harmon-closed", skip_serializing_if = "Option::is_none")]
    pub harmon_closed: Option<HarmonClosed>,
}

/// State of a harmon mute.
#[derive(Debug, Serialize, Deserialize)]
pub struct HarmonClosed {
    #[serde(rename = "@location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Accent, staccato, tenuto, and other articulation marks.
#[derive(Debug, Serialize, Deserialize)]
pub struct Articulations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<PlacedEmpty>,
    #[serde(rename = "strong-accent", skip_serializing_if = "Option::is_none")]
    pub strong_accent: Option<StrongAccent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staccato: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenuto: Option<PlacedEmpty>,
    #[serde(rename = "detached-legato", skip_serializing_if = "Option::is_none")]
    pub detached_legato: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staccatissimo: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spiccato: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoop: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plop: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doit: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub falloff: Option<PlacedEmpty>,
    #[serde(rename = "breath-mark", skip_serializing_if = "Option::is_none")]
    pub breath_mark: Option<BreathMark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caesura: Option<Caesura>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stress: Option<PlacedEmpty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unstress: Option<PlacedEmpty>,
    #[serde(rename = "soft-accent", skip_serializing_if = "Option::is_none")]
    pub soft_accent: Option<PlacedEmpty>,
    #[serde(rename = "other-articulation", skip_serializing_if = "Option::is_none")]
    pub other_articulation: Option<OtherPlacement>,
}

/// A marcato / forte-piano accent.
#[derive(Debug, Serialize, Deserialize)]
pub struct StrongAccent {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    /// `"up"` or `"down"`.
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub accent_type: Option<String>,
}

/// A breath mark.
#[derive(Debug, Serialize, Deserialize)]
pub struct BreathMark {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    /// `"comma"`, `"tick"`, `"upbow"`, `"salzedo"`.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A caesura (pause mark).
#[derive(Debug, Serialize, Deserialize)]
pub struct Caesura {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    /// `"normal"`, `"thick"`, `"short"`, `"curved"`, `"single"`, `"two-line"`.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Dynamic marking inside notations (applies to a single note).
#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicNotation {
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "$value")]
    pub marks: Vec<DynamicMark>,
}

/// A single dynamic symbol.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DynamicMark {
    P(()),
    Pp(()),
    Ppp(()),
    Pppp(()),
    Ppppp(()),
    Pppppp(()),
    F(()),
    Ff(()),
    Fff(()),
    Ffff(()),
    Fffff(()),
    Ffffff(()),
    Mp(()),
    Mf(()),
    Sf(()),
    Sfp(()),
    Sfpp(()),
    Fp(()),
    Rf(()),
    Rfz(()),
    Sfz(()),
    Sffz(()),
    Fz(()),
    N(()),
    Pf(()),
    Sfzp(()),
    #[serde(rename = "other-dynamics")]
    Other(String),
}

/// All notations that can appear on a note.
///
/// Maps to the `<notations>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Notations {
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "footnote", skip_serializing_if = "Option::is_none")]
    pub footnote: Option<FormattedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    #[serde(rename = "tied", default)]
    pub tied: Vec<Tied>,
    #[serde(rename = "slur", default)]
    pub slurs: Vec<Slur>,
    #[serde(rename = "tuplet", default)]
    pub tuplets: Vec<Tuplet>,
    #[serde(rename = "glissando", default)]
    pub glissandos: Vec<Glissando>,
    #[serde(rename = "slide", default)]
    pub slides: Vec<Slide>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ornaments: Option<Ornaments>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical: Option<Technical>,
    #[serde(rename = "articulations", default)]
    pub articulations: Vec<Articulations>,
    #[serde(rename = "dynamics", default)]
    pub dynamics: Vec<DynamicNotation>,
    #[serde(rename = "fermata", default)]
    pub fermatas: Vec<Fermata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arpeggiate: Option<Arpeggiate>,
    #[serde(rename = "non-arpeggiate", skip_serializing_if = "Option::is_none")]
    pub non_arpeggiate: Option<NonArpeggiate>,
    #[serde(rename = "accidental-mark", default)]
    pub accidental_marks: Vec<AccidentalMark>,
    #[serde(rename = "other-notation", default)]
    pub other_notations: Vec<OtherPlacement>,
}

/// A formatted text element (for footnotes inside notations).
#[derive(Debug, Serialize, Deserialize)]
pub struct FormattedText {
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "@justify", skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Editorial level indicator.
#[derive(Debug, Serialize, Deserialize)]
pub struct Level {
    #[serde(rename = "@reference", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(rename = "@parentheses", skip_serializing_if = "Option::is_none")]
    pub parentheses: Option<String>,
    #[serde(rename = "@bracket", skip_serializing_if = "Option::is_none")]
    pub bracket: Option<String>,
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// One syllable of lyrics.
#[derive(Debug, Serialize, Deserialize)]
pub struct Lyric {
    #[serde(rename = "@number", skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "@placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "@time-only", skip_serializing_if = "Option::is_none")]
    pub time_only: Option<String>,
    /// `"begin"`, `"end"`, `"middle"`, `"single"`.
    #[serde(rename = "syllabic", skip_serializing_if = "Option::is_none")]
    pub syllabic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<LyricText>,
    #[serde(rename = "elision", skip_serializing_if = "Option::is_none")]
    pub elision: Option<Elision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extend: Option<Extend>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub laughing: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humming: Option<()>,
    #[serde(rename = "end-line", skip_serializing_if = "Option::is_none")]
    pub end_line: Option<()>,
    #[serde(rename = "end-paragraph", skip_serializing_if = "Option::is_none")]
    pub end_paragraph: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnote: Option<FormattedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
}

/// The text of a lyric syllable.
#[derive(Debug, Serialize, Deserialize)]
pub struct LyricText {
    #[serde(rename = "@font-family", skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-style", skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(rename = "@font-weight", skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An elision character between syllables.
#[derive(Debug, Serialize, Deserialize)]
pub struct Elision {
    #[serde(rename = "@font-family", skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(rename = "@smufl", skip_serializing_if = "Option::is_none")]
    pub smufl: Option<String>,
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Lyric extension line.
#[derive(Debug, Serialize, Deserialize)]
pub struct Extend {
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub extend_type: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Grace note type and steal-time parameters.
#[derive(Debug, Serialize, Deserialize)]
pub struct Grace {
    /// Fraction of the following note's time stolen (for acciaccatura-style rendering).
    #[serde(
        rename = "@steal-time-previous",
        skip_serializing_if = "Option::is_none"
    )]
    pub steal_time_previous: Option<f64>,
    /// Fraction of the following note's time stolen.
    #[serde(
        rename = "@steal-time-following",
        skip_serializing_if = "Option::is_none"
    )]
    pub steal_time_following: Option<f64>,
    /// When `"yes"`, this is a slashed grace note (acciaccatura).
    #[serde(rename = "@slash", skip_serializing_if = "Option::is_none")]
    pub slash: Option<String>,
    #[serde(rename = "@make-time", skip_serializing_if = "Option::is_none")]
    pub make_time: Option<f64>,
}

/// A cue note marker.
#[derive(Debug, Serialize, Deserialize)]
pub struct Cue {}

/// A note, rest, or chord member.
///
/// Maps to the `<note>` element.
///
/// The pitch/rest/unpitched field identifies the note kind:
/// - `pitch` → a pitched note
/// - `rest` → a rest
/// - `unpitched` → an unpitched percussion note
///
/// `chord` is a marker element: when present the note belongs to the same chord as the
/// previous note (same onset time, same duration).
#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    // --- grace / cue ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace: Option<Grace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cue: Option<Cue>,

    // --- pitch kind (exactly one of these will be present) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<Pitch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest: Option<Rest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unpitched: Option<Unpitched>,

    // --- chord membership ---
    /// When present (as an empty element), this note is part of the same chord as the
    /// preceding note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chord: Option<()>,

    // --- duration and timing ---
    /// Duration in divisions (must be present unless this is a grace note).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(rename = "tie", default)]
    pub ties: Vec<Tie>,

    // --- editorial / voice ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnote: Option<FormattedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument: Option<NoteInstrument>,
    #[serde(rename = "voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,

    // --- visual notation ---
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub note_type: Option<NoteType>,
    #[serde(rename = "dot", default)]
    pub dots: Vec<Dot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accidental: Option<Accidental>,
    #[serde(rename = "time-modification", skip_serializing_if = "Option::is_none")]
    pub time_modification: Option<TimeModification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stem: Option<Stem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notehead: Option<Notehead>,
    #[serde(rename = "notehead-text", skip_serializing_if = "Option::is_none")]
    pub notehead_text: Option<NoteheadText>,

    /// Staff number for cross-staff notation (1-based, defaults to 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff: Option<u8>,
    #[serde(rename = "beam", default)]
    pub beams: Vec<Beam>,
    #[serde(rename = "notations", default)]
    pub notations: Vec<Notations>,
    #[serde(rename = "lyric", default)]
    pub lyrics: Vec<Lyric>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play: Option<Play>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<Listen>,

    // --- print attributes ---
    #[serde(rename = "@print-object", skip_serializing_if = "Option::is_none")]
    pub print_object: Option<String>,
    #[serde(rename = "@print-dot", skip_serializing_if = "Option::is_none")]
    pub print_dot: Option<String>,
    #[serde(rename = "@print-spacing", skip_serializing_if = "Option::is_none")]
    pub print_spacing: Option<String>,
    #[serde(rename = "@print-lyric", skip_serializing_if = "Option::is_none")]
    pub print_lyric: Option<String>,
    #[serde(rename = "@dynamics", skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<f64>,
    #[serde(rename = "@end-dynamics", skip_serializing_if = "Option::is_none")]
    pub end_dynamics: Option<f64>,
    #[serde(rename = "@attack", skip_serializing_if = "Option::is_none")]
    pub attack: Option<f64>,
    #[serde(rename = "@release", skip_serializing_if = "Option::is_none")]
    pub release_time: Option<f64>,
    #[serde(rename = "@time-only", skip_serializing_if = "Option::is_none")]
    pub time_only: Option<String>,
    #[serde(rename = "@pizzicato", skip_serializing_if = "Option::is_none")]
    pub pizzicato: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
}

/// References the instrument this note should play on.
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteInstrument {
    #[serde(rename = "@id")]
    pub id: String,
}

/// Alternate notehead text (for figured-bass or shaped-note notations).
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteheadText {
    #[serde(rename = "$value", default)]
    pub content: Vec<NoteheadTextContent>,
}

/// Content inside a notehead text block.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteheadTextContent {
    DisplayText(FormattedText),
    AccidentalText(super::part_list::AccidentalText),
}

/// MIDI/audio playback overrides for a note.
#[derive(Debug, Serialize, Deserialize)]
pub struct Play {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "ipa", skip_serializing_if = "Option::is_none")]
    pub ipa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<String>,
    #[serde(rename = "semi-pitched", skip_serializing_if = "Option::is_none")]
    pub semi_pitched: Option<String>,
    #[serde(rename = "other-play", skip_serializing_if = "Option::is_none")]
    pub other_play: Option<OtherPlacement>,
}

/// Listening cue for synchronized audio.
#[derive(Debug, Serialize, Deserialize)]
pub struct Listen {
    #[serde(rename = "$value", default)]
    pub content: Vec<ListenContent>,
}

/// Content inside a `<listen>` block.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListenContent {
    Assess(Assess),
    Wait(Wait),
    OtherListen(OtherPlacement),
}

/// Assessment point for a listening exercise.
#[derive(Debug, Serialize, Deserialize)]
pub struct Assess {
    #[serde(rename = "@type")]
    pub assess_type: String,
    #[serde(rename = "@player", skip_serializing_if = "Option::is_none")]
    pub player: Option<String>,
    #[serde(rename = "@time-only", skip_serializing_if = "Option::is_none")]
    pub time_only: Option<String>,
}

/// Wait/synchronization point in a listening exercise.
#[derive(Debug, Serialize, Deserialize)]
pub struct Wait {
    #[serde(rename = "@player", skip_serializing_if = "Option::is_none")]
    pub player: Option<String>,
    #[serde(rename = "@time-only", skip_serializing_if = "Option::is_none")]
    pub time_only: Option<String>,
    #[serde(rename = "@type")]
    pub wait_type: String,
}
