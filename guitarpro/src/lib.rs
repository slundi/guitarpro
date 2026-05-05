pub mod audio;
pub mod error;
pub mod io;
pub mod model;

// Re-export error types
pub use crate::error::{GpError, GpResult};

// Re-export core types
pub use crate::model::beat::{Beat, Voice};
pub use crate::model::chord::Chord;
pub use crate::model::enums::*;
pub use crate::model::headers::MeasureHeader;
pub use crate::model::key_signature::{KeySignature, TimeSignature};
pub use crate::model::measure::Measure;
pub use crate::model::note::Note;
pub use crate::model::page::PageSetup;
pub use crate::model::song::Song;
pub use crate::model::track::Track;

// Re-export traits for easy use
pub use crate::audio::midi::SongMidiOps;
pub use crate::io::gpif_import::SongGpifOps;
pub use crate::model::beat::SongBeatOps;
pub use crate::model::chord::SongChordOps;
pub use crate::model::effects::SongEffectOps;
pub use crate::model::headers::SongHeaderOps;
pub use crate::model::lyric::SongLyricOps;
pub use crate::model::measure::SongMeasureOps;
pub use crate::model::mix_table::SongMixTableOps;
pub use crate::model::note::SongNoteOps;
pub use crate::model::page::SongPageOps;
pub use crate::model::rse::SongRseOps;
pub use crate::model::track::SongTrackOps;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_audit;
