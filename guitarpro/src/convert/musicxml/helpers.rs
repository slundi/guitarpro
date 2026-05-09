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
