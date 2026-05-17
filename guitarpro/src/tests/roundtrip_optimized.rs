//! Roundtrip tests: `legacy::Song` → `optimized::LoadedScore` → `legacy::Song` → GP bytes.
//!
//! Goal: the GP bytes written before and after conversion through the optimized model
//! must be identical in length and content.

use crate::{
    convert::{
        legacy::loaded_score_to_legacy_song, optimized::legacy::legacy_song_to_loaded_score,
    },
    model::legacy::song::Song,
};

#[test]
fn test_gp5_via_optimized_roundtrip() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(test_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "gp5"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let fname = path.file_name().unwrap().to_str().unwrap().to_string();
        let data = fs::read(&path).unwrap();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 1. Read GP5 → legacy Song
            let mut song1 = Song::default();
            song1.read_gp5(&data).unwrap();

            // 2. Write before conversion (reference bytes)
            let written_before = song1.write(song1.version.number, None).unwrap();

            // 3. Convert legacy → optimized → legacy
            let loaded = legacy_song_to_loaded_score(&song1);
            let song2 = loaded_score_to_legacy_song(&loaded);

            // 4. Write after conversion
            let written_after = song2.write(song1.version.number, None).unwrap();

            // 5. Compare
            if written_before.len() != written_after.len() {
                panic!(
                    "length mismatch: before={} after={}",
                    written_before.len(),
                    written_after.len()
                );
            }
            if written_before != written_after {
                let pos = written_before
                    .iter()
                    .zip(written_after.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(0);
                let lo = pos.saturating_sub(8);
                let hi_b = written_before.len().min(pos + 8);
                let hi_a = written_after.len().min(pos + 8);
                panic!(
                    "bytes differ at position {pos}: before={:?} after={:?}",
                    &written_before[lo..hi_b],
                    &written_after[lo..hi_a],
                );
            }
        }));

        match result {
            Ok(_) => pass += 1,
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "unknown panic".to_string()
                };
                failures.push(format!("{}: {}", fname, &msg[..msg.len().min(200)]));
            }
        }
    }

    eprintln!(
        "GP5 via-optimized roundtrip: {} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    for f in &failures {
        eprintln!("FAIL: {}", f);
    }
    assert!(
        failures.is_empty(),
        "{} GP5 file(s) failed legacy→optimized→legacy byte-identical roundtrip",
        failures.len()
    );
}
