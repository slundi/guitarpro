//! Format identification plus the byte-level decode/encode pipeline.
//!
//! Everything funnels through the legacy [`Song`] model, which is the
//! workspace's universal intermediate representation: each input format is
//! decoded into a `Song`, and each output format is encoded from one. This
//! mirrors `cli/src/command_convert.rs`, but works on byte slices instead of
//! paths so the FFI never touches the filesystem on the caller's behalf.

use guitarpro::convert::guitarpro::musicxml_to_legacy_song;
use guitarpro::convert::legacy::loaded_score_to_legacy_song;
use guitarpro::convert::mscz::{loaded_score_to_mscx, mscx_to_loaded_score};
use guitarpro::convert::musicxml::song_to_score_partwise;
use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;
use guitarpro::io::mscz::{read_mscz_bytes, write_mscz};
use guitarpro::model::musicxml::ScorePartwise;
use guitarpro::model::optimized::LoadedScore;
use guitarpro::model::optimized::global::Score;
use guitarpro::{MsczArchive, MsczEntry, MsczFile, Song};

/// Score file format, used both to force an interpretation and to report the
/// one that was detected.
///
/// The numeric values are part of the ABI: append new variants, never
/// renumber existing ones.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioFormat {
    /// Detect the format from the file content. Input only.
    Auto = 0,
    /// Guitar Pro 3 (legacy binary).
    Gp3 = 1,
    /// Guitar Pro 4 (legacy binary).
    Gp4 = 2,
    /// Guitar Pro 5 (legacy binary).
    Gp5 = 3,
    /// Guitar Pro 6 (`.gpx`, BCFZ/BCFS container).
    Gpx = 4,
    /// Guitar Pro 7+ (`.gp`, ZIP container).
    Gp = 5,
    /// MusicXML (`score-partwise`, plain XML).
    MusicXml = 6,
    /// Optimized score model, serialized as JSON.
    Score = 7,
    /// MuseScore compressed archive (`.mscz`).
    Mscz = 8,
}

impl GpioFormat {
    /// Convert a raw ABI value into a variant.
    ///
    /// C callers can pass any `uint32_t`, so the value is validated here
    /// rather than transmuted — an out-of-range discriminant would otherwise
    /// be undefined behaviour.
    pub(crate) fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0 => Some(Self::Auto),
            1 => Some(Self::Gp3),
            2 => Some(Self::Gp4),
            3 => Some(Self::Gp5),
            4 => Some(Self::Gpx),
            5 => Some(Self::Gp),
            6 => Some(Self::MusicXml),
            7 => Some(Self::Score),
            8 => Some(Self::Mscz),
            _ => None,
        }
    }

    /// Human-readable name, used in error messages.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto-detect",
            Self::Gp3 => "Guitar Pro 3 (GP3)",
            Self::Gp4 => "Guitar Pro 4 (GP4)",
            Self::Gp5 => "Guitar Pro 5 (GP5)",
            Self::Gpx => "Guitar Pro 6 (GPX)",
            Self::Gp => "Guitar Pro 7+ (GP)",
            Self::MusicXml => "MusicXML",
            Self::Score => "Optimized Score (JSON)",
            Self::Mscz => "MuseScore (MSCZ)",
        }
    }

    /// Largest input accepted for this format.
    ///
    /// The FFI boundary is the one place where the byte slice is fully
    /// attacker-controlled, so each family gets an explicit ceiling instead of
    /// letting a malformed length field drive an unbounded allocation. MSCZ
    /// gets more room than Guitar Pro because MuseScore bundles thumbnails and
    /// soundfont overrides; the text formats get the most because XML and JSON
    /// are far less dense than the binary containers.
    fn max_input_size(self) -> usize {
        match self {
            Self::Mscz => 32 * 1024 * 1024,
            Self::MusicXml | Self::Score => 64 * 1024 * 1024,
            _ => 16 * 1024 * 1024,
        }
    }

    /// Version tuple passed to `Song::write` for the legacy binary formats.
    fn legacy_write_version(self) -> Option<(u8, u8, u8)> {
        match self {
            Self::Gp3 => Some((3, 0, 0)),
            Self::Gp4 => Some((4, 0, 0)),
            Self::Gp5 => Some((5, 1, 0)),
            _ => None,
        }
    }
}

/// ZIP local-file-header magic. Shared by `.mscz` and `.gp` (GP7+); the
/// archive contents disambiguate the two.
const ZIP_MAGIC: &[u8; 4] = b"PK\x03\x04";
/// GP6 container magic, compressed and uncompressed variants.
const BCFZ_MAGIC: &[u8; 4] = b"BCFZ";
const BCFS_MAGIC: &[u8; 4] = b"BCFS";
/// Manifest MuseScore writes and Guitar Pro does not.
const MSCZ_MANIFEST: &str = "META-INF/container.xml";
/// Leading bytes of the legacy Guitar Pro header, after the length byte.
const LEGACY_MAGIC: &str = "FICHIER GUITAR PRO v";
/// Bytes inspected when sniffing the textual formats.
const TEXT_PROBE_LEN: usize = 512;

/// Identify the format of `data` from its content alone.
///
/// Returns `None` when nothing matches; never returns [`GpioFormat::Auto`].
pub(crate) fn detect(data: &[u8]) -> Option<GpioFormat> {
    // The legacy header is the most specific signature, so test it first.
    if let Some((major, _, _)) = detect_legacy_version(data) {
        return match major {
            // GP1/GP2 share enough of the GP3 layout that the GP3 reader
            // loads them, best-effort.
            1..=3 => Some(GpioFormat::Gp3),
            4 => Some(GpioFormat::Gp4),
            5 => Some(GpioFormat::Gp5),
            _ => None,
        };
    }

    if data.starts_with(BCFZ_MAGIC) || data.starts_with(BCFS_MAGIC) {
        return Some(GpioFormat::Gpx);
    }

    if data.starts_with(ZIP_MAGIC) {
        return Some(if is_mscz_archive(data) {
            GpioFormat::Mscz
        } else {
            GpioFormat::Gp
        });
    }

    detect_text_format(data)
}

/// Sniff the textual formats from the first few hundred bytes.
fn detect_text_format(data: &[u8]) -> Option<GpioFormat> {
    let probe = String::from_utf8_lossy(&data[..data.len().min(TEXT_PROBE_LEN)]);
    let head = probe.trim_start_matches('\u{feff}').trim_start();

    if head.starts_with('{') {
        return Some(GpioFormat::Score);
    }
    if head.starts_with("<?xml") || head.starts_with("<!DOCTYPE") || head.starts_with("<score-") {
        return Some(GpioFormat::MusicXml);
    }
    None
}

/// Distinguish MSCZ from `.gp`: both are ZIPs, but only MuseScore writes the
/// OPC-style manifest. Unreadable archives fall through as "not MSCZ" so the
/// GP reader can produce the more specific error.
fn is_mscz_archive(data: &[u8]) -> bool {
    let Ok(mut archive) = zip::ZipArchive::new(std::io::Cursor::new(data)) else {
        return false;
    };
    archive.by_name(MSCZ_MANIFEST).is_ok()
}

/// Parse the `(major, minor, patch)` tuple out of the legacy
/// "FICHIER GUITAR PRO vX.YZ" header, or `None` when it is absent.
fn detect_legacy_version(data: &[u8]) -> Option<(u8, u8, u8)> {
    // One length byte (0x18 = 24) followed by 24 ASCII bytes.
    if data.len() < 30 || data[0] != 0x18 {
        return None;
    }
    let header = std::str::from_utf8(&data[1..25]).ok()?;
    let version = header.strip_prefix(LEGACY_MAGIC)?;
    let mut parts = version.split('.');
    let major: u8 = parts.next()?.parse().ok()?;
    // Minor is two ASCII digits ("00", "10", …) whose second digit is the
    // patch level, matching `Song::version.number`.
    let minor_raw = parts.next()?;
    if minor_raw.len() < 2 {
        return None;
    }
    let minor: u8 = minor_raw[..1].parse().ok()?;
    let patch: u8 = minor_raw[1..2].parse().ok()?;
    Some((major, minor, patch))
}

/// Decode `data` into a [`Song`], returning it alongside the format actually
/// used (which differs from `requested` only when it was [`GpioFormat::Auto`]).
pub(crate) fn decode(data: &[u8], requested: GpioFormat) -> Result<(Song, GpioFormat), String> {
    let resolved = match requested {
        GpioFormat::Auto => detect(data).ok_or_else(|| {
            "cannot detect the input format from the file content; pass an explicit format"
                .to_string()
        })?,
        other => other,
    };

    let limit = resolved.max_input_size();
    if data.len() > limit {
        return Err(format!(
            "{} input is {} bytes, over the {} byte limit",
            resolved.label(),
            data.len(),
            limit
        ));
    }

    let song = match resolved {
        GpioFormat::Gp3 => read_legacy(data, Song::read_gp3, resolved)?,
        GpioFormat::Gp4 => read_legacy(data, Song::read_gp4, resolved)?,
        GpioFormat::Gp5 => read_legacy(data, Song::read_gp5, resolved)?,
        GpioFormat::Gpx => read_legacy(data, Song::read_gpx, resolved)?,
        GpioFormat::Gp => read_legacy(data, Song::read_gp, resolved)?,
        GpioFormat::MusicXml => musicxml_to_legacy_song(&parse_musicxml(data)?),
        GpioFormat::Score => loaded_score_to_legacy_song(&parse_optimized_score(data)?),
        GpioFormat::Mscz => {
            let file =
                read_mscz_bytes(data).map_err(|error| format!("MSCZ read failed: {error}"))?;
            loaded_score_to_legacy_song(&mscx_to_loaded_score(&file.mscx).score)
        }
        // `Auto` was resolved above; this arm only exists to keep the match
        // exhaustive without an `unreachable!` panic at an FFI boundary.
        GpioFormat::Auto => return Err("input format resolution failed".to_string()),
    };

    Ok((song, resolved))
}

/// Shared wrapper around the `Song::read_*` family, which all share the same
/// `(&mut self, &[u8]) -> GpResult<()>` shape.
fn read_legacy(
    data: &[u8],
    reader: fn(&mut Song, &[u8]) -> guitarpro::GpResult<()>,
    format: GpioFormat,
) -> Result<Song, String> {
    let mut song = Song::default();
    reader(&mut song, data).map_err(|error| format!("{} parse failed: {error}", format.label()))?;
    Ok(song)
}

/// Encode `song` into the byte representation of `target`.
pub(crate) fn encode(song: &Song, target: GpioFormat) -> Result<Vec<u8>, String> {
    match target {
        GpioFormat::Gp3 | GpioFormat::Gp4 | GpioFormat::Gp5 => {
            let version = target
                .legacy_write_version()
                .ok_or_else(|| format!("no legacy version mapping for {}", target.label()))?;
            song.write(version, None)
                .map_err(|error| format!("{} encode failed: {error}", target.label()))
        }
        GpioFormat::Gpx => song
            .write_gpx()
            .map_err(|error| format!("GPX encode failed: {error}")),
        GpioFormat::Gp => song
            .write_gp()
            .map_err(|error| format!("GP encode failed: {error}")),
        GpioFormat::MusicXml => serialize_musicxml(&song_to_score_partwise(song)),
        GpioFormat::Score => serde_json::to_vec_pretty(&legacy_song_to_loaded_score(song).score)
            .map_err(|error| format!("Score serialize failed: {error}")),
        GpioFormat::Mscz => encode_mscz(song),
        GpioFormat::Auto => Err("GPIO_FORMAT_AUTO cannot be used as an output format".to_string()),
    }
}

/// Build a minimal MSCZ archive — manifest plus `score.mscx`, no thumbnail or
/// style side files. MuseScore reads such archives.
fn encode_mscz(song: &Song) -> Result<Vec<u8>, String> {
    let mscx = loaded_score_to_mscx(&legacy_song_to_loaded_score(song));
    let manifest = br#"<?xml version="1.0" encoding="UTF-8"?>
<container><rootfiles><rootfile full-path="score.mscx"/></rootfiles></container>"#;
    let archive = MsczArchive {
        rootfiles: vec!["score.mscx".to_string()],
        entries: vec![
            MsczEntry {
                path: MSCZ_MANIFEST.to_string(),
                data: manifest.to_vec(),
            },
            MsczEntry {
                path: "score.mscx".to_string(),
                data: mscx.raw_xml.as_bytes().to_vec(),
            },
        ],
    };
    write_mscz(&MsczFile { archive, mscx }).map_err(|error| format!("MSCZ encode failed: {error}"))
}

/// Strip the `<!DOCTYPE …>` declaration, which `quick-xml` refuses to parse.
fn strip_doctype(xml: &str) -> String {
    if let Some(start) = xml.find("<!DOCTYPE")
        && let Some(relative_end) = xml[start..].find('>')
    {
        return format!("{}{}", &xml[..start], &xml[start + relative_end + 1..]);
    }
    xml.to_string()
}

fn parse_musicxml(data: &[u8]) -> Result<ScorePartwise, String> {
    let raw = std::str::from_utf8(data)
        .map_err(|error| format!("MusicXML is not valid UTF-8: {error}"))?;
    quick_xml::de::from_str(&strip_doctype(raw))
        .map_err(|error| format!("MusicXML parse failed: {error}"))
}

fn serialize_musicxml(score: &ScorePartwise) -> Result<Vec<u8>, String> {
    let body = quick_xml::se::to_string(score)
        .map_err(|error| format!("MusicXML serialize failed: {error}"))?;
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{body}").into_bytes())
}

fn parse_optimized_score(data: &[u8]) -> Result<LoadedScore, String> {
    let score: Score =
        serde_json::from_slice(data).map_err(|error| format!("Score parse failed: {error}"))?;
    Ok(LoadedScore {
        score,
        layout: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample files live at the workspace root; tests run with the crate dir
    /// as CWD.
    fn sample(relative: &str) -> Vec<u8> {
        let path = format!("../{relative}");
        std::fs::read(&path).unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
    }

    fn legacy_header(version: &str) -> Vec<u8> {
        let body = format!("FICHIER GUITAR PRO v{version}");
        assert_eq!(body.len(), 24, "header must be exactly 24 bytes");
        let mut out = vec![0x18];
        out.extend_from_slice(body.as_bytes());
        out.extend_from_slice(&[0; 8]);
        out
    }

    #[test]
    fn maps_every_raw_abi_value() {
        let expected = [
            (0, GpioFormat::Auto),
            (1, GpioFormat::Gp3),
            (2, GpioFormat::Gp4),
            (3, GpioFormat::Gp5),
            (4, GpioFormat::Gpx),
            (5, GpioFormat::Gp),
            (6, GpioFormat::MusicXml),
            (7, GpioFormat::Score),
            (8, GpioFormat::Mscz),
        ];
        for (raw, variant) in expected {
            assert_eq!(GpioFormat::from_raw(raw), Some(variant));
            assert_eq!(variant as u32, raw, "discriminant is part of the ABI");
        }
    }

    #[test]
    fn rejects_out_of_range_format_codes() {
        assert_eq!(GpioFormat::from_raw(9), None);
        assert_eq!(GpioFormat::from_raw(u32::MAX), None);
    }

    #[test]
    fn detects_legacy_versions_from_the_header() {
        assert_eq!(
            detect_legacy_version(&legacy_header("3.00")),
            Some((3, 0, 0))
        );
        assert_eq!(
            detect_legacy_version(&legacy_header("5.10")),
            Some((5, 1, 0))
        );
        assert_eq!(detect(&legacy_header("3.00")), Some(GpioFormat::Gp3));
        assert_eq!(detect(&legacy_header("4.06")), Some(GpioFormat::Gp4));
        assert_eq!(detect(&legacy_header("5.10")), Some(GpioFormat::Gp5));
        // GP1/GP2 are read through the GP3 path.
        assert_eq!(detect(&legacy_header("2.21")), Some(GpioFormat::Gp3));
    }

    #[test]
    fn rejects_unknown_legacy_major_versions() {
        assert_eq!(detect(&legacy_header("9.00")), None);
    }

    #[test]
    fn ignores_truncated_or_foreign_headers() {
        assert_eq!(detect_legacy_version(b"short"), None);
        assert_eq!(
            detect_legacy_version(b"                <DIV class=Sect>"),
            None
        );
    }

    #[test]
    fn detects_real_sample_files() {
        assert_eq!(detect(&sample("test/Chords.gp3")), Some(GpioFormat::Gp3));
        assert_eq!(detect(&sample("test/Chords.gp4")), Some(GpioFormat::Gp4));
        assert_eq!(detect(&sample("test/Chords.gp5")), Some(GpioFormat::Gp5));
        assert_eq!(detect(&sample("test/accent.gpx")), Some(GpioFormat::Gpx));
        assert_eq!(detect(&sample("test/accent.gp")), Some(GpioFormat::Gp));
        assert_eq!(
            detect(&sample("guitarpro/samples/mscz/simple_monophonic.mscz")),
            Some(GpioFormat::Mscz),
            "the container.xml manifest must separate MSCZ from .gp"
        );
    }

    #[test]
    fn detects_textual_formats() {
        assert_eq!(
            detect(b"<?xml version=\"1.0\"?><score-partwise/>"),
            Some(GpioFormat::MusicXml)
        );
        assert_eq!(
            detect(b"<score-partwise version=\"4.0\"/>"),
            Some(GpioFormat::MusicXml)
        );
        assert_eq!(detect(b"{\"metadata\": {}}"), Some(GpioFormat::Score));
        // Leading whitespace and a UTF-8 BOM must not defeat the sniff.
        assert_eq!(
            detect("\u{feff}  {\"a\":1}".as_bytes()),
            Some(GpioFormat::Score)
        );
    }

    #[test]
    fn returns_none_for_unrecognised_content() {
        assert_eq!(detect(b""), None);
        assert_eq!(detect(b"just some prose"), None);
        assert_eq!(detect(&[0xff; 64]), None);
    }

    #[test]
    fn decode_resolves_auto_to_the_detected_format() {
        let (_, resolved) = decode(&sample("test/Chords.gp4"), GpioFormat::Auto)
            .unwrap_or_else(|error| panic!("decode failed: {error}"));
        assert_eq!(resolved, GpioFormat::Gp4);
    }

    #[test]
    fn decode_reports_undetectable_input() {
        let error = decode(b"not a score", GpioFormat::Auto).unwrap_err();
        assert!(
            error.contains("cannot detect"),
            "unexpected message: {error}"
        );
    }

    #[test]
    fn decode_reports_a_format_mismatch() {
        // A GPX container forced through the GP3 reader must fail loudly
        // rather than produce a half-parsed song.
        //
        // Note the legacy readers are deliberately lenient with each other:
        // GP5 bytes fed to `read_gp3` parse without complaint, because the
        // formats share a prefix. Only a genuinely foreign container trips
        // the error path.
        let error = decode(&sample("test/accent.gpx"), GpioFormat::Gp3)
            .err()
            .unwrap_or_else(|| "no error".to_string());
        assert!(error.contains("GP3"), "unexpected message: {error}");
    }

    #[test]
    fn decode_enforces_the_size_limit() {
        let oversized = vec![0u8; GpioFormat::Gp5.max_input_size() + 1];
        let error = decode(&oversized, GpioFormat::Gp5).unwrap_err();
        assert!(error.contains("over the"), "unexpected message: {error}");
    }

    #[test]
    fn decode_rejects_auto_after_resolution_failure() {
        // `Auto` must never reach the codec match arm.
        assert!(decode(b"", GpioFormat::Auto).is_err());
    }

    #[test]
    fn encode_rejects_auto_as_a_target() {
        let (song, _) = decode(&sample("test/Chords.gp5"), GpioFormat::Gp5)
            .unwrap_or_else(|error| panic!("decode failed: {error}"));
        let error = encode(&song, GpioFormat::Auto).unwrap_err();
        assert!(error.contains("AUTO"), "unexpected message: {error}");
    }

    #[test]
    fn encodes_every_output_format() {
        let (song, _) = decode(&sample("test/Chords.gp5"), GpioFormat::Gp5)
            .unwrap_or_else(|error| panic!("decode failed: {error}"));

        for target in [
            GpioFormat::Gp3,
            GpioFormat::Gp4,
            GpioFormat::Gp5,
            GpioFormat::Gpx,
            GpioFormat::Gp,
            GpioFormat::MusicXml,
            GpioFormat::Score,
            GpioFormat::Mscz,
        ] {
            let bytes = encode(&song, target)
                .unwrap_or_else(|error| panic!("{} encode failed: {error}", target.label()));
            assert!(
                !bytes.is_empty(),
                "{} produced an empty buffer",
                target.label()
            );
        }
    }

    #[test]
    fn encoded_output_is_detected_as_its_own_format() {
        let (song, _) = decode(&sample("test/Chords.gp5"), GpioFormat::Gp5)
            .unwrap_or_else(|error| panic!("decode failed: {error}"));

        for target in [
            GpioFormat::Gp3,
            GpioFormat::Gp4,
            GpioFormat::Gp5,
            GpioFormat::Gpx,
            GpioFormat::Gp,
            GpioFormat::MusicXml,
            GpioFormat::Score,
            GpioFormat::Mscz,
        ] {
            let bytes = encode(&song, target)
                .unwrap_or_else(|error| panic!("{} encode failed: {error}", target.label()));
            assert_eq!(
                detect(&bytes),
                Some(target),
                "{} output was not detected as itself",
                target.label()
            );
        }
    }

    #[test]
    fn decodes_what_it_encoded_for_every_format() {
        let (song, _) = decode(&sample("test/Chords.gp5"), GpioFormat::Gp5)
            .unwrap_or_else(|error| panic!("decode failed: {error}"));

        // MusicXML is absent on purpose: `song_to_score_partwise` currently
        // emits every absent optional attribute as `attr=""`, which its own
        // importer rejects ("invalid type: string \"\", expected u16"). That
        // is a pre-existing defect in the guitarpro MusicXML model — it
        // reproduces through `score_tool convert` with no FFI involved — so
        // this crate records the gap rather than asserting a property the
        // library does not yet hold. Re-add the variant once it is fixed.
        for target in [
            GpioFormat::Gp3,
            GpioFormat::Gp4,
            GpioFormat::Gp5,
            GpioFormat::Gpx,
            GpioFormat::Gp,
            GpioFormat::Score,
            GpioFormat::Mscz,
        ] {
            let bytes = encode(&song, target)
                .unwrap_or_else(|error| panic!("{} encode failed: {error}", target.label()));
            let round_tripped = decode(&bytes, target);
            assert!(
                round_tripped.is_ok(),
                "{} output did not decode back: {:?}",
                target.label(),
                round_tripped.err()
            );
        }
    }

    #[test]
    fn strips_a_doctype_declaration() {
        let xml = "<?xml version=\"1.0\"?><!DOCTYPE score-partwise PUBLIC \"x\" \"y\"><root/>";
        assert_eq!(strip_doctype(xml), "<?xml version=\"1.0\"?><root/>");
    }

    #[test]
    fn leaves_xml_without_a_doctype_untouched() {
        let xml = "<?xml version=\"1.0\"?><root/>";
        assert_eq!(strip_doctype(xml), xml);
    }

    #[test]
    fn reports_invalid_musicxml_and_json() {
        assert!(decode(b"<?xml version=\"1.0\"?><nope", GpioFormat::MusicXml).is_err());
        assert!(decode(b"{ not json", GpioFormat::Score).is_err());
        assert!(decode(&[0xff, 0xfe, 0xfd], GpioFormat::MusicXml).is_err());
    }

    #[test]
    fn reports_a_corrupt_mscz_archive() {
        assert!(decode(b"PK\x03\x04garbage", GpioFormat::Mscz).is_err());
    }

    #[test]
    fn every_variant_has_a_label() {
        for raw in 0..=8u32 {
            let variant = GpioFormat::from_raw(raw).unwrap_or(GpioFormat::Auto);
            assert!(!variant.label().is_empty());
        }
    }
}
