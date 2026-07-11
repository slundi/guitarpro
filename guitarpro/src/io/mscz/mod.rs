//! MSCZ (MuseScore compressed archive) I/O.
//!
//! Part 1 of the MSCZ roadmap: container extraction + high-level MSCX AST +
//! byte-stable round-trip. The primary entry points are:
//!
//! * [`read_mscz`] / [`read_mscz_bytes`] — parse an archive into an
//!   [`MsczFile`] (raw entries preserved + parsed `Mscx` view).
//! * [`write_mscz`] — reassemble an archive from an [`MsczFile`].
//! * [`parse_mscx`] / [`write_mscx`] — low-level MSCX XML view helpers.

pub mod container;
pub mod parse;

pub use container::{MAX_MSCZ_BYTES, MAX_MSCZ_ENTRIES, read_container, write_container};
pub use parse::{parse_mscx, write_mscx};

use std::path::Path;

use crate::error::{GpError, GpResult};
use crate::model::mscz::{MsczArchive, MsczFile};

/// Parse an MSCZ archive from a filesystem path.
pub fn read_mscz(path: impl AsRef<Path>) -> GpResult<MsczFile> {
    let bytes = std::fs::read(path.as_ref())?;
    read_mscz_bytes(&bytes)
}

/// Parse an MSCZ archive from raw bytes.
pub fn read_mscz_bytes(data: &[u8]) -> GpResult<MsczFile> {
    let archive = read_container(data)?;
    let mscx_entry = archive
        .mscx_entry()
        .ok_or_else(|| GpError::MsczArchive("no .mscx entry found in archive".to_string()))?;
    let xml = std::str::from_utf8(&mscx_entry.data)
        .map_err(|error| GpError::MsczXml(format!("mscx not utf-8: {error}")))?;
    let mscx = parse_mscx(xml)?;
    Ok(MsczFile { archive, mscx })
}

/// Serialize an [`MsczFile`] to a fresh `Vec<u8>`.
///
/// If [`MsczFile::mscx`] `.raw_xml` differs from the archive's stored MSCX
/// entry, the archive is patched with the new XML before writing so callers
/// that mutate the AST get their change persisted.
pub fn write_mscz(file: &MsczFile) -> GpResult<Vec<u8>> {
    let mut archive = file.archive.clone();
    patch_mscx_entry(&mut archive, &file.mscx.raw_xml)?;
    write_container(&archive)
}

/// Write the archive to disk.
pub fn write_mscz_to_path(file: &MsczFile, path: impl AsRef<Path>) -> GpResult<()> {
    let bytes = write_mscz(file)?;
    std::fs::write(path.as_ref(), bytes)?;
    Ok(())
}

fn patch_mscx_entry(archive: &mut MsczArchive, xml: &str) -> GpResult<()> {
    let target_path = archive
        .mscx_entry()
        .map(|entry| entry.path.clone())
        .ok_or_else(|| GpError::MsczArchive("no .mscx entry to update".to_string()))?;

    for entry in archive.entries.iter_mut() {
        if entry.path == target_path {
            let bytes = xml.as_bytes();
            if entry.data != bytes {
                entry.data = bytes.to_vec();
            }
            break;
        }
    }
    Ok(())
}
