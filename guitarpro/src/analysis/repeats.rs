//! Measure repeat (simile mark) detection for the optimized model.
//!
//! Detects consecutive measures that are musically identical and maps them to
//! the standard notation shorthand:
//!
//! | Symbol | Meaning |
//! |--------|---------|
//! | `%`    | Repeat the previous measure  (`Single`) |
//! | `%%`   | Repeat the previous two measures (`Double`) |
//! | `%%%%` | Repeat the previous four measures (`Fourth`) |
//!
//! Also reads a score's repeat barlines, endings and direction signs
//! and the order a score plays in ([`repeats_analysis`]), and a song's
//! simile marks ([`simile_runs`]).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::model::directions::direction_to_jump;
use crate::model::legacy::headers::MeasureHeader;
use crate::model::legacy::{song::Song, track::Track as LegacyTrack};
use crate::model::optimized::{
    beat::{Beat, Voice},
    global::MeasureIndex,
    note::{Note, NoteValue, PitchStep, TieType},
    timeline::{Arrangement, EndingPlan, JumpKind, MeasureDef, ending_spans},
    track::{MeasureData, MeasureRepeat, MeasureRepeatKind, Track},
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Which simile symbol applies to a measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimileMark {
    /// `%` — repeat the previous measure.
    Single,
    /// `%%` — repeat the previous two measures.
    Double,
    /// `%%%%` — repeat the previous four measures.
    Fourth,
}

/// Detect which measures can be replaced with simile marks.
///
/// Returns a `BTreeMap<MeasureIndex, SimileMark>` covering only the measures
/// that should be annotated; all others retain their original content.
///
/// Rules:
/// - Measures that already carry a `MeasureRepeat` in the input are ignored
///   (they can neither serve as a template nor receive a new mark).
/// - A measure at position > 0 whose `MeasureDef` introduces a new
///   time-signature cannot carry a simile mark.
/// - For `Double` / `Fourth`, all template measures must be original (not
///   themselves annotated in this pass, except as `Single`).
/// - `Single` marks may chain: `A %` followed by the same content again
///   gets another `%` even though its template is already annotated.
pub fn detect_repeats(
    track: &Track,
    timeline: &[MeasureDef],
) -> BTreeMap<MeasureIndex, SimileMark> {
    let measure_indices: Vec<MeasureIndex> = track.measures.keys().copied().collect();
    let n = measure_indices.len();

    // Build a set of MeasureIndex values that open a new time signature.
    let time_sig_change: BTreeMap<MeasureIndex, bool> = timeline
        .iter()
        .map(|md| (md.index, md.time_signature.is_some()))
        .collect();

    // Pre-compute fingerprints from original track data (before any annotation).
    // `None` means the measure already has a repeat or is otherwise unsuitable.
    let fingerprints: Vec<Option<MeasureFingerprint>> = measure_indices
        .iter()
        .map(|idx| track.measures.get(idx).and_then(fingerprint_measure))
        .collect();

    let mut result: BTreeMap<MeasureIndex, SimileMark> = BTreeMap::new();
    // Track which positions we have annotated in this pass.
    let mut annotated = vec![false; n];

    let mut i = 0;
    while i < n {
        let idx = measure_indices[i];

        // A time-signature change (other than at the very first measure) blocks all marks.
        if i > 0 && *time_sig_change.get(&idx).unwrap_or(&false) {
            i += 1;
            continue;
        }

        // --- Try Fourth (i..i+4 repeats i-4..i) ---------------------------------
        if i >= 4 && i + 4 <= n {
            // All four template measures must be non-annotated.
            let template_ok = (i - 4..i).all(|j| !annotated[j]);
            let fp_ok = template_ok
                && (0..4).all(|k| {
                    fingerprints[i - 4 + k].is_some()
                        && fingerprints[i + k].is_some()
                        && fingerprints[i - 4 + k] == fingerprints[i + k]
                });
            if fp_ok {
                // No time-sig change is allowed inside the repeated block.
                let inner_ok = (1..4).all(|k| {
                    !*time_sig_change
                        .get(&measure_indices[i + k])
                        .unwrap_or(&false)
                });
                if inner_ok {
                    for k in 0..4 {
                        result.insert(measure_indices[i + k], SimileMark::Fourth);
                        annotated[i + k] = true;
                    }
                    i += 4;
                    continue;
                }
            }
        }

        // --- Try Double (i..i+2 repeats i-2..i) ----------------------------------
        if i >= 2 && i + 2 <= n {
            let template_ok = (i - 2..i).all(|j| !annotated[j]);
            let fp_ok = template_ok
                && fingerprints[i - 2].is_some()
                && fingerprints[i].is_some()
                && fingerprints[i - 2] == fingerprints[i]
                && fingerprints[i - 1].is_some()
                && fingerprints[i + 1].is_some()
                && fingerprints[i - 1] == fingerprints[i + 1];
            if fp_ok {
                let inner_ok = !*time_sig_change
                    .get(&measure_indices[i + 1])
                    .unwrap_or(&false);
                if inner_ok {
                    result.insert(measure_indices[i], SimileMark::Double);
                    result.insert(measure_indices[i + 1], SimileMark::Double);
                    annotated[i] = true;
                    annotated[i + 1] = true;
                    i += 2;
                    continue;
                }
            }
        }

        // --- Try Single (i repeats i-1) ------------------------------------------
        if i >= 1 {
            let prev_idx = measure_indices[i - 1];
            // Allow the previous measure to be already annotated as Single (chaining).
            let template_ok =
                !annotated[i - 1] || result.get(&prev_idx) == Some(&SimileMark::Single);
            if template_ok
                && fingerprints[i - 1].is_some()
                && fingerprints[i].is_some()
                && fingerprints[i - 1] == fingerprints[i]
            {
                result.insert(idx, SimileMark::Single);
                annotated[i] = true;
                i += 1;
                continue;
            }
        }

        i += 1;
    }

    result
}

/// Apply detected simile marks to the track in-place.
///
/// For each `(MeasureIndex, SimileMark)` in `repeats`, the corresponding
/// `MeasureData` has its voices cleared and a `MeasureRepeat` attached.
/// Measures absent from the map are left unchanged.
pub fn apply_repeats(track: &mut Track, repeats: &BTreeMap<MeasureIndex, SimileMark>) {
    for (idx, mark) in repeats {
        if let Some(md) = track.measures.get_mut(idx) {
            md.voices.clear();
            md.repeat = Some(MeasureRepeat {
                kind: match mark {
                    SimileMark::Single => MeasureRepeatKind::Single,
                    SimileMark::Double => MeasureRepeatKind::Double,
                    SimileMark::Fourth => MeasureRepeatKind::Fourth,
                },
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Navigation of a song's measure headers
// ---------------------------------------------------------------------------

/// A measure's navigation marks: its repeat barlines, ending, direction
/// signs and marker.
#[derive(Debug, Clone, Serialize)]
pub struct NavEvent {
    /// 1-based bar number
    pub bar: u16,
    pub repeat_open: bool,
    /// None = no close; Some(n) = the close's written count (2 without
    /// one), as [`RepeatBlock::total_plays`]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_close: Option<u32>,
    /// The passes the close plays, as [`RepeatBlock::plays`]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_plays: Option<u32>,
    /// 1-based volta numbers decoded from bitmask
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub volta: Vec<u8>,
    /// One direction sign, the one its reader kept before `directions`
    /// had all of them (GP6+: a target over a jump, GP3–5: a jump over
    /// a target), for clients from before that; none in a song from
    /// another format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// A `|: … :|` repeat in a song's measure headers. Serialized with 1-based
/// bar numbers: `open_bar`, `close_bar`, `total_plays`, `plays` and `volta_bars`
/// (each `bar` and its `endings`).
#[derive(Debug, Clone)]
pub struct RepeatBlock {
    /// 0-based index of the measure with `repeat_open`
    pub open_idx: usize,
    /// 0-based index of the measure with `repeat_close`
    pub close_idx: usize,
    /// Total plays as written: the close's repeat count (2 without one).
    pub total_plays: u32,
    /// The passes the play order plays ([`Arrangement::resolve`]): a
    /// repeat with endings goes on to its last ending's number, a close in
    /// an ending sending it back only on that ending's passes.
    pub plays: u32,
    /// Volta bracket measures: (0-based idx, volta_numbers)
    pub voltas: Vec<(usize, Vec<u8>)>,
}

impl Serialize for RepeatBlock {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct VoltaBar<'a> {
            bar: usize,
            endings: &'a [u8],
        }
        #[derive(Serialize)]
        struct Block<'a> {
            open_bar: usize,
            close_bar: usize,
            total_plays: u32,
            plays: u32,
            volta_bars: Vec<VoltaBar<'a>>,
        }
        Block {
            open_bar: self.open_idx + 1,
            close_bar: self.close_idx + 1,
            total_plays: self.total_plays,
            plays: self.plays,
            volta_bars: self
                .voltas
                .iter()
                .map(|(index, endings)| VoltaBar {
                    bar: index + 1,
                    endings,
                })
                .collect(),
        }
        .serialize(serializer)
    }
}

/// The label of the one sign a measure had before it kept all of them,
/// as the reader of a song of `version` took it: the GPIF reader (GP6 and
/// later) a target over a jump, the GP3–5 one a jump over a target; `None`
/// without any. Labelled as [`NavEvent::directions`] are
/// ([`DirectionSign::label`](crate::model::legacy::enums::DirectionSign::label)).
pub(crate) fn direction_label(header: &MeasureHeader, version: (u8, u8, u8)) -> Option<String> {
    let sign = if version.0 >= 6 {
        header.directions.first()
    } else {
        header.directions.last()
    };
    sign.map(|sign| sign.label().to_owned())
}

/// Whether a song has jumps (D.S.S., al Double Coda, …) that
/// [`RepeatsAnalysis::sequence`] can't follow: the model has a single segno
/// and coda.
pub fn has_unfollowed_jumps(headers: &[MeasureHeader]) -> bool {
    headers.iter().any(has_unfollowed_jump)
}

/// Whether a measure has a jump the model can't hold
/// ([`has_unfollowed_jumps`]).
fn has_unfollowed_jump(header: &MeasureHeader) -> bool {
    header
        .directions
        .iter()
        .any(|sign| !sign.is_target() && direction_to_jump(sign).is_none())
}

/// A score's navigation marks, repeats and play order.
#[derive(Debug, Clone)]
pub struct RepeatsAnalysis {
    /// The navigation marks of the measures that have any: repeats and
    /// endings from the score's `timeline` (a song loses other formats'
    /// endings past pass 8 and their repeat counts past 127), direction
    /// signs and markers from the song's `headers`, read from a GP file of
    /// `version`. With `version` `None` (a song from another format), its
    /// events have no [`NavEvent::direction`] (its readers never kept one).
    pub nav_events: Vec<NavEvent>,
    /// The repeats in the `timeline`, each closed at a repeat close from
    /// the last open one; without one, as [`Arrangement::resolve`] plays
    /// it: from the start, or after the last repeat and the endings right
    /// after it.
    pub blocks: Vec<RepeatBlock>,
    /// The bars in the order the score plays them, through its repeats,
    /// endings and jumps ([`Arrangement::resolve`]).
    pub sequence: PlaySequence,
    /// Whether it has jumps the sequence leaves out: ones the model can't
    /// hold ([`has_unfollowed_jumps`]: D.S.S., double coda) in a bar it
    /// plays, or a D.C./D.S.
    /// it can't follow ([`PlaySequence::skips_jumps`]).
    pub skips_jumps: bool,
}

/// The most played bars a [`RepeatsReport`] lists: large repeat counts in
/// nested repeats can make a short score play a million bars.
pub const MAX_PLAY_SEQUENCE: usize = 10_000;

/// A song's repeats as the CLI's `repeats --json` and the web server's
/// repeats endpoint give them.
#[derive(Debug, Clone, Serialize)]
pub struct RepeatsReport {
    pub written_measures: usize,
    /// How many bars it plays (all of them, past [`MAX_PLAY_SEQUENCE`]).
    pub sounding_measures: usize,
    /// Whether the sounding measures follow a D.C./D.S. jump.
    pub sounding_includes_jumps: bool,
    /// Whether it has jumps the sounding measures leave out
    /// ([`RepeatsAnalysis::skips_jumps`]).
    pub sounding_skips_jumps: bool,
    /// Whether the sounding measures were cut short at the resolver's
    /// safety guard (repeats that don't resolve to an end).
    pub sounding_truncated: bool,
    pub navigation_events: Vec<NavEvent>,
    pub repeat_blocks: Vec<RepeatBlock>,
    /// The first [`MAX_PLAY_SEQUENCE`] of the sounding measures.
    pub play_sequence: Vec<PlayedBar>,
    /// Whether `play_sequence` stops short of `sounding_measures`.
    pub play_sequence_truncated: bool,
    pub simile_runs: Vec<SimileRun>,
}

impl RepeatsReport {
    /// The report of `song`, its repeats and endings from `timeline` (see
    /// [`repeats_analysis`], `version` the GP file's it was read from, if
    /// any), its simile runs on the tracks matching `track_filter`
    /// ([`simile_runs`]).
    pub fn new(
        song: &Song,
        timeline: &[MeasureDef],
        version: Option<(u8, u8, u8)>,
        track_filter: Option<&str>,
    ) -> Self {
        Self::from_parts(
            &song.measure_headers,
            timeline,
            version,
            simile_runs(song, track_filter),
        )
    }

    /// [`RepeatsReport::new`], given just the parts of the song it reads:
    /// its measure `headers` and its simile runs (so a caller can take
    /// those, not the whole song, off to another thread).
    pub fn from_parts(
        headers: &[MeasureHeader],
        timeline: &[MeasureDef],
        version: Option<(u8, u8, u8)>,
        simile_runs: Vec<SimileRun>,
    ) -> Self {
        let RepeatsAnalysis {
            nav_events,
            blocks,
            sequence,
            skips_jumps,
        } = repeats_analysis(headers, timeline, version, Some(MAX_PLAY_SEQUENCE));
        RepeatsReport {
            written_measures: headers.len(),
            sounding_measures: sequence.played,
            sounding_includes_jumps: sequence.follows_jumps,
            sounding_skips_jumps: skips_jumps,
            sounding_truncated: sequence.truncated,
            navigation_events: nav_events,
            repeat_blocks: blocks,
            play_sequence_truncated: sequence.played > sequence.bars.len(),
            play_sequence: sequence.bars,
            simile_runs,
        }
    }
}

/// A score's [`RepeatsAnalysis`], working out the timeline's endings once
/// for all three parts. The sequence keeps only its first `max_bars`
/// bars, if given (a short score with long nested repeats plays up to a
/// million), still counting them all ([`PlaySequence::played`]).
pub fn repeats_analysis(
    headers: &[MeasureHeader],
    timeline: &[MeasureDef],
    version: Option<(u8, u8, u8)>,
    max_bars: Option<usize>,
) -> RepeatsAnalysis {
    let spans = ending_spans(timeline);
    let plan = EndingPlan::new(timeline, &spans);
    let sequence = play_sequence(timeline, &plan, max_bars);
    RepeatsAnalysis {
        nav_events: nav_events(headers, timeline, version, &plan),
        blocks: repeat_blocks(timeline, &plan),
        // Only a jump in a bar it plays is left out: one in a bar no pass
        // reaches (an ending every pass skips) isn't missed. A sequence cut
        // short at the guard didn't get to say which bars it reaches: any
        // such jump counts.
        skips_jumps: sequence.skips_jumps
            || if sequence.truncated {
                has_unfollowed_jumps(headers)
            } else {
                headers
                    .iter()
                    .zip(&sequence.reached)
                    .any(|(header, &reached)| reached && has_unfollowed_jump(header))
            },
        sequence,
    }
}

/// [`RepeatsAnalysis::nav_events`], given how the endings play.
fn nav_events(
    headers: &[MeasureHeader],
    timeline: &[MeasureDef],
    version: Option<(u8, u8, u8)>,
    plan: &EndingPlan<'_>,
) -> Vec<NavEvent> {
    let endings = &plan.endings;
    // Every measure of either (they're the same score's, as long).
    (0..timeline.len().max(headers.len()))
        .filter_map(|index| {
            let measure = timeline.get(index);
            let header = headers.get(index);
            let repeat_open = measure.is_some_and(|m| m.has_navigation(JumpKind::RepeatOpen));
            let count = measure.and_then(repeat_close_count);
            let repeat_close = count.map(|count| u32::from(count.unwrap_or(2)));
            let repeat_plays = count.map(|count| u32::from(plan.played_passes(index, count)));
            let directions = header
                .map(MeasureHeader::direction_labels)
                .unwrap_or_default();
            let marker = header
                .and_then(|header| header.marker.as_ref())
                .map(|marker| marker.title.clone());
            let volta = endings
                .get(index)
                .copied()
                .flatten()
                .map(<[u8]>::to_vec)
                .unwrap_or_default();
            (repeat_open
                || repeat_close.is_some()
                || !volta.is_empty()
                || !directions.is_empty()
                || marker.is_some())
            .then(|| NavEvent {
                bar: u16::try_from(index + 1).unwrap_or(u16::MAX),
                repeat_open,
                repeat_close,
                repeat_plays,
                volta,
                direction: header
                    .zip(version)
                    .and_then(|(header, version)| direction_label(header, version)),
                directions,
                marker,
            })
        })
        .collect()
}

/// The repeat count of a measure's repeat close (`None` without one), if
/// it has one.
fn repeat_close_count(measure: &MeasureDef) -> Option<Option<u8>> {
    measure
        .navigation
        .iter()
        .find(|event| event.kind == JumpKind::RepeatClose)
        .map(|event| event.repeat_count)
}

/// [`RepeatsAnalysis::blocks`], given each measure's ending passes.
fn repeat_blocks(timeline: &[MeasureDef], plan: &EndingPlan<'_>) -> Vec<RepeatBlock> {
    let endings = &plan.endings;
    let mut blocks = Vec::new();
    let mut open_stack: Vec<usize> = Vec::new(); // stack of open_idx
    // Where a repeat closed without an open one starts.
    let mut implicit_open = 0usize;

    for (i, measure) in timeline.iter().enumerate() {
        if measure.has_navigation(JumpKind::RepeatOpen) {
            open_stack.push(i);
        }
        if let Some(count) = repeat_close_count(measure) {
            let total_plays = u32::from(count.unwrap_or(2));
            // As the play order plays it: a repeat with endings plays each.
            let plays = u32::from(plan.played_passes(i, count));
            let open_idx = open_stack.pop().unwrap_or_else(|| {
                (implicit_open..i)
                    .find(|&index| endings[index].is_none())
                    .unwrap_or(i)
            });
            let voltas: Vec<(usize, Vec<u8>)> = (open_idx..=i)
                .filter_map(|index| Some((index, endings[index]?.to_vec())))
                .collect();
            blocks.push(RepeatBlock {
                open_idx,
                close_idx: i,
                total_plays,
                plays,
                voltas,
            });
            if open_stack.is_empty() {
                implicit_open = i + 1;
            }
        }
    }
    blocks
}

/// One bar in the order a score plays.
#[derive(Debug, Clone, Serialize)]
pub struct PlayedBar {
    /// 1-based bar number
    #[serde(rename = "bar")]
    pub bar_number: u16,
    /// 1-based pass number within the innermost active repeat block
    /// ([`Arrangement::passes`])
    pub pass: u32,
}

/// The bars in the order a score plays them.
#[derive(Debug, Clone)]
pub struct PlaySequence {
    /// The bars it plays, in order: the first `max_bars` given to
    /// [`repeats_analysis`].
    pub bars: Vec<PlayedBar>,
    /// How many bars it plays (more than `bars` holds when cut to
    /// `max_bars`).
    pub played: usize,
    /// Per measure: whether it plays at all ([`Arrangement::reached`]).
    pub reached: Vec<bool>,
    /// Whether it follows a D.C./D.S. jump.
    pub follows_jumps: bool,
    /// Whether it reaches a jump it can't follow (a D.S. without a segno
    /// before it), whether or not it follows another
    /// ([`Arrangement::unfollowable`]).
    pub skips_jumps: bool,
    /// Whether it was cut short at the resolver's safety guard
    /// ([`Arrangement::truncated`]).
    pub truncated: bool,
}

/// [`RepeatsAnalysis::sequence`], given how the timeline's endings play.
fn play_sequence(
    timeline: &[MeasureDef],
    plan: &EndingPlan<'_>,
    max_bars: Option<usize>,
) -> PlaySequence {
    let Arrangement {
        order,
        passes,
        jumped,
        unfollowable,
        truncated,
        played,
        reached,
    } = Arrangement::resolve_with_plan(timeline, plan, max_bars.unwrap_or(usize::MAX));
    let bars = order
        .into_iter()
        .zip(passes)
        .map(|(index, pass)| PlayedBar {
            bar_number: index.0.saturating_add(1),
            pass: pass.into(),
        })
        .collect();
    PlaySequence {
        bars,
        played,
        reached,
        follows_jumps: jumped,
        skips_jumps: unfollowable,
        truncated,
    }
}

/// A run of GP simile marks (`%`) on a track's bars. Serialized with
/// 1-based bar ranges: `track`, `bars`, `source_bars` (left out for a run
/// without enough bars before it to repeat) and `kind`.
#[derive(Debug, Clone)]
pub struct SimileRun {
    pub track_name: String,
    /// 0-based index of first bar in the run (within track.measures)
    pub run_start: usize,
    /// Length of the run in bars
    pub run_len: usize,
    /// 0-based index of the source bars being referenced: the block of
    /// `source_len` bars right before the run, which each of its marks
    /// repeats in turn
    pub source_start: usize,
    /// Length of the source block in bars: 1 for `%`, 2 for a double
    pub source_len: usize,
    /// "1-bar (Simple)", "2-bar (Double)", or "unknown (…)"
    pub kind: String,
}

impl SimileRun {
    /// Its bars, 1-based: first and last.
    pub fn bars(&self) -> (usize, usize) {
        (self.run_start + 1, self.run_start + self.run_len)
    }

    /// The bars it repeats, 1-based: first and last; `None` for a run
    /// without them before it.
    pub fn source_bars(&self) -> Option<(usize, usize)> {
        (self.source_len > 0).then(|| (self.source_start + 1, self.source_start + self.source_len))
    }
}

impl Serialize for SimileRun {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Run<'a> {
            track: &'a str,
            bars: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            source_bars: Option<String>,
            kind: &'a str,
        }
        let range = |(first, last): (usize, usize)| format!("{first}-{last}");
        Run {
            track: &self.track_name,
            bars: range(self.bars()),
            source_bars: self.source_bars().map(range),
            kind: &self.kind,
        }
        .serialize(serializer)
    }
}

/// The runs of simile marks on the tracks whose name contains
/// `track_filter` (any case), or on all of them.
pub fn simile_runs(song: &Song, track_filter: Option<&str>) -> Vec<SimileRun> {
    song.tracks
        .iter()
        .filter(|track| {
            track_filter.is_none_or(|f| track.name.to_lowercase().contains(&f.to_lowercase()))
        })
        .flat_map(track_simile_runs)
        .collect()
}

/// The bars a simile mark repeats; `None` for an unknown mark.
fn simile_step(mark: &str) -> Option<usize> {
    match mark {
        "Simple" => Some(1),
        "FirstOfDouble" | "SecondOfDouble" => Some(2),
        _ => None,
    }
}

/// The runs of simile marks on a track, in order.
pub fn track_simile_runs(track: &LegacyTrack) -> Vec<SimileRun> {
    let mut runs = Vec::new();
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

        let run_start = i;
        i += 1;
        let (step, kind_label) = match simile_step(sm) {
            Some(step) => {
                // Extend the run while marks of its kind continue.
                while measures
                    .get(i)
                    .and_then(|m| m.simile_mark.as_deref())
                    .is_some_and(|mark| simile_step(mark) == Some(step))
                {
                    i += 1;
                }
                let label = if step == 1 {
                    "1-bar (Simple)"
                } else {
                    "2-bar (Double)"
                };
                (step, label.to_owned())
            }
            // Unknown — treat as 1-bar run
            None => (1, format!("unknown ({sm})")),
        };

        let run_len = i - run_start;
        // Each mark repeats the `step` bars right before the run; with
        // fewer bars before it, it has none to repeat.
        let (source_start, source_len) = match run_start.checked_sub(step) {
            Some(source_start) => (source_start, step),
            None => (run_start, 0),
        };

        runs.push(SimileRun {
            track_name: track.name.clone(),
            run_start,
            run_len,
            source_start,
            source_len,
            kind: kind_label,
        });
    }

    runs
}

// ---------------------------------------------------------------------------
// Internal fingerprinting
// ---------------------------------------------------------------------------

/// Musical identity of a measure (voices, beats, note pitches/frets, rhythms).
/// Display and GP-roundtrip metadata are intentionally excluded.
#[derive(PartialEq, Eq)]
struct MeasureFingerprint(Vec<VoiceFingerprint>);

#[derive(PartialEq, Eq)]
struct VoiceFingerprint {
    voice_id: u8,
    beats: Vec<BeatFingerprint>,
}

#[derive(PartialEq, Eq)]
struct BeatFingerprint {
    dur_base: u8,
    dur_dots: u8,
    tuplet: Option<(u8, u8)>,
    is_rest: bool,
    notes: Vec<NoteFingerprint>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct NoteFingerprint {
    key: u32,
    /// Note continues from the previous measure (silent attack, pitch already ringing).
    is_tie_end: bool,
    /// Note rings into the next measure (tie chain continues past the barline).
    is_tie_start: bool,
}

// --- helpers ---

fn note_value_to_u8(v: NoteValue) -> u8 {
    match v {
        NoteValue::Whole => 0,
        NoteValue::Half => 1,
        NoteValue::Quarter => 2,
        NoteValue::Eighth => 3,
        NoteValue::Sixteenth => 4,
        NoteValue::ThirtySecond => 5,
        NoteValue::SixtyFourth => 6,
        NoteValue::HundredTwentyEighth => 7,
        NoteValue::Other(n) => 8u8.saturating_add(n.min(247) as u8),
    }
}

fn pitch_step_to_u8(s: PitchStep) -> u8 {
    match s {
        PitchStep::C => 0,
        PitchStep::D => 1,
        PitchStep::E => 2,
        PitchStep::F => 3,
        PitchStep::G => 4,
        PitchStep::A => 5,
        PitchStep::B => 6,
    }
}

/// Encode a note's musical identity as a `u32` key.
///
/// Tab notes use string+fret; pitch notes use step+alter+octave; ghost/rest notes → 0.
fn note_key(note: &Note) -> u32 {
    if let (Some(s), Some(f)) = (note.string, note.fret) {
        // Tab: bit 24 set | string in 16..23 | fret in 0..15
        (1u32 << 24) | ((s as u32) << 16) | (f as u32)
    } else if let Some(p) = &note.pitch {
        // Pitch: step in 16..23 | (alter+2) in 8..15 | octave in 0..7
        ((pitch_step_to_u8(p.step) as u32) << 16)
            | (((p.alter + 2) as u8 as u32) << 8)
            | (p.octave as u32)
    } else {
        0
    }
}

fn fingerprint_note(note: &Note) -> NoteFingerprint {
    NoteFingerprint {
        key: note_key(note),
        is_tie_end: note.tie.is_some_and(TieType::ends_tie),
        is_tie_start: note.tie.is_some_and(TieType::starts_tie),
    }
}

fn fingerprint_beat(beat: &Beat) -> BeatFingerprint {
    let mut notes: Vec<NoteFingerprint> = beat.notes.iter().map(fingerprint_note).collect();
    notes.sort();
    BeatFingerprint {
        dur_base: note_value_to_u8(beat.duration.base),
        dur_dots: beat.duration.dots,
        tuplet: beat.duration.tuplet.map(|t| (t.actual, t.normal)),
        is_rest: beat.is_rest(),
        notes,
    }
}

fn fingerprint_voice(voice: &Voice) -> VoiceFingerprint {
    VoiceFingerprint {
        voice_id: voice.voice_id,
        beats: voice.beats.iter().map(fingerprint_beat).collect(),
    }
}

/// Returns `None` if the measure already has a `MeasureRepeat` (unsuitable as
/// template or repeat target).
fn fingerprint_measure(md: &MeasureData) -> Option<MeasureFingerprint> {
    if md.repeat.is_some() {
        return None;
    }
    let mut voices: Vec<VoiceFingerprint> = md.voices.values().map(fingerprint_voice).collect();
    voices.sort_by_key(|v| v.voice_id);
    Some(MeasureFingerprint(voices))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::model::optimized::{
        beat::{Beat, Duration, Voice},
        global::{InstrumentId, MeasureIndex, TrackId},
        note::{Note, NoteValue, Pitch, PitchStep},
        timeline::MeasureDef,
        track::{MeasureData, Track},
    };

    // -------------------------------------------------------------------------
    // Minimal constructors
    // -------------------------------------------------------------------------

    fn pitch(step: PitchStep) -> Pitch {
        Pitch {
            step,
            alter: 0,
            octave: 4,
        }
    }

    fn bare_note(p: Pitch) -> Note {
        Note {
            pitch: Some(p),
            string: None,
            fret: None,
            tie: None,
            techniques: vec![],
            ornaments: vec![],
            articulations: vec![],
            left_finger: None,
            right_finger: None,
            notehead: None,
            stem: None,
            accidental: None,
            arpeggiate: None,
            display_pitch: None,
            gp_harmonic: None,
            gp_grace: None,
            gp_bend: None,
            gp_trill: None,
            gp_ghost: false,
            gp_duration_percent: 1.0,
            gp_swap_accidentals: false,
            gp_velocity: None,
            gp_note_type_raw: None,
            gp_is_rest: false,
            gp_ornament: None,
            gp_note_duration: None,
            gp_note_tuplet: None,
        }
    }

    fn quarter_beat(p: Pitch) -> Beat {
        Beat {
            tick_offset: 0,
            duration: Duration {
                base: NoteValue::Quarter,
                dots: 0,
                tuplet: None,
            },
            notes: vec![bare_note(p)],
            events: vec![],
            dynamic: None,
            slur: None,
            lyric: None,
            beam_group: None,
            tuplet: None,
            beams: vec![],
            grace_notes: vec![],
            cue: false,
            chord: None,
            fermata: false,
            gp_empty: false,
            gp_rest: false,
            gp_vibrato: false,
            gp_fade_in: false,
            gp_stroke: None,
            gp_pick_stroke: None,
            gp_beat_flags2: None,
            gp_break_secondary: None,
            gp_slap_effect: None,
            gp_rasgueado: false,
            gp_text: String::new(),
            gp_mix_table: None,
            gp_tremolo_bar: None,
            gp_chord: None,
        }
    }

    /// Build a one-voice MeasureData with the given beats on voice 0.
    fn measure_with_beats(index: u16, beats: Vec<Beat>) -> MeasureData {
        let mut voices = HashMap::new();
        voices.insert(0u8, Voice { voice_id: 0, beats });
        MeasureData {
            measure_index: MeasureIndex(index),
            track_id: TrackId(0),
            repeat: None,
            voices,
            barline_right: None,
            gp_line_break: 0,
            gp_simile_mark: None,
            clef: None,
        }
    }

    fn empty_track_with_measures(measures: Vec<MeasureData>) -> Track {
        let mut btree = BTreeMap::new();
        for md in measures {
            btree.insert(md.measure_index, md);
        }
        Track {
            id: TrackId(0),
            name: String::new(),
            instrument: InstrumentId(0),
            staves: vec![],
            measures: btree,
        }
    }

    fn empty_timeline(n: u16) -> Vec<MeasureDef> {
        (0..n)
            .map(|i| MeasureDef {
                index: MeasureIndex(i),
                tempo: None,
                time_signature: None,
                key_signature: None,
                navigation: vec![],
                marker: None,
                tick_resolution: 960,
                duration_ticks: 3840,
                barline_left: None,
                barline_right: None,
                gp_beams: None,
                gp_fermatas: vec![],
                gp_free_time: false,
            })
            .collect()
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    /// No measures → empty result.
    #[test]
    fn empty_track() {
        let track = empty_track_with_measures(vec![]);
        let result = detect_repeats(&track, &[]);
        assert!(result.is_empty());
    }

    /// Single measure → nothing to repeat.
    #[test]
    fn single_measure_no_repeat() {
        let track = empty_track_with_measures(vec![measure_with_beats(
            0,
            vec![quarter_beat(pitch(PitchStep::C))],
        )]);
        let tl = empty_timeline(1);
        let result = detect_repeats(&track, &tl);
        assert!(result.is_empty());
    }

    /// A A → second measure gets Single.
    #[test]
    fn single_repeat_aa() {
        let a = || vec![quarter_beat(pitch(PitchStep::C))];
        let track =
            empty_track_with_measures(vec![measure_with_beats(0, a()), measure_with_beats(1, a())]);
        let tl = empty_timeline(2);
        let result = detect_repeats(&track, &tl);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&MeasureIndex(1)], SimileMark::Single);
    }

    /// A B → no repeat.
    #[test]
    fn no_repeat_ab() {
        let track = empty_track_with_measures(vec![
            measure_with_beats(0, vec![quarter_beat(pitch(PitchStep::C))]),
            measure_with_beats(1, vec![quarter_beat(pitch(PitchStep::D))]),
        ]);
        let tl = empty_timeline(2);
        let result = detect_repeats(&track, &tl);
        assert!(result.is_empty());
    }

    /// A A A A → A % % % (Single chains).
    #[test]
    fn single_repeat_chain() {
        let a = || vec![quarter_beat(pitch(PitchStep::E))];
        let track = empty_track_with_measures(vec![
            measure_with_beats(0, a()),
            measure_with_beats(1, a()),
            measure_with_beats(2, a()),
            measure_with_beats(3, a()),
        ]);
        let tl = empty_timeline(4);
        let result = detect_repeats(&track, &tl);
        // Measures 1, 2, 3 all repeat
        assert_eq!(result[&MeasureIndex(1)], SimileMark::Single);
        assert_eq!(result[&MeasureIndex(2)], SimileMark::Single);
        assert_eq!(result[&MeasureIndex(3)], SimileMark::Single);
    }

    /// A B A B → last two measures get Double.
    #[test]
    fn double_repeat_abab() {
        let a = || vec![quarter_beat(pitch(PitchStep::C))];
        let b = || vec![quarter_beat(pitch(PitchStep::G))];
        let track = empty_track_with_measures(vec![
            measure_with_beats(0, a()),
            measure_with_beats(1, b()),
            measure_with_beats(2, a()),
            measure_with_beats(3, b()),
        ]);
        let tl = empty_timeline(4);
        let result = detect_repeats(&track, &tl);
        assert_eq!(result[&MeasureIndex(2)], SimileMark::Double);
        assert_eq!(result[&MeasureIndex(3)], SimileMark::Double);
        // First two measures untouched
        assert!(!result.contains_key(&MeasureIndex(0)));
        assert!(!result.contains_key(&MeasureIndex(1)));
    }

    /// A B C D A B C D → last four measures get Fourth.
    #[test]
    fn fourth_repeat() {
        let steps = [PitchStep::C, PitchStep::D, PitchStep::E, PitchStep::F];
        let measures: Vec<MeasureData> = steps
            .iter()
            .chain(steps.iter())
            .enumerate()
            .map(|(i, &s)| measure_with_beats(i as u16, vec![quarter_beat(pitch(s))]))
            .collect();
        let track = empty_track_with_measures(measures);
        let tl = empty_timeline(8);
        let result = detect_repeats(&track, &tl);
        for i in 4..8u16 {
            assert_eq!(
                result[&MeasureIndex(i)],
                SimileMark::Fourth,
                "measure {i} should be Fourth"
            );
        }
        for i in 0..4u16 {
            assert!(
                !result.contains_key(&MeasureIndex(i)),
                "template measure {i} should not be annotated"
            );
        }
    }

    /// A time-signature change blocks a simile mark on that measure.
    #[test]
    fn time_sig_change_blocks() {
        let a = || vec![quarter_beat(pitch(PitchStep::C))];
        let track = empty_track_with_measures(vec![
            measure_with_beats(0, a()),
            measure_with_beats(1, a()), // would normally get Single
        ]);
        // Inject a time-signature change at measure 1.
        let mut tl = empty_timeline(2);
        tl[1].time_signature = Some(crate::model::optimized::metadata::TimeSignature {
            numerator: 3,
            denominator: 4,
        });
        let result = detect_repeats(&track, &tl);
        assert!(
            result.is_empty(),
            "time-sig change should block Single mark"
        );
    }

    /// Existing MeasureRepeat in input → measure is skipped (not usable as template or target).
    #[test]
    fn existing_repeat_skipped() {
        use crate::model::optimized::track::{MeasureRepeat, MeasureRepeatKind};

        let a = || vec![quarter_beat(pitch(PitchStep::C))];
        let mut pre_repeated = measure_with_beats(1, vec![]);
        pre_repeated.repeat = Some(MeasureRepeat {
            kind: MeasureRepeatKind::Single,
        });

        let track = empty_track_with_measures(vec![
            measure_with_beats(0, a()),
            pre_repeated,
            measure_with_beats(2, a()),
        ]);
        let tl = empty_timeline(3);
        let result = detect_repeats(&track, &tl);
        // Measure 2 matches measure 0 in content, but measure 1 blocks as template.
        // Measure 1 is None fingerprint → not suitable template for Double.
        // Single check: template for measure 2 is measure 1 (None fingerprint) → blocked.
        assert!(
            !result.contains_key(&MeasureIndex(2)),
            "should not detect repeat through pre-existing repeat"
        );
    }

    fn quarter_beat_tie_start(p: Pitch) -> Beat {
        let mut b = quarter_beat(p);
        b.notes[0].tie = Some(TieType::Start);
        b
    }

    /// A measure with a tie-start note must NOT match a measure where the same
    /// note stops cleanly (no tie). The ringing-into-next-measure relationship
    /// is part of the musical identity.
    #[test]
    fn tie_start_prevents_false_match() {
        let track = empty_track_with_measures(vec![
            measure_with_beats(0, vec![quarter_beat(pitch(PitchStep::C))]),
            measure_with_beats(1, vec![quarter_beat_tie_start(pitch(PitchStep::C))]),
        ]);
        let tl = empty_timeline(2);
        let result = detect_repeats(&track, &tl);
        assert!(
            result.is_empty(),
            "tie-start note must prevent a Single match with a clean note"
        );
    }

    /// Two identical measures where both have a tie-start note CAN match.
    #[test]
    fn tie_start_matches_same_tie_start() {
        let track = empty_track_with_measures(vec![
            measure_with_beats(0, vec![quarter_beat_tie_start(pitch(PitchStep::C))]),
            measure_with_beats(1, vec![quarter_beat_tie_start(pitch(PitchStep::C))]),
        ]);
        let tl = empty_timeline(2);
        let result = detect_repeats(&track, &tl);
        assert_eq!(
            result.get(&MeasureIndex(1)),
            Some(&SimileMark::Single),
            "two identical tie-start measures should match"
        );
    }

    /// apply_repeats clears voices and sets MeasureRepeat.
    #[test]
    fn apply_repeats_clears_voices() {
        let a = || vec![quarter_beat(pitch(PitchStep::A))];
        let mut track =
            empty_track_with_measures(vec![measure_with_beats(0, a()), measure_with_beats(1, a())]);
        let tl = empty_timeline(2);
        let detected = detect_repeats(&track, &tl);
        apply_repeats(&mut track, &detected);

        let md1 = &track.measures[&MeasureIndex(1)];
        assert!(
            md1.voices.is_empty(),
            "voices should be cleared after apply"
        );
        assert!(md1.repeat.is_some(), "repeat should be set after apply");
        assert_eq!(md1.repeat.as_ref().unwrap().kind, MeasureRepeatKind::Single);
    }
}
