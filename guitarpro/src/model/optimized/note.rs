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

impl PitchStep {
    /// The steps in octave order, from C.
    pub const ALL: [PitchStep; 7] = [
        PitchStep::C,
        PitchStep::D,
        PitchStep::E,
        PitchStep::F,
        PitchStep::G,
        PitchStep::A,
        PitchStep::B,
    ];

    /// The step at `index` in the octave (C = 0, wrapping around: −1 = B).
    /// Inverse of [`PitchStep::index`].
    pub fn from_index(index: i32) -> PitchStep {
        Self::ALL[index.rem_euclid(7) as usize]
    }

    /// The step's position in the octave, from C = 0 to B = 6.
    pub fn index(self) -> i32 {
        match self {
            PitchStep::C => 0,
            PitchStep::D => 1,
            PitchStep::E => 2,
            PitchStep::F => 3,
            PitchStep::G => 4,
            PitchStep::A => 5,
            PitchStep::B => 6,
        }
    }

    /// The natural step's semitones above C (E = 4, B = 11).
    pub fn semitone(self) -> i32 {
        match self {
            PitchStep::C => 0,
            PitchStep::D => 2,
            PitchStep::E => 4,
            PitchStep::F => 5,
            PitchStep::G => 7,
            PitchStep::A => 9,
            PitchStep::B => 11,
        }
    }

    /// The natural step's position on the line of fifths, relative to C
    /// (F = −1, G = 1, B = 5).
    pub fn fifths(self) -> i32 {
        match self {
            PitchStep::F => -1,
            PitchStep::C => 0,
            PitchStep::G => 1,
            PitchStep::D => 2,
            PitchStep::A => 3,
            PitchStep::E => 4,
            PitchStep::B => 5,
        }
    }
}

impl Pitch {
    /// The MIDI note number (C4 = 60).
    pub fn midi(self) -> i32 {
        (self.octave as i32 + 1) * 12 + self.step.semitone() + self.alter as i32
    }

    /// The pitch class's position on the line of fifths, relative to C
    /// (F♯ = 6, B♭ = −2). Inverse of [`Pitch::from_fifths`].
    pub fn fifths(self) -> i32 {
        self.step.fifths() + 7 * self.alter as i32
    }

    /// The pitch class `fifths` from C on the line of fifths, in octave 4;
    /// `None` beyond a double sharp/flat.
    pub fn from_fifths(fifths: i32) -> Option<Pitch> {
        let alter = (fifths + 1).div_euclid(7);
        if !(-2..=2).contains(&alter) {
            return None;
        }
        let step = match fifths - 7 * alter {
            -1 => PitchStep::F,
            0 => PitchStep::C,
            1 => PitchStep::G,
            2 => PitchStep::D,
            3 => PitchStep::A,
            4 => PitchStep::E,
            _ => PitchStep::B,
        };
        Some(Pitch {
            step,
            alter: alter as i8,
            octave: 4,
        })
    }
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

impl NoteValue {
    /// The denominator of its fraction of a whole note (4 for a quarter);
    /// `Other`'s own, at least 1.
    pub fn denominator(self) -> u32 {
        match self {
            NoteValue::Whole => 1,
            NoteValue::Half => 2,
            NoteValue::Quarter => 4,
            NoteValue::Eighth => 8,
            NoteValue::Sixteenth => 16,
            NoteValue::ThirtySecond => 32,
            NoteValue::SixtyFourth => 64,
            NoteValue::HundredTwentyEighth => 128,
            NoteValue::Other(n) => u32::from(n).max(1),
        }
    }

    /// Its length in ticks, at `divisions` ticks per quarter.
    pub fn ticks(self, divisions: u32) -> u32 {
        divisions * 4 / self.denominator()
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TieType {
    /// This note starts a tie chain; sounding duration extends to the End note.
    Start,
    /// No new MIDI attack; extends the Start note's sounding duration.
    End,
    /// Ends one tie and starts the next: the middle of a tie chain.
    Continue,
}

impl TieType {
    /// Whether the note is tied from the one before it.
    pub fn ends_tie(self) -> bool {
        matches!(self, TieType::End | TieType::Continue)
    }

    /// Whether the note is tied to the one after it.
    pub fn starts_tie(self) -> bool {
        matches!(self, TieType::Start | TieType::Continue)
    }

    /// The tie type for a note tied from the one before and/or to the one
    /// after.
    pub fn from_ends(from_previous: bool, to_next: bool) -> Option<TieType> {
        match (from_previous, to_next) {
            (true, true) => Some(TieType::Continue),
            (true, false) => Some(TieType::End),
            (false, true) => Some(TieType::Start),
            (false, false) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Ornaments
// ---------------------------------------------------------------------------

/// Ornament marks attached to a note. `Other` holds a symbol with no
/// variant (a SMuFL name such as `ornamentPrallMordent`) verbatim.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
    /// A turn stood on end.
    VerticalTurn,
    InvertedVerticalTurn,
    /// A slide up into the note (Schleifer).
    Schleifer,
    /// Haydn's ornament.
    Haydn,
    Other(String),
}

// ---------------------------------------------------------------------------
// Articulations
// ---------------------------------------------------------------------------

/// Articulation marks. `Other` holds a symbol with no variant (a SMuFL
/// name such as `brassMuteClosed`) verbatim.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
    // Added after the ones above, so a positional (binary) save's indices
    // stay those of its version.
    /// Tenuto and staccato together (portato).
    DetachedLegato,
    Other(String),
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
    /// GP7/GPX ornament string on this note (e.g. "UpperMordent", "Turn").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_ornament: Option<String>,
    /// GP3/4 note-level duration override (flags & 0x01). `None` = not present.
    /// When present, two extra bytes (duration + tuplet) are written after the flags byte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_note_duration: Option<i8>,
    /// GP3/4 note-level tuplet override (flags & 0x01). `None` = not present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_note_tuplet: Option<i8>,
}

fn default_one_f32() -> f32 {
    1.0
}
fn is_one_f32(v: &f32) -> bool {
    (v - 1.0).abs() < 1e-6
}
