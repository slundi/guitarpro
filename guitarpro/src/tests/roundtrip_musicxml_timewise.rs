//! Tests for `musicxml::ScoreTimewise` → `optimized::LoadedScore`.
//!
//! The converter under test ([`score_timewise_to_loaded_score`]) transposes a
//! `score-timewise` document into the equivalent `score-partwise` document
//! *within the `musicxml` model* — it never routes through the legacy `Song`
//! model — and reuses the partwise converter. These tests verify that:
//!
//! 1. **Behavioral equivalence** — for every MusicXML file in `test/`, the
//!    timewise path yields a byte-for-byte identical `LoadedScore` to the
//!    partwise path. Each partwise document is transposed into an equivalent
//!    timewise document (the inverse of the production transpose), and both are
//!    converted and compared.
//! 2. **Structural invariants** hold for hand-written timewise fixtures
//!    (single part, multiple parts, empty parts).

use std::collections::BTreeMap;

use crate::{
    convert::optimized::{score_partwise_to_loaded_score, score_timewise_to_loaded_score},
    model::{
        musicxml::{
            ScorePartwise, ScoreTimewise, TimewiseMeasure, TimewisePart, measure::Measure,
            part_list::PartListItem,
        },
        optimized::{beat::Voice, global::Score},
    },
};

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

/// A deterministic, comparable string form of a [`Score`].
///
/// Every field of `Score` has a stable `Debug` representation *except* each
/// measure's `voices`, which is a `HashMap` (per-instance random seed → random
/// iteration order). Since voice *order* carries no meaning (voices are keyed by
/// id), we sort them by id before rendering. Two conversions of the same music
/// then produce byte-identical strings.
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
        // `measures` is a BTreeMap: already iterated in sorted key order.
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

/// Strip `<!DOCTYPE ...>` declarations so quick_xml's serde driver can parse the
/// file. Mirrors the helper in `roundtrip_musicxml`.
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

/// Transpose an owned [`ScorePartwise`] into the equivalent [`ScoreTimewise`].
///
/// This is the inverse of the production `timewise_to_partwise`. It consumes the
/// partwise document, regrouping its per-part measure sequences into per-measure
/// part fragments. Measures are aligned by index across parts; measure-level
/// attributes (number, width, …) are taken from the first part that has a
/// measure at that index. Part order within each timewise measure follows the
/// document order of `<part>` elements.
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
        // entries by index, whereas the timewise transpose regroups strictly in
        // part-list order. When a file's `<part>` order differs from its
        // part-list order (rare, non-standard), the two paths legitimately
        // diverge; skip those to keep the equivalence assertion meaningful.
        let list_ids = part_list_ids(&src);
        let part_ids: Vec<&str> = src.parts.iter().map(|part| part.id.as_str()).collect();
        if list_ids != part_ids {
            skip_reorder += 1;
            continue;
        }

        let partwise_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            canonical_score(&score_partwise_to_loaded_score(&src).score)
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
            canonical_score(&score_timewise_to_loaded_score(timewise).score)
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
            failures.push(format!(
                "{fname}: timewise LoadedScore differs from partwise"
            ));
        }
    }

    eprintln!(
        "timewise ≡ partwise: {pass} pass, {} fail, {skip} unparsable, {skip_reorder} reordered-parts skipped",
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

fn assert_contiguous_timeline(loaded: &crate::model::optimized::LoadedScore) {
    for (index, measure_def) in loaded.score.timeline.iter().enumerate() {
        assert_eq!(
            measure_def.index.0 as usize, index,
            "timeline index must be contiguous from 0"
        );
    }
}

#[test]
fn test_timewise_single_part() {
    let src = parse_timewise(TIMEWISE_SINGLE_PART).expect("parse timewise");
    let loaded = score_timewise_to_loaded_score(src);

    assert_eq!(loaded.score.tracks.len(), 1, "one track for one part");
    assert_eq!(loaded.score.instruments.len(), 1, "one instrument");
    assert_eq!(
        loaded.score.timeline.len(),
        2,
        "two measures on the timeline"
    );
    assert_eq!(
        loaded.score.tracks[0].name, "Guitar",
        "track name from part-name"
    );
    // Both measures must be present in the track's measure map.
    assert_eq!(loaded.score.tracks[0].measures.len(), 2);
    assert_contiguous_timeline(&loaded);
}

#[test]
fn test_timewise_multi_part() {
    let src = parse_timewise(TIMEWISE_MULTI_PART).expect("parse timewise");
    let loaded = score_timewise_to_loaded_score(src);

    assert_eq!(loaded.score.tracks.len(), 2, "two tracks for two parts");
    assert_eq!(loaded.score.instruments.len(), 2, "two instruments");
    assert_eq!(
        loaded.score.timeline.len(),
        1,
        "single measure on the timeline"
    );
    assert_eq!(loaded.score.tracks[0].name, "Lead");
    assert_eq!(loaded.score.tracks[1].name, "Rhythm");
    // Each part contributes its single measure to its own track.
    assert_eq!(loaded.score.tracks[0].measures.len(), 1);
    assert_eq!(loaded.score.tracks[1].measures.len(), 1);
    assert_contiguous_timeline(&loaded);
}

#[test]
fn test_timewise_matches_equivalent_partwise() {
    // Same music expressed both ways must convert identically.
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

    let from_partwise = score_partwise_to_loaded_score(&partwise);
    let from_timewise = score_timewise_to_loaded_score(timewise);

    assert_eq!(
        canonical_score(&from_partwise.score),
        canonical_score(&from_timewise.score),
        "timewise and equivalent partwise must produce identical LoadedScores"
    );
}

#[test]
fn test_timewise_part_without_measures() {
    let src = parse_timewise(TIMEWISE_EMPTY_PART).expect("parse timewise");
    let loaded = score_timewise_to_loaded_score(src);

    // A part declared in the part-list but absent from every measure still
    // yields a track — just with no measure data.
    assert_eq!(
        loaded.score.tracks.len(),
        2,
        "both listed parts become tracks"
    );
    assert_eq!(loaded.score.timeline.len(), 1);
    assert_eq!(
        loaded.score.tracks[0].measures.len(),
        1,
        "P1 has one measure"
    );
    assert_eq!(
        loaded.score.tracks[1].measures.len(),
        0,
        "P2 has no measure data"
    );
    assert_contiguous_timeline(&loaded);
}
