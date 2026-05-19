//! Direct conversion from `musicxml::ScorePartwise` to `legacy::Song`.
//!
//! Entry point: [`musicxml_to_legacy_song`].
//!
//! Converts MusicXML elements directly to the Guitar Pro legacy model without
//! passing through the optimized intermediate representation.

use std::collections::HashMap;

use crate::{
    audio::midi::MidiChannel,
    model::{
        legacy::{
            beat::{Beat, Voice},
            enums::{BeatStatus, NoteType as LegacyNoteType},
            headers::MeasureHeader,
            key_signature::{
                DURATION_QUARTER_TIME, Duration as LegacyDuration, KeySignature, TimeSignature,
            },
            measure::Measure,
            note::Note as LegacyNote,
            song::Song,
            track::Track,
        },
        musicxml::{
            Part, ScorePartwise,
            measure::MusicData,
            note::{Note as XmlNote, NoteTypeValue},
            part_list::PartListItem,
        },
    },
};

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Convert a MusicXML [`ScorePartwise`] document into a Guitar Pro [`Song`].
///
/// Maps directly from MusicXML elements without passing through the optimized
/// intermediate model.
pub fn musicxml_to_legacy_song(src: &ScorePartwise) -> Song {
    let mut song = Song::default();

    build_metadata(&mut song, src);
    build_channels(&mut song, src);

    // Extract global tempo from first part's first measure
    let global_tempo = extract_global_tempo(src).unwrap_or(song.tempo);
    song.tempo = global_tempo;

    song.measure_headers = build_measure_headers(src, global_tempo);
    let measure_count = song.measure_headers.len();
    song.tracks = build_tracks(src, measure_count);

    song
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

fn build_metadata(song: &mut Song, src: &ScorePartwise) {
    if let Some(work) = &src.work
        && let Some(title) = &work.work_title
    {
        song.name = title.clone();
    }
    if song.name.is_empty()
        && let Some(mt) = &src.movement_title
    {
        song.name = mt.clone();
    }

    let Some(ident) = &src.identification else {
        return;
    };

    for creator in &ident.creators {
        match creator.creator_type.as_deref() {
            Some("composer") => song.artist = creator.value.clone(),
            Some("arranger") => song.author = creator.value.clone(),
            Some("lyricist") => song.words = creator.value.clone(),
            Some("transcriber") => song.writer = creator.value.clone(),
            _ => {}
        }
    }

    if let Some(rights) = ident.rights.first() {
        song.copyright = rights.value.clone();
    }

    if let Some(misc) = &ident.miscellaneous {
        for field in &misc.fields {
            match field.name.as_str() {
                "subtitle" => song.subtitle = field.value.clone(),
                "album" => song.album = field.value.clone(),
                "date" => song.date = field.value.clone(),
                "instructions" => song.instructions = field.value.clone(),
                name if name.starts_with("notice-") => {
                    song.notice.push(field.value.clone());
                }
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Channels (one per score-part)
// ---------------------------------------------------------------------------

fn build_channels(song: &mut Song, src: &ScorePartwise) {
    for item in &src.part_list.items {
        let PartListItem::ScorePart(sp) = item else {
            continue;
        };
        let mut channel = MidiChannel::default();
        if let Some(midi) = sp.midi_instruments.first() {
            if let Some(ch) = midi.midi_channel {
                channel.channel = ch.saturating_sub(1);
                channel.effect_channel = ch; // GP convention: effect = ch + 1
            }
            if let Some(prog) = midi.midi_program {
                channel.instrument = (prog as i32).saturating_sub(1);
            }
            if let Some(vol) = midi.volume {
                channel.volume = (vol / 100.0 * 127.0).round().clamp(0.0, 127.0) as i8;
            }
            if let Some(pan) = midi.pan {
                channel.balance = ((pan / 90.0 * 63.0) + 64.0).round().clamp(0.0, 127.0) as i8;
            }
            if let Some(bank) = midi.midi_bank {
                channel.bank = bank as u8;
            }
        }
        song.channels.push(channel);
    }

    // Always ensure at least one channel
    if song.channels.is_empty() {
        song.channels.push(MidiChannel::default());
    }
}

// ---------------------------------------------------------------------------
// Tempo extraction
// ---------------------------------------------------------------------------

fn extract_global_tempo(src: &ScorePartwise) -> Option<i16> {
    let first = first_part(src)?;
    let first_measure = first.measures.first()?;
    for event in &first_measure.music_data {
        match event {
            MusicData::Direction(dir) => {
                if let Some(sound) = &dir.sound
                    && let Some(t) = sound.tempo
                {
                    return Some(t.round() as i16);
                }
            }
            MusicData::Sound(sound) => {
                if let Some(t) = sound.tempo {
                    return Some(t.round() as i16);
                }
            }
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Measure headers (from first part)
// ---------------------------------------------------------------------------

fn build_measure_headers(src: &ScorePartwise, global_tempo: i16) -> Vec<MeasureHeader> {
    let Some(first) = first_part(src) else {
        return vec![];
    };

    let mut headers = Vec::with_capacity(first.measures.len());
    let mut running_key = KeySignature::default();
    let mut running_time = TimeSignature::default();
    let mut current_tempo = global_tempo;
    let mut start: i64 = DURATION_QUARTER_TIME;

    for (mi, xml_measure) in first.measures.iter().enumerate() {
        let mut tempo_change: Option<i16> = None;
        let mut repeat_open = false;
        let mut repeat_close: i8 = -1;

        for event in &xml_measure.music_data {
            match event {
                MusicData::Attributes(attr) => {
                    if let Some(key) = attr.keys.first()
                        && let Some(fifths) = key.fifths
                    {
                        running_key.key = fifths;
                        running_key.is_minor = key.mode.as_deref() == Some("minor");
                    }
                    if let Some(time) = attr.times.first() {
                        let num = time
                            .beats
                            .as_deref()
                            .unwrap_or("4")
                            .parse::<i8>()
                            .unwrap_or(4);
                        let den = time
                            .beat_type
                            .as_deref()
                            .unwrap_or("4")
                            .parse::<u16>()
                            .unwrap_or(4);
                        running_time = TimeSignature {
                            numerator: num,
                            denominator: LegacyDuration {
                                value: den,
                                ..LegacyDuration::default()
                            },
                            beams: vec![2, 2, 2, 2],
                        };
                    }
                }
                MusicData::Direction(dir) => {
                    if let Some(sound) = &dir.sound
                        && let Some(t) = sound.tempo
                    {
                        let t = t.round() as i16;
                        if mi > 0 {
                            current_tempo = t;
                            tempo_change = Some(t);
                        }
                    }
                }
                MusicData::Sound(sound) => {
                    if let Some(t) = sound.tempo {
                        let t = t.round() as i16;
                        if mi > 0 {
                            current_tempo = t;
                            tempo_change = Some(t);
                        }
                    }
                }
                MusicData::Barline(b) => {
                    if let Some(rep) = &b.repeat {
                        match rep.direction.as_str() {
                            "forward" => repeat_open = true,
                            "backward" => repeat_close = rep.times.unwrap_or(1) as i8,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        let _ = current_tempo; // used to track running tempo

        let header = MeasureHeader {
            number: (mi + 1) as u16,
            start,
            time_signature: running_time.clone(),
            tempo: tempo_change.map(|t| t as i32).unwrap_or(0),
            repeat_open,
            repeat_close,
            key_signature: running_key.clone(),
            ..MeasureHeader::default()
        };

        let beat_duration = DURATION_QUARTER_TIME * 4 / running_time.denominator.value as i64;
        start += running_time.numerator as i64 * beat_duration;

        headers.push(header);
    }

    headers
}

// ---------------------------------------------------------------------------
// Track building
// ---------------------------------------------------------------------------

fn build_tracks(src: &ScorePartwise, measure_count: usize) -> Vec<Track> {
    let score_parts: Vec<_> = src
        .part_list
        .items
        .iter()
        .filter_map(|item| {
            if let PartListItem::ScorePart(sp) = item {
                Some(sp)
            } else {
                None
            }
        })
        .collect();

    score_parts
        .iter()
        .enumerate()
        .map(|(ti, sp)| {
            let part = src.parts.iter().find(|p| p.id == sp.id);
            let strings = extract_string_tunings(part);
            let percussion = sp
                .midi_instruments
                .first()
                .map(|m| m.midi_unpitched.is_some())
                .unwrap_or(false);
            let name = sp
                .part_name
                .as_ref()
                .and_then(|pn| pn.value.as_deref())
                .unwrap_or("Track")
                .to_string();
            let short_name = sp
                .part_abbreviation
                .as_ref()
                .and_then(|pn| pn.value.as_deref())
                .unwrap_or("")
                .to_string();

            let mut track = Track {
                number: ti as i32,
                name,
                short_name,
                strings: strings.clone(),
                channel_index: ti.min(63), // channels capped at 64
                percussion_track: percussion,
                ..Track::default()
            };

            if let Some(part) = part {
                let mut divisions: u32 = 960;
                for (mi, xml_measure) in part.measures.iter().enumerate() {
                    let measure =
                        build_legacy_measure(xml_measure, mi, ti, &strings, &mut divisions);
                    track.measures.push(measure);
                }
            }

            // Pad with empty measures if the part has fewer measures than headers
            while track.measures.len() < measure_count {
                let mi = track.measures.len();
                track.measures.push(Measure {
                    header_index: mi,
                    track_index: ti,
                    voices: vec![Voice::default()],
                    ..Measure::default()
                });
            }

            track
        })
        .collect()
}

fn extract_string_tunings(part: Option<&Part>) -> Vec<(i8, i8)> {
    if let Some(part) = part
        && let Some(first_measure) = part.measures.first()
    {
        for event in &first_measure.music_data {
            if let MusicData::Attributes(attr) = event {
                for sd in &attr.staff_details {
                    if !sd.staff_tunings.is_empty() {
                        // MusicXML: line 1 = lowest string.
                        // GP: string 1 = highest-pitched.
                        // Reverse so highest-pitched becomes string 1.
                        let mut midis: Vec<i8> = sd
                            .staff_tunings
                            .iter()
                            .map(|st| {
                                pitch_step_to_midi(
                                    &st.tuning_step,
                                    st.tuning_alter,
                                    st.tuning_octave,
                                )
                            })
                            .collect();
                        midis.reverse();
                        return midis
                            .into_iter()
                            .enumerate()
                            .map(|(i, midi)| ((i + 1) as i8, midi))
                            .collect();
                    }
                }
            }
        }
    }
    // Default: standard 6-string guitar (E4 B3 G3 D3 A2 E2)
    vec![(1, 64), (2, 59), (3, 55), (4, 50), (5, 45), (6, 40)]
}

fn pitch_step_to_midi(step: &str, alter: Option<f64>, octave: i8) -> i8 {
    let base: i32 = match step {
        "C" => 0,
        "D" => 2,
        "E" => 4,
        "F" => 5,
        "G" => 7,
        "A" => 9,
        "B" => 11,
        _ => 0,
    };
    let alt = alter.unwrap_or(0.0).round() as i32;
    ((octave as i32 + 1) * 12 + base + alt).clamp(-128, 127) as i8
}

// ---------------------------------------------------------------------------
// Measure → legacy
// ---------------------------------------------------------------------------

fn build_legacy_measure(
    xml_measure: &crate::model::musicxml::measure::Measure,
    header_index: usize,
    track_index: usize,
    strings: &[(i8, i8)],
    divisions: &mut u32,
) -> Measure {
    let voices = build_voices(xml_measure, strings, divisions);
    Measure {
        header_index,
        track_index,
        voices,
        ..Measure::default()
    }
}

// ---------------------------------------------------------------------------
// Voice / beat building
// ---------------------------------------------------------------------------

fn build_voices(
    xml_measure: &crate::model::musicxml::measure::Measure,
    strings: &[(i8, i8)],
    divisions: &mut u32,
) -> Vec<Voice> {
    let mut cursor: i64 = 0;
    let mut last_onset: HashMap<String, i64> = HashMap::new();
    // Collect (voice_str, onset, &XmlNote)
    let mut pending: Vec<(String, i64, &XmlNote)> = vec![];

    for event in &xml_measure.music_data {
        match event {
            MusicData::Attributes(attr) => {
                if let Some(div) = attr.divisions {
                    *divisions = div;
                }
            }
            MusicData::Backup(b) => {
                cursor = (cursor - b.duration as i64).max(0);
            }
            MusicData::Forward(f) => {
                cursor += f.duration as i64;
            }
            MusicData::Note(note) => {
                if note.grace.is_some() {
                    continue; // skip grace notes
                }
                let voice_str = note.voice.clone().unwrap_or_else(|| "1".to_string());
                let onset = if note.chord.is_some() {
                    // Chord note: same onset as preceding note in this voice
                    *last_onset.get(&voice_str).unwrap_or(&cursor)
                } else {
                    let o = cursor;
                    cursor += note.duration.unwrap_or(0) as i64;
                    o
                };
                last_onset.insert(voice_str.clone(), onset);
                pending.push((voice_str, onset, note));
            }
            _ => {}
        }
    }

    // Group by voice, then by onset (consecutive same-onset = chord)
    let mut voices_map: HashMap<String, Vec<(i64, Vec<&XmlNote>)>> = HashMap::new();
    for (voice_str, onset, note) in &pending {
        let beats = voices_map.entry(voice_str.clone()).or_default();
        match beats.last_mut() {
            Some(last) if last.0 == *onset => last.1.push(note),
            _ => beats.push((*onset, vec![note])),
        }
    }

    // Build Voices in sorted voice order
    let mut voice_keys: Vec<String> = voices_map.keys().cloned().collect();
    voice_keys.sort();

    if voice_keys.is_empty() {
        // Empty measure: one voice with a whole-measure rest
        return vec![Voice::default()];
    }

    voice_keys
        .iter()
        .map(|vk| {
            let beats_data = &voices_map[vk];
            let beats = beats_data
                .iter()
                .map(|(_, notes)| build_beat(notes, strings, *divisions))
                .collect();
            Voice {
                beats,
                ..Voice::default()
            }
        })
        .collect()
}

fn build_beat(notes: &[&XmlNote], strings: &[(i8, i8)], divisions: u32) -> Beat {
    let first = notes[0];
    let duration = xml_note_to_duration(first, divisions);

    let all_rests = notes.iter().all(|n| n.rest.is_some());
    if all_rests {
        return Beat {
            duration,
            status: BeatStatus::Rest,
            ..Beat::default()
        };
    }

    let legacy_notes: Vec<LegacyNote> = notes
        .iter()
        .filter_map(|n| xml_note_to_legacy(n, strings))
        .collect();

    if legacy_notes.is_empty() {
        return Beat {
            duration,
            status: BeatStatus::Rest,
            ..Beat::default()
        };
    }

    Beat {
        notes: legacy_notes,
        duration,
        status: BeatStatus::Normal,
        ..Beat::default()
    }
}

fn xml_note_to_legacy(note: &XmlNote, strings: &[(i8, i8)]) -> Option<LegacyNote> {
    if note.rest.is_some() {
        return None;
    }

    let kind = if note.ties.iter().any(|t| t.tie_type == "stop") {
        LegacyNoteType::Tie
    } else {
        LegacyNoteType::Normal
    };

    // Prefer explicit string/fret from <technical> if present
    let tab = note.notations.iter().find_map(|n| {
        n.technical.as_ref().and_then(|tech| {
            if let (Some(s), Some(f)) = (&tech.string, &tech.fret) {
                Some((s.value as i8, f.value as i16))
            } else {
                None
            }
        })
    });

    let (string_num, fret_val) = if let Some(tab) = tab {
        tab
    } else if let Some(pitch) = &note.pitch {
        pitch_to_string_fret(xml_pitch_to_midi(pitch), strings)?
    } else {
        return None;
    };

    Some(LegacyNote {
        value: fret_val,
        string: string_num,
        kind,
        ..LegacyNote::default()
    })
}

// ---------------------------------------------------------------------------
// Duration helpers
// ---------------------------------------------------------------------------

fn xml_note_to_duration(note: &XmlNote, divisions: u32) -> LegacyDuration {
    let value = if let Some(nt) = &note.note_type {
        note_type_value_to_u16(nt.value)
    } else if let Some(dur) = note.duration {
        if dur == 0 || divisions == 0 {
            4
        } else {
            let v = ((divisions as f64 * 4.0) / dur as f64).round() as u16;
            match v {
                1 | 2 | 4 | 8 | 16 | 32 | 64 | 128 => v,
                _ => 4,
            }
        }
    } else {
        4
    };

    let dotted = !note.dots.is_empty();
    let (tuplet_enters, tuplet_times) = note
        .time_modification
        .as_ref()
        .map(|tm| (tm.actual_notes, tm.normal_notes))
        .unwrap_or((1, 1));

    LegacyDuration {
        value,
        dotted,
        tuplet_enters,
        tuplet_times,
        ..LegacyDuration::default()
    }
}

fn note_type_value_to_u16(v: NoteTypeValue) -> u16 {
    match v {
        NoteTypeValue::Whole => 1,
        NoteTypeValue::Half => 2,
        NoteTypeValue::Quarter => 4,
        NoteTypeValue::Eighth => 8,
        NoteTypeValue::N16th => 16,
        NoteTypeValue::N32nd => 32,
        NoteTypeValue::N64th => 64,
        NoteTypeValue::N128th => 128,
        _ => 4, // Breve, Long, Maxima, N256th, N512th, N1024th → fallback
    }
}

// ---------------------------------------------------------------------------
// Pitch helpers
// ---------------------------------------------------------------------------

fn xml_pitch_to_midi(pitch: &crate::model::musicxml::note::Pitch) -> i32 {
    let base: i32 = match pitch.step.as_str() {
        "C" => 0,
        "D" => 2,
        "E" => 4,
        "F" => 5,
        "G" => 7,
        "A" => 9,
        "B" => 11,
        _ => 0,
    };
    let alter = pitch.alter.unwrap_or(0.0).round() as i32;
    (pitch.octave as i32 + 1) * 12 + base + alter
}

fn pitch_to_string_fret(midi: i32, strings: &[(i8, i8)]) -> Option<(i8, i16)> {
    let mut best: Option<(i8, i16)> = None;
    for &(string_num, open_midi) in strings {
        let fret = midi - open_midi as i32;
        if (0..=24).contains(&fret) {
            let fret = fret as i16;
            if best.is_none_or(|(_, bf)| fret < bf) {
                best = Some((string_num, fret));
            }
        }
    }
    best
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn first_part(src: &ScorePartwise) -> Option<&Part> {
    let first_id = src.part_list.items.iter().find_map(|i| {
        if let PartListItem::ScorePart(sp) = i {
            Some(sp.id.as_str())
        } else {
            None
        }
    })?;
    src.parts.iter().find(|p| p.id == first_id)
}
