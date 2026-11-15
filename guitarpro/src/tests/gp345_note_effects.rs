use super::common::read_file;
use crate::model::legacy::song::Song;

#[test]
fn test_gp3_effects() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Effects.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_effects() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Effects.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_effects() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Effects.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Harmonics.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Harmonics.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Harmonics.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_vibrato() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Vibrato.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_vibrato() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/vibrato.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_basic_bend() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/basic-bend.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_ghost_note() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/ghost_note.gp3")))
        .unwrap();
}
#[test]
fn test_gp5_grace() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/grace.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_heavy_accent() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/heavy-accent.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_high_pitch() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/high-pitch.gp3")))
        .unwrap();
}

#[test]
fn test_gp5_tremolos() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/tremolos.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_trill() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/trill.gp4")))
        .unwrap();
}

/// A tied note takes the fret of the note before it on its string in its
/// voice; after a rest, the voice's last one there, and else an earlier
/// measure's in any voice.
#[test]
fn a_tied_note_takes_the_fret_before_it() {
    use crate::model::legacy::{
        beat::{Beat, Voice},
        enums::{BeatStatus, NoteType},
        measure::{Measure, resolve_tied_frets},
        note::Note,
        track::Track,
    };

    let note = |string: i8, value: i16, kind: NoteType| Note {
        string,
        value,
        kind,
        ..Note::default()
    };
    let beat = |notes: Vec<Note>| Beat {
        status: if notes.is_empty() {
            BeatStatus::Rest
        } else {
            BeatStatus::Normal
        },
        notes,
        ..Beat::default()
    };
    let voice = |beats: Vec<Beat>| Voice {
        beats,
        ..Voice::default()
    };
    let measure = |voices: Vec<Voice>| Measure {
        voices,
        ..Measure::default()
    };
    let mut track = Track {
        measures: vec![
            measure(vec![voice(vec![
                beat(vec![
                    note(1, 5, NoteType::Normal),
                    note(2, 7, NoteType::Normal),
                ]),
                beat(vec![]),
                beat(vec![note(1, 0, NoteType::Tie)]),
            ])]),
            measure(vec![
                voice(vec![
                    beat(vec![note(1, 3, NoteType::Normal)]),
                    beat(vec![note(1, 0, NoteType::Tie)]),
                ]),
                voice(vec![beat(vec![note(2, 0, NoteType::Tie)])]),
            ]),
        ],
        ..Track::default()
    };
    resolve_tied_frets(&mut track);
    let frets: Vec<Vec<i16>> = track
        .measures
        .iter()
        .flat_map(|m| &m.voices)
        .map(|v| {
            v.beats
                .iter()
                .flat_map(|b| &b.notes)
                .map(|n| n.value)
                .collect()
        })
        .collect();
    assert_eq!(frets, vec![vec![5, 7, 5], vec![3, 3], vec![7]]);
}

/// Of notes starting together on a string in several voices, a tie in the
/// next measure is from its own voice's.
#[test]
fn a_tie_across_a_barline_is_from_its_own_voice_of_notes_together() {
    use crate::model::legacy::{
        beat::{Beat, Voice},
        enums::{BeatStatus, NoteType},
        measure::{Measure, resolve_tied_frets},
        note::Note,
        track::Track,
    };

    let voice = |value: i16, kind: NoteType| Voice {
        beats: vec![Beat {
            status: BeatStatus::Normal,
            notes: vec![Note {
                string: 3,
                value,
                kind,
                ..Note::default()
            }],
            ..Beat::default()
        }],
        ..Voice::default()
    };
    let measure = |voices: Vec<Voice>| Measure {
        voices,
        ..Measure::default()
    };
    let mut track = Track {
        measures: vec![
            measure(vec![voice(5, NoteType::Normal), voice(7, NoteType::Normal)]),
            measure(vec![voice(0, NoteType::Tie), voice(0, NoteType::Tie)]),
        ],
        ..Track::default()
    };
    resolve_tied_frets(&mut track);
    let tied: Vec<i16> = track.measures[1]
        .voices
        .iter()
        .map(|v| v.beats[0].notes[0].value)
        .collect();
    assert_eq!(tied, vec![5, 7]);
}
