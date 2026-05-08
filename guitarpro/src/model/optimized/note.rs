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

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub pitch: Option<Pitch>, // None = ghost note / rest
    pub string: Option<u8>,   // 1-based string number (tab)
    pub fret: Option<u8>,     // fret number (tab)
    pub tie: Option<TieType>,
    pub techniques: Vec<Technique>,
    pub articulations: Vec<Articulation>,
    pub left_finger: Option<Finger>,
    pub right_finger: Option<Finger>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Technique {
    pub kind: TechniqueKind,
    pub params: TechniqueParams,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TechniqueKind {
    Bend,
    HammerOn,
    PullOff,
    Vibrato,
    SlideUp,
    SlideDown,
    Tapping,
    Harmonic,
    TremoloPicking,
    Whammy,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TechniqueParams {
    None,
    Bend { value: f32, vibrato: bool }, // value in semitones
    Tremolo { speed: NoteValue },
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Articulation {
    Staccato,
    Accent,
    Tenuto,
    Marcato,
    Fermata,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
    Open,
}
