//! Roundtrip tests: `musicxml::ScorePartwise` → `legacy::Song` → `musicxml::ScorePartwise`.
//!
//! Roadmap `Conversion`: this roundtrip *may be lossy* — MusicXML carries
//! notation detail that the Guitar Pro model does not represent, so exact
//! equality is not expected. Instead the tests assert the properties that must
//! survive a lossy roundtrip:
//!
//! 1. **Structural preservation** — the number of parts (part-list) and the
//!    measure count of the first part are unchanged by one roundtrip.
//! 2. **Fixed point / idempotency** — once the document has passed through the
//!    Guitar Pro model once, a *second* roundtrip changes nothing. This is the
//!    defining property of a well-behaved lossy conversion: the loss happens on
//!    the first pass, after which the representation is stable.
//!
//! The roundtrip uses **only** the MusicXML↔legacy converters
//! ([`musicxml_to_legacy_song`] and [`song_to_score_partwise`]); it never passes
//! through the `optimized` model or any other intermediate representation.

use crate::{
    convert::{guitarpro::musicxml_to_legacy_song, musicxml::song_to_score_partwise},
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

/// One `musicxml → legacy → musicxml` roundtrip, using only the two converters.
fn roundtrip(src: &ScorePartwise) -> ScorePartwise {
    song_to_score_partwise(&musicxml_to_legacy_song(src))
}

/// Number of `<score-part>` entries in the part-list (drives track creation).
fn part_count(src: &ScorePartwise) -> usize {
    src.part_list
        .items
        .iter()
        .filter(|item| matches!(item, PartListItem::ScorePart(_)))
        .count()
}

/// Number of measures in the first `<part>` of the document.
fn first_part_measure_count(src: &ScorePartwise) -> usize {
    src.parts
        .first()
        .map(|part| part.measures.len())
        .unwrap_or(0)
}

#[test]
fn test_musicxml_legacy_musicxml_roundtrip_for_all_files() {
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
            // First roundtrip: the (possibly lossy) pass through the GP model.
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
        "musicxml → legacy → musicxml: {pass} pass, {} fail, {skip} unparsable",
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
        <divisions>1</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
      </note>
    </measure>
  </part>
  <part id="P2">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>G</step><octave>3</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
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
    // A second roundtrip must be a no-op relative to the first.
    let src = parse_partwise(SIMPLE_PARTWISE.as_bytes()).expect("parse");
    let rt1 = roundtrip(&src);
    let rt2 = roundtrip(&rt1);

    assert_eq!(
        format!("{rt1:?}"),
        format!("{rt2:?}"),
        "roundtrip must reach a fixed point after the first pass"
    );
}
