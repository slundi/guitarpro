//! MSCZ pitch spelling: tonal pitch classes, transposing instruments and
//! their written keys.

use crate::convert::mscz::spelling::{
    pitch_class_from_tpc, pitch_from_midi_tpc, tpc_from_pitch, tpc_spells_midi, transpose_fifths,
    written_pitch,
};
use crate::model::optimized::{
    global::Transpose,
    metadata::normalized_fifths,
    note::{Pitch, PitchStep},
};

fn pitch(step: PitchStep, alter: i8, octave: u8) -> Pitch {
    Pitch {
        step,
        alter,
        octave,
    }
}

#[test]
fn tpc_round_trips_spellings() {
    for (p, tpc) in [
        (pitch(PitchStep::C, 0, 4), 14),
        (pitch(PitchStep::C, 1, 4), 21),
        (pitch(PitchStep::B, -1, 3), 12),
        (pitch(PitchStep::B, 1, 3), 26),
        (pitch(PitchStep::E, 0, 4), 18),
    ] {
        assert_eq!(tpc_from_pitch(p), tpc);
        let midi = p.midi() as u8;
        assert_eq!(pitch_from_midi_tpc(midi, tpc as i8), Some(p));
    }
}

#[test]
fn tells_octave_minus_one_from_misspellings() {
    // F-1 (MIDI 5) spelled F: a valid spelling, but below octave 0.
    assert_eq!(pitch_from_midi_tpc(5, 13), None);
    assert!(tpc_spells_midi(5, 13));
    // C♭0 is MIDI 11, so it still fits.
    assert_eq!(pitch_from_midi_tpc(11, 7), Some(pitch(PitchStep::C, -1, 0)));
    // MIDI 61 isn't any C.
    assert!(!tpc_spells_midi(61, 14));
}

#[test]
fn writes_transposing_instruments() {
    let tenor_sax = Transpose {
        diatonic: Some(-8),
        chromatic: -14,
        octave_change: None,
    };
    // Sounding E3 is written F♯4.
    assert_eq!(
        written_pitch(pitch(PitchStep::E, 0, 3), &tenor_sax),
        Some(pitch(PitchStep::F, 1, 4))
    );
    // A second down that is five semitones can't be spelled.
    let mismatched = Transpose {
        diatonic: Some(-1),
        chromatic: -5,
        octave_change: None,
    };
    assert_eq!(written_pitch(pitch(PitchStep::C, 0, 4), &mismatched), None);
    assert_eq!(transpose_fifths(&tenor_sax), -2);
    let alto_sax = Transpose {
        diatonic: None,
        chromatic: -9,
        octave_change: None,
    };
    assert_eq!(transpose_fifths(&alto_sax), -3);
    assert_eq!(normalized_fifths(3 + 5), -4);
}

#[test]
fn chord_roots_by_tonal_pitch_class() {
    let e_flat = pitch_class_from_tpc(11);
    assert_eq!(e_flat.map(|p| (p.step, p.alter)), Some((PitchStep::E, -1)));
    assert_eq!(e_flat.map(tpc_from_pitch), Some(11));
}
