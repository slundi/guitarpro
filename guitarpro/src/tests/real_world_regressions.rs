//! Regressions found by loading a large corpus of real-world Guitar Pro files.
//! Each test guards one distinct root cause; the fixture files are anonymised
//! copies of user-supplied `.gp*` files.

use super::common::read_file;
use crate::model::legacy::key_signature::read_duration;
use crate::model::legacy::song::Song;

/// Some GP5 exports (e.g. Judas Priest — "Blood Red Skies") store a beat
/// duration byte outside the canonical `-2..=3` range. The reader used to
/// panic on `attempt to add with overflow` (`i8 + 2` on `b == 127`) before
/// falling back to the safe default. The fix widens the intermediate shift to
/// `i16` so the range check runs before any arithmetic overflow can occur.
#[test]
fn gp5_out_of_range_duration_byte_loads() {
    let mut song = Song::default();
    song.read_gp5(&read_file(String::from(
        "test/edge_cases/gp5_duration_byte_out_of_range.gp5",
    )))
    .expect("file should load without overflow");
    assert!(!song.measure_headers.is_empty());
    assert!(!song.tracks.is_empty());
}

/// `read_duration` used to panic on any byte outside `-2..=3` because
/// `let shift = b + 2` overflowed on `b == 127`. The regression test drives
/// the primitive directly with the worst-case value.
#[test]
fn read_duration_does_not_overflow_on_garbage_byte() {
    let data = [127u8, 0, 0, 0, 0];
    let mut seek = 0usize;
    let d = read_duration(&data, &mut seek, 0).expect("should return whole-note fallback");
    assert_eq!(d.value, 1, "out-of-range duration should fall back to 1");
}

/// GP7 (`.gp`) files store the tempo-automation `<Position>` as a fractional
/// value in `[0.0, 1.0)` (e.g. `0.75`, `0.875`). Deserialising it as `i32`
/// used to reject every file that carried a mid-bar tempo change.
#[test]
fn gp7_automation_position_accepts_float() {
    let mut song = Song::default();
    song.read_gp(&read_file(String::from(
        "test/edge_cases/gp7_automation_position_float.gp",
    )))
    .expect("file should load with fractional Position");
    assert!(!song.tracks.is_empty());
}

// ---------------------------------------------------------------------------
// GP3 chord diagram layout
// ---------------------------------------------------------------------------
// The v3 "new format" chord (marker byte `0x01`) uses the same byte-oriented
// layout as GP4 for root/kind/extension/fifth/ninth/eleventh and the barre
// section — NOT the 4-byte-int layout the reader used to assume. Real-world
// GP3 exports (Muse — "Bliss", AC/DC — "Back in Black" and thousands of
// others in the sweep) fail with `chord root ~ 84090884 out of range` under
// the old int layout.

#[test]
fn gp3_new_format_chord_loads() {
    let mut song = Song::default();
    song.read_gp3(&read_file(String::from(
        "test/edge_cases/gp3_int_chord_layout_diagram.gp3",
    )))
    .expect("chord diagram bytes should parse as GP4-style layout");
    assert!(!song.measure_headers.is_empty());
}

// ---------------------------------------------------------------------------
// Tolerant scalar reads
// ---------------------------------------------------------------------------
// A large class of failing files carries garbage in fields that Guitar Pro
// itself tolerates: tremolo-bar value, string tunings for unused slots,
// chord bass/tonality, key signature, etc. The reader now truncates rather
// than rejecting the file so downstream tools still get a usable `Song`.

#[test]
fn gp3_chord_and_tremolo_overflow_load() {
    // AC/DC "Back In Black" (rev 5): first the chord format needs the byte
    // layout fix (root fits), then the tremolo-bar `int` value slot carries
    // ASCII bytes ("Roar") that overflow `i16` unless we widen to a
    // truncating cast.
    let mut song = Song::default();
    song.read_gp3(&read_file(String::from(
        "test/edge_cases/gp3_tremolo_bar_and_chord_overflow.gp3",
    )))
    .expect("file should load with tolerant scalar reads");
    assert!(!song.measure_headers.is_empty());
}

#[test]
fn gp_string_tuning_overflow_loads() {
    // At least one string slot's tuning `int` was `0x00_00_7F_00` (`32512`),
    // which doesn't fit `i8`. Since only strings up to `string_count` are
    // meaningful, unused slots are truncated rather than validated.
    let mut song = Song::default();
    song.read_gp3(&read_file(String::from(
        "test/edge_cases/gp_string_tuning_overflow.gp3",
    )))
    .expect("file should load with tolerant string tuning");
    assert!(!song.tracks.is_empty());
}

#[test]
fn gp_repeat_alternative_clamped() {
    // Some editors emit a raw alternate-ending ordinal (e.g. `10`), which
    // makes `(1 << value) - 1` overflow the u8 result. We clamp to 8 (the
    // spec limit) so parsing succeeds.
    let mut song = Song::default();
    song.read_gp3(&read_file(String::from(
        "test/edge_cases/gp_repeat_alternative_overflow.gp3",
    )))
    .expect("file should load with clamped repeat alternative");
    assert!(!song.measure_headers.is_empty());
}

#[test]
fn slide_type_unknown_falls_back_to_none() {
    use crate::model::legacy::enums::{SlideType, get_slide_type};
    assert_eq!(get_slide_type(-2).unwrap(), SlideType::IntoFromAbove);
    assert_eq!(get_slide_type(0).unwrap(), SlideType::None);
    assert_eq!(get_slide_type(4).unwrap(), SlideType::OutUpWards);
    // Was previously `Err(InvalidValue)`; now returns `None`.
    assert_eq!(get_slide_type(42).unwrap(), SlideType::None);
    assert_eq!(get_slide_type(-99).unwrap(), SlideType::None);
}
