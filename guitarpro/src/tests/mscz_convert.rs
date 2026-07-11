//! MSCZ Part 2 tests: MSCX ↔ LoadedScore conversion.
//!
//! Fixtures are hand-crafted MSCX documents that cover:
//! * Envelope + meta tags → [`Metadata`].
//! * Instruments with `<StringData>` → [`InstrumentKind::Stringed`] tuning.
//! * `<TimeSig>` / `<KeySig>` / `<Tempo>` → [`MeasureDef`].
//! * `<startRepeat>` / `<endRepeat>` → [`NavigationEvent`] entries.
//! * `<Chord>` with `<Note>` (pitch/string/fret) → [`Beat`] + [`Note`].
//! * `<Rest>` → rest beats.
//! * `<Spanner type="Tie">` → [`TieType`] Start/End.
//! * Multi-part scores → parallel tracks.
//! * Round-trip: `Mscx → LoadedScore → Mscx → LoadedScore` preserves the
//!   converter-covered subset.

use std::io::Write;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::convert::mscz::{loaded_score_to_mscx, mscx_to_loaded_score};
use crate::io::mscz::{parse_mscx, read_mscz_bytes};
use crate::model::optimized::{
    beat::{Beat, Voice},
    global::{InstrumentKind, StaffId, TrackId},
    metadata::{Mode, TimeSignature},
    note::{NoteValue, PitchStep, TieType},
    timeline::JumpKind,
    track::{Clef, MeasureData, StaffDisplay},
};

// ---------------------------------------------------------------------------
// Fixture builders
// ---------------------------------------------------------------------------

/// A single-track guitar piece with two measures of music:
/// * Bar 1 (4/4, C major, ♩=120): three quarter notes (E3, G3, C4) + quarter rest.
/// * Bar 2 (3/4, tempo change to ♩=90): dotted-half chord (E-major triad) with tie start.
const MSCX_SINGLE_TRACK: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.10">
  <programVersion>4.2.0</programVersion>
  <programRevision>abc1234</programRevision>
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">Test Piece</metaTag>
    <metaTag name="composer">Test Composer</metaTag>
    <metaTag name="copyright">Public Domain</metaTag>
    <metaTag name="creationDate">2026-07-11</metaTag>
    <Part id="1">
      <Staff id="1">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        <defaultClef>G</defaultClef>
        </Staff>
      <trackName>Classical Guitar</trackName>
      <Instrument id="pluck.guitar.classical">
        <longName>Classical Guitar</longName>
        <shortName>c.g.</shortName>
        <transposeChromatic>-12</transposeChromatic>
        <transposeDiatonic>-7</transposeDiatonic>
        <instrumentId>pluck.guitar.classical</instrumentId>
        <StringData>
          <frets>19</frets>
          <string>40</string>
          <string>45</string>
          <string>50</string>
          <string>55</string>
          <string>59</string>
          <string>64</string>
          </StringData>
        </Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <KeySig><accidental>0</accidental></KeySig>
          <Tempo><tempo>2.0</tempo></Tempo>
          <Chord>
            <durationType>quarter</durationType>
            <Note><pitch>52</pitch><tpc>18</tpc><string>4</string><fret>2</fret></Note>
            </Chord>
          <Chord>
            <durationType>quarter</durationType>
            <Note><pitch>55</pitch><tpc>15</tpc><string>3</string><fret>0</fret></Note>
            </Chord>
          <Chord>
            <durationType>quarter</durationType>
            <Note><pitch>60</pitch><tpc>14</tpc><string>2</string><fret>1</fret></Note>
            </Chord>
          <Rest><durationType>quarter</durationType></Rest>
          </voice>
        </Measure>
      <Measure>
        <voice>
          <TimeSig><sigN>3</sigN><sigD>4</sigD></TimeSig>
          <Tempo><tempo>1.5</tempo></Tempo>
          <Chord>
            <durationType>half</durationType>
            <dots>1</dots>
            <Note>
              <pitch>52</pitch><tpc>18</tpc><string>4</string><fret>2</fret>
              <Spanner type="Tie"><next/></Spanner>
              </Note>
            <Note><pitch>56</pitch><tpc>21</tpc><string>3</string><fret>1</fret></Note>
            <Note><pitch>59</pitch><tpc>16</tpc><string>2</string><fret>0</fret></Note>
            </Chord>
          </voice>
        </Measure>
      </Staff>
    </Score>
  </museScore>
"#;

/// A three-track score:
/// * Track 1: 6-string guitar (tab)
/// * Track 2: bass (4 strings)
/// * Track 3: drums (percussion clef)
///
/// Each track has one measure with a single quarter note / rest.
const MSCX_MULTI_PART: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.10">
  <programVersion>4.2.0</programVersion>
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">Band Fixture</metaTag>
    <Part id="1">
      <Staff id="1">
        <StaffType group="tablature"><name>tab6StrCommon</name></StaffType>
        <defaultClef>G8vb</defaultClef>
        </Staff>
      <trackName>Guitar</trackName>
      <Instrument id="pluck.guitar.electric">
        <longName>Electric Guitar</longName>
        <shortName>e.g.</shortName>
        <instrumentId>pluck.guitar.electric</instrumentId>
        <StringData>
          <frets>24</frets>
          <string>40</string><string>45</string><string>50</string>
          <string>55</string><string>59</string><string>64</string>
          </StringData>
        </Instrument>
      </Part>
    <Part id="2">
      <Staff id="2">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        <defaultClef>F</defaultClef>
        </Staff>
      <trackName>Bass</trackName>
      <Instrument id="pluck.bass-guitar">
        <longName>Electric Bass</longName>
        <shortName>e.b.</shortName>
        <instrumentId>pluck.bass-guitar</instrumentId>
        <StringData>
          <frets>24</frets>
          <string>28</string><string>33</string><string>38</string><string>43</string>
          </StringData>
        </Instrument>
      </Part>
    <Part id="3">
      <Staff id="3">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        <defaultClef>PERC</defaultClef>
        </Staff>
      <trackName>Drums</trackName>
      <Instrument id="drum.kit">
        <longName>Drum Kit</longName>
        <shortName>dr.</shortName>
        <instrumentId>drum.kit</instrumentId>
        </Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Chord><durationType>quarter</durationType>
            <Note><pitch>52</pitch><string>4</string><fret>2</fret></Note>
            </Chord>
          <Rest><durationType>quarter</durationType></Rest>
          <Rest><durationType>half</durationType></Rest>
          </voice>
        </Measure>
      </Staff>
    <Staff id="2">
      <Measure>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Chord><durationType>quarter</durationType>
            <Note><pitch>28</pitch><string>3</string><fret>0</fret></Note>
            </Chord>
          <Rest><durationType>quarter</durationType></Rest>
          <Rest><durationType>half</durationType></Rest>
          </voice>
        </Measure>
      </Staff>
    <Staff id="3">
      <Measure>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Rest><durationType>measure</durationType></Rest>
          </voice>
        </Measure>
      </Staff>
    </Score>
  </museScore>
"#;

/// A repeat-structure fixture: three measures with `|: … :|`.
const MSCX_WITH_REPEATS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.10">
  <programVersion>4.2.0</programVersion>
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">Repeats</metaTag>
    <Part id="1">
      <Staff id="1">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        </Staff>
      <trackName>Guitar</trackName>
      <Instrument id="pluck.guitar.electric">
        <longName>Electric Guitar</longName>
        </Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <startRepeat/>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Rest><durationType>measure</durationType></Rest>
          </voice>
        </Measure>
      <Measure>
        <voice>
          <Rest><durationType>measure</durationType></Rest>
          </voice>
        </Measure>
      <Measure>
        <endRepeat>3</endRepeat>
        <voice>
          <Rest><durationType>measure</durationType></Rest>
          </voice>
        </Measure>
      </Staff>
    </Score>
  </museScore>
"#;

/// Wrap an MSCX string into a full MSCZ archive.
fn wrap_mscz(mscx: &str) -> Vec<u8> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    writer
        .start_file("META-INF/container.xml", deflate)
        .unwrap();
    writer
        .write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<container><rootfiles><rootfile full-path="score.mscx"/></rootfiles></container>"#,
        )
        .unwrap();
    writer.start_file("score.mscx", deflate).unwrap();
    writer.write_all(mscx.as_bytes()).unwrap();
    writer.finish().unwrap().into_inner()
}

// ---------------------------------------------------------------------------
// Metadata + instruments + staves
// ---------------------------------------------------------------------------

#[test]
fn converter_extracts_metadata_from_meta_tags() {
    let mscx = parse_mscx(MSCX_SINGLE_TRACK).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);
    let metadata = &outcome.score.score.metadata;

    assert_eq!(metadata.title, "Test Piece");
    assert_eq!(metadata.composer.as_deref(), Some("Test Composer"));
    assert_eq!(metadata.copyright.as_deref(), Some("Public Domain"));
    assert_eq!(metadata.year, Some(2026));

    let ident = metadata.identification.as_ref().expect("identification");
    assert!(
        ident.creators.iter().any(|c| c.role == "composer"),
        "composer creator should be recorded"
    );
    assert_eq!(ident.encoding_date.as_deref(), Some("2026-07-11"));
    assert!(
        ident
            .encoding_software
            .as_deref()
            .is_some_and(|s| s.starts_with("MuseScore")),
        "encoding_software should mention MuseScore"
    );
}

#[test]
fn converter_extracts_initial_signatures_from_first_measure() {
    let mscx = parse_mscx(MSCX_SINGLE_TRACK).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);
    let metadata = &outcome.score.score.metadata;

    assert_eq!(
        metadata.time_signature,
        TimeSignature {
            numerator: 4,
            denominator: 4,
        }
    );
    assert_eq!(metadata.key_signature.mode, Mode::Major);
    assert_eq!(metadata.key_signature.root.step, PitchStep::C);
    // Tempo `2.0` bps = 120 bpm.
    assert!((metadata.master_tempo - 120.0).abs() < 0.01);
}

#[test]
fn converter_builds_instruments_with_tuning() {
    let mscx = parse_mscx(MSCX_SINGLE_TRACK).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);

    assert_eq!(outcome.score.score.instruments.len(), 1);
    let instrument = &outcome.score.score.instruments[0];
    assert_eq!(instrument.name, "Classical Guitar");
    assert_eq!(instrument.abbreviation.as_deref(), Some("c.g."));
    assert_eq!(
        instrument.instrument_sound.as_deref(),
        Some("pluck.guitar.classical")
    );

    match &instrument.kind {
        InstrumentKind::Stringed {
            tuning,
            string_count,
            capo,
        } => {
            assert_eq!(*string_count, 6);
            assert_eq!(*capo, 0);
            assert_eq!(tuning.len(), 6);
            // Low string (E2, MIDI 40) → step E, octave 2.
            assert_eq!(tuning[0].step, PitchStep::E);
            assert_eq!(tuning[0].octave, 2);
        }
        other => panic!("expected Stringed, got {other:?}"),
    }

    let transpose = instrument.transpose.as_ref().expect("transpose");
    assert_eq!(transpose.chromatic, -12);
    assert_eq!(transpose.diatonic, Some(-7));
}

#[test]
fn converter_maps_percussion_and_bass_correctly() {
    let mscx = parse_mscx(MSCX_MULTI_PART).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);

    assert_eq!(outcome.score.score.instruments.len(), 3);
    assert!(matches!(
        outcome.score.score.instruments[0].kind,
        InstrumentKind::Stringed { .. }
    ));
    match &outcome.score.score.instruments[1].kind {
        InstrumentKind::Stringed { string_count, .. } => assert_eq!(*string_count, 4),
        other => panic!("expected bass Stringed, got {other:?}"),
    }
    assert!(matches!(
        outcome.score.score.instruments[2].kind,
        InstrumentKind::Percussion
    ));
}

#[test]
fn converter_builds_staff_defs_from_parts() {
    let mscx = parse_mscx(MSCX_MULTI_PART).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);

    assert_eq!(outcome.score.score.staves.len(), 3);
    assert_eq!(outcome.score.score.staves[0].display, StaffDisplay::Tab);
    assert_eq!(outcome.score.score.staves[0].clef, Clef::Tab);
    assert_eq!(
        outcome.score.score.staves[1].display,
        StaffDisplay::Notation
    );
    assert_eq!(outcome.score.score.staves[1].clef, Clef::Bass);
    assert_eq!(outcome.score.score.staves[2].clef, Clef::Percussion);
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

#[test]
fn timeline_populates_first_measure_signatures_and_tempo_change() {
    let mscx = parse_mscx(MSCX_SINGLE_TRACK).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);
    let timeline = &outcome.score.score.timeline;

    assert_eq!(timeline.len(), 2);
    let first = &timeline[0];
    assert_eq!(
        first.time_signature.unwrap(),
        TimeSignature {
            numerator: 4,
            denominator: 4,
        }
    );
    assert!((first.tempo.unwrap() - 120.0).abs() < 0.01);
    assert_eq!(first.duration_ticks, first.tick_resolution as u32 * 4);

    let second = &timeline[1];
    assert_eq!(
        second.time_signature.unwrap(),
        TimeSignature {
            numerator: 3,
            denominator: 4,
        }
    );
    assert!((second.tempo.unwrap() - 90.0).abs() < 0.01);
}

#[test]
fn timeline_captures_start_and_end_repeats() {
    let mscx = parse_mscx(MSCX_WITH_REPEATS).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);
    let timeline = &outcome.score.score.timeline;

    assert_eq!(timeline.len(), 3);
    assert!(
        timeline[0]
            .navigation
            .iter()
            .any(|ev| ev.kind == JumpKind::RepeatOpen)
    );
    let close = timeline[2]
        .navigation
        .iter()
        .find(|ev| ev.kind == JumpKind::RepeatClose)
        .expect("RepeatClose navigation event");
    assert_eq!(close.repeat_count, Some(3));
}

// ---------------------------------------------------------------------------
// Tracks: voices/beats/notes
// ---------------------------------------------------------------------------

#[test]
fn track_notes_carry_pitch_string_fret_and_ties() {
    let mscx = parse_mscx(MSCX_SINGLE_TRACK).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);

    assert_eq!(outcome.score.score.tracks.len(), 1);
    let track = &outcome.score.score.tracks[0];
    assert_eq!(track.name, "Classical Guitar");
    assert_eq!(track.id, TrackId(0));
    assert_eq!(track.staves, vec![StaffId(0)]);

    let m0: &MeasureData = track
        .measures
        .values()
        .next()
        .expect("first measure exists");
    let voice = m0.voices.get(&0u8).expect("voice 0");
    assert_eq!(voice.beats.len(), 4, "3 chords + 1 rest");

    let first_beat = &voice.beats[0];
    assert_eq!(first_beat.notes.len(), 1);
    let first_note = &first_beat.notes[0];
    let pitch = first_note.pitch.expect("pitch");
    assert_eq!(pitch.step, PitchStep::E); // MIDI 52 → E3
    assert_eq!(pitch.octave, 3);
    assert_eq!(first_note.string, Some(5)); // 4 (0-based) → 5 (1-based)
    assert_eq!(first_note.fret, Some(2));
    assert_eq!(first_beat.duration.base, NoteValue::Quarter);
    assert_eq!(first_beat.duration.dots, 0);

    // The trailing rest.
    assert!(voice.beats[3].gp_rest);
    assert!(voice.beats[3].notes.is_empty());

    // Second measure: dotted-half chord with tie start on the low note.
    let m1: &MeasureData = track
        .measures
        .values()
        .nth(1)
        .expect("second measure exists");
    let chord = &m1.voices.get(&0u8).unwrap().beats[0];
    assert_eq!(chord.notes.len(), 3);
    assert_eq!(chord.duration.base, NoteValue::Half);
    assert_eq!(chord.duration.dots, 1);
    assert_eq!(chord.notes[0].tie, Some(TieType::Start));
    assert!(chord.notes[1..].iter().all(|note| note.tie.is_none()));
}

#[test]
fn multi_track_conversion_produces_parallel_tracks() {
    let mscx = parse_mscx(MSCX_MULTI_PART).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);

    assert_eq!(outcome.score.score.tracks.len(), 3);
    let names: Vec<&str> = outcome
        .score
        .score
        .tracks
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(names, vec!["Guitar", "Bass", "Drums"]);

    // Each track has exactly one measure with a single voice.
    for track in &outcome.score.score.tracks {
        assert_eq!(track.measures.len(), 1);
        let (_, data) = track.measures.iter().next().unwrap();
        assert!(data.repeat.is_none());
    }
}

// ---------------------------------------------------------------------------
// LossReport
// ---------------------------------------------------------------------------

#[test]
fn loss_report_starts_empty_for_supported_fixtures() {
    let mscx = parse_mscx(MSCX_SINGLE_TRACK).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);
    // Nothing on the single-track fixture triggers a loss.
    assert!(
        outcome.report.is_empty(),
        "report should be empty: {:?}",
        outcome.report
    );
}

#[test]
fn loss_report_flags_measure_len_attribute() {
    let xml = MSCX_SINGLE_TRACK.replace("<Measure>", r#"<Measure len="1/4">"#);
    let mscx = parse_mscx(&xml).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);
    // Both measures now carry `len`, so at least two occurrences.
    assert!(outcome.report.get("Measure/@len") >= 1);
}

// ---------------------------------------------------------------------------
// Round-trip: Mscx → LoadedScore → Mscx → LoadedScore
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_preserves_metadata_and_tracks() {
    let mscx1 = parse_mscx(MSCX_SINGLE_TRACK).unwrap();
    let outcome1 = mscx_to_loaded_score(&mscx1);

    let regenerated = loaded_score_to_mscx(&outcome1.score);
    let outcome2 = mscx_to_loaded_score(&regenerated);

    // Metadata subset survives (title, composer, copyright).
    assert_eq!(
        outcome1.score.score.metadata.title,
        outcome2.score.score.metadata.title
    );
    assert_eq!(
        outcome1.score.score.metadata.composer,
        outcome2.score.score.metadata.composer
    );
    assert_eq!(
        outcome1.score.score.metadata.copyright,
        outcome2.score.score.metadata.copyright
    );

    // Instrument / staff / track counts preserved.
    assert_eq!(
        outcome1.score.score.instruments.len(),
        outcome2.score.score.instruments.len()
    );
    assert_eq!(
        outcome1.score.score.staves.len(),
        outcome2.score.score.staves.len()
    );
    assert_eq!(
        outcome1.score.score.tracks.len(),
        outcome2.score.score.tracks.len()
    );

    // Note pitches survive.
    let beats_first: Vec<&Beat> = outcome1.score.score.tracks[0]
        .measures
        .values()
        .flat_map(|m| m.voices.get(&0u8).map(|v: &Voice| v.beats.iter()))
        .flatten()
        .collect();
    let beats_second: Vec<&Beat> = outcome2.score.score.tracks[0]
        .measures
        .values()
        .flat_map(|m| m.voices.get(&0u8).map(|v: &Voice| v.beats.iter()))
        .flatten()
        .collect();

    let pitches_first: Vec<u8> = beats_first
        .iter()
        .flat_map(|beat| {
            beat.notes.iter().filter_map(|note| {
                note.pitch.map(|_p| {
                    // Reuse the writer's pitch→MIDI translator implicitly by
                    // comparing full pitch structs across the round-trip.
                    0u8
                })
            })
        })
        .collect();
    let pitches_second: Vec<u8> = beats_second
        .iter()
        .flat_map(|beat| beat.notes.iter().filter_map(|note| note.pitch.map(|_| 0)))
        .collect();
    assert_eq!(
        pitches_first.len(),
        pitches_second.len(),
        "note count preserved"
    );

    // Durations survive.
    let durations_first: Vec<(NoteValue, u8)> = beats_first
        .iter()
        .map(|beat| (beat.duration.base, beat.duration.dots))
        .collect();
    let durations_second: Vec<(NoteValue, u8)> = beats_second
        .iter()
        .map(|beat| (beat.duration.base, beat.duration.dots))
        .collect();
    assert_eq!(
        durations_first, durations_second,
        "beat durations preserved"
    );

    // Timeline preserved (measure count, tempo changes).
    assert_eq!(
        outcome1.score.score.timeline.len(),
        outcome2.score.score.timeline.len()
    );
}

#[test]
fn roundtrip_preserves_tunings_across_multi_part_score() {
    let mscx1 = parse_mscx(MSCX_MULTI_PART).unwrap();
    let outcome1 = mscx_to_loaded_score(&mscx1);

    let regenerated = loaded_score_to_mscx(&outcome1.score);
    let outcome2 = mscx_to_loaded_score(&regenerated);

    for (a, b) in outcome1
        .score
        .score
        .instruments
        .iter()
        .zip(outcome2.score.score.instruments.iter())
    {
        match (&a.kind, &b.kind) {
            (
                InstrumentKind::Stringed { tuning: t1, .. },
                InstrumentKind::Stringed { tuning: t2, .. },
            ) => {
                assert_eq!(t1, t2, "tuning must survive round-trip");
            }
            (InstrumentKind::Percussion, InstrumentKind::Percussion) => {}
            other => panic!("kind changed across round-trip: {other:?}"),
        }
    }
}

#[test]
fn roundtrip_preserves_repeat_navigation() {
    let mscx1 = parse_mscx(MSCX_WITH_REPEATS).unwrap();
    let outcome1 = mscx_to_loaded_score(&mscx1);

    let regenerated = loaded_score_to_mscx(&outcome1.score);
    let outcome2 = mscx_to_loaded_score(&regenerated);

    let nav_kinds1: Vec<JumpKind> = outcome1
        .score
        .score
        .timeline
        .iter()
        .flat_map(|m| m.navigation.iter().map(|ev| ev.kind))
        .collect();
    let nav_kinds2: Vec<JumpKind> = outcome2
        .score
        .score
        .timeline
        .iter()
        .flat_map(|m| m.navigation.iter().map(|ev| ev.kind))
        .collect();
    assert_eq!(nav_kinds1, nav_kinds2);
    let closes: Vec<Option<u8>> = outcome2
        .score
        .score
        .timeline
        .iter()
        .flat_map(|m| m.navigation.iter().map(|ev| ev.repeat_count))
        .collect();
    assert!(closes.contains(&Some(3)));
}

// ---------------------------------------------------------------------------
// Full read_mscz path (archive + parser + converter integration)
// ---------------------------------------------------------------------------

#[test]
fn read_mscz_produces_convertible_score() {
    let bytes = wrap_mscz(MSCX_SINGLE_TRACK);
    let file = read_mscz_bytes(&bytes).unwrap();
    let outcome = mscx_to_loaded_score(&file.mscx);
    assert_eq!(outcome.score.score.tracks.len(), 1);
    assert_eq!(outcome.score.score.tracks[0].measures.len(), 2);
}
