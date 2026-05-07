pub mod audio;
pub mod error;
pub mod io;
pub mod model;

// Re-export error types
pub use crate::error::{GpError, GpResult};

// Re-export core types
pub use crate::model::legacy::beat::{Beat, Voice};
pub use crate::model::legacy::chord::Chord;
pub use crate::model::legacy::enums::*;
pub use crate::model::legacy::headers::MeasureHeader;
pub use crate::model::legacy::key_signature::{KeySignature, TimeSignature};
pub use crate::model::legacy::measure::Measure;
pub use crate::model::legacy::note::Note;
pub use crate::model::legacy::page::PageSetup;
pub use crate::model::legacy::song::Song;
pub use crate::model::legacy::track::Track;

// Re-export traits for easy use
pub use crate::audio::midi::SongMidiOps;
pub use crate::io::gpif_export::SongGpifExportOps;
pub use crate::io::gpif_import::SongGpifOps;
pub use crate::model::legacy::beat::SongBeatOps;
pub use crate::model::legacy::chord::SongChordOps;
pub use crate::model::legacy::effects::SongEffectOps;
pub use crate::model::legacy::headers::SongHeaderOps;
pub use crate::model::legacy::lyric::SongLyricOps;
pub use crate::model::legacy::measure::SongMeasureOps;
pub use crate::model::legacy::mix_table::SongMixTableOps;
pub use crate::model::legacy::note::SongNoteOps;
pub use crate::model::legacy::page::SongPageOps;
pub use crate::model::legacy::rse::SongRseOps;
pub use crate::model::legacy::track::SongTrackOps;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_audit;
