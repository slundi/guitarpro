//! Roundtrip tests: `optimized::LoadedScore` → `legacy::Song` → `optimized::LoadedScore`.
//!
//! Roadmap `Conversion`. The companion `roundtrip_optimized` test proves the
//! *other* direction (`legacy → optimized → legacy`) is byte-lossless; this test
//! proves the reverse: an optimized score, converted down to the legacy `Song`
//! and back up, is unchanged.
//!
//! A source optimized score can only be obtained from a legacy `Song` or a
//! MusicXML document. We use GP binary files (parsed to `Song`, then lifted to
//! `LoadedScore` via `legacy_song_to_loaded_score`) so the roundtrip under test
//! — `LoadedScore → Song → LoadedScore` — uses **only** the legacy↔optimized
//! converters and never passes through the MusicXML model or any other
//! intermediate representation.

use std::collections::BTreeMap;

use crate::tests::common::TestResult;
use crate::{
    convert::{
        legacy::loaded_score_to_legacy_song, optimized::legacy::legacy_song_to_loaded_score,
    },
    model::{
        legacy::song::Song,
        optimized::{LoadedScore, beat::Voice, global::Score},
    },
};

/// A deterministic, comparable string form of a [`Score`].
///
/// Every field has a stable `Debug` representation except each measure's
/// `voices`, a `HashMap` (per-instance random seed → random iteration order).
/// Voice *order* is meaningless (voices are keyed by id), so we sort by id.
fn canonical_score(score: &Score) -> String {
    let mut out = String::new();
    out.push_str(&format!("META {:?}\n", score.metadata));
    out.push_str(&format!("INSTRUMENTS {:?}\n", score.instruments));
    out.push_str(&format!("STAVES {:?}\n", score.staves));
    out.push_str(&format!("GROUPS {:?}\n", score.groups));
    out.push_str(&format!("TIMELINE {:?}\n", score.timeline));
    out.push_str(&format!("LYRIC_LINES {:?}\n", score.lyric_lines));
    out.push_str(&format!(
        "LYRIC_PROJECTIONS {:?}\n",
        score.lyric_projections
    ));
    out.push_str(&format!("DEFAULTS {:?}\n", score.defaults));
    for track in &score.tracks {
        out.push_str(&format!(
            "TRACK id={:?} name={:?} instrument={:?} staves={:?}\n",
            track.id, track.name, track.instrument, track.staves
        ));
        // `measures` is a BTreeMap: iterated in sorted key order.
        for (index, measure_data) in &track.measures {
            let voices: BTreeMap<u8, &Voice> = measure_data
                .voices
                .iter()
                .map(|(id, voice)| (*id, voice))
                .collect();
            out.push_str(&format!(
                "  MEASURE {index:?} repeat={:?} line_break={} simile={:?} voices={voices:?}\n",
                measure_data.repeat, measure_data.gp_line_break, measure_data.gp_simile_mark,
            ));
        }
    }
    out
}

/// One `LoadedScore → Song → LoadedScore` roundtrip.
fn roundtrip(source: &LoadedScore) -> LoadedScore {
    legacy_song_to_loaded_score(&loaded_score_to_legacy_song(source))
}

/// Number of notes across every track/measure/voice/beat.
fn note_count(score: &Score) -> usize {
    score
        .tracks
        .iter()
        .flat_map(|track| track.measures.values())
        .flat_map(|measure| measure.voices.values())
        .flat_map(|voice| &voice.beats)
        .map(|beat| beat.notes.len())
        .sum()
}

/// Assert the `optimized → legacy → optimized` roundtrip is well-behaved:
///
/// * **Structural preservation** — track count, per-track measure counts and the
///   total note count survive one roundtrip.
/// * **Fixed point** — the first roundtrip may normalize the model (e.g. the GP
///   writer requires five lyric lines, so `optimized → legacy` pads them); a
///   *second* roundtrip must then change nothing.
fn assert_roundtrip(source: &LoadedScore) -> Result<(), String> {
    let rt1 = roundtrip(source);

    // --- Structural preservation (source → rt1) ---
    if rt1.score.tracks.len() != source.score.tracks.len() {
        return Err(format!(
            "track count changed: {} → {}",
            source.score.tracks.len(),
            rt1.score.tracks.len()
        ));
    }
    for (track_idx, (before, after)) in source
        .score
        .tracks
        .iter()
        .zip(rt1.score.tracks.iter())
        .enumerate()
    {
        if before.measures.len() != after.measures.len() {
            return Err(format!(
                "track {track_idx} measure count changed: {} → {}",
                before.measures.len(),
                after.measures.len()
            ));
        }
    }
    if note_count(&rt1.score) != note_count(&source.score) {
        return Err(format!(
            "note count changed: {} → {}",
            note_count(&source.score),
            note_count(&rt1.score)
        ));
    }

    // --- Fixed point (rt1 → rt2 must be a no-op) ---
    let rt2 = roundtrip(&rt1);
    let before = canonical_score(&rt1.score);
    let after = canonical_score(&rt2.score);
    if before != after {
        let pos = before
            .bytes()
            .zip(after.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| before.len().min(after.len()));
        let lo = pos.saturating_sub(40);
        let show = |s: &str| {
            let hi = (pos + 80).min(s.len());
            s.get(lo..hi).unwrap_or("").to_string()
        };
        return Err(format!(
            "roundtrip not idempotent at byte {pos}:\n  rt1: …{}…\n  rt2: …{}…",
            show(&before),
            show(&after)
        ));
    }
    Ok(())
}

fn run_via_legacy<R>(label: &str, ext: &str, read_fn: R)
where
    R: Fn(&mut Song, &[u8]),
{
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(test_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|x| x == ext))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let fname = path.file_name().unwrap().to_str().unwrap().to_string();
        let data = fs::read(&path).unwrap();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Obtain a source optimized score from the GP file (via legacy).
            let mut song = Song::default();
            read_fn(&mut song, &data);
            let source = legacy_song_to_loaded_score(&song);

            // Roundtrip under test: optimized → legacy → optimized.
            assert_roundtrip(&source)
        }));

        match result {
            Ok(Ok(())) => pass += 1,
            Ok(Err(msg)) => failures.push(format!("{fname}: {}", &msg[..msg.len().min(4000)])),
            Err(_) => failures.push(format!("{fname}: panicked")),
        }
    }

    eprintln!(
        "{label} optimized→legacy→optimized: {pass} pass, {} fail out of {}",
        failures.len(),
        pass + failures.len()
    );
    for failure in &failures {
        eprintln!("FAIL: {failure}");
    }
    assert!(
        failures.is_empty(),
        "{} {label} file(s) failed optimized→legacy→optimized roundtrip",
        failures.len()
    );
}

#[test]
fn test_gp3_optimized_legacy_optimized_roundtrip() {
    run_via_legacy("GP3", "gp3", |song, data| song.read_gp3(data).unwrap());
}

#[test]
fn test_gp4_optimized_legacy_optimized_roundtrip() {
    run_via_legacy("GP4", "gp4", |song, data| song.read_gp4(data).unwrap());
}

#[test]
fn test_gp5_optimized_legacy_optimized_roundtrip() {
    run_via_legacy("GP5", "gp5", |song, data| song.read_gp5(data).unwrap());
}

#[test]
fn test_gpx_optimized_legacy_optimized_roundtrip() {
    run_via_legacy("GPX", "gpx", |song, data| song.read_gpx(data).unwrap());
}

#[test]
fn test_gp7_optimized_legacy_optimized_roundtrip() {
    run_via_legacy("GP7", "gp", |song, data| song.read_gp(data).unwrap());
}

/// GP marks only a tie's end, tying it from the note before: a MusicXML
/// tie stop without its start (33i, bar 3) mustn't become a tie from the
/// untied note before it (bar 2) through a GP file.
#[test]
fn an_unstarted_tie_stays_untied_through_a_guitar_pro_song() -> TestResult {
    use crate::convert::optimized::score_partwise_to_loaded_score;
    use crate::model::optimized::note::TieType;
    use crate::tests::common::read_musicxml;

    let loaded = score_partwise_to_loaded_score(&read_musicxml("33i-Ties-NotEnded.xml")?);
    let back = legacy_song_to_loaded_score(&loaded_score_to_legacy_song(&loaded));
    let ties: Vec<Vec<Option<TieType>>> = back.score.tracks[0]
        .measures
        .values()
        .map(|data| {
            data.voices
                .values()
                .flat_map(|voice| &voice.beats)
                .flat_map(|beat| &beat.notes)
                .map(|note| note.tie)
                .collect()
        })
        .collect();
    // Bar 1's and 3's unended starts can't be kept; bar 4's tie is.
    assert_eq!(
        ties,
        vec![
            vec![None],
            vec![None],
            vec![None],
            vec![Some(TieType::Start)],
            vec![Some(TieType::End)],
        ]
    );
    Ok(())
}

/// A GP tie is from the last note on its string in its voice, past rests
/// and empty beats, or with none there, from an earlier measure's in any
/// voice (not one in the same measure).
#[test]
fn a_tie_is_from_the_last_note_on_its_string() {
    use crate::model::legacy::{
        enums::BeatStatus,
        measure::{TieChains, on_tie_chains},
    };

    let mut chains = TieChains::new();
    assert!(on_tie_chains(&BeatStatus::Normal));
    assert!(!on_tie_chains(&BeatStatus::Rest));
    assert!(!on_tie_chains(&BeatStatus::Empty));
    chains.next_measure();
    chains.note(0, 1, 0, "bar 1, voice 1");
    assert_eq!(chains.previous(1, 1), None);
    chains.next_measure();
    assert_eq!(chains.previous(0, 1), Some(&"bar 1, voice 1"));
    assert_eq!(chains.previous(1, 1), Some(&"bar 1, voice 1"));
    chains.note(1, 1, 0, "bar 2, voice 2");
    assert_eq!(chains.previous(0, 1), Some(&"bar 1, voice 1"));
    assert_eq!(chains.previous(1, 1), Some(&"bar 2, voice 2"));
    // From an earlier measure: the note that sounds last, not the last
    // voice's.
    chains.next_measure();
    chains.note(0, 2, 960, "bar 3, voice 1, beat 2");
    chains.note(1, 2, 0, "bar 3, voice 2, beat 1");
    chains.next_measure();
    assert_eq!(chains.previous(2, 2), Some(&"bar 3, voice 1, beat 2"));
    // A voice's note in an older measure is past: the measure before's
    // last note on the string is the one.
    chains.note(0, 3, 0, "bar 4, voice 1, beat 1");
    chains.note(1, 3, 960, "bar 4, voice 2, beat 2");
    chains.next_measure();
    assert_eq!(chains.previous(0, 3), Some(&"bar 4, voice 2, beat 2"));
}

/// A tied note sounds the note it's tied from, also in a song whose tied
/// notes still have the file's frets (not resolved by a reader).
#[test]
fn a_tied_notes_pitch_is_the_one_it_is_tied_from() -> TestResult {
    use crate::model::legacy::enums::NoteType;
    use crate::model::optimized::{note::Pitch, track::Track};
    use crate::tests::common::read_file;

    let pitches = |tracks: &[Track]| -> Vec<Option<Pitch>> {
        tracks
            .iter()
            .flat_map(|track| track.measures.values())
            .flat_map(|data| {
                let mut voices: Vec<_> = data.voices.iter().collect();
                voices.sort_by_key(|(id, _)| **id);
                voices.into_iter().map(|(_, voice)| voice)
            })
            .flat_map(|voice| &voice.beats)
            .flat_map(|beat| &beat.notes)
            .map(|note| note.pitch)
            .collect()
    };
    let mut song = Song::default();
    song.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    let resolved = pitches(&legacy_song_to_loaded_score(&song).score.tracks);
    let mut ties = 0;
    for note in song
        .tracks
        .iter_mut()
        .flat_map(|track| &mut track.measures)
        .flat_map(|measure| &mut measure.voices)
        .flat_map(|voice| &mut voice.beats)
        .flat_map(|beat| &mut beat.notes)
        .filter(|note| note.kind == NoteType::Tie)
    {
        note.value = 0;
        ties += 1;
    }
    assert!(ties > 0);
    let unresolved = pitches(&legacy_song_to_loaded_score(&song).score.tracks);
    assert_eq!(unresolved, resolved);
    Ok(())
}

/// A GP tie after a rest stays a tie through the optimized model.
#[test]
fn a_tie_after_a_rest_stays_tied_through_the_optimized_model() -> TestResult {
    use crate::model::legacy::{
        beat::Beat,
        enums::{BeatStatus, NoteType},
    };
    use crate::tests::common::read_file;

    let mut song = Song::default();
    song.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    // The first tied note's place: track, measure, voice, beat.
    let place = song.tracks.iter().enumerate().find_map(|(t, track)| {
        track.measures.iter().enumerate().find_map(|(m, measure)| {
            measure.voices.iter().enumerate().find_map(|(v, voice)| {
                voice
                    .beats
                    .iter()
                    .position(|beat| beat.notes.iter().any(|n| n.kind == NoteType::Tie))
                    .map(|b| (t, m, v, b))
            })
        })
    });
    let (t, m, v, b) = place.ok_or("a tied note")?;
    let rest = Beat {
        status: BeatStatus::Rest,
        notes: Vec::new(),
        ..song.tracks[t].measures[m].voices[v].beats[b].clone()
    };
    song.tracks[t].measures[m].voices[v].beats.insert(b, rest);

    let back = loaded_score_to_legacy_song(&legacy_song_to_loaded_score(&song));
    let tied = &back.tracks[t].measures[m].voices[v].beats[b + 1];
    assert!(tied.notes.iter().any(|n| n.kind == NoteType::Tie));
    Ok(())
}

/// Scores saved from GP before ties were linked on import marked only each
/// tie's end, with its pitch unset: loading one links them as the import
/// does now, so exports neither re-attack tied notes nor read them as rests.
#[test]
fn a_saved_scores_unlinked_ties_are_linked_on_load() -> TestResult {
    use crate::convert::optimized::score_partwise_to_loaded_score;
    use crate::model::optimized::note::TieType;
    use crate::tests::common::read_musicxml;

    let loaded = score_partwise_to_loaded_score(&read_musicxml("33b-Spanners-Tie.xml")?);
    let mut score = legacy_song_to_loaded_score(&loaded_score_to_legacy_song(&loaded)).score;
    let notes = |score: &crate::model::optimized::global::Score| {
        score
            .tracks
            .iter()
            .flat_map(|track| track.measures.values())
            .flat_map(|data| data.voices.values())
            .flat_map(|voice| &voice.beats)
            .flat_map(|beat| &beat.notes)
            .map(|note| (note.tie, note.pitch, note.fret))
            .collect::<Vec<_>>()
    };
    let expected = notes(&score);
    assert!(
        expected
            .iter()
            .any(|(tie, ..)| *tie == Some(TieType::Start))
    );
    // As the import saved it then.
    for note in score
        .tracks
        .iter_mut()
        .flat_map(|track| track.measures.values_mut())
        .flat_map(|data| data.voices.values_mut())
        .flat_map(|voice| &mut voice.beats)
        .flat_map(|beat| &mut beat.notes)
    {
        note.tie = note.tie.filter(|tie| tie.ends_tie()).map(|_| TieType::End);
        if note.tie.is_some() {
            note.pitch = None;
        }
    }
    let json = crate::tests::common::unversioned_save(&score)?;
    let reloaded: crate::model::optimized::global::Score = serde_json::from_str(&json)?;
    assert_eq!(notes(&reloaded), expected);
    assert_eq!(
        reloaded.format_version,
        crate::model::optimized::global::SCORE_FORMAT_VERSION
    );

    // A save with a format version is in its form: nothing is migrated,
    // its ties stay as saved.
    let saved = notes(&score);
    let versioned = serde_json::to_string(&score)?;
    let reloaded: crate::model::optimized::global::Score = serde_json::from_str(&versioned)?;
    assert_eq!(notes(&reloaded), saved);
    Ok(())
}

/// Scores saved from GP before dynamics were kept as changes marked every
/// beat's level but left GP's default forte unmarked: loading one marks the
/// changes as the import does now, so a forte after a quieter passage isn't
/// exported at the quieter level.
#[test]
fn a_saved_scores_unmarked_forte_beats_are_migrated_on_load() -> TestResult {
    use crate::convert::legacy::dynamics::velocity_to_dynamic;
    use crate::model::legacy::effects::DEFAULT_VELOCITY;
    use crate::model::optimized::beat::Dynamic;

    let mut song = Song::default();
    song.read_gp5(&crate::tests::common::read_file(String::from(
        "test/dynamic.gp5",
    )))?;
    let mut score = legacy_song_to_loaded_score(&song).score;
    // By track, measure, voice and beat (voices are in a hash map).
    let dynamics = |score: &Score| {
        let mut dynamics = Vec::new();
        for (track, track_data) in score.tracks.iter().enumerate() {
            for (measure, data) in &track_data.measures {
                for (&voice, voice_data) in &data.voices {
                    for (index, beat) in voice_data.beats.iter().enumerate() {
                        dynamics.push((track, measure.0, voice, index, beat.dynamic.clone()));
                    }
                }
            }
        }
        dynamics.sort_by_key(|&(track, measure, voice, index, _)| (track, measure, voice, index));
        dynamics
    };
    let expected = dynamics(&score);
    // A forte marked as a change back from another level.
    assert!(
        expected
            .iter()
            .any(|(.., dynamic)| *dynamic == Some(Dynamic::F))
    );
    // As the import saved it then: each beat's level, forte left unmarked.
    for beat in score
        .tracks
        .iter_mut()
        .flat_map(|track| track.measures.values_mut())
        .flat_map(|data| data.voices.values_mut())
        .flat_map(|voice| &mut voice.beats)
    {
        beat.dynamic = beat
            .notes
            .first()
            .and_then(|note| note.gp_velocity)
            .filter(|&velocity| velocity != DEFAULT_VELOCITY)
            .map(velocity_to_dynamic);
    }
    let json = crate::tests::common::unversioned_save(&score)?;
    let reloaded: Score = serde_json::from_str(&json)?;
    assert_eq!(dynamics(&reloaded), expected);
    Ok(())
}

/// A save from a newer version of the library, in a form this one doesn't
/// know, is refused rather than misread.
#[test]
fn a_save_newer_than_the_library_is_refused() -> TestResult {
    let mut song = Song::default();
    song.read_gp5(&crate::tests::common::read_file(String::from(
        "test/dynamic.gp5",
    )))?;
    let score = legacy_song_to_loaded_score(&song).score;
    let mut json: serde_json::Value = serde_json::to_value(&score)?;
    json["format_version"] = (crate::model::optimized::global::SCORE_FORMAT_VERSION + 1).into();
    assert!(serde_json::from_value::<Score>(json).is_err());
    Ok(())
}

/// A score is from GP by GP's measure fields or, with those stripped, by
/// its notes' own velocities (only the GP import gives them): the older
/// saves' migrations read its voltas, ties and dynamics as GP's.
#[test]
fn a_gp_score_without_gp_measure_fields_is_still_from_gp() -> TestResult {
    use crate::convert::optimized::score_partwise_to_loaded_score;
    use crate::tests::common::read_musicxml;

    let mut song = Song::default();
    song.read_gp5(&crate::tests::common::read_file(String::from(
        "test/dynamic.gp5",
    )))?;
    let mut score = legacy_song_to_loaded_score(&song).score;
    for measure in &mut score.timeline {
        measure.gp_beams = None;
        measure.gp_fermatas.clear();
        measure.gp_free_time = false;
        if let Some(marker) = &mut measure.marker {
            marker.gp_color = None;
        }
    }
    assert!(score.is_gp_sourced());

    let musicxml = score_partwise_to_loaded_score(&read_musicxml("33b-Spanners-Tie.xml")?);
    assert!(!musicxml.score.is_gp_sourced());
    Ok(())
}

/// Only a score from GP has its unlinked ties linked on load: another
/// format's tie end without a start stays as it was saved.
#[test]
fn a_saved_scores_unstarted_tie_from_another_format_stays() -> TestResult {
    use crate::convert::optimized::score_partwise_to_loaded_score;
    use crate::model::optimized::note::TieType;
    use crate::tests::common::read_musicxml;

    let mut score = score_partwise_to_loaded_score(&read_musicxml("33b-Spanners-Tie.xml")?).score;
    assert!(
        score
            .timeline
            .iter()
            .all(|measure| measure.gp_beams.is_none())
    );
    for note in score
        .tracks
        .iter_mut()
        .flat_map(|track| track.measures.values_mut())
        .flat_map(|data| data.voices.values_mut())
        .flat_map(|voice| &mut voice.beats)
        .flat_map(|beat| &mut beat.notes)
    {
        note.tie = note.tie.filter(|tie| tie.ends_tie()).map(|_| TieType::End);
    }
    let notes = |score: &crate::model::optimized::global::Score| {
        score
            .tracks
            .iter()
            .flat_map(|track| track.measures.values())
            .flat_map(|data| data.voices.values())
            .flat_map(|voice| &voice.beats)
            .flat_map(|beat| &beat.notes)
            .map(|note| (note.tie, note.pitch, note.fret))
            .collect::<Vec<_>>()
    };
    let expected = notes(&score);
    assert!(expected.iter().any(|(tie, ..)| *tie == Some(TieType::End)));
    let reloaded: crate::model::optimized::global::Score =
        serde_json::from_str(&crate::tests::common::unversioned_save(&score)?)?;
    assert_eq!(notes(&reloaded), expected);
    Ok(())
}

/// A chord known only by pitch (C4 and C♯4, both nearest on the B string)
/// is placed one note per string, so the GP5 file reads back whole.
#[test]
fn a_pitch_only_chord_takes_one_string_per_note() -> TestResult {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::song::Song;
    use crate::model::optimized::note::{Pitch, PitchStep};
    use crate::tests::common::read_file;

    let mut song = Song::default();
    song.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    let mut score = legacy_song_to_loaded_score(&song);
    let beat = score.score.tracks[0]
        .measures
        .values_mut()
        .flat_map(|data| data.voices.values_mut())
        .flat_map(|voice| voice.beats.iter_mut())
        .find(|beat| beat.notes.iter().any(|note| note.string.is_some()))
        .ok_or("no beat with a note")?;
    let template = serde_json::to_value(&beat.notes[0])?;
    let note = |alter: i8| -> Result<_, serde_json::Error> {
        let mut note = template.clone();
        note["string"] = serde_json::Value::Null;
        note["fret"] = serde_json::Value::Null;
        note["tie"] = serde_json::Value::Null;
        note["gp_is_rest"] = serde_json::json!(false);
        note["pitch"] = serde_json::to_value(Pitch {
            step: PitchStep::C,
            alter,
            octave: 4,
        })?;
        serde_json::from_value(note)
    };
    beat.notes = vec![note(0)?, note(1)?];
    let exported = loaded_score_to_legacy_song(&score);
    let beat = exported.tracks[0]
        .measures
        .iter()
        .flat_map(|measure| &measure.voices)
        .flat_map(|voice| &voice.beats)
        .find(|beat| beat.notes.len() == 2)
        .ok_or("no two-note beat")?;
    assert_ne!(beat.notes[0].string, beat.notes[1].string);
    assert!(exported.write_losses((5, 1, 0)).is_empty());
    let mut read = Song::default();
    read.read_gp5(&exported.write((5, 1, 0), None)?)?;
    assert_eq!(
        read.tracks[0].measures.len(),
        exported.tracks[0].measures.len()
    );
    Ok(())
}

/// A sounding beat at the measure's start holding one note on `string` at
/// `fret`, `tie` marking it, from `template`'s shape.
fn beat_with(
    template: &serde_json::Value,
    string: u8,
    fret: u8,
    tie: Option<crate::model::optimized::note::TieType>,
) -> Result<crate::model::optimized::beat::Beat, serde_json::Error> {
    let mut beat = template.clone();
    let mut note = beat["notes"][0].clone();
    note["string"] = serde_json::json!(string);
    note["fret"] = serde_json::json!(fret);
    note["tie"] = serde_json::to_value(tie)?;
    note["gp_is_rest"] = serde_json::json!(false);
    beat["notes"] = serde_json::json!([note]);
    beat["tick_offset"] = serde_json::json!(0);
    beat["gp_empty"] = serde_json::json!(false);
    beat["gp_rest"] = serde_json::json!(false);
    serde_json::from_value(beat)
}

/// GP import ties pair by voice id, not by place in the measure: a voice
/// missing from one measure doesn't shift another onto its chain.
#[test]
fn ties_pair_by_voice_id_when_a_voice_is_missing() -> TestResult {
    use std::collections::HashMap;

    use crate::convert::optimized::legacy::{legacy_song_to_loaded_score, link_ties};
    use crate::model::legacy::song::Song;
    use crate::model::optimized::beat::Voice;
    use crate::model::optimized::global::MeasureIndex;
    use crate::model::optimized::note::TieType;
    use crate::tests::common::read_file;

    let mut song = Song::default();
    song.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    let mut score = legacy_song_to_loaded_score(&song).score;
    let track = &mut score.tracks[0];
    let template = track
        .measures
        .values()
        .flat_map(|data| data.voices.values())
        .flat_map(|voice| &voice.beats)
        .find(|beat| beat.notes.iter().any(|note| note.string.is_some()))
        .map(serde_json::to_value)
        .ok_or("no beat with a note")??;
    let mut measures = std::mem::take(&mut track.measures);
    measures.retain(|index, _| index.0 < 2);
    assert_eq!(measures.len(), 2);
    let voice = |voice_id, beat| Voice {
        voice_id,
        beats: vec![beat],
    };
    // Measure 1: voices 0 and 1 on string 3, at frets 5 and 7.
    let first = measures.get_mut(&MeasureIndex(0)).ok_or("measure 1")?;
    first.voices = HashMap::from([
        (0, voice(0, beat_with(&template, 3, 5, None)?)),
        (1, voice(1, beat_with(&template, 3, 7, None)?)),
    ]);
    // Measure 2: only voice 1, tied on string 3.
    let second = measures.get_mut(&MeasureIndex(1)).ok_or("measure 2")?;
    second.voices = HashMap::from([(1, voice(1, beat_with(&template, 3, 0, Some(TieType::End))?))]);

    link_ties(&mut measures, true);

    let note = |measure: u16, voice: u8| {
        &measures[&MeasureIndex(measure)].voices[&voice].beats[0].notes[0]
    };
    // Tied from voice 1's note, not from voice 0's (the first voice in
    // measure 2's place order).
    assert_eq!(note(1, 1).fret, Some(7));
    assert!(note(0, 1).tie.is_some_and(TieType::starts_tie));
    assert_eq!(note(0, 0).tie, None);
    Ok(())
}
