//! Conversion from `optimized::LoadedScore` to `musicxml::ScorePartwise`.
//!
//! Entry point: [`loaded_score_to_score_partwise`].
//!
//! This is the reverse of [`crate::convert::optimized::score_partwise_to_loaded_score`].
//! It converts the compact `optimized` model directly into a MusicXML
//! `score-partwise` document, **without** routing through the legacy `Song`
//! model or any other intermediate representation.
//!
//! The conversion is intentionally focused on the core musical content that the
//! two models share — score header, part list, per-measure attributes
//! (divisions, key, time, clef), tempo, and notes (pitch/rest, duration, dots,
//! tuplet, voice, chord, tie). Notation detail that the optimized model carries
//! but that does not survive the forward converter's own round-trip (effects,
//! techniques, ornaments, tab tunings, lyrics, …) is not re-emitted. The result
//! is therefore lossy, but it reaches a **fixed point**: converting a document
//! `musicxml → optimized → musicxml` once and then again yields identical output.

use crate::{
    convert::optimized::timeline::key_sig_from_fifths,
    model::{
        musicxml,
        optimized::{
            LoadedScore,
            beat::{Beat, Duration, Tuplet, Voice},
            global::{Instrument, Score},
            metadata::{KeySignature, Metadata, Mode},
            note::{NoteValue, Pitch as OptPitch, PitchStep, TieType},
            timeline::MeasureDef,
            track::{Clef as OptClef, Track},
        },
    },
};

/// Divisions per quarter note emitted throughout the document.
///
/// A fixed constant: the value is not derived from the source model, so both the
/// first and second `musicxml → optimized → musicxml` passes emit the same
/// `<divisions>` and note durations (guaranteeing idempotency).
const DIVISIONS: u32 = 480;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Convert an [`LoadedScore`] into a MusicXML [`ScorePartwise`] document.
pub fn loaded_score_to_score_partwise(loaded: &LoadedScore) -> musicxml::ScorePartwise {
    let score = &loaded.score;

    musicxml::ScorePartwise {
        version: Some("4.0".to_string()),
        work: build_work(&score.metadata),
        movement_number: score.metadata.movement_number.clone(),
        movement_title: None,
        identification: score
            .metadata
            .identification
            .as_ref()
            .map(build_identification),
        defaults: None,
        credits: vec![],
        part_list: build_part_list(score),
        parts: build_parts(score),
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn build_work(metadata: &Metadata) -> Option<musicxml::Work> {
    let title = if metadata.title.is_empty() {
        None
    } else {
        Some(metadata.title.clone())
    };
    let number = metadata.work.as_ref().and_then(|work| work.number.clone());
    if title.is_none() && number.is_none() {
        return None;
    }
    Some(musicxml::Work {
        work_number: number,
        work_title: title,
        opus: None,
    })
}

fn build_identification(
    id: &crate::model::optimized::metadata::Identification,
) -> musicxml::identification::Identification {
    use musicxml::identification::{Creator, Encoding, Identification, MiscellaneousField, Rights};

    let creators = id
        .creators
        .iter()
        .map(|creator| Creator {
            creator_type: Some(creator.role.clone()),
            value: creator.name.clone(),
        })
        .collect();

    let rights = id
        .rights
        .iter()
        .map(|value| Rights {
            rights_type: None,
            value: value.clone(),
        })
        .collect::<Vec<_>>();

    let encoding = if id.encoding_software.is_some() || id.encoding_date.is_some() {
        Some(Encoding {
            encoding_date: id.encoding_date.clone(),
            encoders: vec![],
            software: id.encoding_software.clone().into_iter().collect(),
            encoding_description: vec![],
            supports: vec![],
        })
    } else {
        None
    };

    let miscellaneous = if id.miscellaneous.is_empty() {
        None
    } else {
        Some(musicxml::identification::Miscellaneous {
            fields: id
                .miscellaneous
                .iter()
                .map(|(name, value)| MiscellaneousField {
                    name: name.clone(),
                    value: value.clone(),
                })
                .collect(),
        })
    };

    Identification {
        creators,
        rights,
        encoding,
        source: id.source.clone(),
        relations: vec![],
        miscellaneous,
    }
}

// ---------------------------------------------------------------------------
// Part list
// ---------------------------------------------------------------------------

fn build_part_list(score: &Score) -> musicxml::part_list::PartList {
    use musicxml::part_list::{MidiInstrument, PartListItem, PartName, ScoreInstrument, ScorePart};

    let items = score
        .tracks
        .iter()
        .enumerate()
        .map(|(track_idx, track)| {
            let part_id = format!("P{}", track_idx + 1);
            let instrument_id = format!("P{}-I1", track_idx + 1);
            let instrument: Option<&Instrument> =
                score.instruments.get(track.instrument.0 as usize);

            let midi_instrument = MidiInstrument {
                id: instrument_id.clone(),
                // optimized MIDI channel/program are 0-based; MusicXML is 1-based.
                midi_channel: Some(
                    instrument
                        .map(|ins| ins.midi_channel)
                        .unwrap_or(0)
                        .saturating_add(1),
                ),
                midi_name: None,
                midi_bank: instrument.and_then(|ins| ins.midi_bank),
                midi_program: Some(
                    instrument
                        .map(|ins| ins.midi_program)
                        .unwrap_or(0)
                        .saturating_add(1),
                ),
                midi_unpitched: None,
                volume: None,
                pan: None,
                elevation: None,
            };

            let abbreviation = instrument
                .and_then(|ins| ins.abbreviation.clone())
                .filter(|s| !s.is_empty());

            PartListItem::ScorePart(ScorePart {
                id: part_id,
                identification: None,
                part_name: Some(PartName {
                    print_object: None,
                    justify: None,
                    value: Some(track.name.clone()),
                }),
                part_name_display: None,
                part_abbreviation: abbreviation.map(|value| PartName {
                    print_object: None,
                    justify: None,
                    value: Some(value),
                }),
                part_abbreviation_display: None,
                groups: vec![],
                score_instruments: vec![ScoreInstrument {
                    id: instrument_id,
                    instrument_name: track.name.clone(),
                    instrument_abbreviation: None,
                    instrument_sound: None,
                    solo: None,
                    ensemble: None,
                }],
                players: vec![],
                midi_devices: vec![],
                midi_instruments: vec![midi_instrument],
            })
        })
        .collect();

    musicxml::part_list::PartList { items }
}

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

fn build_parts(score: &Score) -> Vec<musicxml::Part> {
    let measure_count = score.timeline.len();

    score
        .tracks
        .iter()
        .enumerate()
        .map(|(track_idx, track)| {
            let measures = (0..measure_count)
                .map(|measure_idx| build_measure(score, track, track_idx, measure_idx))
                .collect();
            musicxml::Part {
                id: format!("P{}", track_idx + 1),
                measures,
            }
        })
        .collect()
}

fn build_measure(
    score: &Score,
    track: &Track,
    track_idx: usize,
    measure_idx: usize,
) -> musicxml::measure::Measure {
    use musicxml::measure::MusicData;

    let measure_def = score.timeline.get(measure_idx);
    let mut music_data: Vec<MusicData> = vec![];

    // --- Attributes: divisions + clefs on measure 0; key/time on any change ---
    if let Some(attrs) = build_attributes(score, track, measure_idx, measure_def) {
        music_data.push(MusicData::Attributes(attrs));
    }

    // --- Tempo (first track only, when the timeline records a change) ---
    if track_idx == 0
        && let Some(tempo) = measure_def.and_then(|md| md.tempo)
    {
        music_data.push(MusicData::Sound(make_sound_tempo(tempo)));
    }

    // --- Voices → notes ---
    music_data.extend(build_voices(track, measure_idx));

    let number = measure_def
        .map(|md| md.index.0 + 1)
        .unwrap_or((measure_idx + 1) as u16);

    musicxml::measure::Measure {
        number: number.to_string(),
        implicit: None,
        non_controlling: None,
        width: None,
        text: None,
        id: None,
        music_data,
    }
}

fn build_attributes(
    score: &Score,
    track: &Track,
    measure_idx: usize,
    measure_def: Option<&MeasureDef>,
) -> Option<musicxml::attributes::Attributes> {
    use musicxml::attributes::{Attributes, Key, Time};

    let is_first = measure_idx == 0;
    let time_sig = measure_def.and_then(|md| md.time_signature);
    let key_sig = measure_def.and_then(|md| md.key_signature);

    // Nothing to emit unless it's the first measure or a signature change.
    if !is_first && time_sig.is_none() && key_sig.is_none() {
        return None;
    }

    let keys = key_sig
        .map(|ks| {
            vec![Key {
                number: None,
                print_object: None,
                cancel: None,
                fifths: Some(fifths_from_key_sig(&ks)),
                mode: Some(mode_str(ks.mode).to_string()),
                key_steps: vec![],
                key_alters: vec![],
                key_accidentals: vec![],
                key_octaves: vec![],
            }]
        })
        .unwrap_or_default();

    let times = time_sig
        .map(|ts| {
            vec![Time {
                number: None,
                symbol: None,
                separator: None,
                print_object: None,
                beats: Some(ts.numerator.to_string()),
                beat_type: Some(ts.denominator.to_string()),
                senza_misura: None,
                interchangeable: None,
            }]
        })
        .unwrap_or_default();

    let clefs = if is_first {
        build_clefs(score, track)
    } else {
        vec![]
    };

    Some(Attributes {
        divisions: if is_first { Some(DIVISIONS) } else { None },
        keys,
        times,
        staves: None,
        part_symbol: None,
        instruments: None,
        clefs,
        staff_details: vec![],
        transposes: vec![],
        for_parts: vec![],
        directives: vec![],
        measure_styles: vec![],
    })
}

fn build_clefs(score: &Score, track: &Track) -> Vec<musicxml::attributes::Clef> {
    let multi = track.staves.len() > 1;
    track
        .staves
        .iter()
        .enumerate()
        .filter_map(|(staff_idx, staff_id)| {
            let staff = score.staves.get(staff_id.0 as usize)?;
            let number = if multi {
                Some((staff_idx + 1) as u8)
            } else {
                None
            };
            Some(make_clef(staff.clef, number))
        })
        .collect()
}

fn make_clef(clef: OptClef, number: Option<u8>) -> musicxml::attributes::Clef {
    let (sign, line) = match clef {
        OptClef::Treble => ("G", Some(2)),
        OptClef::Bass => ("F", Some(4)),
        OptClef::Alto => ("C", Some(3)),
        OptClef::Tenor => ("C", Some(4)),
        OptClef::Percussion => ("percussion", None),
        OptClef::Tab => ("TAB", None),
    };
    musicxml::attributes::Clef {
        number,
        additional: None,
        size: None,
        after_barline: None,
        print_object: None,
        sign: sign.to_string(),
        line,
        clef_octave_change: None,
    }
}

// ---------------------------------------------------------------------------
// Voices → notes
// ---------------------------------------------------------------------------

fn build_voices(track: &Track, measure_idx: usize) -> Vec<musicxml::measure::MusicData> {
    use musicxml::measure::{Backup, MusicData};

    let mut result: Vec<MusicData> = vec![];

    let Some(measure_data) = track
        .measures
        .get(&crate::model::optimized::global::MeasureIndex(
            measure_idx as u16,
        ))
    else {
        return result;
    };

    // Deterministic voice order (HashMap iteration is unordered).
    let mut voice_ids: Vec<u8> = measure_data.voices.keys().copied().collect();
    voice_ids.sort_unstable();

    let mut prev_voice_ticks: u32 = 0;
    for (voice_pos, voice_id) in voice_ids.iter().enumerate() {
        let Some(voice) = measure_data.voices.get(voice_id) else {
            continue;
        };

        // Rewind the time cursor to the measure start before voices 2+.
        if voice_pos > 0 && prev_voice_ticks > 0 {
            result.push(MusicData::Backup(Backup {
                duration: prev_voice_ticks,
            }));
        }

        let voice_num = (voice_pos + 1).to_string();
        prev_voice_ticks = emit_voice(voice, &voice_num, &mut result);
    }

    result
}

/// Emit one voice's beats as note events, returning the total ticks consumed.
fn emit_voice(voice: &Voice, voice_num: &str, out: &mut Vec<musicxml::measure::MusicData>) -> u32 {
    use musicxml::measure::MusicData;

    let mut total_ticks: u32 = 0;
    let mut beats: Vec<&Beat> = voice.beats.iter().collect();
    beats.sort_by_key(|beat| beat.tick_offset);

    for beat in beats {
        let divisions = duration_divisions(&beat.duration);
        total_ticks += divisions;

        if is_rest(beat) {
            out.push(MusicData::Note(make_note(
                None,
                false,
                false,
                divisions,
                &beat.duration,
                None,
                voice_num,
            )));
            continue;
        }

        for (note_idx, note) in beat.notes.iter().enumerate() {
            let Some(pitch) = note.pitch else {
                continue;
            };
            out.push(MusicData::Note(make_note(
                Some(pitch),
                false,
                note_idx > 0, // chord flag on all but the first note
                divisions,
                &beat.duration,
                note.tie,
                voice_num,
            )));
        }
    }

    total_ticks
}

/// A beat renders as a rest when it has no notes, or none of its notes carries a
/// pitch (the forward converter represents rests as pitch-less notes).
fn is_rest(beat: &Beat) -> bool {
    beat.gp_rest || beat.notes.is_empty() || beat.notes.iter().all(|note| note.pitch.is_none())
}

#[allow(clippy::too_many_arguments)]
fn make_note(
    pitch: Option<OptPitch>,
    _cue: bool,
    is_chord: bool,
    divisions: u32,
    duration: &Duration,
    tie: Option<TieType>,
    voice: &str,
) -> musicxml::note::Note {
    use musicxml::note::{Dot, NoteType, Rest, Tie};

    let (pitch_out, rest_out) = match pitch {
        Some(pitch) => (Some(pitch_out(&pitch)), None),
        None => (
            None,
            Some(Rest {
                measure: None,
                display_step: None,
                display_octave: None,
            }),
        ),
    };

    let ties = match tie {
        Some(TieType::Start) => vec![Tie {
            tie_type: "start".to_string(),
            time_only: None,
        }],
        Some(TieType::End) => vec![Tie {
            tie_type: "stop".to_string(),
            time_only: None,
        }],
        None => vec![],
    };

    let dots = (0..duration.dots)
        .map(|_| Dot {
            default_x: None,
            default_y: None,
            placement: None,
        })
        .collect();

    musicxml::note::Note {
        grace: None,
        cue: None,
        pitch: pitch_out,
        rest: rest_out,
        unpitched: None,
        chord: if is_chord { Some(()) } else { None },
        duration: Some(divisions),
        ties,
        footnote: None,
        level: None,
        instrument: None,
        voice: Some(voice.to_string()),
        note_type: Some(NoteType {
            size: None,
            value: note_value_to_type(duration.base),
        }),
        dots,
        accidental: None,
        time_modification: build_time_modification(duration),
        stem: None,
        notehead: None,
        notehead_text: None,
        staff: None,
        beams: vec![],
        notations: vec![],
        lyrics: vec![],
        play: None,
        listen: None,
        print_object: None,
        print_dot: None,
        print_spacing: None,
        print_lyric: None,
        dynamics: None,
        end_dynamics: None,
        attack: None,
        release_time: None,
        default_x: None,
        default_y: None,
        time_only: None,
        pizzicato: None,
        id: None,
    }
}

fn make_sound_tempo(tempo: f32) -> musicxml::direction::Sound {
    musicxml::direction::Sound {
        tempo: Some(tempo as f64),
        dynamics: None,
        dacapo: None,
        segno: None,
        dalsegno: None,
        coda: None,
        tocoda: None,
        divisions: None,
        forward_repeat: None,
        fine: None,
        time_only: None,
        pizzicato: None,
        pan: None,
        elevation: None,
        damper_pedal: None,
        soft_pedal: None,
        sostenuto_pedal: None,
        id: None,
        midi_devices: vec![],
        midi_instruments: vec![],
        plays: vec![],
        swing: None,
        offset: None,
    }
}

// ---------------------------------------------------------------------------
// Pitch / duration / key helpers
// ---------------------------------------------------------------------------

fn pitch_out(pitch: &OptPitch) -> musicxml::note::Pitch {
    let step = match pitch.step {
        PitchStep::C => "C",
        PitchStep::D => "D",
        PitchStep::E => "E",
        PitchStep::F => "F",
        PitchStep::G => "G",
        PitchStep::A => "A",
        PitchStep::B => "B",
    };
    musicxml::note::Pitch {
        step: step.to_string(),
        alter: if pitch.alter != 0 {
            Some(pitch.alter as f64)
        } else {
            None
        },
        octave: pitch.octave as i8,
    }
}

/// Note-value denominator: Whole = 1, Half = 2, Quarter = 4, …
fn note_value_number(value: NoteValue) -> u32 {
    match value {
        NoteValue::Whole => 1,
        NoteValue::Half => 2,
        NoteValue::Quarter => 4,
        NoteValue::Eighth => 8,
        NoteValue::Sixteenth => 16,
        NoteValue::ThirtySecond => 32,
        NoteValue::SixtyFourth => 64,
        NoteValue::HundredTwentyEighth => 128,
        NoteValue::Other(n) => (n as u32).max(1),
    }
}

/// Duration in `<divisions>` ticks, applying dots and tuplet scaling.
fn duration_divisions(duration: &Duration) -> u32 {
    let base = (DIVISIONS * 4) / note_value_number(duration.base);

    // Dots: each dot adds half of the running value.
    let mut ticks = base;
    let mut increment = base;
    for _ in 0..duration.dots {
        increment /= 2;
        ticks += increment;
    }

    if let Some(Tuplet { actual, normal }) = duration.tuplet
        && actual > 0
    {
        ticks = ticks * normal as u32 / actual as u32;
    }
    ticks
}

fn note_value_to_type(value: NoteValue) -> musicxml::note::NoteTypeValue {
    use musicxml::note::NoteTypeValue as T;
    match value {
        NoteValue::Whole => T::Whole,
        NoteValue::Half => T::Half,
        NoteValue::Quarter => T::Quarter,
        NoteValue::Eighth => T::Eighth,
        NoteValue::Sixteenth => T::N16th,
        NoteValue::ThirtySecond => T::N32nd,
        NoteValue::SixtyFourth => T::N64th,
        NoteValue::HundredTwentyEighth => T::N128th,
        NoteValue::Other(_) => T::Quarter,
    }
}

fn build_time_modification(duration: &Duration) -> Option<musicxml::note::TimeModification> {
    let Tuplet { actual, normal } = duration.tuplet?;
    Some(musicxml::note::TimeModification {
        actual_notes: actual,
        normal_notes: normal,
        normal_type: None,
        normal_dots: vec![],
    })
}

fn mode_str(mode: Mode) -> &'static str {
    match mode {
        Mode::Major => "major",
        Mode::Minor => "minor",
        Mode::Dorian => "dorian",
        Mode::Phrygian => "phrygian",
        Mode::Lydian => "lydian",
        Mode::Mixolydian => "mixolydian",
        Mode::Locrian => "locrian",
    }
}

/// Inverse of [`key_sig_from_fifths`]: find the fifths value in −7..=7 that
/// reproduces this key signature. Reusing the forward function guarantees the
/// mapping is a consistent round-trip.
fn fifths_from_key_sig(key_sig: &KeySignature) -> i8 {
    let mode = mode_str(key_sig.mode);
    (-7..=7)
        .find(|&fifths| key_sig_from_fifths(fifths, Some(mode)) == *key_sig)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn dur(base: NoteValue, dots: u8, tuplet: Option<Tuplet>) -> Duration {
        Duration { base, dots, tuplet }
    }

    #[test]
    fn note_value_number_maps_denominators() {
        assert_eq!(note_value_number(NoteValue::Whole), 1);
        assert_eq!(note_value_number(NoteValue::Quarter), 4);
        assert_eq!(note_value_number(NoteValue::SixtyFourth), 64);
        assert_eq!(note_value_number(NoteValue::Other(3)), 3);
        // Other(0) must not become a divide-by-zero.
        assert_eq!(note_value_number(NoteValue::Other(0)), 1);
    }

    #[test]
    fn quarter_note_is_one_division_unit() {
        assert_eq!(
            duration_divisions(&dur(NoteValue::Quarter, 0, None)),
            DIVISIONS
        );
    }

    #[test]
    fn whole_note_is_four_division_units() {
        assert_eq!(
            duration_divisions(&dur(NoteValue::Whole, 0, None)),
            DIVISIONS * 4
        );
    }

    #[test]
    fn dotted_quarter_adds_half() {
        assert_eq!(
            duration_divisions(&dur(NoteValue::Quarter, 1, None)),
            DIVISIONS + DIVISIONS / 2
        );
    }

    #[test]
    fn double_dotted_quarter_adds_half_then_quarter() {
        assert_eq!(
            duration_divisions(&dur(NoteValue::Quarter, 2, None)),
            DIVISIONS + DIVISIONS / 2 + DIVISIONS / 4
        );
    }

    #[test]
    fn triplet_eighth_scales_by_two_thirds() {
        // Eighth = DIVISIONS/2; triplet 3:2 → * 2 / 3.
        let tuplet = Tuplet {
            actual: 3,
            normal: 2,
        };
        let base = DIVISIONS / 2;
        assert_eq!(
            duration_divisions(&dur(NoteValue::Eighth, 0, Some(tuplet))),
            base * 2 / 3
        );
    }

    #[test]
    fn note_value_to_type_roundtrips_supported_values() {
        use musicxml::note::NoteTypeValue as T;
        assert_eq!(note_value_to_type(NoteValue::Whole), T::Whole);
        assert_eq!(note_value_to_type(NoteValue::Sixteenth), T::N16th);
        assert_eq!(note_value_to_type(NoteValue::ThirtySecond), T::N32nd);
        assert_eq!(note_value_to_type(NoteValue::SixtyFourth), T::N64th);
    }

    #[test]
    fn pitch_out_natural_has_no_alter() {
        let pitch = OptPitch {
            step: PitchStep::C,
            alter: 0,
            octave: 4,
        };
        let out = pitch_out(&pitch);
        assert_eq!(out.step, "C");
        assert_eq!(out.octave, 4);
        assert!(out.alter.is_none());
    }

    #[test]
    fn pitch_out_sharp_has_positive_alter() {
        let pitch = OptPitch {
            step: PitchStep::F,
            alter: 1,
            octave: 5,
        };
        let out = pitch_out(&pitch);
        assert_eq!(out.step, "F");
        assert_eq!(out.alter, Some(1.0));
        assert_eq!(out.octave, 5);
    }

    #[test]
    fn time_modification_none_without_tuplet() {
        assert!(build_time_modification(&dur(NoteValue::Quarter, 0, None)).is_none());
    }

    #[test]
    fn time_modification_some_with_tuplet() {
        let tuplet = Tuplet {
            actual: 3,
            normal: 2,
        };
        let tm = build_time_modification(&dur(NoteValue::Eighth, 0, Some(tuplet))).unwrap();
        assert_eq!(tm.actual_notes, 3);
        assert_eq!(tm.normal_notes, 2);
    }

    #[test]
    fn fifths_from_key_sig_inverts_major_keys() {
        // Every major key signature must round-trip through the forward mapping.
        for fifths in -7..=7i8 {
            let key_sig = key_sig_from_fifths(fifths, Some("major"));
            assert_eq!(
                fifths_from_key_sig(&key_sig),
                fifths,
                "major fifths={fifths}"
            );
        }
    }

    #[test]
    fn fifths_from_key_sig_inverts_minor_keys() {
        for fifths in -7..=7i8 {
            let key_sig = key_sig_from_fifths(fifths, Some("minor"));
            assert_eq!(
                fifths_from_key_sig(&key_sig),
                fifths,
                "minor fifths={fifths}"
            );
        }
    }

    #[test]
    fn clef_signs_map_correctly() {
        assert_eq!(make_clef(OptClef::Treble, None).sign, "G");
        assert_eq!(make_clef(OptClef::Bass, None).sign, "F");
        assert_eq!(make_clef(OptClef::Tab, Some(2)).sign, "TAB");
        assert_eq!(make_clef(OptClef::Tab, Some(2)).number, Some(2));
        assert_eq!(make_clef(OptClef::Percussion, None).sign, "percussion");
    }
}
