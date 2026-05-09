use crate::{
    convert::musicxml::DIVISIONS,
    model::{legacy::key_signature::Duration, musicxml::note::NoteTypeValue},
};

// ---------------------------------------------------------------------------
// Pitch helpers
// ---------------------------------------------------------------------------

/// Convert a MIDI note number to a MusicXML pitch triple `(step, alter, octave)`.
///
/// Uses sharps for accidentals (alter = 1.0).
/// MIDI 60 = C4 (middle C).
pub fn midi_to_pitch(midi: i8) -> (String, Option<f64>, i8) {
    // MIDI octave: octave = (midi / 12) - 1, but we need signed-safe arithmetic
    let midi_u = midi as i32;
    let pitch_class = midi_u.rem_euclid(12) as u8;
    let octave = (midi_u / 12 - 1) as i8;

    let (step, alter) = match pitch_class {
        0 => ("C", None),
        1 => ("C", Some(1.0)),
        2 => ("D", None),
        3 => ("D", Some(1.0)),
        4 => ("E", None),
        5 => ("F", None),
        6 => ("F", Some(1.0)),
        7 => ("G", None),
        8 => ("G", Some(1.0)),
        9 => ("A", None),
        10 => ("A", Some(1.0)),
        11 => ("B", None),
        _ => unreachable!(),
    };

    (step.to_string(), alter, octave)
}

// ---------------------------------------------------------------------------
// Duration helpers
// ---------------------------------------------------------------------------

/// Convert a legacy [`Duration`] to MusicXML `<divisions>` ticks.
///
/// Result is expressed in units of [`DIVISIONS`] per quarter note.
/// Dotted and tuplet adjustments are applied.
pub fn duration_to_divisions(d: &Duration) -> u32 {
    // Base ticks for this note value (quarter = DIVISIONS)
    let base = (DIVISIONS * 4) / d.value as u32;

    // Dotted: multiply by 3/2
    let dotted = if d.dotted { base + base / 2 } else { base };

    // Tuplet: scale by tuplet_times / tuplet_enters
    if d.tuplet_enters != 1 || d.tuplet_times != 1 {
        dotted * d.tuplet_times as u32 / d.tuplet_enters as u32
    } else {
        dotted
    }
}

/// Convert a legacy [`Duration`] value to the MusicXML [`NoteTypeValue`] symbol.
pub fn duration_to_note_type(d: &Duration) -> NoteTypeValue {
    match d.value {
        1 => NoteTypeValue::Whole,
        2 => NoteTypeValue::Half,
        4 => NoteTypeValue::Quarter,
        8 => NoteTypeValue::Eighth,
        16 => NoteTypeValue::N16th,
        32 => NoteTypeValue::N32nd,
        64 => NoteTypeValue::N64th,
        128 => NoteTypeValue::N128th,
        _ => NoteTypeValue::Quarter, // fallback
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::legacy::key_signature::{DURATION_QUARTER as QUARTER, Duration};

    // --- midi_to_pitch ---

    #[test]
    fn midi_60_is_c4() {
        assert_eq!(midi_to_pitch(60), ("C".to_string(), None, 4));
    }

    #[test]
    fn midi_69_is_a4() {
        // 69 / 12 = 5 remainder 9 → A, octave 5 - 1 = 4
        assert_eq!(midi_to_pitch(69), ("A".to_string(), None, 4));
    }

    #[test]
    fn midi_61_is_c_sharp_4() {
        assert_eq!(midi_to_pitch(61), ("C".to_string(), Some(1.0), 4));
    }

    #[test]
    fn midi_0_is_c_neg1() {
        // 0 / 12 = 0 remainder 0 → C, octave 0 - 1 = -1
        assert_eq!(midi_to_pitch(0), ("C".to_string(), None, -1));
    }

    // --- duration_to_divisions ---

    #[test]
    fn quarter_note_equals_divisions() {
        let d = Duration::default(); // value = QUARTER (4)
        assert_eq!(duration_to_divisions(&d), super::super::DIVISIONS);
    }

    #[test]
    fn half_note_is_double_divisions() {
        let d = Duration {
            value: 2,
            ..Duration::default()
        };
        assert_eq!(duration_to_divisions(&d), super::super::DIVISIONS * 2);
    }

    #[test]
    fn whole_note_is_four_divisions() {
        let d = Duration {
            value: 1,
            ..Duration::default()
        };
        assert_eq!(duration_to_divisions(&d), super::super::DIVISIONS * 4);
    }

    #[test]
    fn dotted_quarter_is_one_and_half_divisions() {
        let d = Duration {
            dotted: true,
            ..Duration::default()
        };
        let expected = super::super::DIVISIONS + super::super::DIVISIONS / 2;
        assert_eq!(duration_to_divisions(&d), expected);
    }

    #[test]
    fn triplet_quarter_rounds_correctly() {
        // 3:2 tuplet on a quarter: divisions * 2 / 3
        let d = Duration {
            value: QUARTER as u16,
            tuplet_enters: 3,
            tuplet_times: 2,
            ..Duration::default()
        };
        let expected = super::super::DIVISIONS * 2 / 3;
        assert_eq!(duration_to_divisions(&d), expected);
    }

    // --- duration_to_note_type ---

    #[test]
    fn note_type_whole() {
        let d = Duration {
            value: 1,
            ..Duration::default()
        };
        assert_eq!(duration_to_note_type(&d), NoteTypeValue::Whole);
    }

    #[test]
    fn note_type_quarter() {
        assert_eq!(
            duration_to_note_type(&Duration::default()),
            NoteTypeValue::Quarter
        );
    }

    #[test]
    fn note_type_eighth() {
        let d = Duration {
            value: 8,
            ..Duration::default()
        };
        assert_eq!(duration_to_note_type(&d), NoteTypeValue::Eighth);
    }

    #[test]
    fn note_type_unknown_value_falls_back_to_quarter() {
        let d = Duration {
            value: 7,
            ..Duration::default()
        };
        assert_eq!(duration_to_note_type(&d), NoteTypeValue::Quarter);
    }
}
