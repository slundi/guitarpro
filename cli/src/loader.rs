use std::fs;
use std::io::Read;
use std::path::Path;

use guitarpro::Song;

const MAX_FILE_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// Load a Guitar Pro file from disk and return the parsed `Song` and the
/// detected format string (e.g. `"GP5"`, `"GPX"`).
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

    if size > MAX_FILE_SIZE {
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
            "Unsupported format '{}'. Supported extensions: .gp3, .gp4, .gp5, .gp, .gpx",
            ext
        ),
    };

    Ok((song, format_label))
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
