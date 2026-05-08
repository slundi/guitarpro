//! Top-level score structure, ID newtypes, instrument and lyric types.

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    metadata::Metadata,
    note::Pitch,
    timeline::MeasureDef,
    track::{StaffDef, Track},
};

// --- Compact ID newtypes -------------------------------------------------------
// Arenas are plain Vec<T> indexed by id.0 as usize. usize is never stored.

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct InstrumentId(pub u8); // max 255 instruments

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TrackId(pub u8); // max 255 tracks

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StaffId(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct LyricLineId(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MeasureIndex(pub u16); // max 65535 measures; Ord needed for BTreeMap keys

// --- Score (top level) --------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Score {
    pub metadata: Metadata,
    pub instruments: Vec<Instrument>, // indexed by InstrumentId
    pub staves: Vec<StaffDef>,        // indexed by StaffId
    pub tracks: Vec<Track>,           // indexed by TrackId
    pub timeline: Vec<MeasureDef>,    // ordered, shared across all tracks
    pub lyric_lines: Vec<LyricLine>,  // indexed by LyricLineId
    pub lyric_projections: Vec<LyricProjection>,
}

impl Score {
    // fn get_lyrics_for_display(
    //     &self,
    //     display_track: TrackId,
    //     measure_index: MeasureIndex,
    // ) -> Vec<(/*tick*/ usize, &LyricSyllable)> {
    //     self.lyric_projections
    //         .iter()
    //         .filter(|p| p.display_track == display_track)
    //         .flat_map(|p| {
    //             // find LyricAnchor anchor_track for the given measure
    //             let anchor_track = self.tracks[p.anchor_track.0 as usize];
    //             let lyric_line = self.lyric_lines[p.lyric_line_id.0 as usize];
    //             anchor_track.measures[&measure_index]
    //                 .all_beats()
    //                 .filter_map(|beat| {
    //                     beat.lyric
    //                         .as_ref()
    //                         .filter(|a| a.lyric_line_id == p.lyric_line_id)
    //                         .map(|a| (beat.tick_offset, &lyric_line.syllables[a.syllable_index]))
    //                 })
    //         })
    //         .collect()
    // }
}

// --- Instrument ---------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Instrument {
    pub name: String,
    pub midi_program: u8,
    pub midi_channel: u8,
    pub kind: InstrumentKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InstrumentKind {
    Pitched,
    Percussion,
    Stringed {
        tuning: Vec<Pitch>, // low → high, e.g. [E2, A2, D3, G3, B3, E4]
        string_count: u8,
        capo: u8,
    },
}

// --- Lyrics -------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct LyricLine {
    pub label: Option<String>,    // "Verse 1", "Chorus", …
    pub language: Option<String>, // BCP47: "en", "fr", …
    pub syllables: Vec<LyricSyllable>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LyricSyllable {
    pub text: String,
    pub hyphen: bool,     // true if more syllables follow in the same word
    pub line_break: bool, // visual line break in lyric display
}

/// Declares where to visually render a lyric line.
/// `anchor_track` = where LyricAnchors live (usually the vocal track).
/// `display_track` = where to draw the lyrics (can be a guitar tab track).
#[derive(Serialize, Deserialize, Debug)]
pub struct LyricProjection {
    pub lyric_line_id: LyricLineId,
    pub anchor_track: TrackId,
    pub display_track: TrackId,
}
