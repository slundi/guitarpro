//! Roundtrip tests: `musicxml::ScorePartwise` → `optimized::LoadedScore` → `musicxml::ScorePartwise`.
//!
//! Roadmap `Conversion`. This roundtrip is lossy — the compact `optimized` model
//! keeps the shared musical content (parts, measures, voices, notes, signatures)
//! but not every MusicXML notation detail — so exact equality is not expected.
//! The tests assert the properties that must survive:
//!
//! 1. **Structural preservation** — the number of parts and the measure count of
//!    the first part are unchanged by one roundtrip.
//! 2. **Fixed point / idempotency** — a *second* roundtrip changes nothing. The
//!    loss happens on the first pass, after which the representation is stable.
//!
//! The roundtrip uses **only** the MusicXML↔optimized converters
//! ([`score_partwise_to_loaded_score`] and [`loaded_score_to_score_partwise`]);
//! it never passes through the legacy `Song` model or any other intermediate.

use crate::{
    convert::{
        musicxml::loaded_score_to_score_partwise, optimized::score_partwise_to_loaded_score,
    },
    model::musicxml::{ScorePartwise, part_list::PartListItem},
};

fn strip_doctype(xml: &str) -> String {
    let Some(start) = xml.find("<!DOCTYPE") else {
        return xml.to_string();
    };
    let Some(rel_end) = xml[start..].find('>') else {
        return xml.to_string();
    };
    let end = start + rel_end + 1;
    format!("{}{}", &xml[..start], &xml[end..])
}

fn parse_partwise(data: &[u8]) -> Result<ScorePartwise, String> {
    let raw = std::str::from_utf8(data).map_err(|error| format!("UTF-8: {error}"))?;
    let cleaned = strip_doctype(raw);
    quick_xml::de::from_str(&cleaned).map_err(|error| format!("XML: {error}"))
}

/// One `musicxml → optimized → musicxml` roundtrip, using only the two converters.
fn roundtrip(src: &ScorePartwise) -> ScorePartwise {
    loaded_score_to_score_partwise(&score_partwise_to_loaded_score(src))
}

fn part_count(src: &ScorePartwise) -> usize {
    src.part_list
        .items
        .iter()
        .filter(|item| matches!(item, PartListItem::ScorePart(_)))
        .count()
}

fn first_part_measure_count(src: &ScorePartwise) -> usize {
    src.parts
        .first()
        .map(|part| part.measures.len())
        .unwrap_or(0)
}

#[test]
fn test_musicxml_optimized_musicxml_roundtrip_for_all_files() {
    use std::fs;

    let test_dir = "../test";
    let mut pass = 0usize;
    let mut skip = 0usize; // files that fail XML parse (not our bug)
    let mut failures: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(test_dir)
        .expect("test/ directory not found")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "xml"))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let fname = path.file_name().unwrap().to_str().unwrap().to_string();
        let data = fs::read(&path).unwrap();

        let src = match parse_partwise(&data) {
            Ok(score) => score,
            Err(error) => {
                eprintln!("SKIP: {fname}: parse error: {error}");
                skip += 1;
                continue;
            }
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // First roundtrip: the (lossy) pass through the optimized model.
            let rt1 = roundtrip(&src);

            // 1. Structural preservation.
            if part_count(&rt1) != part_count(&src) {
                return Err(format!(
                    "part count changed: {} → {}",
                    part_count(&src),
                    part_count(&rt1)
                ));
            }
            if first_part_measure_count(&rt1) != first_part_measure_count(&src) {
                return Err(format!(
                    "first-part measure count changed: {} → {}",
                    first_part_measure_count(&src),
                    first_part_measure_count(&rt1)
                ));
            }

            // 2. Fixed point: a second roundtrip must change nothing.
            let rt2 = roundtrip(&rt1);
            if format!("{rt1:?}") != format!("{rt2:?}") {
                return Err("roundtrip is not idempotent (rt1 != rt2)".to_string());
            }

            Ok(())
        }));

        match result {
            Ok(Ok(())) => pass += 1,
            Ok(Err(msg)) => failures.push(format!("{fname}: {msg}")),
            Err(_) => failures.push(format!("{fname}: panicked during roundtrip")),
        }
    }

    eprintln!(
        "musicxml → optimized → musicxml: {pass} pass, {} fail, {skip} unparsable",
        failures.len()
    );
    for failure in &failures {
        eprintln!("FAIL: {failure}");
    }

    assert!(
        failures.is_empty(),
        "{} MusicXML file(s) failed the roundtrip",
        failures.len()
    );
    assert!(pass > 0, "no files were exercised");
}

// --- Focused inline fixtures ------------------------------------------------

const SIMPLE_PARTWISE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <movement-title>Roundtrip Sample</movement-title>
  <part-list>
    <score-part id="P1">
      <part-name>Guitar</part-name>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key><fifths>0</fifths><mode>major</mode></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
      </note>
    </measure>
    <measure number="2">
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

const TWO_PART_PARTWISE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1"><part-name>Lead</part-name></score-part>
    <score-part id="P2"><part-name>Rhythm</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>2</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>4</duration><voice>1</voice><type>half</type>
      </note>
      <note>
        <pitch><step>F</step><octave>4</octave></pitch>
        <duration>4</duration><voice>1</voice><type>half</type>
      </note>
    </measure>
  </part>
  <part id="P2">
    <measure number="1">
      <attributes>
        <divisions>2</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>F</sign><line>4</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>3</octave></pitch>
        <duration>8</duration><voice>1</voice><type>whole</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

#[test]
fn test_roundtrip_preserves_structure_single_part() {
    let src = parse_partwise(SIMPLE_PARTWISE.as_bytes()).expect("parse");
    let rt = roundtrip(&src);

    assert_eq!(part_count(&rt), 1, "one part preserved");
    assert_eq!(first_part_measure_count(&rt), 2, "two measures preserved");
}

#[test]
fn test_roundtrip_preserves_structure_two_parts() {
    let src = parse_partwise(TWO_PART_PARTWISE.as_bytes()).expect("parse");
    let rt = roundtrip(&src);

    assert_eq!(part_count(&rt), 2, "two parts preserved");
    assert_eq!(first_part_measure_count(&rt), 1, "measure count preserved");
}

#[test]
fn test_roundtrip_is_idempotent() {
    let src = parse_partwise(SIMPLE_PARTWISE.as_bytes()).expect("parse");
    let rt1 = roundtrip(&src);
    let rt2 = roundtrip(&rt1);

    assert_eq!(
        format!("{rt1:?}"),
        format!("{rt2:?}"),
        "roundtrip must reach a fixed point after the first pass"
    );
}

#[test]
fn test_roundtrip_preserves_pitches_single_part() {
    // The first note's pitch (C4) must survive the roundtrip.
    let src = parse_partwise(SIMPLE_PARTWISE.as_bytes()).expect("parse");
    let rt = roundtrip(&src);

    use crate::model::musicxml::measure::MusicData;
    let first_note_step = rt.parts[0].measures[0]
        .music_data
        .iter()
        .find_map(|event| match event {
            MusicData::Note(note) => note.pitch.as_ref().map(|pitch| pitch.step.clone()),
            _ => None,
        });
    assert_eq!(first_note_step.as_deref(), Some("C"), "C4 preserved");
}

type SlurMarks = Vec<Vec<Option<(u8, crate::model::optimized::beat::SlurKind)>>>;

/// The slur marks of a MusicXML document's first part, per measure.
fn musicxml_slur_marks(
    document: &crate::model::musicxml::ScorePartwise,
) -> crate::tests::common::TestResult<SlurMarks> {
    let loaded = crate::convert::optimized::score_partwise_to_loaded_score(document);
    let track = loaded.score.tracks.first().ok_or("no track")?;
    Ok(track
        .measures
        .values()
        .map(|data| {
            let mut voices: Vec<_> = data.voices.values().collect();
            voices.sort_by_key(|voice| voice.voice_id);
            voices
                .into_iter()
                .flat_map(|voice| &voice.beats)
                .map(|beat| beat.slur.map(|slur| (slur.slur_id, slur.kind)))
                .collect()
        })
        .collect())
}

/// MusicXML slurs pair by `number`: a stop and a start on one note make
/// an `EndStart`, nested slurs keep their own ids, and slurs on a chord's
/// later notes count for the chord.
#[test]
fn musicxml_slurs_pair_by_number() -> crate::tests::common::TestResult {
    use crate::model::optimized::beat::SlurKind::{End, EndStart, Start};
    use crate::tests::common::read_musicxml;

    assert_eq!(
        musicxml_slur_marks(&read_musicxml("33c-Spanners-Slurs.xml")?)?,
        vec![
            vec![
                Some((0, Start)),
                Some((0, EndStart)),
                Some((0, EndStart)),
                Some((0, End))
            ],
            vec![
                Some((0, Start)),
                Some((1, Start)),
                Some((1, End)),
                Some((0, End))
            ],
        ]
    );
    assert_eq!(
        musicxml_slur_marks(&read_musicxml("33g-Slur-ChordedNotes.xml")?)?,
        vec![vec![
            Some((0, Start)),
            Some((1, Start)),
            Some((0, End)),
            Some((1, End))
        ]]
    );
    Ok(())
}

/// A MusicXML slur open at a barline ends in a later measure under its id.
#[test]
fn musicxml_slurs_cross_barlines() -> crate::tests::common::TestResult {
    use crate::model::optimized::beat::SlurKind::{End, EndStart, Start};

    // 33c with bar 1's last slur running on into bar 2 (to bar 2's outer
    // slur's stop): bar 1's last stop and bar 2's outer start removed.
    let raw = String::from_utf8(crate::tests::common::read_file(
        "test/33c-Spanners-Slurs.xml".into(),
    ))?;
    let without_third = |raw: &str, element: &str| -> Option<String> {
        let (at, _) = raw.match_indices(element).nth(2)?;
        Some(format!("{}{}", &raw[..at], &raw[at + element.len()..]))
    };
    let raw = without_third(&raw, r#"<slur number="1" type="stop"/>"#).ok_or("no stop")?;
    let raw = without_third(&raw, r#"<slur number="1" placement="above" type="start"/>"#)
        .ok_or("no start")?;
    let start = raw.find("<!DOCTYPE").ok_or("no doctype")?;
    let end = start + raw[start..].find('>').ok_or("no doctype end")?;
    let document = quick_xml::de::from_str(&format!("{}{}", &raw[..start], &raw[end + 1..]))?;
    assert_eq!(
        musicxml_slur_marks(&document)?,
        vec![
            vec![
                Some((0, Start)),
                Some((0, EndStart)),
                Some((0, EndStart)),
                None
            ],
            vec![None, Some((1, Start)), Some((1, End)), Some((0, End))],
        ]
    );
    Ok(())
}
