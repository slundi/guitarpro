//! Voice, Beat, Duration, and related rhythm/articulation types.

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    effect::BeatEvent,
    global::LyricLineId,
    metadata::ChordSymbol,
    note::{Note, NoteValue},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Voice {
    pub voice_id: u8,
    pub beats: Vec<Beat>,
}

/// A grace note preceding a beat.
///
/// Grace notes are rendered before the beat's `tick_offset` and typically
/// steal time from the following note (appoggiatura) or the preceding one
/// (acciaccatura with `slash = true`).
#[derive(Serialize, Deserialize, Debug)]
pub struct GraceNote {
    pub note: Note,
    /// Acciaccatura (slashed grace note) when `true`; appoggiatura when `false`.
    pub slash: bool,
    /// Fraction of the beat duration to steal for this grace note (0.0–1.0).
    /// `None` = renderer decides based on context and style.
    pub steal_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Beat {
    pub tick_offset: u32,
    pub duration: Duration,
    pub notes: Vec<Note>,
    pub events: Vec<BeatEvent>,
    pub dynamic: Option<Dynamic>,
    pub slur: Option<Slur>,
    pub lyric: Option<LyricAnchor>,
    pub beam_group: Option<u8>,
    pub tuplet: Option<Tuplet>,
    /// Explicit beam begin/continue/end markings per beam level.
    /// Level 1 = eighth-note beam, level 2 = sixteenth, etc.
    /// When empty the renderer infers beaming from duration and context.
    pub beams: Vec<Beam>,
    /// Grace notes that sound immediately before this beat.
    pub grace_notes: Vec<GraceNote>,
    /// When `true`, this beat is a cue (shown smaller, not counted in the part's dynamics).
    pub cue: bool,
    /// Chord symbol that begins on this beat. Rendered above the staff.
    pub chord: Option<ChordSymbol>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Duration {
    pub base: NoteValue,
    pub dots: u8, // 0, 1, or 2
    pub tuplet: Option<Tuplet>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Tuplet {
    pub actual: u8, // e.g. 3 (triplet plays 3 notes…)
    pub normal: u8, // …in the time of 2
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Dynamic {
    PPP,
    PP,
    P,
    MP,
    MF,
    F,
    FF,
    FFF,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Slur {
    pub slur_id: u8,
    pub kind: SlurKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SlurKind {
    Start,
    End,
}

/// Binds a beat to a specific syllable in a LyricLine.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct LyricAnchor {
    pub lyric_line_id: LyricLineId,
    pub syllable_index: u16, // index into LyricLine.syllables
}

// ---------------------------------------------------------------------------
// Beams
// ---------------------------------------------------------------------------

/// Explicit beam segment for one beam level on this beat.
///
/// Level 1 spans eighth notes, level 2 sixteenth notes, etc.
/// `ForwardHook` and `BackwardHook` represent partial beams (e.g. on dotted rhythms).
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Beam {
    /// Beam level, 1-based (1 = outermost / thickest beam).
    pub level: u8,
    pub kind: BeamKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum BeamKind {
    Begin,
    Continue,
    End,
    ForwardHook,
    BackwardHook,
}
