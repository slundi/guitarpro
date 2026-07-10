//! Conversion from `legacy::Song` to `optimized::LoadedScore`.

use std::collections::HashMap;

use crate::model::{
    legacy::{
        beat::{Beat as LBeat, Voice as LVoice},
        effects::DEFAULT_VELOCITY,
        enums::{
            BeatStrokeDirection, DirectionSign, Fingering as LFingering, HarmonicType, NoteType,
            Octave, SlapEffect, SlideType, TupletBracket, VoiceDirection,
        },
        headers::MeasureHeader,
        key_signature::{
            DURATION_QUARTER_TIME, Duration as LDuration, KeySignature as LKeySignature,
        },
        measure::Measure as LMeasure,
        mix_table::MixTableItem as LMixTableItem,
        note::Note as LNote,
        song::Song,
    },
    optimized::{
        LoadedScore,
        beat::{Beat, Duration, Dynamic, Tuplet, Voice},
        beat::{GpChord, GpMixTableChange, GpMixTableItem},
        global::{Instrument, InstrumentId, InstrumentKind, MeasureIndex, Score, StaffId, TrackId},
        metadata::{Identification, KeySignature, Metadata, Mode, TimeSignature},
        note::{
            Articulation, Finger, GpBendEffect, GpBendPoint, GpGraceEffect, GpHarmonicEffect, Note,
            NoteValue, Notehead, Pitch, PitchStep, Technique, TechniqueKind, TechniqueParams,
            TieType,
        },
        timeline::{
            Barline, BarlineStyle, JumpKind, Marker, MarkerKind, MeasureDef, NavigationEvent,
        },
        track::{Clef, MeasureData, StaffDef, StaffDisplay, Track},
    },
};

use super::timeline::key_sig_from_fifths;

/// Convert a legacy [`Song`] into an optimized [`LoadedScore`].
pub fn legacy_song_to_loaded_score(song: &Song) -> LoadedScore {
    let timeline = build_timeline(song);
    let metadata = build_metadata(song, &timeline);
    let instruments = build_instruments(song);
    let (staves, staves_for_track) = build_staves(song);
    let tracks = build_tracks(song, &staves_for_track, &timeline);

    LoadedScore {
        score: Score {
            metadata,
            instruments,
            staves,
            tracks,
            groups: Vec::new(),
            timeline,
            lyric_lines: Vec::new(),
            lyric_projections: Vec::new(),
            defaults: None,
        },
        layout: None,
    }
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

fn build_metadata(song: &Song, timeline: &[MeasureDef]) -> Metadata {
    let initial_tempo = timeline
        .first()
        .and_then(|m| m.tempo)
        .unwrap_or(song.tempo as f32);
    let initial_time = timeline
        .first()
        .and_then(|m| m.time_signature)
        .unwrap_or(TimeSignature {
            numerator: 4,
            denominator: 4,
        });
    let initial_key = timeline
        .first()
        .and_then(|m| m.key_signature)
        .unwrap_or(KeySignature {
            root: Pitch {
                step: PitchStep::C,
                alter: 0,
                octave: 4,
            },
            mode: Mode::Major,
        });

    // Store GP5-specific fields that don't map to standard optimized fields.
    // These are recovered in the reverse conversion via identification.miscellaneous.
    let mut misc: Vec<(String, String)> = Vec::new();
    if !song.subtitle.is_empty() {
        misc.push(("gp.subtitle".into(), song.subtitle.clone()));
    }
    if !song.words.is_empty() {
        misc.push(("gp.words".into(), song.words.clone()));
    }
    if !song.writer.is_empty() {
        misc.push(("gp.writer".into(), song.writer.clone()));
    }
    if !song.instructions.is_empty() {
        misc.push(("gp.instructions".into(), song.instructions.clone()));
    }
    if !song.date.is_empty() {
        misc.push(("gp.date".into(), song.date.clone()));
    }
    if !song.transcriber.is_empty() {
        misc.push(("gp.transcriber".into(), song.transcriber.clone()));
    }
    if !song.comments.is_empty() {
        misc.push(("gp.comments".into(), song.comments.clone()));
    }
    for (i, n) in song.notice.iter().enumerate() {
        misc.push((format!("gp.notice.{}", i), n.clone()));
    }
    misc.push((
        "gp.triplet_feel".into(),
        crate::model::legacy::enums::from_triplet_feel(&song.triplet_feel).to_string(),
    ));
    misc.push(("gp.tempo_name".into(), song.tempo_name.clone()));
    misc.push((
        "gp.version".into(),
        format!(
            "{}.{}.{}",
            song.version.number.0, song.version.number.1, song.version.number.2
        ),
    ));
    misc.push(("gp.hide_tempo".into(), song.hide_tempo.to_string()));
    // RSE master effect (written before page setup for v > 5.0.0, reverb written after MIDI channels).
    misc.push((
        "gp.master_reverb".into(),
        (song.master_effect.reverb as i32).to_string(),
    ));
    misc.push((
        "gp.master_vol".into(),
        (song.master_effect.volume as i32).to_string(),
    ));
    let me_eq: String = song
        .master_effect
        .equalizer
        .knobs
        .iter()
        .map(|k| ((-k * 10.0).round() as i8).to_string())
        .collect::<Vec<_>>()
        .join(",");
    misc.push(("gp.master_eq".into(), me_eq));

    // Page setup — preserve all strings and settings verbatim.
    misc.push(("gp.page.title".into(), song.page_setup.title.clone()));
    misc.push(("gp.page.subtitle".into(), song.page_setup.subtitle.clone()));
    misc.push(("gp.page.artist".into(), song.page_setup.artist.clone()));
    misc.push(("gp.page.album".into(), song.page_setup.album.clone()));
    misc.push(("gp.page.words".into(), song.page_setup.words.clone()));
    misc.push(("gp.page.music".into(), song.page_setup.music.clone()));
    misc.push((
        "gp.page.word_and_music".into(),
        song.page_setup.word_and_music.clone(),
    ));
    misc.push((
        "gp.page.copyright".into(),
        song.page_setup.copyright.clone(),
    ));
    misc.push((
        "gp.page.page_number".into(),
        song.page_setup.page_number.clone(),
    ));
    misc.push((
        "gp.page.size_x".into(),
        song.page_setup.page_size.x.to_string(),
    ));
    misc.push((
        "gp.page.size_y".into(),
        song.page_setup.page_size.y.to_string(),
    ));
    misc.push((
        "gp.page.margin_left".into(),
        song.page_setup.page_margin.left.to_string(),
    ));
    misc.push((
        "gp.page.margin_right".into(),
        song.page_setup.page_margin.right.to_string(),
    ));
    misc.push((
        "gp.page.margin_top".into(),
        song.page_setup.page_margin.top.to_string(),
    ));
    misc.push((
        "gp.page.margin_bottom".into(),
        song.page_setup.page_margin.bottom.to_string(),
    ));
    misc.push((
        "gp.page.scale".into(),
        ((song.page_setup.score_size_proportion * 100.0).ceil() as i32).to_string(),
    ));
    misc.push((
        "gp.page.header_footer".into(),
        song.page_setup.header_and_footer.to_string(),
    ));

    // All 64 MIDI channels — preserve verbatim for byte-identical roundtrip.
    // Format per channel: instr,vol,balance,chorus,reverb,phaser,tremolo,bank,effect_ch
    let channels_str: String = song
        .channels
        .iter()
        .map(|c| {
            format!(
                "{},{},{},{},{},{},{},{},{}",
                c.instrument,
                c.volume,
                c.balance,
                c.chorus,
                c.reverb,
                c.phaser,
                c.tremolo,
                c.bank,
                c.effect_channel
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    misc.push(("gp.channels".into(), channels_str));

    // Lyrics: GP4/5 have track_choice + 5 lines (starting_measure + text)
    misc.push((
        "gp.lyrics.track_choice".into(),
        song.lyrics.track_choice.to_string(),
    ));
    for line in &song.lyrics.lines {
        misc.push((format!("gp.lyrics.{}.start", line.0), line.1.to_string()));
        if !line.2.is_empty() {
            misc.push((format!("gp.lyrics.{}.text", line.0), line.2.clone()));
        }
    }

    // Direction signs on measure headers (Coda, Fine, Segno, Da Capo, etc.)
    for (i, header) in song.measure_headers.iter().enumerate() {
        if let Some(d) = &header.direction {
            let name = match d {
                DirectionSign::Coda => "Coda",
                DirectionSign::DoubleCoda => "DoubleCoda",
                DirectionSign::Segno => "Segno",
                DirectionSign::SegnoSegno => "SegnoSegno",
                DirectionSign::Fine => "Fine",
                DirectionSign::DaCapo => "DaCapo",
                DirectionSign::DaCapoAlCoda => "DaCapoAlCoda",
                DirectionSign::DaCapoAlDoubleCoda => "DaCapoAlDoubleCoda",
                DirectionSign::DaCapoAlFine => "DaCapoAlFine",
                DirectionSign::DaSegno => "DaSegno",
                DirectionSign::DaSegnoAlCoda => "DaSegnoAlCoda",
                DirectionSign::DaSegnoAlDoubleCoda => "DaSegnoAlDoubleCoda",
                DirectionSign::DaSegnoAlFine => "DaSegnoAlFine",
                DirectionSign::DaSegnoSegno => "DaSegnoSegno",
                DirectionSign::DaSegnoSegnoAlCoda => "DaSegnoSegnoAlCoda",
                DirectionSign::DaSegnoSegnoAlDoubleCoda => "DaSegnoSegnoAlDoubleCoda",
                DirectionSign::DaSegnoSegnoAlFine => "DaSegnoSegnoAlFine",
                DirectionSign::DaCoda => "DaCoda",
                DirectionSign::DaDoubleCoda => "DaDoubleCoda",
            };
            misc.push((format!("gp.direction.{i}"), name.to_string()));
        }
    }

    // Per-track GP5/GPX/GP7 fields not in the optimized model.
    for (i, track) in song.tracks.iter().enumerate() {
        let rse = &track.rse;
        misc.push((format!("gp.track.{i}.short_name"), track.short_name.clone()));
        misc.push((
            format!("gp.track.{i}.transpose_chromatic"),
            track.transpose_chromatic.to_string(),
        ));
        misc.push((
            format!("gp.track.{i}.transpose_octave"),
            track.transpose_octave.to_string(),
        ));
        misc.push((
            format!("gp.track.{i}.channel_index"),
            track.channel_index.to_string(),
        ));
        if let Some(prog) = track.midi_program_gpif {
            misc.push((format!("gp.track.{i}.midi_program_gpif"), prog.to_string()));
        }
        misc.push((format!("gp.track.{i}.port"), track.port.to_string()));
        misc.push((format!("gp.track.{i}.color"), track.color.to_string()));
        misc.push((format!("gp.track.{i}.offset"), track.offset.to_string()));
        misc.push((
            format!("gp.track.{i}.fret_count"),
            track.fret_count.to_string(),
        ));
        misc.push((format!("gp.track.{i}.visible"), track.visible.to_string()));
        misc.push((format!("gp.track.{i}.solo"), track.solo.to_string()));
        misc.push((format!("gp.track.{i}.mute"), track.mute.to_string()));
        misc.push((format!("gp.track.{i}.use_rse"), track.use_rse.to_string()));
        misc.push((
            format!("gp.track.{i}.indicate_tuning"),
            track.indicate_tuning.to_string(),
        ));
        misc.push((
            format!("gp.track.{i}.twelve_stringed"),
            track.twelve_stringed_guitar_track.to_string(),
        ));
        misc.push((format!("gp.track.{i}.banjo"), track.banjo_track.to_string()));
        // TrackSettings as flags2 bitmask
        let flags2 = track_settings_flags(&track.settings);
        misc.push((format!("gp.track.{i}.settings_flags"), flags2.to_string()));
        misc.push((
            format!("gp.track.{i}.rse_humanize"),
            rse.humanize.to_string(),
        ));
        misc.push((
            format!("gp.track.{i}.rse_auto_acc"),
            crate::model::legacy::enums::from_accentuation(&rse.auto_accentuation).to_string(),
        ));
        let bank = song
            .channels
            .get(track.channel_index)
            .map_or(0u8, |c| c.bank);
        misc.push((format!("gp.track.{i}.bank"), bank.to_string()));
        let eq_str: String = rse
            .equalizer
            .knobs
            .iter()
            .map(|k| format!("{}", ((-k * 10.0).round() as i8)))
            .collect::<Vec<_>>()
            .join(",");
        misc.push((format!("gp.track.{i}.rse_eq"), eq_str));
        misc.push((
            format!("gp.track.{i}.rse_instr"),
            format!(
                "{},{},{},{}",
                rse.instrument.instrument,
                rse.instrument.unknown,
                rse.instrument.sound_bank,
                rse.instrument.effect_number
            ),
        ));
        misc.push((
            format!("gp.track.{i}.rse_effect"),
            rse.instrument.effect.clone(),
        ));
        misc.push((
            format!("gp.track.{i}.rse_effect_cat"),
            rse.instrument.effect_category.clone(),
        ));
    }

    let identification = Some(Identification {
        creators: Vec::new(),
        rights: None,
        encoding_software: None,
        encoding_date: None,
        source: None,
        miscellaneous: misc,
    });

    Metadata {
        work: None,
        movement_number: None,
        title: song.name.clone(),
        artist: non_empty(song.artist.clone()),
        album: non_empty(song.album.clone()),
        composer: non_empty(song.author.clone()),
        year: None,
        copyright: non_empty(song.copyright.clone()),
        identification,
        credits: Vec::new(),
        master_tempo: initial_tempo,
        time_signature: initial_time,
        key_signature: initial_key,
        chords: Vec::new(),
        scale_hint: None,
    }
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

fn build_timeline(song: &Song) -> Vec<MeasureDef> {
    let mut measures = Vec::new();
    let mut prev_tempo: f32 = song.tempo as f32;
    let mut prev_time = TimeSignature {
        numerator: 4,
        denominator: 4,
    };
    let mut prev_key = legacy_key_to_opt_key(&song.key);
    let mut first = true;

    for (i, header) in song.measure_headers.iter().enumerate() {
        // GP7/GPX: mh.tempo is non-zero when a per-bar tempo automation exists.
        // GP3/4/5: mh.tempo is always 0; use song-level tempo as fallback.
        let tempo = if header.tempo > 0 {
            header.tempo as f32
        } else {
            song.tempo as f32
        };
        let time = TimeSignature {
            numerator: header.time_signature.numerator as u8,
            denominator: header.time_signature.denominator.value as u8,
        };
        let key = legacy_key_to_opt_key(&header.key_signature);

        let measure_tempo = if first || tempo != prev_tempo {
            Some(tempo)
        } else {
            None
        };
        let measure_time = if first || time != prev_time {
            Some(time)
        } else {
            None
        };
        let measure_key = if first || key != prev_key {
            Some(key)
        } else {
            None
        };

        prev_tempo = tempo;
        prev_time = time;
        prev_key = key;
        first = false;

        let mut navigation = Vec::new();
        if header.repeat_open {
            navigation.push(NavigationEvent {
                measure_index: MeasureIndex(i as u16),
                kind: JumpKind::RepeatOpen,
                repeat_count: None,
                volta: None,
                volta_last: false,
            });
        }
        if header.repeat_close >= 0 {
            navigation.push(NavigationEvent {
                measure_index: MeasureIndex(i as u16),
                kind: JumpKind::RepeatClose,
                repeat_count: Some(header.repeat_close as u8 + 1),
                volta: if header.repeat_alternative > 0 {
                    Some(header.repeat_alternative)
                } else {
                    None
                },
                volta_last: false,
            });
        } else if header.repeat_alternative > 0 {
            // Volta bracket without a repeat close in this measure
            navigation.push(NavigationEvent {
                measure_index: MeasureIndex(i as u16),
                kind: JumpKind::RepeatClose,
                repeat_count: None,
                volta: Some(header.repeat_alternative),
                volta_last: false,
            });
        }

        let numerator = header.time_signature.numerator as u32;
        let denom_val = header.time_signature.denominator.value as u32;
        let duration_ticks = (DURATION_QUARTER_TIME as u32 * 4)
            .checked_div(denom_val)
            .map(|q| numerator * q)
            .unwrap_or(numerator * DURATION_QUARTER_TIME as u32);

        let barline_right = if header.double_bar {
            Some(Barline {
                style: BarlineStyle::LightLight,
                ending: None,
            })
        } else {
            None
        };

        let marker = header.marker.as_ref().map(|m| Marker {
            label: m.title.clone(),
            kind: MarkerKind::Custom,
            gp_color: Some(m.color as u32),
        });

        // Preserve GP beam grouping when the time signature is recorded.
        let gp_beams = if measure_time.is_some() {
            let b = &header.time_signature.beams;
            Some([
                b.first().copied().unwrap_or(0),
                b.get(1).copied().unwrap_or(0),
                b.get(2).copied().unwrap_or(0),
                b.get(3).copied().unwrap_or(0),
            ])
        } else {
            None
        };

        measures.push(MeasureDef {
            index: MeasureIndex(i as u16),
            tempo: measure_tempo,
            time_signature: measure_time,
            key_signature: measure_key,
            marker,
            navigation,
            tick_resolution: DURATION_QUARTER_TIME as u16,
            duration_ticks,
            barline_left: None,
            barline_right,
            gp_beams,
            gp_fermatas: header.fermatas.clone(),
            gp_free_time: header.free_time,
        });
    }

    measures
}

// ---------------------------------------------------------------------------
// Instruments & staves
// ---------------------------------------------------------------------------

fn build_instruments(song: &Song) -> Vec<Instrument> {
    song.tracks
        .iter()
        .map(|track| {
            let channel = song
                .channels
                .get(track.channel_index)
                .cloned()
                .unwrap_or_default();
            // Always store raw MIDI tuning values for a lossless roundtrip.
            // midi_to_pitch is lossy for MIDI values < 12 (octave clamp).
            let gp_strings: Vec<i8> = track.strings.iter().map(|(_, midi)| *midi).collect();
            let kind = if track.percussion_track {
                InstrumentKind::Percussion
            } else if !track.strings.is_empty() {
                let tuning: Vec<Pitch> = track
                    .strings
                    .iter()
                    .map(|(_, midi)| midi_to_pitch(*midi))
                    .collect();
                let string_count = tuning.len() as u8;
                InstrumentKind::Stringed {
                    tuning,
                    string_count,
                    capo: 0,
                }
            } else {
                InstrumentKind::Pitched
            };

            Instrument {
                name: track.name.clone(),
                abbreviation: None,
                instrument_sound: None,
                midi_channel: channel.channel,
                midi_program: channel.instrument.clamp(0, 127) as u8,
                midi_bank: None,
                volume: Some(channel.volume as f32 / 127.0),
                pan: Some((channel.balance as f32 - 64.0) / 64.0),
                kind,
                transpose: None,
                gp_strings,
            }
        })
        .collect()
}

fn build_staves(song: &Song) -> (Vec<StaffDef>, Vec<Vec<StaffId>>) {
    let mut staves: Vec<StaffDef> = Vec::new();
    let mut staves_for_track: Vec<Vec<StaffId>> = Vec::new();

    for track in &song.tracks {
        let mut track_staves: Vec<StaffId> = Vec::new();
        if track.percussion_track {
            let id = StaffId(staves.len() as u8);
            staves.push(StaffDef {
                clef: Clef::Percussion,
                display: StaffDisplay::Notation,
            });
            track_staves.push(id);
        } else {
            let id = StaffId(staves.len() as u8);
            staves.push(StaffDef {
                clef: Clef::Treble,
                display: StaffDisplay::Notation,
            });
            track_staves.push(id);
            let id2 = StaffId(staves.len() as u8);
            staves.push(StaffDef {
                clef: Clef::Tab,
                display: StaffDisplay::Tab,
            });
            track_staves.push(id2);
        }
        staves_for_track.push(track_staves);
    }

    (staves, staves_for_track)
}

// ---------------------------------------------------------------------------
// Tracks / measures / voices / beats / notes
// ---------------------------------------------------------------------------

fn build_tracks(
    song: &Song,
    staves_for_track: &[Vec<StaffId>],
    _timeline: &[MeasureDef],
) -> Vec<Track> {
    song.tracks
        .iter()
        .enumerate()
        .map(|(track_idx, legacy_track)| {
            let track_id = TrackId(track_idx as u8);
            let instrument_id = InstrumentId(track_idx as u8);
            let staves = staves_for_track.get(track_idx).cloned().unwrap_or_default();

            let mut measures = std::collections::BTreeMap::new();
            for (mh_idx, header) in song.measure_headers.iter().enumerate() {
                if let Some(legacy_measure) = legacy_track.measures.get(mh_idx) {
                    let measure_index = MeasureIndex(mh_idx as u16);
                    let md = build_measure_data(
                        legacy_measure,
                        measure_index,
                        track_id,
                        header,
                        &legacy_track.strings,
                    );
                    measures.insert(measure_index, md);
                }
            }

            Track {
                id: track_id,
                name: legacy_track.name.clone(),
                instrument: instrument_id,
                staves,
                measures,
            }
        })
        .collect()
}

fn build_measure_data(
    legacy_measure: &LMeasure,
    measure_index: MeasureIndex,
    track_id: TrackId,
    _header: &MeasureHeader,
    strings: &[(i8, i8)],
) -> MeasureData {
    let mut voices: HashMap<u8, Voice> = HashMap::new();
    for (vi, legacy_voice) in legacy_measure.voices.iter().enumerate() {
        // Beat.start values in legacy GP are always relative to DURATION_QUARTER_TIME (960),
        // not to the absolute song position. Each measure resets start to 960.
        let voice = build_voice(legacy_voice, vi as u8, DURATION_QUARTER_TIME, strings);
        voices.insert(vi as u8, voice);
    }
    use crate::model::legacy::enums::from_line_break;
    MeasureData {
        measure_index,
        track_id,
        repeat: None,
        voices,
        gp_line_break: from_line_break(&legacy_measure.line_break),
        gp_simile_mark: legacy_measure.simile_mark.clone(),
    }
}

fn build_voice(
    legacy_voice: &LVoice,
    voice_id: u8,
    measure_start: i64,
    strings: &[(i8, i8)],
) -> Voice {
    let beats = legacy_voice
        .beats
        .iter()
        .map(|b| build_beat(b, measure_start, strings))
        .collect();
    Voice { voice_id, beats }
}

fn build_beat(legacy_beat: &LBeat, measure_start: i64, strings: &[(i8, i8)]) -> Beat {
    use crate::model::legacy::enums::BeatStatus;
    let tick_offset = legacy_beat
        .start
        .map(|s| (s - measure_start).max(0) as u32)
        .unwrap_or(0);

    let duration = convert_duration(&legacy_beat.duration);
    let dynamic = legacy_beat
        .notes
        .first()
        .and_then(|n| velocity_to_dynamic(n.velocity));
    let notes = legacy_beat
        .notes
        .iter()
        .map(|n| build_note(n, strings))
        .collect();
    let gp_empty = legacy_beat.status == BeatStatus::Empty;
    let gp_rest = legacy_beat.status == BeatStatus::Rest;
    let gp_vibrato = legacy_beat.effect.vibrato;
    let gp_fade_in = legacy_beat.effect.fade_in;
    let gp_stroke = {
        let s = &legacy_beat.effect.stroke;
        if s.direction != BeatStrokeDirection::None && s.value != 0 {
            Some((s.value, s.direction == BeatStrokeDirection::Up))
        } else {
            None
        }
    };
    let gp_pick_stroke = match legacy_beat.effect.pick_stroke {
        BeatStrokeDirection::Up => Some(true),
        BeatStrokeDirection::Down => Some(false),
        BeatStrokeDirection::None => None,
    };
    let gp_beat_flags2 = {
        let mut f = 0i16;
        if legacy_beat.display.break_beam {
            f |= 0x0001;
        }
        if legacy_beat.display.beam_direction == VoiceDirection::Down {
            f |= 0x0002;
        }
        if legacy_beat.display.force_beam {
            f |= 0x0004;
        }
        if legacy_beat.display.beam_direction == VoiceDirection::Up {
            f |= 0x0008;
        }
        if legacy_beat.octave == Octave::Ottava {
            f |= 0x0010;
        }
        if legacy_beat.octave == Octave::OttavaBassa {
            f |= 0x0020;
        }
        if legacy_beat.octave == Octave::Quindicesima {
            f |= 0x0040;
        }
        if legacy_beat.octave == Octave::QuindicesimaBassa {
            f |= 0x0100;
        }
        if legacy_beat.display.tuplet_bracket == TupletBracket::Start {
            f |= 0x0200;
        }
        if legacy_beat.display.tuplet_bracket == TupletBracket::End {
            f |= 0x0400;
        }
        if legacy_beat.display.break_secondary != 0 {
            f |= 0x0800;
        }
        if legacy_beat.display.break_secondary_tuplet {
            f |= 0x1000;
        }
        if legacy_beat.display.force_bracket {
            f |= 0x2000;
        }
        if f != 0 { Some(f) } else { None }
    };
    let gp_break_secondary = if legacy_beat.display.break_secondary != 0 {
        Some(legacy_beat.display.break_secondary)
    } else {
        None
    };
    let gp_slap_effect = match legacy_beat.effect.slap_effect {
        SlapEffect::None => None,
        SlapEffect::Tapping => Some(1u8),
        SlapEffect::Slapping => Some(2u8),
        SlapEffect::Popping => Some(3u8),
    };
    let gp_rasgueado = legacy_beat.effect.has_rasgueado;
    let gp_text = legacy_beat.text.clone();
    let gp_mix_table = legacy_beat.effect.mix_table_change.as_ref().map(|mtc| {
        let item = |opt: &Option<LMixTableItem>| {
            opt.as_ref().map(|i| GpMixTableItem {
                value: i.value,
                duration: i.duration,
                all_tracks: i.all_tracks,
            })
        };
        GpMixTableChange {
            instrument: item(&mtc.instrument),
            rse_instrument: mtc.rse.instrument,
            rse_unknown: mtc.rse.unknown,
            rse_sound_bank: mtc.rse.sound_bank,
            rse_effect_number: mtc.rse.effect_number,
            rse_effect_category: mtc.rse.effect_category.clone(),
            rse_effect: mtc.rse.effect.clone(),
            volume: item(&mtc.volume),
            balance: item(&mtc.balance),
            chorus: item(&mtc.chorus),
            reverb: item(&mtc.reverb),
            phaser: item(&mtc.phaser),
            tremolo: item(&mtc.tremolo),
            tempo_name: mtc.tempo_name.clone(),
            tempo: item(&mtc.tempo),
            hide_tempo: mtc.hide_tempo,
            wah: mtc.wah.as_ref().map(|w| (w.value, w.display)),
            use_rse: mtc.use_rse,
        }
    });

    Beat {
        tick_offset,
        duration,
        notes,
        events: Vec::new(),
        dynamic,
        slur: None,
        lyric: None,
        gp_empty,
        gp_rest,
        gp_vibrato,
        gp_fade_in,
        gp_stroke,
        gp_pick_stroke,
        gp_beat_flags2,
        gp_break_secondary,
        gp_slap_effect,
        gp_rasgueado,
        gp_text,
        gp_mix_table,
        gp_tremolo_bar: legacy_beat.effect.tremolo_bar.as_ref().map(|tb| {
            use crate::model::legacy::enums::from_bend_type;
            crate::model::optimized::note::GpBendEffect {
                kind: from_bend_type(&tb.kind),
                value: tb.value,
                points: tb
                    .points
                    .iter()
                    .map(|p| crate::model::optimized::note::GpBendPoint {
                        position: p.position,
                        value: p.value,
                        vibrato: p.vibrato,
                    })
                    .collect(),
            }
        }),
        gp_chord: legacy_beat.effect.chord.as_ref().map(|c| {
            use crate::model::legacy::enums::{
                from_chord_alteration, from_chord_extension, from_chord_type, from_fingering,
            };
            GpChord {
                new_format: c.new_format == Some(true),
                length: c.length,
                sharp: c.sharp == Some(true),
                root: c.root.as_ref().map(|r| r.value).unwrap_or(0),
                kind: c.kind.as_ref().map(from_chord_type).unwrap_or(0),
                extension: c.extension.as_ref().map(from_chord_extension).unwrap_or(0),
                bass: c.bass.as_ref().map(|b| b.value as i32).unwrap_or(0),
                tonality: c
                    .tonality
                    .as_ref()
                    .map(|t| from_chord_alteration(t) as i32)
                    .unwrap_or(0),
                add: c.add == Some(true),
                name: c.name.clone(),
                fifth: c.fifth.as_ref().map(from_chord_alteration).unwrap_or(0),
                ninth: c.ninth.as_ref().map(from_chord_alteration).unwrap_or(0),
                eleventh: c.eleventh.as_ref().map(from_chord_alteration).unwrap_or(0),
                first_fret: c.first_fret.map(|f| f as i32).unwrap_or(0),
                strings: c.strings.iter().map(|&s| s as i32).collect(),
                barres: c
                    .barres
                    .iter()
                    .map(|b| (b.fret as u8, b.start as u8, b.end as u8))
                    .collect(),
                omissions: c.omissions.clone(),
                fingerings: c.fingerings.iter().map(from_fingering).collect(),
                show: c.show == Some(true),
            }
        }),
        beam_group: None,
        tuplet: None,
        beams: Vec::new(),
        grace_notes: Vec::new(),
        cue: false,
        chord: None,
    }
}

fn build_note(legacy_note: &LNote, strings: &[(i8, i8)]) -> Note {
    let is_rest = legacy_note.kind == NoteType::Rest;
    let is_tie = legacy_note.kind == NoteType::Tie;
    let is_dead = legacy_note.kind == NoteType::Dead;

    let pitch = if is_rest || is_tie {
        None
    } else if legacy_note.string > 0 && (legacy_note.string as usize) <= strings.len() {
        let string_idx = (legacy_note.string - 1) as usize;
        let midi = strings[string_idx]
            .1
            .saturating_add(legacy_note.value as i8);
        Some(midi_to_pitch(midi))
    } else {
        None
    };

    // Rest notes carry both a string number and a value in the binary format.
    let (string, fret) = (
        Some(legacy_note.string as u8),
        Some(legacy_note.value as u8),
    );

    let tie = if is_tie { Some(TieType::End) } else { None };
    let notehead = if is_dead { Some(Notehead::X) } else { None };

    let mut articulations = Vec::new();
    if legacy_note.effect.staccato {
        articulations.push(Articulation::Staccato);
    }
    if legacy_note.effect.accentuated_note {
        articulations.push(Articulation::Accent);
    }
    if legacy_note.effect.heavy_accentuated_note {
        articulations.push(Articulation::Marcato);
    }

    let mut techniques = Vec::new();
    let mut gp_harmonic: Option<GpHarmonicEffect> = None;
    let mut gp_grace: Option<GpGraceEffect> = None;
    let mut gp_bend: Option<GpBendEffect> = None;
    if legacy_note.effect.hammer {
        techniques.push(Technique {
            kind: TechniqueKind::HammerOn,
            params: TechniqueParams::None,
        });
    }
    if legacy_note.effect.let_ring {
        techniques.push(Technique {
            kind: TechniqueKind::LetRing,
            params: TechniqueParams::None,
        });
    }
    if legacy_note.effect.vibrato {
        techniques.push(Technique {
            kind: TechniqueKind::Vibrato,
            params: TechniqueParams::None,
        });
    }
    if legacy_note.effect.palm_mute {
        techniques.push(Technique {
            kind: TechniqueKind::HalfMuted,
            params: TechniqueParams::None,
        });
    }
    for slide in &legacy_note.effect.slides {
        let kind = match slide {
            SlideType::ShiftSlideTo => TechniqueKind::SlideUp,
            SlideType::LegatoSlideTo => TechniqueKind::SlideLegato,
            SlideType::OutDownwards => TechniqueKind::SlideDown,
            SlideType::OutUpWards => TechniqueKind::SlideOutUp,
            SlideType::IntoFromAbove => TechniqueKind::SlideIntoAbove,
            SlideType::IntoFromBelow => TechniqueKind::SlideIntoBelow,
            SlideType::None => continue,
        };
        techniques.push(Technique {
            kind,
            params: TechniqueParams::None,
        });
    }
    if let Some(bend) = &legacy_note.effect.bend {
        use crate::model::legacy::enums::from_bend_type;
        let semitones = bend.value as f32 / 25.0;
        techniques.push(Technique {
            kind: TechniqueKind::Bend,
            params: TechniqueParams::Bend {
                value: semitones,
                vibrato: false,
            },
        });
        gp_bend = Some(GpBendEffect {
            kind: from_bend_type(&bend.kind),
            value: bend.value,
            points: bend
                .points
                .iter()
                .map(|p| GpBendPoint {
                    position: p.position,
                    value: p.value,
                    vibrato: p.vibrato,
                })
                .collect(),
        });
    }
    if let Some(harmonic) = &legacy_note.effect.harmonic {
        let natural = harmonic.kind == HarmonicType::Natural;
        techniques.push(Technique {
            kind: TechniqueKind::Harmonic,
            params: TechniqueParams::Harmonic { natural },
        });
        gp_harmonic = Some(GpHarmonicEffect {
            kind: harmonic.kind.clone() as u8,
            pitch_just: harmonic.pitch.as_ref().map(|p| p.just),
            pitch_accidental: harmonic.pitch.as_ref().map(|p| p.accidental),
            octave: harmonic.octave.as_ref().map(|o| match o {
                Octave::None => 0u8,
                Octave::Ottava => 1,
                Octave::Quindicesima => 2,
                Octave::OttavaBassa => 3,
                Octave::QuindicesimaBassa => 4,
            }),
            fret: harmonic.fret,
        });
    }
    if let Some(grace) = &legacy_note.effect.grace {
        use crate::model::legacy::enums::from_grace_effect_transition;
        gp_grace = Some(GpGraceEffect {
            fret: grace.fret,
            velocity: grace.velocity,
            duration: grace.duration,
            transition: from_grace_effect_transition(&grace.transition),
            is_dead: grace.is_dead,
            is_on_beat: grace.is_on_beat,
        });
    }
    let gp_trill = legacy_note.effect.trill.as_ref().map(|t| {
        use crate::model::legacy::key_signature::{DURATION_SIXTY_FOURTH, DURATION_THIRTY_SECOND};
        let period = match t.duration.value as u8 {
            v if v == DURATION_THIRTY_SECOND => 2i8,
            v if v == DURATION_SIXTY_FOURTH => 3i8,
            _ => 1i8, // sixteenth
        };
        crate::model::optimized::note::GpTrillEffect {
            fret: t.fret,
            period,
        }
    });
    let gp_ghost = legacy_note.effect.ghost_note;
    let gp_duration_percent = legacy_note.duration_percent;
    let gp_swap_accidentals = legacy_note.swap_accidentals;
    if let Some(tp) = &legacy_note.effect.tremolo_picking {
        use crate::model::legacy::key_signature::{DURATION_EIGHTH, DURATION_THIRTY_SECOND};
        let speed = match tp.duration.value as u8 {
            v if v == DURATION_EIGHTH => NoteValue::Eighth,
            v if v == DURATION_THIRTY_SECOND => NoteValue::ThirtySecond,
            _ => NoteValue::Sixteenth,
        };
        techniques.push(Technique {
            kind: TechniqueKind::TremoloPicking,
            params: TechniqueParams::Tremolo { speed },
        });
    }

    Note {
        pitch,
        string,
        fret,
        tie,
        techniques,
        ornaments: Vec::new(),
        articulations,
        left_finger: convert_finger(&legacy_note.effect.left_hand_finger),
        right_finger: convert_finger(&legacy_note.effect.right_hand_finger),
        notehead,
        stem: None,
        accidental: None,
        arpeggiate: None,
        display_pitch: None,
        gp_harmonic,
        gp_grace,
        gp_bend,
        gp_trill,
        gp_ghost,
        gp_duration_percent,
        gp_swap_accidentals,
        gp_velocity: Some(legacy_note.velocity),
        gp_note_type_raw: if let crate::model::legacy::enums::NoteType::Unknown(v) =
            legacy_note.kind
        {
            Some(v)
        } else {
            None
        },
        gp_is_rest: is_rest,
        gp_note_duration: legacy_note.duration,
        gp_note_tuplet: legacy_note.tuplet,
        gp_ornament: legacy_note.effect.ornament.clone(),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn convert_duration(dur: &LDuration) -> Duration {
    let base = match dur.value {
        1 => NoteValue::Whole,
        2 => NoteValue::Half,
        4 => NoteValue::Quarter,
        8 => NoteValue::Eighth,
        16 => NoteValue::Sixteenth,
        32 => NoteValue::ThirtySecond,
        64 => NoteValue::SixtyFourth,
        128 => NoteValue::HundredTwentyEighth,
        v => NoteValue::Other(v),
    };
    let dots = if dur.dotted { 1 } else { 0 };
    let tuplet = if dur.tuplet_enters != 1 || dur.tuplet_times != 1 {
        Some(Tuplet {
            actual: dur.tuplet_enters,
            normal: dur.tuplet_times,
        })
    } else {
        None
    };
    Duration { base, dots, tuplet }
}

fn convert_finger(f: &LFingering) -> Option<Finger> {
    match f {
        LFingering::Open => None,
        LFingering::Thumb => Some(Finger::Thumb),
        LFingering::Index => Some(Finger::Index),
        LFingering::Middle => Some(Finger::Middle),
        LFingering::Annular => Some(Finger::Ring),
        LFingering::Little => Some(Finger::Pinky),
        LFingering::Unknown(_) => None,
    }
}

pub(crate) fn midi_to_pitch(midi: i8) -> Pitch {
    let m = (midi as i32).max(0);
    let octave = ((m / 12) - 1).max(0) as u8;
    match m % 12 {
        0 => Pitch {
            step: PitchStep::C,
            alter: 0,
            octave,
        },
        1 => Pitch {
            step: PitchStep::C,
            alter: 1,
            octave,
        },
        2 => Pitch {
            step: PitchStep::D,
            alter: 0,
            octave,
        },
        3 => Pitch {
            step: PitchStep::D,
            alter: 1,
            octave,
        },
        4 => Pitch {
            step: PitchStep::E,
            alter: 0,
            octave,
        },
        5 => Pitch {
            step: PitchStep::F,
            alter: 0,
            octave,
        },
        6 => Pitch {
            step: PitchStep::F,
            alter: 1,
            octave,
        },
        7 => Pitch {
            step: PitchStep::G,
            alter: 0,
            octave,
        },
        8 => Pitch {
            step: PitchStep::G,
            alter: 1,
            octave,
        },
        9 => Pitch {
            step: PitchStep::A,
            alter: 0,
            octave,
        },
        10 => Pitch {
            step: PitchStep::A,
            alter: 1,
            octave,
        },
        11 => Pitch {
            step: PitchStep::B,
            alter: 0,
            octave,
        },
        _ => unreachable!(),
    }
}

fn velocity_to_dynamic(v: i16) -> Option<Dynamic> {
    // Legacy velocities are packed as: unpacked = MIN_VELOCITY + VELOCITY_INCREMENT * (packed - 1)
    // MIN_VELOCITY = 15, VELOCITY_INCREMENT = 16
    // packed 1..8 → unpacked 15, 31, 47, 63, 79, 95, 111, 127
    if v == DEFAULT_VELOCITY {
        return None; // default — no explicit dynamic marker
    }
    Some(match v {
        ..=22 => Dynamic::PPP,
        23..=38 => Dynamic::PP,
        39..=54 => Dynamic::P,
        55..=70 => Dynamic::MP,
        71..=86 => Dynamic::MF,
        87..=102 => Dynamic::F,
        103..=118 => Dynamic::FF,
        _ => Dynamic::FFF,
    })
}

pub(crate) fn legacy_key_to_opt_key(key: &LKeySignature) -> KeySignature {
    let mode_str = if key.is_minor { Some("minor") } else { None };
    key_sig_from_fifths(key.key, mode_str)
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn track_settings_flags(s: &crate::model::legacy::track::TrackSettings) -> i16 {
    let mut f: i16 = 0;
    if s.tablature {
        f |= 0x0001;
    }
    if s.notation {
        f |= 0x0002;
    }
    if s.diagram_are_below {
        f |= 0x0004;
    }
    if s.show_rhythm {
        f |= 0x0008;
    }
    if s.force_horizontal {
        f |= 0x0010;
    }
    if s.force_channels {
        f |= 0x0020;
    }
    if s.diagram_list {
        f |= 0x0040;
    }
    if s.diagram_in_score {
        f |= 0x0080;
    }
    if s.auto_let_ring {
        f |= 0x0200;
    }
    if s.auto_brush {
        f |= 0x0400;
    }
    if s.extend_rhythmic {
        f |= 0x0800;
    }
    f
}
