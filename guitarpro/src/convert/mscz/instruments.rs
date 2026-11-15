//! MuseScore instrument identities for the [`Instrument`]s written by
//! [`super::from_optimized`].
//!
//! A MuseScore part names its instrument twice: `<Instrument id="…">`
//! picks the instrument template (`bb-clarinet`) and `<instrumentId>` the
//! sound MuseScore 4 plays it with (`wind.reed.clarinet.bflat`). A part
//! whose sound MuseScore doesn't know plays as a piano, so both are derived
//! here from [`Instrument::instrument_sound`] (with MusicXML's names for
//! MuseScore's sounds translated) or, failing that, the GM program.

use crate::model::optimized::global::{Instrument, InstrumentKind};

/// `(template id, sound id)` pairs of MuseScore 4's instrument list.
const INSTRUMENTS: &[(&str, &str)] = &[
    ("piano", "keyboard.piano"),
    ("electric-piano", "keyboard.piano.electric"),
    ("glockenspiel", "pitched-percussion.glockenspiel"),
    ("vibraphone", "pitched-percussion.vibraphone"),
    ("marimba", "pitched-percussion.marimba"),
    ("xylophone", "pitched-percussion.xylophone"),
    ("tubular-bells", "pitched-percussion.tubular-bells"),
    ("pipe-organ", "keyboard.organ.pipe"),
    ("guitar-nylon", "pluck.guitar.nylon-string"),
    ("guitar-steel", "pluck.guitar.acoustic"),
    ("electric-guitar", "pluck.guitar.electric"),
    ("acoustic-bass", "pluck.bass.acoustic"),
    ("electric-bass", "pluck.bass.electric"),
    ("violin", "strings.violin"),
    ("viola", "strings.viola"),
    ("violoncello", "strings.cello"),
    ("double-bass", "strings.contrabass"),
    ("violins", "strings.group"),
    ("harp", "pluck.harp"),
    ("timpani", "drum.timpani"),
    ("voice", "voice.vocals"),
    ("bb-trumpet", "brass.trumpet.bflat"),
    ("trombone", "brass.trombone"),
    ("tuba", "brass.tuba"),
    ("horn", "brass.french-horn"),
    ("soprano-saxophone", "wind.reed.saxophone.soprano"),
    ("alto-saxophone", "wind.reed.saxophone.alto"),
    ("tenor-saxophone", "wind.reed.saxophone.tenor"),
    ("baritone-saxophone", "wind.reed.saxophone.baritone"),
    ("oboe", "wind.reed.oboe"),
    ("english-horn", "wind.reed.english-horn"),
    ("bassoon", "wind.reed.bassoon"),
    ("bb-clarinet", "wind.reed.clarinet.bflat"),
    ("eb-clarinet", "wind.reed.clarinet.eflat"),
    ("bass-clarinet", "wind.reed.clarinet.bass"),
    ("piccolo", "wind.flutes.flute.piccolo"),
    ("flute", "wind.flutes.flute"),
    ("drumset", "drum.group.set"),
];

/// MusicXML sound ids (as a MusicXML import carries them) that MuseScore
/// knows by another of [`INSTRUMENTS`]' sounds.
const SOUND_ALIASES: &[(&str, &str)] = &[
    ("keyboard.piano.grand", "keyboard.piano"),
    ("keyboard.piano.upright", "keyboard.piano"),
    ("pluck.guitar.steel-string", "pluck.guitar.acoustic"),
];

/// The `(template id, sound id)` MuseScore should use for `instrument`.
///
/// A known `instrument_sound` (or a MusicXML alias of one) wins; any other
/// is kept as the sound, as MuseScore knows far more sounds than
/// [`INSTRUMENTS`] lists, with the template picked from the program. Percussion always takes the
/// drum set template, whose notes are drum-map pitches (and its sound,
/// without one of its own or with a pitched one).
pub(crate) fn mscx_instrument(instrument: &Instrument) -> (&'static str, String) {
    let percussion = matches!(instrument.kind, InstrumentKind::Percussion);
    let (id, default_sound) = if percussion {
        ("drumset", "drum.group.set")
    } else {
        from_program(instrument)
    };
    match instrument.instrument_sound.as_deref() {
        // A pitched sound would play the drum set's drum-map pitches.
        Some(sound) if !sound.is_empty() && !(percussion && is_pitched_sound(sound)) => {
            let sound = SOUND_ALIASES
                .iter()
                .find(|(alias, _)| *alias == sound)
                .map_or(sound, |(_, known)| *known);
            let id = INSTRUMENTS
                .iter()
                .find(|(_, known)| !percussion && *known == sound)
                .map_or(id, |(known_id, _)| *known_id);
            (id, sound.to_string())
        }
        _ => (id, default_sound.to_string()),
    }
}

/// Whether a sound id is of a pitched instrument family (MuseScore's and
/// MusicXML's ids start with their family: `pluck.guitar.electric`).
fn is_pitched_sound(sound: &str) -> bool {
    let family = sound.split('.').next().unwrap_or_default();
    matches!(
        family,
        "keyboard" | "pluck" | "strings" | "brass" | "wind" | "voice" | "pitched-percussion"
    )
}

/// By General MIDI program, with clarinets told apart by transposition.
fn from_program(instrument: &Instrument) -> (&'static str, &'static str) {
    let chromatic = instrument
        .transpose
        .as_ref()
        .map_or(0, |t| t.folded().chromatic);
    let name = match instrument.midi_program {
        4 | 5 => "electric-piano",
        9 | 10 | 15 => "glockenspiel",
        8 | 11 => "vibraphone",
        12 => "marimba",
        13 => "xylophone",
        14 => "tubular-bells",
        16..=20 => "pipe-organ",
        24 => "guitar-nylon",
        25 => "guitar-steel",
        26..=31 => "electric-guitar",
        32 => "acoustic-bass",
        33..=39 => "electric-bass",
        40 => "violin",
        41 => "viola",
        42 => "violoncello",
        43 => "double-bass",
        44 | 45 | 48..=51 => "violins",
        46 => "harp",
        47 => "timpani",
        52..=54 => "voice",
        56 | 59 => "bb-trumpet",
        57 | 61..=63 => "trombone",
        58 => "tuba",
        60 => "horn",
        64 => "soprano-saxophone",
        65 => "alto-saxophone",
        66 => "tenor-saxophone",
        67 => "baritone-saxophone",
        68 => "oboe",
        69 => "english-horn",
        70 => "bassoon",
        71 if chromatic == 3 => "eb-clarinet",
        71 if chromatic == -14 => "bass-clarinet",
        71 => "bb-clarinet",
        72 => "piccolo",
        73..=79 => "flute",
        _ => "piano",
    };
    INSTRUMENTS
        .iter()
        .find(|(id, _)| *id == name)
        .copied()
        .unwrap_or(("piano", "keyboard.piano"))
}
