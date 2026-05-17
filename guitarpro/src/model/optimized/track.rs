//! Track, staff, and per-track measure content.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    beat::Voice,
    global::{InstrumentId, MeasureIndex, StaffId, TrackId},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub instrument: InstrumentId,
    pub staves: Vec<StaffId>, // ordered: treble before bass
    pub measures: std::collections::BTreeMap<MeasureIndex, MeasureData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StaffDef {
    pub clef: Clef,
    pub display: StaffDisplay,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Clef {
    Treble,
    Bass,
    Alto,
    Tenor,
    Percussion,
    Tab,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum StaffDisplay {
    Notation,    // standard staff only
    Tab,         // tablature only
    NotationTab, // both staves (notation above, tab below)
}

/// Per-track, per-measure content. If `repeat` is Some, `voices` is empty.
#[derive(Serialize, Deserialize, Debug)]
pub struct MeasureData {
    pub measure_index: MeasureIndex,
    pub track_id: TrackId,
    pub repeat: Option<MeasureRepeat>, // if Some, voices is empty
    pub voices: HashMap<u8, Voice>,    // voice_id → Voice
    /// GP5 line-break byte (0=None, 1=Break, 2=Protect). Written after each measure in GP5.
    #[serde(default, skip_serializing_if = "crate::model::optimized::is_zero_u8")]
    pub gp_line_break: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MeasureRepeat {
    pub kind: MeasureRepeatKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum MeasureRepeatKind {
    Single, // %  — repeat the previous measure
    Double, // %% — repeat the previous two measures
    Fourth, // %%%% — repeat the previous 4 measures
    Slash,  // /  — repeat the previous beat (jazz notation)
}
