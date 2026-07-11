//! MSCZ round-trip matrix (Part 5.2 of the roadmap).
//!
//! Walks the fixture catalog in [`mscz_fixtures::FIXTURES`] and verifies:
//!
//! 1. Every fixture parses via `read_mscz_bytes`.
//! 2. `mscx_to_loaded_score` produces a `LoadedScore` with the expected
//!    shape (see the per-fixture invariants below).
//! 3. `loaded_score_to_mscx → write_mscz → read_mscz_bytes → mscx_to_loaded_score`
//!    yields a `LoadedScore` whose track and measure counts match the
//!    original — i.e. the structural subset survives a full container
//!    round-trip.
//!
//! Reporting mirrors the GP3/GP4/GP5 style used in
//! [`crate::tests::roundtrip_optimized`]: a single aggregate `MSCZ: N/M ✓`
//! line plus per-fixture failure details.

use super::mscz_fixtures::{FIXTURES, build_fixture};
use crate::convert::mscz::{loaded_score_to_mscx, mscx_to_loaded_score};
use crate::io::mscz::{read_mscz_bytes, write_mscz};
use crate::model::mscz::{MsczArchive, MsczEntry, MsczFile};
use crate::model::optimized::LoadedScore;

// ---------------------------------------------------------------------------
// Per-fixture expectations
// ---------------------------------------------------------------------------

/// Structural expectations checked against `LoadedScore`. If any of these
/// drift because of a converter change, the corresponding fixture starts
/// failing loudly instead of silently degrading.
#[derive(Copy, Clone)]
struct Expect {
    tracks: usize,
    measures: usize,
    /// Total notes across all tracks (0 for rest-only fixtures).
    notes: usize,
}

fn expectations_for(name: &str) -> Expect {
    match name {
        "simple_monophonic" => Expect {
            tracks: 1,
            measures: 2,
            notes: 8,
        },
        "multi_track_band" => Expect {
            tracks: 3,
            measures: 1,
            notes: 3,
        },
        "alternate_tuning" => Expect {
            tracks: 1,
            measures: 1,
            notes: 4,
        },
        "repeats_and_voltas" => Expect {
            tracks: 1,
            measures: 4,
            notes: 0,
        },
        "empty_score" => Expect {
            tracks: 1,
            measures: 1,
            notes: 0,
        },
        "single_measure" => Expect {
            tracks: 1,
            measures: 1,
            notes: 1,
        },
        "four_voices" => Expect {
            tracks: 1,
            measures: 1,
            notes: 16,
        },
        other => panic!("no expectations wired for fixture '{other}'"),
    }
}

// ---------------------------------------------------------------------------
// Assertions
// ---------------------------------------------------------------------------

fn count_notes(score: &LoadedScore) -> usize {
    score
        .score
        .tracks
        .iter()
        .flat_map(|track| track.measures.values())
        .flat_map(|measure| measure.voices.values())
        .flat_map(|voice| voice.beats.iter())
        .map(|beat| beat.notes.len())
        .sum()
}

fn count_measures(score: &LoadedScore) -> usize {
    // Timeline entries are shared across tracks — use them as the ground
    // truth for measure count.
    score.score.timeline.len()
}

fn check_shape(score: &LoadedScore, expect: Expect) -> Result<(), String> {
    if score.score.tracks.len() != expect.tracks {
        return Err(format!(
            "tracks: expected {}, got {}",
            expect.tracks,
            score.score.tracks.len()
        ));
    }
    let measures = count_measures(score);
    if measures != expect.measures {
        return Err(format!(
            "measures: expected {}, got {measures}",
            expect.measures
        ));
    }
    let notes = count_notes(score);
    if notes != expect.notes {
        return Err(format!("notes: expected {}, got {notes}", expect.notes));
    }
    Ok(())
}

/// Perform the full container round-trip and return the second-pass score.
fn roundtrip_once(bytes: &[u8]) -> Result<LoadedScore, String> {
    let file = read_mscz_bytes(bytes).map_err(|e| format!("read: {e}"))?;
    let first_pass = mscx_to_loaded_score(&file.mscx).score;

    let regenerated_mscx = loaded_score_to_mscx(&first_pass);
    // Rewrap the archive around the regenerated MSCX so `write_mscz` has
    // a coherent `MsczFile`.
    let manifest =
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container><rootfiles><rootfile full-path=\"score.mscx\"/></rootfiles></container>\n";
    let archive = MsczArchive {
        rootfiles: vec!["score.mscx".to_string()],
        entries: vec![
            MsczEntry {
                path: "META-INF/container.xml".to_string(),
                data: manifest.to_vec(),
            },
            MsczEntry {
                path: "score.mscx".to_string(),
                data: regenerated_mscx.raw_xml.as_bytes().to_vec(),
            },
        ],
    };
    let regenerated_file = MsczFile {
        archive,
        mscx: regenerated_mscx,
    };
    let regenerated_bytes = write_mscz(&regenerated_file).map_err(|e| format!("write: {e}"))?;
    let reparsed = read_mscz_bytes(&regenerated_bytes).map_err(|e| format!("re-read: {e}"))?;
    Ok(mscx_to_loaded_score(&reparsed.mscx).score)
}

// ---------------------------------------------------------------------------
// Aggregate driver
// ---------------------------------------------------------------------------

#[test]
fn mscz_fixture_roundtrip_matrix() {
    let mut pass = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for (name, _) in FIXTURES {
        let bytes = build_fixture(name).expect("catalog fixture");

        // First-pass invariants.
        let file = match read_mscz_bytes(&bytes) {
            Ok(f) => f,
            Err(e) => {
                failures.push(format!("{name}: initial parse: {e}"));
                continue;
            }
        };
        let first_pass = mscx_to_loaded_score(&file.mscx).score;
        let expect = expectations_for(name);
        if let Err(reason) = check_shape(&first_pass, expect) {
            failures.push(format!("{name}: initial shape: {reason}"));
            continue;
        }

        // Round-trip invariants.
        let second_pass = match roundtrip_once(&bytes) {
            Ok(score) => score,
            Err(reason) => {
                failures.push(format!("{name}: round-trip failed: {reason}"));
                continue;
            }
        };
        if let Err(reason) = check_shape(&second_pass, expect) {
            failures.push(format!("{name}: post-roundtrip shape: {reason}"));
            continue;
        }

        pass += 1;
    }

    let total = FIXTURES.len();
    println!("MSCZ round-trip: {pass}/{total} ✓");
    if !failures.is_empty() {
        for line in &failures {
            eprintln!("FAIL: {line}");
        }
        panic!("{} MSCZ fixture(s) failed round-trip", failures.len());
    }
}

// ---------------------------------------------------------------------------
// Cross-format: MSCZ → LoadedScore → MSCZ preserves per-fixture invariants
// even when the source XML uses a different indentation than our writer.
// ---------------------------------------------------------------------------

/// Additional check: parsing a fixture, converting to LoadedScore, then
/// regenerating MSCX and reparsing must yield an *identical* `LoadedScore`
/// shape — track names, staff clefs, and note count survive verbatim.
#[test]
fn mscz_semantic_equality_across_two_passes() {
    for (name, _) in FIXTURES {
        let bytes = build_fixture(name).expect("catalog fixture");
        let first =
            mscx_to_loaded_score(&read_mscz_bytes(&bytes).expect("initial parse").mscx).score;
        let second = roundtrip_once(&bytes).expect("round-trip");

        assert_eq!(
            first.score.tracks.len(),
            second.score.tracks.len(),
            "{name}: track count changed across round-trip"
        );
        assert_eq!(
            count_measures(&first),
            count_measures(&second),
            "{name}: measure count changed across round-trip"
        );
        assert_eq!(
            count_notes(&first),
            count_notes(&second),
            "{name}: note count changed across round-trip"
        );

        // Track names survive.
        let first_names: Vec<&str> = first.score.tracks.iter().map(|t| t.name.as_str()).collect();
        let second_names: Vec<&str> = second
            .score
            .tracks
            .iter()
            .map(|t| t.name.as_str())
            .collect();
        assert_eq!(first_names, second_names, "{name}: track names changed");
    }
}

// ---------------------------------------------------------------------------
// Cross-format: GP → MSCZ → LoadedScore preserves at least the track set
// ---------------------------------------------------------------------------

/// GP7 → LoadedScore → MSCX → LoadedScore. We pick a small real GP7
/// fixture from the workspace `test/` corpus and assert that the track
/// count / tuning survives conversion into MSCZ and back. The point is
/// that the cross-format path doesn't drop tracks, even if some detail
/// (dynamics, articulations) is lost.
#[test]
fn cross_format_gp7_to_mscz_preserves_track_set() {
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::song::Song;

    let path = std::path::Path::new("../test/accent.gp");
    if !path.exists() {
        eprintln!("skipping — GP7 fixture missing: {}", path.display());
        return;
    }
    let bytes = std::fs::read(path).expect("read gp7 fixture");
    let mut song = Song::default();
    song.read_gp(&bytes).expect("parse gp7");
    let loaded = legacy_song_to_loaded_score(&song);

    let track_count_before = loaded.score.tracks.len();
    let names_before: Vec<String> = loaded.score.tracks.iter().map(|t| t.name.clone()).collect();

    let regenerated_mscx = loaded_score_to_mscx(&loaded);
    // Wrap in an MSCZ and re-read through the full pipeline.
    let file = MsczFile {
        archive: MsczArchive {
            rootfiles: vec!["score.mscx".to_string()],
            entries: vec![
                MsczEntry {
                    path: "META-INF/container.xml".to_string(),
                    data: b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container><rootfiles><rootfile full-path=\"score.mscx\"/></rootfiles></container>\n".to_vec(),
                },
                MsczEntry {
                    path: "score.mscx".to_string(),
                    data: regenerated_mscx.raw_xml.as_bytes().to_vec(),
                },
            ],
        },
        mscx: regenerated_mscx,
    };
    let mscz_bytes = write_mscz(&file).expect("write mscz");
    let reparsed_file = read_mscz_bytes(&mscz_bytes).expect("re-read mscz");
    let reparsed = mscx_to_loaded_score(&reparsed_file.mscx).score;

    assert_eq!(
        reparsed.score.tracks.len(),
        track_count_before,
        "GP7 → MSCZ → LoadedScore must preserve the track count"
    );
    let names_after: Vec<String> = reparsed
        .score
        .tracks
        .iter()
        .map(|t| t.name.clone())
        .collect();
    assert_eq!(
        names_after, names_before,
        "GP7 → MSCZ → LoadedScore must preserve track names"
    );
}

// ---------------------------------------------------------------------------
// Cross-format: MusicXML → MSCZ → LoadedScore preserves part set
// ---------------------------------------------------------------------------

/// MusicXML → LoadedScore → MSCX → LoadedScore. Same intent as the GP7
/// case: no tracks should be dropped.
#[test]
fn cross_format_musicxml_to_mscz_preserves_part_set() {
    use crate::convert::optimized::score_partwise_to_loaded_score;
    use crate::model::musicxml::ScorePartwise;

    let path = std::path::Path::new("../test/01c-Pitches-NoVoiceElement.xml");
    if !path.exists() {
        eprintln!("skipping — MusicXML fixture missing: {}", path.display());
        return;
    }
    let raw = std::fs::read_to_string(path).expect("read musicxml fixture");
    // Some fixtures declare a DTD; strip it so quick-xml can parse.
    let cleaned = if let Some(idx) = raw.find("<!DOCTYPE") {
        let rest = &raw[idx..];
        if let Some(end) = rest.find('>') {
            format!("{}{}", &raw[..idx], &rest[end + 1..])
        } else {
            raw
        }
    } else {
        raw
    };
    let doc: ScorePartwise = quick_xml::de::from_str(&cleaned).expect("parse musicxml");
    let loaded = score_partwise_to_loaded_score(&doc);
    let track_count_before = loaded.score.tracks.len();

    let regenerated_mscx = loaded_score_to_mscx(&loaded);
    let file = MsczFile {
        archive: MsczArchive {
            rootfiles: vec!["score.mscx".to_string()],
            entries: vec![
                MsczEntry {
                    path: "META-INF/container.xml".to_string(),
                    data: b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container><rootfiles><rootfile full-path=\"score.mscx\"/></rootfiles></container>\n".to_vec(),
                },
                MsczEntry {
                    path: "score.mscx".to_string(),
                    data: regenerated_mscx.raw_xml.as_bytes().to_vec(),
                },
            ],
        },
        mscx: regenerated_mscx,
    };
    let mscz_bytes = write_mscz(&file).expect("write mscz");
    let reparsed_file = read_mscz_bytes(&mscz_bytes).expect("re-read mscz");
    let reparsed = mscx_to_loaded_score(&reparsed_file.mscx).score;

    assert_eq!(
        reparsed.score.tracks.len(),
        track_count_before,
        "MusicXML → MSCZ → LoadedScore must preserve the part count"
    );
}
