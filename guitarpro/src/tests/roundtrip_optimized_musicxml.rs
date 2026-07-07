//! Roundtrip tests: `optimized::LoadedScore` → `musicxml::ScorePartwise` → `optimized::LoadedScore`.
//!
//! Roadmap `Conversion`. This roundtrip is lossy — the MusicXML export
//! ([`loaded_score_to_score_partwise`]) re-emits only the core musical content
//! the two models share (parts, measures, voices, notes, signatures, tempo), not
//! GP-specific effects/techniques — so exact equality with the source is not
//! expected. The tests assert the properties that must hold:
//!
//! 1. **Structural preservation** — the track count and the global timeline
//!    length are unchanged by one roundtrip.
//! 2. **Fixed point / idempotency** — a *second* roundtrip changes nothing. The
//!    loss happens on the first pass, after which the representation is stable.
//!
//! The roundtrip uses **only** the optimized↔MusicXML converters
//! ([`loaded_score_to_score_partwise`] and [`score_partwise_to_loaded_score`]);
//! it never passes through the legacy `Song` model or any other intermediate.
//!
//! Source optimized scores are obtained from the MusicXML corpus (`test/*.xml`
//! parsed to `ScorePartwise`, then to `LoadedScore` via
//! `score_partwise_to_loaded_score`) — that acquisition is not part of the
//! roundtrip under test, which is strictly `LoadedScore → ScorePartwise →
//! LoadedScore`, and it too avoids the legacy model entirely.

use std::collections::BTreeMap;

use crate::{
    convert::{
        musicxml::loaded_score_to_score_partwise, optimized::score_partwise_to_loaded_score,
    },
    model::{
        musicxml::ScorePartwise,
        optimized::{LoadedScore, beat::Voice, global::Score},
    },
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

/// A deterministic, comparable string form of a [`Score`] (voices sorted by id,
/// since `MeasureData::voices` is a `HashMap` with unstable iteration order).
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

/// One `LoadedScore → ScorePartwise → LoadedScore` roundtrip.
fn roundtrip(source: &LoadedScore) -> LoadedScore {
    score_partwise_to_loaded_score(&loaded_score_to_score_partwise(source))
}

/// Assert the `optimized → musicxml → optimized` roundtrip is well-behaved:
///
/// * **Structural preservation** — track count and timeline length survive one
///   roundtrip.
/// * **Fixed point** — the first roundtrip normalizes the (lossy) model; a second
///   roundtrip must then change nothing.
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
    if rt1.score.timeline.len() != source.score.timeline.len() {
        return Err(format!(
            "timeline length changed: {} → {}",
            source.score.timeline.len(),
            rt1.score.timeline.len()
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

#[test]
fn test_optimized_musicxml_optimized_roundtrip_for_all_files() {
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
            // Source optimized score derived from the MusicXML document.
            let source = score_partwise_to_loaded_score(&src);
            assert_roundtrip(&source)
        }));

        match result {
            Ok(Ok(())) => pass += 1,
            Ok(Err(msg)) => failures.push(format!("{fname}: {}", &msg[..msg.len().min(400)])),
            Err(_) => failures.push(format!("{fname}: panicked during roundtrip")),
        }
    }

    eprintln!(
        "optimized → musicxml → optimized: {pass} pass, {} fail, {skip} unparsable",
        failures.len()
    );
    for failure in &failures {
        eprintln!("FAIL: {failure}");
    }

    assert!(
        failures.is_empty(),
        "{} file(s) failed the optimized→musicxml→optimized roundtrip",
        failures.len()
    );
    assert!(pass > 0, "no files were exercised");
}

// --- Focused inline fixtures ------------------------------------------------

const SIMPLE_PARTWISE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <movement-title>Roundtrip Sample</movement-title>
  <part-list>
    <score-part id="P1"><part-name>Guitar</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key><fifths>2</fifths><mode>major</mode></key>
        <time><beats>3</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><alter>1</alter><octave>4</octave></pitch>
        <duration>2</duration><voice>1</voice><type>half</type>
      </note>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>1</duration><voice>1</voice><type>quarter</type>
      </note>
    </measure>
    <measure number="2">
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>3</duration><voice>1</voice><type>half</type><dot/>
      </note>
    </measure>
  </part>
</score-partwise>"#;

fn source_from_xml(xml: &str) -> LoadedScore {
    score_partwise_to_loaded_score(&parse_partwise(xml.as_bytes()).expect("parse"))
}

#[test]
fn test_roundtrip_preserves_structure() {
    let source = source_from_xml(SIMPLE_PARTWISE);
    let rt = roundtrip(&source);

    assert_eq!(rt.score.tracks.len(), source.score.tracks.len(), "tracks");
    assert_eq!(
        rt.score.timeline.len(),
        source.score.timeline.len(),
        "timeline length"
    );
    assert_eq!(rt.score.timeline.len(), 2, "two measures");
}

#[test]
fn test_roundtrip_is_idempotent() {
    let source = source_from_xml(SIMPLE_PARTWISE);
    let rt1 = roundtrip(&source);
    let rt2 = roundtrip(&rt1);

    assert_eq!(
        canonical_score(&rt1.score),
        canonical_score(&rt2.score),
        "roundtrip must reach a fixed point after the first pass"
    );
}

#[test]
fn test_roundtrip_preserves_key_and_time_signature() {
    // A sharp key (D major, fifths=2) and 3/4 time must survive the roundtrip.
    let source = source_from_xml(SIMPLE_PARTWISE);
    let rt = roundtrip(&source);

    let ts = rt.score.timeline[0].time_signature.expect("time sig");
    assert_eq!((ts.numerator, ts.denominator), (3, 4), "3/4 preserved");

    let ks = rt.score.timeline[0].key_signature.expect("key sig");
    use crate::model::optimized::{metadata::Mode, note::PitchStep};
    assert_eq!(ks.mode, Mode::Major);
    // D major: root D.
    assert_eq!(ks.root.step, PitchStep::D);
}
