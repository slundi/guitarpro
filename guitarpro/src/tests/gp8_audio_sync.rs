//! GP8 audio sync support: structured `SyncPoint` parsing and embedded
//! audio-asset extraction.
//!
//! SyncPoint automations and embedded backing-track audio (`Content/Assets/*.mp3`)
//! are **GP8** features — GP7 `.gp` files do not carry them (the ZIP `VERSION`
//! marker says `7.0` for both, but GP8 files declare `<GPVersion>8.x</GPVersion>`
//! and `<EncodingDescription>GP8</EncodingDescription>` in the GPIF). These
//! tests pin down:
//!
//! 1. SyncPoint automations parse into structured fields (not just a flat
//!    string list), so consumers can read `frame_offset` etc.
//! 2. `Song` exposes the collected sync points.
//! 3. Reading a `.gp` with an embedded backing track surfaces the raw audio
//!    bytes.

use super::common::read_file;
use crate::io::gpx::{read_gp, read_gp_with_audio};
use crate::model::legacy::song::Song;

// ---------------------------------------------------------------------------
// SyncPoint structured parsing
// ---------------------------------------------------------------------------

/// A `.gp` fixture (GPIF `GPVersion 8.x`, `hasAudio: true`) whose `score.gpif`
/// contains a `SyncPoint` automation must parse into typed fields. Guard
/// against the previous failure mode where the structured `<Value>` crashed
/// the XML reader.
#[test]
fn gp8_syncpoint_automation_parses_structurally() {
    let mut song = Song::default();
    song.read_gp(&read_file(String::from(
        "test/edge_cases/gp8_syncpoint_audio.gp",
    )))
    .expect("file with SyncPoint automation should load");

    let sync_points = &song.sync_points;
    assert!(
        !sync_points.is_empty(),
        "Song.sync_points should be populated from SyncPoint automations"
    );
    let sp = &sync_points[0];
    assert_eq!(sp.bar_index, 0, "first sync point anchors bar 0");
    assert!(sp.frame_offset.is_some(), "frame offset must be parsed");
    assert!(
        sp.modified_tempo.is_some(),
        "modified tempo must be parsed from SyncPoint value"
    );
}

/// The raw `Gpif` layer must expose typed automation values so the web server
/// and CLI can consume them without re-parsing strings.
#[test]
fn gpif_automation_value_exposes_syncpoint_fields() {
    let gpif = read_gp(&read_file(String::from(
        "test/edge_cases/gp8_syncpoint_audio.gp",
    )))
    .expect("read_gp should parse the GPIF");
    let automations = gpif
        .master_track
        .automations
        .as_ref()
        .expect("MasterTrack should carry automations")
        .automations
        .clone();
    let sync = automations
        .iter()
        .find(|a| a.automation_type == "SyncPoint")
        .expect("fixture should contain a SyncPoint automation");

    let value = sync
        .value
        .as_ref()
        .expect("SyncPoint automation must have a Value");
    assert!(
        value.bar_index.is_some(),
        "BarIndex should be parsed structurally"
    );
    assert!(
        value.frame_offset.is_some(),
        "FrameOffset should be parsed structurally"
    );
}

// ---------------------------------------------------------------------------
// Embedded backing-track audio
// ---------------------------------------------------------------------------

/// Reading a `.gp` (GP8) with an embedded backing track should return the raw
/// audio bytes alongside the parsed score.
#[test]
fn gp8_embedded_audio_asset_is_extracted() {
    let bytes = read_file(String::from("test/edge_cases/gp8_syncpoint_audio.gp"));
    let (_, audio) = read_gp_with_audio(&bytes).expect("audio asset should be extracted");
    assert!(
        audio.is_some(),
        "fixture with hasAudio=true should yield a backing track"
    );
    let audio = audio.expect("checked above");
    // MP3 files start with the ID3 tag.
    assert_eq!(
        &audio[0..3],
        b"ID3",
        "extracted asset should be an MP3 (ID3 header)"
    );
}

/// `Song::read_gp` should retain the embedded audio in memory for downstream
/// consumers (web `/audio` endpoint, CLI extraction).
#[test]
fn song_read_gp_retains_audio() {
    let mut song = Song::default();
    let bytes = read_file(String::from("test/edge_cases/gp8_syncpoint_audio.gp"));
    song.read_gp(&bytes).expect("GP should parse");
    assert!(
        song.backing_track_audio.is_some(),
        "Song should retain embedded backing-track audio"
    );
    let audio = song.backing_track_audio.as_ref().unwrap();
    assert_eq!(&audio[0..3], b"ID3", "audio should be MP3 data");
}
