//! Key and scale detection for the optimized model.
//!
//! Two related but distinct operations are provided:
//!
//! **Scale/key guessing** (`guess_key`): statistical inference from pitch-class content.
//! Given a slice of beats, returns a ranked list of `(root, scale)` candidates scored by
//! coverage (fraction of expected notes found) and purity (fraction of found notes that
//! are in the scale).
//!
//! **Key signature detection** (`detect_key_changes`): normative derivation. Maps the
//! best-scoring `(root, scale)` per measure to the canonical circle-of-fifths key
//! signature that minimises accidentals, then smooths transients and emits a timeline
//! of `KeyChange` events.
//!
//! ## Relationship to `chords.rs`
//! Reuses `PitchClass` from the sibling module.

use crate::analysis::chords::PitchClass;
use crate::model::legacy::key_signature::KeySignature;
use crate::model::optimized::{beat::Beat, note::PitchStep, track::Track};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Musical scale / mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scale {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    PentatonicMajor,
    PentatonicMinor,
    /// Minor pentatonic + ♭5 (the "blue note").
    Blues,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    /// Symmetric 6-note scale (no canonical key signature).
    WholeTone,
    /// Symmetric 8-note scale (no canonical key signature).
    Diminished,
}

/// A key + scale candidate scored against the pitch-class content of a segment.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyGuess {
    /// Tonal centre (0 = C, 1 = C♯/D♭, …, 11 = B).
    pub root: PitchClass,
    pub scale: Scale,
    /// Fraction of total note weight that falls within the scale (`0.0`–`1.0`).
    pub coverage: f32,
    /// `1 − (extra pitch classes / total pitch classes present)` — how "clean" the match is.
    pub purity: f32,
}

/// A key-signature change event on the timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyChange {
    /// 0-based position of the first affected measure in the track's sorted order.
    pub at_measure: usize,
    /// The canonical key signature (sharps/flats + major-or-relative-minor label).
    pub key: KeySignature,
}

// ---------------------------------------------------------------------------
// Scale interval tables
// ---------------------------------------------------------------------------

const ALL_SCALES: [Scale; 14] = [
    Scale::Major,
    Scale::NaturalMinor,
    Scale::HarmonicMinor,
    Scale::MelodicMinor,
    Scale::PentatonicMajor,
    Scale::PentatonicMinor,
    Scale::Blues,
    Scale::Dorian,
    Scale::Phrygian,
    Scale::Lydian,
    Scale::Mixolydian,
    Scale::Locrian,
    Scale::WholeTone,
    Scale::Diminished,
];

/// Sorted semitone intervals from the root (mod 12) for each scale.
fn scale_intervals(scale: Scale) -> &'static [u8] {
    match scale {
        Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
        Scale::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
        Scale::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
        Scale::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
        Scale::PentatonicMajor => &[0, 2, 4, 7, 9],
        Scale::PentatonicMinor => &[0, 3, 5, 7, 10],
        Scale::Blues => &[0, 3, 5, 6, 7, 10],
        Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
        Scale::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
        Scale::Lydian => &[0, 2, 4, 6, 7, 9, 11],
        Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
        Scale::Locrian => &[0, 1, 3, 5, 6, 8, 10],
        Scale::WholeTone => &[0, 2, 4, 6, 8, 10],
        Scale::Diminished => &[0, 2, 3, 5, 6, 8, 9, 11],
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Infer the most likely key and scale from a slice of beats.
///
/// `strings` maps 1-based string numbers to open-string MIDI pitches (same
/// convention as the other analysis functions).
///
/// Returns candidates ordered from **most to least likely**. Returns an empty
/// `Vec` when no notes are found.
pub fn guess_key(beats: &[&Beat], strings: &[(i8, i8)]) -> Vec<KeyGuess> {
    let hist = build_histogram(beats, strings);
    if hist.iter().all(|&w| w == 0.0) {
        return Vec::new();
    }

    let mut candidates: Vec<(f32, KeyGuess)> = Vec::new();

    for root in 0u8..12 {
        for &scale in &ALL_SCALES {
            let (coverage, purity) = score_scale(&hist, root, scale);
            if coverage == 0.0 {
                continue;
            }

            // Specificity: fraction of scale notes that appear in the input.
            // Penalises large scales when only a small subset of their notes is present,
            // allowing a 5-note pentatonic to beat a 7-note major on a pentatonic input.
            let specificity = scale_specificity(&hist, root, scale);

            // Weak prior: prefer keys with fewer sharps/flats to break ties.
            let prior = to_key_signature(root, scale)
                .map(|k| 1.0 - k.key.unsigned_abs() as f32 * 0.005)
                .unwrap_or(0.99);

            let score = 0.50 * coverage + 0.35 * purity + 0.14 * specificity + 0.001 * prior;
            candidates.push((
                score,
                KeyGuess {
                    root: PitchClass(root),
                    scale,
                    coverage,
                    purity,
                },
            ));
        }
    }

    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    candidates.into_iter().map(|(_, g)| g).collect()
}

/// Derive a timeline of key-signature changes from a track.
///
/// Per-measure key guesses are smoothed (single-measure transients are
/// suppressed) before boundaries are emitted. The first entry in the returned
/// `Vec` is always at `at_measure = 0` when at least one measure exists and
/// has notes.
pub fn detect_key_changes(track: &Track, strings: &[(i8, i8)]) -> Vec<KeyChange> {
    let measure_indices: Vec<_> = track.measures.keys().copied().collect();
    let n = measure_indices.len();
    if n == 0 {
        return Vec::new();
    }

    // Best key signature per measure (None = silent or undetermined).
    let per_measure: Vec<Option<KeySignature>> = measure_indices
        .iter()
        .map(|idx| {
            let md = track.measures.get(idx)?;
            let beats: Vec<&Beat> = md.voices.values().flat_map(|v| v.beats.iter()).collect();
            guess_key(&beats, strings)
                .into_iter()
                .find_map(|g| to_key_signature(g.root.0, g.scale))
        })
        .collect();

    // Smooth transients: if a measure's key differs from both neighbours,
    // replace it with the neighbour's key.
    let smoothed = smooth_key_sequence(&per_measure);

    // Emit a KeyChange whenever the key actually changes.
    let mut result: Vec<KeyChange> = Vec::new();
    let mut current: Option<&KeySignature> = None;

    for (i, ks_opt) in smoothed.iter().enumerate() {
        if let Some(ks) = ks_opt
            && current != Some(ks)
        {
            result.push(KeyChange {
                at_measure: i,
                key: ks.clone(),
            });
            current = Some(ks);
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Internal: key-signature mapping
// ---------------------------------------------------------------------------

/// Map `(root_pc, scale)` to the canonical `KeySignature`.
///
/// For modal scales the key signature is that of the **relative major** (the
/// parent diatonic set). Returns `None` for `WholeTone` and `Diminished`,
/// which have no canonical key signature.
pub(crate) fn to_key_signature(root: u8, scale: Scale) -> Option<KeySignature> {
    let (rel_pc, is_minor) = match scale {
        // Major and major-like modes: key sig IS the root's major
        Scale::Major | Scale::PentatonicMajor => (root, false),
        // Modes: map to the parent major scale
        Scale::Lydian => ((root + 7) % 12, false), // F Lydian → C major
        Scale::Mixolydian => ((root + 5) % 12, false), // G Mixolydian → C major
        Scale::Dorian => ((root + 10) % 12, false), // D Dorian → C major
        Scale::Phrygian => ((root + 8) % 12, false), // E Phrygian → C major
        Scale::Locrian => ((root + 1) % 12, false), // B Locrian → C major
        // Minor-family: relative major is a minor 3rd up
        Scale::NaturalMinor | Scale::HarmonicMinor | Scale::MelodicMinor => ((root + 3) % 12, true),
        Scale::PentatonicMinor | Scale::Blues => ((root + 3) % 12, true),
        // No canonical key signature
        Scale::WholeTone | Scale::Diminished => return None,
    };
    Some(KeySignature {
        key: pc_to_major_fifths(rel_pc),
        is_minor,
    })
}

/// Convert a major-key root pitch class to its circle-of-fifths offset
/// (`key` field in `KeySignature`).  Enharmonic ambiguities are resolved by
/// preferring the side that minimises `|key|`.
fn pc_to_major_fifths(pc: u8) -> i8 {
    match pc % 12 {
        0 => 0,   // C
        7 => 1,   // G
        2 => 2,   // D
        9 => 3,   // A
        4 => 4,   // E
        11 => 5,  // B
        6 => 6,   // F♯ (prefer sharp over G♭ = −6; both have |key|=6, pick +)
        1 => -5,  // D♭  (C♯=7 has more sharps; D♭=−5 is preferred)
        8 => -4,  // A♭
        3 => -3,  // E♭
        10 => -2, // B♭
        5 => -1,  // F
        _ => unreachable!(),
    }
}

// ---------------------------------------------------------------------------
// Internal: scoring
// ---------------------------------------------------------------------------

/// Pitch-class histogram: `hist[i]` = number of notes with pitch class `i`.
fn build_histogram(beats: &[&Beat], strings: &[(i8, i8)]) -> [f32; 12] {
    let mut hist = [0.0f32; 12];
    for beat in beats {
        if beat.gp_rest || beat.gp_empty {
            continue;
        }
        for note in &beat.notes {
            if note.gp_is_rest {
                continue;
            }
            if let Some(pc) = note_to_pc(note, strings) {
                hist[pc as usize] += 1.0;
            }
        }
    }
    hist
}

/// Convert a note to its pitch class (0–11), preferring tab over pitch.
fn note_to_pc(note: &crate::model::optimized::note::Note, strings: &[(i8, i8)]) -> Option<u8> {
    if let (Some(s), Some(f)) = (note.string, note.fret)
        && let Some(&(_, open)) = strings.iter().find(|&&(sn, _)| sn as u8 == s)
    {
        return Some(((open as i32 + f as i32).rem_euclid(12)) as u8);
    }
    note.pitch
        .as_ref()
        .map(|p| (pitch_step_semitone(p.step) + p.alter as i32).rem_euclid(12) as u8)
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

/// Score a `(root, scale)` pair against a pitch-class histogram.
///
/// Returns `(coverage, purity)`:
/// - `coverage` = fraction of total note weight that lands in the scale.
/// - `purity`   = fraction of distinct present pitch classes that are in the scale.
fn score_scale(hist: &[f32; 12], root: u8, scale: Scale) -> (f32, f32) {
    let total_weight: f32 = hist.iter().sum();
    if total_weight == 0.0 {
        return (0.0, 0.0);
    }

    // Build the expected pitch-class set.
    let mut expected = [false; 12];
    for &interval in scale_intervals(scale) {
        expected[((root as u32 + interval as u32) % 12) as usize] = true;
    }

    // Coverage: weighted fraction of notes in the scale.
    let covered_weight: f32 = hist
        .iter()
        .enumerate()
        .filter(|&(i, _)| expected[i])
        .map(|(_, &w)| w)
        .sum();
    let coverage = covered_weight / total_weight;

    // Purity: fraction of distinct PCs that are in the scale.
    let present: Vec<usize> = hist
        .iter()
        .enumerate()
        .filter(|&(_, &w)| w > 0.0)
        .map(|(i, _)| i)
        .collect();
    let purity = if present.is_empty() {
        1.0
    } else {
        let in_scale = present.iter().filter(|&&i| expected[i]).count();
        in_scale as f32 / present.len() as f32
    };

    (coverage, purity)
}

/// Fraction of the scale's own notes that are present in the input.
///
/// A large scale (7 notes) scores lower than a small one (5 notes) when the
/// input contains only a pentatonic subset, rewarding tighter fits.
fn scale_specificity(hist: &[f32; 12], root: u8, scale: Scale) -> f32 {
    let intervals = scale_intervals(scale);
    let in_scale_present = intervals
        .iter()
        .filter(|&&iv| hist[((root as u32 + iv as u32) % 12) as usize] > 0.0)
        .count() as f32;
    in_scale_present / intervals.len() as f32
}

// ---------------------------------------------------------------------------
// Internal: smoothing
// ---------------------------------------------------------------------------

/// Suppress single-measure transients: if measure `i` has a different key
/// from both its predecessor and successor (and both exist), replace it with
/// the predecessor's key.
fn smooth_key_sequence(seq: &[Option<KeySignature>]) -> Vec<Option<KeySignature>> {
    let n = seq.len();
    let mut result = seq.to_vec();
    for i in 1..n.saturating_sub(1) {
        let prev = seq[i - 1].as_ref();
        let curr = seq[i].as_ref();
        let next = seq[i + 1].as_ref();
        if let (Some(p), Some(c), Some(nx)) = (prev, curr, next)
            && p != c
            && nx == p
        {
            result[i] = Some(p.clone());
        }
    }
    result
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
    // Test helpers
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

    fn beat_from_pcs(pcs: &[u8]) -> Beat {
        Beat {
            tick_offset: 0,
            duration: Duration {
                base: NoteValue::Quarter,
                dots: 0,
                tuplet: None,
            },
            notes: pcs.iter().map(|&pc| note_from_pc(pc)).collect(),
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

    fn rest_beat() -> Beat {
        let mut b = beat_from_pcs(&[]);
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

    /// Build a uniform track of `count` measures each containing one beat with the
    /// given pitch classes.
    fn uniform_track(count: u16, pcs: &[u8]) -> Track {
        let mds = (0..count)
            .map(|i| measure(i, vec![beat_from_pcs(pcs)]))
            .collect();
        track_from_measures(mds)
    }

    // -------------------------------------------------------------------------
    // Tests: guess_key
    // -------------------------------------------------------------------------

    #[test]
    fn guess_key_empty_beats_returns_empty() {
        assert!(guess_key(&[], &[]).is_empty());
    }

    #[test]
    fn guess_key_rest_beat_returns_empty() {
        let b = rest_beat();
        assert!(guess_key(&[&b], &[]).is_empty());
    }

    #[test]
    fn guess_key_c_major_scale_ranks_c_major_first() {
        // All 7 notes of C major: C D E F G A B
        let b = beat_from_pcs(&[0, 2, 4, 5, 7, 9, 11]);
        let result = guess_key(&[&b], &[]);
        assert!(!result.is_empty());
        let top = &result[0];
        assert_eq!(top.root.0, 0, "root should be C (0)");
        assert_eq!(top.scale, Scale::Major, "scale should be Major");
        assert!((top.coverage - 1.0).abs() < 1e-4);
        assert!((top.purity - 1.0).abs() < 1e-4);
    }

    #[test]
    fn guess_key_a_natural_minor_in_candidates() {
        // A natural minor: A B C D E F G (PCs: 9 11 0 2 4 5 7).
        // NOTE: these notes are the exact same set as C major, D Dorian, etc.
        // The algorithm cannot determine the tonal centre from pitch classes alone;
        // we only assert that A natural minor appears somewhere in the ranked list.
        let b = beat_from_pcs(&[9, 11, 0, 2, 4, 5, 7]);
        let result = guess_key(&[&b], &[]);
        assert!(!result.is_empty());
        let a_minor = result
            .iter()
            .find(|g| g.root.0 == 9 && g.scale == Scale::NaturalMinor);
        assert!(a_minor.is_some(), "A natural minor should be in candidates");
        assert!((a_minor.unwrap().coverage - 1.0).abs() < 1e-4);
    }

    #[test]
    fn guess_key_pentatonic_major_wins_over_full_major() {
        // Only the 5 notes of C pentatonic major: C D E G A
        let b = beat_from_pcs(&[0, 2, 4, 7, 9]);
        let result = guess_key(&[&b], &[]);
        assert!(!result.is_empty());
        // The top result should have coverage=1.0 and purity=1.0 for C pentatonic major.
        // (C major would have coverage=5/7 ≈ 0.71 for the same notes.)
        let top = &result[0];
        assert_eq!(top.root.0, 0);
        assert_eq!(top.scale, Scale::PentatonicMajor);
        assert!((top.coverage - 1.0).abs() < 1e-4);
        assert!((top.purity - 1.0).abs() < 1e-4);
    }

    #[test]
    fn guess_key_g_major_in_candidates() {
        // G major: G A B C D E F# (PCs: 7 9 11 0 2 4 6).
        // NOTE: this set is shared with E natural minor, C Lydian, D Mixolydian, etc.
        // Assert that G major is in the candidates (with perfect scores), not necessarily first.
        let b = beat_from_pcs(&[7, 9, 11, 0, 2, 4, 6]);
        let result = guess_key(&[&b], &[]);
        assert!(!result.is_empty());
        let g_major = result
            .iter()
            .find(|g| g.root.0 == 7 && g.scale == Scale::Major);
        assert!(g_major.is_some(), "G major should be in candidates");
        assert!((g_major.unwrap().coverage - 1.0).abs() < 1e-4);
        assert!((g_major.unwrap().purity - 1.0).abs() < 1e-4);
    }

    #[test]
    fn guess_key_dorian_recognized() {
        // D Dorian: D E F G A B C (PCs: 2 4 5 7 9 11 0) — same as C major
        // but starting on D (the 2nd degree).
        let b = beat_from_pcs(&[2, 4, 5, 7, 9, 11, 0]);
        let result = guess_key(&[&b], &[]);
        assert!(!result.is_empty());
        // D Dorian and C Major share the same notes; both should appear near the top.
        // At minimum, D Dorian should be in the result.
        let dorian_d = result
            .iter()
            .find(|g| g.root.0 == 2 && g.scale == Scale::Dorian);
        assert!(dorian_d.is_some(), "D Dorian should be in candidates");
    }

    // -------------------------------------------------------------------------
    // Tests: score_scale
    // -------------------------------------------------------------------------

    #[test]
    fn score_scale_perfect_coverage_and_purity() {
        // Histogram with exactly C major notes.
        let mut hist = [0.0f32; 12];
        for &pc in &[0u8, 2, 4, 5, 7, 9, 11] {
            hist[pc as usize] = 1.0;
        }
        let (cov, pur) = score_scale(&hist, 0, Scale::Major);
        assert!((cov - 1.0).abs() < 1e-4, "coverage {cov}");
        assert!((pur - 1.0).abs() < 1e-4, "purity {pur}");
    }

    #[test]
    fn score_scale_partial_coverage() {
        // Only 5 of 7 C major notes present (C D E G A — pentatonic subset).
        let mut hist = [0.0f32; 12];
        for &pc in &[0u8, 2, 4, 7, 9] {
            hist[pc as usize] = 1.0;
        }
        let (cov, pur) = score_scale(&hist, 0, Scale::Major);
        // 5 notes in scale / 5 total → coverage = 1.0; purity = 1.0 (no extras)
        assert!((cov - 1.0).abs() < 1e-4, "coverage {cov}");
        assert!((pur - 1.0).abs() < 1e-4, "purity {pur}");
    }

    #[test]
    fn score_scale_impure_match() {
        // C major notes + one extra (C# is outside C major).
        let mut hist = [0.0f32; 12];
        for &pc in &[0u8, 1, 2, 4, 5, 7, 9, 11] {
            hist[pc as usize] = 1.0;
        }
        let (cov, pur) = score_scale(&hist, 0, Scale::Major);
        // 7 of 8 notes are in scale → coverage = 7/8 = 0.875
        assert!((cov - 7.0 / 8.0).abs() < 1e-4, "coverage {cov}");
        // 7 of 8 distinct PCs are in scale → purity = 7/8
        assert!((pur - 7.0 / 8.0).abs() < 1e-4, "purity {pur}");
    }

    // -------------------------------------------------------------------------
    // Tests: to_key_signature
    // -------------------------------------------------------------------------

    #[test]
    fn to_key_sig_c_major() {
        let ks = to_key_signature(0, Scale::Major).unwrap();
        assert_eq!(ks.key, 0);
        assert!(!ks.is_minor);
    }

    #[test]
    fn to_key_sig_a_natural_minor() {
        let ks = to_key_signature(9, Scale::NaturalMinor).unwrap();
        assert_eq!(ks.key, 0);
        assert!(ks.is_minor);
    }

    #[test]
    fn to_key_sig_g_major() {
        let ks = to_key_signature(7, Scale::Major).unwrap();
        assert_eq!(ks.key, 1);
        assert!(!ks.is_minor);
    }

    #[test]
    fn to_key_sig_f_major() {
        let ks = to_key_signature(5, Scale::Major).unwrap();
        assert_eq!(ks.key, -1);
        assert!(!ks.is_minor);
    }

    #[test]
    fn to_key_sig_d_dorian_same_as_c_major() {
        // D Dorian belongs to the C major key signature.
        let ks = to_key_signature(2, Scale::Dorian).unwrap();
        assert_eq!(ks.key, 0);
        assert!(!ks.is_minor);
    }

    #[test]
    fn to_key_sig_whole_tone_is_none() {
        assert!(to_key_signature(0, Scale::WholeTone).is_none());
    }

    #[test]
    fn to_key_sig_diminished_is_none() {
        assert!(to_key_signature(0, Scale::Diminished).is_none());
    }

    // -------------------------------------------------------------------------
    // Tests: detect_key_changes
    // -------------------------------------------------------------------------

    #[test]
    fn detect_key_changes_empty_track() {
        let track = track_from_measures(vec![]);
        assert!(detect_key_changes(&track, &[]).is_empty());
    }

    #[test]
    fn detect_key_changes_uniform_c_major() {
        // 8 measures of C major — should produce exactly one KeyChange at measure 0.
        let track = uniform_track(8, &[0, 2, 4, 5, 7, 9, 11]);
        let changes = detect_key_changes(&track, &[]);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].at_measure, 0);
        assert_eq!(changes[0].key.key, 0);
        assert!(!changes[0].key.is_minor);
    }

    #[test]
    fn detect_key_changes_modulation_c_to_g() {
        // First 4 measures: C major (0 sharps), last 4 measures: G major (1 sharp).
        let c_major = [0u8, 2, 4, 5, 7, 9, 11];
        let g_major = [7u8, 9, 11, 0, 2, 4, 6]; // G A B C D E F#
        let mut mds: Vec<MeasureData> = (0..4)
            .map(|i| measure(i, vec![beat_from_pcs(&c_major)]))
            .collect();
        for i in 4..8u16 {
            mds.push(measure(i, vec![beat_from_pcs(&g_major)]));
        }
        let track = track_from_measures(mds);
        let changes = detect_key_changes(&track, &[]);
        assert!(
            changes.len() >= 2,
            "expected at least 2 key changes, got {}",
            changes.len()
        );
        assert_eq!(changes[0].key.key, 0); // C major
        let later = changes.iter().find(|c| c.key.key == 1);
        assert!(later.is_some(), "expected a G major section (key=1)");
    }

    #[test]
    fn detect_key_changes_single_measure_transient_suppressed() {
        // 4 measures C major, 1 measure G major, 4 measures C major.
        // The single G-major measure should be smoothed away.
        let c_major = [0u8, 2, 4, 5, 7, 9, 11];
        let g_major = [7u8, 9, 11, 0, 2, 4, 6];
        let mut mds: Vec<MeasureData> = (0..4)
            .map(|i| measure(i, vec![beat_from_pcs(&c_major)]))
            .collect();
        mds.push(measure(4, vec![beat_from_pcs(&g_major)]));
        for i in 5..9u16 {
            mds.push(measure(i, vec![beat_from_pcs(&c_major)]));
        }
        let track = track_from_measures(mds);
        let changes = detect_key_changes(&track, &[]);
        // After smoothing the single G-major measure should revert to C major.
        // We should see only one key section (C major throughout).
        let g_sections: Vec<_> = changes.iter().filter(|c| c.key.key == 1).collect();
        assert!(
            g_sections.is_empty(),
            "single-measure G major transient should be smoothed away, got: {g_sections:?}"
        );
    }

    #[test]
    fn detect_key_changes_minor_key() {
        // A harmonic minor: A B C D E F G# (PCs: 9 11 0 2 4 5 8).
        // G# (pc=8) is absent from C major, making this set unambiguously minor-family.
        // The relative major of A minor is C (key=0, is_minor=true).
        let track = uniform_track(4, &[9, 11, 0, 2, 4, 5, 8]);
        let changes = detect_key_changes(&track, &[]);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].key.key, 0);
        assert!(changes[0].key.is_minor);
    }
}
