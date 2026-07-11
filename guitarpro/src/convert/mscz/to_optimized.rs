//! MSCX → [`LoadedScore`] conversion.

use std::collections::{BTreeMap, HashMap};

use crate::convert::mscz::validate::LossReport;
use crate::convert::optimized::legacy::midi_to_pitch;
use crate::model::mscz::{
    Instrument as MscxInstrument, MetaTag, Mscx, MscxBeat, MscxBeatKind, MscxDuration,
    MscxDurationKind, MscxKeySig, MscxMeasure, MscxNote, MscxStaff, MscxTimeSig, MscxVoice,
    Part as MscxPart, Staff as MscxPartStaff,
};
use crate::model::optimized::{
    LoadedScore,
    beat::{Beat, Duration, Voice},
    global::{
        Instrument, InstrumentId, InstrumentKind, MeasureIndex, PartGroup, Score, StaffId, TrackId,
        Transpose,
    },
    metadata::{Creator, Identification, KeySignature, Metadata, Mode, TimeSignature, Work},
    note::{Note, NoteValue, Pitch, PitchStep, TieType},
    timeline::{JumpKind, MeasureDef, NavigationEvent},
    track::{Clef, MeasureData, StaffDef, StaffDisplay, Track},
};

/// Ticks per quarter note used by the produced [`LoadedScore`].
///
/// Matches [`crate::convert::musicxml::DIVISIONS`] so tick arithmetic aligns
/// with the shared timeline expected by other converters.
pub const DIVISIONS: u32 = crate::model::legacy::key_signature::DURATION_QUARTER_TIME as u32;

const DEFAULT_TEMPO: f32 = 120.0;

/// Result of a MSCX → LoadedScore conversion.
pub struct ConvertOutcome {
    pub score: LoadedScore,
    pub report: LossReport,
}

/// Convert a parsed [`Mscx`] into a [`LoadedScore`], gathering unhandled
/// elements into a [`LossReport`].
pub fn mscx_to_loaded_score(mscx: &Mscx) -> ConvertOutcome {
    let mut report = LossReport::new();

    let metadata = build_metadata(mscx);
    let instruments = build_instruments(&mscx.parts);
    let (staves, part_staff_ids) = build_staves(&mscx.parts);
    let timeline = build_timeline(mscx, &mut report);
    let tracks = build_tracks(mscx, &part_staff_ids, timeline.len(), &mut report);

    let score = LoadedScore {
        score: Score {
            metadata,
            instruments,
            staves,
            tracks,
            groups: Vec::<PartGroup>::new(),
            timeline,
            lyric_lines: Vec::new(),
            lyric_projections: Vec::new(),
            defaults: None,
        },
        layout: None,
    };

    ConvertOutcome { score, report }
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

fn build_metadata(mscx: &Mscx) -> Metadata {
    let meta = |name: &str| meta_value(&mscx.meta_tags, name);

    let title = meta("workTitle").unwrap_or_default();
    let movement_title = meta("movementTitle");
    let effective_title = if !title.is_empty() {
        title.clone()
    } else {
        movement_title.clone().unwrap_or_default()
    };

    let composer = meta("composer").filter(|s| !s.is_empty());
    let lyricist = meta("lyricist").filter(|s| !s.is_empty());
    let arranger = meta("arranger").filter(|s| !s.is_empty());
    let copyright = meta("copyright").filter(|s| !s.is_empty());
    let source = meta("source").filter(|s| !s.is_empty());
    let encoding_date = meta("creationDate").filter(|s| !s.is_empty());

    let mut creators: Vec<Creator> = Vec::new();
    if let Some(name) = composer.clone() {
        creators.push(Creator {
            role: "composer".to_string(),
            name,
        });
    }
    if let Some(name) = lyricist {
        creators.push(Creator {
            role: "lyricist".to_string(),
            name,
        });
    }
    if let Some(name) = arranger {
        creators.push(Creator {
            role: "arranger".to_string(),
            name,
        });
    }

    let identification = if !creators.is_empty()
        || copyright.is_some()
        || source.is_some()
        || encoding_date.is_some()
    {
        Some(Identification {
            creators,
            rights: copyright.clone(),
            encoding_software: Some(format!(
                "MuseScore {}",
                mscx.program_version
                    .clone()
                    .unwrap_or_else(|| "4".to_string())
            )),
            encoding_date,
            source,
            miscellaneous: Vec::new(),
        })
    } else {
        None
    };

    let work_number = meta("workNumber").filter(|s| !s.is_empty());
    let work = if !title.is_empty() || work_number.is_some() {
        Some(Work {
            number: work_number,
            title: Some(title.clone()).filter(|s| !s.is_empty()),
            opus: None,
        })
    } else {
        None
    };

    let (tempo, time_signature, key_signature) = derive_initial_signatures(mscx);

    Metadata {
        work,
        movement_number: meta("movementNumber").filter(|s| !s.is_empty()),
        title: effective_title,
        artist: meta("artist").filter(|s| !s.is_empty()),
        album: meta("album").filter(|s| !s.is_empty()),
        composer,
        year: encoding_year_from_meta(&mscx.meta_tags),
        copyright,
        identification,
        credits: Vec::new(),
        master_tempo: tempo,
        time_signature,
        key_signature,
        chords: Vec::new(),
        scale_hint: None,
    }
}

fn encoding_year_from_meta(tags: &[MetaTag]) -> Option<u16> {
    let raw = meta_value(tags, "creationDate")?;
    raw.split('-').next()?.parse::<u16>().ok()
}

fn meta_value(tags: &[MetaTag], name: &str) -> Option<String> {
    tags.iter()
        .find(|tag| tag.name == name)
        .map(|tag| tag.value.clone())
}

fn derive_initial_signatures(mscx: &Mscx) -> (f32, TimeSignature, KeySignature) {
    let first = mscx
        .score_staves
        .first()
        .and_then(|staff| staff.measures.first());

    let tempo_bpm = first
        .and_then(|measure| measure.tempo_bps)
        .map(bps_to_bpm)
        .unwrap_or(DEFAULT_TEMPO);

    let time_signature = first
        .and_then(|measure| measure.time_sig)
        .map(convert_time_sig)
        .unwrap_or(TimeSignature {
            numerator: 4,
            denominator: 4,
        });

    let key_signature = first
        .and_then(|measure| measure.key_sig)
        .map(convert_key_sig)
        .unwrap_or_else(default_key_signature);

    (tempo_bpm, time_signature, key_signature)
}

fn default_key_signature() -> KeySignature {
    KeySignature {
        root: Pitch {
            step: PitchStep::C,
            alter: 0,
            octave: 4,
        },
        mode: Mode::Major,
    }
}

fn convert_time_sig(sig: MscxTimeSig) -> TimeSignature {
    TimeSignature {
        numerator: sig.numerator.max(1),
        denominator: sig.denominator.max(1),
    }
}

fn convert_key_sig(sig: MscxKeySig) -> KeySignature {
    KeySignature {
        root: root_from_fifths(sig.fifths),
        mode: Mode::Major,
    }
}

/// Map a signed fifth count to the corresponding major-key tonic in
/// scientific pitch notation (octave 4).
fn root_from_fifths(fifths: i8) -> Pitch {
    let (step, alter) = match fifths {
        -7 => (PitchStep::C, -1),
        -6 => (PitchStep::G, -1),
        -5 => (PitchStep::D, -1),
        -4 => (PitchStep::A, -1),
        -3 => (PitchStep::E, -1),
        -2 => (PitchStep::B, -1),
        -1 => (PitchStep::F, 0),
        0 => (PitchStep::C, 0),
        1 => (PitchStep::G, 0),
        2 => (PitchStep::D, 0),
        3 => (PitchStep::A, 0),
        4 => (PitchStep::E, 0),
        5 => (PitchStep::B, 0),
        6 => (PitchStep::F, 1),
        7 => (PitchStep::C, 1),
        _ => (PitchStep::C, 0),
    };
    Pitch {
        step,
        alter,
        octave: 4,
    }
}

fn bps_to_bpm(bps: f32) -> f32 {
    bps * 60.0
}

// ---------------------------------------------------------------------------
// Instruments
// ---------------------------------------------------------------------------

fn build_instruments(parts: &[MscxPart]) -> Vec<Instrument> {
    parts
        .iter()
        .enumerate()
        .map(|(index, part)| convert_instrument(index, part))
        .collect()
}

fn convert_instrument(index: usize, part: &MscxPart) -> Instrument {
    let default_name = format!("Part {}", index + 1);
    let inst = part.instrument.as_ref();

    let name = inst
        .and_then(|i| i.long_name.clone())
        .filter(|s| !s.is_empty())
        .or_else(|| part.track_name.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or(default_name);

    let abbreviation = inst
        .and_then(|i| i.short_name.clone())
        .filter(|s| !s.is_empty());

    let instrument_sound = inst
        .and_then(|i| i.instrument_id.clone())
        .filter(|s| !s.is_empty());

    let transpose = build_transpose(inst);
    let kind = build_instrument_kind(inst);

    Instrument {
        name,
        abbreviation,
        instrument_sound,
        midi_channel: 0,
        midi_program: 0,
        midi_bank: None,
        volume: None,
        pan: None,
        kind,
        transpose,
        gp_strings: Vec::new(),
    }
}

fn build_transpose(inst: Option<&MscxInstrument>) -> Option<Transpose> {
    let inst = inst?;
    let chromatic = inst.transpose_chromatic?;
    if chromatic == 0 && inst.transpose_diatonic.unwrap_or(0) == 0 {
        return None;
    }
    Some(Transpose {
        diatonic: inst.transpose_diatonic.map(i16::from),
        chromatic: chromatic as i16,
        octave_change: None,
    })
}

fn build_instrument_kind(inst: Option<&MscxInstrument>) -> InstrumentKind {
    let strings = inst.and_then(|i| i.string_data.as_ref());
    match strings {
        Some(data) if !data.strings.is_empty() => {
            let tuning: Vec<Pitch> = data
                .strings
                .iter()
                .map(|midi| midi_to_pitch(*midi as i8))
                .collect();
            InstrumentKind::Stringed {
                string_count: tuning.len() as u8,
                tuning,
                capo: 0,
            }
        }
        _ => {
            // Percussion detection: MuseScore uses `<instrumentId>` prefixes
            // like `drum.` or the `percussion` clef under the part staff.
            let is_percussion = inst
                .and_then(|i| i.instrument_id.as_deref())
                .is_some_and(|id| id.starts_with("drum."));
            if is_percussion {
                InstrumentKind::Percussion
            } else {
                InstrumentKind::Pitched
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Staves
// ---------------------------------------------------------------------------

fn build_staves(parts: &[MscxPart]) -> (Vec<StaffDef>, Vec<Vec<StaffId>>) {
    let mut staves: Vec<StaffDef> = Vec::new();
    let mut per_part: Vec<Vec<StaffId>> = Vec::with_capacity(parts.len());

    for part in parts {
        let mut ids: Vec<StaffId> = Vec::new();
        if part.staves.is_empty() {
            let id = StaffId(staves.len() as u8);
            staves.push(default_staff());
            ids.push(id);
        } else {
            for staff in &part.staves {
                let id = StaffId(staves.len() as u8);
                staves.push(convert_part_staff(staff));
                ids.push(id);
            }
        }
        per_part.push(ids);
    }

    (staves, per_part)
}

fn convert_part_staff(staff: &MscxPartStaff) -> StaffDef {
    let is_tab = staff.group.as_deref() == Some("tablature");
    let clef = clef_from_default(staff.default_clef.as_deref(), is_tab);
    let display = if is_tab {
        StaffDisplay::Tab
    } else {
        StaffDisplay::Notation
    };
    StaffDef { clef, display }
}

fn default_staff() -> StaffDef {
    StaffDef {
        clef: Clef::Treble,
        display: StaffDisplay::Notation,
    }
}

fn clef_from_default(clef: Option<&str>, is_tab: bool) -> Clef {
    if is_tab {
        return Clef::Tab;
    }
    match clef.unwrap_or("G") {
        "G" | "G8vb" | "G8va" => Clef::Treble,
        "F" | "F8va" | "F8vb" | "F15ma" | "F15mb" => Clef::Bass,
        "C1" | "C2" | "C3" => Clef::Alto,
        "C4" | "C5" => Clef::Tenor,
        "PERC" | "PERC2" => Clef::Percussion,
        "TAB" | "TAB4" | "TAB6" | "TAB_SERIF" => Clef::Tab,
        _ => Clef::Treble,
    }
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

fn build_timeline(mscx: &Mscx, report: &mut LossReport) -> Vec<MeasureDef> {
    let Some(staff) = mscx.score_staves.first() else {
        return Vec::new();
    };
    let mut current_time = TimeSignature {
        numerator: 4,
        denominator: 4,
    };
    let mut result: Vec<MeasureDef> = Vec::with_capacity(staff.measures.len());
    let mut first = true;

    for (index, measure) in staff.measures.iter().enumerate() {
        let time_sig = measure.time_sig.map(convert_time_sig);
        let key_sig = measure.key_sig.map(convert_key_sig);
        let tempo = measure.tempo_bps.map(bps_to_bpm);

        if let Some(sig) = time_sig {
            current_time = sig;
        }

        let mut navigation: Vec<NavigationEvent> = Vec::new();
        let idx = MeasureIndex(index as u16);
        if measure.start_repeat {
            navigation.push(NavigationEvent {
                measure_index: idx,
                kind: JumpKind::RepeatOpen,
                repeat_count: None,
                volta: None,
                volta_last: false,
            });
        }
        if let Some(count) = measure.end_repeat {
            navigation.push(NavigationEvent {
                measure_index: idx,
                kind: JumpKind::RepeatClose,
                repeat_count: Some(count),
                volta: None,
                volta_last: false,
            });
        }

        // Ensure the first measure always announces time/key/tempo so
        // downstream consumers have a starting signature.
        let (announce_time, announce_key, announce_tempo) = if first {
            let ts = time_sig.unwrap_or(current_time);
            let ks = key_sig.unwrap_or_else(default_key_signature);
            let tp = tempo.unwrap_or(DEFAULT_TEMPO);
            first = false;
            (Some(ts), Some(ks), Some(tp))
        } else {
            (time_sig, key_sig, tempo)
        };

        if measure.len.is_some() {
            report.note("Measure/@len");
        }

        result.push(MeasureDef {
            index: idx,
            tempo: announce_tempo,
            time_signature: announce_time,
            key_signature: announce_key,
            marker: None,
            navigation,
            tick_resolution: DIVISIONS as u16,
            duration_ticks: measure_duration_ticks(current_time),
            barline_left: None,
            barline_right: None,
            gp_beams: None,
            gp_fermatas: Vec::new(),
            gp_free_time: false,
        });
    }

    result
}

fn measure_duration_ticks(time: TimeSignature) -> u32 {
    // 1 quarter = DIVISIONS ticks. duration = numerator × (DIVISIONS × 4 / denominator).
    let per_beat = (DIVISIONS * 4) / time.denominator.max(1) as u32;
    per_beat * time.numerator as u32
}

// ---------------------------------------------------------------------------
// Tracks (per-part measure data)
// ---------------------------------------------------------------------------

fn build_tracks(
    mscx: &Mscx,
    part_staff_ids: &[Vec<StaffId>],
    _timeline_len: usize,
    report: &mut LossReport,
) -> Vec<Track> {
    let staff_count = mscx.score_staves.len();
    if staff_count == 0 || mscx.parts.is_empty() {
        return Vec::new();
    }

    // In MuseScore, `Part` and score-level `Staff` are aligned: the *first*
    // staff of a multi-staff part is the first `<Staff>` under `<Score>` and
    // additional staves follow in order. We map track N to score-staff N.
    let track_count = mscx.parts.len().min(staff_count);

    (0..track_count)
        .map(|part_idx| {
            let part = &mscx.parts[part_idx];
            let staff_ids = part_staff_ids.get(part_idx).cloned().unwrap_or_default();
            let score_staff = &mscx.score_staves[part_idx];
            let name = part
                .track_name
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| part.instrument.as_ref().and_then(|i| i.long_name.clone()))
                .unwrap_or_else(|| format!("Track {}", part_idx + 1));

            Track {
                id: TrackId(part_idx as u8),
                name,
                instrument: InstrumentId(part_idx as u8),
                staves: staff_ids,
                measures: build_measures_for_track(part_idx, score_staff, report),
            }
        })
        .collect()
}

fn build_measures_for_track(
    part_idx: usize,
    score_staff: &MscxStaff,
    report: &mut LossReport,
) -> BTreeMap<MeasureIndex, MeasureData> {
    let mut measures = BTreeMap::new();
    for (index, measure) in score_staff.measures.iter().enumerate() {
        let data = build_measure_data(part_idx, index, measure, report);
        measures.insert(MeasureIndex(index as u16), data);
    }
    measures
}

fn build_measure_data(
    part_idx: usize,
    measure_idx: usize,
    measure: &MscxMeasure,
    report: &mut LossReport,
) -> MeasureData {
    let mut voices: HashMap<u8, Voice> = HashMap::new();

    for (voice_idx, voice) in measure.voices.iter().enumerate() {
        let voice_id = voice_idx as u8;
        let converted = convert_voice(voice, voice_id, report);
        voices.insert(voice_id, converted);
    }

    MeasureData {
        measure_index: MeasureIndex(measure_idx as u16),
        track_id: TrackId(part_idx as u8),
        repeat: None,
        voices,
        gp_line_break: 0,
        gp_simile_mark: None,
    }
}

fn convert_voice(voice: &MscxVoice, voice_id: u8, report: &mut LossReport) -> Voice {
    let mut beats: Vec<Beat> = Vec::with_capacity(voice.beats.len());
    let mut tick_offset: u32 = 0;

    for beat in &voice.beats {
        let duration_ticks = duration_ticks(beat.duration);
        let optimized_beat = convert_beat(beat, tick_offset, report);
        beats.push(optimized_beat);
        tick_offset = tick_offset.saturating_add(duration_ticks);
    }

    Voice { voice_id, beats }
}

fn convert_beat(beat: &MscxBeat, tick_offset: u32, report: &mut LossReport) -> Beat {
    let duration = convert_duration(beat.duration);
    let (notes, gp_rest) = match &beat.kind {
        MscxBeatKind::Rest => (Vec::new(), true),
        MscxBeatKind::Chord(mscx_notes) => (convert_notes(mscx_notes, report), false),
    };

    Beat {
        tick_offset,
        duration,
        notes,
        events: Vec::new(),
        dynamic: None,
        slur: None,
        lyric: None,
        beam_group: None,
        tuplet: None,
        beams: Vec::new(),
        grace_notes: Vec::new(),
        cue: false,
        chord: None,
        gp_empty: false,
        gp_rest,
        gp_vibrato: false,
        gp_fade_in: false,
        gp_stroke: None,
        gp_pick_stroke: None,
        gp_beat_flags2: None,
        gp_break_secondary: None,
        gp_slap_effect: None,
        gp_rasgueado: false,
        gp_text: String::new(),
        gp_mix_table: None,
        gp_tremolo_bar: None,
        gp_chord: None,
    }
}

fn convert_duration(duration: MscxDuration) -> Duration {
    Duration {
        base: convert_duration_kind(duration.kind),
        dots: duration.dots,
        tuplet: None,
    }
}

fn convert_duration_kind(kind: MscxDurationKind) -> NoteValue {
    match kind {
        MscxDurationKind::Whole => NoteValue::Whole,
        MscxDurationKind::Half => NoteValue::Half,
        MscxDurationKind::Quarter => NoteValue::Quarter,
        MscxDurationKind::Eighth => NoteValue::Eighth,
        MscxDurationKind::Sixteenth => NoteValue::Sixteenth,
        MscxDurationKind::ThirtySecond => NoteValue::ThirtySecond,
        MscxDurationKind::SixtyFourth => NoteValue::SixtyFourth,
        MscxDurationKind::HundredTwentyEighth => NoteValue::HundredTwentyEighth,
        // Measure = whole-measure rest — treat as whole for now.
        MscxDurationKind::Measure => NoteValue::Whole,
    }
}

fn duration_ticks(duration: MscxDuration) -> u32 {
    let base = match duration.kind {
        MscxDurationKind::Whole => DIVISIONS * 4,
        MscxDurationKind::Half => DIVISIONS * 2,
        MscxDurationKind::Quarter => DIVISIONS,
        MscxDurationKind::Eighth => DIVISIONS / 2,
        MscxDurationKind::Sixteenth => DIVISIONS / 4,
        MscxDurationKind::ThirtySecond => DIVISIONS / 8,
        MscxDurationKind::SixtyFourth => DIVISIONS / 16,
        MscxDurationKind::HundredTwentyEighth => DIVISIONS / 32,
        MscxDurationKind::Measure => DIVISIONS * 4,
    };
    apply_dots(base, duration.dots)
}

fn apply_dots(base: u32, dots: u8) -> u32 {
    // Dotted note lengthens by half, then a quarter, etc. Cap at three dots.
    let mut total = base;
    let mut increment = base / 2;
    let cap = dots.min(3);
    for _ in 0..cap {
        total = total.saturating_add(increment);
        increment /= 2;
        if increment == 0 {
            break;
        }
    }
    total
}

fn convert_notes(notes: &[MscxNote], report: &mut LossReport) -> Vec<Note> {
    notes
        .iter()
        .map(|note| convert_note(note, report))
        .collect()
}

fn convert_note(note: &MscxNote, _report: &mut LossReport) -> Note {
    let pitch = note.pitch.map(|midi| midi_to_pitch(midi as i8));
    let tie = if note.tie_end {
        Some(TieType::End)
    } else if note.tie_start {
        Some(TieType::Start)
    } else {
        None
    };
    Note {
        pitch,
        // MuseScore string is 0-based (top = 0); the optimized model is
        // 1-based. Convert here.
        string: note.string.map(|s| s.saturating_add(1)),
        fret: note.fret,
        tie,
        techniques: Vec::new(),
        ornaments: Vec::new(),
        articulations: Vec::new(),
        left_finger: None,
        right_finger: None,
        notehead: None,
        stem: None,
        accidental: None,
        arpeggiate: None,
        display_pitch: None,
        gp_harmonic: None,
        gp_grace: None,
        gp_bend: None,
        gp_trill: None,
        gp_ghost: false,
        gp_duration_percent: 1.0,
        gp_swap_accidentals: false,
        gp_velocity: None,
        gp_note_type_raw: None,
        gp_is_rest: false,
        gp_ornament: None,
        gp_note_duration: None,
        gp_note_tuplet: None,
    }
}
