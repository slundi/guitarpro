//! MuseScore instrument template and sound picked for an `Instrument`.

use crate::convert::mscz::instruments::mscx_instrument;
use crate::model::optimized::global::{Instrument, InstrumentKind, Transpose};

fn instrument(program: u8, sound: Option<&str>, chromatic: i16) -> Instrument {
    instrument_of_kind(InstrumentKind::Pitched, program, sound, chromatic)
}

fn instrument_of_kind(
    kind: InstrumentKind,
    program: u8,
    sound: Option<&str>,
    chromatic: i16,
) -> Instrument {
    Instrument {
        name: String::new(),
        abbreviation: None,
        instrument_sound: sound.map(str::to_string),
        midi_channel: 0,
        midi_program: program,
        midi_bank: None,
        volume: None,
        pan: None,
        kind,
        transpose: (chromatic != 0).then_some(Transpose {
            diatonic: None,
            chromatic,
            octave_change: None,
        }),
        gp_strings: Vec::new(),
    }
}

#[test]
fn picks_instruments_by_sound_then_program() {
    let id = |i: &Instrument| mscx_instrument(i).0;
    assert_eq!(id(&instrument(0, Some("wind.reed.oboe"), 0)), "oboe");
    assert_eq!(id(&instrument(71, None, -2)), "bb-clarinet");
    assert_eq!(id(&instrument(71, None, 3)), "eb-clarinet");
    assert_eq!(
        mscx_instrument(&instrument(42, None, 0)),
        ("violoncello", "strings.cello".to_string())
    );
    // An unknown sound is kept; the template comes from the program.
    assert_eq!(
        mscx_instrument(&instrument(73, Some("wind.flutes.flute.alto"), -5)),
        ("flute", "wind.flutes.flute.alto".to_string())
    );
    assert_eq!(id(&instrument(127, None, 0)), "piano");
}

#[test]
fn musicxml_names_for_musescore_sounds_are_translated() {
    assert_eq!(
        mscx_instrument(&instrument(25, Some("pluck.guitar.steel-string"), 0)),
        ("guitar-steel", "pluck.guitar.acoustic".to_string())
    );
}

#[test]
fn percussion_keeps_the_drum_set_template() {
    let drums = |sound| instrument_of_kind(InstrumentKind::Percussion, 0, sound, 0);
    assert_eq!(
        mscx_instrument(&drums(None)),
        ("drumset", "drum.group.set".to_string())
    );
    // An unknown drum sound is kept, but not with a pitched template.
    assert_eq!(
        mscx_instrument(&drums(Some("drum.snare-drum"))),
        ("drumset", "drum.snare-drum".to_string())
    );
    // Nor is a known pitched one: the notes are drum-map pitches.
    assert_eq!(
        mscx_instrument(&drums(Some("drum.timpani"))),
        ("drumset", "drum.timpani".to_string())
    );
    // A pitched sound would play the drum-map pitches: the drum set's.
    assert_eq!(
        mscx_instrument(&drums(Some("pitched-percussion.xylophone"))),
        ("drumset", "drum.group.set".to_string())
    );
    assert_eq!(
        mscx_instrument(&drums(Some("pluck.guitar.electric"))),
        ("drumset", "drum.group.set".to_string())
    );
}

#[test]
fn a_separate_octave_change_counts_toward_the_transposition() {
    // A bass clarinet as MusicXML writes it: a major second and an octave.
    let mut bass_clarinet = instrument(71, None, -2);
    bass_clarinet.transpose = Some(Transpose {
        diatonic: Some(-1),
        chromatic: -2,
        octave_change: Some(-1),
    });
    assert_eq!(mscx_instrument(&bass_clarinet).0, "bass-clarinet");
    assert_eq!(
        bass_clarinet.transpose.map(|t| t.folded()),
        Some(Transpose {
            diatonic: Some(-8),
            chromatic: -14,
            octave_change: None,
        })
    );
}

#[test]
fn a_drum_set_is_written_on_a_percussion_staff() -> crate::error::GpResult<()> {
    use crate::convert::mscz::loaded_score_to_mscx;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::song::Song;
    use crate::tests::common::read_file;

    let mut song = Song::default();
    song.read_gp5(&read_file("test/all-percussion.gp5".to_string()))?;
    let xml = loaded_score_to_mscx(&legacy_song_to_loaded_score(&song)).raw_xml;
    assert!(xml.contains(r#"<Instrument id="drumset">"#));
    assert!(xml.contains("<StaffType group=\"percussion\">\n          <name>perc5Line</name>"));
    assert!(!xml.contains(r#"<StaffType group="pitched">"#));
    Ok(())
}
