pub mod beat;
pub(crate) fn is_zero_u8(v: &u8) -> bool {
    *v == 0
}
pub mod display;
pub mod effect;
pub mod global;
pub mod metadata;
pub mod note;
pub mod playback;
pub mod timeline;
pub mod track;

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    display::{DisplayHint, LayoutFile},
    global::{MeasureIndex, Score, TrackId},
};

#[derive(Serialize, Deserialize)]
pub struct FileHeader {
    pub magic: [u8; 4], // b"MSOR" "Music Score Optimized Representation"
    pub version: u16,   // version for future migrations
    pub flags: u16,     // bit 0 = has_display, bit 1 = has_lyrics, ...
}

pub struct LoadedScore {
    pub score: Score,
    pub layout: Option<LayoutFile>, // None if no layout file
}

impl LoadedScore {
    pub fn resolve_hint(&self, measure: MeasureIndex, track: TrackId) -> Option<&DisplayHint> {
        self.layout
            .as_ref()?
            .hints
            .iter()
            .find(|h| h.measure_index == measure && h.track_id.is_none_or(|t| t == track))
    }
}
