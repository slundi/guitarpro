//! MSCZ ZIP container (read + write).
//!
//! An MSCZ file is a plain ZIP archive containing:
//! * `META-INF/container.xml` — OPC-style manifest listing `<rootfile>`s
//! * `<name>.mscx` — the primary MuseScore XML
//! * Zero or more side files (`.mss` styles, thumbnails, JSON settings, …)
//!
//! Both read and write paths preserve every entry byte-for-byte so callers
//! that only touch container-level metadata get a byte-stable round-trip.

use std::io::{Cursor, Read, Write};

use quick_xml::Reader as XmlReader;
use quick_xml::escape::unescape;
use quick_xml::events::Event as XmlEvent;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::{GpError, GpResult};
use crate::model::mscz::{MsczArchive, MsczEntry};

/// Maximum accepted MSCZ file size (32 MB — matches the roadmap).
pub const MAX_MSCZ_BYTES: usize = 32 * 1024 * 1024;

/// Maximum number of entries in an MSCZ archive.
pub const MAX_MSCZ_ENTRIES: usize = 256;

/// Read an MSCZ archive from raw bytes.
///
/// * Validates the total input size and entry count against
///   [`MAX_MSCZ_BYTES`] / [`MAX_MSCZ_ENTRIES`].
/// * Parses `META-INF/container.xml` (when present) to populate
///   [`MsczArchive::rootfiles`].
/// * Preserves every entry — including `container.xml` — in the returned
///   [`MsczArchive::entries`], so a later `write_container` call reproduces
///   the archive byte-for-byte (given a stable zip encoder).
pub fn read_container(data: &[u8]) -> GpResult<MsczArchive> {
    if data.len() > MAX_MSCZ_BYTES {
        return Err(GpError::MsczArchive(format!(
            "MSCZ input exceeds {} bytes ({} bytes)",
            MAX_MSCZ_BYTES,
            data.len()
        )));
    }

    let cursor = Cursor::new(data);
    let mut zip = ZipArchive::new(cursor)
        .map_err(|error| GpError::MsczArchive(format!("open zip: {error}")))?;

    if zip.len() > MAX_MSCZ_ENTRIES {
        return Err(GpError::MsczArchive(format!(
            "MSCZ has {} entries (max {})",
            zip.len(),
            MAX_MSCZ_ENTRIES
        )));
    }

    let mut entries = Vec::with_capacity(zip.len());
    for index in 0..zip.len() {
        let mut file = zip
            .by_index(index)
            .map_err(|error| GpError::MsczArchive(format!("entry {index}: {error}")))?;

        if file.is_dir() {
            continue;
        }
        let path = file.name().to_string();
        let mut buffer = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buffer)
            .map_err(|error| GpError::MsczArchive(format!("read entry {path}: {error}")))?;
        entries.push(MsczEntry { path, data: buffer });
    }

    let rootfiles = entries
        .iter()
        .find(|entry| entry.path == "META-INF/container.xml")
        .map(|entry| parse_container_manifest(&entry.data))
        .transpose()?
        .unwrap_or_default();

    Ok(MsczArchive { rootfiles, entries })
}

/// Write an MSCZ archive to a fresh `Vec<u8>`.
///
/// * Entries are written in `archive.entries` order (deterministic).
/// * Compression: `Deflated` for XML/JSON/text, `Stored` for PNG/other
///   already-compressed data.
///
/// Note: ZIP field defaults (dates, extra fields) come from the `zip` crate,
/// so byte equality with archives produced by MuseScore itself is not
/// promised. `read_container(write_container(read_container(x)))` is stable
/// and that's the round-trip [`crate::io::mscz`] tests exercise.
pub fn write_container(archive: &MsczArchive) -> GpResult<Vec<u8>> {
    let buffer = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(buffer);

    for entry in &archive.entries {
        let method = compression_for(&entry.path);
        let options = SimpleFileOptions::default().compression_method(method);
        writer
            .start_file(&entry.path, options)
            .map_err(|error| GpError::MsczArchive(format!("start {}: {error}", entry.path)))?;
        writer
            .write_all(&entry.data)
            .map_err(|error| GpError::MsczArchive(format!("write {}: {error}", entry.path)))?;
    }

    let cursor = writer
        .finish()
        .map_err(|error| GpError::MsczArchive(format!("finalize: {error}")))?;
    Ok(cursor.into_inner())
}

/// Choose a compression method based on file extension.
///
/// PNG and JPEG are already compressed; storing them uncompressed avoids the
/// pathological deflate expansion MuseScore's thumbnails sometimes trigger.
fn compression_for(path: &str) -> CompressionMethod {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        CompressionMethod::Stored
    } else {
        CompressionMethod::Deflated
    }
}

/// Parse `META-INF/container.xml` and return the ordered list of
/// `<rootfile full-path="…"/>` paths.
fn parse_container_manifest(data: &[u8]) -> GpResult<Vec<String>> {
    let text = std::str::from_utf8(data)
        .map_err(|error| GpError::MsczArchive(format!("container.xml not utf-8: {error}")))?;

    let mut reader = XmlReader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut rootfiles = Vec::new();
    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(XmlEvent::Empty(element)) | Ok(XmlEvent::Start(element))
                if element.name().as_ref() == "rootfile" =>
            {
                for attribute in element.attributes().flatten() {
                    if attribute.key.as_ref() == "full-path" {
                        let value = unescape(attribute.value.as_ref()).map_err(|error| {
                            GpError::MsczArchive(format!("container.xml attr: {error}"))
                        })?;
                        rootfiles.push(value.into_owned());
                    }
                }
            }
            Ok(XmlEvent::Eof) => break,
            Err(error) => {
                return Err(GpError::MsczArchive(format!(
                    "container.xml parse: {error}"
                )));
            }
            _ => {}
        }
        buffer.clear();
    }

    Ok(rootfiles)
}
