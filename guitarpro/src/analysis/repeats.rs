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

use std::collections::BTreeMap;

use crate::model::optimized::{
    beat::{Beat, Voice},
    global::MeasureIndex,
    note::{Note, NoteValue, PitchStep, TieType},
    timeline::MeasureDef,
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
        is_tie_end: matches!(note.tie, Some(TieType::End)),
        is_tie_start: matches!(note.tie, Some(TieType::Start)),
    }
}

fn fingerprint_beat(beat: &Beat) -> BeatFingerprint {
    let mut notes: Vec<NoteFingerprint> = beat.notes.iter().map(fingerprint_note).collect();
    notes.sort();
    BeatFingerprint {
        dur_base: note_value_to_u8(beat.duration.base),
        dur_dots: beat.duration.dots,
        tuplet: beat.duration.tuplet.map(|t| (t.actual, t.normal)),
        is_rest: beat.gp_rest || beat.notes.is_empty(),
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
            gp_line_break: 0,
            gp_simile_mark: None,
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
