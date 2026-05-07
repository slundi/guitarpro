//! Conversion from `legacy::Song` to `musicxml::ScorePartwise`.
//!
//! Entry point: [`song_to_score_partwise`].

use crate::model::{
    legacy::{
        key_signature::{Duration, DURATION_QUARTER_TIME},
        song::Song,
    },
    musicxml::{
        self,
        note::NoteTypeValue,
    },
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
        0  => ("C", None),
        1  => ("C", Some(1.0)),
        2  => ("D", None),
        3  => ("D", Some(1.0)),
        4  => ("E", None),
        5  => ("F", None),
        6  => ("F", Some(1.0)),
        7  => ("G", None),
        8  => ("G", Some(1.0)),
        9  => ("A", None),
        10 => ("A", Some(1.0)),
        11 => ("B", None),
        _  => unreachable!(),
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
    let dotted = if d.dotted {
        base + base / 2
    } else {
        base
    };

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
        1   => NoteTypeValue::Whole,
        2   => NoteTypeValue::Half,
        4   => NoteTypeValue::Quarter,
        8   => NoteTypeValue::Eighth,
        16  => NoteTypeValue::N16th,
        32  => NoteTypeValue::N32nd,
        64  => NoteTypeValue::N64th,
        128 => NoteTypeValue::N128th,
        _   => NoteTypeValue::Quarter, // fallback
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
        Some(musicxml::identification::Miscellaneous { fields: misc_fields })
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
                midi_bank: if channel.bank > 0 { Some(channel.bank as u16) } else { None },
                midi_program,
                midi_unpitched: if track.percussion_track { Some(60) } else { None },
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
            musicxml::Part { id: part_id, measures }
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
            crate::model::legacy::enums::MeasureClef::Bass   => make_clef("F", Some(4), None),
            crate::model::legacy::enums::MeasureClef::Tenor  => make_clef("C", Some(4), None),
            crate::model::legacy::enums::MeasureClef::Alto   => make_clef("C", Some(3), None),
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
                StaffTuning { line: (i + 1) as u8, tuning_step: step, tuning_alter: alter, tuning_octave: octave }
            })
            .collect();

        music_data.push(MusicData::Attributes(Attributes {
            divisions: Some(DIVISIONS),
            keys: vec![Key {
                number: None,
                print_object: None,
                cancel: None,
                fifths: Some(header.key_signature.key),
                mode: Some(if header.key_signature.is_minor { "minor" } else { "major" }.to_string()),
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
        let tempo_bpm = if header.tempo > 0 { header.tempo as f64 } else { song.tempo as f64 };
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
        music_data.insert(0, MusicData::Barline(Barline {
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
        }));
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

    // --- Voices → notes (filled in next step) ---
    music_data.extend(build_voices(measure));

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

fn build_voices(_measure: &crate::model::legacy::measure::Measure) -> Vec<musicxml::measure::MusicData> {
    vec![] // filled in next step
}
