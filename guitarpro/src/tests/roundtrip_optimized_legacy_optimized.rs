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
