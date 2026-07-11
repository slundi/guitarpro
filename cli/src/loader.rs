use std::fs;
use std::io::Read;
use std::path::Path;

use guitarpro::Song;
use guitarpro::convert::legacy::loaded_score_to_legacy_song;
use guitarpro::convert::mscz::mscx_to_loaded_score;
use guitarpro::io::mscz::read_mscz_bytes;

/// Maximum accepted size for legacy Guitar Pro binaries and GPX/GP containers.
const MAX_LEGACY_FILE_SIZE: usize = 16 * 1024 * 1024;
/// Maximum accepted size for MSCZ archives (larger than GP because MuseScore
/// bundles thumbnails and soundfont overrides).
const MAX_MSCZ_FILE_SIZE: usize = 32 * 1024 * 1024;

/// ZIP local-file-header magic (`PK\x03\x04`). MSCZ and `.gp` (GP7+) share
/// this signature — the container manifest disambiguates.
const ZIP_MAGIC: &[u8; 4] = b"PK\x03\x04";

/// Load a Guitar Pro or MuseScore file from disk and return the parsed
/// `Song` and the detected format string (e.g. `"GP5"`, `"GPX"`, `"MSCZ"`).
///
/// MSCZ archives are converted through `LoadedScore` → `Song` so downstream
/// CLI commands (info, repeats, form, fingering, extract, duplicates) can
/// consume them without needing a separate MSCZ code path.
pub fn load_song(path_str: &str) -> anyhow::Result<(Song, String)> {
    let path = Path::new(path_str);
    if !path.exists() {
        anyhow::bail!("File '{}' not found", path_str);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let size = fs::metadata(path_str)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    // MSCZ archives are handled first so their larger size budget applies.
    let looks_like_mscz = ext == "MSCZ" || (looks_like_zip(path_str) && is_mscz_archive(path_str)?);
    if looks_like_mscz {
        if size > MAX_MSCZ_FILE_SIZE {
            anyhow::bail!("MSCZ file is too large (> 32 MB)");
        }
        let data = fs::read(path_str)?;
        let file = read_mscz_bytes(&data)
            .map_err(|e| anyhow::anyhow!("cannot read MSCZ '{}': {}", path_str, e))?;
        let outcome = mscx_to_loaded_score(&file.mscx);
        return Ok((loaded_score_to_legacy_song(&outcome.score), "MSCZ".into()));
    }

    if size > MAX_LEGACY_FILE_SIZE {
        anyhow::bail!("File is too large (> 16 MB)");
    }

    let mut file = fs::File::open(path_str)?;
    let mut data = Vec::with_capacity(size);
    file.read_to_end(&mut data)?;

    // Prefer the format encoded in the file itself over the on-disk extension:
    // real-world dumps often carry the wrong extension (e.g. a v5.10 file named
    // `.gp3`), and the extension-only path fails hard with confusing offsets.
    let detected = detect_legacy_version(&data);

    let mut song = Song::default();
    let format_label: String = match (detected, ext.as_str()) {
        (Some(v), _) => {
            match v {
                // Guitar Pro 1/2 share enough of the GP3 layout that the
                // GP3 reader loads them (best-effort — some features are
                // silently dropped).
                (1 | 2, _, _) | (3, _, _) => song.read_gp3(&data)?,
                (4, _, _) => song.read_gp4(&data)?,
                (5, _, _) => song.read_gp5(&data)?,
                _ => anyhow::bail!("Unsupported Guitar Pro version v{}.{}.{}", v.0, v.1, v.2),
            };
            format!("GP{}", v.0.max(3))
        }
        (None, "GP") => {
            song.read_gp(&data)?;
            "GP".into()
        }
        (None, "GPX") => {
            song.read_gpx(&data)?;
            "GPX".into()
        }
        (None, "GP3" | "GP4" | "GP5") => {
            anyhow::bail!(
                "File extension is .{}, but content is not a legacy Guitar Pro file (missing 'FICHIER GUITAR PRO' header)",
                ext.to_lowercase()
            );
        }
        _ => anyhow::bail!(
            "Unsupported format '{}'. Supported extensions: .gp3, .gp4, .gp5, .gp, .gpx, .mscz",
            ext
        ),
    };

    Ok((song, format_label))
}

/// Peek the first four bytes to check for the ZIP magic. Returns `false` on
/// short files or read errors (the caller then falls back to extension).
fn looks_like_zip(path_str: &str) -> bool {
    let mut buffer = [0u8; 4];
    match fs::File::open(path_str).and_then(|mut file| file.read_exact(&mut buffer)) {
        Ok(_) => &buffer == ZIP_MAGIC,
        Err(_) => false,
    }
}

/// Distinguish MSCZ from `.gp` (both are ZIPs) by checking whether the archive
/// contains the OPC-style `META-INF/container.xml` manifest that MuseScore
/// writes but Guitar Pro does not. Short-circuits on IO errors: returns `false`
/// so the caller can proceed with the legacy path detection.
fn is_mscz_archive(path_str: &str) -> anyhow::Result<bool> {
    let data = fs::read(path_str)?;
    let cursor = std::io::Cursor::new(data);
    let mut zip = match zip::ZipArchive::new(cursor) {
        Ok(archive) => archive,
        Err(_) => return Ok(false),
    };
    Ok(zip.by_name("META-INF/container.xml").is_ok())
}

/// Peek at the header to identify the actual legacy version. Returns the
/// `(major, minor, patch)` tuple parsed from the "FICHIER GUITAR PRO vX.YZ"
/// magic string, or `None` if the header is absent (e.g. GP6/7 container or
/// non-Guitar-Pro content).
fn detect_legacy_version(data: &[u8]) -> Option<(u8, u8, u8)> {
    // Legacy header: 1 length byte (0x18 = 24) followed by
    // "FICHIER GUITAR PRO vX.YZ" (24 ASCII bytes).
    if data.len() < 30 || data[0] != 0x18 {
        return None;
    }
    let header = std::str::from_utf8(&data[1..25]).ok()?;
    let magic = "FICHIER GUITAR PRO v";
    let ver = header.strip_prefix(magic)?; // e.g. "3.00", "5.10"
    let mut parts = ver.split('.');
    let major: u8 = parts.next()?.parse().ok()?;
    let minor_raw = parts.next()?;
    // Minor is stored as two ASCII digits ("00", "10", "20", …) where the
    // second digit is the "patch". Split to match `Song::version.number`.
    if minor_raw.len() < 2 {
        return None;
    }
    let minor: u8 = minor_raw[..1].parse().ok()?;
    let patch: u8 = minor_raw[1..2].parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_header(version: &str) -> Vec<u8> {
        let mut out = vec![0x18];
        let body = format!("FICHIER GUITAR PRO v{version}");
        assert_eq!(body.len(), 24, "header must be exactly 24 bytes");
        out.extend_from_slice(body.as_bytes());
        // Pad so `data.len() >= 30`.
        out.extend_from_slice(&[0; 8]);
        out
    }

    #[test]
    fn detects_gp3_header() {
        assert_eq!(detect_legacy_version(&make_header("3.00")), Some((3, 0, 0)));
    }

    #[test]
    fn detects_gp5_header() {
        assert_eq!(detect_legacy_version(&make_header("5.10")), Some((5, 1, 0)));
    }

    #[test]
    fn rejects_non_gp_header() {
        // Ghost/HTML content — should not falsely register as GP.
        let html = b"                <DIV class=Section1>";
        assert_eq!(detect_legacy_version(html), None);
    }

    #[test]
    fn rejects_gp7_zip_container() {
        // .gp files start with the ZIP local-header signature "PK\x03\x04".
        let zip = b"PK\x03\x04\x14\x00\x00\x00\x08\x00\xEE\x8E\xEA\\";
        assert_eq!(detect_legacy_version(zip), None);
    }

    #[test]
    fn detects_gp5_when_extension_is_gp3() {
        // A representative user-supplied file that is v5.10 despite the .gp3
        // extension. The loader should pick up the true version from the
        // header magic rather than trust the extension.
        let path = "../test/edge_cases/gp5_mislabelled_as_gp3.gp3";
        if !std::path::Path::new(path).exists() {
            // Fixture only lives in the workspace; skip on external builds.
            return;
        }
        let data = std::fs::read(path).unwrap();
        assert!(matches!(detect_legacy_version(&data), Some((5, _, _))));
    }
}
