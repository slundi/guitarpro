use std::time::Instant;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use guitarpro::DirectionSign;
use serde::Serialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

// ── JSON output types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct RepeatsResponse {
    written_measures: usize,
    sounding_measures: usize,
    sounding_includes_jumps: bool,
    navigation_events: Vec<JsonNavEvent>,
    repeat_blocks: Vec<JsonBlock>,
    play_sequence: Vec<JsonPlayedBar>,
    simile_runs: Vec<JsonSimileRun>,
}

#[derive(Serialize)]
struct JsonNavEvent {
    bar: u16,
    repeat_open: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_close: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    volta: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
}

#[derive(Serialize)]
struct JsonBlock {
    open_bar: usize,
    close_bar: usize,
    total_plays: u32,
    volta_bars: Vec<JsonVoltaBar>,
}

#[derive(Serialize)]
struct JsonVoltaBar {
    bar: usize,
    endings: Vec<u8>,
}

#[derive(Serialize)]
struct JsonPlayedBar {
    bar: u16,
    pass: u32,
}

#[derive(Serialize)]
struct JsonSimileRun {
    track: String,
    bars: String,
    source_bars: String,
    kind: String,
}

// ── Handler ────────────────────────────────────────────────────────────────────

pub async fn repeats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let mut sessions = state.sessions.write().await;
    let loaded = sessions
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("Score session not found"))?;
    loaded.last_accessed = Instant::now();

    let headers = &loaded.song.measure_headers;
    let nav_events = collect_nav_events(headers);
    let blocks = find_repeat_blocks(headers);
    let play_seq = expand_simple_repeats(headers);
    let simile_runs = collect_simile_runs(&loaded.song, None);

    let written = headers.len();
    let sounding = play_seq.len();
    let has_jumps = nav_events.iter().any(|e| {
        e.direction
            .as_deref()
            .map(|d| d.starts_with("D.") || d.starts_with("Da"))
            .unwrap_or(false)
    });

    Ok(Json(RepeatsResponse {
        written_measures: written,
        sounding_measures: sounding,
        sounding_includes_jumps: has_jumps,
        navigation_events: nav_events,
        repeat_blocks: blocks
            .iter()
            .map(|b| JsonBlock {
                open_bar: b.open_idx + 1,
                close_bar: b.close_idx + 1,
                total_plays: b.total_plays,
                volta_bars: b
                    .voltas
                    .iter()
                    .map(|(idx, v)| JsonVoltaBar {
                        bar: idx + 1,
                        endings: v.clone(),
                    })
                    .collect(),
            })
            .collect(),
        play_sequence: play_seq
            .iter()
            .map(|pb| JsonPlayedBar {
                bar: pb.bar_number,
                pass: pb.pass,
            })
            .collect(),
        simile_runs: simile_runs
            .iter()
            .map(|r| {
                let run_end = r.run_start + r.run_len;
                let src_end = r.source_start + r.run_len;
                JsonSimileRun {
                    track: r.track_name.clone(),
                    bars: format!("{}-{}", r.run_start + 1, run_end),
                    source_bars: format!("{}-{}", r.source_start + 1, src_end),
                    kind: r.kind.clone(),
                }
            })
            .collect(),
    }))
}

// ── Domain types ───────────────────────────────────────────────────────────────

struct RepeatBlock {
    open_idx: usize,
    close_idx: usize,
    total_plays: u32,
    voltas: Vec<(usize, Vec<u8>)>,
}

struct PlayedBar {
    bar_number: u16,
    pass: u32,
}

struct SimileRun {
    track_name: String,
    run_start: usize,
    run_len: usize,
    source_start: usize,
    kind: String,
}

// ── Analysis helpers (ported from cli/src/command_repeats.rs) ─────────────────

fn collect_nav_events(headers: &[guitarpro::MeasureHeader]) -> Vec<JsonNavEvent> {
    headers
        .iter()
        .filter(|mh| {
            mh.repeat_open
                || mh.repeat_close >= 0
                || mh.repeat_alternative != 0
                || mh.direction.is_some()
                || mh.marker.is_some()
        })
        .map(|mh| JsonNavEvent {
            bar: mh.number,
            repeat_open: mh.repeat_open,
            repeat_close: if mh.repeat_close >= 0 {
                Some((mh.repeat_close + 1) as u32)
            } else {
                None
            },
            volta: decode_volta_mask(mh.repeat_alternative),
            direction: mh.direction.as_ref().map(direction_label),
            marker: mh.marker.as_ref().map(|m| m.title.clone()),
        })
        .collect()
}

fn decode_volta_mask(bits: u8) -> Vec<u8> {
    (0u8..8)
        .filter(|&i| bits & (1 << i) != 0)
        .map(|i| i + 1)
        .collect()
}

fn direction_label(d: &DirectionSign) -> String {
    match d {
        DirectionSign::Coda => "Coda".to_owned(),
        DirectionSign::DoubleCoda => "Double Coda".to_owned(),
        DirectionSign::Segno => "Segno".to_owned(),
        DirectionSign::SegnoSegno => "Segno Segno".to_owned(),
        DirectionSign::Fine => "Fine".to_owned(),
        DirectionSign::DaCapo => "Da Capo".to_owned(),
        DirectionSign::DaCapoAlCoda => "D.C. al Coda".to_owned(),
        DirectionSign::DaCapoAlDoubleCoda => "D.C. al Double Coda".to_owned(),
        DirectionSign::DaCapoAlFine => "D.C. al Fine".to_owned(),
        DirectionSign::DaSegno => "D.S.".to_owned(),
        DirectionSign::DaSegnoAlCoda => "D.S. al Coda".to_owned(),
        DirectionSign::DaSegnoAlDoubleCoda => "D.S. al Double Coda".to_owned(),
        DirectionSign::DaSegnoAlFine => "D.S. al Fine".to_owned(),
        DirectionSign::DaSegnoSegno => "D.S.S.".to_owned(),
        DirectionSign::DaSegnoSegnoAlCoda => "D.S.S. al Coda".to_owned(),
        DirectionSign::DaSegnoSegnoAlDoubleCoda => "D.S.S. al Double Coda".to_owned(),
        DirectionSign::DaSegnoSegnoAlFine => "D.S.S. al Fine".to_owned(),
        DirectionSign::DaCoda => "Da Coda".to_owned(),
        DirectionSign::DaDoubleCoda => "Da Double Coda".to_owned(),
    }
}

fn find_repeat_blocks(headers: &[guitarpro::MeasureHeader]) -> Vec<RepeatBlock> {
    let mut blocks = Vec::new();
    let mut open_stack: Vec<usize> = Vec::new();

    for (i, mh) in headers.iter().enumerate() {
        if mh.repeat_open {
            open_stack.push(i);
        }
        if mh.repeat_close >= 0 {
            let open_idx = open_stack.pop().unwrap_or(0);
            let total_plays = (mh.repeat_close + 1) as u32;
            let voltas: Vec<(usize, Vec<u8>)> = headers[open_idx..=i]
                .iter()
                .enumerate()
                .filter(|(_, h)| h.repeat_alternative != 0)
                .map(|(rel, h)| (open_idx + rel, decode_volta_mask(h.repeat_alternative)))
                .collect();
            blocks.push(RepeatBlock {
                open_idx,
                close_idx: i,
                total_plays,
                voltas,
            });
        }
    }
    blocks
}

fn expand_simple_repeats(headers: &[guitarpro::MeasureHeader]) -> Vec<PlayedBar> {
    let mut stack: Vec<(usize, u32, u32)> = Vec::new();
    let mut result: Vec<PlayedBar> = Vec::new();
    let mut i = 0usize;
    const SAFETY: usize = 10_000;

    while i < headers.len() && result.len() < SAFETY {
        let mh = &headers[i];

        if mh.repeat_open && !stack.iter().any(|(s, _, _)| *s == i) {
            stack.push((i, 1, 1));
        }
        if mh.repeat_close >= 0 && stack.is_empty() {
            stack.push((0, 1, 1));
        }

        let pass = stack.last().map(|(_, p, _)| *p).unwrap_or(1);

        if !stack.is_empty()
            && mh.repeat_alternative != 0
            && (mh.repeat_alternative >> (pass - 1)) & 1 == 0
        {
            if mh.repeat_close >= 0 && stack.last().is_some_and(|(_, p, t)| p >= t) {
                stack.pop();
            }
            i += 1;
            continue;
        }

        result.push(PlayedBar {
            bar_number: mh.number,
            pass,
        });

        if mh.repeat_close >= 0 {
            let total = (mh.repeat_close + 1) as u32;
            if let Some(top) = stack.last_mut() {
                top.2 = total;
                if top.1 < total {
                    let go_to = top.0;
                    top.1 += 1;
                    i = go_to;
                    continue;
                } else {
                    stack.pop();
                }
            }
        }

        i += 1;
    }

    result
}

fn collect_simile_runs(song: &guitarpro::Song, track_filter: Option<&str>) -> Vec<SimileRun> {
    let mut runs = Vec::new();

    for track in &song.tracks {
        if track_filter.is_some_and(|f| !track.name.to_lowercase().contains(&f.to_lowercase())) {
            continue;
        }

        let measures = &track.measures;
        let mut i = 0usize;

        while i < measures.len() {
            let sm = match &measures[i].simile_mark {
                Some(s) => s.as_str(),
                None => {
                    i += 1;
                    continue;
                }
            };

            let (step, kind_label) = match sm {
                "Simple" => (1usize, "1-bar (Simple)"),
                "FirstOfDouble" | "SecondOfDouble" => (2usize, "2-bar (Double)"),
                other => {
                    runs.push(SimileRun {
                        track_name: track.name.clone(),
                        run_start: i,
                        run_len: 1,
                        source_start: i.saturating_sub(1),
                        kind: format!("unknown ({})", other),
                    });
                    i += 1;
                    continue;
                }
            };
            let _ = step; // used only for source_start calculation below

            let run_start = i;
            let first_mark_kind = sm;
            while i < measures.len() {
                match measures[i].simile_mark.as_deref() {
                    Some("Simple") if first_mark_kind == "Simple" => i += 1,
                    Some("FirstOfDouble" | "SecondOfDouble")
                        if first_mark_kind == "FirstOfDouble"
                            || first_mark_kind == "SecondOfDouble" =>
                    {
                        i += 1
                    }
                    _ => break,
                }
            }

            let run_len = i - run_start;
            let source_start = run_start.saturating_sub(run_len / step * step);

            runs.push(SimileRun {
                track_name: track.name.clone(),
                run_start,
                run_len,
                source_start,
                kind: kind_label.to_owned(),
            });
        }
    }

    runs
}
