use serde::Deserialize;

/// The sounded pitch of a note.
#[derive(Debug, Deserialize)]
pub struct Pitch {
    /// Pitch class: `A`–`G`.
    pub step: String,
    /// Semitone alteration: `1.0` = sharp, `-1.0` = flat, `0.5` = quarter-sharp, etc.
    pub alter: Option<f64>,
    /// Octave in scientific pitch notation (middle C = C4, octave 4).
    pub octave: i8,
}

/// Indicates this note is a rest.
#[derive(Debug, Deserialize)]
pub struct Rest {
    /// When `"yes"`, this rest fills the whole measure regardless of duration.
    #[serde(rename = "@measure")]
    pub measure: Option<String>,
    /// The displayed pitch position of the rest symbol (not the sounded pitch).
    #[serde(rename = "display-step")]
    pub display_step: Option<String>,
    #[serde(rename = "display-octave")]
    pub display_octave: Option<i8>,
}

/// Unpitched note head (for percussion).
#[derive(Debug, Deserialize)]
pub struct Unpitched {
    #[serde(rename = "display-step")]
    pub display_step: Option<String>,
    #[serde(rename = "display-octave")]
    pub display_octave: Option<i8>,
}

/// The written note type value (rhythmic denominator).
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Deserialize)]
pub struct NoteType {
    /// `"full"`, `"cue"`, `"grace-cue"`, `"large"`.
    #[serde(rename = "@size")]
    pub size: Option<String>,
    #[serde(rename = "$text")]
    pub value: NoteTypeValue,
}

/// An augmentation dot.
#[derive(Debug, Deserialize)]
pub struct Dot {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
}

/// Displayed accidental symbol.
#[derive(Debug, Deserialize)]
pub struct Accidental {
    #[serde(rename = "@cautionary")]
    pub cautionary: Option<String>,
    #[serde(rename = "@editorial")]
    pub editorial: Option<String>,
    #[serde(rename = "@bracket")]
    pub bracket: Option<String>,
    #[serde(rename = "@parentheses")]
    pub parentheses: Option<String>,
    #[serde(rename = "@size")]
    pub size: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    /// E.g. `"sharp"`, `"natural"`, `"flat"`, `"double-sharp"`, `"flat-flat"`, `"natural-sharp"`, etc.
    #[serde(rename = "$text")]
    pub value: String,
}

/// Time modification for a tuplet note (e.g. 3 notes in the time of 2).
#[derive(Debug, Deserialize)]
pub struct TimeModification {
    /// Actual number of notes in the tuplet group.
    #[serde(rename = "actual-notes")]
    pub actual_notes: u8,
    /// Normal number of notes the group takes the time of.
    #[serde(rename = "normal-notes")]
    pub normal_notes: u8,
    #[serde(rename = "normal-type")]
    pub normal_type: Option<String>,
    #[serde(rename = "normal-dot", default)]
    pub normal_dots: Vec<()>,
}

/// Direction of a stem.
#[derive(Debug, Deserialize)]
pub struct Stem {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@y-position")]
    pub y_position: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    /// `"up"`, `"down"`, `"double"`, `"none"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// Note head shape.
#[derive(Debug, Deserialize)]
pub struct Notehead {
    #[serde(rename = "@filled")]
    pub filled: Option<String>,
    #[serde(rename = "@parentheses")]
    pub parentheses: Option<String>,
    #[serde(rename = "@font-family")]
    pub font_family: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    /// E.g. `"normal"`, `"diamond"`, `"x"`, `"circle-x"`, `"square"`, `"slash"`,
    /// `"triangle"`, `"arrow down"`, `"arrow up"`, `"circled"`, `"do"`, `"re"`, etc.
    #[serde(rename = "$text")]
    pub value: String,
}

/// Beam notation for a single beam level.
#[derive(Debug, Deserialize)]
pub struct Beam {
    /// Beam level (1 = outer beam, up to 8).
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@fan")]
    pub fan: Option<String>,
    /// `"begin"`, `"continue"`, `"end"`, `"forward hook"`, `"backward hook"`.
    #[serde(rename = "$text")]
    pub value: String,
}

/// A tie notation mark on the note (separate from the logical `<tied>` in notations).
#[derive(Debug, Deserialize)]
pub struct Tie {
    /// `"start"` or `"stop"`.
    #[serde(rename = "@type")]
    pub tie_type: String,
    #[serde(rename = "@time-only")]
    pub time_only: Option<String>,
}

// ---------------------------------------------------------------------------
// Notations
// ---------------------------------------------------------------------------

/// A tie notation marking (inside `<notations>`).
#[derive(Debug, Deserialize)]
pub struct Tied {
    #[serde(rename = "@type")]
    pub tied_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type")]
    pub line_type: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@orientation")]
    pub orientation: Option<String>,
}

/// A slur notation marking.
#[derive(Debug, Deserialize)]
pub struct Slur {
    #[serde(rename = "@type")]
    pub slur_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type")]
    pub line_type: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@orientation")]
    pub orientation: Option<String>,
}

/// Tuplet bracket notation.
#[derive(Debug, Deserialize)]
pub struct Tuplet {
    #[serde(rename = "@type")]
    pub tuplet_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@bracket")]
    pub bracket: Option<String>,
    #[serde(rename = "@show-number")]
    pub show_number: Option<String>,
    #[serde(rename = "@show-type")]
    pub show_type: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "tuplet-actual")]
    pub tuplet_actual: Option<TupletPortion>,
    #[serde(rename = "tuplet-normal")]
    pub tuplet_normal: Option<TupletPortion>,
}

/// The actual or normal portion of a displayed tuplet ratio.
#[derive(Debug, Deserialize)]
pub struct TupletPortion {
    #[serde(rename = "tuplet-number")]
    pub tuplet_number: Option<TupletNumber>,
    #[serde(rename = "tuplet-type")]
    pub tuplet_type: Option<String>,
    #[serde(rename = "tuplet-dot", default)]
    pub tuplet_dots: Vec<()>,
}

/// The number shown on a tuplet bracket.
#[derive(Debug, Deserialize)]
pub struct TupletNumber {
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// A glissando (continuous-line gliss).
#[derive(Debug, Deserialize)]
pub struct Glissando {
    #[serde(rename = "@type")]
    pub glissando_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type")]
    pub line_type: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A slide (solid line from one note to another).
#[derive(Debug, Deserialize)]
pub struct Slide {
    #[serde(rename = "@type")]
    pub slide_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@line-type")]
    pub line_type: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A fermata above or below a note.
#[derive(Debug, Deserialize)]
pub struct Fermata {
    #[serde(rename = "@type")]
    pub fermata_type: Option<String>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    /// E.g. `"normal"`, `"angled"`, `"square"`, `"double-angled"`, `"double-square"`,
    /// `"double-dot"`, `"half-curve"`, `"curlew"`.
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// An arpeggiate notation (rolled chord indicator).
#[derive(Debug, Deserialize)]
pub struct Arpeggiate {
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@direction")]
    pub direction: Option<String>,
    #[serde(rename = "@unbroken")]
    pub unbroken: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
}

/// Indicates a chord is not arpeggiated.
#[derive(Debug, Deserialize)]
pub struct NonArpeggiate {
    #[serde(rename = "@type")]
    pub non_arpeggiate_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
}

/// An accidental in an ornament (e.g. trill accidental).
#[derive(Debug, Deserialize)]
pub struct AccidentalMark {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An empty placement element (many ornaments / articulations have no attributes beyond placement).
#[derive(Debug, Deserialize)]
pub struct PlacedEmpty {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
}

/// Trill, turn, mordent, and other ornament marks.
#[derive(Debug, Deserialize)]
pub struct Ornaments {
    #[serde(rename = "trill-mark")]
    pub trill_mark: Option<PlacedEmpty>,
    pub turn: Option<PlacedEmpty>,
    #[serde(rename = "delayed-turn")]
    pub delayed_turn: Option<PlacedEmpty>,
    #[serde(rename = "inverted-turn")]
    pub inverted_turn: Option<PlacedEmpty>,
    #[serde(rename = "delayed-inverted-turn")]
    pub delayed_inverted_turn: Option<PlacedEmpty>,
    #[serde(rename = "vertical-turn")]
    pub vertical_turn: Option<PlacedEmpty>,
    #[serde(rename = "inverted-vertical-turn")]
    pub inverted_vertical_turn: Option<PlacedEmpty>,
    pub shake: Option<PlacedEmpty>,
    #[serde(rename = "wavy-line")]
    pub wavy_line: Option<WavyLine>,
    pub mordent: Option<Mordent>,
    #[serde(rename = "inverted-mordent")]
    pub inverted_mordent: Option<Mordent>,
    pub schleifer: Option<PlacedEmpty>,
    pub tremolo: Option<Tremolo>,
    pub haydn: Option<PlacedEmpty>,
    #[serde(rename = "other-ornament")]
    pub other_ornament: Option<OtherPlacement>,
    #[serde(rename = "accidental-mark", default)]
    pub accidental_marks: Vec<AccidentalMark>,
}

/// A wavy line (trill extension or vibrato).
#[derive(Debug, Deserialize)]
pub struct WavyLine {
    #[serde(rename = "@type")]
    pub wavy_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
}

/// A mordent ornament with optional long/approach/departure.
#[derive(Debug, Deserialize)]
pub struct Mordent {
    #[serde(rename = "@long")]
    pub long: Option<String>,
    #[serde(rename = "@approach")]
    pub approach: Option<String>,
    #[serde(rename = "@departure")]
    pub departure: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
}

/// A tremolo mark (slashed beams on the stem or between two notes).
#[derive(Debug, Deserialize)]
pub struct Tremolo {
    /// `"single"`, `"start"`, `"stop"`, `"unmeasured"`.
    #[serde(rename = "@type")]
    pub tremolo_type: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    /// Number of slashes (1–8).
    #[serde(rename = "$text")]
    pub marks: Option<u8>,
}

/// A placement element with a text value (for "other" notation types).
#[derive(Debug, Deserialize)]
pub struct OtherPlacement {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// Guitar/string technical notations.
#[derive(Debug, Deserialize)]
pub struct Technical {
    #[serde(rename = "up-bow")]
    pub up_bow: Option<PlacedEmpty>,
    #[serde(rename = "down-bow")]
    pub down_bow: Option<PlacedEmpty>,
    pub harmonic: Option<Harmonic>,
    #[serde(rename = "open-string")]
    pub open_string: Option<PlacedEmpty>,
    #[serde(rename = "thumb-position")]
    pub thumb_position: Option<PlacedEmpty>,
    pub fingering: Option<Fingering>,
    #[serde(rename = "pluck")]
    pub pluck: Option<PlacedEmpty>,
    #[serde(rename = "double-tongue")]
    pub double_tongue: Option<PlacedEmpty>,
    #[serde(rename = "triple-tongue")]
    pub triple_tongue: Option<PlacedEmpty>,
    pub stopped: Option<PlacedEmpty>,
    #[serde(rename = "snap-pizzicato")]
    pub snap_pizzicato: Option<PlacedEmpty>,
    pub fret: Option<Fret>,
    pub string: Option<StringNumber>,
    #[serde(rename = "hammer-on")]
    pub hammer_on: Option<HammerPull>,
    #[serde(rename = "pull-off")]
    pub pull_off: Option<HammerPull>,
    pub bend: Option<Bend>,
    pub tap: Option<OtherPlacement>,
    pub heel: Option<HeelToe>,
    pub toe: Option<HeelToe>,
    #[serde(rename = "fingernails")]
    pub fingernails: Option<PlacedEmpty>,
    pub hole: Option<Hole>,
    pub arrow: Option<Arrow>,
    #[serde(rename = "handbell")]
    pub handbell: Option<OtherPlacement>,
    #[serde(rename = "brass-bend")]
    pub brass_bend: Option<PlacedEmpty>,
    pub flip: Option<PlacedEmpty>,
    pub smear: Option<PlacedEmpty>,
    #[serde(rename = "open")]
    pub open: Option<OtherPlacement>,
    #[serde(rename = "half-muted")]
    pub half_muted: Option<OtherPlacement>,
    #[serde(rename = "harmon-mute")]
    pub harmon_mute: Option<HarmonMute>,
    #[serde(rename = "golpe")]
    pub golpe: Option<PlacedEmpty>,
    #[serde(rename = "other-technical")]
    pub other_technical: Option<OtherPlacement>,
}

/// Harmonic notation (natural or artificial).
#[derive(Debug, Deserialize)]
pub struct Harmonic {
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    pub natural: Option<()>,
    pub artificial: Option<()>,
    #[serde(rename = "base-pitch")]
    pub base_pitch: Option<()>,
    #[serde(rename = "touching-pitch")]
    pub touching_pitch: Option<()>,
    #[serde(rename = "sounding-pitch")]
    pub sounding_pitch: Option<()>,
}

/// Left-hand fingering digit.
#[derive(Debug, Deserialize)]
pub struct Fingering {
    #[serde(rename = "@substitution")]
    pub substitution: Option<String>,
    #[serde(rename = "@alternate")]
    pub alternate: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Fret number (tablature).
#[derive(Debug, Deserialize)]
pub struct Fret {
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// String number (tablature, 1 = highest-pitched string).
#[derive(Debug, Deserialize)]
pub struct StringNumber {
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "$text")]
    pub value: u8,
}

/// Hammer-on or pull-off marking.
#[derive(Debug, Deserialize)]
pub struct HammerPull {
    #[serde(rename = "@type")]
    pub technique_type: String,
    #[serde(rename = "@number")]
    pub number: Option<u8>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A guitar bend.
#[derive(Debug, Deserialize)]
pub struct Bend {
    #[serde(rename = "@shape")]
    pub shape: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    /// Amount of bend in semitones (e.g. `1` = whole step, `0.5` = half step).
    #[serde(rename = "bend-alter")]
    pub bend_alter: f64,
    /// Present when the bend is a pre-bend (bend before striking).
    #[serde(rename = "pre-bend")]
    pub pre_bend: Option<()>,
    /// Present when the bend releases back to the original pitch.
    pub release: Option<BendRelease>,
    #[serde(rename = "with-bar")]
    pub with_bar: Option<OtherPlacement>,
}

/// Release point of a bend.
#[derive(Debug, Deserialize)]
pub struct BendRelease {
    #[serde(rename = "@offset")]
    pub offset: Option<f64>,
}

/// Organ heel/toe markings.
#[derive(Debug, Deserialize)]
pub struct HeelToe {
    #[serde(rename = "@substitution")]
    pub substitution: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
}

/// Wind instrument hole notation.
#[derive(Debug, Deserialize)]
pub struct Hole {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "hole-type")]
    pub hole_type: Option<String>,
    #[serde(rename = "hole-closed")]
    pub hole_closed: Option<HoleClosed>,
    #[serde(rename = "hole-shape")]
    pub hole_shape: Option<String>,
}

/// Whether a wind hole is open, closed, or half-closed.
#[derive(Debug, Deserialize)]
pub struct HoleClosed {
    #[serde(rename = "@location")]
    pub location: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Arrow direction for string notation.
#[derive(Debug, Deserialize)]
pub struct Arrow {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    #[serde(rename = "arrow-direction")]
    pub arrow_direction: Option<String>,
    #[serde(rename = "arrow-style")]
    pub arrow_style: Option<String>,
    #[serde(rename = "circular-arrow")]
    pub circular_arrow: Option<String>,
}

/// Harmon mute (stem in or out).
#[derive(Debug, Deserialize)]
pub struct HarmonMute {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "harmon-closed")]
    pub harmon_closed: Option<HarmonClosed>,
}

/// State of a harmon mute.
#[derive(Debug, Deserialize)]
pub struct HarmonClosed {
    #[serde(rename = "@location")]
    pub location: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Accent, staccato, tenuto, and other articulation marks.
#[derive(Debug, Deserialize)]
pub struct Articulations {
    pub accent: Option<PlacedEmpty>,
    #[serde(rename = "strong-accent")]
    pub strong_accent: Option<StrongAccent>,
    pub staccato: Option<PlacedEmpty>,
    pub tenuto: Option<PlacedEmpty>,
    #[serde(rename = "detached-legato")]
    pub detached_legato: Option<PlacedEmpty>,
    pub staccatissimo: Option<PlacedEmpty>,
    pub spiccato: Option<PlacedEmpty>,
    pub scoop: Option<PlacedEmpty>,
    pub plop: Option<PlacedEmpty>,
    pub doit: Option<PlacedEmpty>,
    pub falloff: Option<PlacedEmpty>,
    #[serde(rename = "breath-mark")]
    pub breath_mark: Option<BreathMark>,
    pub caesura: Option<Caesura>,
    pub stress: Option<PlacedEmpty>,
    pub unstress: Option<PlacedEmpty>,
    #[serde(rename = "soft-accent")]
    pub soft_accent: Option<PlacedEmpty>,
    #[serde(rename = "other-articulation")]
    pub other_articulation: Option<OtherPlacement>,
}

/// A marcato / forte-piano accent.
#[derive(Debug, Deserialize)]
pub struct StrongAccent {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    /// `"up"` or `"down"`.
    #[serde(rename = "@type")]
    pub accent_type: Option<String>,
}

/// A breath mark.
#[derive(Debug, Deserialize)]
pub struct BreathMark {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    /// `"comma"`, `"tick"`, `"upbow"`, `"salzedo"`.
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A caesura (pause mark).
#[derive(Debug, Deserialize)]
pub struct Caesura {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    /// `"normal"`, `"thick"`, `"short"`, `"curved"`, `"single"`, `"two-line"`.
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// Dynamic marking inside notations (applies to a single note).
#[derive(Debug, Deserialize)]
pub struct DynamicNotation {
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "$value")]
    pub marks: Vec<DynamicMark>,
}

/// A single dynamic symbol.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct Notations {
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "footnote")]
    pub footnote: Option<FormattedText>,
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
    pub ornaments: Option<Ornaments>,
    pub technical: Option<Technical>,
    #[serde(rename = "articulations", default)]
    pub articulations: Vec<Articulations>,
    #[serde(rename = "dynamics", default)]
    pub dynamics: Vec<DynamicNotation>,
    #[serde(rename = "fermata", default)]
    pub fermatas: Vec<Fermata>,
    pub arpeggiate: Option<Arpeggiate>,
    #[serde(rename = "non-arpeggiate")]
    pub non_arpeggiate: Option<NonArpeggiate>,
    #[serde(rename = "accidental-mark", default)]
    pub accidental_marks: Vec<AccidentalMark>,
    #[serde(rename = "other-notation", default)]
    pub other_notations: Vec<OtherPlacement>,
}

/// A formatted text element (for footnotes inside notations).
#[derive(Debug, Deserialize)]
pub struct FormattedText {
    #[serde(rename = "@xml:lang")]
    pub lang: Option<String>,
    #[serde(rename = "@justify")]
    pub justify: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Editorial level indicator.
#[derive(Debug, Deserialize)]
pub struct Level {
    #[serde(rename = "@reference")]
    pub reference: Option<String>,
    #[serde(rename = "@parentheses")]
    pub parentheses: Option<String>,
    #[serde(rename = "@bracket")]
    pub bracket: Option<String>,
    #[serde(rename = "@size")]
    pub size: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// One syllable of lyrics.
#[derive(Debug, Deserialize)]
pub struct Lyric {
    #[serde(rename = "@number")]
    pub number: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@placement")]
    pub placement: Option<String>,
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "@time-only")]
    pub time_only: Option<String>,
    /// `"begin"`, `"end"`, `"middle"`, `"single"`.
    #[serde(rename = "syllabic")]
    pub syllabic: Option<String>,
    pub text: Option<LyricText>,
    #[serde(rename = "elision")]
    pub elision: Option<Elision>,
    pub extend: Option<Extend>,
    pub laughing: Option<()>,
    pub humming: Option<()>,
    #[serde(rename = "end-line")]
    pub end_line: Option<()>,
    #[serde(rename = "end-paragraph")]
    pub end_paragraph: Option<()>,
    pub footnote: Option<FormattedText>,
    pub level: Option<Level>,
}

/// The text of a lyric syllable.
#[derive(Debug, Deserialize)]
pub struct LyricText {
    #[serde(rename = "@font-family")]
    pub font_family: Option<String>,
    #[serde(rename = "@font-size")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-style")]
    pub font_style: Option<String>,
    #[serde(rename = "@font-weight")]
    pub font_weight: Option<String>,
    #[serde(rename = "@xml:lang")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An elision character between syllables.
#[derive(Debug, Deserialize)]
pub struct Elision {
    #[serde(rename = "@font-family")]
    pub font_family: Option<String>,
    #[serde(rename = "@smufl")]
    pub smufl: Option<String>,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// Lyric extension line.
#[derive(Debug, Deserialize)]
pub struct Extend {
    #[serde(rename = "@type")]
    pub extend_type: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
    #[serde(rename = "@color")]
    pub color: Option<String>,
}

/// Grace note type and steal-time parameters.
#[derive(Debug, Deserialize)]
pub struct Grace {
    /// Fraction of the following note's time stolen (for acciaccatura-style rendering).
    #[serde(rename = "@steal-time-previous")]
    pub steal_time_previous: Option<f64>,
    /// Fraction of the following note's time stolen.
    #[serde(rename = "@steal-time-following")]
    pub steal_time_following: Option<f64>,
    /// When `"yes"`, this is a slashed grace note (acciaccatura).
    #[serde(rename = "@slash")]
    pub slash: Option<String>,
    #[serde(rename = "@make-time")]
    pub make_time: Option<f64>,
}

/// A cue note marker.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct Note {
    // --- grace / cue ---
    pub grace: Option<Grace>,
    pub cue: Option<Cue>,

    // --- pitch kind (exactly one of these will be present) ---
    pub pitch: Option<Pitch>,
    pub rest: Option<Rest>,
    pub unpitched: Option<Unpitched>,

    // --- chord membership ---
    /// When present (as an empty element), this note is part of the same chord as the
    /// preceding note.
    pub chord: Option<()>,

    // --- duration and timing ---
    /// Duration in divisions (must be present unless this is a grace note).
    pub duration: Option<u32>,
    #[serde(rename = "tie", default)]
    pub ties: Vec<Tie>,

    // --- editorial / voice ---
    pub footnote: Option<FormattedText>,
    pub level: Option<Level>,
    pub instrument: Option<NoteInstrument>,
    #[serde(rename = "voice")]
    pub voice: Option<String>,

    // --- visual notation ---
    #[serde(rename = "type")]
    pub note_type: Option<NoteType>,
    #[serde(rename = "dot", default)]
    pub dots: Vec<Dot>,
    pub accidental: Option<Accidental>,
    #[serde(rename = "time-modification")]
    pub time_modification: Option<TimeModification>,
    pub stem: Option<Stem>,
    pub notehead: Option<Notehead>,
    #[serde(rename = "notehead-text")]
    pub notehead_text: Option<NoteheadText>,

    /// Staff number for cross-staff notation (1-based, defaults to 1).
    pub staff: Option<u8>,
    #[serde(rename = "beam", default)]
    pub beams: Vec<Beam>,
    #[serde(rename = "notations", default)]
    pub notations: Vec<Notations>,
    #[serde(rename = "lyric", default)]
    pub lyrics: Vec<Lyric>,
    pub play: Option<Play>,
    pub listen: Option<Listen>,

    // --- print attributes ---
    #[serde(rename = "@print-object")]
    pub print_object: Option<String>,
    #[serde(rename = "@print-dot")]
    pub print_dot: Option<String>,
    #[serde(rename = "@print-spacing")]
    pub print_spacing: Option<String>,
    #[serde(rename = "@print-lyric")]
    pub print_lyric: Option<String>,
    #[serde(rename = "@dynamics")]
    pub dynamics: Option<f64>,
    #[serde(rename = "@end-dynamics")]
    pub end_dynamics: Option<f64>,
    #[serde(rename = "@attack")]
    pub attack: Option<f64>,
    #[serde(rename = "@release")]
    pub release_time: Option<f64>,
    #[serde(rename = "@time-only")]
    pub time_only: Option<String>,
    #[serde(rename = "@pizzicato")]
    pub pizzicato: Option<String>,
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@default-x")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y")]
    pub default_y: Option<f64>,
}

/// References the instrument this note should play on.
#[derive(Debug, Deserialize)]
pub struct NoteInstrument {
    #[serde(rename = "@id")]
    pub id: String,
}

/// Alternate notehead text (for figured-bass or shaped-note notations).
#[derive(Debug, Deserialize)]
pub struct NoteheadText {
    #[serde(rename = "$value", default)]
    pub content: Vec<NoteheadTextContent>,
}

/// Content inside a notehead text block.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteheadTextContent {
    DisplayText(FormattedText),
    AccidentalText(super::part_list::AccidentalText),
}

/// MIDI/audio playback overrides for a note.
#[derive(Debug, Deserialize)]
pub struct Play {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "ipa")]
    pub ipa: Option<String>,
    pub mute: Option<String>,
    #[serde(rename = "semi-pitched")]
    pub semi_pitched: Option<String>,
    #[serde(rename = "other-play")]
    pub other_play: Option<OtherPlacement>,
}

/// Listening cue for synchronized audio.
#[derive(Debug, Deserialize)]
pub struct Listen {
    #[serde(rename = "$value", default)]
    pub content: Vec<ListenContent>,
}

/// Content inside a `<listen>` block.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListenContent {
    Assess(Assess),
    Wait(Wait),
    OtherListen(OtherPlacement),
}

/// Assessment point for a listening exercise.
#[derive(Debug, Deserialize)]
pub struct Assess {
    #[serde(rename = "@type")]
    pub assess_type: String,
    #[serde(rename = "@player")]
    pub player: Option<String>,
    #[serde(rename = "@time-only")]
    pub time_only: Option<String>,
}

/// Wait/synchronization point in a listening exercise.
#[derive(Debug, Deserialize)]
pub struct Wait {
    #[serde(rename = "@player")]
    pub player: Option<String>,
    #[serde(rename = "@time-only")]
    pub time_only: Option<String>,
    #[serde(rename = "@type")]
    pub wait_type: String,
}
