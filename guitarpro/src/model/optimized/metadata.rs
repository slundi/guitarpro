//! Score-level metadata: title, artist, tempo, key/time signatures, chord symbols.

use serde::{Deserialize, Serialize};

use crate::model::optimized::note::Pitch;

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub composer: Option<String>,
    pub year: Option<u16>,
    pub copyright: Option<String>,

    pub master_tempo: f32, // BPM
    pub time_signature: TimeSignature,
    pub key_signature: KeySignature,

    // derived / computed fields
    pub chords: Vec<ChordSymbol>,
    pub scale_hint: Option<Scale>,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8, // 4 = quarter note, 8 = eighth note, ...
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct KeySignature {
    pub root: Pitch,
    pub mode: Mode,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChordSymbol {
    pub root: Pitch,
    pub kind: String,        // e.g. "maj7", "m", "7", "sus4"
    pub bass: Option<Pitch>, // slash chord bass note
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Scale {
    pub root: Pitch,
    pub name: String, // e.g. "pentatonic minor", "blues", "major"
}
