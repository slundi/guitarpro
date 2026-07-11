//! `score_tool mscz` — inspect and unpack MSCZ archives without going through
//! the score conversion pipeline. Useful when debugging conversion bugs where
//! you need to look at the raw MSCX + side files (thumbnails, `.mss` style)
//! bundled inside.

use std::fs;
use std::path::{Path, PathBuf};

use bpaf::Bpaf;
use guitarpro::io::mscz::read_container;

// ---------------------------------------------------------------------------
// CLI args (union of three sub-actions)
// ---------------------------------------------------------------------------

/// Inspect / unpack MSCZ (MuseScore compressed) archives
#[derive(Bpaf, Debug)]
#[bpaf(command("mscz"))]
pub struct MsczArgs {
    #[bpaf(external(mscz_action))]
    pub action: MsczAction,
}

#[derive(Bpaf, Debug)]
pub enum MsczAction {
    /// List archive entries with sizes
    #[bpaf(command("list"))]
    List {
        /// MSCZ file path
        #[bpaf(short, long, argument("FILE"))]
        input: String,
        /// Print entries as JSON
        #[bpaf(long, switch)]
        json: bool,
    },
    /// Extract all entries to a target directory
    #[bpaf(command("extract"))]
    Extract {
        /// MSCZ file path
        #[bpaf(short, long, argument("FILE"))]
        input: String,
        /// Output directory (created if missing)
        #[bpaf(short, long, argument("DIR"))]
        out: String,
    },
    /// Write the embedded PNG thumbnail to a file
    #[bpaf(command("thumbnail"))]
    Thumbnail {
        /// MSCZ file path
        #[bpaf(short, long, argument("FILE"))]
        input: String,
        /// Output PNG path (defaults to `<input>.thumbnail.png`)
        #[bpaf(long, argument("PATH"))]
        out: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: &MsczArgs) -> anyhow::Result<()> {
    match &args.action {
        MsczAction::List { input, json } => run_list(input, *json),
        MsczAction::Extract { input, out } => run_extract(input, out),
        MsczAction::Thumbnail { input, out } => run_thumbnail(input, out.as_deref()),
    }
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

fn run_list(input: &str, json: bool) -> anyhow::Result<()> {
    let data = fs::read(input).map_err(|e| anyhow::anyhow!("cannot read '{input}': {e}"))?;
    let archive =
        read_container(&data).map_err(|e| anyhow::anyhow!("cannot read MSCZ '{input}': {e}"))?;

    if json {
        let out: Vec<EntryJson> = archive
            .entries
            .iter()
            .map(|entry| EntryJson {
                path: entry.path.clone(),
                size: entry.data.len(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("Archive: {input}");
    if !archive.rootfiles.is_empty() {
        println!("Rootfiles:");
        for path in &archive.rootfiles {
            println!("  {path}");
        }
    }
    println!("Entries ({}):", archive.entries.len());
    let width = archive
        .entries
        .iter()
        .map(|e| e.path.len())
        .max()
        .unwrap_or(20);
    for entry in &archive.entries {
        println!(
            "  {:<width$}  {:>10} bytes",
            entry.path,
            entry.data.len(),
            width = width
        );
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct EntryJson {
    path: String,
    size: usize,
}

// ---------------------------------------------------------------------------
// extract
// ---------------------------------------------------------------------------

fn run_extract(input: &str, out_dir: &str) -> anyhow::Result<()> {
    let data = fs::read(input).map_err(|e| anyhow::anyhow!("cannot read '{input}': {e}"))?;
    let archive =
        read_container(&data).map_err(|e| anyhow::anyhow!("cannot read MSCZ '{input}': {e}"))?;

    let root = PathBuf::from(out_dir);
    fs::create_dir_all(&root)
        .map_err(|e| anyhow::anyhow!("cannot create '{}': {e}", root.display()))?;
    let root = root
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("cannot resolve '{}': {e}", out_dir))?;

    let mut written = 0usize;
    for entry in &archive.entries {
        let dest = safe_join(&root, &entry.path)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("cannot create '{}': {e}", parent.display()))?;
        }
        fs::write(&dest, &entry.data)
            .map_err(|e| anyhow::anyhow!("cannot write '{}': {e}", dest.display()))?;
        eprintln!("  {}  ({} bytes)", entry.path, entry.data.len());
        written += 1;
    }
    eprintln!("Extracted {written} entries to {}", root.display());
    Ok(())
}

/// Join `relative` onto `root`, rejecting any component that would escape.
///
/// Guards against ZIP-slip style paths (`../etc/passwd`, absolute paths,
/// Windows drive prefixes on Unix).
fn safe_join(root: &Path, relative: &str) -> anyhow::Result<PathBuf> {
    let candidate = PathBuf::from(relative);
    if candidate.is_absolute() {
        anyhow::bail!("entry '{relative}' is absolute — refusing to extract");
    }
    let mut out = root.to_path_buf();
    for component in candidate.components() {
        use std::path::Component::*;
        match component {
            Normal(seg) => out.push(seg),
            CurDir => {}
            ParentDir => {
                anyhow::bail!("entry '{relative}' contains '..' — refusing to extract");
            }
            Prefix(_) | RootDir => {
                anyhow::bail!("entry '{relative}' has a filesystem root — refusing to extract");
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// thumbnail
// ---------------------------------------------------------------------------

fn run_thumbnail(input: &str, out: Option<&str>) -> anyhow::Result<()> {
    let data = fs::read(input).map_err(|e| anyhow::anyhow!("cannot read '{input}': {e}"))?;
    let archive =
        read_container(&data).map_err(|e| anyhow::anyhow!("cannot read MSCZ '{input}': {e}"))?;

    let thumbnail = archive
        .thumbnail_entry()
        .ok_or_else(|| anyhow::anyhow!("no thumbnail (Thumbnails/thumbnail.png) in '{input}'"))?;

    let dest = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{input}.thumbnail.png")));

    fs::write(&dest, &thumbnail.data)
        .map_err(|e| anyhow::anyhow!("cannot write '{}': {e}", dest.display()))?;
    eprintln!("Wrote thumbnail: {}", dest.display());
    Ok(())
}
