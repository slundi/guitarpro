//! Track, staff, and per-track measure content.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    beat::{Beat, Voice},
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
    /// Treble clef with an 8 below (tenor voice, guitar notation).
    Treble8vb,
    /// Treble clef with an 8 above.
    Treble8va,
    /// Bass clef with an 8 below (double bass).
    Bass8vb,
    /// Bass clef with an 8 above.
    Bass8va,
}

impl Clef {
    /// Octave shift of the clef's 8 mark: `-1` below, `1` above, `0` none.
    pub fn octave_change(self) -> i8 {
        match self {
            Clef::Treble8vb | Clef::Bass8vb => -1,
            Clef::Treble8va | Clef::Bass8va => 1,
            _ => 0,
        }
    }
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
    /// Clef change taking effect at the start of this measure (on the
    /// track's first staff). `None` = the clef in effect continues.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clef: Option<Clef>,
    /// This track's own right barline, where it differs from the
    /// timeline's [`MeasureDef::barline_right`](crate::model::optimized::timeline::MeasureDef::barline_right)
    /// (MuseScore keeps barline styles per staff).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barline_right: Option<crate::model::optimized::timeline::Barline>,
    /// GP5 line-break byte (0=None, 1=Break, 2=Protect). Written after each measure in GP5.
    #[serde(default, skip_serializing_if = "crate::model::optimized::is_zero_u8")]
    pub gp_line_break: u8,
    /// GP7/GPX simile mark for this bar (e.g. "Simple", "FirstOfDouble", "SecondOfDouble").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_simile_mark: Option<String>,
}

impl MeasureData {
    /// Its beats across voices with their voice and index in it, in
    /// (tick, voice) order: the order dynamics and hairpins are paired in.
    pub(crate) fn beats_in_time_order(&self) -> Vec<(u8, usize, &Beat)> {
        let mut beats: Vec<(u8, usize, &Beat)> = self
            .voices
            .iter()
            .flat_map(|(&voice, v)| {
                v.beats
                    .iter()
                    .enumerate()
                    .map(move |(index, beat)| (voice, index, beat))
            })
            .collect();
        beats.sort_by_key(|&(voice, _, beat)| time_order(voice, beat));
        beats
    }

    /// [`MeasureData::beats_in_time_order`], to change them.
    pub(crate) fn beats_in_time_order_mut(&mut self) -> Vec<(u8, &mut Beat)> {
        let mut beats: Vec<(u8, &mut Beat)> = self
            .voices
            .iter_mut()
            .flat_map(|(&voice, v)| v.beats.iter_mut().map(move |beat| (voice, beat)))
            .collect();
        beats.sort_by_key(|(voice, beat)| time_order(*voice, beat));
        beats
    }
}

/// Where a beat in `voice` goes in [`MeasureData::beats_in_time_order`]:
/// by tick, then voice (stable, so in its voice's order).
fn time_order(voice: u8, beat: &Beat) -> (u32, u8) {
    (beat.tick_offset, voice)
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
