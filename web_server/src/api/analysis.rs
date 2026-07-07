use std::collections::{BTreeSet, HashMap};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use guitarpro::analysis::fingering::{FingerRole, suggest_fingering};
use guitarpro::model::optimized::track::MeasureData;
use guitarpro::{DirectionSign, MeasureHeader, NoteType};
use serde::{Deserialize, Serialize};
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
    let sessions = state.sessions.read().await;
    let loaded = sessions
        .get(&id)
        .ok_or_else(|| ApiError::not_found("Score session not found"))?;
    loaded.touch();

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

// ════════════════════════════════════════════════════════════════════════════════
// Form analysis  (ported from cli/src/command_form.rs)
// ════════════════════════════════════════════════════════════════════════════════

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct FormQuery {
    pub track: Option<String>,
    #[serde(default = "form_thresh_default")]
    pub threshold: f64,
    #[serde(default = "form_variant_default")]
    pub variant_threshold: f64,
    #[serde(default = "form_min_section_default")]
    pub min_section: usize,
}

fn form_thresh_default() -> f64 {
    0.75
}
fn form_variant_default() -> f64 {
    0.90
}
fn form_min_section_default() -> usize {
    2
}

// ── JSON output types ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct FormResponse {
    tracks: Vec<FormTrack>,
}

#[derive(Serialize)]
struct FormTrack {
    name: String,
    form: String,
    sections: Vec<FormSection>,
}

#[derive(Serialize)]
struct FormSection {
    label: String,
    bar_start: usize,
    bar_end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn form(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<FormQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let sessions = state.sessions.read().await;
    let loaded = sessions
        .get(&id)
        .ok_or_else(|| ApiError::not_found("Score session not found"))?;
    loaded.touch();

    let threshold = params.threshold.clamp(0.0, 1.0);
    let variant_threshold = params.variant_threshold.clamp(threshold, 1.0);
    let headers = &loaded.song.measure_headers;

    if headers.is_empty() {
        return Ok(Json(FormResponse { tracks: vec![] }));
    }

    let sections = form_segment(headers, params.min_section);

    let mut tracks: Vec<FormTrack> = Vec::new();
    for track in &loaded.song.tracks {
        if params
            .track
            .as_deref()
            .is_some_and(|f| !track.name.to_lowercase().contains(&f.to_lowercase()))
        {
            continue;
        }

        let fps: Vec<SectionFp> = sections
            .iter()
            .map(|s| build_fingerprint(track, s.start, s.end))
            .collect();
        let sims = similarity_matrix(&fps);
        let labeled = assign_labels(&sections, &sims, threshold, variant_threshold);

        let form_str = labeled
            .iter()
            .map(|s| match &s.name {
                Some(n) => format!("[{} {}]", n, s.label),
                None => format!("[{}]", s.label),
            })
            .collect::<Vec<_>>()
            .join(" ");

        tracks.push(FormTrack {
            name: track.name.clone(),
            form: form_str,
            sections: labeled
                .iter()
                .map(|s| {
                    let (bar_start, bar_end) = s.bar_range();
                    FormSection {
                        label: s.label.clone(),
                        bar_start,
                        bar_end,
                        name: s.name.clone(),
                    }
                })
                .collect(),
        });
    }

    Ok(Json(FormResponse { tracks }))
}

// ── Domain types ──────────────────────────────────────────────────────────────

struct FormSectionInner {
    start: usize,
    end: usize,
    name: Option<String>,
}

struct SectionFp {
    pitch_class: [f64; 12],
    note_count: usize,
    len: usize,
}

struct LabeledSection {
    start: usize,
    end: usize,
    name: Option<String>,
    label: String,
}

impl LabeledSection {
    fn bar_range(&self) -> (usize, usize) {
        (self.start + 1, self.end)
    }
}

// ── Segmentation ──────────────────────────────────────────────────────────────

fn form_segment(headers: &[MeasureHeader], min_len: usize) -> Vec<FormSectionInner> {
    let n = headers.len();
    if n == 0 {
        return vec![];
    }

    let mut break_set: BTreeSet<usize> = BTreeSet::new();
    break_set.insert(0);
    break_set.insert(n);

    let mut has_markers = false;
    for (i, h) in headers.iter().enumerate() {
        if h.marker.is_some() && i > 0 {
            break_set.insert(i);
            has_markers = true;
        }
    }

    let has_repeats = headers.iter().any(|h| h.repeat_open || h.repeat_close >= 0);
    if !has_markers && has_repeats {
        for (i, h) in headers.iter().enumerate() {
            if h.repeat_open && i > 0 {
                break_set.insert(i);
            }
            if h.repeat_close >= 0 && i + 1 < n {
                break_set.insert(i + 1);
            }
        }
    }

    let mut breaks: Vec<usize> = break_set.into_iter().collect();

    if breaks.len() <= 2 {
        let w = (n / 4).clamp(4, 16);
        breaks = (0..=n).step_by(w).collect();
        if *breaks.last().unwrap() != n {
            breaks.push(n);
        }
        breaks.dedup();
    }

    breaks = form_merge_short(&breaks, min_len, n);

    breaks
        .windows(2)
        .map(|w| {
            let start = w[0];
            let end = w[1];
            let name = headers[start]
                .marker
                .as_ref()
                .map(|m| form_clean_str(&m.title));
            FormSectionInner { start, end, name }
        })
        .collect()
}

fn form_merge_short(breaks: &[usize], min_len: usize, total: usize) -> Vec<usize> {
    if breaks.len() <= 2 {
        return breaks.to_vec();
    }
    let min_len = min_len.max(1);
    let mut out: Vec<usize> = vec![0];
    for w in breaks.windows(2) {
        if w[1] - w[0] >= min_len {
            out.push(w[1]);
        }
    }
    if *out.last().unwrap() != total {
        out.push(total);
    }
    out.dedup();
    if out.len() <= 1 {
        return vec![0, total];
    }
    out
}

fn form_clean_str(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_owned()
}

// ── Fingerprint ───────────────────────────────────────────────────────────────

fn build_fingerprint(track: &guitarpro::Track, start: usize, end: usize) -> SectionFp {
    let mut pc = [0u32; 12];
    let mut note_count = 0usize;

    for mi in start..end.min(track.measures.len()) {
        for voice in &track.measures[mi].voices {
            for beat in &voice.beats {
                for note in &beat.notes {
                    if note.kind == NoteType::Rest || note.kind == NoteType::Tie {
                        continue;
                    }
                    let midi = if track.percussion_track {
                        note.value
                    } else {
                        let s = note.string as usize;
                        if s > 0 && s <= track.strings.len() {
                            note.value + track.strings[s - 1].1 as i16
                        } else {
                            note.value
                        }
                    };
                    if midi >= 0 {
                        pc[(midi % 12) as usize] += 1;
                        note_count += 1;
                    }
                }
            }
        }
    }

    let total: u32 = pc.iter().sum();
    let pitch_class = if total > 0 {
        pc.map(|c| c as f64 / total as f64)
    } else {
        [0.0f64; 12]
    };

    SectionFp {
        pitch_class,
        note_count,
        len: end - start,
    }
}

// ── Similarity ────────────────────────────────────────────────────────────────

fn fp_similarity(a: &SectionFp, b: &SectionFp) -> f64 {
    if a.note_count == 0 || b.note_count == 0 {
        return if a.note_count == b.note_count {
            1.0
        } else {
            0.0
        };
    }
    let dot: f64 = a
        .pitch_class
        .iter()
        .zip(b.pitch_class.iter())
        .map(|(x, y)| x * y)
        .sum();
    let mag_a: f64 = a.pitch_class.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.pitch_class.iter().map(|x| x * x).sum::<f64>().sqrt();
    let pc_sim = if mag_a * mag_b > 0.0 {
        dot / (mag_a * mag_b)
    } else {
        0.0
    };
    let la = a.len as f64;
    let lb = b.len as f64;
    let len_sim = 1.0 - (la - lb).abs() / la.max(lb).max(1.0);
    0.80 * pc_sim + 0.20 * len_sim
}

fn similarity_matrix(fps: &[SectionFp]) -> Vec<Vec<f64>> {
    let n = fps.len();
    let mut m = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        m[i][i] = 1.0;
        for j in (i + 1)..n {
            let s = fp_similarity(&fps[i], &fps[j]);
            m[i][j] = s;
            m[j][i] = s;
        }
    }
    m
}

// ── Clustering and labels ─────────────────────────────────────────────────────

fn form_find_root(parent: &mut [usize], mut i: usize) -> usize {
    let mut root = i;
    while parent[root] != root {
        root = parent[root];
    }
    while parent[i] != root {
        let next = parent[i];
        parent[i] = root;
        i = next;
    }
    root
}

fn form_cluster(sims: &[Vec<f64>], threshold: f64) -> Vec<usize> {
    let n = sims.len();
    let mut parent: Vec<usize> = (0..n).collect();
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        for j in (i + 1)..n {
            if sims[i][j] >= threshold {
                let ri = form_find_root(&mut parent, i);
                let rj = form_find_root(&mut parent, j);
                if ri != rj {
                    parent[rj] = ri;
                }
            }
        }
    }
    (0..n).map(|i| form_find_root(&mut parent, i)).collect()
}

fn cluster_label(idx: usize) -> String {
    if idx < 26 {
        char::from(b'A' + idx as u8).to_string()
    } else {
        format!(
            "{}{}",
            char::from(b'A' + (idx / 26 - 1) as u8),
            char::from(b'A' + (idx % 26) as u8),
        )
    }
}

fn assign_labels(
    sections: &[FormSectionInner],
    sims: &[Vec<f64>],
    threshold: f64,
    variant_threshold: f64,
) -> Vec<LabeledSection> {
    let n = sections.len();
    if n == 0 {
        return vec![];
    }

    let coarse = form_cluster(sims, threshold);

    let mut root_to_letter: HashMap<usize, usize> = HashMap::new();
    let mut letter_counter = 0usize;
    for &root in &coarse {
        root_to_letter.entry(root).or_insert_with(|| {
            let l = letter_counter;
            letter_counter += 1;
            l
        });
    }

    let mut variant_group: Vec<usize> = vec![0; n];
    let mut cluster_subgroups: HashMap<usize, Vec<usize>> = HashMap::new();

    for i in 0..n {
        let root = coarse[i];
        let subgroups = cluster_subgroups.entry(root).or_default();
        let mut found = None;
        for (sg_idx, &exemplar) in subgroups.iter().enumerate() {
            if sims[exemplar][i] >= variant_threshold {
                found = Some(sg_idx);
                break;
            }
        }
        if let Some(sg_idx) = found {
            variant_group[i] = sg_idx;
        } else {
            let sg_idx = subgroups.len();
            subgroups.push(i);
            variant_group[i] = sg_idx;
        }
    }

    sections
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let root = coarse[i];
            let base_idx = root_to_letter[&root];
            let base = cluster_label(base_idx);
            let vg = variant_group[i];
            let label = if vg == 0 {
                base
            } else {
                format!("{}{}", base, "'".repeat(vg.min(3)))
            };
            LabeledSection {
                start: s.start,
                end: s.end,
                name: s.name.clone(),
                label,
            }
        })
        .collect()
}

// ════════════════════════════════════════════════════════════════════════════════
// Fingering analysis  (Part 5.3)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct FingeringQuery {
    pub track: Option<String>,
}

#[derive(Serialize)]
pub struct FingeringResponse {
    tracks: Vec<FingeringTrack>,
}

#[derive(Serialize)]
struct FingeringTrack {
    name: String,
    measures: Vec<FingeringMeasure>,
}

#[derive(Serialize)]
struct FingeringMeasure {
    measure: u16,
    assignments: Vec<FingeringAssignment>,
}

#[derive(Serialize)]
struct FingeringAssignment {
    string: i8,
    fret: i16,
    finger: u8,
    role: &'static str,
    position_shift: bool,
}

pub async fn fingering(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<FingeringQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let sessions = state.sessions.read().await;
    let loaded = sessions
        .get(&id)
        .ok_or_else(|| ApiError::not_found("Score session not found"))?;
    loaded.touch();

    let filter = params.track.as_deref().map(|s| s.to_lowercase());
    let mut result_tracks: Vec<FingeringTrack> = Vec::new();

    for track in &loaded.score.score.tracks {
        if filter
            .as_deref()
            .is_some_and(|f| !track.name.to_lowercase().contains(f))
        {
            continue;
        }

        let measures: Vec<&MeasureData> = track.measures.values().collect();
        let all_assignments = suggest_fingering(&measures, &[]);

        let mut result_measures: Vec<FingeringMeasure> = Vec::new();
        for (mdata, measure_assignments) in track.measures.values().zip(all_assignments.iter()) {
            if measure_assignments.is_empty() {
                continue;
            }
            result_measures.push(FingeringMeasure {
                measure: mdata.measure_index.0 + 1,
                assignments: measure_assignments
                    .iter()
                    .map(|a| FingeringAssignment {
                        string: a.string,
                        fret: a.fret,
                        finger: a.finger,
                        role: find_role_str(a.role),
                        position_shift: a.position_shift,
                    })
                    .collect(),
            });
        }

        result_tracks.push(FingeringTrack {
            name: track.name.clone(),
            measures: result_measures,
        });
    }

    Ok(Json(FingeringResponse {
        tracks: result_tracks,
    }))
}

fn find_role_str(role: FingerRole) -> &'static str {
    match role {
        FingerRole::Single => "single",
        FingerRole::BarreAnchor => "barre_anchor",
        FingerRole::BarreMember => "barre_member",
    }
}
