//! Chord name detection for the optimized model.
//!
//! Given the notes sounding simultaneously on a beat, identifies the chord
//! name: root, quality, optional slash-bass, and a confidence flag.
//!
//! ## Algorithm
//! 1. Convert every note in the beat to a MIDI pitch (tab string+fret or
//!    explicit `Pitch`). Exclude rest notes; include tie-end notes (still sounding).
//! 2. Reduce to a **pitch-class set** (pitches mod 12, deduplicated).
//! 3. For each of the 12 possible roots and each chord template, compute:
//!    - `extra`  = pitch classes in the input not covered by the template
//!    - `missing` = template pitch classes absent from the input
//! 4. The root must be present in the input.
//! 5. Pick the candidate that minimises `(extra, missing)`, preferring larger
//!    (more specific) templates on ties.
//! 6. Detect slash bass: if the lowest-pitched note's pitch class differs from
//!    the chosen root, record it as the bass (e.g. G/B).
//! 7. `uncertain = true` when `extra > 0 || missing > 0` (no clean match found).

use crate::model::optimized::{
    beat::Beat,
    note::{Note, PitchStep},
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A pitch class: 0 = C, 1 = C♯/D♭, …, 11 = B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PitchClass(pub u8);

impl PitchClass {
    /// Preferred sharp spelling (e.g. `"C#"`, `"F#"`).
    pub fn name_sharp(self) -> &'static str {
        SHARP_NAMES[self.0 as usize % 12]
    }

    /// Preferred flat spelling (e.g. `"Db"`, `"Gb"`).
    pub fn name_flat(self) -> &'static str {
        FLAT_NAMES[self.0 as usize % 12]
    }
}

const SHARP_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
const FLAT_NAMES: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];

impl std::fmt::Display for PitchClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_sharp())
    }
}

/// Chord quality / type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    // --- triads ---
    Major,
    Minor,
    Dim,
    Aug,
    Sus2,
    Sus4,
    // --- 7th chords ---
    Dom7,
    Maj7,
    Min7,
    MinMaj7,
    Dim7,
    HalfDim7,
    AugMaj7,
    Dom7Sus4,
    // --- added-tone ---
    Add9,
    MinAdd9,
    // --- extended ---
    Dom9,
    Maj9,
    Min9,
    // --- other ---
    Power, // root + fifth only; checked last
}

impl std::fmt::Display for ChordQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Dom7 => "7",
            ChordQuality::Maj7 => "maj7",
            ChordQuality::Min7 => "m7",
            ChordQuality::MinMaj7 => "m(maj7)",
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::Dim => "dim",
            ChordQuality::Dim7 => "dim7",
            ChordQuality::HalfDim7 => "m7b5",
            ChordQuality::Aug => "aug",
            ChordQuality::AugMaj7 => "aug(maj7)",
            ChordQuality::Dom7Sus4 => "7sus4",
            ChordQuality::Add9 => "add9",
            ChordQuality::MinAdd9 => "madd9",
            ChordQuality::Dom9 => "9",
            ChordQuality::Maj9 => "maj9",
            ChordQuality::Min9 => "m9",
            ChordQuality::Power => "5",
        };
        f.write_str(s)
    }
}

/// A recognised chord name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordName {
    /// Pitch class of the chord root (0 = C … 11 = B).
    pub root: PitchClass,
    pub quality: ChordQuality,
    /// Lowest-note pitch class when it differs from the root (slash chord, e.g. G/B).
    pub bass: Option<PitchClass>,
    /// `true` when no root/template pair gives a perfect match; best-effort result.
    pub uncertain: bool,
}

impl std::fmt::Display for ChordName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.root, self.quality)?;
        if let Some(bass) = self.bass {
            write!(f, "/{bass}")?;
        }
        if self.uncertain {
            f.write_str("?")?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Chord template table
// ---------------------------------------------------------------------------

struct Template {
    quality: ChordQuality,
    /// Semitone intervals above the root (root = 0 is always included).
    intervals: &'static [u8],
}

/// All templates to try, ordered roughly by specificity (most notes first so
/// that a 5-note extended chord wins over a triad when the score is tied).
/// The Power chord is last because it matches a subset of almost every chord.
static TEMPLATES: &[Template] = &[
    // 5-note extended
    Template {
        quality: ChordQuality::Dom9,
        intervals: &[0, 2, 4, 7, 10],
    },
    Template {
        quality: ChordQuality::Maj9,
        intervals: &[0, 2, 4, 7, 11],
    },
    Template {
        quality: ChordQuality::Min9,
        intervals: &[0, 2, 3, 7, 10],
    },
    // 4-note 7th / added-tone
    Template {
        quality: ChordQuality::Maj7,
        intervals: &[0, 4, 7, 11],
    },
    Template {
        quality: ChordQuality::Dom7,
        intervals: &[0, 4, 7, 10],
    },
    Template {
        quality: ChordQuality::Min7,
        intervals: &[0, 3, 7, 10],
    },
    Template {
        quality: ChordQuality::MinMaj7,
        intervals: &[0, 3, 7, 11],
    },
    Template {
        quality: ChordQuality::Dim7,
        intervals: &[0, 3, 6, 9],
    },
    Template {
        quality: ChordQuality::HalfDim7,
        intervals: &[0, 3, 6, 10],
    },
    Template {
        quality: ChordQuality::AugMaj7,
        intervals: &[0, 4, 8, 11],
    },
    Template {
        quality: ChordQuality::Dom7Sus4,
        intervals: &[0, 5, 7, 10],
    },
    Template {
        quality: ChordQuality::Add9,
        intervals: &[0, 2, 4, 7],
    },
    Template {
        quality: ChordQuality::MinAdd9,
        intervals: &[0, 2, 3, 7],
    },
    // 3-note triads
    Template {
        quality: ChordQuality::Major,
        intervals: &[0, 4, 7],
    },
    Template {
        quality: ChordQuality::Minor,
        intervals: &[0, 3, 7],
    },
    Template {
        quality: ChordQuality::Dim,
        intervals: &[0, 3, 6],
    },
    Template {
        quality: ChordQuality::Aug,
        intervals: &[0, 4, 8],
    },
    Template {
        quality: ChordQuality::Sus2,
        intervals: &[0, 2, 7],
    },
    Template {
        quality: ChordQuality::Sus4,
        intervals: &[0, 5, 7],
    },
    // 2-note — last resort
    Template {
        quality: ChordQuality::Power,
        intervals: &[0, 7],
    },
];

// ---------------------------------------------------------------------------
// Core function
// ---------------------------------------------------------------------------

/// Identify the chord sounding on `beat`.
///
/// `strings` maps each string number (1-based, same as `Note::string`) to its
/// open-string MIDI pitch, e.g. `[(1, 64), (2, 59), (3, 55), (4, 50), (5, 45), (6, 40)]`
/// for standard guitar tuning (string 1 = high E).
///
/// Returns `None` when fewer than two distinct pitch classes are sounding
/// (single note, unison, or all-rest beat).
pub fn identify_chord(beat: &Beat, strings: &[(i8, i8)]) -> Option<ChordName> {
    // --- 1. Collect MIDI pitches -----------------------------------------------
    // Include tie-end notes (they are still sounding); exclude rests.
    let midis: Vec<i32> = beat
        .notes
        .iter()
        .filter(|n| !n.gp_is_rest)
        .filter_map(|n| note_to_midi(n, strings))
        .collect();

    if midis.is_empty() {
        return None;
    }

    // --- 2. Build pitch-class set ----------------------------------------------
    let mut pc_present = [false; 12];
    for &m in &midis {
        pc_present[m.rem_euclid(12) as usize] = true;
    }
    let pc_count = pc_present.iter().filter(|&&b| b).count();

    if pc_count < 2 {
        return None; // unison or single note — not a chord
    }

    // --- 3. Bass note (lowest MIDI pitch) -------------------------------------
    let Some(&bass_midi) = midis.iter().min() else {
        return None;
    };
    let bass_pc = PitchClass(bass_midi.rem_euclid(12) as u8);

    // --- 4. Score all (root, template) pairs ----------------------------------
    // Score key: (extra, missing, neg_template_size, root) — minimised lexicographically.
    type ScoreKey = (usize, usize, isize, u8);

    let mut best_score: Option<ScoreKey> = None;
    let mut best_root = 0u8;
    let mut best_quality = ChordQuality::Major;

    for root in 0u8..12 {
        // Root must be present in the input.
        if !pc_present[root as usize] {
            continue;
        }

        for tmpl in TEMPLATES {
            // Build the set of pitch classes this template expects.
            let mut expected = [false; 12];
            for &interval in tmpl.intervals {
                expected[((root as usize) + interval as usize) % 12] = true;
            }

            let mut present_in_both = 0usize;
            for i in 0..12 {
                if pc_present[i] && expected[i] {
                    present_in_both += 1;
                }
            }
            let extra = pc_count - present_in_both;
            let missing = tmpl.intervals.len() - present_in_both;
            let neg_size = -(tmpl.intervals.len() as isize);
            let key: ScoreKey = (extra, missing, neg_size, root);

            if best_score.is_none_or(|b| key < b) {
                best_score = Some(key);
                best_root = root;
                best_quality = tmpl.quality;
            }
        }
    }

    let score = best_score?;
    let uncertain = score.0 > 0 || score.1 > 0; // extra > 0 || missing > 0

    let root = PitchClass(best_root);
    let bass = if bass_pc != root { Some(bass_pc) } else { None };

    Some(ChordName {
        root,
        quality: best_quality,
        bass,
        uncertain,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Convert a `Note` to a MIDI pitch integer.
/// Prefers tab (string + fret) over explicit `Pitch` when both are available.
/// Returns `None` for ghost notes with no pitch or tab information.
fn note_to_midi(note: &Note, strings: &[(i8, i8)]) -> Option<i32> {
    if let (Some(s), Some(f)) = (note.string, note.fret) {
        let open = strings
            .iter()
            .find(|&&(sn, _)| sn == s as i8)
            .map(|&(_, m)| m as i32)?;
        Some(open + f as i32)
    } else {
        // Scientific pitch: C4 = MIDI 60 → (octave + 1) * 12 + step + alter
        note.pitch
            .as_ref()
            .map(|p| (p.octave as i32 + 1) * 12 + pitch_step_semitone(p.step) + p.alter as i32)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::optimized::{
        beat::{Beat, Duration},
        note::{Note, NoteValue, Pitch, PitchStep},
    };

    // Standard guitar tuning: string 1 (high E) = 64, … string 6 (low E) = 40.
    const GUITAR_STRINGS: &[(i8, i8)] = &[(1, 64), (2, 59), (3, 55), (4, 50), (5, 45), (6, 40)];

    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    fn pitch_note(step: PitchStep, alter: i8, octave: u8) -> Note {
        Note {
            pitch: Some(Pitch {
                step,
                alter,
                octave,
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

    fn beat_from_notes(notes: Vec<Note>) -> Beat {
        Beat {
            tick_offset: 0,
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

    // -------------------------------------------------------------------------
    // Helper
    // -------------------------------------------------------------------------

    fn id(b: &Beat) -> Option<ChordName> {
        identify_chord(b, &[])
    }

    fn id_guitar(b: &Beat) -> Option<ChordName> {
        identify_chord(b, GUITAR_STRINGS)
    }

    // -------------------------------------------------------------------------
    // Tests: single note / unison → None
    // -------------------------------------------------------------------------

    #[test]
    fn single_note_returns_none() {
        let b = beat_from_notes(vec![pitch_note(PitchStep::C, 0, 4)]);
        assert_eq!(id(&b), None);
    }

    #[test]
    fn unison_returns_none() {
        // Two C notes at different octaves — only one pitch class.
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::C, 0, 4),
            pitch_note(PitchStep::C, 0, 5),
        ]);
        assert_eq!(id(&b), None);
    }

    #[test]
    fn empty_beat_returns_none() {
        let b = beat_from_notes(vec![]);
        assert_eq!(id(&b), None);
    }

    // -------------------------------------------------------------------------
    // Tests: clean triads (pitch-based)
    // -------------------------------------------------------------------------

    #[test]
    fn c_major_triad() {
        // C4, E4, G4
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::C, 0, 4),
            pitch_note(PitchStep::E, 0, 4),
            pitch_note(PitchStep::G, 0, 4),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(0)); // C
        assert_eq!(chord.quality, ChordQuality::Major);
        assert_eq!(chord.bass, None);
        assert!(!chord.uncertain);
        assert_eq!(chord.to_string(), "C");
    }

    #[test]
    fn a_minor_triad() {
        // A3, C4, E4
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::A, 0, 3),
            pitch_note(PitchStep::C, 0, 4),
            pitch_note(PitchStep::E, 0, 4),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(9)); // A
        assert_eq!(chord.quality, ChordQuality::Minor);
        assert!(!chord.uncertain);
        assert_eq!(chord.to_string(), "Am");
    }

    #[test]
    fn g_dominant_7th() {
        // G3, B3, D4, F4
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::G, 0, 3),
            pitch_note(PitchStep::B, 0, 3),
            pitch_note(PitchStep::D, 0, 4),
            pitch_note(PitchStep::F, 0, 4),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(7)); // G
        assert_eq!(chord.quality, ChordQuality::Dom7);
        assert!(!chord.uncertain);
        assert_eq!(chord.to_string(), "G7");
    }

    #[test]
    fn d_sus4() {
        // D3, G3, A3
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::D, 0, 3),
            pitch_note(PitchStep::G, 0, 3),
            pitch_note(PitchStep::A, 0, 3),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(2)); // D
        assert_eq!(chord.quality, ChordQuality::Sus4);
        assert!(!chord.uncertain);
    }

    #[test]
    fn b_diminished() {
        // B3, D4, F4
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::B, 0, 3),
            pitch_note(PitchStep::D, 0, 4),
            pitch_note(PitchStep::F, 0, 4),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(11)); // B
        assert_eq!(chord.quality, ChordQuality::Dim);
        assert!(!chord.uncertain);
    }

    #[test]
    fn c_augmented() {
        // C4, E4, G#4
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::C, 0, 4),
            pitch_note(PitchStep::E, 0, 4),
            pitch_note(PitchStep::G, 1, 4), // G#
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(0)); // C
        assert_eq!(chord.quality, ChordQuality::Aug);
        assert!(!chord.uncertain);
    }

    // -------------------------------------------------------------------------
    // Tests: power chord
    // -------------------------------------------------------------------------

    #[test]
    fn e_power_chord() {
        // E2, B2 — no third
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::E, 0, 2),
            pitch_note(PitchStep::B, 0, 2),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(4)); // E
        assert_eq!(chord.quality, ChordQuality::Power);
        assert!(!chord.uncertain);
        assert_eq!(chord.to_string(), "E5");
    }

    // -------------------------------------------------------------------------
    // Tests: extended / added-tone
    // -------------------------------------------------------------------------

    #[test]
    fn c_add9() {
        // C4, D4, E4, G4
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::C, 0, 4),
            pitch_note(PitchStep::D, 0, 4),
            pitch_note(PitchStep::E, 0, 4),
            pitch_note(PitchStep::G, 0, 4),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(0));
        assert_eq!(chord.quality, ChordQuality::Add9);
        assert!(!chord.uncertain);
    }

    #[test]
    fn g_major_9th() {
        // G3, B3, D4, F#4, A4
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::G, 0, 3),
            pitch_note(PitchStep::B, 0, 3),
            pitch_note(PitchStep::D, 0, 4),
            pitch_note(PitchStep::F, 1, 4), // F#
            pitch_note(PitchStep::A, 0, 4),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(7)); // G
        assert_eq!(chord.quality, ChordQuality::Maj9);
        assert!(!chord.uncertain);
    }

    // -------------------------------------------------------------------------
    // Tests: slash chord (bass ≠ root)
    // -------------------------------------------------------------------------

    #[test]
    fn g_over_b_slash_chord() {
        // B2, D3, G3, B3 — G major, first inversion, lowest note = B
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::B, 0, 2), // lowest
            pitch_note(PitchStep::D, 0, 3),
            pitch_note(PitchStep::G, 0, 3),
            pitch_note(PitchStep::B, 0, 3),
        ]);
        let chord = id(&b).unwrap();
        assert_eq!(chord.root, PitchClass(7)); // G
        assert_eq!(chord.quality, ChordQuality::Major);
        assert_eq!(chord.bass, Some(PitchClass(11))); // B
        assert!(!chord.uncertain);
        assert_eq!(chord.to_string(), "G/B");
    }

    // -------------------------------------------------------------------------
    // Tests: tab-based notes (string + fret)
    // -------------------------------------------------------------------------

    #[test]
    fn open_e_minor_chord_tab() {
        // Standard open Em chord on guitar:
        // String 6 fret 0 = E2 (40), String 5 fret 2 = B2 (47), String 4 fret 2 = E3 (52)
        // String 3 fret 0 = G3 (55), String 2 fret 0 = B3 (59), String 1 fret 0 = E4 (64)
        // Pitch classes: E(4), B(11), G(7) → Em
        let b = beat_from_notes(vec![
            tab_note(6, 0),
            tab_note(5, 2),
            tab_note(4, 2),
            tab_note(3, 0),
            tab_note(2, 0),
            tab_note(1, 0),
        ]);
        let chord = id_guitar(&b).unwrap();
        assert_eq!(chord.root, PitchClass(4)); // E
        assert_eq!(chord.quality, ChordQuality::Minor);
        assert!(!chord.uncertain);
    }

    #[test]
    fn open_a_major_chord_tab() {
        // Standard open A chord: x02220
        // String 5 fret 0 = A2(45), String 4 fret 2 = B3... wait let me compute:
        // String 5 open = A2 = 45, fret 0 → 45 (A)
        // String 4 open = D3 = 50, fret 2 → 52 (E)
        // String 3 open = G3 = 55, fret 2 → 57 (A)
        // String 2 open = B3 = 59, fret 2 → 61 (C#)
        // String 1 open = E4 = 64, fret 0 → 64 (E)
        // Pitch classes: A(9), E(4), C#(1) → A major
        let b = beat_from_notes(vec![
            tab_note(5, 0),
            tab_note(4, 2),
            tab_note(3, 2),
            tab_note(2, 2),
            tab_note(1, 0),
        ]);
        let chord = id_guitar(&b).unwrap();
        assert_eq!(chord.root, PitchClass(9)); // A
        assert_eq!(chord.quality, ChordQuality::Major);
        assert!(!chord.uncertain);
    }

    // -------------------------------------------------------------------------
    // Tests: uncertain / best-effort
    // -------------------------------------------------------------------------

    #[test]
    fn two_note_interval_uncertain() {
        // C and E only — not a complete triad; best guess is C major (missing G)
        let b = beat_from_notes(vec![
            pitch_note(PitchStep::C, 0, 4),
            pitch_note(PitchStep::E, 0, 4),
        ]);
        let chord = id(&b).unwrap();
        // Best match is C major (extra=0, missing=1) over C minor (extra=0, missing=1 too)
        // C major wins because it has a smaller root index... actually both missing=1.
        // C major root=0, C minor root=0 — same root, so template_size=3 for both, root=0.
        // They tie on all criteria except quality order in TEMPLATES: Major comes before Minor.
        assert_eq!(chord.root, PitchClass(0));
        assert!(chord.uncertain, "incomplete chord should be uncertain");
    }

    #[test]
    fn rest_notes_excluded() {
        let mut rest = pitch_note(PitchStep::G, 0, 3);
        rest.gp_is_rest = true;
        let b = beat_from_notes(vec![pitch_note(PitchStep::C, 0, 4), rest]);
        // Only C is sounding → single pitch class → None
        assert_eq!(id(&b), None);
    }

    // -------------------------------------------------------------------------
    // Tests: Display
    // -------------------------------------------------------------------------

    #[test]
    fn display_chord_name() {
        assert_eq!(
            ChordName {
                root: PitchClass(7),
                quality: ChordQuality::Min7,
                bass: None,
                uncertain: false,
            }
            .to_string(),
            "Gm7"
        );
        assert_eq!(
            ChordName {
                root: PitchClass(2),
                quality: ChordQuality::Major,
                bass: Some(PitchClass(6)),
                uncertain: false,
            }
            .to_string(),
            "D/F#"
        );
        assert_eq!(
            ChordName {
                root: PitchClass(5),
                quality: ChordQuality::Minor,
                bass: None,
                uncertain: true,
            }
            .to_string(),
            "Fm?"
        );
    }
}
