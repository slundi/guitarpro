//! Roundtrip tests: `legacy::Song` → `optimized::LoadedScore` → `legacy::Song` → GP bytes.
//!
//! Goal: the bytes produced by writing before and after conversion through the optimized
//! model must be identical.  For GP3/GP4/GP5 this checks against the on-disk bytes;
//! for GPX/.gp (XML-based) it checks write-idempotency through the optimized model.

use crate::{
    convert::{
        legacy::loaded_score_to_legacy_song, optimized::legacy::legacy_song_to_loaded_score,
    },
    model::legacy::song::Song,
};

/// Core helper: for every file with extension `ext` under `../test`, reads with `read_fn`,
/// writes with `write_fn` to get reference bytes, converts through the optimized model,
/// writes again, and asserts the bytes are identical.
fn run_via_optimized<R, W>(label: &str, ext: &str, read_fn: R, write_fn: W)
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
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == ext))
        .collect();
    entries.sort_by_key(|e| e.file_name());

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

            // 3. Convert legacy → optimized → legacy
            let loaded = legacy_song_to_loaded_score(&song1);
            let song2 = loaded_score_to_legacy_song(&loaded);

            // 4. Write after conversion
            let written_after = write_fn(&song2);

            // 5. Compare
            if written_before.len() != written_after.len() {
                // Find first difference to help diagnose
                let min_len = written_before.len().min(written_after.len());
                let first_diff = (0..min_len).find(|&i| written_before[i] != written_after[i]);
                panic!(
                    "length mismatch: before={} after={}, first diff at {:?}",
                    written_before.len(),
                    written_after.len(),
                    first_diff.map(|pos| {
                        let lo = pos.saturating_sub(8);
                        format!(
                            "pos={pos} before={:?} after={:?}",
                            &written_before[lo..written_before.len().min(pos + 8)],
                            &written_after[lo..written_after.len().min(pos + 8)]
                        )
                    })
                );
            }
            if written_before != written_after {
                let pos = written_before
                    .iter()
                    .zip(written_after.iter())
                    .position(|(a, b)| a != b)
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
        "{label} via-optimized roundtrip: {} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    for f in &failures {
        eprintln!("FAIL: {}", f);
    }
    assert!(
        failures.is_empty(),
        "{} {label} file(s) failed legacy→optimized→legacy roundtrip",
        failures.len()
    );
}

#[test]
fn test_gp3_via_optimized_roundtrip() {
    run_via_optimized(
        "GP3",
        "gp3",
        |song, data| song.read_gp3(data).unwrap(),
        |song| song.write(song.version.number, None).unwrap(),
    );
}

#[test]
fn test_gp4_via_optimized_roundtrip() {
    run_via_optimized(
        "GP4",
        "gp4",
        |song, data| song.read_gp4(data).unwrap(),
        |song| song.write(song.version.number, None).unwrap(),
    );
}

#[test]
fn test_gp5_via_optimized_roundtrip() {
    run_via_optimized(
        "GP5",
        "gp5",
        |song, data| song.read_gp5(data).unwrap(),
        |song| song.write(song.version.number, None).unwrap(),
    );
}

#[test]
fn test_gpx_via_optimized_roundtrip() {
    run_via_optimized(
        "GPX",
        "gpx",
        |song, data| song.read_gpx(data).unwrap(),
        |song| song.write_gpx().unwrap(),
    );
}

#[test]
fn test_gp7_via_optimized_roundtrip() {
    run_via_optimized(
        "GP7",
        "gp",
        |song, data| song.read_gp(data).unwrap(),
        |song| song.write_gp().unwrap(),
    );
}

/// A measure's ticks at a resolution too large for them are `None`, not
/// an overflow.
#[test]
fn measure_ticks_too_large_are_none() {
    use crate::model::optimized::metadata::TimeSignature;

    let time = TimeSignature {
        numerator: 6,
        denominator: 8,
    };
    assert_eq!(time.measure_ticks(3), Some(9));
    assert_eq!(time.measure_ticks(u64::MAX / 4), None);
}

/// A positional (non-self-describing) deserializer over JSON arrays, as a
/// stand-in for Postcard.
struct Positional<D>(D);

impl<'de, D: serde::Deserializer<'de>> serde::Deserializer<'de> for Positional<D> {
    type Error = D::Error;

    fn deserialize_any<V: serde::de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_any(visitor)
    }

    fn is_human_readable(&self) -> bool {
        false
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

/// A binary save of another version is refused by its version, before the
/// rest (here, not even a score) is decoded.
#[test]
fn binary_saves_of_another_version_are_refused_first() {
    use serde::Deserialize;

    use crate::model::optimized::global::Score;

    for version in [0, 1, 3] {
        let json = format!(r#"[{version}, "not a score"]"#);
        let mut deserializer = serde_json::Deserializer::from_str(&json);
        let error = match Score::deserialize(Positional(&mut deserializer)) {
            Ok(_) => panic!("version {version} read"),
            Err(error) => error.to_string(),
        };
        assert!(
            error.contains(&format!("binary score format version {version} isn't")),
            "{error}"
        );
    }
}
