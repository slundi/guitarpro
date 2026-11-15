//! Tests: `musicxml::ScorePartwise` → `legacy::Song` via `convert::guitarpro`.
//!
//! Reads every MusicXML file in `test/`, converts it to a Guitar Pro `Song`,
//! then checks structural and note-level invariants.

use crate::{
    convert::{guitarpro::musicxml_to_legacy_song, musicxml::song_to_score_partwise},
    model::{
        legacy::enums::NoteType,
        musicxml::{ScorePartwise, part_list::PartListItem},
    },
};

// ---------------------------------------------------------------------------
// XML parsing helper (shared with roundtrip_musicxml.rs)
// ---------------------------------------------------------------------------

fn strip_doctype(xml: &str) -> String {
    if let Some(start) = xml.find("<!DOCTYPE")
        && let Some(rel_end) = xml[start..].find('>')
    {
        return format!("{}{}", &xml[..start], &xml[start + rel_end + 1..]);
    }
    xml.to_string()
}

fn parse_musicxml(data: &[u8]) -> Result<ScorePartwise, String> {
    let raw = std::str::from_utf8(data).map_err(|e| format!("UTF-8: {e}"))?;
    let cleaned = strip_doctype(raw);
    quick_xml::de::from_str(&cleaned).map_err(|e| format!("XML parse: {e}"))
}

// ---------------------------------------------------------------------------
// Helpers to count things in the source XML
// ---------------------------------------------------------------------------

/// Number of `<score-part>` entries in the part-list (drives track creation).
fn xml_part_count(src: &ScorePartwise) -> usize {
    src.part_list
        .items
        .iter()
        .filter(|i| matches!(i, PartListItem::ScorePart(_)))
        .count()
}

/// ID of the first listed score-part.
fn first_part_id(src: &ScorePartwise) -> Option<&str> {
    src.part_list.items.iter().find_map(|i| {
        if let PartListItem::ScorePart(sp) = i {
            Some(sp.id.as_str())
        } else {
            None
        }
    })
}

/// Number of measures in the first listed part.
fn xml_measure_count(src: &ScorePartwise) -> usize {
    first_part_id(src)
        .and_then(|id| src.parts.iter().find(|p| p.id == id))
        .map(|p| p.measures.len())
        .unwrap_or(0)
}

/// Count notes (non-rest, non-chord-continuation) in the first listed part.
fn xml_real_note_count(src: &ScorePartwise) -> usize {
    let Some(part_id) = first_part_id(src) else {
        return 0;
    };
    let Some(part) = src.parts.iter().find(|p| p.id == part_id) else {
        return 0;
    };
    let mut count = 0;
    for measure in &part.measures {
        for event in &measure.music_data {
            use crate::model::musicxml::measure::MusicData;
            if let MusicData::Note(n) = event
                && n.rest.is_none()
                && n.chord.is_none()
            {
                count += 1;
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Invariant checker
// ---------------------------------------------------------------------------

fn check_invariants(
    src: &ScorePartwise,
    song: &crate::model::legacy::song::Song,
) -> Result<(), String> {
    let num_parts = xml_part_count(src);
    let num_measures = xml_measure_count(src);

    // Track count matches part-list size.
    if song.tracks.len() != num_parts {
        return Err(format!(
            "tracks: got {}, expected {num_parts}",
            song.tracks.len()
        ));
    }

    // Measure-header count matches the first part's measure count.
    if song.measure_headers.len() != num_measures {
        return Err(format!(
            "measure_headers: got {}, expected {num_measures}",
            song.measure_headers.len()
        ));
    }

    // Every track has exactly as many measures as the global header list.
    for (ti, track) in song.tracks.iter().enumerate() {
        if track.measures.len() != num_measures {
            return Err(format!(
                "track {ti}: {} measures, expected {num_measures}",
                track.measures.len()
            ));
        }
    }

    // Tempo must be positive.
    if song.tempo <= 0 {
        return Err(format!("tempo is {}", song.tempo));
    }

    // If the source has real notes, at least some beats in track 0 must be
    // non-rest (pitch→tab conversion should not turn everything into rests).
    let xml_notes = xml_real_note_count(src);
    if xml_notes > 0 && !song.tracks.is_empty() {
        let normal_notes: usize = song.tracks[0]
            .measures
            .iter()
            .flat_map(|m| &m.voices)
            .flat_map(|v| &v.beats)
            .flat_map(|b| &b.notes)
            .filter(|n| matches!(n.kind, NoteType::Normal | NoteType::Tie))
            .count();
        if normal_notes == 0 {
            return Err(format!(
                "track 0 has 0 normal notes but source has {xml_notes} pitched notes"
            ));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[test]
fn test_musicxml_to_legacy_song() {
    use std::fs;

    let test_dir = "../test";
    let mut pass = 0usize;
    let mut skip = 0usize;
    let mut failures: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(test_dir)
        .expect("test/ directory not found")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "xml"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let fname = path.file_name().unwrap().to_str().unwrap().to_string();
        let data = fs::read(&path).unwrap();

        let src = match parse_musicxml(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("SKIP: {fname}: {e}");
                skip += 1;
                continue;
            }
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            musicxml_to_legacy_song(&src)
        }));

        let song = match result {
            Ok(s) => s,
            Err(_) => {
                failures.push(format!("{fname}: panicked during conversion"));
                continue;
            }
        };

        // Check Song structural invariants
        if let Err(msg) = check_invariants(&src, &song) {
            failures.push(format!("{fname}: {msg}"));
            continue;
        }

        // Roundtrip: Song → ScorePartwise, check part/measure counts are preserved
        let roundtripped = song_to_score_partwise(&song);
        let rt_parts = roundtripped
            .part_list
            .items
            .iter()
            .filter(|i| matches!(i, PartListItem::ScorePart(_)))
            .count();
        let rt_measures = roundtripped
            .parts
            .first()
            .map(|p| p.measures.len())
            .unwrap_or(0);
        let expected_parts = xml_part_count(&src);
        let expected_measures = xml_measure_count(&src);
        if rt_parts != expected_parts {
            failures.push(format!(
                "{fname}: roundtrip parts: got {rt_parts}, expected {expected_parts}"
            ));
            continue;
        }
        if rt_measures != expected_measures {
            failures.push(format!(
                "{fname}: roundtrip measures: got {rt_measures}, expected {expected_measures}"
            ));
            continue;
        }
        pass += 1;
    }

    let total = pass + failures.len();
    eprintln!(
        "MusicXML → legacy Song: {pass} pass, {} fail, {skip} skipped out of {} xml files",
        failures.len(),
        total + skip
    );
    for f in &failures {
        eprintln!("FAIL: {f}");
    }

    assert!(
        failures.is_empty(),
        "{} MusicXML file(s) failed conversion",
        failures.len()
    );
}

/// A MusicXML score of `measures` (each a measure's notes) for one part.
fn one_part_score(measures: &[&str]) -> String {
    let body: String = measures
        .iter()
        .enumerate()
        .map(|(i, notes)| {
            let attributes = if i == 0 {
                "<attributes><divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>"
            } else {
                ""
            };
            format!("<measure number=\"{}\">{attributes}{notes}</measure>", i + 1)
        })
        .collect();
    format!(
        r#"<score-partwise version="3.1"><part-list><score-part id="P1"><part-name>Guitar</part-name></score-part></part-list><part id="P1">{body}</part></score-partwise>"#
    )
}

fn whole_note(step: &str, octave: u8, chord: bool, tie: Option<&str>) -> String {
    let chord = if chord { "<chord/>" } else { "" };
    let tie = tie.map_or(String::new(), |kind| format!("<tie type=\"{kind}\"/>"));
    format!(
        "<note>{chord}<pitch><step>{step}</step><octave>{octave}</octave></pitch><duration>4</duration>{tie}<voice>1</voice><type>whole</type></note>"
    )
}

/// A chord note no free string reaches (E2 and F2: both only on the low
/// E string) is left out and reported, not doubled on one string.
#[test]
fn musicxml_chord_notes_without_a_free_string_are_reported() -> Result<(), String> {
    use crate::convert::guitarpro::musicxml_to_legacy_song_with_losses;

    let chord = whole_note("E", 2, false, None) + &whole_note("F", 2, true, None);
    let src = parse_musicxml(one_part_score(&[&chord]).as_bytes())?;
    let (song, losses) = musicxml_to_legacy_song_with_losses(&src);
    assert_eq!(losses.get("Note on no free string of its track"), 1);
    assert_eq!(song.tracks[0].measures[0].voices[0].beats[0].notes.len(), 1);
    Ok(())
}

/// A note held on while one is added below it keeps its string.
#[test]
fn musicxml_held_notes_keep_their_string() -> Result<(), String> {
    use crate::convert::guitarpro::musicxml_to_legacy_song;
    use crate::model::legacy::enums::NoteType;

    let held = whole_note("C", 4, false, Some("start"));
    let added = whole_note("B", 3, false, None) + &whole_note("C", 4, true, Some("stop"));
    let src = parse_musicxml(one_part_score(&[&held, &added]).as_bytes())?;
    let song = musicxml_to_legacy_song(&src);
    let first = &song.tracks[0].measures[0].voices[0].beats[0].notes[0];
    let second = &song.tracks[0].measures[1].voices[0].beats[0].notes;
    let tied = second
        .iter()
        .find(|note| note.kind == NoteType::Tie)
        .ok_or("no tied note")?;
    assert_eq!((tied.string, tied.value), (first.string, first.value));
    assert_ne!(second[0].string, second[1].string);
    Ok(())
}

/// A unison, one note tied on: the tie continues on that note's string,
/// not the untied one's.
#[test]
fn musicxml_ties_from_a_unison_keep_the_tied_string() -> Result<(), String> {
    use crate::convert::guitarpro::musicxml_to_legacy_song;
    use crate::model::legacy::enums::NoteType;

    let unison = whole_note("E", 4, false, Some("start")) + &whole_note("E", 4, true, None);
    let tied = whole_note("E", 4, false, Some("stop"));
    let src = parse_musicxml(one_part_score(&[&unison, &tied]).as_bytes())?;
    let song = musicxml_to_legacy_song(&src);
    let first = &song.tracks[0].measures[0].voices[0].beats[0].notes;
    let second = &song.tracks[0].measures[1].voices[0].beats[0].notes[0];
    assert_ne!(first[0].string, first[1].string);
    assert_eq!(second.kind, NoteType::Tie);
    assert_eq!(second.string, first[0].string);
    Ok(())
}

/// A clef at the first measure's start, written after a <backup> back to
/// it (between voices), is the staff's opening clef: not lost.
#[test]
fn musicxml_opening_clef_after_a_backup_holds() -> Result<(), String> {
    use crate::convert::optimized::score_partwise_to_loaded_score;
    use crate::model::optimized::track::Clef;

    let notes = whole_note("C", 4, false, None)
        + "<backup><duration>4</duration></backup>"
        + "<attributes><clef><sign>F</sign><line>4</line></clef></attributes>"
        + &whole_note("C", 3, false, None).replace("<voice>1</voice>", "<voice>2</voice>");
    let src = parse_musicxml(one_part_score(&[&notes]).as_bytes())?;
    let score = score_partwise_to_loaded_score(&src).score;
    let staff = score.tracks[0].staves.first().ok_or("no staff")?;
    let clef = score
        .staves
        .get(staff.0 as usize)
        .ok_or("no staff def")?
        .clef;
    assert_eq!(clef, Clef::Bass);
    Ok(())
}

/// A tie start is used up by its tie end: a later tie stop without a start
/// of its own doesn't claim that string, so the chord's other notes keep
/// their best ones.
#[test]
fn musicxml_ended_ties_free_their_string() -> Result<(), String> {
    use crate::convert::guitarpro::musicxml_to_legacy_song;

    let start = whole_note("C", 4, false, Some("start"));
    let stop = whole_note("C", 4, false, Some("stop"));
    // B3 (nearest the B string, open) with a C4 whose stop has no start.
    let chord = whole_note("B", 3, false, None) + &whole_note("C", 4, true, Some("stop"));
    let src = parse_musicxml(one_part_score(&[&start, &stop, &chord]).as_bytes())?;
    let song = musicxml_to_legacy_song(&src);
    let held = &song.tracks[0].measures[0].voices[0].beats[0].notes[0];
    let b3 = &song.tracks[0].measures[2].voices[0].beats[0].notes[0];
    assert_eq!((b3.string, b3.value), (held.string, 0));
    Ok(())
}
