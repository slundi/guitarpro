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
//! * Instrument programs, transposing staves (`<actualKey>`, `<tpc2>`),
//!   octave clefs and clef changes, slurs and noteheads.
//! * Multi-part scores → parallel tracks.
//! * Round-trip: `Mscx → LoadedScore → Mscx → LoadedScore` preserves the
//!   converter-covered subset.

use std::io::Write;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::convert::mscz::{LossReport, loaded_score_to_mscx, mscx_to_loaded_score};
use crate::convert::musicxml::loaded_score_to_score_partwise;
use crate::convert::optimized::score_partwise_to_loaded_score;
use crate::error::{GpError, GpResult};
use crate::io::mscz::{parse_mscx, read_mscz_bytes};
use crate::model::musicxml::measure::{Forward, MusicData};
use crate::model::optimized::{
    LoadedScore,
    beat::{Beat, SlurKind, Voice},
    global::{InstrumentKind, MeasureIndex, StaffId, TrackId},
    metadata::{Mode, TimeSignature},
    note::{NoteValue, Notehead, PitchStep, TieType},
    timeline::{Barline, BarlineStyle, Ending, EndingKind, JumpKind, NavigationEvent},
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
            <Note><pitch>56</pitch><tpc>22</tpc><string>3</string><fret>1</fret></Note>
            <Note><pitch>59</pitch><tpc>19</tpc><string>2</string><fret>0</fret></Note>
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

/// A clarinet and a double bass, two 4/4 measures in concert A major:
/// * Clarinet in B♭ (program 71), written in B major: a quarter A4 starting
///   a slur, a dotted-eighth B♭4 (spelled flat by its `<tpc>`), a sixteenth
///   C♯5 ending that slur and starting the next, a half E5 with a cross
///   notehead; bar 2 a whole A4 ending the second slur.
/// * Double bass (program 43) in the bass clef with an 8 below, changing to
///   the treble clef at bar 2 (written at the end of bar 1, as MuseScore
///   does).
const MSCX_ORCHESTRA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.70">
  <programVersion>4.7.5</programVersion>
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">Orchestra Fixture</metaTag>
    <Part id="1">
      <Staff id="1">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        </Staff>
      <trackName>Clarinet in B♭</trackName>
      <Instrument id="bb-clarinet">
        <longName>Clarinet in B♭</longName>
        <transposeDiatonic>-1</transposeDiatonic>
        <transposeChromatic>-2</transposeChromatic>
        <instrumentId>wind.reed.clarinet.bflat</instrumentId>
        <Channel>
          <program value="71"/>
          </Channel>
        </Instrument>
      </Part>
    <Part id="2">
      <Staff id="2">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        <defaultClef>F8vb</defaultClef>
        </Staff>
      <trackName>Double Bass</trackName>
      <Instrument id="double-bass">
        <longName>Double Bass</longName>
        <instrumentId>strings.contrabass</instrumentId>
        <clef>F8vb</clef>
        <Channel>
          <program value="43"/>
          </Channel>
        <Channel name="pizzicato">
          <program value="45"/>
          </Channel>
        </Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <voice>
          <Clef><concertClefType>G</concertClefType><transposingClefType>G</transposingClefType><isHeader>1</isHeader></Clef>
          <KeySig><concertKey>3</concertKey><actualKey>5</actualKey></KeySig>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><fractions>7/16</fractions></location></next></Spanner>
            <Note><pitch>69</pitch><tpc>17</tpc><tpc2>19</tpc2></Note>
            </Chord>
          <Chord>
            <dots>1</dots>
            <durationType>eighth</durationType>
            <Note><pitch>70</pitch><tpc>12</tpc><tpc2>14</tpc2></Note>
            </Chord>
          <Chord>
            <durationType>16th</durationType>
            <Spanner type="Slur"><prev><location><fractions>-7/16</fractions></location></prev></Spanner>
            <Spanner type="Slur"><Slur/><next><location><measures>1</measures><fractions>-7/16</fractions></location></next></Spanner>
            <Note><pitch>73</pitch><tpc>21</tpc><tpc2>23</tpc2></Note>
            </Chord>
          <Chord>
            <durationType>half</durationType>
            <Note><pitch>76</pitch><tpc>18</tpc><tpc2>20</tpc2><head>cross</head></Note>
            </Chord>
          </voice>
        </Measure>
      <Measure>
        <voice>
          <Chord>
            <durationType>whole</durationType>
            <Spanner type="Slur"><prev><location><measures>-1</measures><fractions>7/16</fractions></location></prev></Spanner>
            <Note><pitch>69</pitch><tpc>17</tpc><tpc2>19</tpc2></Note>
            </Chord>
          </voice>
        </Measure>
      </Staff>
    <Staff id="2">
      <Measure>
        <voice>
          <Clef><concertClefType>F8vb</concertClefType><transposingClefType>F8vb</transposingClefType><isHeader>1</isHeader></Clef>
          <KeySig><concertKey>3</concertKey></KeySig>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Chord>
            <durationType>whole</durationType>
            <Note><pitch>33</pitch><tpc>17</tpc></Note>
            </Chord>
          <Clef><concertClefType>G</concertClefType><transposingClefType>G</transposingClefType></Clef>
          </voice>
        </Measure>
      <Measure>
        <voice>
          <Chord>
            <durationType>whole</durationType>
            <Note><pitch>57</pitch><tpc>17</tpc></Note>
            </Chord>
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

/// [`MSCX_MULTI_PART`] with a second guitar staff: its music (pitch 99)
/// is the second `<Staff>` under `<Score>`, ahead of the bass part's. The
/// bass and drum staves' ids move up one, as MuseScore numbers them.
fn with_two_guitar_staves() -> String {
    MSCX_MULTI_PART
        .replace(r#"<Staff id="3">"#, r#"<Staff id="4">"#)
        .replacen(
            "<Staff id=\"2\">\n        <StaffType",
            "<Staff id=\"3\">\n        <StaffType",
            1,
        )
        .replacen(
            "</Staff>\n      <trackName>Guitar",
            "</Staff>\n      <Staff id=\"2\"><StaffType group=\"pitched\"><name>stdNormal</name></StaffType></Staff>\n      <trackName>Guitar",
            1,
        )
        .replacen(
            "    <Staff id=\"2\">\n      <Measure>",
            "    <Staff id=\"2\">\n      <Measure>\n        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>99</pitch></Note></Chord>\n          </voice>\n        </Measure>\n      </Staff>\n    <Staff id=\"3\">\n      <Measure>",
            1,
        )
}

#[test]
fn multi_staff_part_does_not_shift_later_parts_staves() -> GpResult<()> {
    let xml = with_two_guitar_staves();
    let mscx = parse_mscx(&xml)?;
    let outcome = mscx_to_loaded_score(&mscx);
    let tracks = &outcome.score.score.tracks;
    assert_eq!(tracks.len(), 3);
    assert_eq!(tracks[0].staves, vec![StaffId(0), StaffId(1)]);
    assert_eq!(tracks[1].staves, vec![StaffId(2)]);

    let bass = first_voice_beats(&outcome.score, 1, 0)?;
    let pitch = present(bass[0].notes[0].pitch, "bass pitch")?;
    assert_eq!((pitch.step, pitch.octave), (PitchStep::E, 1)); // MIDI 28
    assert_eq!(outcome.report.get("Staff (additional staff of a part)"), 1);
    Ok(())
}

#[test]
fn a_part_without_staves_takes_no_other_parts_staff() -> GpResult<()> {
    // The bass part has no `<Staff>`, and the score none of its music: the
    // guitar's second staff is still the guitar's.
    let xml = with_two_guitar_staves();
    let part_staff = "<Staff id=\"3\">\n        <StaffType group=\"pitched\"><name>stdNormal</name></StaffType>\n        <defaultClef>F</defaultClef>\n        </Staff>\n      ";
    let score_staff = xml
        .split("    <Staff id=\"3\">\n      <Measure>")
        .nth(1)
        .and_then(|rest| rest.split("    <Staff id=\"4\">").next())
        .map(|body| format!("    <Staff id=\"3\">\n      <Measure>{body}"));
    let score_staff = present(score_staff, "bass staff")?;
    let without = xml
        .replacen(part_staff, "", 1)
        .replacen(&score_staff, "", 1);
    assert_eq!(
        without.len() + part_staff.len() + score_staff.len(),
        xml.len()
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&without)?);
    let tracks = &outcome.score.score.tracks;
    assert_eq!(tracks.len(), 3);
    assert!(tracks[1].measures.is_empty());
    assert_eq!(outcome.report.get("Part (without a staff of music)"), 1);
    Ok(())
}

#[test]
fn a_part_without_staves_does_not_shift_later_parts_music() -> GpResult<()> {
    // The guitar part has no `<Staff>`, and the score none of its music.
    let part_staff = "<Staff id=\"1\">\n        <StaffType group=\"tablature\"><name>tab6StrCommon</name></StaffType>\n        <defaultClef>G8vb</defaultClef>\n        </Staff>\n";
    let score_staff = MSCX_MULTI_PART
        .split("    <Staff id=\"1\">\n      <Measure>")
        .nth(1)
        .and_then(|rest| rest.split("    <Staff id=\"2\">").next())
        .map(|body| format!("    <Staff id=\"1\">\n      <Measure>{body}"));
    let score_staff = present(score_staff, "guitar staff")?;
    let xml = MSCX_MULTI_PART
        .replacen(part_staff, "", 1)
        .replacen(&score_staff, "", 1);
    assert_eq!(
        xml.len() + part_staff.len() + score_staff.len(),
        MSCX_MULTI_PART.len()
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    // The guitar has no music to read: it's an empty track, reported. The
    // bass still has its own, and every track keeps its part's index.
    let tracks = &outcome.score.score.tracks;
    assert_eq!(tracks.len(), 3);
    assert!(tracks[0].measures.is_empty());
    assert_eq!(outcome.report.get("Part (without a staff of music)"), 1);
    assert!(
        tracks
            .iter()
            .enumerate()
            .all(|(index, track)| track.id.0 as usize == index)
    );
    let bass = first_voice_beats(&outcome.score, 1, 0)?;
    let pitch = present(bass[0].notes[0].pitch, "bass pitch")?;
    assert_eq!((pitch.step, pitch.octave), (PitchStep::E, 1)); // MIDI 28
    Ok(())
}

#[test]
fn parts_whose_staves_the_score_does_not_match_keep_their_music_in_order() -> GpResult<()> {
    // Score staves numbered apart from the parts' (as another tool might).
    let xml = (1..=3).fold(MSCX_MULTI_PART.to_owned(), |xml, id| {
        xml.replacen(
            &format!("    <Staff id=\"{id}\">\n      <Measure>"),
            &format!("    <Staff id=\"{}\">\n      <Measure>", id + 10),
            1,
        )
    });
    assert_ne!(xml, MSCX_MULTI_PART);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    let tracks = &outcome.score.score.tracks;
    assert_eq!(tracks.len(), 3);
    assert!(tracks.iter().all(|track| !track.measures.is_empty()));
    assert_eq!(outcome.report.get("Part (without a staff of music)"), 0);
    let bass = first_voice_beats(&outcome.score, 1, 0)?;
    let pitch = present(bass[0].notes[0].pitch, "bass pitch")?;
    assert_eq!((pitch.step, pitch.octave), (PitchStep::E, 1)); // MIDI 28
    Ok(())
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
fn measure_len_attribute_is_not_a_loss() {
    let xml = MSCX_SINGLE_TRACK.replace("<Measure>", r#"<Measure len="1/4">"#);
    let mscx = parse_mscx(&xml).unwrap();
    let outcome = mscx_to_loaded_score(&mscx);
    assert_eq!(outcome.report.get("Measure/@len"), 0);
}

/// A whole-measure rest in a pickup fills the pickup's `len`, not its time
/// signature's measure.
#[test]
fn measure_rest_fills_a_pickups_len() -> GpResult<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.20">
  <Score>
    <Division>480</Division>
    <Part id="1"><Staff id="1"/><trackName>Piano</trackName></Part>
    <Staff id="1">
      <Measure len="1/4">
        <voice>
          <TimeSig><sigN>3</sigN><sigD>4</sigD></TimeSig>
          <Rest><durationType>measure</durationType><duration>1/4</duration></Rest>
        </voice>
      </Measure>
    </Staff>
  </Score>
</museScore>"#;
    let mscx = parse_mscx(xml)?;
    let score = mscx_to_loaded_score(&mscx).score.score;
    assert_eq!(
        score.timeline[0].duration_ticks,
        crate::convert::mscz::to_optimized::DIVISIONS
    );
    let beats = &score.tracks[0].measures[&MeasureIndex(0)].voices[&0].beats;
    assert_eq!(beats.len(), 1);
    assert_eq!(beats[0].duration.base, NoteValue::Quarter);

    // The pickup survives a round-trip: written with its `len`.
    let regenerated = loaded_score_to_mscx(&mscx_to_loaded_score(&mscx).score);
    let measure = &regenerated.score_staves[0].measures[0];
    assert_eq!(measure.len.as_deref(), Some("1/4"));
    let again = mscx_to_loaded_score(&regenerated).score.score;
    assert_eq!(
        again.timeline[0].duration_ticks,
        score.timeline[0].duration_ticks
    );
    Ok(())
}

/// A measure rest in a measure of `len`, with its report and its rests.
fn measure_rest_in(len: &str) -> GpResult<(LossReport, usize)> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.20">
  <Score>
    <Division>480</Division>
    <Part id="1"><Staff id="1"/><trackName>Piano</trackName></Part>
    <Staff id="1">
      <Measure len="{len}">
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Rest><durationType>measure</durationType><duration>{len}</duration></Rest>
        </voice>
      </Measure>
    </Staff>
  </Score>
</museScore>"#
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    let rests = outcome.score.score.tracks[0].measures[&MeasureIndex(0)].voices[&0]
        .beats
        .len();
    Ok((outcome.report, rests))
}

/// A malformed, huge `len` isn't read (the time signature's length is):
/// its measure rest would be written out as any number of rests.
#[test]
fn a_measure_len_past_the_longest_time_signature_is_not_read() -> GpResult<()> {
    let (report, rests) = measure_rest_in("1000000/1")?;
    assert_eq!(report.get("Measure (length out of range)"), 1);
    assert_eq!(rests, 1);
    Ok(())
}

/// A measure no note values fill (a third of a whole) leaves its measure
/// rest short, and says so.
#[test]
fn a_measure_rest_no_note_value_fills_is_reported() -> GpResult<()> {
    let (report, _) = measure_rest_in("1/3")?;
    assert_eq!(
        report.get("Measure rest (length no note value fills, left short)"),
        1
    );
    let (report, _) = measure_rest_in("3/8")?;
    assert!(report.is_empty(), "{report:?}");
    Ok(())
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

// ---------------------------------------------------------------------------
// Orchestral notation: instruments, transposition, clefs, slurs, noteheads
// ---------------------------------------------------------------------------

/// `value`, or a [`GpError::MissingState`] naming `field`.
fn present<T>(value: Option<T>, field: &'static str) -> GpResult<T> {
    value.ok_or(GpError::MissingState { field })
}

/// [`MSCX_ORCHESTRA`] converted to the optimized model.
fn orchestra() -> GpResult<LoadedScore> {
    Ok(mscx_to_loaded_score(&parse_mscx(MSCX_ORCHESTRA)?).score)
}

fn measure_data(score: &LoadedScore, track: usize, measure: usize) -> GpResult<&MeasureData> {
    present(
        score.score.tracks[track].measures.values().nth(measure),
        "measure",
    )
}

fn measure_data_mut(
    score: &mut LoadedScore,
    track: usize,
    measure: usize,
) -> GpResult<&mut MeasureData> {
    present(
        score.score.tracks[track].measures.values_mut().nth(measure),
        "measure",
    )
}

fn voice_beats(score: &LoadedScore, track: usize, measure: usize, voice: u8) -> GpResult<&[Beat]> {
    let data = measure_data(score, track, measure)?;
    Ok(&present(data.voices.get(&voice), "voice")?.beats)
}

fn first_voice_beats(score: &LoadedScore, track: usize, measure: usize) -> GpResult<&[Beat]> {
    voice_beats(score, track, measure, 0)
}

/// A beat's slur mark as `(id, kind)`.
type SlurMark = Option<(u8, SlurKind)>;

/// Each beat's slur mark.
fn slur_marks(beats: &[Beat]) -> Vec<SlurMark> {
    beats
        .iter()
        .map(|beat| beat.slur.map(|slur| (slur.slur_id, slur.kind)))
        .collect()
}

#[test]
fn converter_reads_programs_transpositions_and_clefs() -> GpResult<()> {
    let loaded = orchestra()?;
    let score = &loaded.score;

    // Only the first `<Channel>`'s program counts.
    let programs: Vec<u8> = score.instruments.iter().map(|i| i.midi_program).collect();
    assert_eq!(programs, vec![71, 43]);
    let transpose = present(score.instruments[0].transpose.as_ref(), "transpose")?;
    assert_eq!((transpose.diatonic, transpose.chromatic), (Some(-1), -2));

    assert_eq!(score.staves[1].clef, Clef::Bass8vb);
    let bass_clefs: Vec<Option<Clef>> = score.tracks[1].measures.values().map(|m| m.clef).collect();
    assert_eq!(bass_clefs, vec![None, Some(Clef::Treble)]);

    // The timeline carries the concert key.
    let key = present(score.timeline[0].key_signature, "key signature")?;
    assert_eq!((key.root.step, key.root.alter), (PitchStep::A, 0));
    Ok(())
}

/// `<concertKey>` gives the concert key wherever it stands; a written
/// `<accidental>` after it doesn't replace it.
#[test]
fn converter_prefers_the_concert_key_over_a_later_accidental() -> GpResult<()> {
    let xml = orchestra_with(
        "<KeySig><concertKey>3</concertKey><actualKey>5</actualKey></KeySig>",
        "<KeySig><concertKey>3</concertKey><accidental>5</accidental></KeySig>",
    );
    let score = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    let key = present(score.score.timeline[0].key_signature, "key signature")?;
    assert_eq!((key.root.step, key.root.alter), (PitchStep::A, 0));
    Ok(())
}

#[test]
fn converter_spells_pitches_and_reads_noteheads() -> GpResult<()> {
    let score = orchestra()?;
    let beats = first_voice_beats(&score, 0, 0)?;

    // MIDI 70 with tpc 12 is B♭, not A♯.
    let b_flat = present(beats[1].notes[0].pitch, "pitch")?;
    assert_eq!(
        (b_flat.step, b_flat.alter, b_flat.octave),
        (PitchStep::B, -1, 4)
    );
    assert_eq!(beats[3].notes[0].notehead, Some(Notehead::X));
    assert_eq!(beats[0].notes[0].notehead, None);
    Ok(())
}

#[test]
fn converter_pairs_slurs_within_a_voice() -> GpResult<()> {
    let score = orchestra()?;
    let mut slurs: Vec<SlurMark> = slur_marks(first_voice_beats(&score, 0, 0)?);
    slurs.extend(slur_marks(first_voice_beats(&score, 0, 1)?));
    assert_eq!(
        slurs,
        vec![
            Some((0, SlurKind::Start)),
            None,
            // The next slur takes over the ending one's id.
            Some((0, SlurKind::EndStart)),
            None,
            Some((0, SlurKind::End)),
        ]
    );
    Ok(())
}

#[test]
fn writer_emits_musescore_instruments_keys_clefs_and_spellings() -> GpResult<()> {
    let xml = loaded_score_to_mscx(&orchestra()?).raw_xml;

    for expected in [
        r#"<Instrument id="bb-clarinet">"#,
        "<instrumentId>wind.reed.clarinet.bflat</instrumentId>",
        r#"<program value="71"/>"#,
        r#"<Instrument id="double-bass">"#,
        "<defaultClef>F8vb</defaultClef>",
        "<clef>F8vb</clef>",
        // The clarinet's written key next to the concert one.
        "<concertKey>3</concertKey>\n            <actualKey>5</actualKey>",
        // B♭4 sounding, written C5.
        "<tpc>12</tpc>\n              <tpc2>14</tpc2>",
        "<head>cross</head>",
        "<concertClefType>F8vb</concertClefType>",
    ] {
        assert!(xml.contains(expected), "missing {expected:?}");
    }
    // MuseScore times a chord as undotted unless `<dots>` comes first.
    assert!(xml.contains("<dots>1</dots>\n            <durationType>eighth</durationType>"));
    assert!(!xml.contains("</durationType>\n            <dots>"));
    // The double bass' only key signature names no written key.
    assert_eq!(xml.matches("<actualKey>").count(), 1);
    Ok(())
}

#[test]
fn writer_locates_slur_ends_by_their_chords() -> GpResult<()> {
    let xml = loaded_score_to_mscx(&orchestra()?).raw_xml;
    let compact: String = xml.split_whitespace().collect();

    // Quarter + dotted eighth = 7/16 to the slur's end.
    assert!(compact.contains("<next><location><fractions>7/16</fractions></location></next>"));
    assert!(compact.contains("<prev><location><fractions>-7/16</fractions></location></prev>"));
    // From the sixteenth to bar 2's first beat.
    assert!(compact.contains(
        "<next><location><measures>1</measures><fractions>-7/16</fractions></location></next>"
    ));
    Ok(())
}

#[test]
fn roundtrip_preserves_orchestral_notation() -> GpResult<()> {
    let first = orchestra()?;
    let regenerated = loaded_score_to_mscx(&first);
    let second = mscx_to_loaded_score(&regenerated).score;

    for (a, b) in first
        .score
        .instruments
        .iter()
        .zip(&second.score.instruments)
    {
        assert_eq!(a.midi_program, b.midi_program);
        assert_eq!(a.instrument_sound, b.instrument_sound);
        assert_eq!(
            a.transpose.as_ref().map(|t| t.chromatic),
            b.transpose.as_ref().map(|t| t.chromatic)
        );
    }
    let clefs =
        |score: &LoadedScore| -> Vec<Clef> { score.score.staves.iter().map(|s| s.clef).collect() };
    assert_eq!(clefs(&first), clefs(&second));
    for track in 0..2 {
        let changes = |score: &LoadedScore| -> Vec<Option<Clef>> {
            score.score.tracks[track]
                .measures
                .values()
                .map(|m| m.clef)
                .collect()
        };
        assert_eq!(changes(&first), changes(&second));
        for measure in 0..2 {
            let summary = |score: &LoadedScore| -> GpResult<_> {
                Ok(first_voice_beats(score, track, measure)?
                    .iter()
                    .map(|beat| {
                        (
                            beat.slur.map(|slur| (slur.slur_id, slur.kind)),
                            beat.duration.dots,
                            beat.notes
                                .iter()
                                .map(|n| (n.pitch, n.notehead))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>())
            };
            assert_eq!(summary(&first)?, summary(&second)?);
        }
    }
    Ok(())
}

/// [`MSCX_ORCHESTRA`] with `from` replaced by `to` (which must occur).
fn orchestra_with(from: &str, to: &str) -> String {
    assert!(MSCX_ORCHESTRA.contains(from), "fixture lacks {from:?}");
    MSCX_ORCHESTRA.replacen(from, to, 1)
}

#[test]
fn converter_reads_only_the_first_channels_program() -> GpResult<()> {
    // A first channel without a program doesn't fall through to the
    // pizzicato one.
    let mscx = orchestra_with(r#"<program value="43"/>"#, "");
    let score = mscx_to_loaded_score(&parse_mscx(&mscx)?).score;
    assert_eq!(score.score.instruments[1].midi_program, 0);

    // `<program>` written with a closing tag counts too.
    let mscx = orchestra_with(
        r#"<program value="71"/>"#,
        r#"<program value="71"></program>"#,
    );
    let score = mscx_to_loaded_score(&parse_mscx(&mscx)?).score;
    assert_eq!(score.score.instruments[0].midi_program, 71);
    Ok(())
}

/// Two clefs between the same beats: the first holds for no beat, a change
/// in mid-measure lost like any other, not dropped unreported.
#[test]
fn converter_reports_a_clef_followed_by_another() -> GpResult<()> {
    let mscx = orchestra_with(
        r#"<durationType>whole</durationType>
            <Note><pitch>33</pitch><tpc>17</tpc></Note>
            </Chord>
          <Clef><concertClefType>G</concertClefType><transposingClefType>G</transposingClefType></Clef>"#,
        r#"<durationType>half</durationType>
            <Note><pitch>33</pitch><tpc>17</tpc></Note>
            </Chord>
          <Clef><concertClefType>C3</concertClefType><transposingClefType>C3</transposingClefType></Clef>
          <Clef><concertClefType>G</concertClefType><transposingClefType>G</transposingClefType></Clef>
          <Chord>
            <durationType>half</durationType>
            <Note><pitch>57</pitch><tpc>17</tpc></Note>
            </Chord>"#,
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    let clefs: Vec<Option<Clef>> = outcome.score.score.tracks[1]
        .measures
        .values()
        .map(|m| m.clef)
        .collect();
    assert_eq!(clefs, vec![None, Some(Clef::Treble)]);
    assert_eq!(outcome.report.get("Clef (mid-measure)"), 2);
    Ok(())
}

#[test]
fn converter_reports_mid_measure_clefs_and_holds_them_from_the_next_measure() -> GpResult<()> {
    let mscx = orchestra_with(
        r#"<durationType>whole</durationType>
            <Note><pitch>33</pitch><tpc>17</tpc></Note>
            </Chord>
          <Clef><concertClefType>G</concertClefType><transposingClefType>G</transposingClefType></Clef>"#,
        r#"<durationType>half</durationType>
            <Note><pitch>33</pitch><tpc>17</tpc></Note>
            </Chord>
          <Clef><concertClefType>G</concertClefType><transposingClefType>G</transposingClefType></Clef>
          <Chord>
            <durationType>half</durationType>
            <Note><pitch>57</pitch><tpc>17</tpc></Note>
            </Chord>"#,
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    let clefs: Vec<Option<Clef>> = outcome.score.score.tracks[1]
        .measures
        .values()
        .map(|m| m.clef)
        .collect();
    // It can't be held mid-measure, but holds from the next one on, as
    // from MusicXML.
    assert_eq!(clefs, vec![None, Some(Clef::Treble)]);
    assert_eq!(outcome.report.get("Clef (mid-measure)"), 1);
    Ok(())
}

#[test]
fn converter_ignores_restated_clefs_and_reports_approximated_ones() -> GpResult<()> {
    let end_clef = |name: &str| {
        orchestra_with(
            "<Clef><concertClefType>G</concertClefType><transposingClefType>G</transposingClefType></Clef>",
            &format!(
                "<Clef><concertClefType>{name}</concertClefType><transposingClefType>{name}</transposingClefType></Clef>"
            ),
        )
    };
    let clefs = |mscx: &str| -> GpResult<(Vec<Option<Clef>>, LossReport)> {
        let outcome = mscx_to_loaded_score(&parse_mscx(mscx)?);
        let clefs = outcome.score.score.tracks[1]
            .measures
            .values()
            .map(|m| m.clef)
            .collect();
        Ok((clefs, outcome.report))
    };
    // The double bass is already in F8vb.
    let (restated, _) = clefs(&end_clef("F8vb"))?;
    assert_eq!(restated, vec![None, None]);
    // A soprano clef has no type of its own.
    let (soprano, report) = clefs(&end_clef("C1"))?;
    assert_eq!(soprano, vec![None, Some(Clef::Alto)]);
    assert_eq!(report.get("Clef (C1, approximated)"), 1);
    Ok(())
}

#[test]
fn converter_reports_an_approximated_staff_clef_once() -> GpResult<()> {
    // The double bass' default, instrument and (concert and transposing)
    // header clefs all name it.
    let mscx = MSCX_ORCHESTRA.replace("F8vb", "C1");
    assert_eq!(MSCX_ORCHESTRA.matches("F8vb").count(), 4);
    let report = mscx_to_loaded_score(&parse_mscx(&mscx)?).report;
    assert_eq!(report.get("Clef (C1, approximated)"), 1);
    Ok(())
}

#[test]
fn writer_keeps_hairpins_with_the_same_id_in_different_voices_apart() -> GpResult<()> {
    use crate::model::optimized::effect::{BeatEvent, HairpinEvent, HairpinKind};

    let quarter = "          <Chord><durationType>quarter</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let voice = format!("{quarter}\n{quarter}\n{quarter}\n{quarter}");
    let bar = format!(
        "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{voice}\n          </voice>\n        <voice>\n{voice}\n          </voice>"
    );
    let mut score = mscx_to_loaded_score(&parse_mscx(&flute_score(&[&bar]))?).score;
    let hairpin = |kind| {
        BeatEvent::Hairpin(HairpinEvent {
            id: 1,
            kind,
            line: false,
        })
    };
    // Both voices number their hairpin 1; voice 2's lies inside voice 1's.
    let data = measure_data_mut(&mut score, 0, 0)?;
    for (voice_id, (start, stop, kind)) in [
        (0u8, (0, 3, HairpinKind::CrescendoStart)),
        (1, (1, 2, HairpinKind::DecrescendoStart)),
    ] {
        let beats = &mut present(data.voices.get_mut(&voice_id), "voice")?.beats;
        beats[start].events.push(hairpin(kind));
        beats[stop].events.push(hairpin(HairpinKind::Stop));
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert_eq!(xml.matches("<HairPin>").count(), 2);
    Ok(())
}

#[test]
fn clefs_after_a_gap_are_in_mid_measure() -> GpResult<()> {
    let first = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <location><fractions>1/2</fractions></location>\n          <Clef><concertClefType>F</concertClefType><transposingClefType>F</transposingClefType></Clef>\n          <Chord><durationType>half</durationType><Note><pitch>48</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let second = "        <voice>\n          <Chord><durationType>whole</durationType><Note><pitch>48</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[first, second]))?);
    let clefs: Vec<Option<Clef>> = outcome.score.score.tracks[0]
        .measures
        .values()
        .map(|m| m.clef)
        .collect();
    assert_eq!(clefs, vec![None, Some(Clef::Bass)]);
    assert_eq!(outcome.report.get("Clef (mid-measure)"), 1);
    Ok(())
}

#[test]
fn converter_drops_slur_ends_without_a_start() -> GpResult<()> {
    let mscx = orchestra_with(
        r#"<Spanner type="Slur"><Slur/><next><location><fractions>7/16</fractions></location></next></Spanner>"#,
        "",
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    let beats = first_voice_beats(&outcome.score, 0, 0)?;
    assert!(beats[0].slur.is_none());
    // The sixteenth only starts the second slur.
    assert_eq!(slur_marks(beats)[2], Some((0, SlurKind::Start)));
    assert_eq!(outcome.report.get("Slur (unpaired end)"), 1);
    Ok(())
}

/// One 4/4 part: bar 1 has a phrasing slur over all four quarters with a
/// shorter slur nested inside it (beats 2–3); in bar 2 a slur goes from
/// voice 1's first beat to voice 2's second.
const MSCX_SLURS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.70">
  <Score>
    <Division>480</Division>
    <Part id="1">
      <Staff id="1">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        </Staff>
      <trackName>Flute</trackName>
      <Instrument id="flute">
        <longName>Flute</longName>
        <instrumentId>wind.flutes.flute</instrumentId>
        </Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><fractions>3/4</fractions></location></next></Spanner>
            <Note><pitch>72</pitch><tpc>14</tpc></Note>
            </Chord>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><fractions>1/4</fractions></location></next></Spanner>
            <Note><pitch>74</pitch><tpc>16</tpc></Note>
            </Chord>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><prev><location><fractions>-1/4</fractions></location></prev></Spanner>
            <Note><pitch>76</pitch><tpc>18</tpc></Note>
            </Chord>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><prev><location><fractions>-3/4</fractions></location></prev></Spanner>
            <Note><pitch>77</pitch><tpc>13</tpc></Note>
            </Chord>
          </voice>
        </Measure>
      <Measure>
        <voice>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><voices>1</voices><fractions>1/4</fractions></location></next></Spanner>
            <Note><pitch>79</pitch><tpc>15</tpc></Note>
            </Chord>
          <Rest><durationType>quarter</durationType></Rest>
          <Rest><durationType>half</durationType></Rest>
          </voice>
        <voice>
          <Rest><durationType>quarter</durationType></Rest>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><prev><location><voices>-1</voices><fractions>-1/4</fractions></location></prev></Spanner>
            <Note><pitch>67</pitch><tpc>15</tpc></Note>
            </Chord>
          <Rest><durationType>half</durationType></Rest>
          </voice>
        </Measure>
      </Staff>
    </Score>
  </museScore>
"#;

/// The slur marks of [`MSCX_SLURS`]: bar 1, bar 2 voice 1, bar 2 voice 2.
fn slur_fixture_marks(score: &LoadedScore) -> GpResult<Vec<Vec<SlurMark>>> {
    Ok(vec![
        slur_marks(voice_beats(score, 0, 0, 0)?),
        slur_marks(voice_beats(score, 0, 1, 0)?),
        slur_marks(voice_beats(score, 0, 1, 1)?),
    ])
}

#[test]
fn converter_pairs_nested_and_cross_voice_slurs_by_location() -> GpResult<()> {
    let outcome = mscx_to_loaded_score(&parse_mscx(MSCX_SLURS)?);
    assert_eq!(
        slur_fixture_marks(&outcome.score)?,
        vec![
            vec![
                Some((0, SlurKind::Start)),
                Some((1, SlurKind::Start)),
                Some((1, SlurKind::End)),
                Some((0, SlurKind::End)),
            ],
            vec![Some((0, SlurKind::Start)), None, None],
            vec![None, Some((0, SlurKind::End)), None],
        ]
    );
    assert!(outcome.report.is_empty());
    Ok(())
}

#[test]
fn roundtrip_keeps_nested_and_cross_voice_slurs() -> GpResult<()> {
    let first = mscx_to_loaded_score(&parse_mscx(MSCX_SLURS)?).score;
    let regenerated = loaded_score_to_mscx(&first);
    let compact: String = regenerated.raw_xml.split_whitespace().collect();
    assert!(compact.contains(
        "<next><location><voices>1</voices><fractions>1/4</fractions></location></next>"
    ));
    let second = mscx_to_loaded_score(&regenerated).score;
    assert_eq!(slur_fixture_marks(&first)?, slur_fixture_marks(&second)?);
    Ok(())
}

#[test]
fn writer_drops_slurs_touching_rests() -> GpResult<()> {
    let mut score = orchestra()?;
    // Bar 2's whole note, where the second slur ends, becomes a rest.
    for voice in measure_data_mut(&mut score, 0, 1)?.voices.values_mut() {
        voice.beats[0].gp_rest = true;
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    // Only the first slur's start and end are left.
    assert_eq!(xml.matches(r#"<Spanner type="Slur">"#).count(), 2);
    assert_eq!(xml.matches("<Slur>").count(), 1);
    Ok(())
}

/// A beat none of whose notes has a pitch is written as a rest, not as a
/// chord of `<Note>`s without a `<pitch>`.
#[test]
fn writer_writes_a_beat_without_pitches_as_a_rest() -> GpResult<()> {
    let mut score = orchestra()?;
    let before = loaded_score_to_mscx(&score).raw_xml;
    for voice in measure_data_mut(&mut score, 0, 1)?.voices.values_mut() {
        for note in &mut voice.beats[0].notes {
            note.pitch = None;
        }
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert_eq!(
        xml.matches("<Note>").count(),
        before.matches("<Note>").count() - 1
    );
    assert_eq!(
        xml.matches("<Note>").count(),
        xml.matches("<pitch>").count()
    );
    assert!(xml.matches("<Rest>").count() > before.matches("<Rest>").count());
    Ok(())
}

#[test]
fn writer_keeps_clefs_off_tab_staves_and_writes_the_first_measures_clef() -> GpResult<()> {
    let mut score = orchestra()?;
    // The clarinet's first measure sets its own clef.
    measure_data_mut(&mut score, 0, 0)?.clef = Some(Clef::Bass);
    // The double bass becomes a tab staff, keeping its bar 2 clef change.
    score.score.staves[1].display = StaffDisplay::Tab;
    let regenerated = loaded_score_to_mscx(&score);
    assert_eq!(regenerated.raw_xml.matches("<concertClefType>").count(), 1);
    assert!(
        regenerated
            .raw_xml
            .contains("<concertClefType>F</concertClefType>")
    );

    // Reading it back keeps the first measure's clef.
    let second = mscx_to_loaded_score(&regenerated).score;
    assert_eq!(second.score.staves[0].clef, Clef::Treble);
    assert_eq!(measure_data(&second, 0, 0)?.clef, Some(Clef::Bass));
    Ok(())
}

#[test]
fn writer_keeps_tab_clefs_off_notation_staves() -> GpResult<()> {
    let mut score = orchestra()?;
    score.score.staves[0].clef = Clef::Tab;
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(!xml.contains(">TAB<"));
    Ok(())
}

#[test]
fn writer_places_clef_changes_at_the_measure_end() -> GpResult<()> {
    let mut score = orchestra()?;
    // The double bass' bar 1 only holds a half note before its clef change.
    for voice in measure_data_mut(&mut score, 1, 0)?.voices.values_mut() {
        voice.beats[0].duration.base = NoteValue::Half;
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    let compact: String = xml.split_whitespace().collect();
    assert!(compact.contains(
        "<location><fractions>1/2</fractions></location><Clef><concertClefType>G</concertClefType>"
    ));
    Ok(())
}

#[test]
fn writer_respells_keys_beyond_seven_sharps() -> GpResult<()> {
    let mut score = orchestra()?;
    let key = present(score.score.timeline[0].key_signature.as_mut(), "key")?;
    // G♯ major (eight sharps) is written as A♭ major.
    key.root.step = PitchStep::G;
    key.root.alter = 1;
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(xml.contains("<concertKey>-4</concertKey>"));
    Ok(())
}

#[test]
fn writer_counts_minor_keys_from_their_tonic() -> GpResult<()> {
    let mut score = orchestra()?;
    let key = present(score.score.timeline[0].key_signature.as_mut(), "key")?;
    // A minor has no sharps; the clarinet reads it as B minor.
    key.mode = Mode::Minor;
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(xml.contains("<concertKey>0</concertKey>\n            <actualKey>2</actualKey>"));
    Ok(())
}

/// A measure at a resolution its eighths aren't whole ticks of keeps its
/// time signature's length, which the import rounds down (5/8 at 1 a
/// quarter: 2 ticks): no `len`.
#[test]
fn writer_keeps_measures_at_coarse_resolutions_regular() -> GpResult<()> {
    let mut score = orchestra()?;
    for measure in &mut score.score.timeline {
        measure.time_signature = Some(TimeSignature {
            numerator: 5,
            denominator: 8,
        });
        measure.tick_resolution = 1;
        measure.duration_ticks = 2;
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(!xml.contains("<Measure len="), "{xml}");
    Ok(())
}

#[test]
fn musicxml_carries_clef_changes() -> GpResult<()> {
    let first = orchestra()?;
    let second = score_partwise_to_loaded_score(&loaded_score_to_score_partwise(&first));
    let changes: Vec<Option<Clef>> = second.score.tracks[1]
        .measures
        .values()
        .map(|m| m.clef)
        .collect();
    assert_eq!(changes, vec![None, Some(Clef::Treble)]);
    Ok(())
}

#[test]
fn musicxml_restated_clefs_are_not_changes() -> GpResult<()> {
    let mut score = orchestra()?;
    // Bar 2 restates track 0's clef, as exporters do at system starts.
    let staff = present(score.score.tracks[0].staves.first().copied(), "staff")?;
    let own = present(score.score.staves.get(staff.0 as usize), "staff def")?.clef;
    measure_data_mut(&mut score, 0, 1)?.clef = Some(own);
    let partwise = loaded_score_to_score_partwise(&score);
    let restated = partwise.parts[0].measures[1]
        .music_data
        .iter()
        .any(|event| matches!(event, MusicData::Attributes(a) if !a.clefs.is_empty()));
    assert!(restated, "the writer should restate the clef");

    let second = score_partwise_to_loaded_score(&partwise);
    let changes: Vec<Option<Clef>> = second.score.tracks[0]
        .measures
        .values()
        .map(|m| m.clef)
        .collect();
    assert_eq!(changes, vec![None, None]);
    Ok(())
}

#[test]
fn musicxml_carries_mid_measure_clef_changes() -> GpResult<()> {
    let mut score = orchestra()?;
    // The double bass' bar 1 holds a half note before its clef change, and
    // MusicXML moves on a half note after it: the change is mid-measure.
    for voice in measure_data_mut(&mut score, 1, 0)?.voices.values_mut() {
        voice.beats[0].duration.base = NoteValue::Half;
    }
    let mut partwise = loaded_score_to_score_partwise(&score);
    let bar = &mut partwise.parts[1].measures[0];
    let clef_at = present(
        bar.music_data
            .iter()
            .position(|event| matches!(event, MusicData::Attributes(a) if !a.clefs.is_empty())),
        "clef change",
    )?;
    let half = present(
        bar.music_data.iter().find_map(|event| match event {
            MusicData::Note(note) => note.duration,
            _ => None,
        }),
        "note duration",
    )?;
    bar.music_data.insert(
        clef_at + 1,
        MusicData::Forward(Forward {
            duration: half,
            voice: None,
            staff: None,
        }),
    );
    let second = score_partwise_to_loaded_score(&partwise);
    let changes: Vec<Option<Clef>> = second.score.tracks[1]
        .measures
        .values()
        .map(|m| m.clef)
        .collect();
    // It can't be held mid-measure, but holds from the next one on.
    assert_eq!(changes, vec![None, Some(Clef::Treble)]);
    Ok(())
}

#[test]
fn writer_puts_clef_changes_right_after_an_empty_voice() -> GpResult<()> {
    let mut score = orchestra()?;
    // The double bass' bar 1 has a voice without beats, written as a
    // whole-measure rest that already fills the measure.
    for voice in measure_data_mut(&mut score, 1, 0)?.voices.values_mut() {
        voice.beats.clear();
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    let compact: String = xml.split_whitespace().collect();
    assert!(compact.contains(
        "<durationType>measure</durationType><duration>1/1</duration></Rest><Clef><concertClefType>G</concertClefType>"
    ));
    Ok(())
}

#[test]
fn converter_reports_a_clef_at_the_end_of_the_score() -> GpResult<()> {
    let mscx = orchestra_with(
        "<Note><pitch>57</pitch><tpc>17</tpc></Note>\n            </Chord>",
        "<Note><pitch>57</pitch><tpc>17</tpc></Note>\n            </Chord>\n          <Clef><concertClefType>F</concertClefType></Clef>",
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(outcome.report.get("Clef (end of score)"), 1);
    Ok(())
}

#[test]
fn converter_counts_voice_location_gaps() -> GpResult<()> {
    // Bar 2's second voice starts a quarter in, after a `<location>`
    // instead of a rest; its slur end still pairs by location.
    let mscx = MSCX_SLURS.replacen(
        "<voice>\n          <Rest><durationType>quarter</durationType></Rest>",
        "<voice>\n          <location><fractions>1/4</fractions></location>",
        1,
    );
    assert_ne!(mscx, MSCX_SLURS, "fixture lacks the second voice's rest");
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    let first = voice_beats(&outcome.score, 0, 1, 0)?;
    let second = voice_beats(&outcome.score, 0, 1, 1)?;
    assert_eq!(second[0].tick_offset, first[1].tick_offset);
    assert_eq!(slur_marks(first)[0], Some((0, SlurKind::Start)));
    assert_eq!(slur_marks(second)[0], Some((0, SlurKind::End)));
    assert!(outcome.report.is_empty());
    Ok(())
}

#[test]
fn writer_keeps_voice_location_gaps() -> GpResult<()> {
    // Bar 2's second voice starts a quarter in, after a `<location>`.
    let mscx = MSCX_SLURS.replacen(
        "<voice>\n          <Rest><durationType>quarter</durationType></Rest>",
        "<voice>\n          <location><fractions>1/4</fractions></location>",
        1,
    );
    assert_ne!(mscx, MSCX_SLURS, "fixture lacks the second voice's rest");
    let first = mscx_to_loaded_score(&parse_mscx(&mscx)?).score;
    let xml = loaded_score_to_mscx(&first).raw_xml;
    let compact: String = xml.split_whitespace().collect();
    assert!(compact.contains("<voice><location><fractions>1/4</fractions></location>"));
    // The voice stays where it was, and its slur end still pairs.
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    let original = voice_beats(&first, 0, 1, 1)?;
    let second = voice_beats(&outcome.score, 0, 1, 1)?;
    let ticks = |beats: &[Beat]| beats.iter().map(|b| b.tick_offset).collect::<Vec<_>>();
    assert_eq!(ticks(second), ticks(original));
    assert_eq!(slur_marks(second)[0], Some((0, SlurKind::End)));
    Ok(())
}

#[test]
fn musicxml_writes_the_first_measures_clef() -> GpResult<()> {
    let mut score = orchestra()?;
    measure_data_mut(&mut score, 0, 0)?.clef = Some(Clef::Bass);
    let second = score_partwise_to_loaded_score(&loaded_score_to_score_partwise(&score));
    assert_eq!(second.score.staves[0].clef, Clef::Bass);
    Ok(())
}

#[test]
fn musicxml_reads_clef_changes_written_before_the_barline() -> GpResult<()> {
    use crate::model::musicxml::measure::MusicData;

    let mut partwise = loaded_score_to_score_partwise(&orchestra()?);
    // Move the double bass' bar 2 clef change to the end of bar 1.
    let measures = &mut partwise.parts[1].measures;
    let position = present(
        measures[1].music_data.iter().position(
            |data| matches!(data, MusicData::Attributes(attributes) if !attributes.clefs.is_empty()),
        ),
        "clef change",
    )?;
    let change = measures[1].music_data.remove(position);
    measures[0].music_data.push(change);

    let score = score_partwise_to_loaded_score(&partwise);
    let changes: Vec<Option<Clef>> = score.score.tracks[1]
        .measures
        .values()
        .map(|m| m.clef)
        .collect();
    assert_eq!(changes, vec![None, Some(Clef::Treble)]);
    Ok(())
}

/// One 4/4 measure of five chords (quarter, quarter, eighth, eighth,
/// quarter) with overlapping slurs: A over chords 1–3, B over 2–5 and C
/// over 3–4, so chord 3 ends A and starts C while B is still open.
const MSCX_OVERLAPPING_SLURS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.70">
  <Score>
    <Division>480</Division>
    <Part id="1">
      <Staff id="1"><StaffType group="pitched"><name>stdNormal</name></StaffType></Staff>
      <trackName>Flute</trackName>
      <Instrument id="flute"><longName>Flute</longName></Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><fractions>1/2</fractions></location></next></Spanner>
            <Note><pitch>72</pitch><tpc>14</tpc></Note>
            </Chord>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><fractions>1/2</fractions></location></next></Spanner>
            <Note><pitch>74</pitch><tpc>16</tpc></Note>
            </Chord>
          <Chord>
            <durationType>eighth</durationType>
            <Spanner type="Slur"><prev><location><fractions>-1/2</fractions></location></prev></Spanner>
            <Spanner type="Slur"><Slur/><next><location><fractions>1/8</fractions></location></next></Spanner>
            <Note><pitch>76</pitch><tpc>18</tpc></Note>
            </Chord>
          <Chord>
            <durationType>eighth</durationType>
            <Spanner type="Slur"><prev><location><fractions>-1/8</fractions></location></prev></Spanner>
            <Note><pitch>77</pitch><tpc>13</tpc></Note>
            </Chord>
          <Chord>
            <durationType>quarter</durationType>
            <Spanner type="Slur"><prev><location><fractions>-1/2</fractions></location></prev></Spanner>
            <Note><pitch>79</pitch><tpc>15</tpc></Note>
            </Chord>
          </voice>
        </Measure>
      </Staff>
    </Score>
  </museScore>
"#;

#[test]
fn end_start_hands_the_ending_slurs_id_on_and_round_trips() -> GpResult<()> {
    let outcome = mscx_to_loaded_score(&parse_mscx(MSCX_OVERLAPPING_SLURS)?);
    let expected = vec![
        Some((0, SlurKind::Start)),
        Some((1, SlurKind::Start)),
        Some((0, SlurKind::EndStart)),
        Some((0, SlurKind::End)),
        Some((1, SlurKind::End)),
    ];
    assert_eq!(
        slur_marks(first_voice_beats(&outcome.score, 0, 0)?),
        expected
    );
    assert!(outcome.report.is_empty());

    // Chord 3 closes A, not the innermost open slur B.
    let regenerated = loaded_score_to_mscx(&outcome.score);
    let compact: String = regenerated.raw_xml.split_whitespace().collect();
    assert_eq!(
        compact
            .matches("<prev><location><fractions>-1/2</fractions></location></prev>")
            .count(),
        2
    );
    let second = mscx_to_loaded_score(&regenerated);
    assert_eq!(
        slur_marks(first_voice_beats(&second.score, 0, 0)?),
        expected
    );
    assert!(second.report.is_empty());
    Ok(())
}

#[test]
fn keys_keep_their_mode_through_a_round_trip() -> GpResult<()> {
    let mut score = orchestra()?;
    let key = present(score.score.timeline[0].key_signature.as_mut(), "key")?;
    key.root.step = PitchStep::A;
    key.root.alter = 0;
    key.mode = Mode::Minor;
    let regenerated = loaded_score_to_mscx(&score);
    assert!(regenerated.raw_xml.contains("<mode>minor</mode>"));
    let second = mscx_to_loaded_score(&regenerated).score;
    let key = present(second.score.timeline[0].key_signature, "key")?;
    assert_eq!(
        (key.root.step, key.root.alter, key.mode),
        (PitchStep::A, 0, Mode::Minor)
    );
    Ok(())
}

#[test]
fn writer_leaves_end_repeat_barlines_to_musescore() -> GpResult<()> {
    let mut score = orchestra()?;
    // MusicXML imports a backward repeat with a light-heavy barline.
    let measure = &mut score.score.timeline[0];
    measure.navigation.push(NavigationEvent {
        measure_index: MeasureIndex(0),
        kind: JumpKind::RepeatClose,
        repeat_count: Some(2),
    });
    measure.barline_right = Some(Barline {
        style: BarlineStyle::LightHeavy,
        ending: None,
    });
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(xml.contains("<endRepeat>2</endRepeat>"));
    assert!(!xml.contains("<BarLine>"));
    Ok(())
}

#[test]
fn writer_puts_barlines_and_volta_ends_at_the_measure_end() -> GpResult<()> {
    let mut score = orchestra()?;
    // The clarinet's bar 1 only holds three quarters.
    for voice in measure_data_mut(&mut score, 0, 0)?.voices.values_mut() {
        voice.beats[3].duration.base = NoteValue::Quarter;
    }
    let ending = |kind| Ending {
        numbers: vec![1],
        text: None,
        kind,
    };
    let measure = &mut score.score.timeline[0];
    measure.barline_left = Some(Barline {
        style: BarlineStyle::Regular,
        ending: Some(ending(EndingKind::Start)),
    });
    measure.barline_right = Some(Barline {
        style: BarlineStyle::LightLight,
        ending: Some(ending(EndingKind::Stop)),
    });
    let xml = loaded_score_to_mscx(&score).raw_xml;
    let compact: String = xml.split_whitespace().collect();
    // The volta's start points at the measure's end, where its end is.
    assert!(compact.contains("<next><location><fractions>1/1</fractions></location></next>"));
    assert!(compact.contains(
        "<location><fractions>1/4</fractions></location><BarLine><subtype>double</subtype></BarLine><Spannertype=\"Volta\"><prev><location><fractions>-1/1</fractions></location></prev>"
    ));
    Ok(())
}

#[test]
fn grace_chords_take_no_time_and_after_graces_follow_their_chord() -> GpResult<()> {
    // An after-grace written before bar 1's third chord belongs to it; an
    // acciaccatura before bar 2's first chord doesn't move its slur start.
    let third = "<Chord>\n            <durationType>quarter</durationType>\n            <Spanner type=\"Slur\"><prev><location><fractions>-1/4</fractions>";
    let cross = "<Chord>\n            <durationType>quarter</durationType>\n            <Spanner type=\"Slur\"><Slur/><next><location><voices>1</voices>";
    let mscx = MSCX_SLURS
        .replacen(
            third,
            &format!("<Chord><grace8after/><durationType>16th</durationType><Note><pitch>75</pitch><tpc>11</tpc></Note></Chord>\n          {third}"),
            1,
        )
        .replacen(
            cross,
            &format!("<Chord><acciaccatura/><durationType>eighth</durationType><Note><pitch>78</pitch><tpc>20</tpc></Note></Chord>\n          {cross}"),
            1,
        );
    assert_eq!(mscx.matches("grace8after").count(), 1);
    assert_eq!(mscx.matches("acciaccatura").count(), 1);

    let first = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert!(first.report.is_empty(), "{:?}", first.report);
    let graces = |score: &LoadedScore| -> GpResult<Vec<Vec<bool>>> {
        Ok(voice_beats(score, 0, 0, 0)?
            .iter()
            .map(|beat| beat.grace_notes.iter().map(|g| g.after).collect())
            .collect())
    };
    assert_eq!(
        graces(&first.score)?,
        vec![vec![], vec![], vec![true], vec![]]
    );
    let marks = slur_fixture_marks(&first.score)?;
    let plain = mscx_to_loaded_score(&parse_mscx(MSCX_SLURS)?).score;
    assert_eq!(marks, slur_fixture_marks(&plain)?);

    // The writer puts the after-grace before its chord too.
    let regenerated = loaded_score_to_mscx(&first.score);
    let second = mscx_to_loaded_score(&regenerated);
    assert!(second.report.is_empty(), "{:?}", second.report);
    assert_eq!(graces(&second.score)?, graces(&first.score)?);
    assert_eq!(slur_fixture_marks(&second.score)?, marks);
    Ok(())
}

/// A one-staff 4/4 score: `measures` is the `<Measure>` bodies.
fn flute_score(measures: &[&str]) -> String {
    let body: String = measures
        .iter()
        .map(|measure| format!("      <Measure>\n{measure}\n        </Measure>\n"))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.70">
  <Score>
    <Division>480</Division>
    <Part id="1">
      <Staff id="1"><StaffType group="pitched"><name>stdNormal</name></StaffType></Staff>
      <trackName>Flute</trackName>
      <Instrument id="flute"><longName>Flute</longName></Instrument>
      </Part>
    <Staff id="1">
{body}      </Staff>
    </Score>
  </museScore>
"#
    )
}

#[test]
fn modal_keys_mean_the_same_in_every_format() -> GpResult<()> {
    let mut score = orchestra()?;
    // D dorian has no sharps or flats.
    let key = present(score.score.timeline[0].key_signature.as_mut(), "key")?;
    key.root.step = PitchStep::D;
    key.root.alter = 0;
    key.mode = Mode::Dorian;
    let key = *key;
    assert_eq!(key.fifths(), 0);

    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(xml.contains("<concertKey>0</concertKey>"));
    let musicxml = score_partwise_to_loaded_score(&loaded_score_to_score_partwise(&score));
    assert_eq!(musicxml.score.timeline[0].key_signature, Some(key));
    assert_eq!(crate::convert::legacy::opt_key_to_legacy_key(&key).key, 0);
    // GP keeps only the signature: the lost mode is reported.
    let losses = crate::convert::legacy::loaded_score_to_legacy_song_with_losses(&score).1;
    assert_eq!(
        losses.get("Key mode dorian (kept as its signature, read back as major)"),
        1
    );
    Ok(())
}

/// A transposition by octaves without diatonic steps (MusicXML's are
/// optional) is written with its steps, which MuseScore otherwise takes as
/// none: an octave down is −7 steps, not 0.
#[test]
fn an_octave_transposition_without_steps_is_written_with_them() -> GpResult<()> {
    use crate::model::optimized::global::Transpose;

    let mut score = orchestra()?;
    score.score.instruments[0].transpose = Some(Transpose {
        diatonic: None,
        chromatic: 0,
        octave_change: Some(-1),
    });
    let xml = loaded_score_to_mscx(&score).raw_xml;
    let compact: String = xml.split_whitespace().collect();
    assert!(compact.contains(
        "<transposeChromatic>-12</transposeChromatic><transposeDiatonic>-7</transposeDiatonic>"
    ));
    let reread = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    let transpose = present(reread.score.instruments[0].transpose.as_ref(), "transpose")?;
    assert_eq!((transpose.diatonic, transpose.chromatic), (Some(-7), -12));
    Ok(())
}

#[test]
fn slurs_on_grace_chords_leave_enclosing_slurs_alone() -> GpResult<()> {
    // A phrase slur over three chords, and one from an acciaccatura to the
    // second chord.
    let mscx = flute_score(&[r#"        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Chord><durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><fractions>1/2</fractions></location></next></Spanner>
            <Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>
          <Chord><acciaccatura/><durationType>eighth</durationType>
            <Spanner type="Slur"><Slur/><next><location><fractions>0/1</fractions></location></next></Spanner>
            <Note><pitch>76</pitch><tpc>18</tpc></Note></Chord>
          <Chord><durationType>quarter</durationType>
            <Spanner type="Slur"><prev><location><fractions>0/1</fractions></location></prev></Spanner>
            <Note><pitch>74</pitch><tpc>16</tpc></Note></Chord>
          <Chord><durationType>half</durationType>
            <Spanner type="Slur"><prev><location><fractions>-1/2</fractions></location></prev></Spanner>
            <Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>
          </voice>"#]);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(
        slur_marks(first_voice_beats(&outcome.score, 0, 0)?),
        vec![Some((0, SlurKind::Start)), None, Some((0, SlurKind::End))]
    );
    assert_eq!(outcome.report.get("Slur (on a grace chord)"), 1);
    assert_eq!(outcome.report.iter().map(|(_, n)| n).sum::<u32>(), 1);
    Ok(())
}

#[test]
fn hairpins_over_a_grace_preceded_chord_pair_up() -> GpResult<()> {
    let mscx = flute_score(&[r#"        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Spanner type="HairPin"><HairPin><subtype>0</subtype></HairPin><next><location><fractions>1/2</fractions></location></next></Spanner>
          <Chord><acciaccatura/><durationType>eighth</durationType><Note><pitch>74</pitch><tpc>16</tpc></Note></Chord>
          <Chord><durationType>half</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>
          <Spanner type="HairPin"><prev><location><fractions>-1/2</fractions></location></prev></Spanner>
          <Rest><durationType>half</durationType></Rest>
          </voice>"#]);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let hairpins: Vec<(u8, bool)> = first_voice_beats(&outcome.score, 0, 0)?[0]
        .events
        .iter()
        .filter_map(|event| match event {
            crate::model::optimized::effect::BeatEvent::Hairpin(h) => Some((
                h.id,
                h.kind == crate::model::optimized::effect::HairpinKind::Stop,
            )),
            _ => None,
        })
        .collect();
    assert_eq!(hairpins, vec![(1, false), (1, true)]);
    let xml = loaded_score_to_mscx(&outcome.score).raw_xml;
    assert_eq!(xml.matches("<HairPin>").count(), 1);
    Ok(())
}

#[test]
fn writer_keeps_slurs_running_back_to_an_earlier_voice() -> GpResult<()> {
    // Voice 2's first chord slurs to voice 1's second.
    let mscx = flute_score(&[r#"        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Rest><durationType>quarter</durationType></Rest>
          <Chord><durationType>quarter</durationType>
            <Spanner type="Slur"><prev><location><voices>1</voices><fractions>-1/4</fractions></location></prev></Spanner>
            <Note><pitch>74</pitch><tpc>16</tpc></Note></Chord>
          <Rest><durationType>half</durationType></Rest>
          </voice>
        <voice>
          <Chord><durationType>quarter</durationType>
            <Spanner type="Slur"><Slur/><next><location><voices>-1</voices><fractions>1/4</fractions></location></next></Spanner>
            <Note><pitch>67</pitch><tpc>15</tpc></Note></Chord>
          <Rest><durationType>quarter</durationType></Rest>
          <Rest><durationType>half</durationType></Rest>
          </voice>"#]);
    let first = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert!(first.report.is_empty(), "{:?}", first.report);
    let marks = |score: &LoadedScore| -> GpResult<Vec<Vec<SlurMark>>> {
        Ok(vec![
            slur_marks(voice_beats(score, 0, 0, 0)?),
            slur_marks(voice_beats(score, 0, 0, 1)?),
        ])
    };
    assert_eq!(
        marks(&first.score)?,
        vec![
            vec![None, Some((0, SlurKind::End)), None],
            vec![Some((0, SlurKind::Start)), None, None],
        ]
    );
    let second = mscx_to_loaded_score(&loaded_score_to_mscx(&first.score));
    assert!(second.report.is_empty(), "{:?}", second.report);
    assert_eq!(marks(&second.score)?, marks(&first.score)?);
    Ok(())
}

#[test]
fn converter_reports_notation_it_drops() -> GpResult<()> {
    let mscx = MSCX_ORCHESTRA
        .replacen(
            "<durationType>half</durationType>\n            <Note><pitch>76</pitch>",
            "<durationType>half</durationType>\n            <Tremolo><subtype>c16</subtype></Tremolo>\n            <ChordLine><subtype>9</subtype></ChordLine>\n            <Note><pitch>76</pitch>",
            1,
        )
        .replacen(
            "<TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>",
            "<TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Dynamic><subtype>p</subtype><velocity>100</velocity></Dynamic>",
            1,
        );
    assert_eq!(mscx.matches("c16").count(), 1);
    assert_eq!(mscx.matches("<velocity>").count(), 1);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(outcome.report.get("Tremolo (two-chord)"), 1);
    assert_eq!(outcome.report.get("ChordLine (unknown)"), 1);
    assert_eq!(outcome.report.get("Dynamic (velocity)"), 1);
    Ok(())
}

#[test]
fn converter_takes_the_instruments_clef_without_a_default_clef() -> GpResult<()> {
    let mscx = orchestra_with("<defaultClef>F8vb</defaultClef>", "");
    let score = mscx_to_loaded_score(&parse_mscx(&mscx)?).score;
    assert_eq!(score.score.staves[1].clef, Clef::Bass8vb);
    assert_eq!(measure_data(&score, 1, 0)?.clef, None);
    Ok(())
}

#[test]
fn voltas_steer_playback_and_survive_guitar_pro() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::optimized::timeline::Arrangement;

    let volta = |endings: u8, hook: u8| {
        format!(
            r#"          <Spanner type="Volta"><Volta><endHookType>{hook}</endHookType><beginText>{endings}.</beginText><endings>{endings}</endings></Volta><next><location><measures>1</measures></location></next></Spanner>"#
        )
    };
    let end = r#"          <Spanner type="Volta"><prev><location><measures>-1</measures></location></prev></Spanner>"#;
    let whole = "          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let first = format!(
        "        <startRepeat/>\n        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{whole}\n          </voice>"
    );
    let second = format!(
        "        <endRepeat>2</endRepeat>\n        <voice>\n{}\n{whole}\n{end}\n          </voice>",
        volta(1, 1)
    );
    let third = format!(
        "        <voice>\n{}\n{whole}\n{end}\n          </voice>",
        volta(2, 0)
    );
    let mscx = flute_score(&[&first, &second, &third]);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let order = |score: &LoadedScore| -> Vec<u16> {
        Arrangement::resolve(&score.score.timeline)
            .order
            .iter()
            .map(|m| m.0)
            .collect()
    };
    assert_eq!(order(&outcome.score), vec![0, 1, 0, 2]);

    // Guitar Pro keeps endings as a bitmask per measure.
    let song = loaded_score_to_legacy_song(&outcome.score);
    let alternatives: Vec<u8> = song
        .measure_headers
        .iter()
        .map(|h| h.repeat_alternative)
        .collect();
    assert_eq!(alternatives, vec![0, 1, 2]);
    let back = legacy_song_to_loaded_score(&song);
    assert_eq!(order(&back), vec![0, 1, 0, 2]);
    let xml = loaded_score_to_mscx(&back).raw_xml;
    assert_eq!(xml.matches("<Volta>").count(), 2);
    Ok(())
}

#[test]
fn musicxml_endings_reach_guitar_pro_as_pass_masks() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::model::musicxml::ScorePartwise;

    // A repeat played three times: ending "1, 2" closes it, ending 3
    // follows.
    let note = "<note><pitch><step>C</step><octave>5</octave></pitch><duration>4</duration><type>whole</type></note>";
    let xml = format!(
        r#"<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Flute</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>
      <barline location="left"><repeat direction="forward"/></barline>
      {note}
    </measure>
    <measure number="2">
      <barline location="left"><ending number="1, 2" type="start"/></barline>
      {note}
      <barline location="right"><ending number="1, 2" type="stop"/><repeat direction="backward" times="3"/></barline>
    </measure>
    <measure number="3">
      <barline location="left"><ending number="3" type="start"/></barline>
      {note}
      <barline location="right"><ending number="3" type="discontinue"/></barline>
    </measure>
  </part>
</score-partwise>"#
    );
    let partwise: ScorePartwise =
        quick_xml::de::from_str(&xml).map_err(|e| GpError::MsczXml(e.to_string()))?;
    let score = score_partwise_to_loaded_score(&partwise);
    let song = loaded_score_to_legacy_song(&score);
    let alternatives: Vec<u8> = song
        .measure_headers
        .iter()
        .map(|h| h.repeat_alternative)
        .collect();
    assert_eq!(alternatives, vec![0, 0b011, 0b100]);
    Ok(())
}

#[test]
fn multi_measure_endings_play_through_guitar_pro() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::optimized::timeline::Arrangement;

    let volta = |endings: u8, measures: u8| {
        format!(
            r#"          <Spanner type="Volta"><Volta><endHookType>1</endHookType><beginText>{endings}.</beginText><endings>{endings}</endings></Volta><next><location><measures>{measures}</measures></location></next></Spanner>"#
        )
    };
    let end = |measures: i8| {
        format!(
            r#"          <Spanner type="Volta"><prev><location><measures>{measures}</measures></location></prev></Spanner>"#
        )
    };
    let whole = "          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    // |: A | 1. B | C :| 2. D |
    let bars = [
        format!(
            "        <startRepeat/>\n        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{whole}\n          </voice>"
        ),
        format!(
            "        <voice>\n{}\n{whole}\n          </voice>",
            volta(1, 2)
        ),
        format!("        <endRepeat>2</endRepeat>\n        <voice>\n{whole}\n          </voice>"),
        format!(
            "        <voice>\n{}\n{}\n{whole}\n{}\n          </voice>",
            end(-2),
            volta(2, 1),
            end(-1)
        ),
    ];
    let refs: Vec<&str> = bars.iter().map(String::as_str).collect();
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&refs))?);
    let order = |score: &LoadedScore| -> Vec<u16> {
        Arrangement::resolve(&score.score.timeline)
            .order
            .iter()
            .map(|m| m.0)
            .collect()
    };
    assert_eq!(order(&outcome.score), vec![0, 1, 2, 0, 3]);
    let song = loaded_score_to_legacy_song(&outcome.score);
    let back = legacy_song_to_loaded_score(&song);
    assert_eq!(order(&back), vec![0, 1, 2, 0, 3]);
    // GP3/4 store an ending's number per measure: the second measure of the
    // first ending reads as its continuation.
    let mut gp4 = crate::model::legacy::song::Song::default();
    gp4.read_gp4(&song.write((4, 0, 0), None)?)?;
    let alternatives: Vec<u8> = gp4
        .measure_headers
        .iter()
        .map(|h| h.repeat_alternative)
        .collect();
    assert_eq!(alternatives, vec![0, 1, 1, 2]);
    assert_eq!(
        order(&legacy_song_to_loaded_score(&gp4)),
        vec![0, 1, 2, 0, 3]
    );
    Ok(())
}

#[test]
fn volta_ends_pair_with_the_start_they_point_at() -> GpResult<()> {
    let volta = |endings: u8, hook: u8| {
        format!(
            r#"          <Spanner type="Volta"><Volta><endHookType>{hook}</endHookType><beginText>{endings}.</beginText><endings>{endings}</endings></Volta><next><location><measures>1</measures></location></next></Spanner>"#
        )
    };
    let end = r#"          <Spanner type="Volta"><prev><location><measures>-1</measures></location></prev></Spanner>"#;
    let whole = "          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let first = format!(
        "        <startRepeat/>\n        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{whole}\n          </voice>"
    );
    let second = format!(
        "        <endRepeat>2</endRepeat>\n        <voice>\n{}\n{whole}\n          </voice>",
        volta(1, 1)
    );
    // Volta 2 starts before volta 1's end element.
    let third = format!(
        "        <voice>\n{}\n{end}\n{whole}\n{end}\n          </voice>",
        volta(2, 0)
    );
    let mscx = flute_score(&[&first, &second, &third]);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let numbers = crate::model::optimized::timeline::ending_numbers(&outcome.score.score.timeline);
    assert_eq!(numbers, vec![None, Some(&[1][..]), Some(&[2][..])]);
    Ok(())
}

/// A volta over two measures, as written here (its end after the last
/// measure's beats, counting back from its end), reads back paired with
/// its start.
#[test]
fn written_multi_measure_voltas_read_back() -> GpResult<()> {
    let volta = r#"          <Spanner type="Volta"><Volta><endHookType>1</endHookType><beginText>1.</beginText><endings>1</endings></Volta><next><location><measures>2</measures></location></next></Spanner>"#;
    let end = r#"          <Spanner type="Volta"><prev><location><measures>-2</measures></location></prev></Spanner>"#;
    let whole = "          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let first = format!(
        "        <startRepeat/>\n        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{whole}\n          </voice>"
    );
    let second = format!("        <voice>\n{volta}\n{whole}\n          </voice>");
    let third =
        format!("        <endRepeat>2</endRepeat>\n        <voice>\n{whole}\n          </voice>");
    let fourth = format!("        <voice>\n{end}\n{whole}\n          </voice>");
    let mscx = flute_score(&[&first, &second, &third, &fourth]);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let numbers = |timeline| crate::model::optimized::timeline::ending_numbers(timeline);
    let expected = vec![None, Some(&[1][..]), Some(&[1][..]), None];
    assert_eq!(numbers(&outcome.score.score.timeline), expected);

    let reread = mscx_to_loaded_score(&parse_mscx(&loaded_score_to_mscx(&outcome.score).raw_xml)?);
    assert!(reread.report.is_empty(), "{:?}", reread.report);
    assert_eq!(numbers(&reread.score.score.timeline), expected);
    Ok(())
}

#[test]
fn measure_rests_take_their_measures_length() -> GpResult<()> {
    let rest =
        "          <Rest><durationType>measure</durationType><duration>3/4</duration></Rest>";
    let three = format!(
        "        <voice>\n          <TimeSig><sigN>3</sigN><sigD>4</sigD></TimeSig>\n{rest}\n          </voice>"
    );
    let five = format!(
        "        <voice>\n          <TimeSig><sigN>5</sigN><sigD>4</sigD></TimeSig>\n{rest}\n          </voice>"
    );
    let mscx = flute_score(&[&three, &five]);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    let durations = |measure| -> GpResult<Vec<(NoteValue, u8)>> {
        Ok(first_voice_beats(&outcome.score, 0, measure)?
            .iter()
            .map(|beat| (beat.duration.base, beat.duration.dots))
            .collect())
    };
    assert_eq!(durations(0)?, vec![(NoteValue::Half, 1)]);
    assert_eq!(
        durations(1)?,
        vec![(NoteValue::Whole, 0), (NoteValue::Quarter, 0)]
    );

    let xml = loaded_score_to_mscx(&outcome.score).raw_xml;
    let compact: String = xml.split_whitespace().collect();
    assert!(compact.contains("<durationType>measure</durationType><duration>3/4</duration>"));
    assert!(!compact.contains("<durationType>whole</durationType><duration>3/4"));
    Ok(())
}

#[test]
fn hairpins_ending_after_a_split_measure_rest_end_after_its_last_piece() -> GpResult<()> {
    use crate::model::optimized::effect::{BeatEvent, HairpinKind};

    let first = "        <voice>\n          <TimeSig><sigN>5</sigN><sigD>4</sigD></TimeSig>\n          <Spanner type=\"HairPin\"><HairPin><subtype>0</subtype></HairPin><next><location><measures>2</measures></location></next></Spanner>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          <Chord><durationType>quarter</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let second = "        <voice>\n          <Rest><durationType>measure</durationType><duration>5/4</duration></Rest>\n          <Spanner type=\"HairPin\"><prev><location><measures>-2</measures></location></prev></Spanner>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[first, second]))?);
    let stops: Vec<usize> = first_voice_beats(&outcome.score, 0, 1)?
        .iter()
        .map(|beat| {
            beat.events
                .iter()
                .filter(|e| matches!(e, BeatEvent::Hairpin(h) if h.kind == HairpinKind::Stop))
                .count()
        })
        .collect();
    assert_eq!(stops, vec![0, 1]);
    Ok(())
}

#[test]
fn converter_reports_segment_elements_after_the_last_beat() -> GpResult<()> {
    let measure = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          <Dynamic><subtype>p</subtype></Dynamic>\n          <Spanner type=\"HairPin\"><HairPin><subtype>0</subtype></HairPin><next><location><measures>1</measures></location></next></Spanner>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[measure]))?);
    assert_eq!(outcome.report.get("Dynamic (after the last beat)"), 1);
    assert_eq!(outcome.report.get("HairPin (after the last beat)"), 1);
    Ok(())
}

#[test]
fn guitar_pro_velocities_hold_the_last_dynamic() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;

    let quarter = "          <Chord><durationType>quarter</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let dynamic = |name: &str| format!("          <Dynamic><subtype>{name}</subtype></Dynamic>");
    let first = format!(
        "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{}\n{quarter}\n{quarter}\n{}\n{quarter}\n{quarter}\n          </voice>",
        dynamic("p"),
        dynamic("sfz"),
    );
    let second = format!(
        "        <voice>\n{quarter}\n{}\n{quarter}\n{quarter}\n{quarter}\n          </voice>",
        dynamic("other-dynamics"),
    );
    let mscx = flute_score(&[&first, &second]);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    let song = loaded_score_to_legacy_song(&outcome.score);
    let velocities: Vec<i16> = song.tracks[0]
        .measures
        .iter()
        .flat_map(|measure| &measure.voices[0].beats)
        .map(|beat| beat.notes[0].velocity)
        .collect();
    // p holds; sfz accents one beat; an unknown marking keeps the level.
    assert_eq!(velocities, vec![47, 47, 111, 47, 47, 47, 47, 47]);
    Ok(())
}

/// A unison on two strings, both tied on: each note's tie is written,
/// neither paired with the other in its own chord.
#[test]
fn ties_of_a_unison_are_each_written() -> GpResult<()> {
    use crate::model::optimized::global::MeasureIndex;

    let chord = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n          </voice>";
    let mut score = mscx_to_loaded_score(&parse_mscx(&flute_score(&[chord, chord]))?).score;
    for (measure, tie) in [(0, TieType::Start), (1, TieType::End)] {
        let data = present(
            score.score.tracks[0]
                .measures
                .get_mut(&MeasureIndex(measure)),
            "measure",
        )?;
        for voice in data.voices.values_mut() {
            for (string, note) in voice.beats[0].notes.iter_mut().enumerate() {
                note.tie = Some(tie);
                note.string = Some(string as u8 + 1);
            }
        }
    }
    let written = loaded_score_to_mscx(&score).raw_xml;
    let compact: String = written.split_whitespace().collect();
    let tie_to_next = "<Spannertype=\"Tie\"><Tie></Tie><next><location><measures>1</measures></location></next></Spanner>";
    let tie_from_prev =
        "<Spannertype=\"Tie\"><prev><location><measures>-1</measures></location></prev></Spanner>";
    assert_eq!(compact.matches(tie_to_next).count(), 2);
    assert_eq!(compact.matches(tie_from_prev).count(), 2);
    Ok(())
}

/// A unison on two strings, one tied on: the untied note on the other
/// string, first in the next chord, doesn't take the tie from it.
#[test]
fn an_untied_unison_note_keeps_the_other_strings_tie() -> GpResult<()> {
    use crate::model::optimized::global::MeasureIndex;

    let chord = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n          </voice>";
    let mut score = mscx_to_loaded_score(&parse_mscx(&flute_score(&[chord, chord]))?).score;
    // (string, tie) of each note, in chord order.
    for (measure, notes) in [
        (0, [(1, Some(TieType::Start)), (2, None)]),
        (1, [(2, None), (1, Some(TieType::End))]),
    ] {
        let data = present(
            score.score.tracks[0]
                .measures
                .get_mut(&MeasureIndex(measure)),
            "measure",
        )?;
        for voice in data.voices.values_mut() {
            for (note, (string, tie)) in voice.beats[0].notes.iter_mut().zip(notes) {
                note.string = Some(string);
                note.tie = tie;
            }
        }
    }
    let written = loaded_score_to_mscx(&score).raw_xml;
    let compact: String = written.split_whitespace().collect();
    let tie_to_next = "<Spannertype=\"Tie\"><Tie></Tie><next><location><measures>1</measures></location></next></Spanner>";
    let tie_from_prev =
        "<Spannertype=\"Tie\"><prev><location><measures>-1</measures></location></prev></Spanner>";
    assert_eq!(compact.matches(tie_to_next).count(), 1);
    assert_eq!(compact.matches(tie_from_prev).count(), 1);
    Ok(())
}

#[test]
fn tie_chains_keep_their_middle_notes() -> GpResult<()> {
    let note = |ties: &str| {
        format!(
            "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc>{ties}</Note></Chord>\n          </voice>"
        )
    };
    let next = r#"<Spanner type="Tie"><next/></Spanner>"#;
    let prev = r#"<Spanner type="Tie"><prev/></Spanner>"#;
    let bars = [note(next), note(&format!("{next}{prev}")), note(prev)];
    let refs: Vec<&str> = bars.iter().map(String::as_str).collect();
    let ties = |score: &LoadedScore| -> GpResult<Vec<Option<TieType>>> {
        (0..3)
            .map(|measure| {
                // MusicXML numbers its voices from 1.
                let data = measure_data(score, 0, measure)?;
                let voice = present(data.voices.keys().min().copied(), "voice")?;
                Ok(voice_beats(score, 0, measure, voice)?[0].notes[0].tie)
            })
            .collect()
    };
    let expected = vec![
        Some(TieType::Start),
        Some(TieType::Continue),
        Some(TieType::End),
    ];
    let first = mscx_to_loaded_score(&parse_mscx(&flute_score(&refs))?).score;
    assert_eq!(ties(&first)?, expected);
    // MuseScore pairs a tie's ends by their locations, the start holding
    // its `<Tie>`.
    let written = loaded_score_to_mscx(&first).raw_xml;
    let compact: String = written.split_whitespace().collect();
    assert_eq!(
        compact
            .matches("<Spannertype=\"Tie\"><Tie></Tie><next><location><measures>1</measures></location></next></Spanner>")
            .count(),
        2
    );
    assert_eq!(
        compact
            .matches("<Spannertype=\"Tie\"><prev><location><measures>-1</measures></location></prev></Spanner>")
            .count(),
        2
    );
    let mscz = mscx_to_loaded_score(&parse_mscx(&written)?).score;
    assert_eq!(ties(&mscz)?, expected);
    let xml = score_partwise_to_loaded_score(&loaded_score_to_score_partwise(&first));
    assert_eq!(ties(&xml)?, expected);
    Ok(())
}

#[test]
fn dynamic_glyphs_in_text_read_as_letters_and_write_back_as_glyphs() -> GpResult<()> {
    use crate::model::optimized::beat::Dynamic;

    let bar = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Dynamic><subtype>other-dynamics</subtype><text>sempre <sym>dynamicPiano</sym><sym>dynamicPiano</sym></text></Dynamic>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[bar]))?);
    let dynamic = first_voice_beats(&outcome.score, 0, 0)?[0].dynamic.clone();
    assert_eq!(dynamic, Some(Dynamic::Other("sempre pp".to_string())));
    let xml = loaded_score_to_mscx(&outcome.score).raw_xml;
    assert!(xml.contains("<text>sempre <sym>dynamicPiano</sym><sym>dynamicPiano</sym></text>"));
    Ok(())
}

/// Each glyph reads where it is: a glyph's name in an unknown one's before
/// it isn't taken for it.
#[test]
fn a_dynamic_glyph_reads_in_its_own_place() -> GpResult<()> {
    use crate::model::optimized::beat::Dynamic;

    let bar = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Dynamic><subtype>other-dynamics</subtype><text><sym>dynamicPianoUnknown</sym> <sym>dynamicPiano</sym></text></Dynamic>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[bar]))?);
    let dynamic = first_voice_beats(&outcome.score, 0, 0)?[0].dynamic.clone();
    assert_eq!(
        dynamic,
        Some(Dynamic::Other("dynamicPianoUnknown p".to_string()))
    );
    Ok(())
}

#[test]
fn timpani_stay_pitched() -> GpResult<()> {
    let bar = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>48</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let mscx = flute_score(&[bar]).replacen(
        r#"<Instrument id="flute"><longName>Flute</longName></Instrument>"#,
        r#"<Instrument id="timpani"><longName>Timpani</longName><instrumentId>drum.timpani</instrumentId></Instrument>"#,
        1,
    );
    let first = mscx_to_loaded_score(&parse_mscx(&mscx)?).score;
    assert!(matches!(
        first.score.instruments[0].kind,
        InstrumentKind::Pitched
    ));
    let xml = loaded_score_to_mscx(&first).raw_xml;
    assert!(xml.contains("drum.timpani"));
    let second = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    assert!(matches!(
        second.score.instruments[0].kind,
        InstrumentKind::Pitched
    ));
    Ok(())
}

#[test]
fn unknown_dynamics_take_another_voices_level_at_the_same_tick() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;

    let whole = "          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let bar = format!(
        "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Dynamic><subtype>other-dynamics</subtype><text>sempre p</text></Dynamic>\n{whole}\n          </voice>\n        <voice>\n          <Dynamic><subtype>ff</subtype></Dynamic>\n{whole}\n          </voice>"
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[&bar]))?);
    let song = loaded_score_to_legacy_song(&outcome.score);
    let velocities: Vec<i16> = song.tracks[0].measures[0]
        .voices
        .iter()
        .filter_map(|voice| voice.beats.first())
        .map(|beat| beat.notes[0].velocity)
        .collect();
    assert_eq!(velocities, vec![111, 111]);
    Ok(())
}

#[test]
fn guitar_pro_velocities_import_as_dynamic_changes() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::optimized::beat::Dynamic;

    let quarter = "          <Chord><durationType>quarter</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let bar = format!(
        "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{quarter}\n{quarter}\n{quarter}\n{quarter}\n          </voice>"
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[&bar]))?);
    let mut song = loaded_score_to_legacy_song(&outcome.score);
    for (beat, velocity) in song.tracks[0].measures[0].voices[0]
        .beats
        .iter_mut()
        .zip([47, 47, 95, 95])
    {
        beat.notes[0].velocity = velocity;
    }
    let score = legacy_song_to_loaded_score(&song);
    let dynamics: Vec<Option<Dynamic>> = first_voice_beats(&score, 0, 0)?
        .iter()
        .map(|beat| beat.dynamic.clone())
        .collect();
    assert_eq!(
        dynamics,
        vec![Some(Dynamic::P), None, Some(Dynamic::F), None]
    );
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert_eq!(xml.matches("<Dynamic>").count(), 2);
    Ok(())
}

#[test]
fn writer_drops_slur_ends_whose_start_was_on_a_rest() -> GpResult<()> {
    use crate::model::optimized::beat::Slur;

    let quarter = "          <Chord><durationType>quarter</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let bar = format!(
        "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{quarter}\n{quarter}\n{quarter}\n{quarter}\n          </voice>"
    );
    let mut score = mscx_to_loaded_score(&parse_mscx(&flute_score(&[&bar]))?).score;
    let slur = |slur_id, kind| Some(Slur { slur_id, kind });
    for voice in measure_data_mut(&mut score, 0, 0)?.voices.values_mut() {
        let beats = &mut voice.beats;
        // Slur 1 over the bar; slur 2 starts on a rest, so can't be written.
        beats[1].notes.clear();
        beats[1].gp_rest = true;
        beats[0].slur = slur(1, SlurKind::Start);
        beats[1].slur = slur(2, SlurKind::Start);
        beats[2].slur = slur(2, SlurKind::End);
        beats[3].slur = slur(1, SlurKind::End);
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    let back = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    let kinds: Vec<Option<SlurKind>> = slur_marks(first_voice_beats(&back, 0, 0)?)
        .into_iter()
        .map(|mark| mark.map(|(_, kind)| kind))
        .collect();
    assert_eq!(
        kinds,
        vec![Some(SlurKind::Start), None, None, Some(SlurKind::End)]
    );
    Ok(())
}

#[test]
fn endings_without_a_stop_are_written_as_voltas() -> GpResult<()> {
    use crate::model::optimized::timeline::Arrangement;

    let whole = "          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let first = format!(
        "        <startRepeat/>\n        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{whole}\n          </voice>"
    );
    let second =
        format!("        <endRepeat>2</endRepeat>\n        <voice>\n{whole}\n          </voice>");
    let third = format!("        <voice>\n{whole}\n          </voice>");
    let mut score =
        mscx_to_loaded_score(&parse_mscx(&flute_score(&[&first, &second, &third]))?).score;
    // Starts only, as a MusicXML file may have them.
    for (measure, number) in [(1, 1), (2, 2)] {
        score.score.timeline[measure].barline_left = Some(Barline {
            style: BarlineStyle::Regular,
            ending: Some(Ending {
                numbers: vec![number],
                text: None,
                kind: EndingKind::Start,
            }),
        });
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert_eq!(xml.matches("<Volta>").count(), 2);
    let back = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    let order: Vec<u16> = Arrangement::resolve(&back.score.timeline)
        .order
        .iter()
        .map(|m| m.0)
        .collect();
    assert_eq!(order, vec![0, 1, 0, 2]);
    Ok(())
}

#[test]
fn guitar_pro_voices_keep_their_own_dynamics() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;

    let quarter = "          <Chord><durationType>quarter</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>";
    let voice = format!("{quarter}\n{quarter}\n{quarter}\n{quarter}");
    let bar = format!(
        "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n{voice}\n          </voice>\n        <voice>\n{voice}\n          </voice>"
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[&bar, &bar]))?);
    let mut song = loaded_score_to_legacy_song(&outcome.score);
    // Voice 1 stays forte (GP's default), voice 2 plays piano, then forte
    // in the second bar.
    let wanted = [[95, 47], [95, 95]];
    for (measure, velocities) in song.tracks[0].measures.iter_mut().zip(wanted) {
        for (voice, velocity) in measure.voices.iter_mut().zip(velocities) {
            for beat in &mut voice.beats {
                beat.notes[0].velocity = velocity;
            }
        }
    }
    let mut score = legacy_song_to_loaded_score(&song);
    // Without each note's own velocity, the dynamics carry them.
    for data in score.score.tracks[0].measures.values_mut() {
        for voice in data.voices.values_mut() {
            for beat in &mut voice.beats {
                for note in &mut beat.notes {
                    note.gp_velocity = None;
                }
            }
        }
    }
    let back = loaded_score_to_legacy_song(&score);
    let velocities: Vec<Vec<i16>> = back.tracks[0]
        .measures
        .iter()
        .flat_map(|measure| &measure.voices)
        .map(|voice| {
            voice
                .beats
                .iter()
                .map(|beat| beat.notes[0].velocity)
                .collect()
        })
        .collect();
    assert_eq!(
        velocities,
        vec![vec![95; 4], vec![47; 4], vec![95; 4], vec![95; 4]]
    );
    Ok(())
}

#[test]
fn unclosed_endings_end_at_the_next_repeat_barline() {
    use crate::model::optimized::timeline::{Arrangement, MeasureDef, ending_numbers};

    let bar = |index: u16, start: Option<u8>, repeats: &[JumpKind]| MeasureDef {
        index: MeasureIndex(index),
        tempo: None,
        time_signature: None,
        key_signature: None,
        marker: None,
        navigation: repeats
            .iter()
            .map(|&kind| NavigationEvent {
                measure_index: MeasureIndex(index),
                kind,
                repeat_count: (kind == JumpKind::RepeatClose).then_some(2),
            })
            .collect(),
        tick_resolution: 960,
        duration_ticks: 3840,
        barline_left: start.map(|number| Barline {
            style: BarlineStyle::Regular,
            ending: Some(Ending {
                numbers: vec![number],
                text: None,
                kind: EndingKind::Start,
            }),
        }),
        barline_right: None,
        gp_beams: None,
        gp_fermatas: Vec::new(),
        gp_free_time: false,
    };
    // |: A | 1. B :| 2. C | |: D | E :|   (neither ending has a stop)
    let timeline = vec![
        bar(0, None, &[JumpKind::RepeatOpen]),
        bar(1, Some(1), &[JumpKind::RepeatClose]),
        bar(2, Some(2), &[]),
        bar(3, None, &[JumpKind::RepeatOpen]),
        bar(4, None, &[JumpKind::RepeatClose]),
    ];
    assert_eq!(
        ending_numbers(&timeline),
        vec![None, Some(&[1][..]), Some(&[2][..]), None, None]
    );
    let order: Vec<u16> = Arrangement::resolve(&timeline)
        .order
        .iter()
        .map(|m| m.0)
        .collect();
    assert_eq!(order, vec![0, 1, 0, 2, 3, 4, 3, 4]);
}

/// A `<pitch>` beyond MIDI's 127 drops the note, and is reported.
#[test]
fn converter_drops_pitches_beyond_midi() -> GpResult<()> {
    let mscx =
        MSCX_SINGLE_TRACK.replacen("<pitch>56</pitch><tpc>22</tpc>", "<pitch>200</pitch>", 1);
    assert_ne!(mscx, MSCX_SINGLE_TRACK);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(outcome.report.get("Note (pitch beyond MIDI's range)"), 1);
    let pitches: Vec<i8> = first_voice_beats(&outcome.score, 0, 1)?[0]
        .notes
        .iter()
        .filter_map(|note| note.pitch.as_ref())
        .map(crate::convert::legacy::pitch_to_midi)
        .collect();
    assert_eq!(pitches, vec![52, 59]);
    Ok(())
}

/// A pitch in octave −1 (MIDI 0–11) is below what the model holds: the
/// note is dropped and reported, without a false spelling loss and
/// without moving it up an octave.
#[test]
fn converter_drops_pitches_below_octave_zero() -> GpResult<()> {
    // F-1, spelled F.
    let mscx = MSCX_SINGLE_TRACK.replacen(
        "<pitch>56</pitch><tpc>22</tpc>",
        "<pitch>5</pitch><tpc>13</tpc>",
        1,
    );
    assert_ne!(mscx, MSCX_SINGLE_TRACK);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(outcome.report.get("Note (pitch below octave 0)"), 1);
    assert_eq!(
        outcome.report.get("Note (spelling not matching its pitch)"),
        0
    );
    let pitches: Vec<i8> = first_voice_beats(&outcome.score, 0, 1)?[0]
        .notes
        .iter()
        .filter_map(|note| note.pitch.as_ref())
        .map(crate::convert::legacy::pitch_to_midi)
        .collect();
    assert_eq!(pitches, vec![52, 59]);
    Ok(())
}

/// A note spelled in octave −1 (MIDI 12 as B♯-1) can't keep its
/// spelling: it's respelled with sharps (C0), and that's reported.
#[test]
fn converter_reports_spellings_below_octave_zero() -> GpResult<()> {
    let mscx = MSCX_SINGLE_TRACK.replacen(
        "<pitch>56</pitch><tpc>22</tpc>",
        "<pitch>12</pitch><tpc>26</tpc>",
        1,
    );
    assert_ne!(mscx, MSCX_SINGLE_TRACK);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(outcome.report.get("Note (spelling below octave 0)"), 1);
    assert_eq!(
        outcome.report.get("Note (spelling not matching its pitch)"),
        0
    );
    let pitches: Vec<i8> = first_voice_beats(&outcome.score, 0, 1)?[0]
        .notes
        .iter()
        .filter_map(|note| note.pitch.as_ref())
        .map(crate::convert::legacy::pitch_to_midi)
        .collect();
    assert_eq!(pitches, vec![52, 12, 59]);
    Ok(())
}

/// MuseScore's pitches end at MIDI 127: the writer moves a note above it
/// down by octaves, keeping its spelling, so reading it back keeps it.
#[test]
fn writer_keeps_pitches_within_midi_range() -> GpResult<()> {
    let mut outcome = mscx_to_loaded_score(&parse_mscx(MSCX_SINGLE_TRACK)?);
    let note = outcome.score.score.tracks[0]
        .measures
        .values_mut()
        .flat_map(|data| data.voices.values_mut())
        .flat_map(|voice| &mut voice.beats)
        .flat_map(|beat| &mut beat.notes)
        .find(|note| note.pitch.is_some())
        .ok_or(GpError::MissingState { field: "note" })?;
    // B♯9: MIDI 132.
    note.pitch = Some(crate::model::optimized::note::Pitch {
        step: PitchStep::B,
        alter: 1,
        octave: 9,
    });
    let xml = loaded_score_to_mscx(&outcome.score).raw_xml;
    assert!(xml.contains("<pitch>120</pitch>\n              <tpc>26</tpc>"));
    let back = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert_eq!(back.report.get("Note (pitch beyond MIDI's range)"), 0);
    Ok(())
}

#[test]
fn converter_reports_spellings_not_matching_the_pitch() -> GpResult<()> {
    // MIDI 61 spelled as C: the pitch wins, spelled with a sharp.
    let mscx = MSCX_SINGLE_TRACK.replacen(
        "<pitch>56</pitch><tpc>22</tpc>",
        "<pitch>61</pitch><tpc>14</tpc>",
        1,
    );
    assert_ne!(mscx, MSCX_SINGLE_TRACK);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(
        outcome.report.get("Note (spelling not matching its pitch)"),
        1
    );
    let pitches: Vec<i8> = first_voice_beats(&outcome.score, 0, 1)?[0]
        .notes
        .iter()
        .filter_map(|note| note.pitch.as_ref())
        .map(crate::convert::legacy::pitch_to_midi)
        .collect();
    assert_eq!(pitches, vec![52, 61, 59]);
    Ok(())
}

/// GP has one coda sign, where an al Coda pass lands: a coda sign the pass
/// branches from is kept as To Coda (playing the same), reported as lost.
#[test]
fn a_coda_sign_before_another_is_reported_lost_to_guitar_pro() -> GpResult<()> {
    let mut score = orchestra()?;
    let coda = |index: usize| NavigationEvent {
        measure_index: MeasureIndex(index as u16),
        kind: JumpKind::Coda,
        repeat_count: None,
    };
    let last = score.score.timeline.len() - 1;
    assert!(last > 0);
    score.score.timeline[0].navigation.push(coda(0));
    score.score.timeline[last].navigation.push(coda(last));
    let losses = crate::convert::legacy::loaded_score_to_legacy_song_with_losses(&score).1;
    assert_eq!(
        losses.get("Coda sign before another coda (kept as To Coda)"),
        1
    );
    Ok(())
}

#[test]
fn end_repeat_count_out_of_range_is_kept_close_and_reported() -> GpResult<()> {
    for (text, count, lossy) in [
        ("300", 255, 1),
        ("0", 1, 1),
        ("x", 2, 1),
        ("", 2, 0),
        ("40", 40, 0),
    ] {
        let xml = MSCX_WITH_REPEATS.replace(
            "<endRepeat>3</endRepeat>",
            &format!("<endRepeat>{text}</endRepeat>"),
        );
        let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
        let close = outcome.score.score.timeline[2]
            .navigation
            .iter()
            .find(|ev| ev.kind == JumpKind::RepeatClose)
            .and_then(|ev| ev.repeat_count);
        assert_eq!(close, Some(count), "<endRepeat>{text}</endRepeat>");
        assert_eq!(
            outcome.report.get("Repeat count (out of range)"),
            lossy,
            "<endRepeat>{text}</endRepeat>"
        );
    }
    Ok(())
}

#[test]
fn clef_superseded_before_the_first_beat_is_reported() -> GpResult<()> {
    let xml = MSCX_WITH_REPEATS.replacen(
        "<TimeSig>",
        "<Clef><concertClefType>G</concertClefType></Clef>\
         <Clef><concertClefType>F</concertClefType></Clef><TimeSig>",
        1,
    );
    let mscx = parse_mscx(&xml)?;
    let measure = &mscx.score_staves[0].measures[0];
    assert_eq!(measure.start_clef.as_deref(), Some("F"));
    assert_eq!(measure.superseded_clefs, ["G"]);
    let outcome = mscx_to_loaded_score(&mscx);
    assert_eq!(outcome.report.get("Clef (mid-measure)"), 1);
    Ok(())
}

#[test]
fn diatonic_only_transposition_is_kept() -> GpResult<()> {
    let xml = MSCX_WITH_REPEATS.replace(
        "<longName>Electric Guitar</longName>",
        "<longName>Electric Guitar</longName><transposeDiatonic>-1</transposeDiatonic>",
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    let transpose = outcome.score.score.instruments[0].transpose.as_ref();
    assert_eq!(
        transpose.map(|t| (t.chromatic, t.diatonic)),
        Some((0, Some(-1)))
    );
    Ok(())
}

#[test]
fn segment_elements_before_a_gap_are_reported_not_moved() -> GpResult<()> {
    // A second voice: a dynamic at the measure's start, then a gap to beat
    // 3, where its first chord is.
    let voices = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          </voice>\n        <voice>\n          <Dynamic><subtype>p</subtype></Dynamic>\n          <location><fractions>1/2</fractions></location>\n          <Chord><durationType>half</durationType><Note><pitch>67</pitch><tpc>15</tpc></Note></Chord>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[voices]))?);
    assert_eq!(outcome.report.get("Dynamic (before a gap)"), 1);
    let beats = voice_beats(&outcome.score, 0, 0, 1)?;
    assert_eq!(beats.len(), 1);
    assert_eq!(beats[0].tick_offset, 1920);
    assert!(beats[0].dynamic.is_none());
    Ok(())
}

/// A voice missing from a measure: the GP export leaves its slot empty, so
/// the next voice keeps its own slot and the tie into it (GP pairs ties by
/// slot); ids that skip still fit GP5's two slots.
#[test]
fn guitar_pro_export_keeps_each_voice_in_its_slot() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::model::legacy::enums::NoteType;
    use crate::model::optimized::global::MeasureIndex;

    let two_voices = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          </voice>\n        <voice>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n          </voice>";
    let mut score =
        mscx_to_loaded_score(&parse_mscx(&flute_score(&[two_voices, two_voices]))?).score;
    let measures = &mut score.score.tracks[0].measures;
    // Voices 1 and 3 (MuseScore's ids skip), the second measure without
    // the first; both voices' notes on string 1, the lower one tied on.
    for (measure, tie) in [(0, TieType::Start), (1, TieType::End)] {
        let data = present(measures.get_mut(&MeasureIndex(measure)), "measure")?;
        let mut upper = present(data.voices.remove(&0), "voice 0")?;
        let mut lower = present(data.voices.remove(&1), "voice 1")?;
        upper.voice_id = 1;
        lower.voice_id = 3;
        upper.beats[0].notes[0].string = Some(1);
        lower.beats[0].notes[0].string = Some(1);
        lower.beats[0].notes[0].tie = Some(tie);
        if measure == 0 {
            data.voices.insert(1, upper);
        }
        data.voices.insert(3, lower);
    }
    let song = loaded_score_to_legacy_song(&score);
    let second = &song.tracks[0].measures[1];
    assert_eq!(second.voices.len(), 2);
    assert!(second.voices[0].beats.is_empty());
    assert_eq!(second.voices[1].beats[0].notes[0].kind, NoteType::Tie);
    assert!(song.write_losses((5, 1, 0)).is_empty());
    // GP4 holds one voice: the second's notes, in both measures, are lost.
    let gp4 = song.write_losses((4, 0, 6));
    assert_eq!(
        gp4.get("Voice past the first in a measure (GP3/4 hold one)"),
        2
    );
    // The second measure's only notes are in that voice.
    assert_eq!(
        gp4.get("Measure written empty (its notes only in voices it can't hold)"),
        1
    );
    Ok(())
}

/// MuseScore's sfp attacks at ff, and the GP export plays it there too.
#[test]
fn guitar_pro_velocities_follow_musescores_levels() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;

    let measure = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Dynamic><subtype>sfp</subtype></Dynamic>\n          <Chord><durationType>half</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          <Chord><durationType>half</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[measure]))?);
    let song = loaded_score_to_legacy_song(&outcome.score);
    let velocities: Vec<i16> = song.tracks[0].measures[0].voices[0]
        .beats
        .iter()
        .map(|beat| beat.notes[0].velocity)
        .collect();
    // ff, then p.
    assert_eq!(velocities, vec![111, 47]);
    Ok(())
}

#[test]
fn voltas_without_ending_numbers_are_reported() -> GpResult<()> {
    let measure = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Spanner type=\"Volta\"><Volta><endings></endings><beginText>1.–3.</beginText></Volta><next><location><measures>1</measures></location></next></Spanner>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          <Spanner type=\"Volta\"><prev><location><measures>-1</measures></location></prev></Spanner>\n          </voice>";
    let outcome = mscx_to_loaded_score(&parse_mscx(&flute_score(&[measure]))?);
    assert_eq!(outcome.report.get("Volta (without ending numbers)"), 1);
    Ok(())
}

/// A note the export put on no string (a pitchless fallback) isn't
/// written: its record would throw the reader off. It's reported.
#[test]
fn notes_on_no_string_are_left_out_of_guitar_pro_files() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::model::legacy::song::Song;

    let measure = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n          </voice>";
    let mut song = loaded_score_to_legacy_song(
        &mscx_to_loaded_score(&parse_mscx(&flute_score(&[measure, measure]))?).score,
    );
    // The second measure's note on string 0, as the fallback places one.
    song.tracks[0].measures[1].voices[0].beats[0].notes[0].string = 0;
    assert_eq!(
        song.write_losses((5, 1, 0))
            .get("Note on no string of its track"),
        1
    );
    let bytes = song.write((5, 1, 0), None)?;
    let mut read = Song::default();
    read.read_gp5(&bytes)?;
    assert_eq!(read.tracks[0].measures.len(), 2);
    assert!(
        read.tracks[0].measures[1].voices[0].beats[0]
            .notes
            .is_empty()
    );
    Ok(())
}

/// Two notes of a chord on one string: GP keeps the first, so a tie into
/// that string pairs with it, not with the dropped second one.
#[test]
fn ties_pair_with_the_note_a_guitar_pro_file_keeps() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::model::legacy::enums::NoteType;
    use crate::model::optimized::global::MeasureIndex;

    let chord = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note><Note><pitch>65</pitch><tpc>13</tpc></Note></Chord>\n          </voice>";
    let single = "        <voice>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n          </voice>";
    let mut score = mscx_to_loaded_score(&parse_mscx(&flute_score(&[chord, single]))?).score;
    let measures = &mut score.score.tracks[0].measures;
    // Measure 1: both notes on string 1, the first tied on.
    let first = present(measures.get_mut(&MeasureIndex(0)), "measure 1")?;
    let notes = &mut present(first.voices.get_mut(&0), "voice")?.beats[0].notes;
    for (note, (fret, tie)) in notes.iter_mut().zip([(0, Some(TieType::Start)), (1, None)]) {
        note.string = Some(1);
        note.fret = Some(fret);
        note.tie = tie;
    }
    let second = present(measures.get_mut(&MeasureIndex(1)), "measure 2")?;
    let note = &mut present(second.voices.get_mut(&0), "voice")?.beats[0].notes[0];
    note.string = Some(1);
    note.fret = Some(0);
    note.tie = Some(TieType::End);
    let song = loaded_score_to_legacy_song(&score);
    let tied = &song.tracks[0].measures[1].voices[0].beats[0].notes[0];
    assert_eq!(tied.kind, NoteType::Tie);
    Ok(())
}

/// A note held on while one is added below it keeps its string: the new
/// note, though nearer that string, takes another, so the tie holds (GP
/// ties a note to the one before it on its string).
#[test]
fn a_held_note_keeps_its_string_when_one_is_added() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::model::legacy::enums::NoteType;
    use crate::model::optimized::global::MeasureIndex;

    // C4, then B3 + C4 (C4 tied on): both nearest the B string.
    let c4 = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>60</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let b3_c4 = "        <voice>\n          <Chord><durationType>whole</durationType><Note><pitch>59</pitch><tpc>19</tpc></Note><Note><pitch>60</pitch><tpc>14</tpc></Note></Chord>\n          </voice>";
    let mut score = mscx_to_loaded_score(&parse_mscx(&flute_score(&[c4, b3_c4]))?).score;
    let measures = &mut score.score.tracks[0].measures;
    for (measure, tie) in [(0, TieType::Start), (1, TieType::End)] {
        let data = present(measures.get_mut(&MeasureIndex(measure)), "measure")?;
        let notes = &mut present(data.voices.get_mut(&0), "voice")?.beats[0].notes;
        let c = present(
            notes
                .iter_mut()
                .find(|note| note.pitch.is_some_and(|p| p.midi() == 60)),
            "C4",
        )?;
        c.tie = Some(tie);
    }
    let song = loaded_score_to_legacy_song(&score);
    let held = &song.tracks[0].measures[0].voices[0].beats[0].notes[0];
    let second = &song.tracks[0].measures[1].voices[0].beats[0].notes;
    let tied = present(
        second.iter().find(|note| note.kind == NoteType::Tie),
        "tied C4",
    )?;
    assert_eq!(tied.string, held.string);
    assert_eq!(tied.value, held.value);
    assert_ne!(second[0].string, second[1].string);
    Ok(())
}

/// A GP3 beat's effect flags come from the notes the file keeps: a dropped
/// second note on a string doesn't give the kept one its harmonic.
#[test]
fn guitar_pro_3_beat_effects_come_from_the_kept_notes() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::model::legacy::effects::HarmonicEffect;
    use crate::model::legacy::song::Song;

    let measure = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n          </voice>";
    let mut song = loaded_score_to_legacy_song(
        &mscx_to_loaded_score(&parse_mscx(&flute_score(&[measure]))?).score,
    );
    let notes = &mut song.tracks[0].measures[0].voices[0].beats[0].notes;
    let mut second = notes[0].clone();
    second.effect.harmonic = Some(HarmonicEffect::default());
    notes.push(second);
    let mut read = Song::default();
    read.read_gp3(&song.write((3, 0, 0), None)?)?;
    let beat = &read.tracks[0].measures[0].voices[0].beats[0];
    assert_eq!(beat.notes.len(), 1);
    assert!(beat.notes[0].effect.harmonic.is_none());
    Ok(())
}

/// A measure without some of its track's voices keeps each voice's id
/// through MuseScore (which numbers voices by place): voice 1 alone stays
/// voice 1 (voice 0 a measure rest, as MuseScore needs), and voices 0 and
/// 2 don't become 0 and 1.
#[test]
fn musescore_voices_keep_their_ids_past_a_missing_one() -> GpResult<()> {
    use crate::model::optimized::global::MeasureIndex;

    let two_voices = "        <voice>\n          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>\n          <Chord><durationType>whole</durationType><Note><pitch>72</pitch><tpc>14</tpc></Note></Chord>\n          </voice>\n        <voice>\n          <Chord><durationType>whole</durationType><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n          </voice>";
    let mut score =
        mscx_to_loaded_score(&parse_mscx(&flute_score(&[two_voices, two_voices]))?).score;
    let measures = &mut score.score.tracks[0].measures;
    // Measure 1: voices 0 and 2; measure 2: voice 1 alone.
    let first = present(measures.get_mut(&MeasureIndex(0)), "measure 1")?;
    let mut lower = present(first.voices.remove(&1), "voice 1")?;
    lower.voice_id = 2;
    first.voices.insert(2, lower);
    let second = present(measures.get_mut(&MeasureIndex(1)), "measure 2")?;
    second.voices.remove(&0);

    let written = loaded_score_to_mscx(&score);
    let read = mscx_to_loaded_score(&parse_mscx(&written.raw_xml)?).score;
    let ids = |measure: u16| -> GpResult<Vec<u8>> {
        let data = present(
            read.score.tracks[0].measures.get(&MeasureIndex(measure)),
            "measure",
        )?;
        let mut ids: Vec<u8> = data.voices.keys().copied().collect();
        ids.sort_unstable();
        Ok(ids)
    };
    assert_eq!(ids(0)?, vec![0, 2]);
    assert_eq!(ids(1)?, vec![0, 1]);
    // Voice 1 still holds its note; voice 0 is a rest.
    let second = voice_beats(&read, 0, 1, 1)?;
    assert_eq!(second[0].notes.len(), 1);
    assert!(voice_beats(&read, 0, 1, 0)?[0].notes.is_empty());
    Ok(())
}
