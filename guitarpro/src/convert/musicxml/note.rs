//! Beat and note builders: rest notes, pitched notes, time modification.

use crate::{
    convert::musicxml::{helpers, notations},
    model::{
        legacy::{
            enums::{BeatStatus, NoteType as GpNoteType},
            key_signature::Duration,
            measure::Measure,
            track::Track,
        },
        musicxml::{self, note::NoteTypeValue},
    },
};

// ---------------------------------------------------------------------------
// Voice → notes
// ---------------------------------------------------------------------------

/// Convert all voices of a measure into a flat list of `MusicData` items.
///
/// GP supports up to 2 voices per measure. In MusicXML multiple voices are encoded
/// linearly: voice 1 notes come first, then a `<backup>` rewinds the time cursor
/// to the start of the measure, followed by voice 2 notes.
pub(super) fn build_voices(measure: &Measure, track: &Track) -> Vec<musicxml::measure::MusicData> {
    use musicxml::measure::{Backup, MusicData};

    let mut result: Vec<MusicData> = vec![];

    for (voice_idx, voice) in measure.voices.iter().enumerate() {
        // Skip empty voices
        if voice.beats.iter().all(|b| b.status == BeatStatus::Empty) {
            continue;
        }

        // Insert <backup> before voice 2+ to rewind the time cursor
        if voice_idx > 0 {
            let backup_duration: u32 = measure.voices[0]
                .beats
                .iter()
                .map(|b| helpers::duration_to_divisions(&b.duration))
                .sum();
            if backup_duration > 0 {
                result.push(MusicData::Backup(Backup {
                    duration: backup_duration,
                }));
            }
        }

        for beat in &voice.beats {
            if beat.status == BeatStatus::Empty {
                continue;
            }

            let is_rest = beat.status == BeatStatus::Rest || beat.notes.is_empty();
            let voice_num = format!("{}", voice_idx + 1);
            let divisions = helpers::duration_to_divisions(&beat.duration);
            let note_type = helpers::duration_to_note_type(&beat.duration);

            if is_rest {
                result.push(MusicData::Note(make_rest_note(
                    divisions,
                    note_type,
                    &beat.duration,
                    &voice_num,
                )));
            } else {
                for (note_idx, note) in beat.notes.iter().enumerate() {
                    let is_chord = note_idx > 0;
                    result.push(MusicData::Note(make_note(
                        note,
                        &track.strings,
                        divisions,
                        note_type,
                        &beat.duration,
                        &voice_num,
                        is_chord,
                    )));
                }
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Note constructors
// ---------------------------------------------------------------------------

pub(super) fn make_rest_note(
    divisions: u32,
    note_type: NoteTypeValue,
    duration: &Duration,
    voice: &str,
) -> musicxml::note::Note {
    use musicxml::note::{NoteType, Rest};
    musicxml::note::Note {
        grace: None,
        cue: None,
        pitch: None,
        rest: Some(Rest {
            measure: None,
            display_step: None,
            display_octave: None,
        }),
        unpitched: None,
        chord: None,
        duration: Some(divisions),
        ties: vec![],
        footnote: None,
        level: None,
        instrument: None,
        voice: Some(voice.to_string()),
        note_type: Some(NoteType {
            size: None,
            value: note_type,
        }),
        dots: if duration.dotted {
            vec![musicxml::note::Dot {
                default_x: None,
                default_y: None,
                placement: None,
            }]
        } else {
            vec![]
        },
        accidental: None,
        time_modification: build_time_modification(duration),
        stem: None,
        notehead: None,
        notehead_text: None,
        staff: None,
        beams: vec![],
        notations: vec![],
        lyrics: vec![],
        play: None,
        listen: None,
        print_object: None,
        print_dot: None,
        print_spacing: None,
        print_lyric: None,
        dynamics: None,
        end_dynamics: None,
        attack: None,
        release_time: None,
        default_x: None,
        default_y: None,
        time_only: None,
        pizzicato: None,
        id: None,
    }
}

pub(super) fn make_note(
    note: &crate::model::legacy::note::Note,
    strings: &[(i8, i8)],
    divisions: u32,
    note_type: NoteTypeValue,
    duration: &Duration,
    voice: &str,
    is_chord: bool,
) -> musicxml::note::Note {
    use musicxml::note::{NoteType, Pitch, Rest, Tie};

    let is_rest = note.kind == GpNoteType::Rest;
    let is_tie_stop = note.kind == GpNoteType::Tie;

    // Compute pitch from fret + open string tuning
    let (pitch, rest) = if is_rest {
        (
            None,
            Some(Rest {
                measure: None,
                display_step: None,
                display_octave: None,
            }),
        )
    } else {
        let midi = if note.string > 0 && (note.string as usize) <= strings.len() {
            (note.value as i8).saturating_add(strings[(note.string as usize) - 1].1)
        } else {
            note.value as i8
        };
        let (step, alter, octave) = helpers::midi_to_pitch(midi);
        (
            Some(Pitch {
                step,
                alter,
                octave,
            }),
            None,
        )
    };

    let ties = if is_tie_stop {
        vec![Tie {
            tie_type: "stop".to_string(),
            time_only: None,
        }]
    } else {
        vec![]
    };

    musicxml::note::Note {
        grace: None,
        cue: None,
        pitch,
        rest,
        unpitched: None,
        chord: if is_chord { Some(()) } else { None },
        duration: Some(divisions),
        ties,
        footnote: None,
        level: None,
        instrument: None,
        voice: Some(voice.to_string()),
        note_type: Some(NoteType {
            size: None,
            value: note_type,
        }),
        dots: if duration.dotted {
            vec![musicxml::note::Dot {
                default_x: None,
                default_y: None,
                placement: None,
            }]
        } else {
            vec![]
        },
        accidental: None,
        time_modification: build_time_modification(duration),
        stem: None,
        notehead: None,
        notehead_text: None,
        staff: None,
        beams: vec![],
        notations: notations::build_notations(note, strings),
        lyrics: vec![],
        play: None,
        listen: None,
        print_object: None,
        print_dot: None,
        print_spacing: None,
        print_lyric: None,
        dynamics: None,
        end_dynamics: None,
        attack: None,
        release_time: None,
        default_x: None,
        default_y: None,
        time_only: None,
        pizzicato: None,
        id: None,
    }
}

// ---------------------------------------------------------------------------
// Duration helpers
// ---------------------------------------------------------------------------

pub(super) fn build_time_modification(
    duration: &Duration,
) -> Option<musicxml::note::TimeModification> {
    if duration.is_default_tuplet() {
        return None;
    }
    Some(musicxml::note::TimeModification {
        actual_notes: duration.tuplet_enters,
        normal_notes: duration.tuplet_times,
        normal_type: None,
        normal_dots: vec![],
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::legacy::key_signature::{DURATION_QUARTER as QUARTER, Duration};

    // DIVISIONS lives in the parent (musicxml) module
    use super::super::DIVISIONS;

    fn quarter() -> Duration {
        Duration::default() // value=4, dotted=false, tuplet 1:1
    }

    fn triplet() -> Duration {
        Duration {
            value: QUARTER as u16,
            tuplet_enters: 3,
            tuplet_times: 2,
            ..Duration::default()
        }
    }

    #[test]
    fn time_modification_none_for_default_tuplet() {
        assert!(build_time_modification(&quarter()).is_none());
    }

    #[test]
    fn time_modification_some_for_triplet() {
        let tm = build_time_modification(&triplet()).unwrap();
        assert_eq!(tm.actual_notes, 3);
        assert_eq!(tm.normal_notes, 2);
    }

    #[test]
    fn rest_note_has_rest_and_no_pitch() {
        let d = quarter();
        let note = make_rest_note(DIVISIONS, NoteTypeValue::Quarter, &d, "1");
        assert!(note.rest.is_some());
        assert!(note.pitch.is_none());
    }

    #[test]
    fn rest_note_voice_matches_argument() {
        let d = quarter();
        let note = make_rest_note(DIVISIONS, NoteTypeValue::Quarter, &d, "2");
        assert_eq!(note.voice.as_deref(), Some("2"));
    }

    #[test]
    fn rest_note_dotted_adds_dot() {
        let d = Duration {
            dotted: true,
            ..quarter()
        };
        let note = make_rest_note(DIVISIONS, NoteTypeValue::Quarter, &d, "1");
        assert_eq!(note.dots.len(), 1);
    }

    #[test]
    fn rest_note_undotted_has_no_dots() {
        let note = make_rest_note(DIVISIONS, NoteTypeValue::Quarter, &quarter(), "1");
        assert!(note.dots.is_empty());
    }
}
