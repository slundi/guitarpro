use std::collections::HashSet;

use bpaf::Bpaf;

use crate::command_convert::{ConvertFormats, resolve_formats};

/// Extract one or more tracks into a new score file
#[derive(Bpaf, Debug)]
#[bpaf(command("extract"))]
pub struct ExtractArgs {
    /// Input score file
    #[bpaf(short, long, argument("PATH"))]
    pub input: String,

    /// Output file (format auto-detected from extension, or use --to)
    #[bpaf(short, long, argument("PATH"))]
    pub output: String,

    /// Select tracks by name (comma-separated, case-insensitive substring match)
    #[bpaf(long, argument("NAMES"))]
    pub tracks: Option<String>,

    /// Select tracks by 0-based index (comma-separated, e.g. "0,2")
    #[bpaf(long("track-index"), argument("INDICES"))]
    pub track_index: Option<String>,

    /// Keep all tracks EXCEPT the selected ones
    #[bpaf(long, switch)]
    pub invert: bool,

    /// Force output format: gp3, gp4, gp5, gpx, gp, xml, score
    #[bpaf(long("to"), argument("FORMAT"))]
    pub to: Option<String>,

    /// Force input format: gp3, gp4, gp5, gpx, gp, xml, score
    #[bpaf(long("from"), argument("FORMAT"))]
    pub from: Option<String>,
}

pub fn run(args: &ExtractArgs) -> anyhow::Result<()> {
    if args.tracks.is_none() && args.track_index.is_none() {
        anyhow::bail!("specify at least one of --tracks or --track-index");
    }

    let ConvertFormats { src, dst } = resolve_formats(
        args.from.as_deref(),
        &args.input,
        args.to.as_deref(),
        &args.output,
    )?;

    // Load the source file
    let mut song = crate::command_convert::load_as_song(&args.input, src)?;

    let total = song.tracks.len();

    // Build the set of selected indices
    let mut selected: HashSet<usize> = HashSet::new();

    if let Some(names) = &args.tracks {
        for name in names.split(',') {
            let needle = name.trim().to_lowercase();
            for (i, track) in song.tracks.iter().enumerate() {
                if track.name.to_lowercase().contains(&needle) {
                    selected.insert(i);
                }
            }
        }
    }

    if let Some(indices) = &args.track_index {
        for part in indices.split(',') {
            let part = part.trim();
            let idx: usize = part
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid track index '{}': must be a number", part))?;
            if idx >= total {
                anyhow::bail!(
                    "track index {} out of range (file has {} track(s), indices 0..{})",
                    idx,
                    total,
                    total.saturating_sub(1)
                );
            }
            selected.insert(idx);
        }
    }

    // Apply --invert
    let keep: Vec<bool> = (0..total)
        .map(|i| {
            if args.invert {
                !selected.contains(&i)
            } else {
                selected.contains(&i)
            }
        })
        .collect();

    let kept_count = keep.iter().filter(|&&k| k).count();
    if kept_count == 0 {
        anyhow::bail!("no tracks match the given criteria — nothing to extract");
    }

    // Report what we're keeping
    let kept_names: Vec<&str> = song
        .tracks
        .iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, t)| t.name.as_str())
        .collect();
    eprintln!(
        "Extracting {}/{} track(s): {}",
        kept_count,
        total,
        kept_names.join(", ")
    );

    // Filter tracks in-place and renumber
    let mut new_tracks: Vec<_> = song
        .tracks
        .into_iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, t)| t)
        .collect();

    for (i, track) in new_tracks.iter_mut().enumerate() {
        track.number = (i + 1) as i32;
    }
    song.tracks = new_tracks;

    // Encode and write
    let bytes = crate::command_convert::encode_song(&song, dst)?;
    std::fs::write(&args.output, &bytes)
        .map_err(|e| anyhow::anyhow!("cannot write '{}': {}", args.output, e))?;

    eprintln!("Written: {}", args.output);
    Ok(())
}
