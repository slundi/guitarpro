//! MSCX → [`LoadedScore`] conversion.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::convert::loss::LossReport;
use crate::convert::mscz::notation::{
    ChordMark, articulation_from_breath, articulation_from_chord_line, barline_style,
    chord_marks_from_symbol, doubled_direction, dynamic_from_mscx, dynamics_voice_only,
    hairpin_from_subtype, jump_kind, tremolo_from_subtype,
};
use crate::convert::mscz::spelling::{pitch_class_from_tpc, pitch_from_midi_tpc, tpc_spells_midi};
use crate::convert::optimized::legacy::midi_to_pitch;
use crate::io::mscz::parse::parse_fraction;
use crate::model::legacy::enums::DirectionSign;
use crate::model::mscz::{
    Instrument as MscxInstrument, MetaTag, Mscx, MscxBeat, MscxBeatKind, MscxChordMark,
    MscxDuration, MscxDurationKind, MscxGrace, MscxHarmony, MscxKeySig, MscxLocation, MscxMeasure,
    MscxNote, MscxSegment, MscxStaff, MscxTimeSig, MscxVoice, Part as MscxPart,
    Staff as MscxPartStaff,
};
use crate::model::optimized::{
    LoadedScore,
    beat::{Beat, Duration, GraceNote, Slur, SlurKind, Voice},
    effect::{BeatEvent, HairpinEvent, HairpinKind},
    global::{
        Instrument, InstrumentId, InstrumentKind, MeasureIndex, PartGroup, SCORE_FORMAT_VERSION,
        Score, StaffId, TrackId, Transpose,
    },
    metadata::{
        ChordSymbol, Creator, Identification, KeySignature, Metadata, Mode, TimeSignature, Work,
    },
    note::{Articulation, Note, NoteValue, Notehead, Ornament, Pitch, TieType},
    timeline::{Barline, BarlineStyle, JumpKind, MeasureDef, NavigationEvent, set_ending},
    track::{Clef, MeasureData, StaffDef, StaffDisplay, Track},
};

/// Ticks per quarter note used by the produced [`LoadedScore`].
///
/// Matches [`crate::convert::musicxml::DIVISIONS`] so tick arithmetic aligns
/// with the shared timeline expected by other converters.
pub const DIVISIONS: u32 = crate::model::legacy::key_signature::DURATION_QUARTER_TIME as u32;

/// Ticks per whole note: what MSCX fractions are fractions of.
pub(crate) const WHOLE_TICKS: u32 = DIVISIONS * 4;

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

    let mut metadata = build_metadata(mscx);
    let instruments = build_instruments(&mscx.parts);
    let (staves, part_staff_ids) = build_staves(&mscx.parts, &mut report);
    let (timeline, directions) = build_timeline(mscx, &mut report);
    keep_direction_signs(&mut metadata, directions);
    let tracks = build_tracks(mscx, &staves, &part_staff_ids, &timeline, &mut report);

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
            format_version: SCORE_FORMAT_VERSION,
        },
        layout: None,
    };

    ConvertOutcome { score, report }
}

/// Keeps the GP direction signs the timeline has no jump for in
/// `gp.direction.N`, as the GP import does: a GP export writes them, and
/// the repeats analysis sees the jumps playback skips.
fn keep_direction_signs(metadata: &mut Metadata, directions: Vec<(usize, Vec<DirectionSign>)>) {
    if directions.is_empty() {
        return;
    }
    let identification = metadata
        .identification
        .get_or_insert_with(|| Identification {
            creators: Vec::new(),
            rights: None,
            encoding_software: None,
            encoding_date: None,
            source: None,
            miscellaneous: Vec::new(),
        });
    for (index, signs) in directions {
        identification
            .miscellaneous
            .push(crate::model::directions::direction_entry(
                index,
                signs.iter().map(|sign| sign.name()),
            ));
    }
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
    KeySignature::from_fifths(0, Mode::Major)
}

fn convert_time_sig(sig: MscxTimeSig) -> TimeSignature {
    TimeSignature {
        numerator: sig.numerator.max(1),
        denominator: sig.denominator.max(1),
    }
}

/// The key by its signature and `<mode>` (major without one).
fn convert_key_sig(sig: MscxKeySig) -> KeySignature {
    KeySignature::from_fifths(sig.fifths, sig.mode.unwrap_or(Mode::Major))
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
    let kind = build_instrument_kind(part);

    Instrument {
        name,
        abbreviation,
        instrument_sound,
        midi_channel: 0,
        midi_program: inst.and_then(|i| i.program).unwrap_or(0),
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
    // Either one alone still transposes (a missing one counts as 0).
    let chromatic = inst.transpose_chromatic.unwrap_or(0);
    if chromatic == 0 && inst.transpose_diatonic.unwrap_or(0) == 0 {
        return None;
    }
    Some(Transpose {
        diatonic: inst.transpose_diatonic.map(i16::from),
        chromatic: i16::from(chromatic),
        octave_change: None,
    })
}

fn build_instrument_kind(part: &MscxPart) -> InstrumentKind {
    let inst = part.instrument.as_ref();
    let strings = inst.and_then(|i| i.string_data.as_ref());
    match strings {
        Some(data) if !data.strings.is_empty() => {
            let tuning: Vec<Pitch> = data
                .strings
                .iter()
                .map(|&midi| midi_to_pitch(midi.min(127) as i8))
                .collect();
            InstrumentKind::Stringed {
                string_count: tuning.len() as u8,
                tuning,
                capo: 0,
            }
        }
        _ => {
            // Percussion: a percussion staff or clef, or a `drum.` sound
            // other than the pitched timpani.
            let is_perc_clef = |clef: &str| clef.starts_with("PERC");
            let is_percussion = part.staves.iter().any(|staff| {
                staff.group.as_deref() == Some("percussion")
                    || staff.default_clef.as_deref().is_some_and(is_perc_clef)
            }) || inst.is_some_and(|i| {
                i.clefs.iter().any(|(_, clef)| is_perc_clef(clef))
                    || i.instrument_id
                        .as_deref()
                        .is_some_and(|id| id.starts_with("drum.") && id != "drum.timpani")
            });
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

fn build_staves(parts: &[MscxPart], report: &mut LossReport) -> (Vec<StaffDef>, Vec<Vec<StaffId>>) {
    let mut staves: Vec<StaffDef> = Vec::new();
    let mut per_part: Vec<Vec<StaffId>> = Vec::with_capacity(parts.len());

    for part in parts {
        let mut ids: Vec<StaffId> = Vec::new();
        if part.staves.is_empty() {
            let id = StaffId(staves.len() as u8);
            staves.push(default_staff());
            ids.push(id);
        } else {
            for (index, staff) in part.staves.iter().enumerate() {
                let id = StaffId(staves.len() as u8);
                // MuseScore takes the instrument's clef for a staff
                // without a `<defaultClef>` of its own.
                let instrument_clef = part.instrument.as_ref().and_then(|instrument| {
                    instrument
                        .clefs
                        .iter()
                        .find(|(staff, _)| *staff == index)
                        .map(|(_, clef)| clef.as_str())
                });
                staves.push(convert_part_staff(staff, instrument_clef, report));
                ids.push(id);
            }
        }
        per_part.push(ids);
    }

    (staves, per_part)
}

fn convert_part_staff(
    staff: &MscxPartStaff,
    instrument_clef: Option<&str>,
    report: &mut LossReport,
) -> StaffDef {
    let is_tab = staff.group.as_deref() == Some("tablature");
    let clef = clef_from_default(
        staff.default_clef.as_deref().or(instrument_clef),
        is_tab,
        report,
    );
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

fn clef_from_default(clef: Option<&str>, is_tab: bool, report: &mut LossReport) -> Clef {
    if is_tab {
        return Clef::Tab;
    }
    let name = clef.unwrap_or("G");
    report_clef(name, report);
    clef_from_name(name).unwrap_or(Clef::Treble)
}

/// Notes a clef the model has no type for, approximated or unknown. Done
/// once per clef in effect, not per element naming it.
fn report_clef(name: &str, report: &mut LossReport) {
    match read_clef(name) {
        None => report.note(format!("Clef ({name}, unknown)")),
        Some((_, false)) => report.note(format!("Clef ({name}, approximated)")),
        Some((_, true)) => {}
    }
}

/// A MuseScore clef type name (`<defaultClef>`, `<concertClefType>`).
fn clef_from_name(name: &str) -> Option<Clef> {
    read_clef(name).map(|(clef, _)| clef)
}

/// A MuseScore clef type name as the model's clef, and whether that's
/// exact: one the model has no type for is read as the nearest.
fn read_clef(name: &str) -> Option<(Clef, bool)> {
    let exact = match name {
        "G8vb" | "G8vbo" | "G8vbp" => Clef::Treble8vb,
        "G8va" => Clef::Treble8va,
        "F8vb" => Clef::Bass8vb,
        "F8va" => Clef::Bass8va,
        "G" => Clef::Treble,
        "F" => Clef::Bass,
        "C3" => Clef::Alto,
        "C4" => Clef::Tenor,
        "PERC" | "PERC2" => Clef::Percussion,
        "TAB" | "TAB4" | "TAB6" | "TAB_SERIF" => Clef::Tab,
        _ => {
            let nearest = match name {
                "G15ma" => Clef::Treble8va,
                "G1" => Clef::Treble,
                "G15mb" => Clef::Treble8vb,
                "F15ma" => Clef::Bass8va,
                "F15mb" => Clef::Bass8vb,
                "F_B" | "F_C" => Clef::Bass,
                "C1" | "C2" => Clef::Alto,
                "C5" => Clef::Tenor,
                _ => return None,
            };
            return Some((nearest, false));
        }
    };
    Some((exact, true))
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

/// The global timeline, and per measure (by index) the GP direction signs of
/// its second segno and coda marks, which it has no jump for.
fn build_timeline(
    mscx: &Mscx,
    report: &mut LossReport,
) -> (Vec<MeasureDef>, Vec<(usize, Vec<DirectionSign>)>) {
    let mut directions = Vec::new();
    let Some(staff) = mscx.score_staves.first() else {
        return (Vec::new(), directions);
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
            });
        }
        if measure.end_repeat_lossy {
            report.note("Repeat count (out of range)");
        }
        if let Some(count) = measure.end_repeat {
            navigation.push(NavigationEvent {
                measure_index: idx,
                kind: JumpKind::RepeatClose,
                repeat_count: Some(count),
            });
        }
        // Jumps and markers are system-wide, kept in the first staff. As
        // the GP import orders them: signs before the repeat, jumps after.
        let mut jumps = Vec::new();
        let mut signs = Vec::new();
        for mark in &measure.marks {
            match jump_kind(mark) {
                Some(kind) => jumps.push(kind),
                None => match doubled_direction(mark) {
                    // Kept for a GP export, not for MuseScore's.
                    Some(sign) => {
                        report.note("Jump/Marker (doppio segno or doppia coda)");
                        signs.push(sign);
                    }
                    None => report.note("Jump/Marker (unknown)"),
                },
            }
        }
        if !signs.is_empty() {
            directions.push((index, signs));
        }
        crate::model::directions::add_direction_jumps(&mut navigation, idx, jumps);

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

        result.push(MeasureDef {
            index: idx,
            tempo: announce_tempo,
            time_signature: announce_time,
            key_signature: announce_key,
            marker: None,
            navigation,
            tick_resolution: DIVISIONS as u16,
            duration_ticks: measure
                .len
                .as_deref()
                .and_then(|len| {
                    let ticks = len_ticks(len);
                    if ticks.is_none() {
                        report.note("Measure (length out of range)");
                    }
                    ticks
                })
                .unwrap_or_else(|| measure_duration_ticks(current_time)),
            barline_left: None,
            barline_right: measure_barline(measure).map(|style| Barline {
                style,
                ending: None,
            }),
            gp_beams: None,
            gp_fermatas: Vec::new(),
            gp_free_time: false,
        });
    }

    add_voltas(staff, &mut result, report);
    (result, directions)
}

/// A measure's end barline on one staff: the first voice's, or another
/// voice's where that one has none (an explicit regular one counts).
/// `None` for a regular one.
fn measure_barline(measure: &MscxMeasure) -> Option<BarlineStyle> {
    measure
        .voices
        .iter()
        .find_map(|voice| voice.end_barline.as_ref())
        .map(barline_style)
        .filter(|&style| style != BarlineStyle::Regular)
}

/// Voltas (system-wide, kept in the first staff) as endings on the
/// timeline's barlines: a `Start` on the first measure's left barline,
/// a `Stop` (closed bracket) or `Discontinue` (open) on the last one's
/// right barline. An end pairs with the start its `<prev>` location
/// points at (MuseScore can write the next volta's start before it), else
/// the last one opened (reported, when it pointed elsewhere).
fn add_voltas(staff: &MscxStaff, timeline: &mut [MeasureDef], report: &mut LossReport) {
    let mut open: Vec<(usize, &crate::model::mscz::MscxVolta)> = Vec::new();
    for (m, measure) in staff.measures.iter().enumerate() {
        for voice in &measure.voices {
            for mark in &voice.voltas {
                if let Some(volta) = &mark.start {
                    open.push((m, volta));
                    continue;
                }
                // Its offset counts from where it stands: after a measure's
                // beats, at the next measure's start, unless its fractions
                // count back from this one's end (as this crate writes it).
                let back = mark.fractions.is_some_and(|(numerator, _)| numerator < 0);
                let at = if mark.beats_before > 0 && !back {
                    m + 1
                } else {
                    m
                };
                let pointed = mark
                    .measures
                    .and_then(|offset| usize::try_from(at as i64 + offset as i64).ok())
                    .and_then(|start| open.iter().rposition(|&(at, _)| at == start));
                // One pointing where no volta starts is paired by guess.
                if mark.measures.is_some() && pointed.is_none() && !open.is_empty() {
                    report.note("Volta (end not at its start)");
                }
                let Some((start, volta)) = pointed
                    .or(open.len().checked_sub(1))
                    .map(|index| open.remove(index))
                else {
                    report.note("Volta (unpaired end)");
                    continue;
                };
                // An end element before any beat closes the volta at the
                // previous measure's end.
                let end = if mark.beats_before > 0 {
                    m
                } else {
                    m.saturating_sub(1)
                }
                .max(start);
                let closed = volta.end_hook_type.is_some_and(|hook| hook != 0);
                // An ending for no pass (its `<endings>` empty or not
                // numbers) isn't one: nothing plays or writes it.
                if volta.endings.is_empty() {
                    report.note("Volta (without ending numbers)");
                    continue;
                }
                set_ending(
                    timeline,
                    start,
                    end,
                    &volta.endings,
                    volta.begin_text.clone(),
                    closed,
                );
            }
        }
    }
    for _ in open {
        report.note("Volta (unterminated)");
    }
}

/// The longest measure read: as long as the longest time signature
/// (255/1). A whole-measure rest is written out as rests filling its
/// measure, so a longer one (from a malformed `len`) could make any
/// number of them.
const MAX_MEASURE_TICKS: u32 = 255 * WHOLE_TICKS;

/// A `<Measure len="n/d">`'s length: a pickup's or irregular measure's;
/// `None` for one not positive or past [`MAX_MEASURE_TICKS`].
fn len_ticks(len: &str) -> Option<u32> {
    parse_fraction(len)
        .map(fraction_ticks)
        .and_then(|ticks| u32::try_from(ticks).ok())
        .filter(|&ticks| ticks > 0 && ticks <= MAX_MEASURE_TICKS)
}

/// A measure's length in [`DIVISIONS`] a quarter
/// ([`TimeSignature::measure_ticks`]); a zero denominator's is read as 1.
pub(crate) fn measure_duration_ticks(time: TimeSignature) -> u32 {
    let time = TimeSignature {
        denominator: time.denominator.max(1),
        ..time
    };
    // At most 255 wholes: fits.
    time.measure_ticks(u64::from(DIVISIONS))
        .and_then(|ticks| u32::try_from(ticks).ok())
        .unwrap_or(u32::MAX)
}

// ---------------------------------------------------------------------------
// Tracks (per-part measure data)
// ---------------------------------------------------------------------------

fn build_tracks(
    mscx: &Mscx,
    staves: &[StaffDef],
    part_staff_ids: &[Vec<StaffId>],
    timeline: &[MeasureDef],
    report: &mut LossReport,
) -> Vec<Track> {
    let staff_count = mscx.score_staves.len();
    if staff_count == 0 || mscx.parts.is_empty() {
        return Vec::new();
    }

    // A part's music is the score-level `<Staff>` with its first staff's
    // id. A part without a `<Staff>`, or with none the score's match, takes
    // the next score staff no part names. A track holds one staff of music; a multi-staff part's further
    // staves are not imported.
    let by_id = |part: &MscxPart| {
        let id = &part.staves.first()?.id;
        mscx.score_staves
            .iter()
            .position(|staff| &staff.staff_id == id)
    };
    // Every score staff some part names (a piano's left hand too).
    let named: Vec<usize> = mscx
        .parts
        .iter()
        .flat_map(|part| &part.staves)
        .filter_map(|staff| {
            mscx.score_staves
                .iter()
                .position(|score_staff| score_staff.staff_id == staff.id)
        })
        .collect();
    let mut unnamed = (0..staff_count).filter(|index| !named.contains(index));
    let score_staff_of: Vec<Option<usize>> = mscx
        .parts
        .iter()
        .map(|part| by_id(part).or_else(|| unnamed.next()))
        .collect();
    // Every part is a track, so track ids stay their part's index; one
    // without a score staff of music is reported and left empty.
    (0..mscx.parts.len())
        .map(|part_idx| {
            let part = &mscx.parts[part_idx];
            let staff_ids = part_staff_ids.get(part_idx).cloned().unwrap_or_default();
            let score_staff = score_staff_of
                .get(part_idx)
                .copied()
                .flatten()
                .and_then(|index| mscx.score_staves.get(index));
            report.note_n(
                "Staff (additional staff of a part)",
                part.staves.len().saturating_sub(1),
            );
            let name = part
                .track_name
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| part.instrument.as_ref().and_then(|i| i.long_name.clone()))
                .unwrap_or_else(|| format!("Track {}", part_idx + 1));
            let clef = staff_ids
                .first()
                .and_then(|id| staves.get(id.0 as usize))
                .map_or(Clef::Treble, |staff| staff.clef);

            let measures = match score_staff {
                Some(score_staff) => {
                    build_measures_for_track(part_idx, score_staff, clef, timeline, report)
                }
                None => {
                    report.note("Part (without a staff of music)");
                    BTreeMap::new()
                }
            };
            Track {
                id: TrackId(part_idx as u8),
                name,
                instrument: InstrumentId(part_idx as u8),
                staves: staff_ids,
                measures,
            }
        })
        .collect()
}

fn is_measure_rest(beat: &MscxBeat) -> bool {
    beat.duration.kind == MscxDurationKind::Measure
}

/// A beat's length: a whole-measure rest's is its measure's.
fn beat_ticks(beat: &MscxBeat, measure_ticks: Option<u32>) -> u32 {
    match measure_ticks {
        Some(ticks) if is_measure_rest(beat) => ticks,
        _ => duration_ticks(beat.duration),
    }
}

/// Rests filling `ticks`, longest first (dotted where that fits), and
/// what no note value down to a 128th fills (as of a measure a third of a
/// whole long, which needs a tuplet).
fn rest_durations(mut ticks: u32) -> (Vec<MscxDuration>, u32) {
    const KINDS: [MscxDurationKind; 8] = [
        MscxDurationKind::Whole,
        MscxDurationKind::Half,
        MscxDurationKind::Quarter,
        MscxDurationKind::Eighth,
        MscxDurationKind::Sixteenth,
        MscxDurationKind::ThirtySecond,
        MscxDurationKind::SixtyFourth,
        MscxDurationKind::HundredTwentyEighth,
    ];
    let mut durations = Vec::new();
    for kind in KINDS {
        for dots in [1, 0] {
            let duration = MscxDuration { kind, dots };
            let length = duration_ticks(duration);
            while length > 0 && ticks >= length {
                durations.push(duration);
                ticks -= length;
            }
        }
    }
    (durations, ticks)
}

fn build_measures_for_track(
    part_idx: usize,
    score_staff: &MscxStaff,
    staff_clef: Clef,
    timeline: &[MeasureDef],
    report: &mut LossReport,
) -> BTreeMap<MeasureIndex, MeasureData> {
    let mut measures = BTreeMap::new();
    let slurs = plan_slurs(score_staff, timeline, report);
    let mut current = staff_clef;
    // The name of the clef in effect, as an unknown one reads as no clef.
    let mut current_name: Option<&str> = None;
    for (index, measure) in score_staff.measures.iter().enumerate() {
        // A clef at a measure's start, else the last one in the measure
        // before: at its end (where MuseScore writes changes), or in its
        // middle, which can only hold from here on. Only a clef other than
        // the one in effect is a change.
        let previous = index
            .checked_sub(1)
            .and_then(|i| score_staff.measures.get(i));
        let name = measure.start_clef.as_deref().or(previous.and_then(|m| {
            m.end_clef
                .as_deref()
                .or(m.mid_clefs.last().map(String::as_str))
        }));
        let clef = name
            .and_then(clef_from_name)
            .filter(|&clef| clef != current);
        // Only a change is reported: measure 0's clef restates the staff's
        // own (reported with it), and an unknown one is no clef at all.
        let unknown_change = index > 0
            && name != current_name
            && name.is_some_and(|name| clef_from_name(name).is_none());
        if let Some(name) = name
            && (clef.is_some() || unknown_change)
        {
            report_clef(name, report);
        }
        current_name = name.or(current_name);
        current = clef.unwrap_or(current);
        if index + 1 == score_staff.measures.len() && measure.end_clef.is_some() {
            // Nothing follows for the change to take effect in.
            report.note("Clef (end of score)");
        }
        let measure_ticks = timeline.get(index).map(|def| def.duration_ticks);
        let mut data = build_measure_data(
            (part_idx, index),
            measure,
            measure_ticks,
            clef,
            &slurs,
            report,
        );
        // Barline styles are per staff; the timeline has the first staff's.
        let style = measure_barline(measure).unwrap_or(BarlineStyle::Regular);
        let system = timeline
            .get(index)
            .and_then(|def| def.barline_right.as_ref())
            .map_or(BarlineStyle::Regular, |barline| barline.style);
        if style != system {
            data.barline_right = Some(Barline {
                style,
                ending: None,
            });
        }
        measures.insert(MeasureIndex(index as u16), data);
    }
    attach_leading_marks(score_staff, &mut measures, report);
    report_voice_assignments(score_staff, report);
    number_hairpins(&mut measures, report);
    measures
}

/// Dynamics whose voice assignment the model can't hold. It gives each
/// voice with dynamics a level of its own, and the writer writes them for
/// all voices when only one voice has any, else each for its own voice: a
/// dynamic assigned otherwise is reported.
fn report_voice_assignments(staff: &MscxStaff, report: &mut LossReport) {
    let dynamics: Vec<(usize, bool)> = staff
        .measures
        .iter()
        .flat_map(|measure| measure.voices.iter().enumerate())
        .flat_map(|(voice, v)| {
            v.beats
                .iter()
                .filter_map(move |beat| beat.segment.dynamic.as_ref())
                .map(move |dynamic| (voice, dynamic.voice_only))
        })
        .collect();
    let several = dynamics_voice_only(dynamics.iter().map(|&(voice, _)| voice as u8));
    for &(_, voice_only) in &dynamics {
        if voice_only != several {
            report.note("Dynamic (voice assignment)");
        }
    }
}

/// Breath marks and hairpin ends written before a voice's first beat
/// follow the previous beat in that voice: the last of an earlier measure.
fn attach_leading_marks(
    staff: &MscxStaff,
    measures: &mut BTreeMap<MeasureIndex, MeasureData>,
    report: &mut LossReport,
) {
    // Per voice: the last measure so far with beats in it.
    let mut last_with_beats: HashMap<u8, MeasureIndex> = HashMap::new();
    for (m, measure) in staff.measures.iter().enumerate() {
        let index = MeasureIndex(m as u16);
        for (v, voice) in measure.voices.iter().enumerate() {
            // Those after grace chords before the first main chord too.
            let leading_graces = voice.beats.iter().take_while(|beat| {
                beat.grace.is_some() && matches!(beat.kind, MscxBeatKind::Chord(_))
            });
            let mut breaths = voice.leading_breaths.clone();
            let mut hairpin_ends = voice.leading_hairpin_ends;
            for grace in leading_graces {
                breaths.extend(grace.breaths.iter().cloned());
                hairpin_ends += grace.hairpin_ends;
            }
            if !breaths.is_empty() || hairpin_ends != 0 {
                match last_with_beats
                    .get(&(v as u8))
                    .and_then(|previous| measures.get_mut(previous))
                    .and_then(|data| data.voices.get_mut(&(v as u8)))
                    .and_then(|voice| voice.beats.last_mut())
                {
                    Some(beat) => {
                        add_breaths(beat, &breaths, report);
                        add_hairpin_ends(beat, hairpin_ends);
                    }
                    None => report.note("Breath/HairPin (before the first beat)"),
                }
            }
        }
        if let Some(data) = measures.get(&index) {
            for (&voice_id, voice) in &data.voices {
                if !voice.beats.is_empty() {
                    last_with_beats.insert(voice_id, index);
                }
            }
        }
    }
}

/// Gives each hairpin's start and stop events a shared id: a stop closes
/// its voice's oldest open hairpin (hairpins don't nest, and one can end
/// where the next starts), else the oldest open in another voice (as the
/// writer pairs them: MuseScore can end a hairpin on another voice's
/// chord). Beats are taken in (tick, voice) order. Unpaired stops are
/// dropped.
pub(crate) fn number_hairpins(
    measures: &mut BTreeMap<MeasureIndex, MeasureData>,
    report: &mut LossReport,
) {
    // Per voice: open hairpins' (opening order, id), oldest first.
    let mut open: HashMap<u8, VecDeque<(u64, u8)>> = HashMap::new();
    let mut opened: u64 = 0;
    let mut next_id: u8 = 1;
    // Whether an open hairpin has the id, by id.
    let mut in_use = [false; 256];
    for data in measures.values_mut() {
        for (voice, beat) in data.beats_in_time_order_mut() {
            beat.events.retain_mut(|event| {
                let BeatEvent::Hairpin(hairpin) = event else {
                    return true;
                };
                if hairpin.kind != HairpinKind::Stop {
                    // The next id (1–255, wrapping) no open hairpin has.
                    let id = (0..u8::MAX)
                        .map(|step| ((u16::from(next_id) - 1 + u16::from(step)) % 255 + 1) as u8)
                        .find(|&id| !in_use[usize::from(id)]);
                    let Some(id) = id else {
                        report.note("HairPin (too many open)");
                        return false;
                    };
                    hairpin.id = id;
                    in_use[usize::from(id)] = true;
                    open.entry(voice).or_default().push_back((opened, id));
                    opened += 1;
                    next_id = id % 255 + 1;
                    return true;
                }
                let from = Some(voice)
                    .filter(|voice| open.get(voice).is_some_and(|queue| !queue.is_empty()))
                    .or_else(|| {
                        open.iter()
                            .filter_map(|(&other, queue)| Some((queue.front()?.0, other)))
                            .min()
                            .map(|(_, other)| other)
                    });
                match from.and_then(|from| open.get_mut(&from)?.pop_front()) {
                    Some((_, id)) => {
                        hairpin.id = id;
                        in_use[usize::from(id)] = false;
                        true
                    }
                    None => {
                        report.note("HairPin (unpaired end)");
                        false
                    }
                }
            });
        }
    }
    for _ in open.values().flatten() {
        report.note("HairPin (unterminated)");
    }
}

fn build_measure_data(
    (part_idx, measure_idx): (usize, usize),
    measure: &MscxMeasure,
    measure_ticks: Option<u32>,
    clef: Option<Clef>,
    slurs: &HashMap<BeatKey, Slur>,
    report: &mut LossReport,
) -> MeasureData {
    let mut voices: HashMap<u8, Voice> = HashMap::new();

    // One superseded at the start holds for no beat: as lost as one in
    // mid-measure.
    for _ in measure.superseded_clefs.iter().chain(&measure.mid_clefs) {
        report.note("Clef (mid-measure)");
    }
    for (voice_idx, voice) in measure.voices.iter().enumerate() {
        let voice_id = voice_idx as u8;
        // An empty voice after the first only holds the place of the ones
        // after it (MuseScore numbers voices by place): not a voice.
        if voice_idx > 0 && *voice == MscxVoice::default() {
            continue;
        }
        let converted = convert_voice(
            voice,
            voice_id,
            measure_ticks,
            |beat_idx| slurs.get(&(measure_idx, voice_idx, beat_idx)).copied(),
            report,
        );
        voices.insert(voice_id, converted);
    }

    MeasureData {
        measure_index: MeasureIndex(measure_idx as u16),
        track_id: TrackId(part_idx as u8),
        repeat: None,
        voices,
        barline_right: None,
        gp_line_break: 0,
        gp_simile_mark: None,
        clef,
    }
}

// ---------------------------------------------------------------------------
// Slurs
// ---------------------------------------------------------------------------

/// A beat by (measure, voice position in the measure, beat index).
type BeatKey = (usize, usize, usize);

/// A beat's place on its staff: measure, tick in the measure, voice.
type StaffPoint = (usize, i64, usize);

/// When a chord sounds: measure, tick, then grace chords (by their index
/// in the voice) before the main chord at the same tick.
type Moment = (usize, i64, usize);

/// One end of a slur as read from its chord.
struct SlurSide {
    beat: BeatKey,
    at: StaffPoint,
    moment: Moment,
    /// Whether its chord is a grace chord, which the model holds no slurs
    /// on.
    grace: bool,
    /// Where its `<location>` says the other end is.
    other: Option<StaffPoint>,
}

/// Pairs the staff's slur starts and ends and gives each beat at most one
/// [`Slur`] mark, as the optimized model holds.
///
/// MuseScore pairs a slur's two ends by their relative `<location>`, so
/// nested slurs and slurs between voices pair up exactly; an end whose
/// location matches no start falls back to the innermost open slur of its
/// voice. Ids are unique among the slurs open at the same time and reused
/// once a slur is closed; a chord ending one slur and starting the next
/// hands the ending one's id on. Unpaired ends, and slurs a chord can't hold
/// (a second start or end on the same chord) or the model can't (one on a
/// grace chord, dropped whole), are reported.
fn plan_slurs(
    staff: &MscxStaff,
    timeline: &[MeasureDef],
    report: &mut LossReport,
) -> HashMap<BeatKey, Slur> {
    let mut starts: Vec<SlurSide> = Vec::new();
    let mut ends: Vec<SlurSide> = Vec::new();
    for (measure_idx, measure) in staff.measures.iter().enumerate() {
        let measure_ticks = timeline.get(measure_idx).map(|def| def.duration_ticks);
        for (voice_idx, voice) in measure.voices.iter().enumerate() {
            let mut tick = 0u32;
            for (beat_idx, beat) in voice.beats.iter().enumerate() {
                tick = tick.saturating_add(gap_ticks(beat));
                let at = (measure_idx, tick as i64, voice_idx);
                let grace = beat.grace.is_some();
                let moment = (at.0, at.1, if grace { beat_idx } else { usize::MAX });
                for slur in &beat.slurs {
                    let side = SlurSide {
                        beat: (measure_idx, voice_idx, beat_idx),
                        at,
                        moment,
                        grace,
                        other: locate(at, slur.location),
                    };
                    if slur.start {
                        starts.push(side);
                    } else {
                        ends.push(side);
                    }
                }
                // Grace chords take no time.
                if beat.grace.is_none() {
                    tick = tick.saturating_add(beat_ticks(beat, measure_ticks));
                }
            }
        }
    }

    // Pair each end with its start: by location, else the voice's
    // innermost open one. Ends go in time order; the starts before the
    // current end are stacked per voice, so pairing stays linear.
    let mut paired = vec![false; starts.len()];
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    let mut at_point: HashMap<StaffPoint, Vec<usize>> = HashMap::new();
    for (index, start) in starts.iter().enumerate() {
        at_point.entry(start.at).or_default().push(index);
    }
    let mut by_time: Vec<usize> = (0..starts.len()).collect();
    by_time.sort_by_key(|&index| starts[index].moment);
    let mut by_time = by_time.into_iter().peekable();
    let mut voice_stacks: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut by_end: Vec<usize> = (0..ends.len()).collect();
    by_end.sort_by_key(|&e| ends[e].moment);
    for e in by_end {
        let end = &ends[e];
        let moment = end.moment;
        while let Some(&index) = by_time.peek()
            && starts[index].moment < moment
        {
            voice_stacks
                .entry(starts[index].at.2)
                .or_default()
                .push(index);
            by_time.next();
        }
        let by_location = end.other.and_then(|start_at| {
            let candidates = at_point.get(&start_at)?;
            let open = || {
                candidates
                    .iter()
                    .copied()
                    .filter(|&s| !paired[s] && starts[s].moment < moment)
            };
            open()
                .find(|&s| starts[s].other == Some(end.at))
                .or_else(|| open().next())
        });
        let found = by_location.or_else(|| {
            let stack = voice_stacks.get_mut(&end.at.2)?;
            while let Some(&top) = stack.last() {
                if !paired[top] {
                    return Some(top);
                }
                stack.pop();
            }
            None
        });
        match found {
            Some(s) => {
                paired[s] = true;
                pairs.push((s, e));
            }
            None => report.note("Slur (unpaired end)"),
        }
    }
    for _ in paired.iter().filter(|&&p| !p) {
        report.note("Slur (unpaired start)");
    }
    // Paired first, so a slur from a grace note to its main note doesn't
    // leave the main chord's end to close another slur.
    pairs.retain(|&(s, e)| {
        let grace = starts[s].grace || ends[e].grace;
        if grace {
            report.note("Slur (on a grace chord)");
        }
        !grace
    });

    // One start and one end per chord.
    pairs.sort_by_key(|&(s, _)| (starts[s].at.0, starts[s].at.1, starts[s].at.2));
    let mut starting: HashSet<BeatKey> = HashSet::new();
    let mut ending: HashSet<BeatKey> = HashSet::new();
    pairs.retain(|&(s, e)| {
        let fits = !starting.contains(&starts[s].beat) && !ending.contains(&ends[e].beat);
        if fits {
            starting.insert(starts[s].beat);
            ending.insert(ends[e].beat);
        } else {
            report.note("Slur (several on one chord)");
        }
        fits
    });

    // Ids, sweeping the staff in time order: ends free theirs only after
    // the starts at the same moment are numbered. A chord ending one slur
    // and starting the next hands the ending slur's id on to the new one,
    // so its `EndStart` mark names both.
    let mut events: Vec<(usize, i64, bool, usize)> = Vec::new();
    for (i, &(s, e)) in pairs.iter().enumerate() {
        events.push((starts[s].at.0, starts[s].at.1, true, i));
        events.push((ends[e].at.0, ends[e].at.1, false, i));
    }
    events.sort();
    // One slur per end chord (kept above), so no key collides.
    let ending_on: HashMap<BeatKey, usize> = pairs
        .iter()
        .enumerate()
        .map(|(i, &(_, e))| (ends[e].beat, i))
        .collect();
    let mut ids: Vec<Option<u8>> = vec![None; pairs.len()];
    let mut handed_on = vec![false; pairs.len()];
    let mut open: Vec<u8> = Vec::new();
    for moment in events.chunk_by(|a, b| (a.0, a.1) == (b.0, b.1)) {
        for &(_, _, is_start, i) in moment {
            if !is_start {
                continue;
            }
            let inherited = ending_on
                .get(&starts[pairs[i].0].beat)
                .and_then(|&ending| Some((ending, ids[ending]?)));
            if let Some((ending, id)) = inherited {
                handed_on[ending] = true;
                ids[i] = Some(id);
                continue;
            }
            ids[i] = (0..=u8::MAX).find(|id| !open.contains(id));
            match ids[i] {
                Some(id) => open.push(id),
                None => report.note("Slur (too many open)"),
            }
        }
        for &(_, _, is_start, i) in moment {
            if !is_start
                && !handed_on[i]
                && let Some(id) = ids[i]
            {
                open.retain(|&open_id| open_id != id);
            }
        }
    }

    let mut marks: HashMap<BeatKey, Slur> = HashMap::new();
    for (i, &(s, e)) in pairs.iter().enumerate() {
        let Some(slur_id) = ids[i] else { continue };
        marks
            .entry(starts[s].beat)
            .and_modify(|mark| {
                *mark = Slur {
                    slur_id,
                    kind: SlurKind::EndStart,
                }
            })
            .or_insert(Slur {
                slur_id,
                kind: SlurKind::Start,
            });
        marks
            .entry(ends[e].beat)
            .and_modify(|mark| mark.kind = SlurKind::EndStart)
            .or_insert(Slur {
                slur_id,
                kind: SlurKind::End,
            });
    }
    marks
}

/// Ticks of a `(numerator, denominator)` fraction of a whole note.
fn fraction_ticks((numerator, denominator): (i32, i32)) -> i64 {
    numerator as i64 * i64::from(WHOLE_TICKS) / (denominator as i64).max(1)
}

/// Ticks a voice-level `<location>` skips before `beat`.
fn gap_ticks(beat: &MscxBeat) -> u32 {
    beat.gap_before
        .map_or(0, |gap| u32::try_from(fraction_ticks(gap)).unwrap_or(0))
}

/// The staff point a relative `<location>` names from `at`; `None` before
/// the first measure or voice.
fn locate(at: StaffPoint, location: MscxLocation) -> Option<StaffPoint> {
    let measure = usize::try_from(at.0 as i64 + location.measures as i64).ok()?;
    let ticks = location.fractions.map_or(0, fraction_ticks);
    let voice = usize::try_from(at.2 as i64 + location.voices as i64).ok()?;
    Some((measure, at.1 + ticks, voice))
}

fn convert_voice(
    voice: &MscxVoice,
    voice_id: u8,
    measure_ticks: Option<u32>,
    slur_at: impl Fn(usize) -> Option<Slur>,
    report: &mut LossReport,
) -> Voice {
    let mut beats: Vec<Beat> = Vec::with_capacity(voice.beats.len());
    let mut tick_offset: u32 = 0;
    // Grace notes waiting for their main chord, and what was written
    // before them: MuseScore writes all of a chord's grace chords (those
    // after it too) before it, and its segment elements before those.
    let mut graces: Vec<GraceNote> = Vec::new();
    let mut carried: Vec<&MscxSegment> = Vec::new();

    for (beat_idx, beat) in voice.beats.iter().enumerate() {
        tick_offset = tick_offset.saturating_add(gap_ticks(beat));
        if let (Some(grace), MscxBeatKind::Chord(notes)) = (beat.grace, &beat.kind) {
            // Breath marks and hairpin ends after it follow the beat
            // before (one in an earlier measure: see
            // `attach_leading_marks`).
            if let Some(previous) = beats.last_mut() {
                add_breaths(previous, &beat.breaths, report);
                add_hairpin_ends(previous, beat.hairpin_ends);
            }
            graces.append(&mut grace_notes(beat, grace, notes, report));
            carried.push(&beat.segment);
            continue;
        }
        // A whole-measure rest takes its measure's length: one rest, or
        // several where no single note value fits.
        let durations = Some(measure_ticks)
            .flatten()
            .filter(|_| is_measure_rest(beat))
            .map(|ticks| {
                let (durations, left) = rest_durations(ticks);
                if left > 0 {
                    report.note("Measure rest (length no note value fills, left short)");
                }
                durations
            })
            .filter(|durations| !durations.is_empty())
            .unwrap_or_else(|| vec![beat.duration]);
        let Some((&first, more)) = durations.split_first() else {
            continue;
        };
        let mut optimized_beat = convert_beat(beat, tick_offset, slur_at(beat_idx), report);
        optimized_beat.duration = convert_duration(first);
        // A rest takes no grace notes: they wait for the next chord, as
        // in MuseScore, and what was written before them with them.
        if matches!(beat.kind, MscxBeatKind::Chord(_)) {
            optimized_beat.grace_notes = std::mem::take(&mut graces);
            // Latest first: a later dynamic or chord symbol wins, and each
            // earlier segment's hairpins go before the later ones'.
            for segment in carried.drain(..).rev() {
                add_segment_elements(&mut optimized_beat, segment, report);
            }
        }
        // Hairpin ends after a split rest go after its last piece.
        let ends = if more.is_empty() {
            Vec::new()
        } else {
            let (ends, others) = std::mem::take(&mut optimized_beat.events)
                .into_iter()
                .partition(
                    |event| matches!(event, BeatEvent::Hairpin(h) if h.kind == HairpinKind::Stop),
                );
            optimized_beat.events = others;
            ends
        };
        beats.push(optimized_beat);
        tick_offset = tick_offset.saturating_add(duration_ticks(first));
        for &duration in more {
            let rest = MscxBeat::new(duration, MscxBeatKind::Rest);
            beats.push(convert_beat(&rest, tick_offset, None, report));
            tick_offset = tick_offset.saturating_add(duration_ticks(duration));
        }
        if let Some(last) = beats.last_mut() {
            last.events.extend(ends);
        }
    }
    if !graces.is_empty() {
        report.note("Grace (without a main chord)");
    }
    // What was written before those graces has no beat either.
    for segment in carried {
        report_trailing(segment, report);
    }
    report_trailing(&voice.trailing, report);
    for segment in &voice.before_gaps {
        report_unheld(segment, "before a gap", report);
    }

    Voice { voice_id, beats }
}

/// Segment elements after a voice's last beat: there is no beat to hold
/// them.
fn report_trailing(segment: &MscxSegment, report: &mut LossReport) {
    report_unheld(segment, "after the last beat", report);
}

/// Segment elements no beat holds, `place` saying where they were.
fn report_unheld(segment: &MscxSegment, place: &str, report: &mut LossReport) {
    report_superseded(segment, report);
    if segment.dynamic.is_some() {
        report.note(format!("Dynamic ({place})"));
    }
    if segment.harmony.is_some() {
        report.note(format!("Harmony ({place})"));
    }
    for _ in &segment.fermatas {
        report.note(format!("Fermata ({place})"));
    }
    for _ in &segment.hairpin_starts {
        report.note(format!("HairPin ({place})"));
    }
}

/// Dynamics and chord symbols a later one before the same beat replaced:
/// a beat holds one of each.
fn report_superseded(segment: &MscxSegment, report: &mut LossReport) {
    report.note_n(
        "Dynamic (several before one beat, last kept)",
        segment.superseded_dynamics as usize,
    );
    report.note_n(
        "Harmony (several before one beat, last kept)",
        segment.superseded_harmonies as usize,
    );
}

/// A grace chord's notes, the first carrying the chord's marks.
fn grace_notes(
    beat: &MscxBeat,
    grace: MscxGrace,
    notes: &[MscxNote],
    report: &mut LossReport,
) -> Vec<GraceNote> {
    let mut notes = convert_notes(notes, report);
    let mut fermata = false;
    add_chord_marks(beat, &mut notes, &mut fermata, report);
    notes
        .into_iter()
        .enumerate()
        .map(|(i, note)| GraceNote {
            note,
            slash: grace == MscxGrace::Acciaccatura,
            steal_time: None,
            duration: Some(convert_duration(beat.duration)),
            with_previous: i > 0,
            after: grace.is_after(),
        })
        .collect()
}

fn convert_beat(
    beat: &MscxBeat,
    tick_offset: u32,
    slur: Option<Slur>,
    report: &mut LossReport,
) -> Beat {
    let duration = convert_duration(beat.duration);
    let (mut notes, gp_rest) = match &beat.kind {
        MscxBeatKind::Rest => (Vec::new(), true),
        MscxBeatKind::Chord(mscx_notes) => (convert_notes(mscx_notes, report), false),
    };
    let mut fermata = false;
    add_chord_marks(beat, &mut notes, &mut fermata, report);

    let mut out = Beat {
        tick_offset,
        duration,
        notes,
        events: Vec::new(),
        dynamic: None,
        slur,
        lyric: None,
        beam_group: None,
        tuplet: None,
        beams: Vec::new(),
        grace_notes: Vec::new(),
        cue: false,
        chord: None,
        fermata: false,
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
    };
    if fermata {
        set_fermata(&mut out);
    }
    add_segment_elements(&mut out, &beat.segment, report);
    add_breaths(&mut out, &beat.breaths, report);
    add_hairpin_ends(&mut out, beat.hairpin_ends);
    out
}

/// A chord's `<Articulation>`/`<Ornament>`/`<Tremolo>`/`<ChordLine>`
/// marks, on its first note (the model keeps marks per note, MuseScore per
/// chord). A fermata symbol sets `fermata`. Two-chord tremolos and
/// unknown tremolos or chord lines are reported.
fn add_chord_marks(
    beat: &MscxBeat,
    notes: &mut [Note],
    fermata: &mut bool,
    report: &mut LossReport,
) {
    let Some(first) = notes.first_mut() else {
        return;
    };
    for mark in &beat.marks {
        match mark {
            MscxChordMark::Articulation(symbol) | MscxChordMark::Ornament(symbol) => {
                for mark in chord_marks_from_symbol(symbol) {
                    match mark {
                        ChordMark::Articulation(a) => first.articulations.push(a),
                        ChordMark::Ornament(o) => first.ornaments.push(o),
                        ChordMark::Fermata => *fermata = true,
                    }
                }
            }
            MscxChordMark::Tremolo(subtype) => match tremolo_from_subtype(subtype) {
                Some(strokes) => first.ornaments.push(Ornament::Tremolo(strokes)),
                // `c8`...`c64` alternate between two chords.
                None if subtype.trim().starts_with('c') => report.note("Tremolo (two-chord)"),
                None => report.note("Tremolo (unknown)"),
            },
            MscxChordMark::ChordLine(subtype) => match articulation_from_chord_line(subtype) {
                Some(articulation) => first.articulations.push(articulation),
                None => report.note("ChordLine (unknown)"),
            },
        }
    }
}

/// A chord's fermata goes on its first note, a rest's on the beat.
fn set_fermata(beat: &mut Beat) {
    match beat.notes.first_mut() {
        Some(first) => {
            if !first.articulations.contains(&Articulation::Fermata) {
                first.articulations.push(Articulation::Fermata);
            }
        }
        None => beat.fermata = true,
    }
}

/// The segment elements written before a beat: dynamic, chord symbol,
/// fermata and hairpin start. A dynamic or chord symbol the beat already
/// has (written after these) is kept, and this one reported.
fn add_segment_elements(out: &mut Beat, segment: &MscxSegment, report: &mut LossReport) {
    report_superseded(segment, report);
    if let Some(written) = &segment.dynamic {
        if out.dynamic.is_some() {
            report.note("Dynamic (before a grace chord, superseded)");
        } else {
            out.dynamic = dynamic_from_mscx(written);
            // The model has no playback velocity of its own.
            if let Some(dynamic) = &out.dynamic
                && written.velocity.is_some()
                && written.velocity != dynamic.playback_velocity()
            {
                report.note("Dynamic (velocity)");
            }
        }
    }
    if let Some(harmony) = &segment.harmony {
        if out.chord.is_some() {
            report.note("Harmony (before a grace chord, superseded)");
        } else {
            out.chord = chord_symbol(harmony, report);
        }
    }
    if !segment.fermatas.is_empty() {
        set_fermata(out);
    }
    // Before the beat's hairpin stops: hairpins pair in event order, and
    // a segment carried over grace chords comes after the beat's own.
    out.events.splice(
        0..0,
        segment.hairpin_starts.iter().map(|&subtype| {
            let (kind, line) = hairpin_from_subtype(subtype);
            BeatEvent::Hairpin(HairpinEvent { id: 0, kind, line })
        }),
    );
}

/// `<Breath>` symbols after a beat, as articulations on its first note.
fn add_breaths(beat: &mut Beat, symbols: &[String], report: &mut LossReport) {
    for symbol in symbols {
        match beat.notes.first_mut() {
            Some(first) => first.articulations.push(articulation_from_breath(symbol)),
            None => report.note("Breath (after a rest)"),
        }
    }
}

/// Hairpin end elements after a beat (ids are given per track afterwards).
fn add_hairpin_ends(beat: &mut Beat, count: u32) {
    for _ in 0..count {
        beat.events.push(BeatEvent::Hairpin(HairpinEvent {
            id: 0,
            kind: HairpinKind::Stop,
            line: false,
        }));
    }
}

/// A standard `<Harmony>`; Roman-numeral and Nashville ones aren't read.
/// One with no root is "N.C." (no chord).
fn chord_symbol(harmony: &MscxHarmony, report: &mut LossReport) -> Option<ChordSymbol> {
    if harmony.harmony_type.is_some_and(|t| t != 0) {
        report.note("Harmony (Roman numeral/Nashville)");
        return None;
    }
    let name = harmony.name.trim();
    let root = match harmony.root {
        Some(tpc) => match pitch_class_from_tpc(tpc) {
            Some(root) => Some(root),
            None => {
                report.note("Harmony (invalid root)");
                return None;
            }
        },
        None => {
            let letters: String = name.chars().filter(|c| c.is_ascii_alphabetic()).collect();
            if !letters.eq_ignore_ascii_case("nc") {
                report.note("Harmony (without a root)");
                return None;
            }
            None
        }
    };
    let bass = harmony.base.and_then(|tpc| {
        let bass = pitch_class_from_tpc(tpc);
        if bass.is_none() {
            report.note("Harmony (invalid bass)");
        }
        bass
    });
    Some(ChordSymbol {
        root,
        kind: name.to_string(),
        bass,
        inversion: None,
        degrees: Vec::new(),
        frame: None,
    })
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
    Duration::dotted_ticks(
        convert_duration_kind(duration.kind).ticks(DIVISIONS),
        duration.dots,
    )
}

fn convert_notes(notes: &[MscxNote], report: &mut LossReport) -> Vec<Note> {
    notes
        .iter()
        .filter_map(|note| convert_note(note, report))
        .collect()
}

/// The pitch of a note's MIDI `<pitch>`, spelled by its `<tpc>`; without
/// one (or with one that doesn't spell it), sharps are assumed. `None` for
/// a pitch the model can't hold: beyond MIDI's 127, or in octave −1 (the
/// model's octaves start at 0).
fn convert_pitch(midi: u8, tpc: Option<i8>, report: &mut LossReport) -> Option<Pitch> {
    if midi > 127 {
        report.note("Note (pitch beyond MIDI's range)");
        return None;
    }
    if let Some(tpc) = tpc {
        if let Some(spelled) = pitch_from_midi_tpc(midi, tpc) {
            return Some(spelled);
        }
        if !tpc_spells_midi(midi, tpc) {
            report.note("Note (spelling not matching its pitch)");
        } else if midi >= 12 {
            // Spelled in octave −1 (B♯ for C0): kept, respelled with sharps.
            report.note("Note (spelling below octave 0)");
        }
    }
    if midi < 12 {
        report.note("Note (pitch below octave 0)");
        return None;
    }
    Some(midi_to_pitch(midi as i8))
}

/// A note, or `None` (dropped and reported) when its pitch is out of the
/// model's range.
fn convert_note(note: &MscxNote, report: &mut LossReport) -> Option<Note> {
    let pitch = match note.pitch {
        Some(midi) => Some(convert_pitch(midi, note.tpc, report)?),
        None => None,
    };
    let tie = TieType::from_ends(note.tie_end, note.tie_start);
    Some(Note {
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
        notehead: note.head.as_deref().and_then(notehead_from_name),
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
    })
}

/// A MuseScore notehead group (`<head>`); `None` for the normal head and
/// groups without a counterpart.
fn notehead_from_name(name: &str) -> Option<Notehead> {
    Some(match name {
        "cross" | "plus" | "heavy-cross" | "heavy-cross-hat" | "withx" => Notehead::X,
        "xcircle" => Notehead::CircleX,
        "diamond" | "diamond-old" | "large-diamond" => Notehead::Diamond,
        "triangle-up" | "triangle-down" => Notehead::Triangle,
        "la" => Notehead::Square,
        "slash" => Notehead::Slash,
        "large-arrow" => Notehead::ArrowUp,
        _ => return None,
    })
}
