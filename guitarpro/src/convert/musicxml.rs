//! Conversion from `legacy::Song` to `musicxml::ScorePartwise`.
//!
//! Entry point: [`song_to_score_partwise`].

use crate::model::{
    legacy::{
        key_signature::{DURATION_QUARTER_TIME, Duration},
        song::Song,
    },
    musicxml::{self, note::NoteTypeValue},
};

/// Divisions per quarter note used throughout the output document.
///
/// Matches `DURATION_QUARTER_TIME` so legacy tick values map 1-to-1.
pub const DIVISIONS: u32 = DURATION_QUARTER_TIME as u32;

// ---------------------------------------------------------------------------
// Pitch helpers
// ---------------------------------------------------------------------------

/// Convert a MIDI note number to a MusicXML pitch triple `(step, alter, octave)`.
///
/// Uses sharps for accidentals (alter = 1.0).
/// MIDI 60 = C4 (middle C).
pub fn midi_to_pitch(midi: i8) -> (String, Option<f64>, i8) {
    // MIDI octave: octave = (midi / 12) - 1, but we need signed-safe arithmetic
    let midi_u = midi as i32;
    let pitch_class = midi_u.rem_euclid(12) as u8;
    let octave = (midi_u / 12 - 1) as i8;

    let (step, alter) = match pitch_class {
        0 => ("C", None),
        1 => ("C", Some(1.0)),
        2 => ("D", None),
        3 => ("D", Some(1.0)),
        4 => ("E", None),
        5 => ("F", None),
        6 => ("F", Some(1.0)),
        7 => ("G", None),
        8 => ("G", Some(1.0)),
        9 => ("A", None),
        10 => ("A", Some(1.0)),
        11 => ("B", None),
        _ => unreachable!(),
    };

    (step.to_string(), alter, octave)
}

// ---------------------------------------------------------------------------
// Duration helpers
// ---------------------------------------------------------------------------

/// Convert a legacy [`Duration`] to MusicXML `<divisions>` ticks.
///
/// Result is expressed in units of [`DIVISIONS`] per quarter note.
/// Dotted and tuplet adjustments are applied.
pub fn duration_to_divisions(d: &Duration) -> u32 {
    // Base ticks for this note value (quarter = DIVISIONS)
    let base = (DIVISIONS * 4) / d.value as u32;

    // Dotted: multiply by 3/2
    let dotted = if d.dotted { base + base / 2 } else { base };

    // Tuplet: scale by tuplet_times / tuplet_enters
    if d.tuplet_enters != 1 || d.tuplet_times != 1 {
        dotted * d.tuplet_times as u32 / d.tuplet_enters as u32
    } else {
        dotted
    }
}

/// Convert a legacy [`Duration`] value to the MusicXML [`NoteTypeValue`] symbol.
pub fn duration_to_note_type(d: &Duration) -> NoteTypeValue {
    match d.value {
        1 => NoteTypeValue::Whole,
        2 => NoteTypeValue::Half,
        4 => NoteTypeValue::Quarter,
        8 => NoteTypeValue::Eighth,
        16 => NoteTypeValue::N16th,
        32 => NoteTypeValue::N32nd,
        64 => NoteTypeValue::N64th,
        128 => NoteTypeValue::N128th,
        _ => NoteTypeValue::Quarter, // fallback
    }
}

// ---------------------------------------------------------------------------
// Top-level conversion
// ---------------------------------------------------------------------------

/// Convert a legacy [`Song`] into a MusicXML [`ScorePartwise`] document.
///
/// The conversion is organized in passes:
/// 1. Score metadata (work title, identification)
/// 2. Part list (one `ScorePart` per track)
/// 3. Parts (measures → notes, per track)
pub fn song_to_score_partwise(song: &Song) -> musicxml::ScorePartwise {
    let identification = build_identification(song);
    let part_list = build_part_list(song);
    let parts = build_parts(song);

    musicxml::ScorePartwise {
        version: Some("4.0".to_string()),
        work: Some(musicxml::Work {
            work_number: None,
            work_title: Some(song.name.clone()).filter(|s| !s.is_empty()),
            opus: None,
        }),
        movement_number: None,
        movement_title: None,
        identification: Some(identification),
        defaults: None,
        credits: vec![],
        part_list,
        parts,
    }
}

// ---------------------------------------------------------------------------
// Stubs — filled in subsequent commits
// ---------------------------------------------------------------------------

fn build_identification(song: &Song) -> musicxml::identification::Identification {
    use musicxml::identification::{Creator, Encoding, Identification, Rights, Supports};

    let mut creators = vec![];
    if !song.artist.is_empty() {
        creators.push(Creator {
            creator_type: Some("composer".to_string()),
            value: song.artist.clone(),
        });
    }
    if !song.author.is_empty() {
        creators.push(Creator {
            creator_type: Some("arranger".to_string()),
            value: song.author.clone(),
        });
    }
    if !song.words.is_empty() && song.words != song.author {
        creators.push(Creator {
            creator_type: Some("lyricist".to_string()),
            value: song.words.clone(),
        });
    }
    if !song.writer.is_empty() {
        creators.push(Creator {
            creator_type: Some("transcriber".to_string()),
            value: song.writer.clone(),
        });
    }

    let rights = if !song.copyright.is_empty() {
        vec![Rights {
            rights_type: None,
            value: song.copyright.clone(),
        }]
    } else {
        vec![]
    };

    let encoding = Some(Encoding {
        encoding_date: None,
        encoders: vec![],
        software: vec!["guitarpro (Rust)".to_string()],
        encoding_description: vec![],
        supports: vec![
            Supports {
                supports_type: "yes".to_string(),
                element: "accidental".to_string(),
                attribute: None,
                value: None,
            },
            Supports {
                supports_type: "yes".to_string(),
                element: "beam".to_string(),
                attribute: None,
                value: None,
            },
        ],
    });

    // Collect remaining free-text metadata into miscellaneous fields
    let mut misc_fields = vec![];
    if !song.subtitle.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "subtitle".to_string(),
            value: song.subtitle.clone(),
        });
    }
    if !song.album.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "album".to_string(),
            value: song.album.clone(),
        });
    }
    if !song.date.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "date".to_string(),
            value: song.date.clone(),
        });
    }
    if !song.instructions.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "instructions".to_string(),
            value: song.instructions.clone(),
        });
    }
    for (i, notice) in song.notice.iter().enumerate() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: format!("notice-{}", i + 1),
            value: notice.clone(),
        });
    }

    let miscellaneous = if misc_fields.is_empty() {
        None
    } else {
        Some(musicxml::identification::Miscellaneous {
            fields: misc_fields,
        })
    };

    Identification {
        creators,
        rights,
        encoding,
        source: None,
        relations: vec![],
        miscellaneous,
    }
}

fn build_part_list(song: &Song) -> musicxml::part_list::PartList {
    use musicxml::part_list::{MidiInstrument, PartListItem, PartName, ScorePart};

    let items = song
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let part_id = format!("P{}", i + 1);
            let instrument_id = format!("P{}-I1", i + 1);

            let channel = song
                .channels
                .get(track.channel_index)
                .cloned()
                .unwrap_or_default();

            // Guitar Pro program is 0-based; MusicXML midi-program is 1-based
            let midi_program = if track.percussion_track {
                None
            } else {
                Some((channel.instrument as u8).saturating_add(1))
            };

            let midi_instrument = MidiInstrument {
                id: instrument_id.clone(),
                midi_channel: Some(channel.channel + 1), // MusicXML channels are 1-based
                midi_name: None,
                midi_bank: if channel.bank > 0 {
                    Some(channel.bank as u16)
                } else {
                    None
                },
                midi_program,
                midi_unpitched: if track.percussion_track {
                    Some(60)
                } else {
                    None
                },
                volume: Some(f64::from(channel.volume) / 127.0 * 100.0),
                pan: Some((f64::from(channel.balance) - 64.0) / 63.0 * 90.0),
                elevation: None,
            };

            PartListItem::ScorePart(ScorePart {
                id: part_id,
                identification: None,
                part_name: Some(PartName {
                    print_object: None,
                    justify: None,
                    value: Some(track.name.clone()),
                }),
                part_name_display: None,
                part_abbreviation: if !track.short_name.is_empty() {
                    Some(PartName {
                        print_object: None,
                        justify: None,
                        value: Some(track.short_name.clone()),
                    })
                } else {
                    None
                },
                part_abbreviation_display: None,
                groups: vec![],
                score_instruments: vec![musicxml::part_list::ScoreInstrument {
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

fn build_parts(song: &Song) -> Vec<musicxml::Part> {
    song.tracks
        .iter()
        .enumerate()
        .map(|(track_idx, track)| {
            let part_id = format!("P{}", track_idx + 1);
            let measures = track
                .measures
                .iter()
                .enumerate()
                .map(|(measure_idx, measure)| {
                    let header = song
                        .measure_headers
                        .get(measure.header_index)
                        .cloned()
                        .unwrap_or_default();
                    build_measure(song, track, track_idx, measure, &header, measure_idx)
                })
                .collect();
            musicxml::Part {
                id: part_id,
                measures,
            }
        })
        .collect()
}

fn make_clef(sign: &str, line: Option<u8>, number: Option<u8>) -> musicxml::attributes::Clef {
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

fn build_measure(
    song: &Song,
    track: &crate::model::legacy::track::Track,
    track_idx: usize,
    measure: &crate::model::legacy::measure::Measure,
    header: &crate::model::legacy::headers::MeasureHeader,
    measure_idx: usize,
) -> musicxml::measure::Measure {
    use musicxml::{
        attributes::{Attributes, Key, StaffDetails, StaffTuning, Time},
        barline::Barline,
        direction::{Direction, DirectionType, DirectionTypeWrapper, Sound},
        measure::MusicData,
    };

    let mut music_data: Vec<MusicData> = vec![];

    // --- Attributes (first measure only) ---
    if measure_idx == 0 {
        let notation_clef = match measure.clef {
            crate::model::legacy::enums::MeasureClef::Treble => make_clef("G", Some(2), None),
            crate::model::legacy::enums::MeasureClef::Bass => make_clef("F", Some(4), None),
            crate::model::legacy::enums::MeasureClef::Tenor => make_clef("C", Some(4), None),
            crate::model::legacy::enums::MeasureClef::Alto => make_clef("C", Some(3), None),
        };

        // GP strings: high→low (string 1 = highest). MusicXML line 1 = lowest.
        let string_count = track.strings.len();
        let staff_tunings: Vec<StaffTuning> = track
            .strings
            .iter()
            .rev()
            .enumerate()
            .map(|(i, (_str_num, midi_val))| {
                let (step, alter, octave) = midi_to_pitch(*midi_val);
                StaffTuning {
                    line: (i + 1) as u8,
                    tuning_step: step,
                    tuning_alter: alter,
                    tuning_octave: octave,
                }
            })
            .collect();

        music_data.push(MusicData::Attributes(Attributes {
            divisions: Some(DIVISIONS),
            keys: vec![Key {
                number: None,
                print_object: None,
                cancel: None,
                fifths: Some(header.key_signature.key),
                mode: Some(
                    if header.key_signature.is_minor {
                        "minor"
                    } else {
                        "major"
                    }
                    .to_string(),
                ),
                key_steps: vec![],
                key_alters: vec![],
                key_accidentals: vec![],
                key_octaves: vec![],
            }],
            times: vec![Time {
                number: None,
                symbol: None,
                separator: None,
                print_object: None,
                beats: Some(header.time_signature.numerator.to_string()),
                beat_type: Some(header.time_signature.denominator.value.to_string()),
                senza_misura: None,
                interchangeable: None,
            }],
            staves: Some(2),
            part_symbol: None,
            instruments: None,
            clefs: vec![notation_clef, make_clef("TAB", None, Some(2))],
            staff_details: vec![StaffDetails {
                number: Some(2),
                show_frets: None,
                print_object: None,
                print_spacing: None,
                staff_type: Some("tab".to_string()),
                staff_lines: Some(string_count as u8),
                line_details: vec![],
                staff_tunings,
                capo: None,
                staff_size: None,
            }],
            transposes: vec![],
            for_parts: vec![],
            directives: vec![],
            measure_styles: vec![],
        }));
    }

    // --- Tempo direction (first measure of first track only) ---
    if measure_idx == 0 && track_idx == 0 {
        let tempo_bpm = if header.tempo > 0 {
            header.tempo as f64
        } else {
            song.tempo as f64
        };
        music_data.push(MusicData::Direction(Direction {
            placement: Some("above".to_string()),
            directive: None,
            id: None,
            direction_types: vec![DirectionTypeWrapper {
                content: DirectionType::Metronome(musicxml::direction::Metronome {
                    parentheses: None,
                    default_x: None,
                    default_y: None,
                    justify: None,
                    id: None,
                    beat_unit: Some("quarter".to_string()),
                    beat_unit_dots: vec![],
                    beat_unit_tied: None,
                    per_minute: Some(musicxml::direction::PerMinute {
                        font_size: None,
                        value: format!("{}", tempo_bpm as u32),
                    }),
                    metronome_arrows: None,
                    metronome_notes: vec![],
                    metronome_relation: None,
                }),
            }],
            offset: None,
            footnote: None,
            level: None,
            voice: None,
            staff: None,
            sound: Some(Sound {
                tempo: Some(tempo_bpm),
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
            }),
            listening: None,
            wavy_line: None,
            dashes: None,
            bracket: None,
            pedal: None,
        }));
    }

    // --- Repeat open barline ---
    if header.repeat_open {
        music_data.insert(
            0,
            MusicData::Barline(Barline {
                location: Some("left".to_string()),
                segno: None,
                coda: None,
                divisions: None,
                id: None,
                bar_style: Some(musicxml::barline::BarStyleColor {
                    color: None,
                    value: musicxml::barline::BarStyle::HeavyLight,
                }),
                footnote: None,
                level: None,
                wavy_line: None,
                segno_mark: None,
                coda_mark: None,
                ending: None,
                repeat: Some(musicxml::barline::Repeat {
                    direction: "forward".to_string(),
                    times: None,
                    winged: None,
                }),
                fermatas: vec![],
            }),
        );
    }

    // --- Repeat close barline ---
    if header.repeat_close > 0 {
        music_data.push(MusicData::Barline(Barline {
            location: Some("right".to_string()),
            segno: None,
            coda: None,
            divisions: None,
            id: None,
            bar_style: Some(musicxml::barline::BarStyleColor {
                color: None,
                value: musicxml::barline::BarStyle::LightHeavy,
            }),
            footnote: None,
            level: None,
            wavy_line: None,
            segno_mark: None,
            coda_mark: None,
            ending: None,
            repeat: Some(musicxml::barline::Repeat {
                direction: "backward".to_string(),
                times: Some(header.repeat_close as u8),
                winged: None,
            }),
            fermatas: vec![],
        }));
    }

    // --- Voices → notes ---
    music_data.extend(build_voices(measure, track));

    musicxml::measure::Measure {
        number: header.number.to_string(),
        implicit: None,
        non_controlling: None,
        width: None,
        text: None,
        id: None,
        music_data,
    }
}

/// Convert all voices of a measure into a flat list of `MusicData` items.
///
/// GP supports up to 2 voices per measure. In MusicXML multiple voices are encoded
/// linearly: voice 1 notes come first, then a `<backup>` rewinds the time cursor
/// to the start of the measure, followed by voice 2 notes.
fn build_voices(
    measure: &crate::model::legacy::measure::Measure,
    track: &crate::model::legacy::track::Track,
) -> Vec<musicxml::measure::MusicData> {
    use crate::model::legacy::enums::BeatStatus;
    use musicxml::measure::{Backup, MusicData};

    let mut result: Vec<MusicData> = vec![];

    for (voice_idx, voice) in measure.voices.iter().enumerate() {
        // Skip empty voices
        if voice.beats.iter().all(|b| b.status == BeatStatus::Empty) {
            continue;
        }

        // Insert <backup> before voice 2+ to rewind the time cursor
        if voice_idx > 0 {
            // Sum durations of the first voice
            let backup_duration: u32 = measure.voices[0]
                .beats
                .iter()
                .map(|b| duration_to_divisions(&b.duration))
                .sum();
            if backup_duration > 0 {
                result.push(MusicData::Backup(Backup {
                    duration: backup_duration,
                }));
            }
        }

        for beat in &voice.beats {
            if beat.status == BeatStatus::Empty {
                continue;
            }

            let is_rest = beat.status == BeatStatus::Rest || beat.notes.is_empty();
            let voice_num = format!("{}", voice_idx + 1);
            let divisions = duration_to_divisions(&beat.duration);
            let note_type = duration_to_note_type(&beat.duration);

            if is_rest {
                result.push(MusicData::Note(make_rest_note(
                    divisions,
                    note_type,
                    &beat.duration,
                    &voice_num,
                )));
            } else {
                for (note_idx, note) in beat.notes.iter().enumerate() {
                    let is_chord = note_idx > 0;
                    result.push(MusicData::Note(make_note(
                        note,
                        &track.strings,
                        divisions,
                        note_type,
                        &beat.duration,
                        &voice_num,
                        is_chord,
                    )));
                }
            }
        }
    }

    result
}

fn make_rest_note(
    divisions: u32,
    note_type: NoteTypeValue,
    duration: &Duration,
    voice: &str,
) -> musicxml::note::Note {
    use musicxml::note::{NoteType, Rest};
    musicxml::note::Note {
        grace: None,
        cue: None,
        pitch: None,
        rest: Some(Rest {
            measure: None,
            display_step: None,
            display_octave: None,
        }),
        unpitched: None,
        chord: None,
        duration: Some(divisions),
        ties: vec![],
        footnote: None,
        level: None,
        instrument: None,
        voice: Some(voice.to_string()),
        note_type: Some(NoteType {
            size: None,
            value: note_type,
        }),
        dots: if duration.dotted {
            vec![musicxml::note::Dot {
                default_x: None,
                default_y: None,
                placement: None,
            }]
        } else {
            vec![]
        },
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

fn make_note(
    note: &crate::model::legacy::note::Note,
    strings: &[(i8, i8)],
    divisions: u32,
    note_type: NoteTypeValue,
    duration: &Duration,
    voice: &str,
    is_chord: bool,
) -> musicxml::note::Note {
    use crate::model::legacy::enums::NoteType as GpNoteType;
    use musicxml::note::{NoteType, Pitch, Rest, Tie};

    let is_rest = note.kind == GpNoteType::Rest;
    let is_tie_stop = note.kind == GpNoteType::Tie;

    // Compute pitch from fret + open string tuning
    let (pitch, rest) = if is_rest {
        (
            None,
            Some(Rest {
                measure: None,
                display_step: None,
                display_octave: None,
            }),
        )
    } else {
        let midi = if note.string > 0 && (note.string as usize) <= strings.len() {
            (note.value as i8).saturating_add(strings[(note.string as usize) - 1].1)
        } else {
            note.value as i8
        };
        let (step, alter, octave) = midi_to_pitch(midi);
        (
            Some(Pitch {
                step,
                alter,
                octave,
            }),
            None,
        )
    };

    let ties = if is_tie_stop {
        vec![Tie {
            tie_type: "stop".to_string(),
            time_only: None,
        }]
    } else {
        vec![]
    };

    musicxml::note::Note {
        grace: None,
        cue: None,
        pitch,
        rest,
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
            value: note_type,
        }),
        dots: if duration.dotted {
            vec![musicxml::note::Dot {
                default_x: None,
                default_y: None,
                placement: None,
            }]
        } else {
            vec![]
        },
        accidental: None,
        time_modification: build_time_modification(duration),
        stem: None,
        notehead: None,
        notehead_text: None,
        staff: None,
        beams: vec![],
        notations: build_notations(note, strings),
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

/// Build `<notations>` from a legacy `NoteEffect`, including tablature fret/string.
fn build_notations(
    note: &crate::model::legacy::note::Note,
    strings: &[(i8, i8)],
) -> Vec<musicxml::note::Notations> {
    use crate::model::legacy::enums::SlideType;
    use musicxml::note::{
        Articulations, Bend, Fret, HammerPull, Harmonic, Notations, Ornaments, PlacedEmpty, Slide,
        StringNumber, Technical, WavyLine,
    };

    let eff = &note.effect;
    let mut technical = Technical {
        up_bow: None,
        down_bow: None,
        harmonic: None,
        open_string: None,
        thumb_position: None,
        fingering: None,
        pluck: None,
        double_tongue: None,
        triple_tongue: None,
        stopped: None,
        snap_pizzicato: None,
        fret: None,
        string: None,
        hammer_on: None,
        pull_off: None,
        bend: None,
        tap: None,
        heel: None,
        toe: None,
        fingernails: None,
        hole: None,
        arrow: None,
        handbell: None,
        brass_bend: None,
        flip: None,
        smear: None,
        open: None,
        half_muted: None,
        harmon_mute: None,
        golpe: None,
        other_technical: None,
    };

    // Fret + string number for tablature
    if note.string > 0 && (note.string as usize) <= strings.len() {
        technical.fret = Some(Fret {
            font_size: None,
            color: None,
            value: note.value as u8,
        });
        technical.string = Some(StringNumber {
            default_x: None,
            default_y: None,
            placement: None,
            value: note.string as u8,
        });
    }

    // Hammer-on
    if eff.hammer {
        technical.hammer_on = Some(HammerPull {
            technique_type: "start".to_string(),
            number: None,
            placement: None,
            value: None,
        });
    }

    // Bend — GP bend value is in semitone quarters (100 units = 1 semitone).
    // Use the peak value (max of points).
    if let Some(bend) = &eff.bend {
        let peak = bend.points.iter().map(|p| p.value).max().unwrap_or(0);
        if peak > 0 {
            technical.bend = Some(Bend {
                shape: None,
                default_x: None,
                default_y: None,
                bend_alter: peak as f64 / 100.0,
                pre_bend: None,
                release: None,
                with_bar: None,
            });
        }
    }

    // Let ring — no direct MusicXML equivalent, encode as other-technical
    if eff.let_ring {
        technical.other_technical = Some(musicxml::note::OtherPlacement {
            placement: None,
            smufl: None,
            value: Some("let-ring".to_string()),
        });
    }

    // Palm mute → half-muted
    if eff.palm_mute {
        technical.half_muted = Some(musicxml::note::OtherPlacement {
            placement: None,
            smufl: None,
            value: None,
        });
    }

    // Natural harmonic
    if eff.harmonic.is_some() {
        technical.harmonic = Some(Harmonic {
            print_object: None,
            placement: None,
            natural: Some(()),
            artificial: None,
            base_pitch: None,
            touching_pitch: None,
            sounding_pitch: None,
        });
    }

    let has_technical = technical.fret.is_some()
        || technical.string.is_some()
        || technical.hammer_on.is_some()
        || technical.pull_off.is_some()
        || technical.bend.is_some()
        || technical.other_technical.is_some()
        || technical.harmonic.is_some()
        || technical.half_muted.is_some();
    let technical_opt = if has_technical { Some(technical) } else { None };

    // Slides → <slide> elements
    let slides: Vec<Slide> = eff
        .slides
        .iter()
        .filter_map(|s| match s {
            SlideType::ShiftSlideTo | SlideType::LegatoSlideTo => Some(Slide {
                slide_type: "start".to_string(),
                number: None,
                line_type: Some(
                    if *s == SlideType::LegatoSlideTo {
                        "solid"
                    } else {
                        "dashed"
                    }
                    .to_string(),
                ),
                value: None,
            }),
            _ => None,
        })
        .collect();

    // Vibrato → wavy-line ornament
    let ornaments = if eff.vibrato {
        Some(Ornaments {
            trill_mark: None,
            turn: None,
            delayed_turn: None,
            inverted_turn: None,
            delayed_inverted_turn: None,
            vertical_turn: None,
            inverted_vertical_turn: None,
            shake: None,
            wavy_line: Some(WavyLine {
                wavy_type: "start".to_string(),
                number: None,
                placement: None,
            }),
            mordent: None,
            inverted_mordent: None,
            schleifer: None,
            tremolo: None,
            haydn: None,
            other_ornament: None,
            accidental_marks: vec![],
        })
    } else {
        None
    };

    // Articulations
    let placed = |active| {
        if active {
            Some(PlacedEmpty {
                placement: None,
                default_x: None,
                default_y: None,
            })
        } else {
            None
        }
    };
    let staccato = placed(eff.staccato);
    let accent = placed(eff.accentuated_note);
    let strong_accent = if eff.heavy_accentuated_note {
        Some(musicxml::note::StrongAccent {
            placement: None,
            accent_type: None,
        })
    } else {
        None
    };
    let articulations = if staccato.is_some() || accent.is_some() || strong_accent.is_some() {
        vec![Articulations {
            accent,
            strong_accent,
            staccato,
            tenuto: None,
            detached_legato: None,
            staccatissimo: None,
            spiccato: None,
            scoop: None,
            plop: None,
            doit: None,
            falloff: None,
            breath_mark: None,
            caesura: None,
            stress: None,
            unstress: None,
            soft_accent: None,
            other_articulation: None,
        }]
    } else {
        vec![]
    };

    if technical_opt.is_none()
        && slides.is_empty()
        && ornaments.is_none()
        && articulations.is_empty()
    {
        return vec![];
    }

    vec![Notations {
        print_object: None,
        footnote: None,
        level: None,
        tied: vec![],
        slurs: vec![],
        tuplets: vec![],
        glissandos: vec![],
        slides,
        ornaments,
        technical: technical_opt,
        articulations,
        dynamics: vec![],
        fermatas: vec![],
        arpeggiate: None,
        non_arpeggiate: None,
        accidental_marks: vec![],
        other_notations: vec![],
    }]
}

fn build_time_modification(duration: &Duration) -> Option<musicxml::note::TimeModification> {
    if duration.is_default_tuplet() {
        return None;
    }
    Some(musicxml::note::TimeModification {
        actual_notes: duration.tuplet_enters,
        normal_notes: duration.tuplet_times,
        normal_type: None,
        normal_dots: vec![],
    })
}
