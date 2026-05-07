use crate::model::song::Song;

// ==================== Round-trip tests (GP3/GP4/GP5) ====================

#[test]
fn test_gp3_all_files_roundtrip() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gp3") {
            let fname = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = fs::read(&path).unwrap();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut song1 = Song::default();
                song1.read_gp3(&data).unwrap();
                let written1 = song1.write(song1.version.number, None).unwrap();
                let mut song2 = Song::default();
                song2.read_gp3(&written1).unwrap();
                let written2 = song2.write(song2.version.number, None).unwrap();
                if written1 != written2 {
                    let pos = written1
                        .iter()
                        .zip(written2.iter())
                        .position(|(a, b)| a != b)
                        .unwrap_or(written1.len().min(written2.len()));
                    let w1 = &written1[pos.saturating_sub(2)..written1.len().min(pos + 6)];
                    let w2 = &written2[pos.saturating_sub(2)..written2.len().min(pos + 6)];
                    panic!(
                        "round-trip produced different bytes on second write at byte {pos}: {w1:?} vs {w2:?}"
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
                        "unknown".to_string()
                    };
                    failures.push(format!("{}: {}", fname, &msg[..msg.len().min(120)]));
                }
            }
        }
    }
    eprintln!(
        "GP3 round-trip: {} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    for f in &failures {
        eprintln!("FAIL: {}", f);
    }
    assert!(
        failures.is_empty(),
        "{} GP3 files failed round-trip",
        failures.len()
    );
}

#[test]
fn test_gp4_all_files_roundtrip() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gp4") {
            let fname = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = fs::read(&path).unwrap();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut song1 = Song::default();
                song1.read_gp4(&data).unwrap();
                let written1 = song1.write(song1.version.number, None).unwrap();
                let mut song2 = Song::default();
                song2.read_gp4(&written1).unwrap();
                let written2 = song2.write(song2.version.number, None).unwrap();
                assert_eq!(
                    written1, written2,
                    "round-trip produced different bytes on second write"
                );
            }));
            match result {
                Ok(_) => pass += 1,
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown".to_string()
                    };
                    failures.push(format!("{}: {}", fname, &msg[..msg.len().min(120)]));
                }
            }
        }
    }
    eprintln!(
        "GP4 round-trip: {} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    for f in &failures {
        eprintln!("FAIL: {}", f);
    }
    assert!(
        failures.is_empty(),
        "{} GP4 files failed round-trip",
        failures.len()
    );
}

#[test]
fn test_gp5_all_files_roundtrip() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gp5") {
            let fname = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = fs::read(&path).unwrap();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut song1 = Song::default();
                song1.read_gp5(&data).unwrap();
                let written1 = song1.write(song1.version.number, None).unwrap();
                let mut song2 = Song::default();
                song2.read_gp5(&written1).unwrap();
                let written2 = song2.write(song2.version.number, None).unwrap();
                if written1 != written2 {
                    let pos = written1
                        .iter()
                        .zip(written2.iter())
                        .position(|(a, b)| a != b)
                        .unwrap_or(written1.len().min(written2.len()));
                    let w1 = &written1[pos.saturating_sub(8)..written1.len().min(pos + 8)];
                    let w2 = &written2[pos.saturating_sub(8)..written2.len().min(pos + 8)];
                    panic!(
                        "round-trip produced different bytes on second write at byte {pos}: {:?} vs {:?}",
                        w1, w2
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
                        "unknown".to_string()
                    };
                    failures.push(format!("{}: {}", fname, &msg[..msg.len().min(200)]));
                }
            }
        }
    }
    eprintln!(
        "GP5 round-trip: {} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    for f in &failures {
        eprintln!("FAIL: {}", f);
    }
    assert!(
        failures.is_empty(),
        "{} GP5 files failed round-trip",
        failures.len()
    );
}

// ==================== GP7 round-trip tests ====================

#[test]
fn test_gp7_all_files_roundtrip() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();

    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "gp") {
            continue;
        }
        let fname = path.file_name().unwrap().to_str().unwrap().to_string();
        let data = fs::read(&path).unwrap();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // First read + write
            let mut song1 = Song::default();
            song1.read_gp(&data).unwrap();
            let written1 = song1.write_gp().unwrap();

            // Second read + write (idempotency check)
            let mut song2 = Song::default();
            song2.read_gp(&written1).unwrap();
            let written2 = song2.write_gp().unwrap();

            if written1.len() != written2.len() {
                panic!(
                    "length mismatch: write1={} write2={}",
                    written1.len(),
                    written2.len()
                );
            }
            if written1 != written2 {
                let pos = written1
                    .iter()
                    .zip(written2.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(0);
                panic!(
                    "bytes differ at position {pos}: {:?} vs {:?}",
                    &written1[pos.saturating_sub(4)..written1.len().min(pos + 8)],
                    &written2[pos.saturating_sub(4)..written2.len().min(pos + 8)],
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
                    "unknown".to_string()
                };
                failures.push(format!("{}: {}", fname, &msg[..msg.len().min(120)]));
            }
        }
    }

    eprintln!(
        "GP7 round-trip: {} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    for f in &failures {
        eprintln!("FAIL: {f}");
    }
    assert!(
        failures.is_empty(),
        "{} GP7 files failed round-trip",
        failures.len()
    );
}

// ==================== Debug round-trip tests ====================

#[test]
fn test_debug_gp3_roundtrip() {
    use std::fs;
    let path = "../test/Effects.gp3";
    let data = fs::read(path).unwrap();
    let mut song1 = Song::default();
    song1.read_gp3(&data).unwrap();
    let written1 = song1.write(song1.version.number, None).unwrap();

    // Print some structural info
    eprintln!(
        "song1: {} measures, {} tracks",
        song1.measure_headers.len(),
        song1.tracks.len()
    );
    eprintln!(
        "  name={:?} subtitle={:?} artist={:?} album={:?}",
        song1.name, song1.subtitle, song1.artist, song1.album
    );
    eprintln!(
        "  words={:?} copyright={:?} writer={:?} instructions={:?}",
        song1.words, song1.copyright, song1.writer, song1.instructions
    );
    for (i, mh) in song1.measure_headers.iter().enumerate().take(5) {
        eprintln!(
            "  mh[{i}]: repeat_close={}, repeat_alt={}, marker={:?}, keysig={:?}",
            mh.repeat_close,
            mh.repeat_alternative,
            mh.marker.as_ref().map(|m| &m.title),
            mh.key_signature.key
        );
    }
    eprintln!(
        "  triplet_feel={:?} tempo={} key={}",
        song1.triplet_feel, song1.tempo, song1.key.key
    );
    eprintln!("written1 size: {}", written1.len());

    let mut song2 = Song::default();
    song2.read_gp3(&written1).unwrap();
    eprintln!(
        "song2: {} measures, {} tracks",
        song2.measure_headers.len(),
        song2.tracks.len()
    );
    for (i, mh) in song2.measure_headers.iter().enumerate().take(5) {
        eprintln!(
            "  mh[{i}]: repeat_close={}, repeat_alt={}, marker={:?}, keysig={:?}",
            mh.repeat_close,
            mh.repeat_alternative,
            mh.marker.as_ref().map(|m| &m.title),
            mh.key_signature.key
        );
    }

    // Compare beats between song1 and song2
    'outer: for (ti, track1) in song1.tracks.iter().enumerate() {
        let track2 = &song2.tracks[ti];
        for (mi, meas1) in track1.measures.iter().enumerate() {
            let meas2 = &track2.measures[mi];
            for (vi, voice1) in meas1.voices.iter().enumerate() {
                let voice2 = &meas2.voices[vi];
                for (bi, beat1) in voice1.beats.iter().enumerate() {
                    if bi >= voice2.beats.len() {
                        break;
                    }
                    let beat2 = &voice2.beats[bi];
                    if beat1.effect != beat2.effect || beat1.notes.len() != beat2.notes.len() {
                        eprintln!("Beat diff at track={ti} measure={mi} voice={vi} beat={bi}");
                        eprintln!("  beat1.effect={:?}", beat1.effect);
                        eprintln!("  beat2.effect={:?}", beat2.effect);
                        eprintln!(
                            "  beat1.has_harmonic={} beat1.has_vibrato={}",
                            beat1.has_harmonic(),
                            beat1.has_vibrato()
                        );
                        eprintln!(
                            "  beat2.has_harmonic={} beat2.has_vibrato={}",
                            beat2.has_harmonic(),
                            beat2.has_vibrato()
                        );
                        for (ni, n1) in beat1.notes.iter().enumerate() {
                            if ni < beat2.notes.len() {
                                let n2 = &beat2.notes[ni];
                                if n1.effect != n2.effect {
                                    eprintln!(
                                        "  note[{ni}] effect diff: {:?} vs {:?}",
                                        n1.effect, n2.effect
                                    );
                                }
                            }
                        }
                        break 'outer;
                    }
                    // Also check note effects
                    for (ni, n1) in beat1.notes.iter().enumerate() {
                        if ni >= beat2.notes.len() {
                            break;
                        }
                        let n2 = &beat2.notes[ni];
                        if n1.effect != n2.effect {
                            eprintln!(
                                "Note diff at track={ti} measure={mi} voice={vi} beat={bi} note={ni}"
                            );
                            eprintln!("  n1.effect={:?}", n1.effect);
                            eprintln!("  n2.effect={:?}", n2.effect);
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    let written2 = song2.write(song2.version.number, None).unwrap();
    if written1 != written2 {
        let pos = written1
            .iter()
            .zip(written2.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(written1.len().min(written2.len()));
        let w1 = &written1[pos.saturating_sub(8)..written1.len().min(pos + 16)];
        let w2 = &written2[pos.saturating_sub(8)..written2.len().min(pos + 16)];
        eprintln!("First diff at byte {pos}");
        eprintln!("written1[{}..]: {:?}", pos.saturating_sub(8), w1);
        eprintln!("written2[{}..]: {:?}", pos.saturating_sub(8), w2);
        panic!("round-trip mismatch at byte {pos}");
    }
}

// ==================== Debug volta test ====================
#[test]
fn test_debug_volta_gp3() {
    use std::fs;
    let path = "../test/volta.gp3";
    let data = fs::read(path).unwrap();
    let mut song1 = Song::default();
    song1.read_gp3(&data).unwrap();
    for (i, mh) in song1.measure_headers.iter().enumerate() {
        if mh.repeat_open || mh.repeat_alternative > 0 || mh.repeat_close >= 0 {
            eprintln!(
                "MH[{}]: repeat_open={} repeat_alternative={} repeat_close={}",
                i, mh.repeat_open, mh.repeat_alternative, mh.repeat_close
            );
        }
    }
    let written1 = song1.write(song1.version.number, None).unwrap();
    let mut song2 = Song::default();
    song2.read_gp3(&written1).unwrap();
    for (i, mh) in song2.measure_headers.iter().enumerate() {
        if mh.repeat_open || mh.repeat_alternative > 0 || mh.repeat_close >= 0 {
            eprintln!(
                "Song2 MH[{}]: repeat_open={} repeat_alternative={} repeat_close={}",
                i, mh.repeat_open, mh.repeat_alternative, mh.repeat_close
            );
        }
    }
    let written2 = song2.write(song2.version.number, None).unwrap();
    if written1 != written2 {
        let pos = written1
            .iter()
            .zip(written2.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(written1.len().min(written2.len()));
        let w1 = &written1[pos.saturating_sub(4)..written1.len().min(pos + 8)];
        let w2 = &written2[pos.saturating_sub(4)..written2.len().min(pos + 8)];
        panic!("diff at byte {pos}: {w1:?} vs {w2:?}");
    }
}
