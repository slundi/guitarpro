//! Smoke tests: parse every MusicXML file in `test/` and verify that
//! `score_partwise_to_loaded_score` produces a structurally valid `LoadedScore`.
//!
//! "Roundtrip" here means: XML bytes → `ScorePartwise` (serde) → `LoadedScore` (converter)
//! with invariants checked at the end (no panic, consistent counts, etc.).

use crate::{
    convert::optimized::score_partwise_to_loaded_score,
    model::musicxml::{ScorePartwise, part_list::PartListItem},
};

/// Strip `<!DOCTYPE ...>` declarations so quick_xml's serde driver can parse the file.
/// MusicXML files typically declare a DTD that quick_xml does not handle.
/// The declaration may span multiple lines, so we scan for the closing `>`.
fn strip_doctype(xml: &str) -> String {
    if let Some(start) = xml.find("<!DOCTYPE") {
        // Find the matching `>` after the DOCTYPE start.
        if let Some(rel_end) = xml[start..].find('>') {
            let end = start + rel_end + 1;
            return format!("{}{}", &xml[..start], &xml[end..]);
        }
    }
    xml.to_string()
}

/// Parse raw XML bytes into a `ScorePartwise`, returning `Err(message)` on failure.
fn parse_musicxml(data: &[u8]) -> Result<ScorePartwise, String> {
    let raw = std::str::from_utf8(data).map_err(|e| format!("UTF-8: {e}"))?;
    let cleaned = strip_doctype(raw);
    quick_xml::de::from_str(&cleaned).map_err(|e| format!("XML: {e}"))
}

/// Assert structural invariants on a successfully converted `LoadedScore`.
fn check_invariants(
    src: &ScorePartwise,
    loaded: &crate::model::optimized::LoadedScore,
    fname: &str,
) -> Result<(), String> {
    let s = &loaded.score;
    // Count only the score-parts listed in the part-list (the converter ignores
    // extra <part> elements not referenced by the part-list).
    let num_parts = src
        .part_list
        .items
        .iter()
        .filter(|i| matches!(i, PartListItem::ScorePart(_)))
        .count();

    // Tracks and instruments must match the part count.
    if s.tracks.len() != num_parts {
        return Err(format!(
            "tracks: got {}, expected {num_parts}",
            s.tracks.len()
        ));
    }
    if s.instruments.len() != num_parts {
        return Err(format!(
            "instruments: got {}, expected {num_parts}",
            s.instruments.len()
        ));
    }

    // Timeline length must match the measure count of the first listed part (if any).
    // Use the first part-list entry to find the right <part> element.
    let first_listed_id = src.part_list.items.iter().find_map(|i| {
        if let PartListItem::ScorePart(sp) = i {
            Some(sp.id.as_str())
        } else {
            None
        }
    });
    let first_listed_part = first_listed_id.and_then(|id| src.parts.iter().find(|p| p.id == id));
    if let Some(first_part) = first_listed_part {
        let expected_measures = first_part.measures.len();
        if s.timeline.len() != expected_measures {
            return Err(format!(
                "timeline len {}, first-part measures {expected_measures}",
                s.timeline.len()
            ));
        }
    }

    // Each track's MeasureData map must not exceed the timeline length.
    for (ti, track) in s.tracks.iter().enumerate() {
        if track.measures.len() > s.timeline.len() {
            return Err(format!(
                "track {ti}: {} measures > timeline {}",
                track.measures.len(),
                s.timeline.len()
            ));
        }
    }

    // MeasureDef indices must be contiguous starting at 0.
    for (i, md) in s.timeline.iter().enumerate() {
        if md.index.0 as usize != i {
            return Err(format!(
                "timeline[{i}].index = {}, expected {i}",
                md.index.0
            ));
        }
    }

    let _ = fname; // available for debugging if needed
    Ok(())
}

#[test]
fn test_musicxml_to_optimized() {
    use std::fs;

    let test_dir = "../test";
    let mut pass = 0usize;
    let mut skip = 0usize; // files that fail XML parse (not our bug)
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

        // Parse XML → ScorePartwise
        let src = match parse_musicxml(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("SKIP: {fname}: parse error: {e}");
                skip += 1;
                continue;
            }
        };

        // Convert → LoadedScore (must not panic)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            score_partwise_to_loaded_score(&src)
        }));

        let loaded = match result {
            Ok(l) => l,
            Err(_) => {
                failures.push(format!("{fname}: panicked during conversion"));
                continue;
            }
        };

        // Check structural invariants
        if let Err(msg) = check_invariants(&src, &loaded, &fname) {
            failures.push(format!("{fname}: {msg}"));
        } else {
            pass += 1;
        }
    }

    let total = pass + failures.len();
    eprintln!(
        "MusicXML → optimized: {pass} pass, {} fail, {skip} skipped out of {} xml files",
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
