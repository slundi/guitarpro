//! Conversion from MusicXML (`ScorePartwise` / `ScoreTimewise`) to
//! `optimized::LoadedScore`.
//!
//! Entry points: [`score_partwise_to_loaded_score`] and
//! [`score_timewise_to_loaded_score`].
//!
//! Sub-modules follow the score hierarchy:
//! - [`metadata`] — score header, instrument definitions, part groups, lyric pre-pass
//! - [`timeline`] — global measure timeline (tempo, key/time sig, barlines, navigation)
//! - [`note`]     — note, beat, and voice construction

pub mod legacy;
mod metadata;
mod note;
pub(crate) mod timeline;

use std::collections::HashMap;

use crate::model::{
    musicxml::{ScorePartwise, ScoreTimewise, measure::MusicData, part_list::PartListItem},
    optimized::{
        LoadedScore,
        global::{MeasureIndex, SCORE_FORMAT_VERSION, Score, StaffId, TrackId},
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
    let tracks = build_tracks(
        src,
        &score_parts,
        &staves,
        &part_staff_ids,
        &mut lyric_coll.counters,
    );

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
            format_version: SCORE_FORMAT_VERSION,
        },
        layout: None,
    }
}

/// Convert a MusicXML [`ScoreTimewise`] document into an [`LoadedScore`].
///
/// `score-timewise` organizes music measure-first then part; `score-partwise`
/// organizes part-first then measure. Both encode identical musical content —
/// the MusicXML specification ships an official XSLT stylesheet to transpose
/// between them. We perform that structural transpose in-model (staying
/// entirely within the `musicxml` types, never touching the legacy `Song`
/// model) and reuse [`score_partwise_to_loaded_score`].
///
/// The value is consumed because the transpose moves the (non-`Clone`) music
/// data out of the timewise measures rather than duplicating it.
pub fn score_timewise_to_loaded_score(src: ScoreTimewise) -> LoadedScore {
    score_partwise_to_loaded_score(&src.into_partwise())
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

        // The first Attributes block with clefs gives the staves.
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

        // The clefs in effect from the start: the first measure's at its
        // start (before any time passes, or after a <backup> back to it, as
        // between voices). A later one is a change (`build_measure_data`,
        // which moves the position alike).
        let mut opening: Vec<&crate::model::musicxml::attributes::Clef> = Vec::new();
        let mut cursor: u32 = 0;
        for event in part.measures.first().map_or(&[][..], |m| &m.music_data[..]) {
            match event {
                MusicData::Attributes(attrs) if cursor == 0 => opening.extend(attrs.clefs.iter()),
                MusicData::Note(n) if n.grace.is_none() && n.chord.is_none() => {
                    cursor += n.duration.unwrap_or(0);
                }
                MusicData::Backup(b) => cursor = cursor.saturating_sub(b.duration),
                MusicData::Forward(f) => cursor += f.duration,
                _ => {}
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
                // A tab staff stays one; a notation staff without an
                // opening clef starts in the default treble. Clef changes
                // are followed on the first staff only: a later staff's
                // first clef stands for all of it.
                let clef_kind = match convert_clef(clef) {
                    Clef::Tab => Clef::Tab,
                    _ => opening
                        .iter()
                        .rev()
                        .find(|c| c.number.unwrap_or(1) == staff_num)
                        .copied()
                        .or((staff_num != 1).then_some(clef))
                        .map_or(Clef::Treble, convert_clef),
                };

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
    let octave = clef.clef_octave_change.unwrap_or(0);
    match clef.sign.as_str() {
        "G" if octave < 0 => Clef::Treble8vb,
        "G" if octave > 0 => Clef::Treble8va,
        "G" => Clef::Treble,
        "F" if octave < 0 => Clef::Bass8vb,
        "F" if octave > 0 => Clef::Bass8va,
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
    staff_defs: &[StaffDef],
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
            let mut carried = note::PartCarry::default();

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
                    &mut carried,
                );
                measures.insert(measure_index, measure_data);
            }
            // Exporters restate the clef at system starts: only a clef
            // other than the one in effect is a change.
            let mut current = staves
                .first()
                .and_then(|id| staff_defs.get(id.0 as usize))
                .map(|staff| staff.clef);
            for data in measures.values_mut() {
                if data.clef == current {
                    data.clef = None;
                } else if data.clef.is_some() {
                    current = data.clef;
                }
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
