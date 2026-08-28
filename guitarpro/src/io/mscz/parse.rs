//! MSCX XML parser.
//!
//! Streams the MSCX document with `quick-xml`'s event API and populates the
//! high-level [`Mscx`] view (see [`crate::model::mscz`]). The full XML is
//! kept on `Mscx::raw_xml` so higher-level converters and the write path can
//! reserialize it verbatim.

use quick_xml::Reader as XmlReader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event as XmlEvent};

use crate::error::{GpError, GpResult};
use crate::model::mscz::{
    Instrument, MetaTag, Mscx, MscxBeat, MscxBeatKind, MscxDuration, MscxDurationKind, MscxKeySig,
    MscxMeasure, MscxNote, MscxStaff, MscxTimeSig, MscxVoice, Part, Staff, StaffMeasureCount,
    StringData,
};

const SUPPORTED_MAJOR_MIN: u32 = 4;
const SUPPORTED_MAJOR_MAX: u32 = 4;

/// Parse an MSCX XML document into an [`Mscx`] view.
///
/// The full XML is copied onto `Mscx::raw_xml`; the structural fields are
/// best-effort extractors and safely absent (empty vec / `None`) when the
/// element is missing.
///
/// Rejects `<museScore version="X.Y">` outside the 4.x range with
/// [`GpError::MsczUnsupported`].
pub fn parse_mscx(xml: &str) -> GpResult<Mscx> {
    let mut reader = XmlReader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buffer = Vec::new();
    let mut version = String::new();
    let mut program_version: Option<String> = None;
    let mut program_revision: Option<String> = None;
    let mut division: Option<u32> = None;
    let mut meta_tags: Vec<MetaTag> = Vec::new();
    let mut parts: Vec<Part> = Vec::new();
    let mut score_staves: Vec<MscxStaff> = Vec::new();

    // A tiny state machine tracks whether we're inside `<Score>` (top-level
    // matters — the file has a nested `<Score>` shape). `depth_in_score`
    // counts open `<Score>` tags; anything > 0 means "inside the score".
    let mut depth_in_score = 0u32;
    // Nested `<Part>` context (only populated while parsing one).
    let mut current_part: Option<Part> = None;

    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|error| GpError::MsczXml(format!("read event: {error}")))?;

        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "museScore" => {
                version = attribute_value(&element, "version")?.unwrap_or_default();
                validate_version(&version)?;
            }

            XmlEvent::Start(element) if element.name().as_ref() == "Score" => {
                depth_in_score += 1;
            }
            XmlEvent::End(element) if element.name().as_ref() == "Score" => {
                depth_in_score = depth_in_score.saturating_sub(1);
            }

            XmlEvent::Start(element)
                if depth_in_score > 0
                    && element.name().as_ref() == "Staff"
                    // Only the top-level Staff (Score > Staff) — not Part > Staff
                    && current_part.is_none() =>
            {
                let staff_id = attribute_value(&element, "id")?.unwrap_or_default();
                drop(element);
                let staff = parse_score_staff(&mut reader, &mut buffer, staff_id)?;
                score_staves.push(staff);
            }

            XmlEvent::Start(element) if element.name().as_ref() == "programVersion" => {
                program_version = Some(read_text(&mut reader, &mut buffer, "programVersion")?);
            }
            XmlEvent::Start(element) if element.name().as_ref() == "programRevision" => {
                program_revision = Some(read_text(&mut reader, &mut buffer, "programRevision")?);
            }
            XmlEvent::Start(element)
                if depth_in_score > 0 && element.name().as_ref() == "Division" =>
            {
                let text = read_text(&mut reader, &mut buffer, "Division")?;
                division = text.trim().parse::<u32>().ok();
            }
            XmlEvent::Start(element)
                if depth_in_score > 0 && element.name().as_ref() == "metaTag" =>
            {
                let name = attribute_value(&element, "name")?.unwrap_or_default();
                let value = read_text(&mut reader, &mut buffer, "metaTag")?;
                meta_tags.push(MetaTag { name, value });
            }
            XmlEvent::Start(element) if depth_in_score > 0 && element.name().as_ref() == "Part" => {
                let id = attribute_value(&element, "id")?.unwrap_or_default();
                current_part = Some(Part {
                    id,
                    track_name: None,
                    staves: Vec::new(),
                    instrument: None,
                });
            }
            XmlEvent::End(element) if depth_in_score > 0 && element.name().as_ref() == "Part" => {
                if let Some(part) = current_part.take() {
                    parts.push(part);
                }
            }

            // Nested elements inside a `<Part>` — the parser only enters this
            // branch while `current_part` is `Some`.
            XmlEvent::Start(element)
                if current_part.is_some() && element.name().as_ref() == "trackName" =>
            {
                let text = read_text(&mut reader, &mut buffer, "trackName")?;
                if let Some(part) = current_part.as_mut() {
                    part.track_name = Some(text);
                }
            }
            XmlEvent::Start(element)
                if current_part.is_some() && element.name().as_ref() == "Staff" =>
            {
                let id = attribute_value(&element, "id")?.unwrap_or_default();
                drop(element);
                let staff = parse_part_staff(&mut reader, &mut buffer, id)?;
                if let Some(part) = current_part.as_mut() {
                    part.staves.push(staff);
                }
            }
            XmlEvent::Start(element)
                if current_part.is_some() && element.name().as_ref() == "Instrument" =>
            {
                let id = attribute_value(&element, "id")?.unwrap_or_default();
                drop(element);
                let instrument = parse_instrument(&mut reader, &mut buffer, id)?;
                if let Some(part) = current_part.as_mut() {
                    part.instrument = Some(instrument);
                }
            }

            XmlEvent::Eof => break,
            _ => {}
        }

        buffer.clear();
    }

    // Sanity: if `<museScore>` was never seen this isn't an MSCX file.
    if version.is_empty() {
        return Err(GpError::MsczXml(
            "missing <museScore> root element".to_string(),
        ));
    }

    let measure_counts: Vec<StaffMeasureCount> = score_staves
        .iter()
        .map(|staff| StaffMeasureCount {
            staff_id: staff.staff_id.clone(),
            measure_count: staff.measures.len() as u32,
        })
        .collect();

    Ok(Mscx {
        raw_xml: xml.to_string(),
        version,
        program_version,
        program_revision,
        division,
        meta_tags,
        parts,
        measure_counts,
        score_staves,
    })
}

/// Serialize an [`Mscx`] view back to XML.
///
/// Part 1 keeps the raw XML as the source of truth, so this returns the
/// preserved string verbatim. Once the AST covers enough fields to modify
/// structural content, this function will emit a regenerated tree.
pub fn write_mscx(mscx: &Mscx) -> String {
    mscx.raw_xml.clone()
}

// ---------------------------------------------------------------------------
// Part-level sub-parsers (unchanged from Part 1)
// ---------------------------------------------------------------------------

fn parse_part_staff(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
    id: String,
) -> GpResult<Staff> {
    let mut staff = Staff {
        id,
        group: None,
        type_name: None,
        default_clef: None,
    };

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Staff read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "StaffType" => {
                staff.group = attribute_value(&element, "group")?;
            }
            XmlEvent::Start(element) if element.name().as_ref() == "name" => {
                staff.type_name = Some(read_text(reader, buffer, "StaffType/name")?);
            }
            XmlEvent::Start(element) if element.name().as_ref() == "defaultClef" => {
                staff.default_clef = Some(read_text(reader, buffer, "defaultClef")?);
            }
            XmlEvent::End(element) if element.name().as_ref() == "Staff" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(staff)
}

fn parse_instrument(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
    id: String,
) -> GpResult<Instrument> {
    let mut instrument = Instrument {
        id,
        long_name: None,
        short_name: None,
        track_name: None,
        transpose_diatonic: None,
        transpose_chromatic: None,
        instrument_id: None,
        string_data: None,
    };

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Instrument read: {error}")))?;
        match event {
            XmlEvent::Start(element) => match element.name().as_ref() {
                "longName" => {
                    instrument.long_name = Some(read_text(reader, buffer, "longName")?);
                }
                "shortName" => {
                    instrument.short_name = Some(read_text(reader, buffer, "shortName")?);
                }
                "trackName" => {
                    instrument.track_name = Some(read_text(reader, buffer, "trackName")?);
                }
                "transposeDiatonic" => {
                    instrument.transpose_diatonic =
                        parse_i8(&read_text(reader, buffer, "transposeDiatonic")?);
                }
                "transposeChromatic" => {
                    instrument.transpose_chromatic =
                        parse_i8(&read_text(reader, buffer, "transposeChromatic")?);
                }
                "instrumentId" => {
                    instrument.instrument_id = Some(read_text(reader, buffer, "instrumentId")?);
                }
                "StringData" => {
                    instrument.string_data = Some(parse_string_data(reader, buffer)?);
                }
                _ => {}
            },
            XmlEvent::End(element) if element.name().as_ref() == "Instrument" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(instrument)
}

fn parse_string_data(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<StringData> {
    let mut data = StringData {
        frets: None,
        strings: Vec::new(),
    };

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("StringData read: {error}")))?;
        match event {
            XmlEvent::Start(element) => match element.name().as_ref() {
                "frets" => {
                    let text = read_text(reader, buffer, "frets")?;
                    data.frets = text.trim().parse::<u8>().ok();
                }
                "string" => {
                    let text = read_text(reader, buffer, "string")?;
                    if let Ok(value) = text.trim().parse::<u8>() {
                        data.strings.push(value);
                    }
                }
                _ => {}
            },
            XmlEvent::End(element) if element.name().as_ref() == "StringData" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(data)
}

// ---------------------------------------------------------------------------
// Score body sub-parsers (Part 2)
// ---------------------------------------------------------------------------

fn parse_score_staff(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
    staff_id: String,
) -> GpResult<MscxStaff> {
    let mut staff = MscxStaff {
        staff_id,
        measures: Vec::new(),
    };

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Score/Staff read: {error}")))?;
        match event {
            XmlEvent::Empty(element) if element.name().as_ref() == "Measure" => {
                let len = attribute_value(&element, "len")?;
                staff.measures.push(MscxMeasure {
                    len,
                    ..MscxMeasure::default()
                });
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Measure" => {
                let len = attribute_value(&element, "len")?;
                drop(element);
                let measure = parse_measure(reader, buffer, len)?;
                staff.measures.push(measure);
            }
            XmlEvent::End(element) if element.name().as_ref() == "Staff" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(staff)
}

fn parse_measure(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
    len: Option<String>,
) -> GpResult<MscxMeasure> {
    let mut measure = MscxMeasure {
        len,
        ..MscxMeasure::default()
    };

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Measure read: {error}")))?;
        match event {
            XmlEvent::Empty(element) if element.name().as_ref() == "startRepeat" => {
                measure.start_repeat = true;
            }
            XmlEvent::Start(element) if element.name().as_ref() == "endRepeat" => {
                let text = read_text(reader, buffer, "endRepeat")?;
                measure.end_repeat = text.trim().parse::<u8>().ok().or(Some(2));
            }
            XmlEvent::Empty(element) if element.name().as_ref() == "endRepeat" => {
                measure.end_repeat = Some(2);
            }
            XmlEvent::Start(element) if element.name().as_ref() == "voice" => {
                let voice = parse_voice(reader, buffer, &mut measure)?;
                measure.voices.push(voice);
            }
            XmlEvent::End(element) if element.name().as_ref() == "Measure" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(measure)
}

fn parse_voice(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
    measure: &mut MscxMeasure,
) -> GpResult<MscxVoice> {
    let mut voice = MscxVoice::default();

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("voice read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "TimeSig" => {
                let sig = parse_time_sig(reader, buffer)?;
                if measure.time_sig.is_none() {
                    measure.time_sig = Some(sig);
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "KeySig" => {
                let sig = parse_key_sig(reader, buffer)?;
                if measure.key_sig.is_none() {
                    measure.key_sig = Some(sig);
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Tempo" => {
                let tempo = parse_tempo(reader, buffer)?;
                if measure.tempo_bps.is_none() {
                    measure.tempo_bps = tempo;
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Chord" => {
                let beat = parse_chord(reader, buffer)?;
                voice.beats.push(beat);
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Rest" => {
                let beat = parse_rest(reader, buffer)?;
                voice.beats.push(beat);
            }
            XmlEvent::End(element) if element.name().as_ref() == "voice" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(voice)
}

fn parse_time_sig(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<MscxTimeSig> {
    let mut numerator = 4u8;
    let mut denominator = 4u8;

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("TimeSig read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "sigN" => {
                let text = read_text(reader, buffer, "sigN")?;
                if let Ok(value) = text.trim().parse::<u8>() {
                    numerator = value;
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "sigD" => {
                let text = read_text(reader, buffer, "sigD")?;
                if let Ok(value) = text.trim().parse::<u8>() {
                    denominator = value;
                }
            }
            XmlEvent::End(element) if element.name().as_ref() == "TimeSig" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(MscxTimeSig {
        numerator,
        denominator,
    })
}

fn parse_key_sig(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<MscxKeySig> {
    let mut fifths: i8 = 0;

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("KeySig read: {error}")))?;
        match event {
            XmlEvent::Start(element)
                if matches!(element.name().as_ref(), "accidental" | "concertKey") =>
            {
                let context = if element.name().as_ref() == "accidental" {
                    "KeySig/accidental"
                } else {
                    "KeySig/concertKey"
                };
                let text = read_text(reader, buffer, context)?;
                if let Ok(value) = text.trim().parse::<i8>() {
                    fifths = value;
                }
            }
            XmlEvent::End(element) if element.name().as_ref() == "KeySig" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(MscxKeySig { fifths })
}

fn parse_tempo(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<Option<f32>> {
    let mut bps: Option<f32> = None;

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Tempo read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "tempo" => {
                let text = read_text(reader, buffer, "tempo")?;
                bps = text.trim().parse::<f32>().ok();
            }
            XmlEvent::End(element) if element.name().as_ref() == "Tempo" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(bps)
}

fn parse_chord(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<MscxBeat> {
    let mut duration = MscxDuration {
        kind: MscxDurationKind::Quarter,
        dots: 0,
    };
    let mut notes: Vec<MscxNote> = Vec::new();

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Chord read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "durationType" => {
                let text = read_text(reader, buffer, "Chord/durationType")?;
                duration.kind = parse_duration_kind(text.trim());
            }
            XmlEvent::Start(element) if element.name().as_ref() == "dots" => {
                let text = read_text(reader, buffer, "Chord/dots")?;
                if let Ok(value) = text.trim().parse::<u8>() {
                    duration.dots = value;
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Note" => {
                notes.push(parse_note(reader, buffer)?);
            }
            XmlEvent::End(element) if element.name().as_ref() == "Chord" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(MscxBeat {
        duration,
        kind: MscxBeatKind::Chord(notes),
    })
}

fn parse_rest(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<MscxBeat> {
    let mut duration = MscxDuration {
        kind: MscxDurationKind::Measure,
        dots: 0,
    };

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Rest read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "durationType" => {
                let text = read_text(reader, buffer, "Rest/durationType")?;
                duration.kind = parse_duration_kind(text.trim());
            }
            XmlEvent::Start(element) if element.name().as_ref() == "dots" => {
                let text = read_text(reader, buffer, "Rest/dots")?;
                if let Ok(value) = text.trim().parse::<u8>() {
                    duration.dots = value;
                }
            }
            XmlEvent::End(element) if element.name().as_ref() == "Rest" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(MscxBeat {
        duration,
        kind: MscxBeatKind::Rest,
    })
}

fn parse_note(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<MscxNote> {
    let mut note = MscxNote {
        pitch: None,
        tpc: None,
        string: None,
        fret: None,
        tie_start: false,
        tie_end: false,
    };

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Note read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "pitch" => {
                let text = read_text(reader, buffer, "Note/pitch")?;
                note.pitch = text.trim().parse::<u8>().ok();
            }
            XmlEvent::Start(element) if element.name().as_ref() == "tpc" => {
                let text = read_text(reader, buffer, "Note/tpc")?;
                note.tpc = text.trim().parse::<i8>().ok();
            }
            XmlEvent::Start(element) if element.name().as_ref() == "string" => {
                let text = read_text(reader, buffer, "Note/string")?;
                note.string = text.trim().parse::<u8>().ok();
            }
            XmlEvent::Start(element) if element.name().as_ref() == "fret" => {
                let text = read_text(reader, buffer, "Note/fret")?;
                note.fret = text.trim().parse::<u8>().ok();
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Spanner" => {
                // `<Spanner type="Tie">` markers appear as siblings of the note's
                // pitch data. We only care about detecting Tie start/end here;
                // the actual span target is not required for the LoadedScore
                // representation.
                let spanner_type = attribute_value(&element, "type")?.unwrap_or_default();
                drop(element);
                let (has_prev, _has_next) = scan_spanner(reader, buffer)?;
                if spanner_type == "Tie" {
                    if has_prev {
                        note.tie_end = true;
                    } else {
                        note.tie_start = true;
                    }
                }
            }
            XmlEvent::End(element) if element.name().as_ref() == "Note" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(note)
}

/// Consume a `<Spanner>` block, detecting `<prev>` / `<next>` siblings and
/// returning `(has_prev, has_next)`. The presence of `<prev>` means the tie
/// terminates on the current note (this is an End); `<next>` means the tie
/// begins here (Start).
fn scan_spanner(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<(bool, bool)> {
    let mut has_prev = false;
    let mut has_next = false;

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Spanner read: {error}")))?;
        match event {
            XmlEvent::Empty(element) | XmlEvent::Start(element) => match element.name().as_ref() {
                "prev" => has_prev = true,
                "next" => has_next = true,
                _ => {}
            },
            XmlEvent::End(element) if element.name().as_ref() == "Spanner" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok((has_prev, has_next))
}

fn parse_duration_kind(text: &str) -> MscxDurationKind {
    match text {
        "whole" => MscxDurationKind::Whole,
        "half" => MscxDurationKind::Half,
        "quarter" => MscxDurationKind::Quarter,
        "eighth" => MscxDurationKind::Eighth,
        "16th" => MscxDurationKind::Sixteenth,
        "32nd" => MscxDurationKind::ThirtySecond,
        "64th" => MscxDurationKind::SixtyFourth,
        "128th" => MscxDurationKind::HundredTwentyEighth,
        "measure" => MscxDurationKind::Measure,
        _ => MscxDurationKind::Quarter,
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn attribute_value(element: &BytesStart<'_>, key: &str) -> GpResult<Option<String>> {
    for attribute in element.attributes() {
        let attribute =
            attribute.map_err(|error| GpError::MsczXml(format!("attribute: {error}")))?;
        if attribute.key.as_ref() == key {
            let value = unescape(attribute.value.as_ref())
                .map_err(|error| GpError::MsczXml(format!("attribute unescape: {error}")))?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

/// Read the textual content of the currently-open element up to its
/// matching close tag. Nested elements are ignored (their text is skipped).
fn read_text(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
    context: &'static str,
) -> GpResult<String> {
    let mut output = String::new();
    let mut depth = 1u32;
    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("read text ({context}): {error}")))?;
        match event {
            XmlEvent::Text(text) => {
                let unescaped = unescape(text.as_ref())
                    .map_err(|error| GpError::MsczXml(format!("text unescape: {error}")))?;
                output.push_str(&unescaped);
            }
            XmlEvent::GeneralRef(reference) => {
                if let Some(character) = reference
                    .resolve_char_ref()
                    .map_err(|error| GpError::MsczXml(format!("entity ref: {error}")))?
                {
                    output.push(character);
                    continue;
                }
                let name = reference.as_ref();
                let resolved = match name {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    other => {
                        return Err(GpError::MsczXml(format!("unknown entity &{other};")));
                    }
                };
                output.push_str(resolved);
            }
            XmlEvent::CData(text) => {
                output.push_str(text.as_ref());
            }
            XmlEvent::Start(_) => depth += 1,
            XmlEvent::End(_) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(output)
}

fn parse_i8(text: &str) -> Option<i8> {
    text.trim().parse::<i8>().ok()
}

fn validate_version(version: &str) -> GpResult<()> {
    let major_str = version.split('.').next().unwrap_or("");
    let major = major_str
        .parse::<u32>()
        .map_err(|_| GpError::MsczUnsupported {
            got: version.to_string(),
            supported: "4.x",
        })?;
    if !(SUPPORTED_MAJOR_MIN..=SUPPORTED_MAJOR_MAX).contains(&major) {
        return Err(GpError::MsczUnsupported {
            got: version.to_string(),
            supported: "4.x",
        });
    }
    Ok(())
}
