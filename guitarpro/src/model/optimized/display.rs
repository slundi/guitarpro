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

// ---------------------------------------------------------------------------
// Score-level rendering defaults (part of the canonical score, not user hints)
// ---------------------------------------------------------------------------

/// Canonical rendering defaults embedded in the score file.
///
/// These express the engraver's intent (page size, fonts, line widths).
/// User overrides live in [`LayoutFile`].
#[derive(Serialize, Deserialize, Debug)]
pub struct ScoreDefaults {
    /// Physical page dimensions and margins.
    pub page_layout: Option<PageLayout>,
    /// Conversion factor between staff-space tenths and real-world millimetres.
    pub scaling: Option<Scaling>,
    pub music_font: Option<FontDef>,
    pub word_font: Option<FontDef>,
    pub lyric_font: Option<FontDef>,
    /// BCP 47 language tag for lyric text (e.g. `"en"`, `"fr"`).
    pub lyric_language: Option<String>,
    pub appearance: Option<Appearance>,
}

/// Mapping between staff-space tenths and real-world millimetres.
///
/// All distance values in `PageLayout` and `Appearance` are in tenths.
/// To convert: `mm = tenths_value * (millimeters / tenths)`.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Scaling {
    /// Real-world size of `tenths` staff-space units, in millimetres.
    pub millimeters: f32,
    pub tenths: f32,
}

/// Physical page size and margins (all values in millimetres).
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct PageLayout {
    pub width_mm: f32,
    pub height_mm: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
}

/// Font family, size, and style.
#[derive(Serialize, Deserialize, Debug)]
pub struct FontDef {
    pub family: Option<String>,
    /// Point size.
    pub size: Option<f32>,
    pub bold: bool,
    pub italic: bool,
}

/// Engraving appearance overrides for line widths, note sizes, and SMuFL glyphs.
#[derive(Serialize, Deserialize, Debug)]
pub struct Appearance {
    /// Named line widths in tenths. Keys match MusicXML `line-width` type values:
    /// `"beam"`, `"staff"`, `"stem"`, `"bar"`, `"leger"`, `"slur middle"`, etc.
    pub line_widths: Vec<(String, f32)>,
    /// Named note-size percentages relative to the normal note head (100.0 = normal).
    /// Keys: `"cue"`, `"grace"`, `"grace-cue"`.
    pub note_sizes: Vec<(String, f32)>,
    /// SMuFL glyph overrides: `(glyph-type, smufl-glyph-name)`.
    /// E.g. `("g-clef", "gClefSmall")`.
    pub smufl_overrides: Vec<(String, String)>,
}
