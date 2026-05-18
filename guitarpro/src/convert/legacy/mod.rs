//! Conversion from `optimized::LoadedScore` to `legacy::Song`.

use crate::{
    audio::midi::MidiChannel,
    model::{
        legacy::{
            beat::{Beat as LBeat, BeatStroke, Voice as LVoice},
            chord::PitchClass,
            effects::DEFAULT_VELOCITY,
            effects::{GraceEffect, HarmonicEffect},
            enums::{
                BeatStatus, BeatStrokeDirection, DirectionSign, Fingering as LFingering,
                GraceEffectTransition, HarmonicType, LineBreak, MeasureClef, NoteType, Octave,
                SlapEffect, SlideType, TupletBracket, VoiceDirection,
            },
            headers::{Marker as LMarker, MeasureHeader},
            key_signature::{
                DURATION_QUARTER_TIME, Duration as LDuration, KeySignature as LKeySignature,
                TimeSignature as LTimeSig,
            },
            measure::Measure as LMeasure,
            mix_table::{MixTableChange, MixTableItem, WahEffect},
            note::{Note as LNote, NoteEffect},
            rse::RseInstrument,
            song::Song,
            track::Track as LTrack,
        },
        optimized::{
            LoadedScore,
            beat::{Beat, Duration, Dynamic, Voice},
            global::{Instrument, InstrumentKind, MeasureIndex},
            metadata::{KeySignature, Mode},
            note::{
                Articulation, Finger, Note, NoteValue, Notehead, Pitch, PitchStep, TechniqueKind,
                TechniqueParams, TieType,
            },
            timeline::{JumpKind, MeasureDef},
            track::Track,
        },
    },
};

/// Convert an optimized [`LoadedScore`] back into a legacy [`Song`].
///
/// Note: this conversion is **best-effort**. Many GP5-specific fields
/// (RSE effects, exact MIDI velocities, beat status, page layout, etc.)
/// cannot be fully reconstructed from the optimized model and will be
/// set to their defaults.
pub fn loaded_score_to_legacy_song(score: &LoadedScore) -> Song {
    let s = &score.score;
    let mut song = Song {
        name: s.metadata.title.clone(),
        artist: s.metadata.artist.clone().unwrap_or_default(),
        album: s.metadata.album.clone().unwrap_or_default(),
        author: s.metadata.composer.clone().unwrap_or_default(),
        copyright: s.metadata.copyright.clone().unwrap_or_default(),
        tempo: s.metadata.master_tempo as i16,
        ..Song::default()
    };

    // Recover GP5-specific fields stored in identification.miscellaneous.
    if let Some(ident) = &s.metadata.identification {
        let misc = &ident.miscellaneous;
        let get = |k: &str| {
            misc.iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.clone())
        };
        song.subtitle = get("gp.subtitle").unwrap_or_default();
        song.words = get("gp.words").unwrap_or_default();
        song.writer = get("gp.writer").unwrap_or_default();
        song.instructions = get("gp.instructions").unwrap_or_default();
        song.date = get("gp.date").unwrap_or_default();
        song.transcriber = get("gp.transcriber").unwrap_or_default();
        song.comments = get("gp.comments").unwrap_or_default();
        song.tempo_name = get("gp.tempo_name").unwrap_or_else(|| "Moderate".into());
        if let Some(v) = get("gp.triplet_feel")
            && let Ok(n) = v.parse::<i8>()
            && let Ok(tf) = crate::model::legacy::enums::get_triplet_feel(n)
        {
            song.triplet_feel = tf;
        }
        if let Some(v) = get("gp.hide_tempo")
            && let Ok(b) = v.parse::<bool>()
        {
            song.hide_tempo = b;
        }
        if let Some(ver) = get("gp.version") {
            let parts: Vec<&str> = ver.split('.').collect();
            if parts.len() == 3
                && let (Ok(a), Ok(b), Ok(c)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                )
            {
                song.version.number = (a, b, c);
            }
        }
        // Recover notice lines (sorted by index)
        let mut notice_entries: Vec<(usize, String)> = misc
            .iter()
            .filter(|(k, _)| k.starts_with("gp.notice."))
            .filter_map(|(k, v)| {
                k.strip_prefix("gp.notice.")
                    .and_then(|n| n.parse::<usize>().ok())
                    .map(|i| (i, v.clone()))
            })
            .collect();
        notice_entries.sort_by_key(|(i, _)| *i);
        song.notice = notice_entries.into_iter().map(|(_, v)| v).collect();

        // Recover page setup.
        if let Some(v) = get("gp.page.title") {
            song.page_setup.title = v;
        }
        if let Some(v) = get("gp.page.subtitle") {
            song.page_setup.subtitle = v;
        }
        if let Some(v) = get("gp.page.artist") {
            song.page_setup.artist = v;
        }
        if let Some(v) = get("gp.page.album") {
            song.page_setup.album = v;
        }
        if let Some(v) = get("gp.page.words") {
            song.page_setup.words = v;
        }
        if let Some(v) = get("gp.page.music") {
            song.page_setup.music = v;
        }
        if let Some(v) = get("gp.page.word_and_music") {
            song.page_setup.word_and_music = v;
        }
        if let Some(v) = get("gp.page.copyright") {
            song.page_setup.copyright = v;
        }
        if let Some(v) = get("gp.page.page_number") {
            song.page_setup.page_number = v;
        }
        if let Some(v) = get("gp.page.size_x")
            && let Ok(n) = v.parse::<u16>()
        {
            song.page_setup.page_size.x = n;
        }
        if let Some(v) = get("gp.page.size_y")
            && let Ok(n) = v.parse::<u16>()
        {
            song.page_setup.page_size.y = n;
        }
        if let Some(v) = get("gp.page.margin_left")
            && let Ok(n) = v.parse::<u16>()
        {
            song.page_setup.page_margin.left = n;
        }
        if let Some(v) = get("gp.page.margin_right")
            && let Ok(n) = v.parse::<u16>()
        {
            song.page_setup.page_margin.right = n;
        }
        if let Some(v) = get("gp.page.margin_top")
            && let Ok(n) = v.parse::<u16>()
        {
            song.page_setup.page_margin.top = n;
        }
        if let Some(v) = get("gp.page.margin_bottom")
            && let Ok(n) = v.parse::<u16>()
        {
            song.page_setup.page_margin.bottom = n;
        }
        if let Some(v) = get("gp.page.scale")
            && let Ok(n) = v.parse::<i32>()
        {
            song.page_setup.score_size_proportion = n as f32 / 100.0;
        }
        if let Some(v) = get("gp.page.header_footer")
            && let Ok(n) = v.parse::<u16>()
        {
            song.page_setup.header_and_footer = n;
        }

        // Recover RSE master effect.
        if let Some(v) = get("gp.master_reverb")
            && let Ok(n) = v.parse::<i32>()
        {
            song.master_effect.reverb = n as f32;
        }
        if let Some(v) = get("gp.master_vol")
            && let Ok(n) = v.parse::<i32>()
        {
            song.master_effect.volume = n as f32;
        }
        if let Some(eq_str) = get("gp.master_eq") {
            song.master_effect.equalizer.knobs = eq_str
                .split(',')
                .filter_map(|s| s.parse::<i8>().ok())
                .map(|packed| -(packed as f32) / 10.0)
                .collect();
        }
    }

    // --- Key signature ---
    song.key = opt_key_to_legacy_key(&s.metadata.key_signature);

    // --- Measure headers from timeline ---
    song.measure_headers = build_measure_headers(s.timeline.as_slice());

    // --- Channels & tracks ---
    let misc: &[(String, String)] = s
        .metadata
        .identification
        .as_ref()
        .map(|id| id.miscellaneous.as_slice())
        .unwrap_or(&[]);
    let (channels, channel_for_instrument) = build_channels(&s.instruments, misc);
    song.channels = channels;

    song.tracks = build_tracks(score, &channel_for_instrument, misc);

    // --- Lyrics: GP4/5 require exactly 5 lyric lines ---
    song.lyrics.lines = (0u8..5).map(|i| (i, 0u16, String::new())).collect();
    let misc_all: &[(String, String)] = s
        .metadata
        .identification
        .as_ref()
        .map(|id| id.miscellaneous.as_slice())
        .unwrap_or(&[]);
    let get_misc = |k: &str| {
        misc_all
            .iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
    };
    if let Some(v) = get_misc("gp.lyrics.track_choice")
        && let Ok(n) = v.parse::<u8>()
    {
        song.lyrics.track_choice = n;
    }
    for line in &mut song.lyrics.lines {
        if let Some(v) = get_misc(&format!("gp.lyrics.{}.start", line.0))
            && let Ok(n) = v.parse::<u16>()
        {
            line.1 = n;
        }
        if let Some(v) = get_misc(&format!("gp.lyrics.{}.text", line.0)) {
            line.2 = v;
        }
    }

    // --- Direction signs (Coda, Fine, Segno, Da Capo, etc.) ---
    for (key, value) in misc_all.iter() {
        if let Some(idx_str) = key.strip_prefix("gp.direction.")
            && let Ok(idx) = idx_str.parse::<usize>()
            && let Some(header) = song.measure_headers.get_mut(idx)
        {
            header.direction = match value.as_str() {
                "Coda" => Some(DirectionSign::Coda),
                "DoubleCoda" => Some(DirectionSign::DoubleCoda),
                "Segno" => Some(DirectionSign::Segno),
                "SegnoSegno" => Some(DirectionSign::SegnoSegno),
                "Fine" => Some(DirectionSign::Fine),
                "DaCapo" => Some(DirectionSign::DaCapo),
                "DaCapoAlCoda" => Some(DirectionSign::DaCapoAlCoda),
                "DaCapoAlDoubleCoda" => Some(DirectionSign::DaCapoAlDoubleCoda),
                "DaCapoAlFine" => Some(DirectionSign::DaCapoAlFine),
                "DaSegno" => Some(DirectionSign::DaSegno),
                "DaSegnoAlCoda" => Some(DirectionSign::DaSegnoAlCoda),
                "DaSegnoAlDoubleCoda" => Some(DirectionSign::DaSegnoAlDoubleCoda),
                "DaSegnoAlFine" => Some(DirectionSign::DaSegnoAlFine),
                "DaSegnoSegno" => Some(DirectionSign::DaSegnoSegno),
                "DaSegnoSegnoAlCoda" => Some(DirectionSign::DaSegnoSegnoAlCoda),
                "DaSegnoSegnoAlDoubleCoda" => Some(DirectionSign::DaSegnoSegnoAlDoubleCoda),
                "DaSegnoSegnoAlFine" => Some(DirectionSign::DaSegnoSegnoAlFine),
                "DaCoda" => Some(DirectionSign::DaCoda),
                "DaDoubleCoda" => Some(DirectionSign::DaDoubleCoda),
                _ => None,
            };
        }
    }

    song
}

// ---------------------------------------------------------------------------
// Measure headers
// ---------------------------------------------------------------------------

fn build_measure_headers(timeline: &[MeasureDef]) -> Vec<MeasureHeader> {
    let mut headers: Vec<MeasureHeader> = Vec::new();
    let mut running_tempo: f32 = 120.0;
    let mut running_num: i8 = 4;
    let mut running_den_val: u16 = 4;
    let mut running_beams: Vec<u8> = default_beams(4);
    let mut running_key = LKeySignature::default();

    // First pass: build headers with all fields
    let starts = compute_measure_starts(timeline);

    for (i, md) in timeline.iter().enumerate() {
        if let Some(t) = md.tempo {
            running_tempo = t;
        }
        if let Some(ts) = md.time_signature {
            running_num = ts.numerator as i8;
            running_den_val = ts.denominator as u16;
            running_beams = if let Some(b) = md.gp_beams {
                b.to_vec()
            } else {
                default_beams(ts.numerator)
            };
        }
        if let Some(ks) = md.key_signature {
            running_key = opt_key_to_legacy_key(&ks);
        }

        let mut header = MeasureHeader {
            number: (i + 1) as u16,
            start: starts.get(i).copied().unwrap_or(DURATION_QUARTER_TIME),
            tempo: running_tempo as i32,
            time_signature: LTimeSig {
                numerator: running_num,
                denominator: LDuration {
                    value: running_den_val,
                    ..LDuration::default()
                },
                beams: running_beams.clone(),
            },
            key_signature: running_key.clone(),
            double_bar: md.barline_right.as_ref().is_some_and(|b| {
                matches!(
                    b.style,
                    crate::model::optimized::timeline::BarlineStyle::LightLight
                )
            }),
            marker: md.marker.as_ref().map(|m| LMarker {
                title: m.label.clone(),
                color: m.gp_color.unwrap_or(0xff0000) as i32,
            }),
            ..MeasureHeader::default()
        };

        // Navigation events → repeat fields
        for nav in &md.navigation {
            match nav.kind {
                JumpKind::RepeatOpen => {
                    header.repeat_open = true;
                }
                JumpKind::RepeatClose => {
                    if let Some(count) = nav.repeat_count {
                        header.repeat_close = (count.saturating_sub(1)) as i8;
                    }
                    if let Some(v) = nav.volta {
                        header.repeat_alternative = v;
                    }
                }
                _ => {}
            }
        }

        headers.push(header);
    }

    headers
}

fn compute_measure_starts(timeline: &[MeasureDef]) -> Vec<i64> {
    let mut starts = Vec::with_capacity(timeline.len());
    let mut pos: i64 = DURATION_QUARTER_TIME; // first measure at tick 960
    let mut running_num: i64 = 4;
    let mut running_den: i64 = 4;

    for md in timeline {
        starts.push(pos);
        if let Some(ts) = md.time_signature {
            running_num = ts.numerator as i64;
            running_den = ts.denominator as i64;
        }
        let length = if running_den > 0 {
            running_num * DURATION_QUARTER_TIME * 4 / running_den
        } else {
            running_num * DURATION_QUARTER_TIME
        };
        pos += length;
    }

    starts
}

fn default_beams(numerator: u8) -> Vec<u8> {
    match numerator {
        2 => vec![2, 2],
        3 => vec![3],
        4 => vec![2, 2, 2, 2],
        6 => vec![3, 3],
        _ => vec![numerator],
    }
}

// ---------------------------------------------------------------------------
// MIDI channels
// ---------------------------------------------------------------------------

fn build_channels(
    instruments: &[Instrument],
    misc: &[(String, String)],
) -> (Vec<MidiChannel>, Vec<usize>) {
    // GP3/4/5 always write exactly 64 MIDI channels.
    let channel_for_instrument: Vec<usize> = instruments
        .iter()
        .map(|ins| ins.midi_channel as usize)
        .collect();

    // Restore all 64 channels verbatim from the packed misc entry.
    if let Some((_, packed)) = misc.iter().find(|(k, _)| k == "gp.channels") {
        let mut channels: Vec<MidiChannel> = Vec::with_capacity(64);
        for (i, part) in packed.split(';').enumerate() {
            let f: Vec<&str> = part.split(',').collect();
            if f.len() >= 8 {
                let effect_channel = if f.len() >= 9 {
                    f[8].parse().unwrap_or((i as u8).saturating_add(1))
                } else {
                    (i as u8).saturating_add(1)
                };
                channels.push(MidiChannel {
                    channel: i as u8,
                    effect_channel,
                    instrument: f[0].parse().unwrap_or(25),
                    volume: f[1].parse().unwrap_or(95),
                    balance: f[2].parse().unwrap_or(64),
                    chorus: f[3].parse().unwrap_or(0),
                    reverb: f[4].parse().unwrap_or(0),
                    phaser: f[5].parse().unwrap_or(0),
                    tremolo: f[6].parse().unwrap_or(0),
                    bank: f[7].parse().unwrap_or(0),
                });
            }
        }
        while channels.len() < 64 {
            let i = channels.len() as u8;
            channels.push(MidiChannel {
                channel: i,
                effect_channel: i.saturating_add(1),
                instrument: 25,
                volume: 95,
                balance: 64,
                chorus: 0,
                reverb: 0,
                phaser: 0,
                tremolo: 0,
                bank: 0,
            });
        }
        return (channels, channel_for_instrument);
    }

    // Fallback: build from instrument data only.
    let mut channels: Vec<MidiChannel> = Vec::with_capacity(64);
    for i in 0u8..64u8 {
        let mut ch = MidiChannel {
            channel: i,
            effect_channel: i.saturating_add(1),
            instrument: 25,
            volume: 95,
            balance: 64,
            chorus: 0,
            reverb: 0,
            phaser: 0,
            tremolo: 0,
            bank: 0,
        };
        if let Some(instr) = instruments.iter().find(|ins| ins.midi_channel == i) {
            ch.instrument = instr.midi_program as i32;
            ch.volume = instr.volume.map_or(95, |v| (v * 127.0) as i8);
            ch.balance = instr.pan.map_or(64, |p| ((p * 64.0) + 64.0) as i8);
        }
        channels.push(ch);
    }
    (channels, channel_for_instrument)
}

// ---------------------------------------------------------------------------
// Tracks
// ---------------------------------------------------------------------------

fn build_tracks(
    score: &LoadedScore,
    channel_for_instrument: &[usize],
    misc: &[(String, String)],
) -> Vec<LTrack> {
    let s = &score.score;
    let starts = compute_measure_starts(&s.timeline);

    s.tracks
        .iter()
        .enumerate()
        .map(|(track_idx, opt_track)| {
            let instrument = s.instruments.get(opt_track.instrument.0 as usize);
            let channel_index = channel_for_instrument
                .get(opt_track.instrument.0 as usize)
                .copied()
                .unwrap_or(0);

            let percussion =
                instrument.is_some_and(|ins| matches!(ins.kind, InstrumentKind::Percussion));
            let strings = instrument
                .map(|ins| {
                    if !ins.gp_strings.is_empty() {
                        // Prefer raw MIDI values (lossless) over Pitch conversion.
                        ins.gp_strings
                            .iter()
                            .enumerate()
                            .map(|(i, &midi)| ((i + 1) as i8, midi))
                            .collect()
                    } else {
                        instrument_strings(ins)
                    }
                })
                .unwrap_or_default();

            let measures =
                build_legacy_measures(opt_track, &s.timeline, &starts, &strings, percussion);

            let get_track = |suffix: &str| -> Option<String> {
                let key = format!("gp.track.{track_idx}.{suffix}");
                misc.iter().find(|(k, _)| k == &key).map(|(_, v)| v.clone())
            };

            let port = get_track("port")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1u8);
            let color = get_track("color")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0xff0000i32);
            let offset = get_track("offset")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0i32);
            let fret_count = get_track("fret_count")
                .and_then(|v| v.parse().ok())
                .unwrap_or(24u8);
            let visible = get_track("visible")
                .and_then(|v| v.parse().ok())
                .unwrap_or(true);
            let solo = get_track("solo")
                .and_then(|v| v.parse().ok())
                .unwrap_or(false);
            let mute = get_track("mute")
                .and_then(|v| v.parse().ok())
                .unwrap_or(false);
            let use_rse = get_track("use_rse")
                .and_then(|v| v.parse().ok())
                .unwrap_or(false);
            let indicate_tuning = get_track("indicate_tuning")
                .and_then(|v| v.parse().ok())
                .unwrap_or(false);
            let twelve_stringed = get_track("twelve_stringed")
                .and_then(|v| v.parse().ok())
                .unwrap_or(false);
            let banjo = get_track("banjo")
                .and_then(|v| v.parse().ok())
                .unwrap_or(false);

            let settings = if let Some(flags) =
                get_track("settings_flags").and_then(|v| v.parse::<i16>().ok())
            {
                flags_to_track_settings(flags)
            } else {
                crate::model::legacy::track::TrackSettings::default()
            };

            let rse = build_track_rse(&get_track);

            LTrack {
                number: (opt_track.id.0 + 1) as i32,
                name: opt_track.name.clone(),
                channel_index,
                percussion_track: percussion,
                strings: strings.clone(),
                port,
                color,
                offset,
                fret_count,
                visible,
                solo,
                mute,
                use_rse,
                indicate_tuning,
                twelve_stringed_guitar_track: twelve_stringed,
                banjo_track: banjo,
                settings,
                rse,
                measures,
                ..LTrack::default()
            }
        })
        .collect()
}

fn instrument_strings(instrument: &Instrument) -> Vec<(i8, i8)> {
    match &instrument.kind {
        InstrumentKind::Stringed { tuning, .. } => tuning
            .iter()
            .enumerate()
            .map(|(i, pitch)| ((i + 1) as i8, pitch_to_midi(pitch)))
            .collect(),
        _ => Vec::new(),
    }
}

fn build_legacy_measures(
    opt_track: &Track,
    timeline: &[MeasureDef],
    starts: &[i64],
    strings: &[(i8, i8)],
    _percussion: bool,
) -> Vec<LMeasure> {
    timeline
        .iter()
        .enumerate()
        .map(|(i, md)| {
            let measure_index = MeasureIndex(i as u16);
            let measure_start = starts.get(i).copied().unwrap_or(DURATION_QUARTER_TIME);

            let mut measure = LMeasure {
                number: i + 1,
                start: measure_start,
                header_index: i,
                track_index: opt_track.id.0 as usize,
                key_signature: md
                    .key_signature
                    .map(|ks| opt_key_to_legacy_key(&ks))
                    .unwrap_or_default(),
                time_signature: md
                    .time_signature
                    .map(|ts| LTimeSig {
                        numerator: ts.numerator as i8,
                        denominator: LDuration {
                            value: ts.denominator as u16,
                            ..LDuration::default()
                        },
                        beams: default_beams(ts.numerator),
                    })
                    .unwrap_or_default(),
                clef: MeasureClef::Treble,
                voices: Vec::new(),
                line_break: LineBreak::None,
                simile_mark: None,
                has_double_bar: md.barline_right.as_ref().is_some_and(|b| {
                    matches!(
                        b.style,
                        crate::model::optimized::timeline::BarlineStyle::LightLight
                    )
                }),
            };

            if let Some(md_data) = opt_track.measures.get(&measure_index) {
                // Restore GP5 line-break
                measure.line_break =
                    crate::model::legacy::enums::get_line_break(md_data.gp_line_break);
                // GP5 uses 2 voices; GP3 uses 1.
                for vi in 0..2u8 {
                    let voice = if let Some(opt_voice) = md_data.voices.get(&vi) {
                        build_legacy_voice(opt_voice, vi, measure_start, strings)
                    } else {
                        LVoice::default()
                    };
                    measure.voices.push(voice);
                }
            } else {
                measure.voices.push(LVoice::default());
                measure.voices.push(LVoice::default());
            }

            measure
        })
        .collect()
}

fn build_legacy_voice(
    opt_voice: &Voice,
    voice_id: u8,
    measure_start: i64,
    strings: &[(i8, i8)],
) -> LVoice {
    let beats = opt_voice
        .beats
        .iter()
        .map(|b| build_legacy_beat(b, voice_id, measure_start, strings))
        .collect();
    LVoice {
        beats,
        ..LVoice::default()
    }
}

fn build_legacy_beat(
    beat: &Beat,
    _voice_id: u8,
    measure_start: i64,
    strings: &[(i8, i8)],
) -> LBeat {
    let start = Some(measure_start + beat.tick_offset as i64);
    let duration = duration_to_legacy(&beat.duration);
    let velocity = dynamic_to_velocity(beat.dynamic);
    let notes = beat
        .notes
        .iter()
        .map(|n| build_legacy_note(n, velocity, strings))
        .collect();

    let status = if beat.gp_empty {
        BeatStatus::Empty
    } else if beat.gp_rest {
        BeatStatus::Rest
    } else {
        BeatStatus::Normal
    };

    let mut effect = crate::model::legacy::beat::BeatEffects {
        vibrato: beat.gp_vibrato,
        fade_in: beat.gp_fade_in,
        ..Default::default()
    };
    if let Some((value, is_up)) = beat.gp_stroke {
        effect.stroke = BeatStroke {
            direction: if is_up {
                BeatStrokeDirection::Up
            } else {
                BeatStrokeDirection::Down
            },
            value,
            swap: false,
        };
    }
    effect.pick_stroke = match beat.gp_pick_stroke {
        Some(true) => BeatStrokeDirection::Up,
        Some(false) => BeatStrokeDirection::Down,
        None => BeatStrokeDirection::None,
    };
    effect.slap_effect = match beat.gp_slap_effect {
        Some(1) => SlapEffect::Tapping,
        Some(2) => SlapEffect::Slapping,
        Some(3) => SlapEffect::Popping,
        _ => SlapEffect::None,
    };
    effect.has_rasgueado = beat.gp_rasgueado;
    effect.chord = beat.gp_chord.as_ref().map(|gc| {
        use crate::model::legacy::{
            chord::{Barre, Chord, PitchClass},
            enums::{get_chord_alteration, get_chord_extension, get_chord_type, get_fingering},
        };
        let barres = gc
            .barres
            .iter()
            .map(|&(f, s, e)| Barre {
                fret: f as i8,
                start: s as i8,
                end: e as i8,
            })
            .collect();
        Chord {
            sharp: Some(gc.sharp),
            root: Some(PitchClass::from(gc.root, None, Some(gc.sharp))),
            kind: Some(get_chord_type(gc.kind)),
            extension: Some(get_chord_extension(gc.extension)),
            bass: Some(PitchClass::from(gc.bass as i8, None, Some(gc.sharp))),
            tonality: Some(
                get_chord_alteration(gc.tonality as u8)
                    .unwrap_or(crate::model::legacy::enums::ChordAlteration::Perfect),
            ),
            add: Some(gc.add),
            name: gc.name.clone(),
            fifth: Some(
                get_chord_alteration(gc.fifth)
                    .unwrap_or(crate::model::legacy::enums::ChordAlteration::Perfect),
            ),
            ninth: Some(
                get_chord_alteration(gc.ninth)
                    .unwrap_or(crate::model::legacy::enums::ChordAlteration::Perfect),
            ),
            eleventh: Some(
                get_chord_alteration(gc.eleventh)
                    .unwrap_or(crate::model::legacy::enums::ChordAlteration::Perfect),
            ),
            first_fret: Some(gc.first_fret as u8),
            strings: gc.strings.iter().map(|&s| s as i8).collect(),
            barres,
            omissions: gc.omissions.clone(),
            fingerings: gc.fingerings.iter().map(|&f| get_fingering(f)).collect(),
            show: Some(gc.show),
            new_format: Some(gc.new_format),
            length: gc.length,
        }
    });
    effect.tremolo_bar = beat.gp_tremolo_bar.as_ref().map(|tb| {
        use crate::model::legacy::effects::{BendEffect, BendPoint};
        use crate::model::legacy::enums::get_bend_type;
        BendEffect {
            kind: get_bend_type(tb.kind).unwrap_or(crate::model::legacy::enums::BendType::None),
            value: tb.value,
            points: tb
                .points
                .iter()
                .map(|p| BendPoint {
                    position: p.position,
                    value: p.value,
                    vibrato: p.vibrato,
                })
                .collect(),
            ..BendEffect::default()
        }
    });
    effect.mix_table_change = beat.gp_mix_table.as_ref().map(|gmt| {
        let item = |opt: &Option<crate::model::optimized::beat::GpMixTableItem>| {
            opt.as_ref().map(|i| MixTableItem {
                value: i.value,
                duration: i.duration,
                all_tracks: i.all_tracks,
            })
        };
        MixTableChange {
            instrument: item(&gmt.instrument),
            rse: RseInstrument {
                instrument: gmt.rse_instrument,
                unknown: gmt.rse_unknown,
                sound_bank: gmt.rse_sound_bank,
                effect_number: gmt.rse_effect_number,
                effect_category: gmt.rse_effect_category.clone(),
                effect: gmt.rse_effect.clone(),
            },
            volume: item(&gmt.volume),
            balance: item(&gmt.balance),
            chorus: item(&gmt.chorus),
            reverb: item(&gmt.reverb),
            phaser: item(&gmt.phaser),
            tremolo: item(&gmt.tremolo),
            tempo_name: gmt.tempo_name.clone(),
            tempo: item(&gmt.tempo),
            hide_tempo: gmt.hide_tempo,
            wah: gmt.wah.map(|(v, d)| WahEffect {
                value: v,
                display: d,
            }),
            use_rse: gmt.use_rse,
        }
    });

    let mut display = crate::model::legacy::beat::BeatDisplay::default();
    let mut octave = Octave::None;
    if let Some(f) = beat.gp_beat_flags2 {
        display.break_beam = (f & 0x0001) != 0;
        if (f & 0x0002) != 0 {
            display.beam_direction = VoiceDirection::Down;
        }
        display.force_beam = (f & 0x0004) != 0;
        if (f & 0x0008) != 0 {
            display.beam_direction = VoiceDirection::Up;
        }
        if (f & 0x0010) != 0 {
            octave = Octave::Ottava;
        }
        if (f & 0x0020) != 0 {
            octave = Octave::OttavaBassa;
        }
        if (f & 0x0040) != 0 {
            octave = Octave::Quindicesima;
        }
        if (f & 0x0100) != 0 {
            octave = Octave::QuindicesimaBassa;
        }
        if (f & 0x0200) != 0 {
            display.tuplet_bracket = TupletBracket::Start;
        }
        if (f & 0x0400) != 0 {
            display.tuplet_bracket = TupletBracket::End;
        }
        display.break_secondary_tuplet = (f & 0x1000) != 0;
        display.force_bracket = (f & 0x2000) != 0;
    }
    if let Some(bs) = beat.gp_break_secondary {
        display.break_secondary = bs;
    }

    LBeat {
        notes,
        duration,
        start,
        status,
        effect,
        display,
        octave,
        text: beat.gp_text.clone(),
    }
}

fn build_legacy_note(note: &Note, velocity: i16, _strings: &[(i8, i8)]) -> LNote {
    let (string, value, kind) = if note.gp_is_rest {
        // Rest notes: string and value are preserved directly.
        let s = note.string.map(|s| s as i8).unwrap_or(0);
        let v = note.fret.map(|f| f as i16).unwrap_or(0);
        (s, v, NoteType::Rest)
    } else if let (Some(s), Some(f)) = (note.string, note.fret) {
        let kind = if let Some(raw) = note.gp_note_type_raw {
            NoteType::Unknown(raw)
        } else if note.tie == Some(TieType::End) {
            NoteType::Tie
        } else if note.notehead == Some(Notehead::X) {
            NoteType::Dead
        } else {
            NoteType::Normal
        };
        (s as i8, f as i16, kind)
    } else if note.tie == Some(TieType::End) {
        // Tied note without explicit string/fret — keep as tie
        (0, 0, NoteType::Tie)
    } else {
        (0, 0, NoteType::Rest)
    };

    let mut effect = NoteEffect::default();

    // Articulations → note effect
    for art in &note.articulations {
        match art {
            Articulation::Staccato => effect.staccato = true,
            Articulation::Accent => effect.accentuated_note = true,
            Articulation::Marcato => effect.heavy_accentuated_note = true,
            _ => {}
        }
    }

    // Techniques → note effect
    for tech in &note.techniques {
        match tech.kind {
            TechniqueKind::HammerOn | TechniqueKind::PullOff => effect.hammer = true,
            TechniqueKind::LetRing => effect.let_ring = true,
            TechniqueKind::Vibrato => effect.vibrato = true,
            TechniqueKind::HalfMuted => effect.palm_mute = true,
            TechniqueKind::SlideUp => effect.slides.push(SlideType::ShiftSlideTo),
            TechniqueKind::SlideLegato => effect.slides.push(SlideType::LegatoSlideTo),
            TechniqueKind::SlideDown => effect.slides.push(SlideType::OutDownwards),
            TechniqueKind::SlideOutUp => effect.slides.push(SlideType::OutUpWards),
            TechniqueKind::SlideIntoAbove => effect.slides.push(SlideType::IntoFromAbove),
            TechniqueKind::SlideIntoBelow => effect.slides.push(SlideType::IntoFromBelow),
            TechniqueKind::TremoloPicking => {
                use crate::model::legacy::{
                    effects::TremoloPickingEffect,
                    key_signature::{DURATION_EIGHTH, DURATION_SIXTEENTH, DURATION_THIRTY_SECOND},
                };
                use crate::model::optimized::note::NoteValue as ONV;
                let dur_val = if let TechniqueParams::Tremolo { speed } = &tech.params {
                    match speed {
                        ONV::Eighth => DURATION_EIGHTH,
                        ONV::ThirtySecond => DURATION_THIRTY_SECOND,
                        _ => DURATION_SIXTEENTH,
                    }
                } else {
                    DURATION_SIXTEENTH
                };
                effect.tremolo_picking = Some(TremoloPickingEffect {
                    duration: LDuration {
                        value: u16::from(dur_val),
                        ..LDuration::default()
                    },
                });
            }
            _ => {}
        }
    }

    // Bend
    if let Some(bend) = &note.gp_bend {
        use crate::model::legacy::effects::{BendEffect, BendPoint};
        use crate::model::legacy::enums::get_bend_type;
        effect.bend = Some(BendEffect {
            kind: get_bend_type(bend.kind).unwrap_or(crate::model::legacy::enums::BendType::None),
            value: bend.value,
            points: bend
                .points
                .iter()
                .map(|p| BendPoint {
                    position: p.position,
                    value: p.value,
                    vibrato: p.vibrato,
                })
                .collect(),
            ..BendEffect::default()
        });
    }

    // Harmonic
    if let Some(h) = &note.gp_harmonic {
        effect.harmonic = Some(HarmonicEffect {
            kind: match h.kind {
                2 => HarmonicType::Artificial,
                3 => HarmonicType::Tapped,
                4 => HarmonicType::Pinch,
                5 => HarmonicType::Semi,
                _ => HarmonicType::Natural,
            },
            pitch: h
                .pitch_just
                .map(|just| PitchClass::from(just, h.pitch_accidental, None)),
            octave: h.octave.map(|o| match o {
                1 => Octave::Ottava,
                2 => Octave::Quindicesima,
                3 => Octave::OttavaBassa,
                4 => Octave::QuindicesimaBassa,
                _ => Octave::None,
            }),
            fret: h.fret,
        });
    }

    // Grace note
    if let Some(g) = &note.gp_grace {
        use crate::model::legacy::enums::get_grace_effect_transition;
        effect.grace = Some(GraceEffect {
            fret: g.fret,
            velocity: g.velocity,
            duration: g.duration,
            transition: get_grace_effect_transition(g.transition)
                .unwrap_or(GraceEffectTransition::None),
            is_dead: g.is_dead,
            is_on_beat: g.is_on_beat,
        });
    }

    // Trill
    if let Some(t) = &note.gp_trill {
        use crate::model::legacy::key_signature::{
            DURATION_SIXTEENTH, DURATION_SIXTY_FOURTH, DURATION_THIRTY_SECOND,
        };
        let dur_val = match t.period {
            2 => DURATION_THIRTY_SECOND,
            3 => DURATION_SIXTY_FOURTH,
            _ => DURATION_SIXTEENTH,
        };
        effect.trill = Some(crate::model::legacy::effects::TrillEffect {
            fret: t.fret,
            duration: LDuration {
                value: u16::from(dur_val),
                ..LDuration::default()
            },
        });
    }

    // Ghost note
    effect.ghost_note = note.gp_ghost;

    // Finger
    if let Some(f) = note.left_finger {
        effect.left_hand_finger = finger_to_legacy(f);
    }
    if let Some(f) = note.right_finger {
        effect.right_hand_finger = finger_to_legacy(f);
    }

    // Use per-note gp_velocity when available (preserves byte-identical roundtrip for
    // beats where notes have different individual velocities). Fall back to beat-level.
    let note_velocity = note.gp_velocity.unwrap_or(velocity);
    LNote {
        value,
        velocity: note_velocity,
        string,
        effect,
        kind,
        duration_percent: note.gp_duration_percent,
        swap_accidentals: note.gp_swap_accidentals,
        duration: note.gp_note_duration,
        tuplet: note.gp_note_tuplet,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn duration_to_legacy(dur: &Duration) -> LDuration {
    let value = match dur.base {
        NoteValue::Whole => 1u16,
        NoteValue::Half => 2,
        NoteValue::Quarter => 4,
        NoteValue::Eighth => 8,
        NoteValue::Sixteenth => 16,
        NoteValue::ThirtySecond => 32,
        NoteValue::SixtyFourth => 64,
        NoteValue::HundredTwentyEighth => 128,
        NoteValue::Other(v) => v,
    };
    let dotted = dur.dots > 0;
    let (tuplet_enters, tuplet_times) = dur.tuplet.map(|t| (t.actual, t.normal)).unwrap_or((1, 1));
    LDuration {
        value,
        dotted,
        tuplet_enters,
        tuplet_times,
        ..LDuration::default()
    }
}

pub(crate) fn pitch_to_midi(pitch: &Pitch) -> i8 {
    let base: i32 = match pitch.step {
        PitchStep::C => 0,
        PitchStep::D => 2,
        PitchStep::E => 4,
        PitchStep::F => 5,
        PitchStep::G => 7,
        PitchStep::A => 9,
        PitchStep::B => 11,
    };
    let midi = (pitch.octave as i32 + 1) * 12 + base + pitch.alter as i32;
    midi.clamp(-128, 127) as i8
}

fn dynamic_to_velocity(dyn_opt: Option<Dynamic>) -> i16 {
    // Reconstruct exact velocity values used in legacy model
    // MIN_VELOCITY=15, VELOCITY_INCREMENT=16, packed 1..8 → 15,31,47,63,79,95,111,127
    match dyn_opt {
        None => DEFAULT_VELOCITY, // 95 = FORTE
        Some(Dynamic::PPP) => 15,
        Some(Dynamic::PP) => 31,
        Some(Dynamic::P) => 47,
        Some(Dynamic::MP) => 63,
        Some(Dynamic::MF) => 79,
        Some(Dynamic::F) => 95,
        Some(Dynamic::FF) => 111,
        Some(Dynamic::FFF) => 127,
    }
}

pub(crate) fn opt_key_to_legacy_key(ks: &KeySignature) -> LKeySignature {
    let is_minor = matches!(ks.mode, Mode::Minor);
    let key = pitch_and_mode_to_fifths(&ks.root, ks.mode);
    LKeySignature { key, is_minor }
}

fn pitch_and_mode_to_fifths(root: &Pitch, mode: Mode) -> i8 {
    // Reverse lookup: (step, alter) → fifths, preserving enharmonic spelling.
    // Major: C=0 G=1 D=2 A=3 E=4 B=5 F#=6 C#=7 F=-1 Bb=-2 Eb=-3 Ab=-4 Db=-5 Gb=-6 Cb=-7
    // Minor: Am=0 Em=1 Bm=2 F#m=3 C#m=4 G#m=5 D#m=6 A#m=7
    //        Dm=-1 Gm=-2 Cm=-3 Fm=-4 Bbm=-5 Ebm=-6 Abm=-7
    use PitchStep::*;
    let key = (root.step, root.alter);
    if matches!(mode, Mode::Minor) {
        match key {
            (A, 0) => 0,
            (E, 0) => 1,
            (B, 0) => 2,
            (F, 1) => 3, // F#
            (C, 1) => 4, // C#
            (G, 1) => 5, // G#
            (D, 1) => 6, // D#
            (A, 1) => 7, // A#
            (D, 0) => -1,
            (G, 0) => -2,
            (C, 0) => -3,
            (F, 0) => -4,
            (B, -1) => -5, // Bb
            (E, -1) => -6, // Eb
            (A, -1) => -7, // Ab
            _ => 0,
        }
    } else {
        match key {
            (C, 0) => 0,
            (G, 0) => 1,
            (D, 0) => 2,
            (A, 0) => 3,
            (E, 0) => 4,
            (B, 0) => 5,
            (F, 1) => 6, // F#
            (C, 1) => 7, // C#
            (F, 0) => -1,
            (B, -1) => -2, // Bb
            (E, -1) => -3, // Eb
            (A, -1) => -4, // Ab
            (D, -1) => -5, // Db
            (G, -1) => -6, // Gb
            (C, -1) => -7, // Cb
            _ => 0,
        }
    }
}

fn flags_to_track_settings(f: i16) -> crate::model::legacy::track::TrackSettings {
    crate::model::legacy::track::TrackSettings {
        tablature: (f & 0x0001) != 0,
        notation: (f & 0x0002) != 0,
        diagram_are_below: (f & 0x0004) != 0,
        show_rythm: (f & 0x0008) != 0,
        force_horizontal: (f & 0x0010) != 0,
        force_channels: (f & 0x0020) != 0,
        diagram_list: (f & 0x0040) != 0,
        diagram_in_score: (f & 0x0080) != 0,
        auto_let_ring: (f & 0x0200) != 0,
        auto_brush: (f & 0x0400) != 0,
        extend_rythmic: (f & 0x0800) != 0,
    }
}

fn build_track_rse(get: &impl Fn(&str) -> Option<String>) -> crate::model::legacy::rse::TrackRse {
    use crate::model::legacy::{
        enums::Accentuation,
        rse::{RseEqualizer, RseInstrument, TrackRse},
    };

    let humanize = get("rse_humanize")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0u8);

    let auto_accentuation = get("rse_auto_acc")
        .and_then(|v| v.parse::<u8>().ok())
        .map(|n| match n {
            1 => Accentuation::VerySoft,
            2 => Accentuation::Soft,
            3 => Accentuation::Medium,
            4 => Accentuation::Strong,
            5 => Accentuation::VeryStrong,
            _ => Accentuation::None,
        })
        .unwrap_or(Accentuation::None);

    let equalizer = if let Some(eq_str) = get("rse_eq") {
        let knobs: Vec<f32> = eq_str
            .split(',')
            .filter_map(|s| s.parse::<i8>().ok())
            .map(|packed| -(packed as f32) / 10.0)
            .collect();
        RseEqualizer { knobs, gain: 0.0 }
    } else {
        RseEqualizer {
            knobs: vec![0.0; 4],
            gain: 0.0,
        }
    };

    let instrument = if let Some(instr_str) = get("rse_instr") {
        let parts: Vec<&str> = instr_str.split(',').collect();
        if parts.len() == 4 {
            RseInstrument {
                instrument: parts[0].parse().unwrap_or(-1),
                unknown: parts[1].parse().unwrap_or(-1),
                sound_bank: parts[2].parse().unwrap_or(-1),
                effect_number: parts[3].parse().unwrap_or(-1),
                effect: get("rse_effect").unwrap_or_default(),
                effect_category: get("rse_effect_cat").unwrap_or_default(),
            }
        } else {
            RseInstrument::default()
        }
    } else {
        RseInstrument::default()
    };

    TrackRse {
        humanize,
        auto_accentuation,
        equalizer,
        instrument,
    }
}

fn finger_to_legacy(f: Finger) -> LFingering {
    match f {
        Finger::Thumb => LFingering::Thumb,
        Finger::Index => LFingering::Index,
        Finger::Middle => LFingering::Middle,
        Finger::Ring => LFingering::Annular,
        Finger::Pinky => LFingering::Little,
        Finger::Open => LFingering::Open,
    }
}
