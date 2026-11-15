//! Pitch spelling shared by both conversion directions.
//!
//! MuseScore stores a note's sounding MIDI `<pitch>` plus its spelling as a
//! *tonal pitch class* (`<tpc>`): the position on the line of fifths with
//! C = 14, G = 15, F = 13, C♯ = 21, B♭ = 12. A transposing staff also stores
//! the written note's class (`<tpc2>`), and its `<KeySig>` the written key
//! (`<actualKey>`) next to the concert one.
//!
//! [`Note::pitch`](crate::model::optimized::note::Note::pitch) is the
//! sounding pitch; the written one follows from the instrument's
//! [`Transpose`].

use crate::model::optimized::{
    global::Transpose,
    note::{Pitch, PitchStep},
};

/// Tonal pitch class of C.
pub(crate) const TPC_C: i32 = 14;

/// The tonal pitch class (`<tpc>`) of a spelled pitch.
pub(crate) fn tpc_from_pitch(pitch: Pitch) -> i32 {
    TPC_C + pitch.fifths()
}

/// The spelled pitch for a MIDI `<pitch>` and its `<tpc>`; `None` when the
/// class names an accidental beyond a double sharp/flat, isn't a spelling
/// of the pitch at all (see [`tpc_spells_midi`]), or the spelled note lies
/// in octave −1, below what [`Pitch`] holds.
pub(crate) fn pitch_from_midi_tpc(midi: u8, tpc: i8) -> Option<Pitch> {
    let class = pitch_class_from_tpc(tpc as i32)?;
    let natural = natural_midi(midi, class)?;
    let octave = u8::try_from(natural.div_euclid(12) - 1).ok()?;
    Some(Pitch { octave, ..class })
}

/// Whether `<tpc>` names a spelling of the MIDI `<pitch>` (in any octave).
pub(crate) fn tpc_spells_midi(midi: u8, tpc: i8) -> bool {
    pitch_class_from_tpc(tpc as i32).is_some_and(|class| natural_midi(midi, class).is_some())
}

/// The MIDI number of the spelled class's unaltered C in the octave whose
/// spelled note matches the MIDI number (B♯3 is MIDI 60, so 48); `None`
/// when the class isn't a spelling of the pitch.
fn natural_midi(midi: u8, class: Pitch) -> Option<i32> {
    let natural = midi as i32 - class.alter as i32 - class.step.semitone();
    (natural.rem_euclid(12) == 0).then_some(natural)
}

/// A pitch class by its tonal pitch class (a chord symbol's `<root>`), in
/// octave 4; `None` beyond a double sharp/flat.
pub(crate) fn pitch_class_from_tpc(tpc: i32) -> Option<Pitch> {
    Pitch::from_fifths(tpc - TPC_C)
}

/// The transposition's diatonic steps; files that only give the semitones
/// get the nearest interval (−2 → a second, −9 → a sixth).
pub(crate) fn transpose_diatonic(transpose: &Transpose) -> i32 {
    transpose
        .diatonic
        .map(i32::from)
        .unwrap_or_else(|| (transpose.chromatic as f32 * 7.0 / 12.0).round() as i32)
}

/// What the transposition adds to a written key to give the concert one,
/// on the circle of fifths: −2 for a B♭ instrument, −3 for an E♭ one.
pub(crate) fn transpose_fifths(transpose: &Transpose) -> i32 {
    7 * transpose.chromatic as i32 - 12 * transpose_diatonic(transpose)
}

/// The written pitch of a sounding one on a transposing instrument: its
/// letter moved back by the diatonic steps, the accidental following from
/// the semitones. `None` when that needs more than a double sharp/flat
/// (diatonic and chromatic steps that don't fit together) or falls below
/// octave 0.
pub(crate) fn written_pitch(sounding: Pitch, transpose: &Transpose) -> Option<Pitch> {
    let index = sounding.step.index() + 7 * sounding.octave as i32 - transpose_diatonic(transpose);
    let step = PitchStep::from_index(index);
    let octave = u8::try_from(index.div_euclid(7)).ok()?;
    let natural = (octave as i32 + 1) * 12 + step.semitone();
    let alter = sounding.midi() - transpose.chromatic as i32 - natural;
    if !(-2..=2).contains(&alter) {
        return None;
    }
    Some(Pitch {
        step,
        alter: alter as i8,
        octave,
    })
}
