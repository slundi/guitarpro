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
use crate::model::mscz::mscx::dynamic_glyph_letters;
use crate::model::mscz::{
    Instrument, MetaTag, Mscx, MscxBarLine, MscxBeat, MscxBeatKind, MscxChordMark, MscxDuration,
    MscxDurationKind, MscxDynamic, MscxGrace, MscxHarmony, MscxJump, MscxKeyMode, MscxKeySig,
    MscxLocation, MscxMarker, MscxMeasure, MscxMeasureMark, MscxNote, MscxSegment, MscxSlurEnd,
    MscxStaff, MscxTimeSig, MscxVoice, MscxVolta, MscxVoltaMark, Part, Staff, StaffMeasureCount,
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
        program: None,
        string_data: None,
        clefs: Vec::new(),
    };
    // Only the first `<Channel>`'s program counts: later ones are
    // alternative sounds (pizzicato, mute…).
    let mut channels = 0u32;

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
                "Channel" => channels += 1,
                "program" if channels == 1 => {
                    instrument.program = parse_program(&element)?;
                }
                "clef" | "concertClef" => {
                    let staff = attribute_value(&element, "staff")?
                        .and_then(|staff| staff.trim().parse::<usize>().ok())
                        .unwrap_or(1)
                        .saturating_sub(1);
                    let clef = read_text(reader, buffer, "Instrument/clef")?;
                    let clef = clef.trim();
                    if !clef.is_empty() && !instrument.clefs.iter().any(|(s, _)| *s == staff) {
                        instrument.clefs.push((staff, clef.to_string()));
                    }
                }
                _ => {}
            },
            XmlEvent::Empty(element) => match element.name().as_ref() {
                "Channel" => channels += 1,
                "program" if channels == 1 => {
                    instrument.program = parse_program(&element)?;
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

/// `<program value="…"/>` — a GM program, 0-based.
fn parse_program(element: &BytesStart) -> GpResult<Option<u8>> {
    Ok(attribute_value(element, "value")?.and_then(|value| value.trim().parse::<u8>().ok()))
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
                let text = text.trim();
                // An empty one is the default; one of 0, too large or not
                // a number is kept as close as it gets (a repeat plays at
                // least once), and noted.
                let (count, lossy) = match text.parse::<u32>() {
                    Ok(0) => (1, true),
                    Ok(count) => {
                        u8::try_from(count).map_or((u8::MAX, true), |count| (count, false))
                    }
                    Err(_) => (2, !text.is_empty()),
                };
                measure.end_repeat = Some(count);
                measure.end_repeat_lossy = lossy;
            }
            XmlEvent::Empty(element) if element.name().as_ref() == "endRepeat" => {
                measure.end_repeat = Some(2);
            }
            XmlEvent::Start(element) if element.name().as_ref() == "voice" => {
                let voice = parse_voice(reader, buffer, &mut measure)?;
                measure.voices.push(voice);
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Jump" => {
                let fields = read_fields(reader, buffer, "Jump")?;
                measure.marks.push(MscxMeasureMark::Jump(MscxJump {
                    text: owned_child_text(&fields, "text"),
                    jump_to: owned_child_text(&fields, "jumpTo"),
                    play_until: owned_child_text(&fields, "playUntil"),
                    continue_at: owned_child_text(&fields, "continueAt"),
                }));
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Marker" => {
                let fields = read_fields(reader, buffer, "Marker")?;
                measure.marks.push(MscxMeasureMark::Marker(MscxMarker {
                    label: owned_child_text(&fields, "label"),
                    text: owned_child_text(&fields, "text"),
                }));
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
    // Clefs only count in the measure's first voice.
    let first_voice = measure.voices.is_empty();
    // A clef after a beat: the measure's end clef unless another beat
    // follows it.
    let mut pending_clef: Option<String> = None;
    // `<location>` gaps since the last beat, added up.
    let mut pending_gap: Option<(i32, i32)> = None;
    // Segment elements before the next beat (dynamics, chord symbols...).
    let mut pending = MscxSegment::default();

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
            XmlEvent::Start(element) if element.name().as_ref() == "Clef" => {
                let clef = parse_clef(reader, buffer)?;
                if first_voice && clef.is_some() {
                    // After a gap it's already in mid-measure.
                    if voice.beats.is_empty() && pending_gap.is_none() {
                        // One followed by another holds for no beat.
                        if let Some(superseded) = measure.start_clef.take() {
                            measure.superseded_clefs.push(superseded);
                        }
                        measure.start_clef = clef;
                    } else {
                        // One followed by another before the next beat
                        // holds for no beat: a change in mid-measure.
                        measure.mid_clefs.extend(pending_clef.take());
                        pending_clef = clef;
                    }
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "location" => {
                if let Some(gap) = parse_voice_location(reader, buffer)? {
                    // What came before the gap is at its start, not at the
                    // beat after it.
                    if pending != MscxSegment::default() {
                        voice.before_gaps.push(std::mem::take(&mut pending));
                    }
                    // A sum too large for a fraction keeps the gaps so far.
                    pending_gap =
                        Some(pending_gap.map_or(gap, |sum| add_fractions(sum, gap).unwrap_or(sum)));
                }
            }
            // Voice-level spanners: hairpins and voltas are read, others
            // (with `<location>`s of their own) skipped whole.
            XmlEvent::Start(element) if element.name().as_ref() == "Spanner" => {
                let spanner_type = attribute_value(&element, "type")?.unwrap_or_default();
                drop(element);
                let fields = read_fields(reader, buffer, "Spanner")?;
                let is_end = SpannerEnds::of(&fields).has_prev;
                match spanner_type.as_str() {
                    "HairPin" if fields.iter().any(|f| f.name == "HairPin") => {
                        let subtype = descendant_text(&fields, "subtype")
                            .and_then(|t| t.parse().ok())
                            .unwrap_or(0);
                        pending.hairpin_starts.push(subtype);
                    }
                    "HairPin" if is_end => match voice.beats.last_mut() {
                        Some(beat) => beat.hairpin_ends += 1,
                        None => voice.leading_hairpin_ends += 1,
                    },
                    "Volta" => {
                        let start = fields.iter().any(|f| f.name == "Volta").then(|| {
                            let endings = descendant_text(&fields, "endings")
                                .unwrap_or_default()
                                .split(',')
                                .filter_map(|n| n.trim().parse().ok())
                                .collect();
                            MscxVolta {
                                endings,
                                begin_text: descendant_text(&fields, "beginText")
                                    .map(str::to_string),
                                end_hook_type: descendant_text(&fields, "endHookType")
                                    .and_then(|t| t.parse().ok()),
                            }
                        });
                        if start.is_some() || is_end {
                            voice.voltas.push(MscxVoltaMark {
                                start,
                                beats_before: voice.beats.len(),
                                measures: descendant_text(&fields, "measures")
                                    .and_then(|t| t.trim().parse().ok()),
                                fractions: descendant_text(&fields, "fractions")
                                    .and_then(parse_fraction),
                            });
                        }
                    }
                    _ => {}
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Dynamic" => {
                let fields = read_fields(reader, buffer, "Dynamic")?;
                // The text's dynamic glyphs read as their letters, each
                // glyph in turn where it is (after the ones before it, so a
                // glyph's name can't match inside another's).
                let text = child_text(&fields, "text").map(|text| {
                    let mut text = text.to_string();
                    let mut from = 0;
                    for sym in fields.iter().filter(|f| f.depth > 1 && f.name == "sym") {
                        let Some(at) = text
                            .get(from..)
                            .and_then(|rest| rest.find(&sym.text))
                            .map(|offset| from + offset)
                        else {
                            continue;
                        };
                        let letters = dynamic_glyph_letters(sym.text.trim())
                            .unwrap_or_else(|| sym.text.clone());
                        text.replace_range(at..at + sym.text.len(), &letters);
                        from = at + letters.len();
                    }
                    text
                });
                if pending.dynamic.is_some() {
                    pending.superseded_dynamics += 1;
                }
                pending.dynamic = Some(MscxDynamic {
                    subtype: child_text(&fields, "subtype").map(str::to_string),
                    text,
                    velocity: child_text(&fields, "velocity").and_then(|t| t.parse().ok()),
                    voice_only: child_text(&fields, "voiceAssignment")
                        .is_some_and(|t| t.trim() == "currentVoiceOnly"),
                });
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Harmony" => {
                let fields = read_fields(reader, buffer, "Harmony")?;
                // Newer MuseScore versions nest the chord one level deeper.
                let tpc = |name| descendant_text(&fields, name).and_then(|t| t.parse().ok());
                if pending.harmony.is_some() {
                    pending.superseded_harmonies += 1;
                }
                pending.harmony = Some(MscxHarmony {
                    harmony_type: child_text(&fields, "harmonyType").and_then(|t| t.parse().ok()),
                    root: tpc("root"),
                    base: tpc("base"),
                    name: descendant_text(&fields, "name")
                        .unwrap_or_default()
                        .to_string(),
                });
            }
            XmlEvent::Start(element) if element.name().as_ref() == "Fermata" => {
                let fields = read_fields(reader, buffer, "Fermata")?;
                let subtype = child_text(&fields, "subtype").unwrap_or("fermataAbove");
                pending.fermatas.push(subtype.to_string());
            }
            // After the chord it follows.
            XmlEvent::Start(element) if element.name().as_ref() == "Breath" => {
                let fields = read_fields(reader, buffer, "Breath")?;
                let symbol = child_text(&fields, "symbol")
                    .or_else(|| child_text(&fields, "subtype"))
                    .unwrap_or_default()
                    .to_string();
                match voice.beats.last_mut() {
                    Some(beat) => beat.breaths.push(symbol),
                    None => voice.leading_breaths.push(symbol),
                }
            }
            XmlEvent::Start(element) if element.name().as_ref() == "BarLine" => {
                let fields = read_fields(reader, buffer, "BarLine")?;
                voice.end_barline = Some(MscxBarLine {
                    subtype: child_text(&fields, "subtype")
                        .unwrap_or("normal")
                        .to_string(),
                    span_from: child_text(&fields, "spanFrom").and_then(|t| t.parse().ok()),
                    span_to: child_text(&fields, "spanTo").and_then(|t| t.parse().ok()),
                    visible: child_text(&fields, "visible") != Some("0"),
                });
            }
            XmlEvent::Start(element) if matches!(element.name().as_ref(), "Chord" | "Rest") => {
                let chord = element.name().as_ref() == "Chord";
                drop(element);
                let parsed = if chord {
                    parse_chord(reader, buffer)?
                } else {
                    parse_rest(reader, buffer)?
                };
                let mut beat = parsed;
                beat.segment = std::mem::take(&mut pending);
                beat.gap_before = pending_gap.take();
                measure.mid_clefs.extend(pending_clef.take());
                // Only a barline after the last beat ends the measure.
                voice.end_barline = None;
                voice.beats.push(beat);
            }
            XmlEvent::End(element) if element.name().as_ref() == "voice" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if pending_clef.is_some() {
        measure.end_clef = pending_clef;
    }
    voice.trailing = pending;
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

/// The clef type of a `<Clef>`: its `<concertClefType>`, else
/// `<transposingClefType>` or (older files) `<subtype>`.
fn parse_clef(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<Option<String>> {
    let mut concert: Option<String> = None;
    let mut other: Option<String> = None;

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("Clef read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "concertClefType" => {
                concert = Some(read_text(reader, buffer, "Clef/concertClefType")?);
            }
            XmlEvent::Start(element)
                if matches!(element.name().as_ref(), "transposingClefType" | "subtype") =>
            {
                let context = if element.name().as_ref() == "subtype" {
                    "Clef/subtype"
                } else {
                    "Clef/transposingClefType"
                };
                let text = read_text(reader, buffer, context)?;
                other.get_or_insert(text);
            }
            XmlEvent::End(element) if element.name().as_ref() == "Clef" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(concert
        .or(other)
        .map(|clef| clef.trim().to_string())
        .filter(|clef| !clef.is_empty()))
}

fn parse_key_sig(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<MscxKeySig> {
    let mut concert_key: Option<i8> = None;
    let mut accidental: Option<i8> = None;
    let mut mode = None;

    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("KeySig read: {error}")))?;
        match event {
            XmlEvent::Start(element) if element.name().as_ref() == "mode" => {
                mode = MscxKeyMode::from_name(read_text(reader, buffer, "KeySig/mode")?.trim());
            }
            XmlEvent::Start(element) if element.name().as_ref() == "concertKey" => {
                let text = read_text(reader, buffer, "KeySig/concertKey")?;
                concert_key = text.trim().parse::<i8>().ok().or(concert_key);
            }
            XmlEvent::Start(element) if element.name().as_ref() == "accidental" => {
                let text = read_text(reader, buffer, "KeySig/accidental")?;
                accidental = text.trim().parse::<i8>().ok().or(accidental);
            }
            XmlEvent::End(element) if element.name().as_ref() == "KeySig" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    // `<concertKey>` wins wherever it stands; `<accidental>` only stands in
    // for it in files without one.
    let fifths = concert_key.or(accidental).unwrap_or(0);
    Ok(MscxKeySig { fifths, mode })
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
    let mut slurs: Vec<MscxSlurEnd> = Vec::new();
    let mut beat = MscxBeat::new(duration, MscxBeatKind::Rest);

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
            XmlEvent::Empty(element) if MscxGrace::from_tag(element.name().as_ref()).is_some() => {
                beat.grace = MscxGrace::from_tag(element.name().as_ref());
            }
            XmlEvent::Start(element) if MscxGrace::from_tag(element.name().as_ref()).is_some() => {
                beat.grace = MscxGrace::from_tag(element.name().as_ref());
                drop(element);
                read_fields(reader, buffer, "Chord grace")?;
            }
            XmlEvent::Start(element)
                if matches!(
                    element.name().as_ref(),
                    "Articulation" | "Ornament" | "Tremolo" | "TremoloSingleChord" | "ChordLine"
                ) =>
            {
                let name = element.name().as_ref().to_string();
                drop(element);
                let fields = read_fields(reader, buffer, "Chord mark")?;
                let Some(subtype) = child_text(&fields, "subtype").filter(|t| !t.is_empty()) else {
                    buffer.clear();
                    continue;
                };
                let subtype = subtype.to_string();
                beat.marks.push(match name.as_str() {
                    "Articulation" => MscxChordMark::Articulation(subtype),
                    "Ornament" => MscxChordMark::Ornament(subtype),
                    "ChordLine" => MscxChordMark::ChordLine(subtype),
                    _ => MscxChordMark::Tremolo(subtype),
                });
            }
            // Chord-level spanners: MuseScore 4 writes a slur's start (with
            // `<next>`) and end (with `<prev>`) inside their chords.
            XmlEvent::Start(element) if element.name().as_ref() == "Spanner" => {
                let spanner_type = attribute_value(&element, "type")?.unwrap_or_default();
                drop(element);
                if spanner_type == "Slur" {
                    slurs.extend(parse_slur_spanner(reader, buffer)?);
                } else {
                    scan_spanner(reader, buffer)?;
                }
            }
            XmlEvent::End(element) if element.name().as_ref() == "Chord" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    beat.duration = duration;
    beat.kind = MscxBeatKind::Chord(notes);
    beat.slurs = slurs;
    Ok(beat)
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
    Ok(MscxBeat::new(duration, MscxBeatKind::Rest))
}

fn parse_note(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<MscxNote> {
    let mut note = MscxNote {
        pitch: None,
        tpc: None,
        head: None,
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
            XmlEvent::Start(element) if element.name().as_ref() == "head" => {
                let text = read_text(reader, buffer, "Note/head")?;
                note.head = Some(text.trim().to_string()).filter(|head| !head.is_empty());
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
                let ends = scan_spanner(reader, buffer)?;
                if spanner_type == "Tie" {
                    if ends.has_prev {
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

/// Consume a `<Spanner>` block, returning its [`SpannerEnds`].
fn scan_spanner(reader: &mut XmlReader<&[u8]>, buffer: &mut Vec<u8>) -> GpResult<SpannerEnds> {
    Ok(SpannerEnds::of(&read_fields(reader, buffer, "Spanner")?))
}

/// Which ends of a spanner a `<Spanner>` element is: `<prev>` (pointing
/// back to the start) makes it an end, `<next>` a start.
#[derive(Clone, Copy)]
struct SpannerEnds {
    has_prev: bool,
    has_next: bool,
}

impl SpannerEnds {
    /// Of a `<Spanner>`'s fields: its own `<prev>`/`<next>` children.
    fn of(fields: &[Field]) -> Self {
        let child = |name| fields.iter().any(|f| f.depth == 1 && f.name == name);
        Self {
            has_prev: child("prev"),
            has_next: child("next"),
        }
    }

    /// Whether it starts its spanner: a `<prev>` makes it an end, as ties
    /// read it. `None` with neither.
    fn is_start(self) -> Option<bool> {
        (self.has_prev || self.has_next).then_some(!self.has_prev)
    }
}

/// A chord's `<Spanner type="Slur">`: a start (`<next>`) or end (`<prev>`)
/// with the other end's relative `<location>`; `None` with neither.
fn parse_slur_spanner(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
) -> GpResult<Option<MscxSlurEnd>> {
    let fields = read_fields(reader, buffer, "Spanner")?;
    Ok(SpannerEnds::of(&fields)
        .is_start()
        .map(|start| MscxSlurEnd {
            start,
            location: location_from_fields(&fields),
        }))
}

/// A voice-level `<location>`'s `<fractions>`; `None` without one.
fn parse_voice_location(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
) -> GpResult<Option<(i32, i32)>> {
    let fields = read_fields(reader, buffer, "location")?;
    Ok(location_from_fields(&fields).fractions)
}

/// The `<location>` among an element's fields.
fn location_from_fields(fields: &[Field]) -> MscxLocation {
    MscxLocation {
        measures: descendant_text(fields, "measures")
            .and_then(|t| t.parse().ok())
            .unwrap_or(0),
        fractions: descendant_text(fields, "fractions").and_then(parse_fraction),
        voices: descendant_text(fields, "voices")
            .and_then(|t| t.parse().ok())
            .unwrap_or(0),
    }
}

/// `a/b + c/d`, reduced; `None` when that doesn't fit an `i32` fraction.
pub(crate) fn add_fractions((a, b): (i32, i32), (c, d): (i32, i32)) -> Option<(i32, i32)> {
    let (b, d) = (i128::from(b), i128::from(d));
    let (numerator, denominator) =
        crate::model::mscz::mscx::reduce_fraction(i128::from(a) * d + i128::from(c) * b, b * d);
    Some((
        i32::try_from(numerator).ok()?,
        i32::try_from(denominator).ok()?,
    ))
}

/// `"-7/16"` → `(-7, 16)`; `None` for anything else or a zero denominator.
pub(crate) fn parse_fraction(text: &str) -> Option<(i32, i32)> {
    let (numerator, denominator) = text.split_once('/')?;
    let numerator: i32 = numerator.trim().parse().ok()?;
    let denominator: i32 = denominator.trim().parse().ok()?;
    // The sign on the numerator, so a denominator is always positive.
    match denominator {
        0 => None,
        ..0 => Some((numerator.checked_neg()?, denominator.checked_neg()?)),
        _ => Some((numerator, denominator)),
    }
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

/// An element inside the one being read: how deep below it (1 = a child),
/// its name and all its text, its markup's text joined
/// (`<text>To <b>Coda</b></text>` reads `To Coda`).
struct Field {
    depth: u32,
    name: String,
    text: String,
}

/// Reads the currently open element to its end, as [`Field`]s (in the
/// order they close).
fn read_fields(
    reader: &mut XmlReader<&[u8]>,
    buffer: &mut Vec<u8>,
    context: &'static str,
) -> GpResult<Vec<Field>> {
    let mut open: Vec<(String, String)> = Vec::new();
    let mut fields = Vec::new();
    loop {
        let event = reader
            .read_event_into(buffer)
            .map_err(|error| GpError::MsczXml(format!("{context} read: {error}")))?;
        let text = match event {
            XmlEvent::Start(element) => {
                open.push((element.name().as_ref().to_string(), String::new()));
                None
            }
            XmlEvent::Empty(element) => {
                fields.push(Field {
                    depth: open.len() as u32 + 1,
                    name: element.name().as_ref().to_string(),
                    text: String::new(),
                });
                None
            }
            XmlEvent::End(_) => match open.pop() {
                Some((name, text)) => {
                    fields.push(Field {
                        depth: open.len() as u32 + 1,
                        name,
                        text,
                    });
                    None
                }
                None => break,
            },
            XmlEvent::Text(text) => Some(
                unescape(text.as_ref())
                    .map_err(|error| GpError::MsczXml(format!("text unescape: {error}")))?
                    .into_owned(),
            ),
            XmlEvent::CData(text) => Some(text.as_ref().to_string()),
            XmlEvent::GeneralRef(reference) => {
                let character = reference
                    .resolve_char_ref()
                    .map_err(|error| GpError::MsczXml(format!("entity ref: {error}")))?;
                Some(match character {
                    Some(character) => character.to_string(),
                    None => match reference.as_ref() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        _ => "",
                    }
                    .to_string(),
                })
            }
            XmlEvent::Eof => break,
            _ => None,
        };
        if let Some(text) = text {
            for (_, all) in &mut open {
                all.push_str(&text);
            }
        }
        buffer.clear();
    }
    Ok(fields)
}

/// The trimmed text of the first child named `name`.
fn child_text<'a>(fields: &'a [Field], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|f| f.depth == 1 && f.name == name)
        .map(|f| f.text.trim())
}

/// [`child_text`] as an owned string, empty when there is no such child.
fn owned_child_text(fields: &[Field], name: &str) -> String {
    child_text(fields, name).unwrap_or_default().to_string()
}

/// The trimmed text of the first element named `name`, at any depth.
fn descendant_text<'a>(fields: &'a [Field], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.text.trim())
}

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
