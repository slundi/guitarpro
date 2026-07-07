//! Roundtrip tests: `legacy::Song` → `musicxml::ScorePartwise` → `legacy::Song` → GP bytes.
//!
//! Goal (Roadmap `Conversion`): the bytes produced by writing a file before and
//! after a roundtrip through the MusicXML model must be identical — same size,
//! same bytes.
//!
//! The roundtrip uses **only** the legacy↔MusicXML converters
//! ([`song_to_score_partwise`] and [`musicxml_to_legacy_song`]); it never passes
//! through the `optimized` model or any other intermediate representation.
//!
//! **Status: `#[ignore]`d — the goal is not yet met.** Unlike the `optimized`
//! model (purpose-built with lossless `gp.*` passthrough), MusicXML is a lossy
//! target for the Guitar Pro binary formats: it does not model every GP-specific
//! detail (the mandatory five lyric lines, exact MIDI channel/port layout, page
//! setup, flag encodings, …). `musicxml_to_legacy_song` therefore produces a
//! `Song` that does not write back to identical bytes — and in some cases is not
//! yet writable at all (e.g. an empty `lyrics.lines` panics the GP writer, which
//! expects five lines).
//!
//! These tests encode the *target* behaviour and are run on demand with
//! `cargo test -- --ignored` to track progress. Remove the `#[ignore]` once the
//! converters achieve byte-lossless roundtripping. The corresponding Roadmap
//! item stays unchecked until then.

use crate::{
    convert::{guitarpro::musicxml_to_legacy_song, musicxml::song_to_score_partwise},
    model::legacy::song::Song,
};

/// For every file with extension `ext` under `../test`, reads with `read_fn`,
/// writes with `write_fn` to get reference bytes, roundtrips through the MusicXML
/// model, writes again, and asserts the bytes are identical.
fn run_via_musicxml<R, W>(label: &str, ext: &str, read_fn: R, write_fn: W)
where
    R: Fn(&mut Song, &[u8]),
    W: Fn(&Song) -> Vec<u8>,
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
            // 1. Read → legacy Song
            let mut song1 = Song::default();
            read_fn(&mut song1, &data);

            // 2. Write before conversion (reference bytes)
            let written_before = write_fn(&song1);

            // 3. Roundtrip legacy → MusicXML → legacy (no optimized intermediate)
            let partwise = song_to_score_partwise(&song1);
            let song2 = musicxml_to_legacy_song(&partwise);

            // 4. Write after conversion
            let written_after = write_fn(&song2);

            // 5. Compare
            if written_before.len() != written_after.len() {
                let min_len = written_before.len().min(written_after.len());
                let first_diff = (0..min_len).find(|&i| written_before[i] != written_after[i]);
                panic!(
                    "length mismatch: before={} after={}, first diff at {:?}",
                    written_before.len(),
                    written_after.len(),
                    first_diff
                );
            }
            if written_before != written_after {
                let pos = written_before
                    .iter()
                    .zip(written_after.iter())
                    .position(|(before, after)| before != after)
                    .unwrap_or(0);
                let lo = pos.saturating_sub(8);
                panic!(
                    "bytes differ at position {pos}: before={:?} after={:?}",
                    &written_before[lo..written_before.len().min(pos + 8)],
                    &written_after[lo..written_after.len().min(pos + 8)],
                );
            }
        }));

        match result {
            Ok(_) => pass += 1,
            Err(error) => {
                let msg = if let Some(text) = error.downcast_ref::<String>() {
                    text.clone()
                } else if let Some(text) = error.downcast_ref::<&str>() {
                    text.to_string()
                } else {
                    "unknown panic".to_string()
                };
                failures.push(format!("{}: {}", fname, &msg[..msg.len().min(200)]));
            }
        }
    }

    eprintln!(
        "{label} via-musicxml byte roundtrip: {} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    for failure in &failures {
        eprintln!("FAIL: {failure}");
    }
    assert!(
        failures.is_empty(),
        "{} {label} file(s) failed legacy→musicxml→legacy byte roundtrip",
        failures.len()
    );
}

#[test]
#[ignore = "legacy→musicxml→legacy not yet byte-lossless; see module docs"]
fn test_gp3_via_musicxml_byte_roundtrip() {
    run_via_musicxml(
        "GP3",
        "gp3",
        |song, data| song.read_gp3(data).unwrap(),
        |song| song.write(song.version.number, None).unwrap(),
    );
}

#[test]
#[ignore = "legacy→musicxml→legacy not yet byte-lossless; see module docs"]
fn test_gp4_via_musicxml_byte_roundtrip() {
    run_via_musicxml(
        "GP4",
        "gp4",
        |song, data| song.read_gp4(data).unwrap(),
        |song| song.write(song.version.number, None).unwrap(),
    );
}

#[test]
#[ignore = "legacy→musicxml→legacy not yet byte-lossless; see module docs"]
fn test_gp5_via_musicxml_byte_roundtrip() {
    run_via_musicxml(
        "GP5",
        "gp5",
        |song, data| song.read_gp5(data).unwrap(),
        |song| song.write(song.version.number, None).unwrap(),
    );
}

#[test]
#[ignore = "legacy→musicxml→legacy not yet byte-lossless; see module docs"]
fn test_gpx_via_musicxml_byte_roundtrip() {
    run_via_musicxml(
        "GPX",
        "gpx",
        |song, data| song.read_gpx(data).unwrap(),
        |song| song.write_gpx().unwrap(),
    );
}

#[test]
#[ignore = "legacy→musicxml→legacy not yet byte-lossless; see module docs"]
fn test_gp7_via_musicxml_byte_roundtrip() {
    run_via_musicxml(
        "GP7",
        "gp",
        |song, data| song.read_gp(data).unwrap(),
        |song| song.write_gp().unwrap(),
    );
}
