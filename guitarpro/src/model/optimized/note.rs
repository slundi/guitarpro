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
    SlideUp,
    SlideDown,
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
}
