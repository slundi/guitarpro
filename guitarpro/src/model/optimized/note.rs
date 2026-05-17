use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum PitchStep {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Pitch {
    pub step: PitchStep,
    pub alter: i8,  // -2 to +2 semitones (double-flat to double-sharp)
    pub octave: u8, // scientific notation: C4 = middle C
}

/// Base rhythmic value, shared by Duration and TechniqueParams::Tremolo.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum NoteValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    HundredTwentyEighth,
    /// Non-standard duration value (power of 2 beyond 1/128). Preserved for byte-identical roundtrip.
    Other(u16),
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TieType {
    /// This note starts a tie chain; sounding duration extends to the End note.
    Start,
    /// No new MIDI attack; extends the Start note's sounding duration.
    End,
}

// ---------------------------------------------------------------------------
// Ornaments
// ---------------------------------------------------------------------------

/// Ornament marks attached to a note.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Ornament {
    Trill,
    Turn,
    DelayedTurn,
    InvertedTurn,
    DelayedInvertedTurn,
    Mordent,
    InvertedMordent {
        long: bool,
    },
    Shake,
    /// Tremolo slashes on the stem (1–8 marks).
    Tremolo(u8),
    /// Wavy line: trill extension or vibrato continuation.
    WavyLine,
}

// ---------------------------------------------------------------------------
// Articulations
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Articulation {
    // --- standard ---
    Staccato,
    Accent,
    Tenuto,
    Marcato,
    Fermata,
    // --- classical / orchestral ---
    Staccatissimo,
    Spiccato,
    // --- jazz / brass ---
    Scoop,
    Plop,
    Doit,
    Falloff,
    // --- breath / pause ---
    BreathMark,
    Caesura,
    // --- dynamic nuance ---
    Stress,
    Unstress,
    SoftAccent,
}

// ---------------------------------------------------------------------------
// Technical markings
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Technique {
    pub kind: TechniqueKind,
    pub params: TechniqueParams,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TechniqueKind {
    // --- guitar/string specific ---
    Bend,
    HammerOn,
    PullOff,
    Vibrato,
    SlideUp,        // generic upward slide (legacy: ShiftSlideTo)
    SlideDown,      // generic downward slide (legacy: OutDownwards)
    SlideLegato,    // legato slide to next note
    SlideOutUp,     // slide out upward from note
    SlideIntoAbove, // slide into note from above
    SlideIntoBelow, // slide into note from below
    Glissando,
    Tapping,
    Harmonic,
    TremoloPicking,
    Whammy,
    // --- muting / dampening ---
    OpenString,
    Stopped,
    SnapPizzicato,
    HalfMuted,
    HarmonMute,
    // --- bowing ---
    UpBow,
    DownBow,
    // --- sustain / articulation ---
    LetRing,
    // --- other ---
    Fingernails,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TechniqueParams {
    None,
    Bend {
        value: f32,
        vibrato: bool,
    }, // value in semitones
    Tremolo {
        speed: NoteValue,
    },
    /// `natural`: open-string touch harmonic; `false` = artificial (fretted + touch).
    Harmonic {
        natural: bool,
    },
    /// Continuous glide: `chromatic` = true for half-step chromatic gliss.
    Glissando {
        chromatic: bool,
    },
    /// Harmon mute (stem-in/stem-out): `None` = half-open, `Some(true)` = open, `Some(false)` = closed.
    HarmonMute {
        open: Option<bool>,
    },
}

// ---------------------------------------------------------------------------
// Note display attributes
// ---------------------------------------------------------------------------

/// Visual notehead shape (affects display only, not pitch or playback).
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Notehead {
    Normal,
    X,       // muted / dead note
    CircleX, // open / artificial harmonic
    Diamond, // natural harmonic (classical)
    Square,
    Slash, // rhythmic notation
    Triangle,
    ArrowUp,
    ArrowDown,
    None, // invisible notehead (spacing only)
}

/// Stem direction override.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum StemDirection {
    Up,
    Down,
    None, // stemless (e.g. whole notes or explicit stemless notation)
}

/// Whether an accidental symbol is shown and how it is qualified.
/// The accidental pitch value itself is always stored in `Pitch.alter`.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct AccidentalDisplay {
    /// Shown as a reminder (e.g. after a bar line or earlier in the measure).
    pub cautionary: bool,
    /// Added by an editor, shown in square brackets.
    pub editorial: bool,
    /// Enclosed in parentheses.
    pub parentheses: bool,
}

/// Arpeggiation direction for a rolled chord.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum ArpeggiateDirection {
    Up,
    Down,
}

/// Whether this chord is rolled (arpeggiated) or explicitly not rolled.
///
/// When present on any note in a beat, the convention applies to all notes
/// in that beat (matching MusicXML `<notations>` semantics).
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum ArpeggiateKind {
    /// Roll the chord, optionally specifying direction.
    Arpeggiate(Option<ArpeggiateDirection>),
    /// Explicitly suppress any default arpeggiation (bracket notation).
    NonArpeggiate,
}

// ---------------------------------------------------------------------------
// Finger
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
    Open,
}

// ---------------------------------------------------------------------------
// GP-specific note hints (for byte-identical legacy roundtrip)
// ---------------------------------------------------------------------------

/// A single point in a GP bend effect.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpBendPoint {
    pub position: u8,
    pub value: i8,
    pub vibrato: bool,
}

/// GP bend effect (bend, tremolo bar, etc.), preserved for byte-identical roundtrip.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpBendEffect {
    pub kind: i8,
    pub value: i16,
    pub points: Vec<GpBendPoint>,
}

/// GP5 harmonic note effect, preserved for byte-identical roundtrip.
/// `kind`: 1=Natural, 2=Artificial, 3=Tapped, 4=Pinch, 5=Semi.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpHarmonicEffect {
    pub kind: u8,
    /// Semitone value of the harmonic pitch (Artificial only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch_just: Option<i8>,
    /// Accidental of the harmonic pitch (Artificial only; -1=flat, 0=none, 1=sharp).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch_accidental: Option<i8>,
    /// Octave index (Artificial only; 0=None, 1=Ottava, 2=Quindicesima, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub octave: Option<u8>,
    /// Fret number (Tapped only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fret: Option<i8>,
}

/// GP5 grace note effect, preserved for byte-identical roundtrip.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpGraceEffect {
    pub fret: i8,
    pub velocity: i16,
    pub duration: u8,
    pub transition: i8,
    pub is_dead: bool,
    pub is_on_beat: bool,
}

/// GP4/5 trill note effect, preserved for byte-identical roundtrip.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpTrillEffect {
    pub fret: i8,
    /// Encoded period: 1=sixteenth, 2=thirty_second, 3=sixty_fourth.
    pub period: i8,
}

// ---------------------------------------------------------------------------
// Note
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub pitch: Option<Pitch>, // None = ghost note / rest
    pub string: Option<u8>,   // 1-based string number (tab)
    pub fret: Option<u8>,     // fret number (tab)
    pub tie: Option<TieType>,
    pub techniques: Vec<Technique>,
    pub ornaments: Vec<Ornament>,
    pub articulations: Vec<Articulation>,
    pub left_finger: Option<Finger>,
    pub right_finger: Option<Finger>,

    // --- display ---
    pub notehead: Option<Notehead>,
    pub stem: Option<StemDirection>,
    /// Explicit accidental display qualifier. `None` = let the renderer decide.
    pub accidental: Option<AccidentalDisplay>,
    /// Arpeggiation of the chord this note belongs to.
    pub arpeggiate: Option<ArpeggiateKind>,
    /// For unpitched percussion: the staff-line position to display the notehead on.
    /// Distinct from `pitch`, which may be absent for purely rhythmic parts.
    pub display_pitch: Option<Pitch>,

    // --- GP-specific roundtrip hints ---
    /// GP5 harmonic effect (Natural/Artificial/Tapped/Pinch/Semi).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_harmonic: Option<GpHarmonicEffect>,
    /// GP4/5 grace note effect on this note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_grace: Option<GpGraceEffect>,
    /// GP4/5 bend/tremolo bar effect with full point data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_bend: Option<GpBendEffect>,
    /// GP4/5 trill note effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_trill: Option<GpTrillEffect>,
    /// GP4/5 ghost note flag.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_ghost: bool,
    /// GP5 duration percent (1.0 = default; stored only when != 1.0).
    #[serde(default = "default_one_f32", skip_serializing_if = "is_one_f32")]
    pub gp_duration_percent: f32,
    /// GP5 swap accidentals flag.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_swap_accidentals: bool,
    /// Per-note MIDI velocity (GP-specific). `None` = use beat-level dynamic.
    /// Stored to enable byte-identical legacy roundtrip when notes in a beat differ.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_velocity: Option<i16>,
    /// Raw GP note-type byte when it falls outside the known range (0=Rest,1=Normal,2=Tie,3=Dead).
    /// Preserved for byte-identical roundtrip of files that use non-standard type values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_note_type_raw: Option<u8>,
    /// When `true`, this note is a GP Rest (NoteType::Rest). Stored explicitly because
    /// rest notes carry a non-zero value byte in the binary that must be preserved.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_is_rest: bool,
}

fn default_one_f32() -> f32 {
    1.0
}
fn is_one_f32(v: &f32) -> bool {
    (v - 1.0).abs() < 1e-6
}
