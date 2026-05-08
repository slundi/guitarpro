use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    FileHeader,
    global::{MeasureIndex, TrackId},
};

/// User display hints. Since the musical structure will be on a dedicated file, we are storing user display
/// preferences in another file. Example:
/// * music file: `some_song.msor`
/// * user display hints: `some_song.msor_display`
#[derive(Serialize, Deserialize)]
pub struct LayoutFile {
    pub header: FileHeader,
    /// Song file hash to detect desync. If the hash is different, we can warn the user that the layout may be
    /// deprecated.
    pub song_checksum: u32,
    pub hints: Vec<DisplayHint>,
}

#[derive(Serialize, Deserialize)]
pub struct DisplayHint {
    pub measure_index: MeasureIndex,
    pub track_id: Option<TrackId>, // None = global
    pub kind: DisplayHintKind,
}

#[derive(Serialize, Deserialize)]
pub enum DisplayHintKind {
    SystemBreak,
    PageBreak,
    StaffSpacing(f32),
    HideStaff,
    ZoomLevel(f32),
    TrackHeight(u16),
}
