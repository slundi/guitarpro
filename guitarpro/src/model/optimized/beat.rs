//! Voice, Beat, Duration, and related rhythm/articulation types.

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    effect::BeatEvent,
    global::LyricLineId,
    note::{Note, NoteValue},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Voice {
    pub voice_id: u8,
    pub beats: Vec<Beat>,
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
