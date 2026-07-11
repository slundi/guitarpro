//! [`LoadedScore`] → MSCX conversion.
//!
//! The generated MSCX targets MuseScore 4.10 (`<museScore version="4.10">`)
//! and covers the same subset that [`super::to_optimized`] handles:
//! metadata, instruments/staves, timeline (per-measure time/key/tempo),
//! per-track measure content (voices → beats → notes with pitch/string/fret
//! and ties). Everything else is intentionally left out — the caller can
//! diff the resulting XML against the input to see which structural
//! features were dropped.

use std::fmt::Write as _;

use crate::model::mscz::Mscx;
use crate::model::optimized::{
    LoadedScore,
    beat::{Beat, Duration, Voice},
    global::{Instrument, InstrumentKind},
    metadata::{Metadata, TimeSignature},
    note::{Note, NoteValue, Pitch, PitchStep, TieType},
    timeline::{JumpKind, MeasureDef},
    track::{StaffDef, StaffDisplay, Track},
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
        let instrument = score.score.instruments.get(track.instrument.0 as usize);
        let staff_def = track
            .staves
            .first()
            .and_then(|id| score.score.staves.get(id.0 as usize));
        write_part(out, index, track, instrument, staff_def);
    }
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
        let group = match staff.display {
            StaffDisplay::Tab => "tablature",
            StaffDisplay::Notation | StaffDisplay::NotationTab => "pitched",
        };
        writeln!(out, "        <StaffType group=\"{group}\">").ok();
        writeln!(
            out,
            "          <name>{}</name>",
            if group == "tablature" {
                "tab6StrCommon"
            } else {
                "stdNormal"
            }
        )
        .ok();
        writeln!(out, "          </StaffType>").ok();
    }
    writeln!(out, "        </Staff>").ok();
    writeln!(
        out,
        "      <trackName>{}</trackName>",
        xml_escape(&track.name)
    )
    .ok();

    if let Some(instr) = instrument {
        writeln!(
            out,
            "      <Instrument id=\"{}\">",
            xml_escape(instr.instrument_sound.as_deref().unwrap_or("guitar"))
        )
        .ok();
        writeln!(
            out,
            "        <longName>{}</longName>",
            xml_escape(&instr.name)
        )
        .ok();
        if let Some(abbrev) = instr.abbreviation.as_deref() {
            writeln!(out, "        <shortName>{}</shortName>", xml_escape(abbrev)).ok();
        }
        if let Some(sound) = instr.instrument_sound.as_deref() {
            writeln!(
                out,
                "        <instrumentId>{}</instrumentId>",
                xml_escape(sound)
            )
            .ok();
        }
        if let Some(transpose) = instr.transpose.as_ref() {
            writeln!(
                out,
                "        <transposeChromatic>{}</transposeChromatic>",
                transpose.chromatic
            )
            .ok();
            if let Some(diatonic) = transpose.diatonic {
                writeln!(
                    out,
                    "        <transposeDiatonic>{}</transposeDiatonic>",
                    diatonic
                )
                .ok();
            }
        }
        if let InstrumentKind::Stringed { tuning, .. } = &instr.kind {
            writeln!(out, "        <StringData>").ok();
            writeln!(out, "          <frets>24</frets>").ok();
            for pitch in tuning {
                let midi = pitch_to_midi(*pitch);
                writeln!(out, "          <string>{midi}</string>").ok();
            }
            writeln!(out, "          </StringData>").ok();
        }
        writeln!(out, "        </Instrument>").ok();
    }

    writeln!(out, "      </Part>").ok();
}

// ---------------------------------------------------------------------------
// Master staves (measure content per track)
// ---------------------------------------------------------------------------

fn write_master_staves(out: &mut String, score: &LoadedScore) {
    for (track_idx, track) in score.score.tracks.iter().enumerate() {
        writeln!(out, "    <Staff id=\"{}\">", track_idx + 1).ok();
        write_measures(out, track, &score.score.timeline);
        writeln!(out, "      </Staff>").ok();
    }
}

fn write_measures(out: &mut String, track: &Track, timeline: &[MeasureDef]) {
    for measure_def in timeline {
        let idx = measure_def.index;
        writeln!(out, "      <Measure>").ok();

        if measure_def
            .navigation
            .iter()
            .any(|ev| ev.kind == JumpKind::RepeatOpen)
        {
            writeln!(out, "        <startRepeat/>").ok();
        }
        if let Some(count) = measure_def
            .navigation
            .iter()
            .find(|ev| ev.kind == JumpKind::RepeatClose)
            .and_then(|ev| ev.repeat_count)
        {
            writeln!(out, "        <endRepeat>{count}</endRepeat>").ok();
        }

        // Emit each voice; if the measure has no voices, produce an empty
        // whole-rest voice so the output remains a valid MSCX document.
        let measure_data = track.measures.get(&idx);
        let voices: Vec<&Voice> = measure_data
            .map(|data| {
                let mut list: Vec<(u8, &Voice)> =
                    data.voices.iter().map(|(k, v)| (*k, v)).collect();
                list.sort_by_key(|(id, _)| *id);
                list.into_iter().map(|(_, voice)| voice).collect()
            })
            .unwrap_or_default();

        if voices.is_empty() {
            write_empty_voice(out, measure_def, idx == measure_def.index);
        } else {
            for (voice_idx, voice) in voices.iter().enumerate() {
                write_voice(out, voice, measure_def, voice_idx == 0);
            }
        }

        writeln!(out, "        </Measure>").ok();
    }
}

fn write_empty_voice(out: &mut String, measure_def: &MeasureDef, primary: bool) {
    writeln!(out, "        <voice>").ok();
    if primary {
        write_measure_signatures(out, measure_def);
    }
    writeln!(
        out,
        "          <Rest><durationType>measure</durationType></Rest>"
    )
    .ok();
    writeln!(out, "          </voice>").ok();
}

fn write_voice(out: &mut String, voice: &Voice, measure_def: &MeasureDef, primary: bool) {
    writeln!(out, "        <voice>").ok();
    if primary {
        write_measure_signatures(out, measure_def);
    }
    if voice.beats.is_empty() {
        writeln!(
            out,
            "          <Rest><durationType>measure</durationType></Rest>"
        )
        .ok();
    } else {
        for beat in &voice.beats {
            write_beat(out, beat);
        }
    }
    writeln!(out, "          </voice>").ok();
}

fn write_measure_signatures(out: &mut String, measure_def: &MeasureDef) {
    if let Some(sig) = measure_def.time_signature {
        writeln!(out, "          <TimeSig>").ok();
        writeln!(out, "            <sigN>{}</sigN>", sig.numerator).ok();
        writeln!(out, "            <sigD>{}</sigD>", sig.denominator).ok();
        writeln!(out, "            </TimeSig>").ok();
    }
    if let Some(key) = measure_def.key_signature {
        let fifths = fifths_from_root(key.root);
        writeln!(out, "          <KeySig>").ok();
        writeln!(out, "            <accidental>{fifths}</accidental>").ok();
        writeln!(out, "            </KeySig>").ok();
    }
    if let Some(tempo) = measure_def.tempo {
        let bps = tempo / 60.0;
        writeln!(out, "          <Tempo>").ok();
        writeln!(out, "            <tempo>{bps}</tempo>").ok();
        writeln!(out, "            </Tempo>").ok();
    }
}

fn write_beat(out: &mut String, beat: &Beat) {
    if beat.gp_rest || beat.notes.is_empty() {
        writeln!(out, "          <Rest>").ok();
        write_duration(out, &beat.duration);
        writeln!(out, "            </Rest>").ok();
    } else {
        writeln!(out, "          <Chord>").ok();
        write_duration(out, &beat.duration);
        for note in &beat.notes {
            write_note(out, note);
        }
        writeln!(out, "            </Chord>").ok();
    }
}

fn write_duration(out: &mut String, duration: &Duration) {
    let kind = duration_kind_name(duration.base);
    writeln!(out, "            <durationType>{kind}</durationType>").ok();
    if duration.dots > 0 {
        writeln!(out, "            <dots>{}</dots>", duration.dots).ok();
    }
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

fn write_note(out: &mut String, note: &Note) {
    writeln!(out, "            <Note>").ok();
    if let Some(pitch) = note.pitch {
        writeln!(out, "              <pitch>{}</pitch>", pitch_to_midi(pitch)).ok();
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
    match note.tie {
        Some(TieType::Start) => {
            writeln!(out, "              <Spanner type=\"Tie\"><next/></Spanner>").ok();
        }
        Some(TieType::End) => {
            writeln!(out, "              <Spanner type=\"Tie\"><prev/></Spanner>").ok();
        }
        None => {}
    }
    writeln!(out, "              </Note>").ok();
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

fn pitch_to_midi(pitch: Pitch) -> i16 {
    let step_offset = match pitch.step {
        PitchStep::C => 0,
        PitchStep::D => 2,
        PitchStep::E => 4,
        PitchStep::F => 5,
        PitchStep::G => 7,
        PitchStep::A => 9,
        PitchStep::B => 11,
    };
    (pitch.octave as i16 + 1) * 12 + step_offset + pitch.alter as i16
}

/// Inverse of [`super::to_optimized::root_from_fifths`] — recover the signed
/// fifth count from a major-key tonic pitch. Returns 0 for unknown pitches.
fn fifths_from_root(root: Pitch) -> i8 {
    match (root.step, root.alter) {
        (PitchStep::C, -1) => -7,
        (PitchStep::G, -1) => -6,
        (PitchStep::D, -1) => -5,
        (PitchStep::A, -1) => -4,
        (PitchStep::E, -1) => -3,
        (PitchStep::B, -1) => -2,
        (PitchStep::F, 0) => -1,
        (PitchStep::C, 0) => 0,
        (PitchStep::G, 0) => 1,
        (PitchStep::D, 0) => 2,
        (PitchStep::A, 0) => 3,
        (PitchStep::E, 0) => 4,
        (PitchStep::B, 0) => 5,
        (PitchStep::F, 1) => 6,
        (PitchStep::C, 1) => 7,
        _ => 0,
    }
}

// TimeSignature is imported for documentation clarity even though the
// current writer only relies on numerator/denominator, so keep an anchor.
const _: fn(TimeSignature) = |_| ();
