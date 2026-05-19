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
    if let Some(start) = xml.find("<!DOCTYPE") {
        if let Some(rel_end) = xml[start..].find('>') {
            return format!("{}{}", &xml[..start], &xml[start + rel_end + 1..]);
        }
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
            if let MusicData::Note(n) = event {
                if n.rest.is_none() && n.chord.is_none() {
                    count += 1;
                }
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
