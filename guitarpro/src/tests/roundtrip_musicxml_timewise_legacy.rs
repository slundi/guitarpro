//! Tests for `musicxml::ScoreTimewise` → `legacy::Song` via `convert::guitarpro`.
//!
//! The converter under test ([`musicxml_timewise_to_legacy_song`]) transposes a
//! `score-timewise` document into the equivalent `score-partwise` document
//! *within the `musicxml` model* — it never routes through the `optimized`
//! model — and reuses the partwise converter. These tests verify that:
//!
//! 1. **Behavioral equivalence** — for every MusicXML file in `test/`, the
//!    timewise path yields a `Song` identical to the partwise path. Each partwise
//!    document is transposed into an equivalent timewise document (the inverse of
//!    the production transpose), and both are converted and compared. `Song`'s
//!    `Debug` representation is deterministic (no `HashMap` fields), so a direct
//!    string comparison is stable across runs.
//! 2. **Structural invariants** hold for hand-written timewise fixtures
//!    (single part, multiple parts, empty parts).

use crate::{
    convert::guitarpro::{musicxml_timewise_to_legacy_song, musicxml_to_legacy_song},
    model::musicxml::{
        ScorePartwise, ScoreTimewise, TimewiseMeasure, TimewisePart, measure::Measure,
        part_list::PartListItem,
    },
};

/// Strip `<!DOCTYPE ...>` declarations so quick_xml's serde driver can parse the
/// file. Mirrors the helper in the other MusicXML round-trip tests.
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

fn parse_timewise(xml: &str) -> Result<ScoreTimewise, String> {
    let cleaned = strip_doctype(xml);
    quick_xml::de::from_str(&cleaned).map_err(|error| format!("XML: {error}"))
}

/// The ordered part ids declared in a `<part-list>`.
fn part_list_ids(src: &ScorePartwise) -> Vec<&str> {
    src.part_list
        .items
        .iter()
        .filter_map(|item| match item {
            PartListItem::ScorePart(score_part) => Some(score_part.id.as_str()),
            _ => None,
        })
        .collect()
}

/// Measure-level attributes carried on a `<measure>` element:
/// `(number, implicit, non_controlling, width, text, id)`.
type MeasureAttrs = (
    String,
    Option<String>,
    Option<String>,
    Option<f64>,
    Option<String>,
    Option<String>,
);

/// Transpose an owned [`ScorePartwise`] into the equivalent [`ScoreTimewise`].
///
/// This is the inverse of the production `ScoreTimewise::into_partwise`. It
/// consumes the partwise document, regrouping its per-part measure sequences into
/// per-measure part fragments. Measures are aligned by index across parts;
/// measure-level attributes are taken from the first part that has a measure at
/// that index. Part order within each timewise measure follows the document order
/// of `<part>` elements.
fn partwise_to_timewise(src: ScorePartwise) -> ScoreTimewise {
    let ScorePartwise {
        version,
        work,
        movement_number,
        movement_title,
        identification,
        defaults,
        credits,
        part_list,
        parts,
    } = src;

    let max_len = parts
        .iter()
        .map(|part| part.measures.len())
        .max()
        .unwrap_or(0);

    // Consume each part into (id, measure-iterator) so we can pull one measure
    // per part per timewise measure while preserving order.
    let mut part_cursors: Vec<(String, std::vec::IntoIter<Measure>)> = parts
        .into_iter()
        .map(|part| (part.id, part.measures.into_iter()))
        .collect();

    let mut measures: Vec<TimewiseMeasure> = Vec::with_capacity(max_len);
    for measure_idx in 0..max_len {
        let mut tw_parts: Vec<TimewisePart> = Vec::with_capacity(part_cursors.len());
        let mut attrs: Option<MeasureAttrs> = None;

        for (part_id, cursor) in part_cursors.iter_mut() {
            if let Some(measure) = cursor.next() {
                if attrs.is_none() {
                    attrs = Some((
                        measure.number.clone(),
                        measure.implicit.clone(),
                        measure.non_controlling.clone(),
                        measure.width,
                        measure.text.clone(),
                        measure.id.clone(),
                    ));
                }
                tw_parts.push(TimewisePart {
                    id: part_id.clone(),
                    music_data: measure.music_data,
                });
            }
        }

        let (number, implicit, non_controlling, width, text, id) =
            attrs.unwrap_or_else(|| ((measure_idx + 1).to_string(), None, None, None, None, None));

        measures.push(TimewiseMeasure {
            number,
            implicit,
            non_controlling,
            width,
            text,
            id,
            parts: tw_parts,
        });
    }

    ScoreTimewise {
        version,
        work,
        movement_number,
        movement_title,
        identification,
        defaults,
        credits,
        part_list,
        measures,
    }
}

#[test]
fn test_timewise_matches_partwise_for_all_files() {
    use std::fs;

    let test_dir = "../test";
    let mut pass = 0usize;
    let mut skip = 0usize; // files that fail XML parse (not our bug)
    let mut skip_reorder = 0usize; // parts not in part-list order (see below)
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

        // The partwise converter aligns `<part>` elements with `<part-list>`
        // entries, whereas the timewise transpose regroups strictly in part-list
        // order. When a file's `<part>` order differs from its part-list order
        // (rare, non-standard), the two paths legitimately diverge; skip those to
        // keep the equivalence assertion meaningful.
        let list_ids = part_list_ids(&src);
        let part_ids: Vec<&str> = src.parts.iter().map(|part| part.id.as_str()).collect();
        if list_ids != part_ids {
            skip_reorder += 1;
            continue;
        }

        let partwise_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            format!("{:?}", musicxml_to_legacy_song(&src))
        }));
        let partwise_dbg = match partwise_result {
            Ok(dbg) => dbg,
            Err(_) => {
                failures.push(format!("{fname}: panicked during partwise conversion"));
                continue;
            }
        };

        // Transpose partwise → timewise (consuming the owned document) and
        // convert via the code under test.
        let timewise = partwise_to_timewise(src);
        let timewise_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            format!("{:?}", musicxml_timewise_to_legacy_song(timewise))
        }));
        let timewise_dbg = match timewise_result {
            Ok(dbg) => dbg,
            Err(_) => {
                failures.push(format!("{fname}: panicked during timewise conversion"));
                continue;
            }
        };

        if partwise_dbg == timewise_dbg {
            pass += 1;
        } else {
            failures.push(format!("{fname}: timewise Song differs from partwise"));
        }
    }

    eprintln!(
        "timewise ≡ partwise (legacy): {pass} pass, {} fail, {skip} unparsable, {skip_reorder} reordered-parts skipped",
        failures.len()
    );
    for failure in &failures {
        eprintln!("FAIL: {failure}");
    }

    assert!(
        failures.is_empty(),
        "{} MusicXML file(s) produced divergent timewise conversions",
        failures.len()
    );
    assert!(pass > 0, "no files were compared");
}

// --- Focused inline fixtures ------------------------------------------------

/// A minimal single-part `score-timewise` document with two measures.
const TIMEWISE_SINGLE_PART: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-timewise version="4.0">
  <movement-title>Timewise Sample</movement-title>
  <part-list>
    <score-part id="P1">
      <part-name>Guitar</part-name>
    </score-part>
  </part-list>
  <measure number="1">
    <part id="P1">
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
    </part>
  </measure>
  <measure number="2">
    <part id="P1">
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
      </note>
    </part>
  </measure>
</score-timewise>"#;

/// A two-part `score-timewise` document; each measure carries both parts.
const TIMEWISE_MULTI_PART: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-timewise version="4.0">
  <part-list>
    <score-part id="P1"><part-name>Lead</part-name></score-part>
    <score-part id="P2"><part-name>Rhythm</part-name></score-part>
  </part-list>
  <measure number="1">
    <part id="P1">
      <attributes>
        <divisions>1</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
      </note>
    </part>
    <part id="P2">
      <attributes>
        <divisions>1</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>G</step><octave>3</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
      </note>
    </part>
  </measure>
</score-timewise>"#;

/// A part-list entry (`P2`) that never appears in any measure.
const TIMEWISE_EMPTY_PART: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-timewise version="4.0">
  <part-list>
    <score-part id="P1"><part-name>Played</part-name></score-part>
    <score-part id="P2"><part-name>Silent</part-name></score-part>
  </part-list>
  <measure number="1">
    <part id="P1">
      <attributes>
        <divisions>1</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>A</step><octave>3</octave></pitch>
        <duration>4</duration><voice>1</voice><type>whole</type>
      </note>
    </part>
  </measure>
</score-timewise>"#;

#[test]
fn test_timewise_single_part() {
    let src = parse_timewise(TIMEWISE_SINGLE_PART).expect("parse timewise");
    let song = musicxml_timewise_to_legacy_song(src);

    assert_eq!(song.tracks.len(), 1, "one track for one part");
    assert_eq!(song.measure_headers.len(), 2, "two measure headers");
    assert_eq!(song.tracks[0].measures.len(), 2, "track has two measures");
    assert_eq!(song.name, "Timewise Sample", "title from movement-title");
    assert!(song.tempo > 0, "tempo must be positive");
}

#[test]
fn test_timewise_multi_part() {
    let src = parse_timewise(TIMEWISE_MULTI_PART).expect("parse timewise");
    let song = musicxml_timewise_to_legacy_song(src);

    assert_eq!(song.tracks.len(), 2, "two tracks for two parts");
    assert_eq!(song.measure_headers.len(), 1, "single measure header");
    // Both tracks span the whole song.
    assert_eq!(song.tracks[0].measures.len(), 1);
    assert_eq!(song.tracks[1].measures.len(), 1);
    assert!(song.tempo > 0);
}

#[test]
fn test_timewise_part_without_measures() {
    let src = parse_timewise(TIMEWISE_EMPTY_PART).expect("parse timewise");
    let song = musicxml_timewise_to_legacy_song(src);

    // Both listed parts become tracks; every track spans the global header list,
    // so the silent part's measures are present but empty of notes.
    assert_eq!(song.tracks.len(), 2, "both listed parts become tracks");
    assert_eq!(song.measure_headers.len(), 1);
    assert_eq!(song.tracks[0].measures.len(), 1);
    assert_eq!(song.tracks[1].measures.len(), 1);
}

#[test]
fn test_timewise_matches_equivalent_partwise() {
    // Same music expressed both ways must convert to identical Songs.
    let partwise_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <movement-title>Timewise Sample</movement-title>
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

    let partwise = parse_partwise(partwise_xml.as_bytes()).expect("parse partwise");
    let timewise = parse_timewise(TIMEWISE_SINGLE_PART).expect("parse timewise");

    let from_partwise = musicxml_to_legacy_song(&partwise);
    let from_timewise = musicxml_timewise_to_legacy_song(timewise);

    assert_eq!(
        format!("{from_partwise:?}"),
        format!("{from_timewise:?}"),
        "timewise and equivalent partwise must produce identical Songs"
    );
}
