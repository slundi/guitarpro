//! Regression tests for the fixed-width sections of the Guitar Pro writers.
//!
//! Several parts of the GP5 layout have a count fixed by the format rather
//! than by the data: 64 MIDI channels, two voices per measure, five lyric
//! lines, an 11-byte master equalizer and a 4-byte track equalizer. The
//! readers always consume exactly those amounts.
//!
//! A `Song` read from a GP file naturally carries them, so the writers used to
//! get away with emitting `vec.len()` entries. A `Song` built by a *converter*
//! (MusicXML, MSCZ, the optimized model) does not: it holds only what the
//! source format expressed. Writing short then desynchronizes the stream, and
//! the file fails to read back — historically as a nonsense "page width", an
//! unexpected EOF, or an outright panic in the lyric writer.
//!
//! These tests pin the invariant: **whatever the model holds, the writer emits
//! the width the reader expects.**

use crate::{
    audio::midi::{MIDI_CHANNEL_COUNT, SongMidiOps},
    convert::{guitarpro::musicxml_to_legacy_song, musicxml::song_to_score_partwise},
    model::legacy::{
        lyric::Lyrics,
        rse::{MASTER_EQUALIZER_KNOBS, RseEqualizer, SongRseOps, TRACK_EQUALIZER_KNOBS},
        song::Song,
    },
    tests::common::read_file,
};

const GP5: (u8, u8, u8) = (5, 1, 0);
/// Bytes per MIDI channel: instrument `int` + 6 `signed-byte`s + 2 padding.
const MIDI_CHANNEL_BYTES: usize = 12;

fn read_gp5_song(name: &str) -> Song {
    let mut song = Song::default();
    song.read_gp5(&read_file(name.to_string()))
        .unwrap_or_else(|error| panic!("cannot read {name}: {error}"));
    song
}

/// Round-trip `song` through the MusicXML model, exercising the converter path
/// that produces a structurally sparse `Song`.
fn via_musicxml(song: &Song) -> Song {
    let score = song_to_score_partwise(song);
    let xml = quick_xml::se::to_string(&score)
        .unwrap_or_else(|error| panic!("MusicXML serialize failed: {error}"));
    let parsed = quick_xml::de::from_str(&xml)
        .unwrap_or_else(|error| panic!("MusicXML parse failed: {error}"));
    musicxml_to_legacy_song(&parsed)
}

// ---------------------------------------------------------------------------
// End-to-end: a converter-built Song must write a readable GP5
// ---------------------------------------------------------------------------

#[test]
fn musicxml_derived_song_writes_a_readable_gp5() {
    let song = read_gp5_song("Chords.gp5");
    let rebuilt = via_musicxml(&song);

    let bytes = rebuilt
        .write(GP5, None)
        .unwrap_or_else(|error| panic!("GP5 write failed: {error}"));

    let mut reread = Song::default();
    reread
        .read_gp5(&bytes)
        .unwrap_or_else(|error| panic!("GP5 produced from MusicXML does not read back: {error}"));
}

#[test]
fn musicxml_derived_songs_write_readable_gp5_for_every_legacy_source() {
    for name in ["Chords.gp3", "Chords.gp4", "Chords.gp5"] {
        let mut song = Song::default();
        let data = read_file(name.to_string());
        let result = match name {
            "Chords.gp3" => song.read_gp3(&data),
            "Chords.gp4" => song.read_gp4(&data),
            _ => song.read_gp5(&data),
        };
        result.unwrap_or_else(|error| panic!("cannot read {name}: {error}"));

        let bytes = via_musicxml(&song)
            .write(GP5, None)
            .unwrap_or_else(|error| panic!("{name}: GP5 write failed: {error}"));

        let mut reread = Song::default();
        reread
            .read_gp5(&bytes)
            .unwrap_or_else(|error| panic!("{name}: GP5 does not read back: {error}"));
    }
}

/// A GP4 source has no page setup, no RSE master effect and no lyric lines of
/// its own, so writing it as GP5 exercises every padded section at once. This
/// used to fail with `Type conversion failed for page width`.
#[test]
fn gp4_source_writes_a_readable_gp5() {
    let mut song = Song::default();
    song.read_gp4(&read_file("Chords.gp4".to_string()))
        .unwrap_or_else(|error| panic!("cannot read Chords.gp4: {error}"));

    let bytes = song
        .write(GP5, None)
        .unwrap_or_else(|error| panic!("GP5 write failed: {error}"));

    let mut reread = Song::default();
    reread
        .read_gp5(&bytes)
        .unwrap_or_else(|error| panic!("GP4-sourced GP5 does not read back: {error}"));
}

// ---------------------------------------------------------------------------
// Per-section width invariants
// ---------------------------------------------------------------------------

#[test]
fn midi_channels_are_always_written_in_full() {
    // A converter typically fills only the channels it uses.
    let mut song = read_gp5_song("Chords.gp5");
    song.channels.truncate(2);

    let mut data = Vec::new();
    song.write_midi_channels(&mut data);

    assert_eq!(
        data.len(),
        MIDI_CHANNEL_COUNT as usize * MIDI_CHANNEL_BYTES,
        "the reader always consumes {MIDI_CHANNEL_COUNT} channels"
    );
}

#[test]
fn midi_channels_with_none_present_still_write_in_full() {
    let mut song = read_gp5_song("Chords.gp5");
    song.channels.clear();

    let mut data = Vec::new();
    song.write_midi_channels(&mut data);

    assert_eq!(data.len(), MIDI_CHANNEL_COUNT as usize * MIDI_CHANNEL_BYTES);
}

#[test]
fn equalizers_are_written_at_their_fixed_width() {
    let song = read_gp5_song("Chords.gp5");

    for (knobs, label) in [
        (MASTER_EQUALIZER_KNOBS, "master"),
        (TRACK_EQUALIZER_KNOBS, "track"),
    ] {
        // Short, exact and over-long vectors must all produce `knobs` bytes.
        for length in [0usize, knobs as usize, knobs as usize + 5] {
            let equalizer = RseEqualizer {
                knobs: vec![0.0; length],
                gain: 0.0,
            };
            let mut data = Vec::new();
            song.write_equalizer(&mut data, &equalizer, knobs);
            assert_eq!(
                data.len(),
                knobs as usize,
                "{label} equalizer with {length} knobs must still write {knobs} bytes"
            );
        }
    }
}

#[test]
fn a_defaulted_master_equalizer_matches_what_the_reader_consumes() {
    let song = Song::default();
    assert_eq!(
        song.master_effect.equalizer.knobs.len(),
        MASTER_EQUALIZER_KNOBS as usize,
        "the default must carry the gain byte, not just the bands"
    );
}

#[test]
fn empty_lyrics_write_five_slots_without_panicking() {
    let mut song = read_gp5_song("Chords.gp5");
    song.lyrics = Lyrics::default();
    assert!(song.lyrics.lines.is_empty());

    // Historically this panicked with "index out of bounds".
    let bytes = song
        .write(GP5, None)
        .unwrap_or_else(|error| panic!("GP5 write failed with empty lyrics: {error}"));

    let mut reread = Song::default();
    reread
        .read_gp5(&bytes)
        .unwrap_or_else(|error| panic!("GP5 with empty lyrics does not read back: {error}"));
    assert_eq!(reread.lyrics.lines.len(), 5);
}

#[test]
fn a_measure_with_one_voice_writes_both() {
    let mut song = read_gp5_song("Chords.gp5");
    // Mimic the MusicXML importer, which emits one voice per XML `<voice>`.
    for track in &mut song.tracks {
        for measure in &mut track.measures {
            measure.voices.truncate(1);
        }
    }

    let bytes = song
        .write(GP5, None)
        .unwrap_or_else(|error| panic!("GP5 write failed: {error}"));

    let mut reread = Song::default();
    reread
        .read_gp5(&bytes)
        .unwrap_or_else(|error| panic!("single-voice measures do not read back: {error}"));

    for track in &reread.tracks {
        for measure in &track.measures {
            assert_eq!(measure.voices.len(), 2, "GP5 always stores two voices");
        }
    }
}

// ---------------------------------------------------------------------------
// MusicXML serialization: absent optionals must be omitted, not emptied
// ---------------------------------------------------------------------------

#[test]
fn musicxml_output_has_no_empty_attributes_or_elements() {
    let song = read_gp5_song("Chords.gp5");
    let xml = quick_xml::se::to_string(&song_to_score_partwise(&song))
        .unwrap_or_else(|error| panic!("MusicXML serialize failed: {error}"));

    // `None` used to be written as `name=""` / `<name/>`, which the importer
    // then rejected (e.g. `invalid type: string "", expected u16`).
    assert!(
        !xml.contains("=\"\""),
        "serialized MusicXML still contains an empty attribute"
    );

    // Empty *elements* are not wrong by themselves — MusicXML uses a few as
    // presence flags — so allow the legitimate ones and reject the rest.
    const PRESENCE_FLAGS: [&str; 6] = ["chord", "work", "rest", "dot", "grace", "tied"];
    let mut unexpected: Vec<String> = Vec::new();
    let mut rest = xml.as_str();
    while let Some(start) = rest.find("<") {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('>') else { break };
        let tag = &rest[..end];
        // Only bare `<name/>` forms matter; anything carrying attributes is
        // conveying real data.
        if let Some(name) = tag.strip_suffix('/')
            && !name.is_empty()
            && name.chars().all(|c| c.is_ascii_lowercase() || c == '-')
            && !PRESENCE_FLAGS.contains(&name)
        {
            unexpected.push(name.to_string());
        }
        rest = &rest[end + 1..];
    }
    unexpected.sort();
    unexpected.dedup();
    assert!(
        unexpected.is_empty(),
        "absent optional elements must be omitted, found: {unexpected:?}"
    );
}

#[test]
fn musicxml_output_reimports() {
    for name in ["Chords.gp3", "Chords.gp4", "Chords.gp5"] {
        let mut song = Song::default();
        let data = read_file(name.to_string());
        let result = match name {
            "Chords.gp3" => song.read_gp3(&data),
            "Chords.gp4" => song.read_gp4(&data),
            _ => song.read_gp5(&data),
        };
        result.unwrap_or_else(|error| panic!("cannot read {name}: {error}"));

        let xml = quick_xml::se::to_string(&song_to_score_partwise(&song))
            .unwrap_or_else(|error| panic!("{name}: serialize failed: {error}"));
        let reparsed: Result<crate::model::musicxml::ScorePartwise, _> =
            quick_xml::de::from_str(&xml);
        assert!(
            reparsed.is_ok(),
            "{name}: exported MusicXML does not re-import: {:?}",
            reparsed.err()
        );
    }
}

// ---------------------------------------------------------------------------
// Deterministic output
// ---------------------------------------------------------------------------

#[test]
fn gp7_bytes_are_reproducible() {
    // The ZIP header used to carry the wall clock, so two writes of the same
    // song differed whenever they straddled a timestamp tick.
    let song = read_gp5_song("Chords.gp5");
    let first = song
        .write_gp()
        .unwrap_or_else(|error| panic!("GP write failed: {error}"));
    let second = song
        .write_gp()
        .unwrap_or_else(|error| panic!("GP write failed: {error}"));

    assert_eq!(
        first, second,
        "writing the same song twice must produce identical GP7 bytes"
    );
}
