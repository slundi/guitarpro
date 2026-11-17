//! [`LoadedScore`] → MSCX conversion.
//!
//! The generated MSCX targets MuseScore 4.10 (`<museScore version="4.10">`)
//! and covers the same subset that [`super::to_optimized`] handles:
//! metadata, instruments/staves, timeline (per-measure time/key/tempo),
//! per-track measure content (voices → beats → notes with pitch/string/fret,
//! ties, slurs and noteheads), instruments' sounds and programs, clefs, and
//! transposing staves' written keys and spellings. Everything else is
//! intentionally left out — the caller can diff the resulting XML against
//! the input to see which structural features were dropped.

use std::collections::{HashMap, VecDeque};
use std::fmt::Write as _;

use crate::convert::mscz::instruments::mscx_instrument;
use crate::convert::mscz::notation::{
    ArticulationMark, articulation_mark, barline_to_mscx, dynamic_text_markup, dynamic_to_mscx,
    dynamics_voice_only, hairpin_subtype, jump_kind_to_mscx, ornament_symbol, tremolo_subtype,
};
use crate::convert::mscz::spelling::{tpc_from_pitch, transpose_fifths, written_pitch};
use crate::convert::mscz::to_optimized::{DIVISIONS, WHOLE_TICKS, measure_duration_ticks};
use crate::model::mscz::{Mscx, MscxDynamic, MscxMeasureMark};
use crate::model::optimized::{
    LoadedScore,
    beat::{Beat, Duration, GraceNote, SlurKind, Voice},
    effect::{BeatEvent, HairpinKind},
    global::{Instrument, InstrumentKind, Transpose},
    metadata::{ChordSymbol, Metadata, TimeSignature, normalized_fifths},
    note::{Articulation, Note, NoteValue, Notehead, Ornament},
    timeline::{BarlineStyle, JumpKind, MeasureDef, ending_spans},
    track::{Clef, MeasureData, StaffDef, StaffDisplay, Track},
};

const MSCX_VERSION: &str = "4.10";

/// Emit an MSCX view for the given [`LoadedScore`].
///
/// The returned `Mscx.raw_xml` is a freshly generated string that
/// [`crate::io::mscz::write_mscz`] can archive without further processing.
pub fn loaded_score_to_mscx(score: &LoadedScore) -> Mscx {
    let mut out = String::new();
    write_header(&mut out);
    writeln!(out, r#"<museScore version="{MSCX_VERSION}">"#).ok();
    writeln!(out, "  <programVersion>4.10.0</programVersion>").ok();
    writeln!(out, "  <programRevision>guitarpro</programRevision>").ok();
    writeln!(out, "  <Score>").ok();
    writeln!(
        out,
        "    <Division>{}</Division>",
        super::to_optimized::DIVISIONS
    )
    .ok();

    write_meta_tags(&mut out, &score.score.metadata);
    write_parts(&mut out, score);
    write_master_staves(&mut out, score);

    writeln!(out, "  </Score>").ok();
    writeln!(out, "</museScore>").ok();

    // Re-parse the generated XML to populate the AST fields consistently.
    // This keeps `raw_xml` and the structured view coherent.
    match crate::io::mscz::parse::parse_mscx(&out) {
        Ok(mscx) => mscx,
        Err(_) => empty_shell(out),
    }
}

fn empty_shell(raw: String) -> Mscx {
    Mscx {
        raw_xml: raw,
        version: MSCX_VERSION.to_string(),
        program_version: None,
        program_revision: None,
        division: Some(super::to_optimized::DIVISIONS),
        meta_tags: Vec::new(),
        parts: Vec::new(),
        measure_counts: Vec::new(),
        score_staves: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Header + meta tags
// ---------------------------------------------------------------------------

fn write_header(out: &mut String) {
    writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>"#).ok();
}

fn write_meta_tags(out: &mut String, metadata: &Metadata) {
    let pairs = std::iter::once(("workTitle", metadata.title.clone()))
        .chain(metadata.composer.clone().map(|value| ("composer", value)))
        .chain(metadata.copyright.clone().map(|value| ("copyright", value)))
        .chain(
            metadata
                .movement_number
                .clone()
                .map(|value| ("movementNumber", value)),
        )
        .chain(
            metadata
                .identification
                .as_ref()
                .and_then(|id| id.encoding_date.clone())
                .map(|value| ("creationDate", value)),
        );

    for (name, value) in pairs {
        writeln!(
            out,
            "    <metaTag name=\"{}\">{}</metaTag>",
            xml_escape(name),
            xml_escape(&value)
        )
        .ok();
    }
}

// ---------------------------------------------------------------------------
// Parts (instrument definitions)
// ---------------------------------------------------------------------------

fn write_parts(out: &mut String, score: &LoadedScore) {
    for (index, track) in score.score.tracks.iter().enumerate() {
        write_part(
            out,
            index,
            track,
            track_instrument(score, track),
            track_staff(score, track),
        );
    }
}

/// The instrument a track plays.
fn track_instrument<'a>(score: &'a LoadedScore, track: &Track) -> Option<&'a Instrument> {
    score.score.instruments.get(track.instrument.0 as usize)
}

/// A track's first staff, the one its MSCX part and master staff describe.
fn track_staff<'a>(score: &'a LoadedScore, track: &Track) -> Option<&'a StaffDef> {
    track
        .staves
        .first()
        .and_then(|id| score.score.staves.get(id.0 as usize))
}

fn write_part(
    out: &mut String,
    index: usize,
    track: &Track,
    instrument: Option<&Instrument>,
    staff_def: Option<&StaffDef>,
) {
    writeln!(out, "    <Part id=\"{}\">", index + 1).ok();
    writeln!(out, "      <Staff id=\"{}\">", index + 1).ok();

    if let Some(staff) = staff_def {
        // A drum set's notes are drum-map pitches, placed by MuseScore's
        // percussion staff.
        let percussion = instrument
            .is_some_and(|instrument| matches!(instrument.kind, InstrumentKind::Percussion));
        let (group, name) = match staff.display {
            _ if percussion => ("percussion", "perc5Line"),
            StaffDisplay::Tab => ("tablature", "tab6StrCommon"),
            StaffDisplay::Notation | StaffDisplay::NotationTab => ("pitched", "stdNormal"),
        };
        writeln!(out, "        <StaffType group=\"{group}\">").ok();
        writeln!(out, "          <name>{name}</name>").ok();
        writeln!(out, "          </StaffType>").ok();
        if let Some(clef) = default_clef(staff) {
            writeln!(out, "        <defaultClef>{clef}</defaultClef>").ok();
        }
    }
    writeln!(out, "        </Staff>").ok();
    writeln!(
        out,
        "      <trackName>{}</trackName>",
        xml_escape(&track.name)
    )
    .ok();

    if let Some(instr) = instrument {
        let (template, sound) = mscx_instrument(instr);
        writeln!(out, "      <Instrument id=\"{template}\">").ok();
        writeln!(
            out,
            "        <longName>{}</longName>",
            xml_escape(&instr.name)
        )
        .ok();
        if let Some(abbrev) = instr.abbreviation.as_deref() {
            writeln!(out, "        <shortName>{}</shortName>", xml_escape(abbrev)).ok();
        }
        writeln!(
            out,
            "        <instrumentId>{}</instrumentId>",
            xml_escape(&sound)
        )
        .ok();
        if let Some(transpose) = instr.transpose.as_ref().map(Transpose::folded) {
            writeln!(
                out,
                "        <transposeChromatic>{}</transposeChromatic>",
                transpose.chromatic
            )
            .ok();
            // Always paired: without it MuseScore takes 0 steps. Unknown
            // steps are estimated as the `<tpc2>` spelling does.
            writeln!(
                out,
                "        <transposeDiatonic>{}</transposeDiatonic>",
                super::spelling::transpose_diatonic(&transpose)
            )
            .ok();
        }
        if let InstrumentKind::Stringed { tuning, .. } = &instr.kind {
            writeln!(out, "        <StringData>").ok();
            writeln!(out, "          <frets>24</frets>").ok();
            for pitch in tuning {
                let midi = pitch.midi();
                writeln!(out, "          <string>{midi}</string>").ok();
            }
            writeln!(out, "          </StringData>").ok();
        }
        if let Some(clef) = staff_def.and_then(default_clef) {
            writeln!(out, "        <clef>{clef}</clef>").ok();
        }
        writeln!(out, "        <Channel>").ok();
        writeln!(out, "          <program value=\"{}\"/>", instr.midi_program).ok();
        writeln!(out, "          </Channel>").ok();
        writeln!(out, "        </Instrument>").ok();
    }

    writeln!(out, "      </Part>").ok();
}

// ---------------------------------------------------------------------------
// Master staves (measure content per track)
// ---------------------------------------------------------------------------

fn write_master_staves(out: &mut String, score: &LoadedScore) {
    let nominal = nominal_lengths(&score.score.timeline);
    let measure_ticks = measure_lengths(&score.score.timeline, &nominal);
    for (track_idx, track) in score.score.tracks.iter().enumerate() {
        // Jumps, markers and voltas are system-wide: the first staff has them.
        let staff = StaffContext::new(score, track, track_idx == 0, &measure_ticks);
        writeln!(out, "    <Staff id=\"{}\">", track_idx + 1).ok();
        write_measures(out, &score.score.timeline, &nominal, &staff);
        writeln!(out, "      </Staff>").ok();
    }
}

/// What a track's master staff needs beyond its measures: its first clef,
/// its instrument's transposition, each measure's voices in written order
/// and barline, its slurs' and hairpins' spanner ends, and (on the first
/// staff) the voltas'.
struct StaffContext<'a> {
    clef: Option<Clef>,
    /// The instrument's transposition, octaves folded in.
    transpose: Option<Transpose>,
    /// Whether this staff carries the system-wide jumps, markers and voltas.
    system: bool,
    /// Per timeline position: the measure's clef change and its voices.
    measures: Vec<(Option<Clef>, Vec<&'a Voice>)>,
    /// Per timeline position and voice: where its beats start.
    layouts: Vec<Vec<VoiceLayout>>,
    /// Per timeline position: the measure's length by its time signature.
    measure_ticks: &'a [u32],
    /// Per timeline position: the measure's right barline style.
    barlines: Vec<BarlineStyle>,
    slurs: HashMap<BeatPosition, Vec<SpannerEnd>>,
    /// Tie ends per note: (beat, note index).
    ties: HashMap<(BeatPosition, usize), Vec<SpannerEnd>>,
    hairpins: HashMap<BeatPosition, BeatSpanners>,
    /// Whether its dynamics are each for their own voice only: when they
    /// are in several voices, which the model gives levels of their own.
    voice_only_dynamics: bool,
    /// Volta spanner ends at the start and end of a measure's first voice.
    volta_starts: HashMap<usize, Vec<SpannerEnd>>,
    volta_ends: HashMap<usize, Vec<SpannerEnd>>,
}

impl<'a> StaffContext<'a> {
    /// The length of the measure at `position`.
    fn measure_ticks_at(&self, position: usize) -> u32 {
        self.measure_ticks
            .get(position)
            .copied()
            .unwrap_or(measure_duration_ticks(COMMON_TIME))
    }

    fn new(
        score: &'a LoadedScore,
        track: &'a Track,
        system: bool,
        measure_ticks: &'a [u32],
    ) -> Self {
        let staff = track_staff(score, track);
        let notation = staff.is_none_or(|staff| staff.display != StaffDisplay::Tab);
        let clef = staff.and_then(notation_clef);
        let transpose = track_instrument(score, track)
            .and_then(|instrument| instrument.transpose.as_ref())
            .map(Transpose::folded)
            .filter(|t| t.chromatic != 0 || t.diatonic.is_some_and(|d| d != 0));
        let measures: Vec<(Option<Clef>, Vec<&Voice>)> = score
            .score
            .timeline
            .iter()
            .map(|measure_def| {
                let data = track.measures.get(&measure_def.index);
                let clef = data
                    .and_then(|data| data.clef)
                    .filter(|&clef| notation && clef != Clef::Tab);
                (clef, sorted_voices(data))
            })
            .collect();
        let layouts: Vec<Vec<VoiceLayout>> = measures
            .iter()
            .map(|(_, voices)| voices.iter().map(|voice| voice_layout(voice)).collect())
            .collect();
        let placed = beats_in_time_order(&measures, &layouts);
        let slurs = plan_slurs(&placed);
        let ties = plan_ties(&placed);
        let hairpins = plan_hairpins(&placed);
        let voice_only_dynamics = dynamics_voice_only(
            placed
                .iter()
                .filter(|placed| placed.beat.dynamic.is_some())
                .map(|placed| placed.voice_id),
        );
        // A track's own barline replaces the timeline's.
        let barlines = score
            .score
            .timeline
            .iter()
            .map(|measure_def| {
                track
                    .measures
                    .get(&measure_def.index)
                    .and_then(|data| data.barline_right.as_ref())
                    .or(measure_def.barline_right.as_ref())
                    .map_or(BarlineStyle::Regular, |barline| barline.style)
            })
            .collect();
        let (volta_starts, volta_ends) = if system {
            plan_voltas(&score.score.timeline, &layouts, measure_ticks)
        } else {
            Default::default()
        };
        Self {
            clef,
            transpose,
            system,
            measures,
            layouts,
            measure_ticks,
            barlines,
            slurs,
            ties,
            hairpins,
            voice_only_dynamics,
            volta_starts,
            volta_ends,
        }
    }
}

/// A beat's place in the written staff: (timeline position, voice position
/// in the measure, beat index).
type BeatPosition = (usize, usize, usize);

/// The measure's voices in written order (sorted by id).
fn sorted_voices(data: Option<&MeasureData>) -> Vec<&Voice> {
    data.map(|data| {
        let mut list: Vec<(u8, &Voice)> = data.voices.iter().map(|(k, v)| (*k, v)).collect();
        list.sort_by_key(|(id, _)| *id);
        // Each at the place of its id (MuseScore numbers a measure's
        // voices by their place): a missing one's place holds
        // [`GAP_VOICE`], so the ones after it keep theirs.
        let mut voices: Vec<&Voice> = Vec::new();
        for (id, voice) in list {
            voices.resize(voices.len().max(usize::from(id)), &GAP_VOICE);
            voices.push(voice);
        }
        voices
    })
    .unwrap_or_default()
}

/// The place of a voice a measure doesn't have, before one it has (see
/// [`sorted_voices`]): written as the first voice's measure rest (it must
/// fill the measure), as an empty `<voice>` elsewhere.
static GAP_VOICE: Voice = Voice {
    voice_id: 0,
    beats: Vec::new(),
};

/// `nominal`: each measure's length by its time signature
/// ([`nominal_lengths`]).
fn write_measures(
    out: &mut String,
    timeline: &[MeasureDef],
    nominal: &[u32],
    staff: &StaffContext,
) {
    for (position, measure_def) in timeline.iter().enumerate() {
        // A pickup or irregular measure says its length.
        let ticks = staff.measure_ticks_at(position);
        if nominal
            .get(position)
            .is_some_and(|&nominal| nominal != ticks)
        {
            let len = fraction_text(i64::from(ticks));
            writeln!(out, "      <Measure len=\"{len}\">").ok();
        } else {
            writeln!(out, "      <Measure>").ok();
        }
        if staff.system {
            write_measure_marks(out, measure_def);
        }

        if measure_def.has_navigation(JumpKind::RepeatOpen) {
            writeln!(out, "        <startRepeat/>").ok();
        }
        let end_repeat = measure_def
            .navigation
            .iter()
            .find(|ev| ev.kind == JumpKind::RepeatClose)
            // A close without a count plays twice.
            .map(|ev| ev.repeat_count.unwrap_or(2));
        if let Some(count) = end_repeat {
            writeln!(out, "        <endRepeat>{count}</endRepeat>").ok();
        }

        let (clef, voices) = staff
            .measures
            .get(position)
            .map_or((None, &[][..]), |(clef, voices)| (*clef, &voices[..]));
        let primary = PrimaryVoice {
            measure_def,
            // The first measure's own clef, else the staff's.
            header_clef: if position == 0 {
                clef.or(staff.clef)
            } else {
                None
            },
            // MuseScore writes a clef change at the end of the measure
            // before it.
            next_clef: staff.measures.get(position + 1).and_then(|(clef, _)| *clef),
            measure_ticks: staff.measure_ticks_at(position),
            transpose: staff.transpose.as_ref(),
            // MuseScore draws an end repeat's barline itself; an explicit
            // one would replace it.
            barline: staff
                .barlines
                .get(position)
                .copied()
                .filter(|_| end_repeat.is_none())
                .unwrap_or(BarlineStyle::Regular),
            volta_starts: staff
                .volta_starts
                .get(&position)
                .map_or(&[][..], Vec::as_slice),
            volta_ends: staff
                .volta_ends
                .get(&position)
                .map_or(&[][..], Vec::as_slice),
        };

        // Emit each voice; if the measure has no voices, produce an empty
        // whole-rest voice so the output remains a valid MSCX document.
        if voices.is_empty() {
            write_empty_voice(out, &primary);
        } else {
            for (voice_idx, voice) in voices.iter().enumerate() {
                if voice_idx > 0 && std::ptr::eq(*voice, &GAP_VOICE) {
                    writeln!(out, "        <voice>").ok();
                    writeln!(out, "          </voice>").ok();
                    continue;
                }
                let primary = (voice_idx == 0).then_some(&primary);
                write_voice(out, voice, primary, staff, (position, voice_idx));
            }
        }

        writeln!(out, "        </Measure>").ok();
    }
}

/// What only a measure's first voice carries: signatures, clefs, the
/// barline and voltas.
struct PrimaryVoice<'a> {
    measure_def: &'a MeasureDef,
    header_clef: Option<Clef>,
    next_clef: Option<Clef>,
    /// The measure's length by its time signature.
    measure_ticks: u32,
    transpose: Option<&'a Transpose>,
    barline: BarlineStyle,
    volta_starts: &'a [SpannerEnd],
    volta_ends: &'a [SpannerEnd],
}

fn write_empty_voice(out: &mut String, primary: &PrimaryVoice) {
    writeln!(out, "        <voice>").ok();
    write_voice_start(out, primary);
    write_measure_rest(out, primary.measure_ticks);
    write_voice_end(out, primary, None);
    writeln!(out, "          </voice>").ok();
}

fn write_voice(
    out: &mut String,
    voice: &Voice,
    primary: Option<&PrimaryVoice>,
    staff: &StaffContext,
    (position, voice_idx): (usize, usize),
) {
    let measure_ticks = staff.measure_ticks_at(position);
    // The layouts are planned from the same voices, so one is always
    // there; were it not, laying the voice out again beats dropping it
    // (and, for the first voice, the measure's signatures and barline).
    let planned = staff
        .layouts
        .get(position)
        .and_then(|voices| voices.get(voice_idx));
    let fallback;
    let layout = match planned {
        Some(layout) => layout,
        None => {
            fallback = voice_layout(voice);
            &fallback
        }
    };
    writeln!(out, "        <voice>").ok();
    if let Some(primary) = primary {
        write_voice_start(out, primary);
    }
    if voice.beats.is_empty() {
        write_measure_rest(out, measure_ticks);
    } else {
        // A lone rest filling the measure is MuseScore's measure rest.
        let fills_measure = matches!(&voice.beats[..], [beat] if beat.is_staff_rest()
            && beat.duration.tuplet.is_none()
            && layout.starts.first().copied().unwrap_or(0) == 0
            && beat.duration.written_ticks(DIVISIONS) == measure_ticks);
        for (beat_idx, beat) in voice.beats.iter().enumerate() {
            let here = (position, voice_idx, beat_idx);
            if let Some(&gap) = layout.gaps.get(beat_idx)
                && gap > 0
            {
                write_gap(out, gap);
            }
            let slurs = staff.slurs.get(&here).map_or(&[][..], Vec::as_slice);
            let ties: Vec<&[SpannerEnd]> = (0..beat.notes.len())
                .map(|note| staff.ties.get(&(here, note)).map_or(&[][..], Vec::as_slice))
                .collect();
            let hairpins = staff.hairpins.get(&here);
            let measure_rest = fills_measure.then_some(measure_ticks);
            write_beat(
                out,
                beat,
                (slurs, &ties),
                hairpins,
                staff.transpose.as_ref(),
                measure_rest,
                staff.voice_only_dynamics,
            );
        }
    }
    if let Some(primary) = primary {
        write_voice_end(out, primary, layout.end);
    }
    writeln!(out, "          </voice>").ok();
}

fn write_voice_start(out: &mut String, primary: &PrimaryVoice) {
    for volta in primary.volta_starts {
        write_spanner(out, "          ", volta);
    }
    if let Some(clef) = primary.header_clef {
        write_clef(out, clef, true);
    }
    write_measure_signatures(out, primary.measure_def, primary.transpose);
}

/// The clef change for the next measure, the barline and the voltas
/// ending here. They go at the measure's end, so after a voice shorter than
/// the measure (`voice_ticks`, `None` for a whole-measure rest) a
/// `<location>` moves past the gap first; MuseScore would place them
/// mid-measure.
fn write_voice_end(out: &mut String, primary: &PrimaryVoice, voice_ticks: Option<u32>) {
    let barline = barline_to_mscx(primary.barline);
    let at_end = primary.next_clef.is_some() || barline.is_some() || !primary.volta_ends.is_empty();
    if at_end
        && let Some(voice) = voice_ticks
        && primary.measure_ticks > voice
    {
        write_gap(out, primary.measure_ticks - voice);
    }
    if let Some(clef) = primary.next_clef {
        write_clef(out, clef, false);
    }
    if let Some(barline) = barline {
        writeln!(out, "          <BarLine>").ok();
        writeln!(out, "            <subtype>{}</subtype>", barline.subtype).ok();
        if let (Some(from), Some(to)) = (barline.span_from, barline.span_to) {
            writeln!(out, "            <spanFrom>{from}</spanFrom>").ok();
            writeln!(out, "            <spanTo>{to}</spanTo>").ok();
        }
        if !barline.visible {
            writeln!(out, "            <visible>0</visible>").ok();
        }
        writeln!(out, "            </BarLine>").ok();
    }
    for volta in primary.volta_ends {
        write_spanner(out, "          ", volta);
    }
}

/// A whole-measure rest, `ticks` long.
fn write_measure_rest(out: &mut String, ticks: u32) {
    writeln!(out, "          <Rest>").ok();
    writeln!(out, "            <durationType>measure</durationType>").ok();
    writeln!(
        out,
        "            <duration>{}</duration>",
        fraction_text(ticks as i64)
    )
    .ok();
    writeln!(out, "            </Rest>").ok();
}

/// A voice-level `<location>`: MuseScore skips `ticks` in the voice.
fn write_gap(out: &mut String, ticks: u32) {
    let gap = fraction_text(ticks as i64);
    writeln!(out, "          <location>").ok();
    writeln!(out, "            <fractions>{gap}</fractions>").ok();
    writeln!(out, "            </location>").ok();
}

/// A measure's jumps and markers, as the timeline's navigation lists them.
fn write_measure_marks(out: &mut String, measure_def: &MeasureDef) {
    for event in &measure_def.navigation {
        match jump_kind_to_mscx(event.kind) {
            Some(MscxMeasureMark::Marker(marker)) => {
                writeln!(out, "        <Marker>").ok();
                // The text is MSCX markup (`<sym>segno</sym>`).
                writeln!(out, "          <text>{}</text>", marker.text).ok();
                writeln!(out, "          <label>{}</label>", marker.label).ok();
                writeln!(out, "          </Marker>").ok();
            }
            Some(MscxMeasureMark::Jump(jump)) => {
                writeln!(out, "        <Jump>").ok();
                writeln!(out, "          <text>{}</text>", xml_escape(&jump.text)).ok();
                writeln!(out, "          <jumpTo>{}</jumpTo>", jump.jump_to).ok();
                writeln!(out, "          <playUntil>{}</playUntil>", jump.play_until).ok();
                writeln!(
                    out,
                    "          <continueAt>{}</continueAt>",
                    jump.continue_at
                )
                .ok();
                writeln!(out, "          </Jump>").ok();
            }
            None => {}
        }
    }
}

fn write_clef(out: &mut String, clef: Clef, header: bool) {
    let name = clef_name(clef);
    writeln!(out, "          <Clef>").ok();
    writeln!(out, "            <concertClefType>{name}</concertClefType>").ok();
    writeln!(
        out,
        "            <transposingClefType>{name}</transposingClefType>"
    )
    .ok();
    if header {
        writeln!(out, "            <isHeader>1</isHeader>").ok();
    }
    writeln!(out, "            </Clef>").ok();
}

fn write_measure_signatures(
    out: &mut String,
    measure_def: &MeasureDef,
    transpose: Option<&Transpose>,
) {
    if let Some(sig) = measure_def.time_signature {
        writeln!(out, "          <TimeSig>").ok();
        writeln!(out, "            <sigN>{}</sigN>", sig.numerator).ok();
        writeln!(out, "            <sigD>{}</sigD>", sig.denominator).ok();
        writeln!(out, "            </TimeSig>").ok();
    }
    if let Some(key) = measure_def.key_signature {
        // The timeline key is the concert one; a transposing staff also
        // states its written key, or MuseScore shows the concert key there.
        let fifths = i32::from(key.fifths());
        writeln!(out, "          <KeySig>").ok();
        writeln!(out, "            <concertKey>{fifths}</concertKey>").ok();
        if let Some(transpose) = transpose {
            let actual = normalized_fifths(fifths - transpose_fifths(transpose));
            writeln!(out, "            <actualKey>{actual}</actualKey>").ok();
        }
        writeln!(out, "            <mode>{}</mode>", key.mode.name()).ok();
        writeln!(out, "            </KeySig>").ok();
    }
    if let Some(tempo) = measure_def.tempo {
        let bps = tempo / 60.0;
        writeln!(out, "          <Tempo>").ok();
        writeln!(out, "            <tempo>{bps}</tempo>").ok();
        writeln!(out, "            </Tempo>").ok();
    }
}

fn write_beat(
    out: &mut String,
    beat: &Beat,
    // The chord's slur ends, and each note's tie ends.
    (slurs, ties): (&[SpannerEnd], &[&[SpannerEnd]]),
    hairpins: Option<&BeatSpanners>,
    transpose: Option<&Transpose>,
    measure_rest: Option<u32>,
    voice_only_dynamics: bool,
) {
    // Segment elements before the chord: they belong to its segment, as
    // do its grace notes.
    if let Some(chord) = &beat.chord {
        write_harmony(out, chord);
    }
    if let Some(dynamic) = &beat.dynamic {
        let dynamic = MscxDynamic {
            voice_only: voice_only_dynamics,
            ..dynamic_to_mscx(dynamic)
        };
        writeln!(out, "          <Dynamic>").ok();
        if let Some(subtype) = &dynamic.subtype {
            writeln!(out, "            <subtype>{subtype}</subtype>").ok();
        }
        if let Some(velocity) = dynamic.velocity {
            writeln!(out, "            <velocity>{velocity}</velocity>").ok();
        }
        if let Some(text) = &dynamic.text {
            let markup = dynamic_text_markup(text, xml_escape);
            writeln!(out, "            <text>{markup}</text>").ok();
        }
        if dynamic.voice_only {
            writeln!(
                out,
                "            <voiceAssignment>currentVoiceOnly</voiceAssignment>"
            )
            .ok();
        }
        writeln!(out, "            </Dynamic>").ok();
    }
    let fermata = beat.fermata
        || beat
            .notes
            .iter()
            .any(|n| n.articulations.contains(&Articulation::Fermata));
    if fermata {
        writeln!(out, "          <Fermata>").ok();
        writeln!(out, "            <subtype>fermataAbove</subtype>").ok();
        writeln!(out, "            </Fermata>").ok();
    }
    for spanner in hairpins.map_or(&[][..], |h| h.before.as_slice()) {
        write_spanner(out, "          ", spanner);
    }
    // All grace chords go before their main chord, those after it too:
    // MuseScore attaches them to the next chord it reads, so a rest's
    // (which MuseScore can't hold) are left out rather than moved on.
    if !beat.is_staff_rest() {
        let (after, before): (Vec<&GraceNote>, Vec<&GraceNote>) =
            beat.grace_notes.iter().partition(|g| g.after);
        write_grace_chords(out, &before, transpose);
        write_grace_chords(out, &after, transpose);
    }

    if let Some(ticks) = measure_rest {
        write_measure_rest(out, ticks);
    } else if beat.is_staff_rest() {
        writeln!(out, "          <Rest>").ok();
        write_duration(out, &beat.duration);
        writeln!(out, "            </Rest>").ok();
    } else {
        writeln!(out, "          <Chord>").ok();
        write_duration(out, &beat.duration);
        write_chord_marks(out, &beat.notes);
        for slur in slurs {
            write_spanner(out, "            ", slur);
        }
        // A note without a pitch has no place on the staff.
        for (index, note) in beat.notes.iter().enumerate() {
            if note.pitch.is_none() {
                continue;
            }
            write_note(
                out,
                note,
                ties.get(index).copied().unwrap_or_default(),
                transpose,
            );
        }
        writeln!(out, "            </Chord>").ok();
    }

    for symbol in breath_marks(&beat.notes) {
        writeln!(out, "          <Breath>").ok();
        writeln!(out, "            <symbol>{symbol}</symbol>").ok();
        writeln!(out, "            </Breath>").ok();
    }
    for spanner in hairpins.map_or(&[][..], |h| h.after.as_slice()) {
        write_spanner(out, "          ", spanner);
    }
}

/// A chord symbol: `<root>`/`<base>` as tonal pitch classes, the kind as
/// `<name>`; N.C. as a root-less name.
fn write_harmony(out: &mut String, chord: &ChordSymbol) {
    writeln!(out, "          <Harmony>").ok();
    match chord.root {
        Some(root) => {
            writeln!(out, "            <root>{}</root>", tpc_from_pitch(root)).ok();
            if !chord.kind.is_empty() {
                writeln!(out, "            <name>{}</name>", xml_escape(&chord.kind)).ok();
            }
            if let Some(bass) = chord.bass {
                writeln!(out, "            <base>{}</base>", tpc_from_pitch(bass)).ok();
            }
        }
        None => {
            writeln!(out, "            <name>N.C.</name>").ok();
        }
    }
    writeln!(out, "            </Harmony>").ok();
}

/// A chord's articulations and ornaments, merged over its notes (the
/// model keeps marks per note, MuseScore per chord). Fermatas and breath
/// marks are elements of their own.
fn write_chord_marks<'n>(out: &mut String, notes: impl IntoIterator<Item = &'n Note>) {
    let mut articulations: Vec<&Articulation> = Vec::new();
    let mut ornaments: Vec<&Ornament> = Vec::new();
    for note in notes {
        for a in &note.articulations {
            if !articulations.contains(&a) {
                articulations.push(a);
            }
        }
        for o in &note.ornaments {
            if !ornaments.contains(&o) {
                ornaments.push(o);
            }
        }
    }
    for articulation in articulations {
        match articulation_mark(articulation) {
            Some(ArticulationMark::Symbol(symbol)) => {
                writeln!(out, "            <Articulation>").ok();
                writeln!(
                    out,
                    "              <subtype>{}</subtype>",
                    xml_escape(&symbol)
                )
                .ok();
                writeln!(out, "              </Articulation>").ok();
            }
            Some(ArticulationMark::ChordLine(kind)) => {
                writeln!(out, "            <ChordLine>").ok();
                writeln!(out, "              <subtype>{kind}</subtype>").ok();
                writeln!(out, "              </ChordLine>").ok();
            }
            Some(ArticulationMark::Breath(_)) | None => {}
        }
    }
    for ornament in ornaments {
        if let Ornament::Tremolo(strokes) = ornament {
            writeln!(out, "            <Tremolo>").ok();
            writeln!(
                out,
                "              <subtype>{}</subtype>",
                tremolo_subtype(*strokes)
            )
            .ok();
            writeln!(out, "              </Tremolo>").ok();
        } else if let Some(symbol) = ornament_symbol(ornament) {
            writeln!(out, "            <Ornament>").ok();
            writeln!(
                out,
                "              <subtype>{}</subtype>",
                xml_escape(symbol)
            )
            .ok();
            writeln!(out, "              </Ornament>").ok();
        }
    }
}

/// The `<Breath>` symbols for a chord's breath marks and caesuras.
fn breath_marks(notes: &[Note]) -> Vec<&'static str> {
    let mut symbols = Vec::new();
    for articulation in notes.iter().flat_map(|n| &n.articulations) {
        if let Some(ArticulationMark::Breath(symbol)) = articulation_mark(articulation)
            && !symbols.contains(&symbol)
        {
            symbols.push(symbol);
        }
    }
    symbols
}

/// Grace notes as grace chords: a note sounding with the previous one
/// joins its chord.
fn write_grace_chords(out: &mut String, graces: &[&GraceNote], transpose: Option<&Transpose>) {
    let mut chords: Vec<Vec<&GraceNote>> = Vec::new();
    for &grace in graces {
        match chords.last_mut() {
            Some(chord) if grace.with_previous => chord.push(grace),
            _ => chords.push(vec![grace]),
        }
    }
    for mut chord in chords {
        // As in a main chord, a note without a pitch has no place on the
        // staff; a grace chord of none is left out.
        chord.retain(|grace| grace.note.pitch.is_some());
        let Some(&first) = chord.first() else {
            continue;
        };
        let duration = first.duration.unwrap_or(Duration {
            base: NoteValue::Eighth,
            dots: 0,
            tuplet: None,
        });
        writeln!(out, "          <Chord>").ok();
        writeln!(out, "            <{}/>", grace_tag(first, duration.base)).ok();
        write_duration(out, &duration);
        write_chord_marks(out, chord.iter().map(|grace| &grace.note));
        for grace in &chord {
            write_note(out, &grace.note, &[], transpose);
        }
        writeln!(out, "            </Chord>").ok();
    }
}

/// The element marking a grace chord: slashed before the note, after it,
/// or by its value.
fn grace_tag(grace: &GraceNote, value: NoteValue) -> &'static str {
    match (grace.after, value) {
        (true, NoteValue::Sixteenth) => "grace16after",
        (true, NoteValue::ThirtySecond) => "grace32after",
        (true, _) => "grace8after",
        (false, _) if grace.slash => "acciaccatura",
        (false, NoteValue::Quarter) => "grace4",
        (false, NoteValue::Sixteenth) => "grace16",
        (false, NoteValue::ThirtySecond) => "grace32",
        (false, _) => "appoggiatura",
    }
}

/// `<dots>` goes first: MuseScore times a chord whose `<dots>` follows its
/// `<durationType>` as undotted, filling the measure up with rests.
fn write_duration(out: &mut String, duration: &Duration) {
    if duration.dots > 0 {
        writeln!(out, "            <dots>{}</dots>", duration.dots).ok();
    }
    let kind = duration_kind_name(duration.base);
    writeln!(out, "            <durationType>{kind}</durationType>").ok();
}

fn duration_kind_name(value: NoteValue) -> &'static str {
    match value {
        NoteValue::Whole => "whole",
        NoteValue::Half => "half",
        NoteValue::Quarter => "quarter",
        NoteValue::Eighth => "eighth",
        NoteValue::Sixteenth => "16th",
        NoteValue::ThirtySecond => "32nd",
        NoteValue::SixtyFourth => "64th",
        NoteValue::HundredTwentyEighth => "128th",
        NoteValue::Other(_) => "quarter",
    }
}

fn write_note(out: &mut String, note: &Note, ties: &[SpannerEnd], transpose: Option<&Transpose>) {
    writeln!(out, "            <Note>").ok();
    if let Some(pitch) = note.pitch {
        // MuseScore's pitches end at MIDI 127: one above it (B♯9, octave
        // 10) goes down by octaves, keeping its spelling.
        let midi = pitch.midi();
        let midi = if midi > 127 {
            midi - (midi - 127 + 11) / 12 * 12
        } else {
            midi
        };
        writeln!(out, "              <pitch>{midi}</pitch>").ok();
        writeln!(out, "              <tpc>{}</tpc>", tpc_from_pitch(pitch)).ok();
        // Without a spellable written note MuseScore derives `<tpc2>` from
        // the transposition itself.
        if let Some(written) = transpose.and_then(|transpose| written_pitch(pitch, transpose)) {
            writeln!(
                out,
                "              <tpc2>{}</tpc2>",
                tpc_from_pitch(written)
            )
            .ok();
        }
    }
    if let Some(head) = note.notehead.and_then(notehead_name) {
        writeln!(out, "              <head>{head}</head>").ok();
    }
    if let Some(string) = note.string {
        // Convert back from 1-based to MuseScore's 0-based encoding.
        writeln!(
            out,
            "              <string>{}</string>",
            string.saturating_sub(1)
        )
        .ok();
    }
    if let Some(fret) = note.fret {
        writeln!(out, "              <fret>{fret}</fret>").ok();
    }
    for tie in ties {
        write_spanner(out, "              ", tie);
    }
    writeln!(out, "              </Note>").ok();
}

// ---------------------------------------------------------------------------
// Slurs
// ---------------------------------------------------------------------------

/// Each measure's length by its time signature (4/4 until the first).
fn nominal_lengths(timeline: &[MeasureDef]) -> Vec<u32> {
    let mut time = COMMON_TIME;
    timeline
        .iter()
        .map(|measure_def| {
            time = measure_def.time_signature.unwrap_or(time);
            measure_duration_ticks(time)
        })
        .collect()
}

/// Each measure's actual length (a pickup's or irregular measure's
/// differs from its time signature's), or its nominal one where the
/// timeline gives none or gives that one, rounded to its resolution.
/// `nominal` is [`nominal_lengths`]'s.
fn measure_lengths(timeline: &[MeasureDef], nominal: &[u32]) -> Vec<u32> {
    let mut time = COMMON_TIME;
    timeline
        .iter()
        .zip(nominal.iter().copied())
        .map(|(measure_def, nominal)| {
            time = measure_def.time_signature.unwrap_or(time);
            let resolution = u64::from(measure_def.tick_resolution);
            let nominal_there = time.measure_ticks(resolution);
            if nominal_there == Some(u64::from(measure_def.duration_ticks)) {
                return nominal;
            }
            let ticks = u64::from(measure_def.duration_ticks) * u64::from(DIVISIONS);
            ticks
                .checked_div(resolution)
                .and_then(|ticks| u32::try_from(ticks).ok())
                .filter(|&ticks| ticks > 0)
                .unwrap_or(nominal)
        })
        .collect()
}

/// MuseScore's time signature for a measure without one.
const COMMON_TIME: TimeSignature = TimeSignature {
    numerator: 4,
    denominator: 4,
};

/// A slur started but not yet ended while planning.
struct OpenSlur {
    voice_id: u8,
    slur_id: u8,
    at: BeatPosition,
    /// Its tick in its measure.
    tick: u32,
}

/// Pairs each slur end with its start, so each points at where the other
/// chord starts. An `End` or `EndStart` closes the open slur with its id
/// (in its own voice first, then any voice, then the voice's innermost
/// one whose own end never comes, as in an older save whose ends lack
/// their start's id); an `EndStart` then opens the next slur under the
/// same id. Nested and overlapping slurs stay apart this way.
/// Only chords carry slurs: a slur without an end, an end without a start,
/// or one touching a beat written as a `<Rest>` is dropped.
fn plan_slurs(beats: &[PlacedBeat]) -> HashMap<BeatPosition, Vec<SpannerEnd>> {
    let mut ends: HashMap<BeatPosition, Vec<SpannerEnd>> = HashMap::new();
    let mut open: Vec<OpenSlur> = Vec::new();
    // Slurs whose start was dropped (on a rest), by (voice, id): their end
    // is dropped too, rather than closing another slur.
    let mut dropped: Vec<(u8, u8)> = Vec::new();

    // The slur end a beat has, by its id.
    let end_id = |placed: &PlacedBeat| {
        placed
            .beat
            .slur
            .filter(|slur| matches!(slur.kind, SlurKind::End | SlurKind::EndStart))
            .map(|slur| slur.slur_id)
    };
    // Per slur id: its ends after the beat at hand (all, before the first),
    // counted down as the beats go by.
    let mut ends_after = [0u32; 256];
    for id in beats.iter().filter_map(end_id) {
        ends_after[usize::from(id)] += 1;
    }

    for placed in beats {
        if let Some(id) = end_id(placed) {
            ends_after[usize::from(id)] -= 1;
        }
        // Whether a slur with `slur_id` ends after this beat.
        let ends_later = |slur_id: u8| ends_after[usize::from(slur_id)] > 0;
        let here = placed.at;
        let (position, voice_idx, _) = here;
        let (beat, tick) = (placed.beat, placed.tick);
        let chord = !beat.is_staff_rest();
        let in_voice = |o: &OpenSlur| o.voice_id == placed.voice_id;
        let closes = match beat.slur {
            Some(slur) if matches!(slur.kind, SlurKind::End | SlurKind::EndStart) => open
                .iter()
                .rposition(|o| o.slur_id == slur.slur_id && in_voice(o))
                .or_else(|| open.iter().rposition(|o| o.slur_id == slur.slur_id))
                .or_else(|| {
                    let key = (placed.voice_id, slur.slur_id);
                    match dropped.iter().rposition(|&d| d == key) {
                        Some(index) => {
                            dropped.remove(index);
                            None
                        }
                        None => open
                            .iter()
                            .rposition(|o| in_voice(o) && !ends_later(o.slur_id)),
                    }
                }),
            _ => None,
        };
        if let Some(index) = closes
            && let start = open.remove(index)
            && chord
        {
            let measures = (position - start.at.0) as i64;
            let fractions = tick as i64 - start.tick as i64;
            let voices = voice_idx as i64 - start.at.1 as i64;
            let (from, to) = SpannerEnd::pair("Slur", None, measures, fractions, voices);
            ends.entry(start.at).or_default().push(from);
            ends.entry(here).or_default().push(to);
        }
        if let Some(slur) = beat.slur
            && matches!(slur.kind, SlurKind::Start | SlurKind::EndStart)
        {
            if chord {
                open.push(OpenSlur {
                    voice_id: placed.voice_id,
                    slur_id: slur.slur_id,
                    at: here,
                    tick,
                });
            } else {
                dropped.push((placed.voice_id, slur.slur_id));
            }
        }
    }
    ends
}

/// A beat with where it starts and ends in its measure.
struct PlacedBeat<'a> {
    at: BeatPosition,
    voice_id: u8,
    tick: u32,
    end_tick: u32,
    beat: &'a Beat,
}

/// Where a voice's beats start in their measure: right after the beat
/// before, or later at the beat's `tick_offset`, past a gap written as a
/// `<location>` (an earlier `tick_offset` is a tuplet's, which the writer
/// emits at its written length).
struct VoiceLayout {
    starts: Vec<u32>,
    /// The gap before each beat, after the one before (or the measure's
    /// start).
    gaps: Vec<u32>,
    /// Where its last beat ends; `None` without beats, when it's written as
    /// a whole-measure rest.
    end: Option<u32>,
}

fn voice_layout(voice: &Voice) -> VoiceLayout {
    let mut tick = 0u32;
    let (starts, gaps) = voice
        .beats
        .iter()
        .map(|beat| {
            let start = tick.max(beat.tick_offset);
            let gap = start - tick;
            tick = start.saturating_add(beat.duration.written_ticks(DIVISIONS));
            (start, gap)
        })
        .unzip();
    VoiceLayout {
        starts,
        gaps,
        end: (!voice.beats.is_empty()).then_some(tick),
    }
}

/// Every beat of a staff in time order (measure, tick, then voice), so a
/// spanner's start is seen before its end even when it runs back to an
/// earlier voice.
fn beats_in_time_order<'a>(
    measures: &[(Option<Clef>, Vec<&'a Voice>)],
    layouts: &[Vec<VoiceLayout>],
) -> Vec<PlacedBeat<'a>> {
    let mut placed = Vec::new();
    for (position, ((_, voices), layouts)) in measures.iter().zip(layouts).enumerate() {
        let first = placed.len();
        for (voice_idx, (voice, layout)) in voices.iter().zip(layouts).enumerate() {
            for (beat_idx, (beat, &tick)) in voice.beats.iter().zip(&layout.starts).enumerate() {
                placed.push(PlacedBeat {
                    at: (position, voice_idx, beat_idx),
                    voice_id: voice.voice_id,
                    tick,
                    end_tick: tick.saturating_add(beat.duration.written_ticks(DIVISIONS)),
                    beat,
                });
            }
        }
        placed[first..].sort_by_key(|beat| (beat.tick, beat.at.1));
    }
    placed
}

/// Pairs each tied note with the next note of its pitch in its voice that
/// is tied from the one before, locating each end from the other as
/// MuseScore does. A note of that pitch tied from nothing closes the open
/// tie unpaired. Of several open ties of a pitch (a unison on two
/// strings), a note closes the one on its string, else (if it's tied, or
/// either has no string) the first; never one in its own chord. A tie end without a start, or a start without an
/// end, is dropped (MuseScore can't place it).
fn plan_ties(beats: &[PlacedBeat]) -> HashMap<(BeatPosition, usize), Vec<SpannerEnd>> {
    // An open tie: its note, where its chord is, and its string.
    type Open = ((BeatPosition, usize), u32, Option<u8>);
    let mut ends: HashMap<(BeatPosition, usize), Vec<SpannerEnd>> = HashMap::new();
    // Per (voice, MIDI pitch): the open ties, first first.
    let mut open: HashMap<(u8, i32), Vec<Open>> = HashMap::new();
    for placed in beats.iter().filter(|placed| !placed.beat.is_staff_rest()) {
        // Opened after the whole chord, so none closes in its own chord.
        let mut opened: Vec<((u8, i32), Open)> = Vec::new();
        for (index, note) in placed.beat.notes.iter().enumerate() {
            let Some(pitch) = note.pitch else {
                continue;
            };
            let key = (placed.voice_id, pitch.midi());
            let ends_tie = note.tie.is_some_and(|tie| tie.ends_tie());
            let start = open.get_mut(&key).and_then(|ties| {
                let at = ties
                    .iter()
                    .position(|&(_, _, string)| string.is_some() && string == note.string)
                    .or_else(|| {
                        // An untied note closes no tie on another string.
                        ties.iter().position(|&(_, _, string)| {
                            ends_tie || string.is_none() || note.string.is_none()
                        })
                    })?;
                Some(ties.remove(at))
            });
            if let Some((start, tick, _)) = start.filter(|_| ends_tie) {
                let measures = (placed.at.0 - start.0.0) as i64;
                let fractions = placed.tick as i64 - tick as i64;
                let voices = placed.at.1 as i64 - start.0.1 as i64;
                let (from, to) = SpannerEnd::pair("Tie", None, measures, fractions, voices);
                ends.entry(start).or_default().push(from);
                ends.entry((placed.at, index)).or_default().push(to);
            }
            if note.tie.is_some_and(|tie| tie.starts_tie()) {
                opened.push((key, ((placed.at, index), placed.tick, note.string)));
            }
        }
        for (key, tie) in opened {
            open.entry(key).or_default().push(tie);
        }
    }
    ends
}

/// A spanner's `<next>` (from its start) or `<prev>` (from its end)
/// location of the other end, relative to its own.
fn write_location(out: &mut String, indent: &str, spanner: &SpannerEnd) {
    let SpannerEnd {
        start,
        measures,
        fractions,
        voices,
        ..
    } = *spanner;
    let direction = if start { "next" } else { "prev" };
    writeln!(out, "{indent}<{direction}>").ok();
    writeln!(out, "{indent}  <location>").ok();
    if voices != 0 {
        writeln!(out, "{indent}    <voices>{voices}</voices>").ok();
    }
    if measures != 0 {
        writeln!(out, "{indent}    <measures>{measures}</measures>").ok();
    }
    if fractions != 0 {
        writeln!(
            out,
            "{indent}    <fractions>{}</fractions>",
            fraction_text(fractions)
        )
        .ok();
    }
    writeln!(out, "{indent}    </location>").ok();
    writeln!(out, "{indent}  </{direction}>").ok();
}

// ---------------------------------------------------------------------------
// Hairpins and voltas
// ---------------------------------------------------------------------------

/// One end of a spanner (tie, slur, hairpin, volta), written as a
/// `<Spanner type="…">` where it is: the start points `next` to the end
/// and carries the spanner's own element (`body`, else an empty one), the
/// end points `prev` back to the start. MuseScore matches the two by their
/// relative `<location>`.
struct SpannerEnd {
    kind: &'static str,
    body: Option<String>,
    start: bool,
    measures: i64,
    /// Ticks between the two ends.
    fractions: i64,
    voices: i64,
}

impl SpannerEnd {
    /// A spanner's start and end, from where its end lies from its start.
    fn pair(
        kind: &'static str,
        body: Option<String>,
        measures: i64,
        fractions: i64,
        voices: i64,
    ) -> (Self, Self) {
        let start = SpannerEnd {
            kind,
            body,
            start: true,
            measures,
            fractions,
            voices,
        };
        let end = SpannerEnd {
            kind,
            body: None,
            start: false,
            measures: -measures,
            fractions: -fractions,
            voices: -voices,
        };
        (start, end)
    }
}

/// A spanner end with its `<Spanner>` at `indent`.
fn write_spanner(out: &mut String, indent: &str, spanner: &SpannerEnd) {
    let kind = spanner.kind;
    writeln!(out, "{indent}<Spanner type=\"{kind}\">").ok();
    if spanner.start {
        match &spanner.body {
            Some(body) => out.push_str(body),
            None => {
                writeln!(out, "{indent}  <{kind}>").ok();
                writeln!(out, "{indent}    </{kind}>").ok();
            }
        }
    }
    write_location(out, &format!("{indent}  "), spanner);
    writeln!(out, "{indent}  </Spanner>").ok();
}

/// A beat's hairpin starts (before it) and ends (after it).
#[derive(Default)]
struct BeatSpanners {
    before: Vec<SpannerEnd>,
    after: Vec<SpannerEnd>,
}

/// An open hairpin's start: its beat, the beat's tick and the subtype.
type OpenHairpin = (BeatPosition, u32, u8);

/// Pairs each hairpin's start and stop events by id. The start element
/// goes before its beat, the end element after the beat it stops on, each
/// locating the other: MuseScore matches them by those locations. A
/// hairpin without both ends is dropped.
fn plan_hairpins(beats: &[PlacedBeat]) -> HashMap<BeatPosition, BeatSpanners> {
    let mut planned: HashMap<BeatPosition, BeatSpanners> = HashMap::new();
    // A hairpin starts where its beat does and stops where its beat ends:
    // (measure, tick, order, beat). Stops go first at a tick, but a beat
    // without length stops after it starts.
    const STOP: u8 = 0;
    const START: u8 = 1;
    const STOP_UNTIMED: u8 = 2;
    let mut moments: Vec<(usize, u32, u8, usize)> = Vec::with_capacity(beats.len() * 2);
    for (index, placed) in beats.iter().enumerate() {
        let stop = if placed.end_tick > placed.tick {
            STOP
        } else {
            STOP_UNTIMED
        };
        moments.push((placed.at.0, placed.tick, START, index));
        moments.push((placed.at.0, placed.end_tick, stop, index));
    }
    moments.sort_by_key(|&(position, tick, order, _)| (position, tick, order));
    // (voice, id) → open hairpins' (beat, its tick, subtype), oldest first:
    // ids are numbered per voice and may repeat once they wrap.
    let mut open: HashMap<(u8, u8), VecDeque<OpenHairpin>> = HashMap::new();
    for (_, _, order, index) in moments {
        let starting = order == START;
        let placed = &beats[index];
        let here = placed.at;
        let hairpins = placed.beat.events.iter().filter_map(|event| match event {
            BeatEvent::Hairpin(hairpin) => Some(hairpin),
            _ => None,
        });
        for hairpin in hairpins {
            if starting {
                if let Some(subtype) = hairpin_subtype(hairpin) {
                    open.entry((placed.voice_id, hairpin.id))
                        .or_default()
                        .push_back((here, placed.tick, subtype));
                }
                continue;
            }
            if hairpin.kind != HairpinKind::Stop {
                continue;
            }
            // Its voice's, else one in another voice with its id.
            let key = Some((placed.voice_id, hairpin.id))
                .filter(|key| open.get(key).is_some_and(|queue| !queue.is_empty()))
                .or_else(|| {
                    open.iter()
                        .filter(|((_, id), queue)| *id == hairpin.id && !queue.is_empty())
                        .map(|(&key, _)| key)
                        .min()
                });
            let Some((at, start_tick, subtype)) = key
                .and_then(|key| open.get_mut(&key))
                .and_then(VecDeque::pop_front)
            else {
                continue;
            };
            let measures = (here.0 - at.0) as i64;
            let fractions = placed.end_tick as i64 - start_tick as i64;
            let voices = here.1 as i64 - at.1 as i64;
            let body = format!(
                "            <HairPin>\n              <subtype>{subtype}</subtype>\n              </HairPin>\n"
            );
            let (from, to) = SpannerEnd::pair("HairPin", Some(body), measures, fractions, voices);
            planned.entry(at).or_default().before.push(from);
            planned.entry(here).or_default().after.push(to);
        }
    }
    planned
}

type VoltaPlan = (
    HashMap<usize, Vec<SpannerEnd>>,
    HashMap<usize, Vec<SpannerEnd>>,
);

/// The timeline's endings as volta spanners in the first voice: the start
/// at the first measure's start, the end where [`write_voice_end`] puts it
/// in the last measure: its end, or the first voice's when that overfills
/// it.
fn plan_voltas(
    timeline: &[MeasureDef],
    layouts: &[Vec<VoiceLayout>],
    measure_ticks: &[u32],
) -> VoltaPlan {
    let mut starts: HashMap<usize, Vec<SpannerEnd>> = HashMap::new();
    let mut ends: HashMap<usize, Vec<SpannerEnd>> = HashMap::new();
    let lengths: Vec<i64> = measure_ticks
        .iter()
        .enumerate()
        .map(|(position, &measure_ticks)| {
            let first_voice = layouts
                .get(position)
                .and_then(|voices| voices.first())
                .and_then(|layout| layout.end);
            first_voice.map_or(measure_ticks, |voice| voice.max(measure_ticks)) as i64
        })
        .collect();

    for span in ending_spans(timeline) {
        let (start, position, closed) = (span.first, span.last, span.closed);
        let numbers: Vec<String> = span.numbers.iter().map(u8::to_string).collect();
        let numbers = numbers.join(", ");
        let text = span
            .text
            .map_or_else(|| format!("{numbers}."), str::to_string);
        let body = format!(
            "            <Volta>\n              <endHookType>{}</endHookType>\n              <beginText>{}</beginText>\n              <endings>{numbers}</endings>\n              </Volta>\n",
            u8::from(closed),
            xml_escape(&text)
        );
        let span = (position - start) as i64;
        let (from, to) = SpannerEnd::pair("Volta", Some(body), span, lengths[position], 0);
        starts.entry(start).or_default().push(from);
        ends.entry(position).or_default().push(to);
    }
    (starts, ends)
}

/// `ticks` as a reduced fraction of a whole note (`-3/16`).
fn fraction_text(ticks: i64) -> String {
    let (numerator, denominator) =
        crate::model::mscz::mscx::reduce_fraction(i128::from(ticks), i128::from(WHOLE_TICKS));
    format!("{numerator}/{denominator}")
}

// ---------------------------------------------------------------------------
// Clefs and noteheads
// ---------------------------------------------------------------------------

/// MuseScore's name for a clef type.
fn clef_name(clef: Clef) -> &'static str {
    match clef {
        Clef::Treble => "G",
        Clef::Bass => "F",
        Clef::Alto => "C3",
        Clef::Tenor => "C4",
        Clef::Percussion => "PERC",
        Clef::Tab => "TAB",
        Clef::Treble8vb => "G8vb",
        Clef::Treble8va => "G8va",
        Clef::Bass8vb => "F8vb",
        Clef::Bass8va => "F8va",
    }
}

/// The clef a staff shows in notation: none on a tab staff, and a `TAB`
/// clef only belongs on one.
fn notation_clef(staff: &StaffDef) -> Option<Clef> {
    (staff.display != StaffDisplay::Tab && staff.clef != Clef::Tab).then_some(staff.clef)
}

/// A notation staff's `<defaultClef>` (MuseScore assumes `G` without one).
fn default_clef(staff: &StaffDef) -> Option<&'static str> {
    notation_clef(staff)
        .filter(|&clef| clef != Clef::Treble)
        .map(clef_name)
}

/// MuseScore's notehead group for a notehead; `None` for the normal one
/// (and an invisible head, which MuseScore has no group for).
fn notehead_name(head: Notehead) -> Option<&'static str> {
    match head {
        Notehead::Normal | Notehead::None => None,
        Notehead::X => Some("cross"),
        Notehead::CircleX => Some("xcircle"),
        Notehead::Diamond => Some("diamond"),
        Notehead::Square => Some("la"),
        Notehead::Slash => Some("slash"),
        Notehead::Triangle => Some("triangle-up"),
        Notehead::ArrowUp | Notehead::ArrowDown => Some("large-arrow"),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn xml_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}
