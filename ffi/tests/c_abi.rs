//! End-to-end exercises of the C entry points, driven exactly the way a C
//! caller would drive them: raw pointers in, raw pointers out, every handle
//! and buffer released through the matching `*_free`.

use std::ffi::CStr;

use guitarpro_ffi::{
    GpioFormat, gpio_buffer_free, gpio_convert, gpio_last_error, gpio_parse, gpio_score_format,
    gpio_score_free, gpio_score_to_json, gpio_version,
};

/// Sample files live at the workspace root; tests run with the crate dir as
/// the working directory.
fn sample(relative: &str) -> Vec<u8> {
    let path = format!("../{relative}");
    std::fs::read(&path).unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
}

fn last_error() -> Option<String> {
    let pointer = gpio_last_error();
    if pointer.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Copy a buffer's bytes out and release it, mirroring the C usage pattern.
fn take_buffer(buffer: guitarpro_ffi::GpioBuffer) -> Option<Vec<u8>> {
    if buffer.data.is_null() {
        unsafe { gpio_buffer_free(buffer) };
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) }.to_vec();
    unsafe { gpio_buffer_free(buffer) };
    Some(bytes)
}

#[test]
fn exposes_a_version_string() {
    let pointer = gpio_version();
    assert!(!pointer.is_null());
    let version = unsafe { CStr::from_ptr(pointer) }.to_string_lossy();
    assert!(!version.is_empty());
}

#[test]
fn parses_each_supported_input_format() {
    let cases = [
        ("test/Chords.gp3", GpioFormat::Gp3),
        ("test/Chords.gp4", GpioFormat::Gp4),
        ("test/Chords.gp5", GpioFormat::Gp5),
        ("test/accent.gpx", GpioFormat::Gpx),
        ("test/accent.gp", GpioFormat::Gp),
        (
            "guitarpro/samples/mscz/simple_monophonic.mscz",
            GpioFormat::Mscz,
        ),
    ];

    for (path, expected) in cases {
        let bytes = sample(path);
        let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), GpioFormat::Auto as u32) };
        assert!(!handle.is_null(), "{path} failed: {:?}", last_error());
        assert_eq!(
            unsafe { gpio_score_format(handle) },
            expected as u32,
            "{path} was detected as the wrong format"
        );
        unsafe { gpio_score_free(handle) };
    }
}

#[test]
fn an_explicit_format_overrides_detection() {
    let bytes = sample("test/Chords.gp5");
    let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), GpioFormat::Gp5 as u32) };
    assert!(!handle.is_null(), "parse failed: {:?}", last_error());
    assert_eq!(unsafe { gpio_score_format(handle) }, GpioFormat::Gp5 as u32);
    unsafe { gpio_score_free(handle) };
}

#[test]
fn produces_parseable_json_for_a_handle() {
    let bytes = sample("test/Chords.gp5");
    let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), GpioFormat::Auto as u32) };
    assert!(!handle.is_null(), "parse failed: {:?}", last_error());

    let json = take_buffer(unsafe { gpio_score_to_json(handle) })
        .unwrap_or_else(|| panic!("json failed: {:?}", last_error()));
    unsafe { gpio_score_free(handle) };

    let parsed: serde_json::Value = serde_json::from_slice(&json)
        .unwrap_or_else(|error| panic!("output was not valid JSON: {error}"));
    assert!(parsed.is_object(), "expected a JSON object at the root");
}

#[test]
fn converts_one_input_into_every_output_format() {
    let bytes = sample("test/Chords.gp4");
    let targets = [
        GpioFormat::Gp3,
        GpioFormat::Gp4,
        GpioFormat::Gp5,
        GpioFormat::Gpx,
        GpioFormat::Gp,
        GpioFormat::MusicXml,
        GpioFormat::Score,
        GpioFormat::Mscz,
    ];

    for target in targets {
        let out = take_buffer(unsafe {
            gpio_convert(
                bytes.as_ptr(),
                bytes.len(),
                GpioFormat::Auto as u32,
                target as u32,
            )
        })
        .unwrap_or_else(|| panic!("convert to {target:?} failed: {:?}", last_error()));
        assert!(!out.is_empty(), "{target:?} produced no bytes");
    }
}

#[test]
fn converted_output_can_be_parsed_back() {
    // Deliberately a GP5 source. Two pre-existing library defects limit which
    // pairs survive a round trip, and both reproduce through
    // `score_tool convert` with no FFI involved:
    //
    // * GP4 → GP5 writes a page-setup block the GP5 reader then rejects
    //   ("Type conversion failed for page width"), so a GP4 source cannot be
    //   used here.
    // * MusicXML export emits absent optional attributes as `attr=""`, which
    //   its own importer rejects, so that target is excluded below.
    //
    // Export to both is still covered by
    // `converts_one_input_into_every_output_format`; re-widen this list once
    // the library gaps close.
    let bytes = sample("test/Chords.gp5");

    for target in [
        GpioFormat::Gp3,
        GpioFormat::Gp4,
        GpioFormat::Gp5,
        GpioFormat::Gpx,
        GpioFormat::Gp,
        GpioFormat::Score,
        GpioFormat::Mscz,
    ] {
        let out = take_buffer(unsafe {
            gpio_convert(
                bytes.as_ptr(),
                bytes.len(),
                GpioFormat::Gp5 as u32,
                target as u32,
            )
        })
        .unwrap_or_else(|| panic!("convert to {target:?} failed: {:?}", last_error()));

        let handle = unsafe { gpio_parse(out.as_ptr(), out.len(), target as u32) };
        assert!(
            !handle.is_null(),
            "re-parsing {target:?} output failed: {:?}",
            last_error()
        );
        assert_eq!(unsafe { gpio_score_format(handle) }, target as u32);
        unsafe { gpio_score_free(handle) };
    }
}

#[test]
fn reports_failures_without_leaking_a_handle() {
    // Garbage in: no handle, and a message the caller can show.
    let garbage = b"this is not a score file";
    let handle = unsafe { gpio_parse(garbage.as_ptr(), garbage.len(), GpioFormat::Auto as u32) };
    assert!(handle.is_null());
    assert!(
        last_error().is_some_and(|message| message.contains("cannot detect")),
        "expected a detection error, got {:?}",
        last_error()
    );
}

#[test]
fn rejects_auto_as_a_conversion_target() {
    let bytes = sample("test/Chords.gp5");
    let result = take_buffer(unsafe {
        gpio_convert(
            bytes.as_ptr(),
            bytes.len(),
            GpioFormat::Gp5 as u32,
            GpioFormat::Auto as u32,
        )
    });
    assert!(result.is_none());
    assert!(last_error().is_some_and(|message| message.contains("AUTO")));
}

#[test]
fn survives_truncated_input_without_unwinding() {
    // A half file is the classic way to trip an indexing panic deep in a
    // binary parser. The guard must turn that into an ordinary error.
    let full = sample("test/Chords.gp5");
    let truncated = &full[..full.len() / 2];

    let handle = unsafe { gpio_parse(truncated.as_ptr(), truncated.len(), GpioFormat::Gp5 as u32) };
    if handle.is_null() {
        assert!(last_error().is_some(), "a failure must leave a message");
    } else {
        // Some truncations still yield a structurally valid prefix; that is a
        // legitimate outcome as long as the handle is usable and releasable.
        unsafe { gpio_score_free(handle) };
    }
}

#[test]
fn handles_are_independent_across_threads() {
    let first = sample("test/Chords.gp3");
    let second = sample("test/Chords.gp5");

    let worker = std::thread::spawn(move || {
        let handle = unsafe { gpio_parse(second.as_ptr(), second.len(), GpioFormat::Auto as u32) };
        assert!(!handle.is_null());
        let format = unsafe { gpio_score_format(handle) };
        unsafe { gpio_score_free(handle) };
        format
    });

    let handle = unsafe { gpio_parse(first.as_ptr(), first.len(), GpioFormat::Auto as u32) };
    assert!(!handle.is_null());
    assert_eq!(unsafe { gpio_score_format(handle) }, GpioFormat::Gp3 as u32);
    unsafe { gpio_score_free(handle) };

    let other = worker.join().unwrap_or(u32::MAX);
    assert_eq!(other, GpioFormat::Gp5 as u32);
}

#[test]
fn many_parse_free_cycles_stay_stable() {
    // Exercises the allocate/release path repeatedly; under a leak checker or
    // ASan this is where a mismatched free would show up.
    let bytes = sample("test/Chords.gp4");
    for _ in 0..32 {
        let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), GpioFormat::Gp4 as u32) };
        assert!(!handle.is_null());
        let json = take_buffer(unsafe { gpio_score_to_json(handle) });
        assert!(json.is_some());
        unsafe { gpio_score_free(handle) };
    }
}
