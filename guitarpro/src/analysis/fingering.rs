//! Left-hand finger assignment for guitar tablature.
//!
//! Suggests which finger (index = 1, middle = 2, ring = 3, pinky = 4) to use
//! for each fretted note in a sequence of measures.
//!
//! ## Algorithm
//!
//! **Per beat** (chord event = all notes sounding at the same tick):
//! 1. Filter open strings (fret = 0) — they need no left-hand finger.
//! 2. Detect a barre: if ≥ 2 notes share the **minimum** fret of the chord,
//!    the index finger lays flat across those strings.
//! 3. Determine the hand **position** = fret covered by the index finger
//!    (= minimum fret of the chord, which is also the barre fret when present).
//! 4. **Position stability**: if the previous position still accommodates every
//!    fretted note in the current chord (all frets in `[prev, prev+3]`), stay
//!    there rather than shifting.
//! 5. **Assign fingers** positionally: `finger = fret − position + 1` (clamped
//!    to 1–4).  When two non-barre notes land on the same fret (e.g. two
//!    strings fingered at fret 3), each gets a successive finger number so both
//!    remain physically reachable.
//! 6. The first note of a beat that begins a new position carries
//!    `position_shift = true`.
//!
//! **Look-ahead** (applied after the forward pass):
//! For each measure boundary, if the last beat of measure M can be played at
//! the opening position of measure M+1 without violating the span constraint,
//! retarget it.  Because positions are stored and reused for the
//! `position_shift` calculation, this also automatically clears the spurious
//! shift that would otherwise appear on the first beat of M+1.

use std::collections::{BTreeMap, HashMap};

use crate::model::optimized::track::MeasureData;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Structural role of a finger in a chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerRole {
    /// Normal single-string placement.
    Single,
    /// Index finger barring the lowest fret across multiple strings (first string listed).
    BarreAnchor,
    /// Additional string covered by the same barre.
    BarreMember,
}

/// A suggested finger placement for one fretted note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FingerAssignment {
    /// 1-based string number (matches `Note::string`).
    pub string: i8,
    /// Fret number (always ≥ 1; open strings are not included).
    pub fret: i16,
    /// Suggested finger: 1 = index, 2 = middle, 3 = ring, 4 = pinky.
    pub finger: u8,
    pub role: FingerRole,
    /// `true` when this note is the first note after a hand-position shift.
    pub position_shift: bool,
}

/// Suggest left-hand fingering for a window of measures.
///
/// `measures` may contain any number of measures; the algorithm processes them
/// all left-to-right and applies look-ahead at each measure boundary.
/// Pass the current measure **plus one look-ahead** measure to let the last
/// beat of each measure anticipate the next phrase.
///
/// `strings` maps 1-based string numbers to open-string MIDI pitches
/// (same convention as [`crate::analysis::chords::identify_chord`]).
/// Notes that already carry `string` + `fret` are used directly; `strings`
/// is only needed when converting pitch-only notes to tab — currently those
/// are skipped.
///
/// Returns one `Vec<FingerAssignment>` per input measure (empty Vecs for
/// measures that have no fretted notes).
pub fn suggest_fingering(
    measures: &[&MeasureData],
    _strings: &[(i8, i8)],
) -> Vec<Vec<FingerAssignment>> {
    if measures.is_empty() {
        return Vec::new();
    }

    // --- Collect chord events -------------------------------------------------
    // A ChordEvent groups all simultaneously-played fretted notes across voices.
    struct ChordEvent {
        measure_idx: usize,
        notes: Vec<TabNote>, // sorted by (fret, string), fret ≥ 1
    }

    let mut events: Vec<ChordEvent> = Vec::new();
    for (m_idx, measure) in measures.iter().enumerate() {
        for chord in collect_chord_events(measure) {
            if !chord.is_empty() {
                events.push(ChordEvent {
                    measure_idx: m_idx,
                    notes: chord,
                });
            }
        }
    }

    if events.is_empty() {
        return vec![Vec::new(); measures.len()];
    }

    // --- Forward pass: compute hand positions ---------------------------------
    // Separating position computation from assignment lets the look-ahead pass
    // edit positions in-place; the `position_shift` flags then come out
    // automatically correct when we build assignments.
    let mut positions: Vec<i16> = Vec::with_capacity(events.len());
    let mut prev_pos: Option<i16> = None;

    for event in &events {
        let nat = natural_position(&event.notes);
        let pos = match prev_pos {
            // Stay in the current position if the whole chord fits.
            Some(p) if can_play_at(&event.notes, p) => p,
            _ => nat,
        };
        positions.push(pos);
        prev_pos = Some(pos);
    }

    // --- Look-ahead pass -----------------------------------------------------
    // For each measure boundary m → m+1, try to retarget the last beat of m
    // to the opening position of m+1.  Updating `positions` in-place means
    // the shift recalculation below becomes automatically correct.
    for m in 0..measures.len().saturating_sub(1) {
        let last_idx = events.iter().rposition(|e| e.measure_idx == m);
        let first_next_idx = events.iter().position(|e| e.measure_idx == m + 1);

        if let (Some(li), Some(fi)) = (last_idx, first_next_idx) {
            let next_pos = positions[fi];
            if positions[li] != next_pos && can_play_at(&events[li].notes, next_pos) {
                positions[li] = next_pos;
            }
        }
    }

    // --- Build assignments ---------------------------------------------------
    let mut measure_results: Vec<Vec<FingerAssignment>> = vec![Vec::new(); measures.len()];

    for (i, event) in events.iter().enumerate() {
        let pos = positions[i];
        let shifted = i > 0 && positions[i - 1] != pos;
        let assignments = build_assignments(&event.notes, pos, shifted);
        measure_results[event.measure_idx].extend(assignments);
    }

    measure_results
}

// ---------------------------------------------------------------------------
// Internal types and helpers
// ---------------------------------------------------------------------------

/// A fretted note reduced to the two coordinates needed for fingering.
#[derive(Clone)]
struct TabNote {
    string: u8, // 1-based
    fret: u8,   // ≥ 1 (open strings filtered out before construction)
}

/// Collect all chord events (groups of simultaneously-played fretted notes)
/// from one measure, ordered by tick offset.
fn collect_chord_events(measure: &MeasureData) -> Vec<Vec<TabNote>> {
    let mut by_tick: BTreeMap<u32, Vec<TabNote>> = BTreeMap::new();

    for voice in measure.voices.values() {
        for beat in &voice.beats {
            if beat.gp_empty || beat.gp_rest {
                continue;
            }
            let slot = by_tick.entry(beat.tick_offset).or_default();
            for note in &beat.notes {
                if note.gp_is_rest {
                    continue;
                }
                if let (Some(s), Some(f)) = (note.string, note.fret)
                    && f > 0
                    && !slot.iter().any(|n: &TabNote| n.string == s && n.fret == f)
                {
                    slot.push(TabNote { string: s, fret: f });
                }
            }
        }
    }

    // Sort each chord by (fret, string) for deterministic output.
    by_tick
        .into_values()
        .filter(|v| !v.is_empty())
        .map(|mut notes| {
            notes.sort_by_key(|n| (n.fret, n.string));
            notes
        })
        .collect()
}

/// Minimum fret of a chord (≥ 1), used as the natural hand position.
fn natural_position(notes: &[TabNote]) -> i16 {
    notes
        .iter()
        .map(|n| n.fret as i16)
        .min()
        .unwrap_or(1)
        .max(1)
}

/// Whether every note in `notes` can be played with the hand at `pos`
/// (all frets within the comfortable 4-fret span `[pos, pos+3]`).
fn can_play_at(notes: &[TabNote], pos: i16) -> bool {
    notes.iter().all(|n| {
        let f = n.fret as i16;
        f >= pos && f <= pos + 3
    })
}

/// Build `FingerAssignment`s for a single chord event.
///
/// `position` is the fret the index finger sits on.
/// `shifted` is `true` when the position changed from the previous beat.
fn build_assignments(notes: &[TabNote], position: i16, shifted: bool) -> Vec<FingerAssignment> {
    // Barre: ≥ 2 notes share the position fret (= minimum fret).
    let at_pos_count = notes.iter().filter(|n| n.fret as i16 == position).count();
    let is_barre = at_pos_count >= 2;

    // Per-fret counters for same-fret non-barre notes (e.g. two strings at fret 3):
    // each successive note at the same fret gets the next finger number.
    let mut fret_extra: HashMap<u8, u8> = HashMap::new();
    let mut barre_anchor_done = false;
    let mut first = true;

    notes
        .iter()
        .map(|note| {
            let fret = note.fret as i16;
            let at_barre_fret = fret == position && is_barre;

            let (finger, role) = if at_barre_fret {
                let role = if barre_anchor_done {
                    FingerRole::BarreMember
                } else {
                    barre_anchor_done = true;
                    FingerRole::BarreAnchor
                };
                (1u8, role)
            } else {
                // Base finger from hand position; offset for multiple notes at same fret.
                let base = ((fret - position + 1) as u8).max(1);
                let extra = fret_extra.entry(note.fret).or_insert(0);
                let finger = (base + *extra).min(4);
                *extra += 1;
                (finger, FingerRole::Single)
            };

            let assignment = FingerAssignment {
                string: note.string as i8,
                fret,
                finger,
                role,
                position_shift: first && shifted,
            };
            first = false;
            assignment
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::model::optimized::{
        beat::{Beat, Duration},
        global::{MeasureIndex, TrackId},
        note::{Note, NoteValue},
        track::{MeasureData, MeasureRepeat},
    };

    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    fn tab_note(string: u8, fret: u8) -> Note {
        Note {
            pitch: None,
            string: Some(string),
            fret: Some(fret),
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

    fn beat_at_tick(tick: u32, notes: Vec<Note>) -> Beat {
        Beat {
            tick_offset: tick,
            duration: Duration {
                base: NoteValue::Quarter,
                dots: 0,
                tuplet: None,
            },
            notes,
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

    fn measure_with_voice(index: u16, beats: Vec<Beat>) -> MeasureData {
        use crate::model::optimized::beat::Voice;
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

    fn run(measures: &[&MeasureData]) -> Vec<Vec<FingerAssignment>> {
        suggest_fingering(measures, &[])
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn find(assignments: &[FingerAssignment], string: i8, fret: i16) -> Option<&FingerAssignment> {
        assignments
            .iter()
            .find(|a| a.string == string && a.fret == fret)
    }

    // -------------------------------------------------------------------------
    // Tests: basic cases
    // -------------------------------------------------------------------------

    #[test]
    fn empty_measures_returns_empty() {
        assert!(run(&[]).is_empty());
    }

    #[test]
    fn measure_with_no_fretted_notes_returns_empty_vec() {
        // Only open strings (fret 0) — no finger assignments.
        let beat = beat_at_tick(0, vec![tab_note(1, 0), tab_note(2, 0)]);
        let m = measure_with_voice(0, vec![beat]);
        let result = run(&[&m]);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_empty());
    }

    #[test]
    fn single_fretted_note_gets_index_finger() {
        // One note at fret 5 → position 5 → finger 1, no shift.
        let beat = beat_at_tick(0, vec![tab_note(3, 5)]);
        let m = measure_with_voice(0, vec![beat]);
        let result = run(&[&m]);
        let a = find(&result[0], 3, 5).unwrap();
        assert_eq!(a.finger, 1);
        assert_eq!(a.role, FingerRole::Single);
        assert!(!a.position_shift);
    }

    #[test]
    fn ascending_chord_gets_fingers_one_two_three() {
        // Notes at frets 5, 6, 7 → position 5 → fingers 1, 2, 3.
        let beat = beat_at_tick(0, vec![tab_note(4, 5), tab_note(3, 6), tab_note(2, 7)]);
        let m = measure_with_voice(0, vec![beat]);
        let result = run(&[&m]);
        assert_eq!(find(&result[0], 4, 5).unwrap().finger, 1);
        assert_eq!(find(&result[0], 3, 6).unwrap().finger, 2);
        assert_eq!(find(&result[0], 2, 7).unwrap().finger, 3);
    }

    // -------------------------------------------------------------------------
    // Tests: barre detection
    // -------------------------------------------------------------------------

    #[test]
    fn barre_chord_at_position() {
        // F-shape barre: fret 1 across strings 6,2,1; fret 2 string 3; fret 3 strings 5,4.
        let beat = beat_at_tick(
            0,
            vec![
                tab_note(6, 1),
                tab_note(5, 3),
                tab_note(4, 3),
                tab_note(3, 2),
                tab_note(2, 1),
                tab_note(1, 1),
            ],
        );
        let m = measure_with_voice(0, vec![beat]);
        let result = run(&[&m]);
        let assignments = &result[0];

        // All three notes at fret 1 should get finger 1.
        let f1_s6 = find(assignments, 6, 1).unwrap();
        let f1_s2 = find(assignments, 2, 1).unwrap();
        let f1_s1 = find(assignments, 1, 1).unwrap();
        assert_eq!(f1_s6.finger, 1);
        assert_eq!(f1_s2.finger, 1);
        assert_eq!(f1_s1.finger, 1);

        // Exactly one BarreAnchor among the three.
        let anchor_count = [f1_s6, f1_s2, f1_s1]
            .iter()
            .filter(|a| a.role == FingerRole::BarreAnchor)
            .count();
        let member_count = [f1_s6, f1_s2, f1_s1]
            .iter()
            .filter(|a| a.role == FingerRole::BarreMember)
            .count();
        assert_eq!(anchor_count, 1);
        assert_eq!(member_count, 2);

        // Fret 2 → finger 2.
        assert_eq!(find(assignments, 3, 2).unwrap().finger, 2);

        // Two notes at fret 3 → fingers 3 and 4.
        let fret3: Vec<_> = assignments.iter().filter(|a| a.fret == 3).collect();
        assert_eq!(fret3.len(), 2);
        let f3_fingers: Vec<u8> = {
            let mut v: Vec<u8> = fret3.iter().map(|a| a.finger).collect();
            v.sort();
            v
        };
        assert_eq!(f3_fingers, vec![3, 4]);
    }

    #[test]
    fn two_notes_at_minimum_fret_is_barre() {
        // Open-A style: strings 4,3,2 all at fret 2 (no notes below).
        let beat = beat_at_tick(0, vec![tab_note(4, 2), tab_note(3, 2), tab_note(2, 2)]);
        let m = measure_with_voice(0, vec![beat]);
        let result = run(&[&m]);
        let anchor_count = result[0]
            .iter()
            .filter(|a| a.role == FingerRole::BarreAnchor)
            .count();
        let member_count = result[0]
            .iter()
            .filter(|a| a.role == FingerRole::BarreMember)
            .count();
        assert_eq!(anchor_count, 1);
        assert_eq!(member_count, 2);
        assert!(result[0].iter().all(|a| a.finger == 1));
    }

    #[test]
    fn two_notes_at_non_minimum_fret_are_not_barre() {
        // One note at fret 1 (min), two at fret 2 — the pair at fret 2 must NOT
        // form a barre (barre only applies at the minimum fret).
        let beat = beat_at_tick(0, vec![tab_note(3, 1), tab_note(2, 2), tab_note(1, 2)]);
        let m = measure_with_voice(0, vec![beat]);
        let result = run(&[&m]);
        // No barre roles at all.
        assert!(result[0].iter().all(|a| a.role == FingerRole::Single));
        // Note at fret 1 → finger 1.
        assert_eq!(find(&result[0], 3, 1).unwrap().finger, 1);
        // Two notes at fret 2 → fingers 2 and 3 (consecutive).
        let fret2: Vec<u8> = {
            let mut v: Vec<u8> = result[0]
                .iter()
                .filter(|a| a.fret == 2)
                .map(|a| a.finger)
                .collect();
            v.sort();
            v
        };
        assert_eq!(fret2, vec![2, 3]);
    }

    // -------------------------------------------------------------------------
    // Tests: position stability
    // -------------------------------------------------------------------------

    #[test]
    fn hand_stays_in_position_when_chord_fits() {
        // Beat 1: frets 5, 7 → position 5.
        // Beat 2: frets 6, 8 → natural position 6, but fits in [5, 8] → stays at 5.
        let b1 = beat_at_tick(0, vec![tab_note(3, 5), tab_note(2, 7)]);
        let b2 = beat_at_tick(960, vec![tab_note(3, 6), tab_note(2, 8)]);
        let m = measure_with_voice(0, vec![b1, b2]);
        let result = run(&[&m]);

        // Beat 2 notes: position still 5 → fingers 2 and 4, no shift.
        let a6 = find(&result[0], 3, 6).unwrap();
        let a8 = find(&result[0], 2, 8).unwrap();
        assert_eq!(a6.finger, 2); // fret 6 = pos 5 + 1 → finger 2
        assert_eq!(a8.finger, 4); // fret 8 = pos 5 + 3 → finger 4
        assert!(!a6.position_shift);
        assert!(!a8.position_shift);
    }

    #[test]
    fn position_shift_when_chord_does_not_fit() {
        // Beat 1: fret 2 → position 2.
        // Beat 2: fret 8 → doesn't fit in [2, 5] → new position 8, shift.
        let b1 = beat_at_tick(0, vec![tab_note(3, 2)]);
        let b2 = beat_at_tick(960, vec![tab_note(3, 8)]);
        let m = measure_with_voice(0, vec![b1, b2]);
        let result = run(&[&m]);

        let a = find(&result[0], 3, 8).unwrap();
        assert_eq!(a.finger, 1); // index at new position
        assert!(a.position_shift, "should mark a position shift");
    }

    // -------------------------------------------------------------------------
    // Tests: look-ahead
    // -------------------------------------------------------------------------

    #[test]
    fn lookahead_retargets_last_beat_toward_next_measure() {
        // Measure 0: two beats at positions 1 and 3.
        // Measure 1: beat at position 5.
        // Last beat of measure 0 (position 3) fits in [5, 8]? 3 < 5 → no.
        // Last beat of measure 0 (fret 3) fits in [5, 8]? fret 3 < 5 → no.
        // Let's use fret 5 for the last beat of measure 0:
        //   Can it play at position 5 (next measure opening)? fret 5 = pos 5 → yes.
        //   Was at position 3 before look-ahead → retargeted to 5.

        let b0a = beat_at_tick(0, vec![tab_note(3, 3)]); // pos 3
        let b0b = beat_at_tick(960, vec![tab_note(3, 5)]); // natural pos 5; also fits at 5
        let m0 = measure_with_voice(0, vec![b0a, b0b]);

        let b1 = beat_at_tick(0, vec![tab_note(3, 5), tab_note(2, 7)]); // pos 5
        let m1 = measure_with_voice(1, vec![b1]);

        let result = run(&[&m0, &m1]);

        // After look-ahead: last beat of m0 retargeted to pos 5.
        // Its finger = 5 - 5 + 1 = 1 (index), no shift relative to its new position.
        let last = find(&result[0], 3, 5).unwrap();
        assert_eq!(last.finger, 1);

        // Because m0's last beat is now at pos 5, the first beat of m1 has
        // the same position → no position_shift.
        let first_m1 = find(&result[1], 3, 5).unwrap();
        assert!(
            !first_m1.position_shift,
            "no shift at start of m1 after look-ahead"
        );
    }

    #[test]
    fn lookahead_does_not_retarget_when_chord_does_not_fit() {
        // Last beat of m0 has notes at frets 2 and 3 (natural pos 2).
        // Next measure opens at position 9 — frets 2,3 don't fit in [9, 12].
        // Look-ahead should NOT retarget.
        let b0 = beat_at_tick(0, vec![tab_note(3, 2), tab_note(2, 3)]);
        let m0 = measure_with_voice(0, vec![b0]);

        let b1 = beat_at_tick(0, vec![tab_note(3, 9)]);
        let m1 = measure_with_voice(1, vec![b1]);

        let result = run(&[&m0, &m1]);

        // m0 last beat should keep position 2.
        let a = find(&result[0], 3, 2).unwrap();
        assert_eq!(a.finger, 1); // still at position 2

        // m1 first beat should mark a position shift (jumping from 2 to 9).
        let b = find(&result[1], 3, 9).unwrap();
        assert!(b.position_shift);
    }

    // -------------------------------------------------------------------------
    // Tests: open strings and rests are skipped
    // -------------------------------------------------------------------------

    #[test]
    fn open_strings_produce_no_assignments() {
        let beat = beat_at_tick(0, vec![tab_note(1, 0), tab_note(2, 0), tab_note(3, 3)]);
        let m = measure_with_voice(0, vec![beat]);
        let result = run(&[&m]);
        // Open-string notes have no assignments; only fret 3 appears.
        assert_eq!(result[0].len(), 1);
        assert_eq!(result[0][0].fret, 3);
    }

    #[test]
    fn rest_beat_produces_no_assignments() {
        let mut rest_beat = beat_at_tick(0, vec![tab_note(1, 5)]);
        rest_beat.gp_rest = true;
        let m = measure_with_voice(0, vec![rest_beat]);
        let result = run(&[&m]);
        assert!(result[0].is_empty());
    }

    // -------------------------------------------------------------------------
    // Tests: multi-voice merge
    // -------------------------------------------------------------------------

    #[test]
    fn notes_from_multiple_voices_combined_at_same_tick() {
        use crate::model::optimized::beat::Voice;

        // Voice 0: string 1 fret 5 at tick 0.
        // Voice 1: string 2 fret 7 at tick 0.
        // Both should be part of the same chord event.
        let mut voices = HashMap::new();
        voices.insert(
            0u8,
            Voice {
                voice_id: 0,
                beats: vec![beat_at_tick(0, vec![tab_note(1, 5)])],
            },
        );
        voices.insert(
            1u8,
            Voice {
                voice_id: 1,
                beats: vec![beat_at_tick(0, vec![tab_note(2, 7)])],
            },
        );
        let m = MeasureData {
            measure_index: MeasureIndex(0),
            track_id: TrackId(0),
            repeat: None::<MeasureRepeat>,
            voices,
            gp_line_break: 0,
            gp_simile_mark: None,
        };

        let result = suggest_fingering(&[&m], &[]);
        // Both notes at the same tick → same chord → two assignments.
        assert_eq!(result[0].len(), 2);
        // Position = 5 (min fret); fret 5 → finger 1, fret 7 → finger 3.
        assert_eq!(find(&result[0], 1, 5).unwrap().finger, 1);
        assert_eq!(find(&result[0], 2, 7).unwrap().finger, 3);
    }
}
