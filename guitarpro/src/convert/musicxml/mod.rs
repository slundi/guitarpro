//! Conversion from `legacy::Song` to `musicxml::ScorePartwise`.
//!
//! Entry point: [`song_to_score_partwise`].
//!
//! Sub-modules follow the score hierarchy:
//! - [`metadata`] — identification and part list
//! - [`measure`]  — per-measure attributes, barlines, tempo
//! - [`note`]     — beat/note construction
//! - [`notations`] — technical markings, ornaments, articulations

mod from_optimized;
pub mod helpers;
mod measure;
mod metadata;
mod notations;
mod note;

pub use from_optimized::loaded_score_to_score_partwise;

use crate::model::{legacy::song::Song, musicxml};

/// Divisions per quarter note used throughout the output document.
///
/// Matches `DURATION_QUARTER_TIME` so legacy tick values map 1-to-1.
pub const DIVISIONS: u32 = crate::model::legacy::key_signature::DURATION_QUARTER_TIME as u32;

// ---------------------------------------------------------------------------
// Top-level conversion
// ---------------------------------------------------------------------------

/// Convert a legacy [`Song`] into a MusicXML [`ScorePartwise`] document.
///
/// The conversion is organized in passes:
/// 1. Score metadata (work title, identification)
/// 2. Part list (one `ScorePart` per track)
/// 3. Parts (measures → notes, per track)
pub fn song_to_score_partwise(song: &Song) -> musicxml::ScorePartwise {
    let identification = metadata::build_identification(song);
    let part_list = metadata::build_part_list(song);
    let parts = build_parts(song);

    musicxml::ScorePartwise {
        version: Some("4.0".to_string()),
        work: Some(musicxml::Work {
            work_number: None,
            work_title: Some(song.name.clone()).filter(|s| !s.is_empty()),
            opus: None,
        }),
        movement_number: None,
        movement_title: None,
        identification: Some(identification),
        defaults: None,
        credits: vec![],
        part_list,
        parts,
    }
}

fn build_parts(song: &Song) -> Vec<musicxml::Part> {
    song.tracks
        .iter()
        .enumerate()
        .map(|(track_idx, track)| {
            let part_id = format!("P{}", track_idx + 1);
            let measures = track
                .measures
                .iter()
                .enumerate()
                .map(|(measure_idx, m)| {
                    let header = song
                        .measure_headers
                        .get(m.header_index)
                        .cloned()
                        .unwrap_or_default();
                    measure::build_measure(song, track, track_idx, m, &header, measure_idx)
                })
                .collect();
            musicxml::Part {
                id: part_id,
                measures,
            }
        })
        .collect()
}
