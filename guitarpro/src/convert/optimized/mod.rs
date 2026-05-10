//! Conversion from `musicxml::ScorePartwise` to `optimized::LoadedScore`.
//!
//! Entry point: [`score_partwise_to_loaded_score`].
//!
//! Sub-modules follow the score hierarchy:
//! - [`metadata`] — score header, instrument definitions, part groups, lyric pre-pass
//! - [`timeline`] — global measure timeline (tempo, key/time sig, barlines, navigation)
//! - [`note`]     — note, beat, and voice construction

mod metadata;
mod note;
mod timeline;

use std::collections::HashMap;

use crate::model::{
    musicxml::{ScorePartwise, measure::MusicData, part_list::PartListItem},
    optimized::{
        LoadedScore,
        global::{MeasureIndex, Score, StaffId, TrackId},
        track::{Clef, MeasureData, StaffDef, StaffDisplay, Track},
    },
};

use note::LyricState;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Convert a MusicXML [`ScorePartwise`] document into an [`LoadedScore`].
pub fn score_partwise_to_loaded_score(src: &ScorePartwise) -> LoadedScore {
    let score_parts = collect_score_parts(src);

    // 1. Global timeline (tempo, key/time sig, barlines) from the first part.
    let timeline_data = timeline::build_timeline(src);

    // 2. Metadata (title, identification, credits, initial tempo/key/time).
    let score_meta = metadata::build_metadata(src, &timeline_data);

    // 3. Score-level rendering defaults.
    let defaults = metadata::build_defaults(src);

    // 4. Instruments and staves.
    let instruments = metadata::build_instruments(src, &score_parts);
    let (staves, part_staff_ids) = build_staves(src, &score_parts);

    // 5. Part groups.
    let groups = metadata::build_groups(src);

    // 6. Lyric pre-pass: collect all syllables so we can reference them by index.
    let mut lyric_coll = metadata::collect_lyrics(src);

    // 7. Tracks (per-part measures, voices, beats, notes).
    let tracks = build_tracks(src, &score_parts, &part_staff_ids, &mut lyric_coll.counters);

    LoadedScore {
        score: Score {
            metadata: score_meta,
            instruments,
            staves,
            tracks,
            groups,
            timeline: timeline_data.measures,
            lyric_lines: lyric_coll.lines,
            lyric_projections: lyric_coll.projections,
            defaults,
        },
        layout: None,
    }
}

// ---------------------------------------------------------------------------
// Helpers: collect ScorePart references from the part-list
// ---------------------------------------------------------------------------

fn collect_score_parts(src: &ScorePartwise) -> Vec<&crate::model::musicxml::part_list::ScorePart> {
    src.part_list
        .items
        .iter()
        .filter_map(|item| {
            if let PartListItem::ScorePart(sp) = item {
                Some(sp)
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Staves
// ---------------------------------------------------------------------------

fn build_staves(
    src: &ScorePartwise,
    score_parts: &[&crate::model::musicxml::part_list::ScorePart],
) -> (Vec<StaffDef>, Vec<Vec<StaffId>>) {
    let mut staves: Vec<StaffDef> = Vec::new();
    let mut part_staff_ids: Vec<Vec<StaffId>> = Vec::new();

    for (part_idx, _sp) in score_parts.iter().enumerate() {
        let Some(part) = src.parts.get(part_idx) else {
            part_staff_ids.push(vec![]);
            continue;
        };

        // Find the first Attributes block.
        let mut clefs: Vec<&crate::model::musicxml::attributes::Clef> = Vec::new();
        let mut staff_details: Vec<&crate::model::musicxml::attributes::StaffDetails> = Vec::new();

        'outer: for measure in &part.measures {
            for event in &measure.music_data {
                if let MusicData::Attributes(attrs) = event {
                    clefs.extend(attrs.clefs.iter());
                    staff_details.extend(attrs.staff_details.iter());
                    if !clefs.is_empty() {
                        break 'outer;
                    }
                }
            }
        }

        let mut ids: Vec<StaffId> = Vec::new();

        if clefs.is_empty() {
            // Default: single treble notation staff.
            let id = StaffId(staves.len() as u8);
            staves.push(StaffDef {
                clef: Clef::Treble,
                display: StaffDisplay::Notation,
            });
            ids.push(id);
        } else {
            let mut sorted: Vec<(u8, &crate::model::musicxml::attributes::Clef)> =
                clefs.iter().map(|c| (c.number.unwrap_or(1), *c)).collect();
            sorted.sort_by_key(|(n, _)| *n);

            for (staff_num, clef) in sorted {
                let clef_kind = convert_clef(clef);

                // Check if this staff number has a TAB staff-type override.
                let is_tab = clef_kind == Clef::Tab
                    || staff_details.iter().any(|sd| {
                        sd.number.unwrap_or(1) == staff_num
                            && sd.staff_type.as_deref() == Some("tab")
                    });

                let display = if is_tab {
                    StaffDisplay::Tab
                } else {
                    StaffDisplay::Notation
                };

                let id = StaffId(staves.len() as u8);
                staves.push(StaffDef {
                    clef: clef_kind,
                    display,
                });
                ids.push(id);
            }
        }

        part_staff_ids.push(ids);
    }

    (staves, part_staff_ids)
}

fn convert_clef(clef: &crate::model::musicxml::attributes::Clef) -> Clef {
    match clef.sign.as_str() {
        "G" => Clef::Treble,
        "F" => Clef::Bass,
        "C" => match clef.line {
            Some(4) => Clef::Tenor,
            _ => Clef::Alto,
        },
        "percussion" => Clef::Percussion,
        "TAB" | "tab" => Clef::Tab,
        _ => Clef::Treble,
    }
}

// ---------------------------------------------------------------------------
// Tracks
// ---------------------------------------------------------------------------

fn build_tracks(
    src: &ScorePartwise,
    score_parts: &[&crate::model::musicxml::part_list::ScorePart],
    part_staff_ids: &[Vec<StaffId>],
    lyric_counters: &mut HashMap<
        (usize, String),
        (crate::model::optimized::global::LyricLineId, u16),
    >,
) -> Vec<Track> {
    score_parts
        .iter()
        .enumerate()
        .map(|(part_idx, _sp)| {
            let track_id = TrackId(part_idx as u8);
            let instrument_id = crate::model::optimized::global::InstrumentId(part_idx as u8);
            let staves = part_staff_ids.get(part_idx).cloned().unwrap_or_default();

            let part = match src.parts.get(part_idx) {
                Some(p) => p,
                None => {
                    return Track {
                        id: track_id,
                        name: format!("Part {}", part_idx + 1),
                        instrument: instrument_id,
                        staves,
                        measures: std::collections::BTreeMap::new(),
                    };
                }
            };

            let name = _sp
                .part_name
                .as_ref()
                .and_then(|n| n.value.clone())
                .unwrap_or_else(|| format!("Part {}", part_idx + 1));

            let mut divisions: u32 = timeline::DEFAULT_DIVISIONS;
            let mut measures: std::collections::BTreeMap<MeasureIndex, MeasureData> =
                std::collections::BTreeMap::new();

            for (measure_idx, measure) in part.measures.iter().enumerate() {
                let measure_index = MeasureIndex(measure_idx as u16);
                let mut lyric_state = LyricState {
                    counters: lyric_counters,
                };

                let measure_data = note::build_measure_data(
                    measure,
                    measure_index,
                    track_id,
                    part_idx,
                    &mut divisions,
                    &mut lyric_state,
                );
                measures.insert(measure_index, measure_data);
            }

            Track {
                id: track_id,
                name,
                instrument: instrument_id,
                staves,
                measures,
            }
        })
        .collect()
}
