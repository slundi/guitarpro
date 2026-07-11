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
use crate::model::mscz::{Instrument, MetaTag, Mscx, Part, Staff, StaffMeasureCount, StringData};

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
    let mut measure_counts: Vec<StaffMeasureCount> = Vec::new();

    // A tiny state machine tracks whether we're inside `<Score>` (top-level
    // matters — the file has a nested `<Score>` shape). `depth_in_score`
    // counts open `<Score>` tags; anything > 0 means "inside the score".
    let mut depth_in_score = 0u32;
    // When inside `<Score>`, track the currently-open top-level `<Staff>`
    // (the "master" staff that owns measures).
    let mut score_staff: Option<StaffMeasureCount> = None;
    // Nested `<Part>` context (only populated while parsing one).
    let mut current_part: Option<Part> = None;

    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|error| GpError::MsczXml(format!("read event: {error}")))?;

        match event {
            XmlEvent::Start(element) if element.name().as_ref() == b"museScore" => {
                version = attribute_value(&element, b"version")?.unwrap_or_default();
                validate_version(&version)?;
            }

            XmlEvent::Start(element) if element.name().as_ref() == b"Score" => {
                depth_in_score += 1;
            }
            XmlEvent::End(element) if element.name().as_ref() == b"Score" => {
                depth_in_score = depth_in_score.saturating_sub(1);
                if depth_in_score == 0
                    && let Some(staff) = score_staff.take()
                {
                    measure_counts.push(staff);
                }
            }

            XmlEvent::Start(element)
                if depth_in_score > 0
                    && element.name().as_ref() == b"Staff"
                    // Only the top-level Staff (Score > Staff) — not Part > Staff
                    && current_part.is_none() =>
            {
                if let Some(previous) = score_staff.take() {
                    measure_counts.push(previous);
                }
                let staff_id = attribute_value(&element, b"id")?.unwrap_or_default();
                score_staff = Some(StaffMeasureCount {
                    staff_id,
                    measure_count: 0,
                });
            }
            XmlEvent::End(element)
                if depth_in_score > 0
                    && element.name().as_ref() == b"Staff"
                    && current_part.is_none() =>
            {
                if let Some(staff) = score_staff.take() {
                    measure_counts.push(staff);
                }
            }
            XmlEvent::Empty(element) | XmlEvent::Start(element)
                if depth_in_score > 0 && element.name().as_ref() == b"Measure" =>
            {
                if let Some(staff) = score_staff.as_mut() {
                    staff.measure_count += 1;
                }
            }

            XmlEvent::Start(element) if element.name().as_ref() == b"programVersion" => {
                program_version = Some(read_text(&mut reader, &mut buffer, "programVersion")?);
            }
            XmlEvent::Start(element) if element.name().as_ref() == b"programRevision" => {
                program_revision = Some(read_text(&mut reader, &mut buffer, "programRevision")?);
            }
            XmlEvent::Start(element)
                if depth_in_score > 0 && element.name().as_ref() == b"Division" =>
            {
                let text = read_text(&mut reader, &mut buffer, "Division")?;
                division = text.trim().parse::<u32>().ok();
            }
            XmlEvent::Start(element)
                if depth_in_score > 0 && element.name().as_ref() == b"metaTag" =>
            {
                let name = attribute_value(&element, b"name")?.unwrap_or_default();
                let value = read_text(&mut reader, &mut buffer, "metaTag")?;
                meta_tags.push(MetaTag { name, value });
            }
            XmlEvent::Start(element)
                if depth_in_score > 0 && element.name().as_ref() == b"Part" =>
            {
                let id = attribute_value(&element, b"id")?.unwrap_or_default();
                current_part = Some(Part {
                    id,
                    track_name: None,
                    staves: Vec::new(),
                    instrument: None,
                });
            }
            XmlEvent::End(element) if depth_in_score > 0 && element.name().as_ref() == b"Part" => {
                if let Some(part) = current_part.take() {
                    parts.push(part);
                }
            }

            // Nested elements inside a `<Part>` — the parser only enters this
            // branch while `current_part` is `Some`.
            XmlEvent::Start(element)
                if current_part.is_some() && element.name().as_ref() == b"trackName" =>
            {
                let text = read_text(&mut reader, &mut buffer, "trackName")?;
                if let Some(part) = current_part.as_mut() {
                    part.track_name = Some(text);
                }
            }
            XmlEvent::Start(element)
                if current_part.is_some() && element.name().as_ref() == b"Staff" =>
            {
                let id = attribute_value(&element, b"id")?.unwrap_or_default();
                drop(element);
                let staff = parse_part_staff(&mut reader, &mut buffer, id)?;
                if let Some(part) = current_part.as_mut() {
                    part.staves.push(staff);
                }
            }
            XmlEvent::Start(element)
                if current_part.is_some() && element.name().as_ref() == b"Instrument" =>
            {
                let id = attribute_value(&element, b"id")?.unwrap_or_default();
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

    Ok(Mscx {
        raw_xml: xml.to_string(),
        version,
        program_version,
        program_revision,
        division,
        meta_tags,
        parts,
        measure_counts,
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
// Helpers
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
            XmlEvent::Start(element) if element.name().as_ref() == b"StaffType" => {
                staff.group = attribute_value(&element, b"group")?;
            }
            XmlEvent::Start(element) if element.name().as_ref() == b"name" => {
                staff.type_name = Some(read_text(reader, buffer, "StaffType/name")?);
            }
            XmlEvent::Start(element) if element.name().as_ref() == b"defaultClef" => {
                staff.default_clef = Some(read_text(reader, buffer, "defaultClef")?);
            }
            XmlEvent::End(element) if element.name().as_ref() == b"Staff" => break,
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
                b"longName" => {
                    instrument.long_name = Some(read_text(reader, buffer, "longName")?);
                }
                b"shortName" => {
                    instrument.short_name = Some(read_text(reader, buffer, "shortName")?);
                }
                b"trackName" => {
                    instrument.track_name = Some(read_text(reader, buffer, "trackName")?);
                }
                b"transposeDiatonic" => {
                    instrument.transpose_diatonic =
                        parse_i8(&read_text(reader, buffer, "transposeDiatonic")?);
                }
                b"transposeChromatic" => {
                    instrument.transpose_chromatic =
                        parse_i8(&read_text(reader, buffer, "transposeChromatic")?);
                }
                b"instrumentId" => {
                    instrument.instrument_id = Some(read_text(reader, buffer, "instrumentId")?);
                }
                b"StringData" => {
                    instrument.string_data = Some(parse_string_data(reader, buffer)?);
                }
                _ => {}
            },
            XmlEvent::End(element) if element.name().as_ref() == b"Instrument" => break,
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
                b"frets" => {
                    let text = read_text(reader, buffer, "frets")?;
                    data.frets = text.trim().parse::<u8>().ok();
                }
                b"string" => {
                    let text = read_text(reader, buffer, "string")?;
                    if let Ok(value) = text.trim().parse::<u8>() {
                        data.strings.push(value);
                    }
                }
                _ => {}
            },
            XmlEvent::End(element) if element.name().as_ref() == b"StringData" => break,
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(data)
}

fn attribute_value(element: &BytesStart<'_>, key: &[u8]) -> GpResult<Option<String>> {
    for attribute in element.attributes() {
        let attribute =
            attribute.map_err(|error| GpError::MsczXml(format!("attribute: {error}")))?;
        if attribute.key.as_ref() == key {
            let raw = std::str::from_utf8(attribute.value.as_ref())
                .map_err(|error| GpError::MsczXml(format!("attribute utf-8: {error}")))?;
            let value = unescape(raw)
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
                let decoded = text
                    .decode()
                    .map_err(|error| GpError::MsczXml(format!("text decode: {error}")))?;
                let unescaped = unescape(&decoded)
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
                let name = std::str::from_utf8(reference.as_ref()).map_err(|error| {
                    GpError::MsczXml(format!("entity name utf-8 ({context}): {error}"))
                })?;
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
                let value = std::str::from_utf8(text.as_ref()).map_err(|error| {
                    GpError::MsczXml(format!("cdata utf-8 ({context}): {error}"))
                })?;
                output.push_str(value);
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
