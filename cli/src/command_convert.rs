use std::fs;
use std::path::Path;

use bpaf::Bpaf;
use guitarpro::Song;
use guitarpro::convert::guitarpro::musicxml_to_legacy_song;
use guitarpro::convert::legacy::loaded_score_to_legacy_song;
use guitarpro::convert::mscz::{loaded_score_to_mscx, mscx_to_loaded_score};
use guitarpro::convert::musicxml::song_to_score_partwise;
use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;
use guitarpro::io::mscz::{read_mscz_bytes, write_mscz};
use guitarpro::model::musicxml::ScorePartwise;
use guitarpro::model::optimized::global::Score;
use guitarpro::{Mscx, MsczArchive, MsczEntry, MsczFile};

/// Convert a score file between formats (GP3/4/5/GPX/GP, MusicXML, Optimized)
#[derive(Bpaf, Debug)]
#[bpaf(command("convert"))]
pub struct ConvertArgs {
    /// Input file or directory
    #[bpaf(short, long, argument("PATH"))]
    pub input: String,

    /// Output file or directory
    #[bpaf(short, long, argument("PATH"))]
    pub output: String,

    /// Force output format: gp3, gp4, gp5, gpx, gp, xml, score
    #[bpaf(long("to"), argument("FORMAT"))]
    pub to: Option<String>,

    /// Force input format: gp3, gp4, gp5, gpx, gp, xml, score
    #[bpaf(long("from"), argument("FORMAT"))]
    pub from: Option<String>,

    /// Show what would be done without writing any files
    #[bpaf(long, switch)]
    pub dry_run: bool,
}

// ---------------------------------------------------------------------------
// Format enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Format {
    Gp3,
    Gp4,
    Gp5,
    Gpx,
    Gp,
    MusicXml,
    Score,
    Mscz,
}

impl Format {
    fn from_ext(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "gp3" => Some(Self::Gp3),
            "gp4" => Some(Self::Gp4),
            "gp5" => Some(Self::Gp5),
            "gpx" => Some(Self::Gpx),
            "gp" => Some(Self::Gp),
            "xml" | "musicxml" => Some(Self::MusicXml),
            "score" | "msor" => Some(Self::Score),
            "mscz" => Some(Self::Mscz),
            _ => None,
        }
    }

    fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gp3" => Some(Self::Gp3),
            "gp4" => Some(Self::Gp4),
            "gp5" => Some(Self::Gp5),
            "gpx" => Some(Self::Gpx),
            "gp" | "gp7" => Some(Self::Gp),
            "xml" | "musicxml" => Some(Self::MusicXml),
            "score" | "msor" | "opt" => Some(Self::Score),
            "mscz" | "musescore" => Some(Self::Mscz),
            _ => None,
        }
    }

    fn default_ext(self) -> &'static str {
        match self {
            Self::Gp3 => "gp3",
            Self::Gp4 => "gp4",
            Self::Gp5 => "gp5",
            Self::Gpx => "gpx",
            Self::Gp => "gp",
            Self::MusicXml => "musicxml",
            Self::Score => "score",
            Self::Mscz => "mscz",
        }
    }

    fn label(self) -> &'static str {
        match self {
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

    /// GP version tuple used when calling `Song::write`.
    fn gp_write_version(self) -> Option<(u8, u8, u8)> {
        match self {
            Self::Gp3 => Some((3, 0, 0)),
            Self::Gp4 => Some((4, 0, 0)),
            Self::Gp5 => Some((5, 1, 0)),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: &ConvertArgs) -> anyhow::Result<()> {
    let input_path = Path::new(&args.input);

    if input_path.is_dir() {
        run_batch(args)
    } else {
        run_single(args)
    }
}

// ---------------------------------------------------------------------------
// Single-file conversion
// ---------------------------------------------------------------------------

fn run_single(args: &ConvertArgs) -> anyhow::Result<()> {
    let src_fmt = resolve_input_format(args.from.as_deref(), &args.input)?;
    let dst_fmt = resolve_output_format(args.to.as_deref(), &args.output, false)?;
    let out_path = output_path_for(&args.input, &args.output, dst_fmt);

    if args.dry_run {
        println!(
            "[dry-run] {}  →  {}  ({} → {})",
            args.input,
            out_path,
            src_fmt.label(),
            dst_fmt.label()
        );
        return Ok(());
    }

    eprintln!("Converting {} → {} ...", args.input, out_path);
    convert_file(&args.input, &out_path, src_fmt, dst_fmt)?;
    eprintln!("Done: {}", out_path);
    Ok(())
}

// ---------------------------------------------------------------------------
// Batch-directory conversion
// ---------------------------------------------------------------------------

fn run_batch(args: &ConvertArgs) -> anyhow::Result<()> {
    let dst_fmt = resolve_output_format(args.to.as_deref(), &args.output, true)?;
    let out_dir = Path::new(&args.output);

    if !args.dry_run {
        fs::create_dir_all(out_dir).map_err(|e| {
            anyhow::anyhow!(
                "cannot create output directory '{}': {}",
                out_dir.display(),
                e
            )
        })?;
    }

    let mut entries: Vec<_> = fs::read_dir(&args.input)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut converted = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for entry in entries {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let input_str = path.to_string_lossy().into_owned();

        let src_fmt = if let Some(f) = args.from.as_deref().and_then(Format::from_name) {
            f
        } else if let Some(f) = Format::from_ext(ext) {
            f
        } else {
            skipped += 1;
            continue;
        };

        let out_path = output_path_for(&input_str, &args.output, dst_fmt);

        if args.dry_run {
            println!(
                "[dry-run] {}  →  {}  ({})",
                input_str,
                out_path,
                dst_fmt.label()
            );
            converted += 1;
            continue;
        }

        eprint!("  {} → {} ... ", input_str, out_path);
        match convert_file(&input_str, &out_path, src_fmt, dst_fmt) {
            Ok(()) => {
                eprintln!("ok");
                converted += 1;
            }
            Err(e) => {
                eprintln!("FAILED: {e:#}");
                failed += 1;
            }
        }
    }

    eprintln!(
        "\nDone: {} converted, {} skipped, {} failed",
        converted, skipped, failed
    );
    if failed > 0 {
        anyhow::bail!("{} file(s) failed to convert", failed);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Core conversion logic
// ---------------------------------------------------------------------------

fn convert_file(input: &str, output: &str, src: Format, dst: Format) -> anyhow::Result<()> {
    let song = load_as_song(input, src)?;
    let bytes = encode_song(&song, dst)?;
    fs::write(output, &bytes).map_err(|e| anyhow::anyhow!("cannot write '{}': {}", output, e))
}

/// Read any supported input format and return a legacy `Song` as the
/// universal intermediate representation.
pub(crate) fn load_as_song(path: &str, fmt: Format) -> anyhow::Result<Song> {
    match fmt {
        Format::MusicXml => {
            let data = read_bytes(path)?;
            let sp = parse_musicxml(&data)?;
            Ok(musicxml_to_legacy_song(&sp))
        }
        Format::Score => {
            let text = fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("cannot read '{}': {}", path, e))?;
            let score: Score = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("cannot parse score '{}': {}", path, e))?;
            let loaded = guitarpro::model::optimized::LoadedScore {
                score,
                layout: None,
            };
            Ok(loaded_score_to_legacy_song(&loaded))
        }
        Format::Mscz => {
            let data = read_bytes(path)?;
            let file = read_mscz_bytes(&data)
                .map_err(|e| anyhow::anyhow!("cannot read MSCZ '{}': {}", path, e))?;
            let outcome = mscx_to_loaded_score(&file.mscx);
            Ok(loaded_score_to_legacy_song(&outcome.score))
        }
        _ => {
            let (song, _) = crate::loader::load_song(path)?;
            Ok(song)
        }
    }
}

/// Encode a legacy `Song` into the bytes for the target format.
pub(crate) fn encode_song(song: &Song, dst: Format) -> anyhow::Result<Vec<u8>> {
    match dst {
        Format::Gp3 | Format::Gp4 | Format::Gp5 => {
            let ver = dst.gp_write_version().unwrap();
            song.write(ver, None)
                .map_err(|e| anyhow::anyhow!("{} encode failed: {}", dst.label(), e))
        }
        Format::Gpx => song
            .write_gpx()
            .map_err(|e| anyhow::anyhow!("GPX encode failed: {}", e)),
        Format::Gp => song
            .write_gp()
            .map_err(|e| anyhow::anyhow!("GP encode failed: {}", e)),
        Format::MusicXml => {
            let sp = song_to_score_partwise(song);
            serialize_musicxml(&sp)
        }
        Format::Score => {
            let loaded = legacy_song_to_loaded_score(song);
            serde_json::to_vec_pretty(&loaded.score)
                .map_err(|e| anyhow::anyhow!("Score serialize failed: {}", e))
        }
        Format::Mscz => {
            let loaded = legacy_song_to_loaded_score(song);
            let mscx = loaded_score_to_mscx(&loaded);
            let archive = mscz_archive_for(&mscx);
            let file = MsczFile { archive, mscx };
            write_mscz(&file).map_err(|e| anyhow::anyhow!("MSCZ encode failed: {}", e))
        }
    }
}

/// Build a fresh MSCZ archive around a generated MSCX document.
///
/// Writes only the `META-INF/container.xml` manifest plus `score.mscx` — no
/// thumbnail, style, or side JSON. MuseScore reads such minimal archives.
fn mscz_archive_for(mscx: &Mscx) -> MsczArchive {
    let manifest = br#"<?xml version="1.0" encoding="UTF-8"?>
<container><rootfiles><rootfile full-path="score.mscx"/></rootfiles></container>"#;
    MsczArchive {
        rootfiles: vec!["score.mscx".to_string()],
        entries: vec![
            MsczEntry {
                path: "META-INF/container.xml".to_string(),
                data: manifest.to_vec(),
            },
            MsczEntry {
                path: "score.mscx".to_string(),
                data: mscx.raw_xml.as_bytes().to_vec(),
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Format detection helpers
// ---------------------------------------------------------------------------

/// Resolved source + destination format pair (used by extract as well).
pub(crate) struct ConvertFormats {
    pub src: Format,
    pub dst: Format,
}

/// Resolve both formats for a single-file (non-batch) operation.
pub(crate) fn resolve_formats(
    from: Option<&str>,
    input: &str,
    to: Option<&str>,
    output: &str,
) -> anyhow::Result<ConvertFormats> {
    Ok(ConvertFormats {
        src: resolve_input_format(from, input)?,
        dst: resolve_output_format(to, output, false)?,
    })
}

fn resolve_input_format(flag: Option<&str>, path: &str) -> anyhow::Result<Format> {
    if let Some(s) = flag {
        return Format::from_name(s).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown input format '{}'. Valid: gp3, gp4, gp5, gpx, gp, xml, score, mscz",
                s
            )
        });
    }
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    Format::from_ext(ext).ok_or_else(|| {
        anyhow::anyhow!(
            "cannot detect input format from '.{}'. Use --from <format>",
            ext
        )
    })
}

fn resolve_output_format(
    flag: Option<&str>,
    path: &str,
    require_flag: bool,
) -> anyhow::Result<Format> {
    if let Some(s) = flag {
        return Format::from_name(s).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown output format '{}'. Valid: gp3, gp4, gp5, gpx, gp, xml, score, mscz",
                s
            )
        });
    }
    if require_flag {
        anyhow::bail!("batch mode requires --to <format>");
    }
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    Format::from_ext(ext).ok_or_else(|| {
        anyhow::anyhow!(
            "cannot detect output format from '.{}'. Use --to <format>",
            ext
        )
    })
}

/// Compute the final output file path.
///
/// If `output` is an existing directory or ends in a path separator,
/// the output file is placed inside it with the input stem and the
/// target format's default extension. Otherwise `output` is used as-is.
fn output_path_for(input: &str, output: &str, dst: Format) -> String {
    let out = Path::new(output);
    let is_dir =
        out.is_dir() || output.ends_with('/') || output.ends_with(std::path::MAIN_SEPARATOR);

    if is_dir {
        let stem = Path::new(input)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        out.join(format!("{}.{}", stem, dst.default_ext()))
            .to_string_lossy()
            .into_owned()
    } else {
        output.to_owned()
    }
}

// ---------------------------------------------------------------------------
// MusicXML I/O
// ---------------------------------------------------------------------------

fn read_bytes(path: &str) -> anyhow::Result<Vec<u8>> {
    fs::read(path).map_err(|e| anyhow::anyhow!("cannot read '{}': {}", path, e))
}

fn strip_doctype(xml: &str) -> String {
    if let Some(start) = xml.find("<!DOCTYPE")
        && let Some(rel_end) = xml[start..].find('>')
    {
        return format!("{}{}", &xml[..start], &xml[start + rel_end + 1..]);
    }
    xml.to_string()
}

fn parse_musicxml(data: &[u8]) -> anyhow::Result<ScorePartwise> {
    let raw = std::str::from_utf8(data)
        .map_err(|e| anyhow::anyhow!("MusicXML is not valid UTF-8: {}", e))?;
    let cleaned = strip_doctype(raw);
    quick_xml::de::from_str(&cleaned).map_err(|e| anyhow::anyhow!("MusicXML parse failed: {}", e))
}

fn serialize_musicxml(sp: &ScorePartwise) -> anyhow::Result<Vec<u8>> {
    let body = quick_xml::se::to_string(sp)
        .map_err(|e| anyhow::anyhow!("MusicXML serialize failed: {}", e))?;
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&body);
    Ok(out.into_bytes())
}
