//! Structural form detection for the optimized model.
//!
//! Identifies large-scale repetition in a track and assigns formal labels
//! — A, B, A′, C, … — to sections, producing a compact representation of
//! the song's overall form (e.g. `A B A B A′ C`).
//!
//! ## Algorithm
//!
//! **Segmentation** (`find_section_boundaries`):
//! 1. Mark boundaries at silence transitions (all-rest measures) and at the
//!    edges of GP simile-mark runs.
//! 2. Recursively split any section longer than `MAX_SECTION_LEN` at its
//!    midpoint (snapped to the nearest multiple of 4).
//!
//! **Similarity** (`section_similarity`):
//! - Per-measure Jaccard on pitch-class sets (weight 0.55).
//! - Per-measure normalised edit distance on sorted beat-duration sequences
//!   (weight 0.35).
//! - Length ratio `min / max` as a soft penalty (weight 0.10).
//! - Hard filter: sections differing in length by more than 2× score 0.
//!
//! **Labelling** (greedy, in order of first appearance):
//! - First occurrence of a section → new base letter (`A`, `B`, `C`, …),
//!   `similarity = 1.0`.
//! - Later section with `sim ≥ 0.99` → exact repeat, same label.
//! - Later section with `threshold ≤ sim < 0.99` → variation, label gains
//!   one prime per new variation (`A′`, `A″`, …).
//! - Otherwise → new base letter.

use crate::model::optimized::{
    note::{NoteValue, PitchStep},
    track::{MeasureData, Track},
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A labelled section of the track.
#[derive(Debug, Clone, PartialEq)]
pub struct SectionLabel {
    /// 0-based position of the first measure of this section in the track's
    /// sorted measure order.
    pub start_measure: usize,
    /// 0-based position of the first measure of the **next** section
    /// (exclusive end).
    pub end_measure: usize,
    /// Formal label: `"A"`, `"B"`, `"A'"`, `"A''"`, `"C"`, …
    pub label: String,
    /// Similarity vs. the canonical first occurrence of this label.
    /// `1.0` for the canonical itself or an exact repeat; < 1.0 for
    /// variations.
    pub similarity: f32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Detect the musical form of `track` and return a labelled section list.
///
/// `strings` maps 1-based string numbers to open-string MIDI pitches (same
/// convention as the other analysis functions). Used to convert tab notes
/// to pitch classes; pitch-only notes are converted directly.
///
/// `similarity_threshold` controls how similar two sections must be to share
/// a base label. A value of `0.85` is a good default. Use `1.0` to require
/// exact pitch-class + rhythm equality.
pub fn detect_form(
    track: &Track,
    strings: &[(i8, i8)],
    similarity_threshold: f32,
) -> Vec<SectionLabel> {
    let measure_indices: Vec<_> = track.measures.keys().copied().collect();
    let n = measure_indices.len();
    if n == 0 {
        return Vec::new();
    }

    // Per-measure flags and pitch-class sets
    let is_silent: Vec<bool> = measure_indices
        .iter()
        .map(|idx| track.measures.get(idx).is_none_or(measure_is_silent))
        .collect();
    let has_simile: Vec<bool> = measure_indices
        .iter()
        .map(|idx| {
            track
                .measures
                .get(idx)
                .is_some_and(|md| md.gp_simile_mark.is_some())
        })
        .collect();
    let pc_sets: Vec<[bool; 12]> = measure_indices
        .iter()
        .map(|idx| {
            track
                .measures
                .get(idx)
                .map_or([false; 12], |md| measure_pc_set(md, strings))
        })
        .collect();

    // Segmentation
    let boundaries = find_section_boundaries(n, &is_silent, &has_simile, &pc_sets);

    // Build (start, end) index pairs
    let section_ranges: Vec<(usize, usize)> = {
        let mut ranges = Vec::new();
        for i in 0..boundaries.len() {
            let start = boundaries[i];
            let end = boundaries.get(i + 1).copied().unwrap_or(n);
            if end > start {
                ranges.push((start, end));
            }
        }
        ranges
    };

    if section_ranges.is_empty() {
        return Vec::new();
    }

    // Build representations
    let reprs: Vec<SectionRepr> = section_ranges
        .iter()
        .map(|(start, end)| {
            let mds: Vec<&MeasureData> = measure_indices[*start..*end]
                .iter()
                .filter_map(|idx| track.measures.get(idx))
                .collect();
            build_repr(&mds, strings)
        })
        .collect();

    // Greedy labelling
    struct Canonical {
        label_char: char,
        repr: SectionRepr,
        variation_count: usize,
    }

    let mut canonicals: Vec<Canonical> = Vec::new();
    let mut next_char = b'A';
    let mut result: Vec<SectionLabel> = Vec::with_capacity(section_ranges.len());

    for (i, &(start, end)) in section_ranges.iter().enumerate() {
        let repr = &reprs[i];

        // Find best-matching canonical
        let (best_sim, best_ci) = canonicals
            .iter()
            .enumerate()
            .map(|(ci, c)| (section_similarity(repr, &c.repr), ci))
            .fold(
                (0.0f32, 0),
                |(bs, bi), (s, ci)| {
                    if s > bs { (s, ci) } else { (bs, bi) }
                },
            );

        let (label, sim) = if !canonicals.is_empty() && best_sim >= 0.99 {
            // Exact repeat
            (canonicals[best_ci].label_char.to_string(), best_sim)
        } else if !canonicals.is_empty() && best_sim >= similarity_threshold {
            // Variation
            let c = &mut canonicals[best_ci];
            let primes = c.variation_count + 1;
            c.variation_count += 1;
            (label_with_primes(c.label_char, primes), best_sim)
        } else {
            // New canonical
            let ch = next_char as char;
            if next_char < b'Z' {
                next_char += 1;
            }
            canonicals.push(Canonical {
                label_char: ch,
                repr: repr.clone(),
                variation_count: 0,
            });
            (ch.to_string(), 1.0)
        };

        result.push(SectionLabel {
            start_measure: start,
            end_measure: end,
            label,
            similarity: sim,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Segmentation
// ---------------------------------------------------------------------------

/// Maximum number of measures in one section before it is split.
const MAX_SECTION_LEN: usize = 16;

/// Minimum Jaccard similarity between adjacent non-silent measures before a
/// content-change boundary is inserted.  Measures that differ more than this
/// are assumed to belong to different formal sections.
const CONTENT_CHANGE_THRESHOLD: f32 = 0.6;

/// Return the sorted list of section-start positions (0-based measure indices).
fn find_section_boundaries(
    n: usize,
    is_silent: &[bool],
    has_simile: &[bool],
    pc_sets: &[[bool; 12]],
) -> Vec<usize> {
    let mut is_boundary = vec![false; n];
    is_boundary[0] = true;

    for i in 1..n {
        // Boundary at every silence transition and simile-mark edge.
        if is_silent[i] != is_silent[i - 1] || has_simile[i] != has_simile[i - 1] {
            is_boundary[i] = true;
        }
        // Content-change boundary: adjacent non-silent measures with low PC similarity.
        if !is_silent[i] && !is_silent[i - 1] {
            let j = jaccard(&pc_sets[i - 1], &pc_sets[i]);
            if j < CONTENT_CHANGE_THRESHOLD {
                is_boundary[i] = true;
            }
        }
    }

    let mut boundaries: Vec<usize> = (0..n).filter(|&i| is_boundary[i]).collect();

    // Recursively split sections that exceed MAX_SECTION_LEN.
    split_long_sections(&mut boundaries, n);

    boundaries
}

fn split_long_sections(boundaries: &mut Vec<usize>, n: usize) {
    let mut i = 0;
    while i < boundaries.len() {
        let start = boundaries[i];
        let end = boundaries.get(i + 1).copied().unwrap_or(n);
        let len = end - start;
        if len > MAX_SECTION_LEN {
            // Midpoint snapped to nearest multiple of 4.
            let raw_mid = start + len.div_ceil(2);
            let snapped = ((raw_mid + 2) / 4) * 4;
            let mid = if snapped > start && snapped < end {
                snapped
            } else {
                raw_mid
            };
            boundaries.insert(i + 1, mid);
            // Re-examine the first half (don't advance i).
        } else {
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Section representation
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct SectionRepr {
    /// Pitch-class presence array, one entry per measure.
    pc_sets: Vec<[bool; 12]>,
    /// Sorted beat-duration values (as `u8`), one Vec per measure.
    rhythms: Vec<Vec<u8>>,
}

fn build_repr(measures: &[&MeasureData], strings: &[(i8, i8)]) -> SectionRepr {
    let pc_sets = measures
        .iter()
        .map(|md| measure_pc_set(md, strings))
        .collect();
    let rhythms = measures.iter().map(|md| measure_rhythm(md)).collect();
    SectionRepr { pc_sets, rhythms }
}

// ---------------------------------------------------------------------------
// Per-measure features
// ---------------------------------------------------------------------------

/// `true` when every beat in every voice is a rest or filler.
fn measure_is_silent(md: &MeasureData) -> bool {
    md.voices
        .values()
        .all(|v| v.beats.iter().all(|b| b.gp_rest || b.gp_empty))
}

/// Pitch-class presence set for one measure.
fn measure_pc_set(md: &MeasureData, strings: &[(i8, i8)]) -> [bool; 12] {
    let mut pc = [false; 12];
    for voice in md.voices.values() {
        for beat in &voice.beats {
            if beat.gp_rest || beat.gp_empty {
                continue;
            }
            for note in &beat.notes {
                if note.gp_is_rest {
                    continue;
                }
                let pc_val: Option<u8> = if let (Some(s), Some(f)) = (note.string, note.fret) {
                    strings
                        .iter()
                        .find(|&&(sn, _)| sn == s as i8)
                        .map(|&(_, open)| ((open as i32 + f as i32).rem_euclid(12)) as u8)
                } else {
                    note.pitch.as_ref().map(|p| {
                        let semi = pitch_step_semitone(p.step) + p.alter as i32;
                        semi.rem_euclid(12) as u8
                    })
                };
                if let Some(v) = pc_val {
                    pc[v as usize] = true;
                }
            }
        }
    }
    pc
}

/// Sorted beat-duration encoding for one measure (all voices combined).
fn measure_rhythm(md: &MeasureData) -> Vec<u8> {
    let mut durs: Vec<u8> = md
        .voices
        .values()
        .flat_map(|v| v.beats.iter())
        .filter(|b| !b.gp_rest && !b.gp_empty)
        .map(|b| note_value_to_u8(b.duration.base))
        .collect();
    durs.sort();
    durs
}

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
        NoteValue::Other(x) => 8u8.saturating_add(x.min(247) as u8),
    }
}

fn pitch_step_semitone(step: PitchStep) -> i32 {
    match step {
        PitchStep::C => 0,
        PitchStep::D => 2,
        PitchStep::E => 4,
        PitchStep::F => 5,
        PitchStep::G => 7,
        PitchStep::A => 9,
        PitchStep::B => 11,
    }
}

// ---------------------------------------------------------------------------
// Similarity
// ---------------------------------------------------------------------------

/// Compute the overall similarity score between two section representations.
///
/// Returns a value in `[0.0, 1.0]` where `1.0` = identical.
fn section_similarity(a: &SectionRepr, b: &SectionRepr) -> f32 {
    let len_a = a.pc_sets.len();
    let len_b = b.pc_sets.len();
    if len_a == 0 || len_b == 0 {
        return 0.0;
    }

    let min_len = len_a.min(len_b);
    let max_len = len_a.max(len_b);

    // Hard filter: reject sections that differ in length by more than 2×.
    if min_len * 2 < max_len {
        return 0.0;
    }

    let length_ratio = min_len as f32 / max_len as f32;

    // Compare only the overlapping prefix.
    let cmp = min_len;

    let pc_score = (0..cmp)
        .map(|k| jaccard(&a.pc_sets[k], &b.pc_sets[k]))
        .sum::<f32>()
        / cmp as f32;

    let rhythm_score = (0..cmp)
        .map(|k| rhythm_sim(&a.rhythms[k], &b.rhythms[k]))
        .sum::<f32>()
        / cmp as f32;

    0.55 * pc_score + 0.35 * rhythm_score + 0.10 * length_ratio
}

/// Jaccard similarity of two pitch-class presence arrays.
///
/// Returns `1.0` when both sets are empty (two silent measures are identical).
fn jaccard(a: &[bool; 12], b: &[bool; 12]) -> f32 {
    let mut inter = 0u32;
    let mut union = 0u32;
    for i in 0..12 {
        if a[i] || b[i] {
            union += 1;
        }
        if a[i] && b[i] {
            inter += 1;
        }
    }
    if union == 0 {
        1.0
    } else {
        inter as f32 / union as f32
    }
}

/// Normalised edit-distance similarity of two sorted rhythm sequences.
fn rhythm_sim(a: &[u8], b: &[u8]) -> f32 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 1.0;
    }
    let dist = edit_distance(a, b);
    1.0 - (dist as f32 / max_len as f32)
}

/// Levenshtein edit distance using a rolling 1-D DP row.
fn edit_distance(a: &[u8], b: &[u8]) -> usize {
    let n = b.len();
    let mut dp = (0..=n).collect::<Vec<_>>();
    for (i, &ai) in a.iter().enumerate() {
        let mut prev = dp[0];
        dp[0] = i + 1;
        for j in 1..=n {
            let old = dp[j];
            dp[j] = if ai == b[j - 1] {
                prev
            } else {
                1 + prev.min(dp[j]).min(dp[j - 1])
            };
            prev = old;
        }
    }
    dp[n]
}

// ---------------------------------------------------------------------------
// Label helpers
// ---------------------------------------------------------------------------

/// Build a label string: base letter followed by `primes` apostrophes.
/// 0 primes → `"A"`, 1 prime → `"A'"`, 2 primes → `"A''"`, etc.
fn label_with_primes(base: char, primes: usize) -> String {
    let mut s = base.to_string();
    s.extend(std::iter::repeat_n('\'', primes));
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};

    use crate::model::optimized::{
        beat::{Beat, Duration, Voice},
        global::{InstrumentId, MeasureIndex, TrackId},
        note::{Note, NoteValue, Pitch, PitchStep},
        track::{MeasureData, Track},
    };

    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    fn note_from_pc(pc: u8) -> Note {
        let (step, alter) = pc_to_pitch(pc);
        Note {
            pitch: Some(Pitch {
                step,
                alter,
                octave: 4,
            }),
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

    /// Quarter note beat at tick 0 containing notes for the given pitch classes.
    fn chord_beat(pitch_classes: &[u8]) -> Beat {
        Beat {
            tick_offset: 0,
            duration: Duration {
                base: NoteValue::Quarter,
                dots: 0,
                tuplet: None,
            },
            notes: pitch_classes.iter().map(|&pc| note_from_pc(pc)).collect(),
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

    /// Rest beat at tick 0.
    fn rest_beat() -> Beat {
        let mut b = chord_beat(&[]);
        b.gp_rest = true;
        b
    }

    fn measure(index: u16, beats: Vec<Beat>) -> MeasureData {
        let mut voices = HashMap::new();
        voices.insert(0u8, Voice { voice_id: 0, beats });
        MeasureData {
            measure_index: MeasureIndex(index),
            track_id: TrackId(0),
            repeat: None,
            voices,
            gp_line_break: 0,
            gp_simile_mark: None,
        }
    }

    fn track_from_measures(mds: Vec<MeasureData>) -> Track {
        let mut btree = BTreeMap::new();
        for md in mds {
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

    /// Convert a pitch class (0–11) to `(PitchStep, alter)`.
    fn pc_to_pitch(pc: u8) -> (PitchStep, i8) {
        match pc % 12 {
            0 => (PitchStep::C, 0),
            1 => (PitchStep::C, 1),
            2 => (PitchStep::D, 0),
            3 => (PitchStep::D, 1),
            4 => (PitchStep::E, 0),
            5 => (PitchStep::F, 0),
            6 => (PitchStep::F, 1),
            7 => (PitchStep::G, 0),
            8 => (PitchStep::G, 1),
            9 => (PitchStep::A, 0),
            10 => (PitchStep::A, 1),
            11 => (PitchStep::B, 0),
            _ => unreachable!(),
        }
    }

    /// Build a track of `count` identical measures each containing one chord beat.
    fn uniform_track(count: u16, pitch_classes: &[u8]) -> Track {
        let mds = (0..count)
            .map(|i| measure(i, vec![chord_beat(pitch_classes)]))
            .collect();
        track_from_measures(mds)
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn labels(track: &Track) -> Vec<String> {
        detect_form(track, &[], 0.85)
            .into_iter()
            .map(|s| s.label)
            .collect()
    }

    // -------------------------------------------------------------------------
    // Tests: edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn empty_track_returns_empty() {
        let track = track_from_measures(vec![]);
        assert!(detect_form(&track, &[], 0.85).is_empty());
    }

    #[test]
    fn single_measure_is_one_section_a() {
        let track = uniform_track(1, &[0, 4, 7]);
        let result = detect_form(&track, &[], 0.85);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "A");
        assert_eq!(result[0].similarity, 1.0);
    }

    // -------------------------------------------------------------------------
    // Tests: section boundaries
    // -------------------------------------------------------------------------

    #[test]
    fn short_uniform_track_is_one_section() {
        // 8 identical measures — all within MAX_SECTION_LEN=16, no silence,
        // so a single section "A".
        let track = uniform_track(8, &[0, 4, 7]);
        let result = detect_form(&track, &[], 0.85);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "A");
        assert_eq!(result[0].start_measure, 0);
        assert_eq!(result[0].end_measure, 8);
    }

    #[test]
    fn silence_creates_two_sections() {
        // 4 content measures, 2 silent, 4 content measures
        // → boundaries at 0, 4, 6 → sections [0,4), [4,6), [6,10)
        let mut mds: Vec<MeasureData> = (0..4)
            .map(|i| measure(i, vec![chord_beat(&[0, 4, 7])]))
            .collect();
        mds.push(measure(4, vec![rest_beat()]));
        mds.push(measure(5, vec![rest_beat()]));
        for i in 6..10u16 {
            mds.push(measure(i, vec![chord_beat(&[0, 4, 7])]));
        }
        let track = track_from_measures(mds);
        let result = detect_form(&track, &[], 0.85);
        // At least 3 sections (content, silence, content)
        assert!(
            result.len() >= 3,
            "expected ≥3 sections, got {}",
            result.len()
        );
        // First and last sections should have the same label (both are C-major content)
        assert_eq!(result[0].label, result[result.len() - 1].label);
    }

    #[test]
    fn long_track_is_split_at_max_section_len() {
        // 32 identical measures — must be split into sections of ≤ 16.
        let track = uniform_track(32, &[0, 4, 7]);
        let result = detect_form(&track, &[], 0.85);
        assert!(
            result
                .iter()
                .all(|s| s.end_measure - s.start_measure <= MAX_SECTION_LEN),
            "all sections must be ≤ {MAX_SECTION_LEN} measures"
        );
    }

    // -------------------------------------------------------------------------
    // Tests: labelling
    // -------------------------------------------------------------------------

    #[test]
    fn two_different_sections_get_a_and_b() {
        // 8 measures C-major, then 8 measures F#-major (tritone away — very low Jaccard).
        let mut mds: Vec<MeasureData> = (0..8)
            .map(|i| measure(i, vec![chord_beat(&[0, 4, 7])]))
            .collect();
        for i in 8..16u16 {
            mds.push(measure(i, vec![chord_beat(&[6, 10, 1])])); // F# major
        }
        let track = track_from_measures(mds);
        let ls = labels(&track);
        assert!(ls.contains(&"A".to_string()), "should have A");
        assert!(ls.contains(&"B".to_string()), "should have B");
    }

    #[test]
    fn exact_repeat_keeps_same_label() {
        // 4 measures A content, 4 measures A content (exact copy).
        let mut mds: Vec<MeasureData> = (0..4)
            .map(|i| measure(i, vec![chord_beat(&[0, 4, 7])]))
            .collect();
        for i in 4..8u16 {
            mds.push(measure(i, vec![chord_beat(&[0, 4, 7])]));
        }
        let track = track_from_measures(mds);
        let result = detect_form(&track, &[], 0.85);
        // All sections should be labelled "A" (exact repeat, no prime).
        assert!(
            result.iter().all(|s| s.label == "A"),
            "all identical sections should be 'A', got {:?}",
            result.iter().map(|s| &s.label).collect::<Vec<_>>()
        );
        // The repeated occurrence has similarity ≥ 0.99.
        let repeats: Vec<_> = result.iter().skip(1).collect();
        assert!(repeats.iter().all(|s| s.similarity >= 0.99));
    }

    #[test]
    fn variation_gets_prime_label() {
        // Section A: 8 measures of C-major.
        // Section A': 8 measures where 3 are C-minor instead — similar but not identical.
        let a_pcs: &[u8] = &[0, 4, 7]; // C major
        let a_var: &[u8] = &[0, 3, 7]; // C minor (change: 4→3)

        // Build a 16-measure track: first 8 = A, last 8 = mostly A but 3 measures differ.
        let mut mds: Vec<MeasureData> = (0..8)
            .map(|i| measure(i, vec![chord_beat(a_pcs)]))
            .collect();
        for i in 8..16u16 {
            let pcs = if i - 8 < 3 { a_var } else { a_pcs };
            mds.push(measure(i, vec![chord_beat(pcs)]));
        }
        let track = track_from_measures(mds);
        let result = detect_form(&track, &[], 0.85);

        // The first section is "A"; the second should contain a prime (variation).
        // (Exact form depends on segmentation, but we expect "A" and something like "A'")
        let first_label = &result[0].label;
        assert_eq!(first_label, "A");
        let has_variation = result
            .iter()
            .skip(1)
            .any(|s| s.label.starts_with('A') && s.label.contains('\''));
        assert!(
            has_variation,
            "expected a variation label (A'), got: {:?}",
            result.iter().map(|s| &s.label).collect::<Vec<_>>()
        );
    }

    #[test]
    fn abab_pattern_is_detected() {
        // 4-measure sections in order A B A B.
        // A = C-major, B = A-minor (share some pitch classes but are distinct).
        let a_pcs: &[u8] = &[0, 4, 7]; // C E G
        // let b_pcs: &[u8] = &[9, 0, 4]; // A C E  (shares 0,4 with A — but different root)

        // Make B distinct enough from A: use entirely different pitch classes.
        let b_pcs2: &[u8] = &[6, 10, 1]; // F# A# C# — tritone away from C, low Jaccard

        let mut mds = Vec::new();
        for chunk in 0..4u16 {
            let (start, pcs) = if chunk % 2 == 0 {
                (chunk * 4, a_pcs)
            } else {
                (chunk * 4, b_pcs2)
            };
            for j in 0..4u16 {
                mds.push(measure(start + j, vec![chord_beat(pcs)]));
            }
        }
        let track = track_from_measures(mds);
        let result = detect_form(&track, &[], 0.85);

        // Collect unique base letters (ignoring primes).
        let bases: Vec<char> = result
            .iter()
            .map(|s| s.label.chars().next().unwrap())
            .collect();

        // Should alternate between two base letters.
        assert_eq!(bases.len(), 4, "expected 4 sections");
        assert_eq!(
            bases[0], bases[2],
            "1st and 3rd sections should share a label"
        );
        assert_eq!(
            bases[1], bases[3],
            "2nd and 4th sections should share a label"
        );
        assert_ne!(bases[0], bases[1], "A and B should be different");
    }

    // -------------------------------------------------------------------------
    // Tests: low-level helpers
    // -------------------------------------------------------------------------

    #[test]
    fn jaccard_identical_sets() {
        let a = [
            true, false, true, false, true, false, false, true, false, false, false, false,
        ];
        assert!((jaccard(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn jaccard_disjoint_sets() {
        let a = [
            true, false, false, false, false, false, false, false, false, false, false, false,
        ];
        let b = [
            false, true, false, false, false, false, false, false, false, false, false, false,
        ];
        assert!((jaccard(&a, &b)).abs() < 1e-6);
    }

    #[test]
    fn jaccard_empty_sets_returns_one() {
        let empty = [false; 12];
        assert!((jaccard(&empty, &empty) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn edit_distance_equal_sequences() {
        let a = vec![2u8, 3, 4];
        assert_eq!(edit_distance(&a, &a), 0);
    }

    #[test]
    fn edit_distance_single_substitution() {
        let a = vec![2u8, 3, 4];
        let b = vec![2u8, 9, 4];
        assert_eq!(edit_distance(&a, &b), 1);
    }

    #[test]
    fn edit_distance_insertion() {
        let a = vec![2u8, 4];
        let b = vec![2u8, 3, 4];
        assert_eq!(edit_distance(&a, &b), 1);
    }

    #[test]
    fn label_with_primes_zero() {
        assert_eq!(label_with_primes('A', 0), "A");
    }

    #[test]
    fn label_with_primes_two() {
        assert_eq!(label_with_primes('B', 2), "B''");
    }
}
