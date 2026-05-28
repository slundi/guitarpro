use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::{fs, io};

use clap::Args;
use guitarpro::NoteType;
use serde::Serialize;

use crate::loader::load_song;

const GP_EXTENSIONS: &[&str] = &["gp3", "gp4", "gp5", "gp", "gpx"];

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct DuplicatesArgs {
    /// Directory to scan for score files
    #[arg(short, long)]
    pub dir: String,

    /// Similarity threshold 0.0–1.0 (default: 0.85)
    #[arg(long, default_value_t = 0.85)]
    pub threshold: f64,

    /// Print results as JSON
    #[arg(long)]
    pub json: bool,

    /// After reporting, delete all but the first file in each group (asks for confirmation)
    #[arg(long)]
    pub delete_keep_first: bool,

    /// Recurse into subdirectories
    #[arg(long, short = 'r')]
    pub recursive: bool,
}

// ---------------------------------------------------------------------------
// Fingerprint
// ---------------------------------------------------------------------------

struct Fingerprint {
    path: PathBuf,
    /// FNV-1a hash of the normalised note sequence (MIDI pitches, chord-sorted).
    content_hash: u64,
    /// Per-MIDI-pitch note count (0..128).
    note_hist: [u32; 128],
    measure_count: usize,
    title_norm: String,
    artist_norm: String,
}

impl Fingerprint {
    fn build(path: &std::path::Path) -> anyhow::Result<Self> {
        let (song, _fmt) = load_song(&path.to_string_lossy())?;

        let measure_count = song.measure_headers.len();
        let title_norm = song.name.trim().to_lowercase();
        let artist_norm = song.artist.trim().to_lowercase();

        let mut note_hist = [0u32; 128];
        let mut hash_bytes: Vec<u8> = Vec::new();

        for track in &song.tracks {
            for measure in &track.measures {
                for voice in &measure.voices {
                    for beat in &voice.beats {
                        // Collect MIDI pitches for this beat, sorted (chord-order-independent).
                        let mut midi_notes: Vec<u8> = beat
                            .notes
                            .iter()
                            .filter(|n| n.kind != NoteType::Rest && n.kind != NoteType::Tie)
                            .filter_map(|n| {
                                let midi = if track.percussion_track {
                                    // Percussion: value is the GM drum note directly.
                                    n.value
                                } else {
                                    // Fretted: fret + string open tuning = MIDI pitch.
                                    let s = n.string as usize;
                                    if s > 0 && s <= track.strings.len() {
                                        n.value + track.strings[s - 1].1 as i16
                                    } else {
                                        n.value
                                    }
                                };
                                if (0..128).contains(&midi) {
                                    Some(midi as u8)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        midi_notes.sort_unstable();

                        hash_bytes.extend_from_slice(&midi_notes);
                        hash_bytes.push(0xFE); // beat boundary

                        for &p in &midi_notes {
                            note_hist[p as usize] += 1;
                        }
                    }
                }
                hash_bytes.push(0xFD); // measure boundary
            }
            hash_bytes.push(0xFC); // track boundary
        }

        let content_hash = fnv1a(&hash_bytes);

        Ok(Fingerprint {
            path: path.to_path_buf(),
            content_hash,
            note_hist,
            measure_count,
            title_norm,
            artist_norm,
        })
    }
}

/// FNV-1a 64-bit hash.
fn fnv1a(data: &[u8]) -> u64 {
    const OFFSET: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    data.iter()
        .fold(OFFSET, |h, &b| (h ^ b as u64).wrapping_mul(PRIME))
}

// ---------------------------------------------------------------------------
// Similarity
// ---------------------------------------------------------------------------

/// Cosine similarity between two MIDI pitch histograms, 0.0..=1.0.
fn histogram_similarity(a: &[u32; 128], b: &[u32; 128]) -> f64 {
    let dot: u64 = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| x as u64 * y as u64)
        .sum();
    let mag_a: f64 = a.iter().map(|&x| (x as f64).powi(2)).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|&x| (x as f64).powi(2)).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot as f64 / (mag_a * mag_b)
}

/// Combined similarity score between two fingerprints, 0.0..=1.0.
fn similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    // Exact content match (normalised, format-independent).
    if a.content_hash == b.content_hash {
        return 1.0;
    }

    // Note-sequence cosine similarity (60%).
    let hist_sim = histogram_similarity(&a.note_hist, &b.note_hist);

    // Metadata match: exact title + artist (25%).
    let meta_sim = if !a.title_norm.is_empty()
        && a.title_norm == b.title_norm
        && a.artist_norm == b.artist_norm
    {
        1.0_f64
    } else {
        0.0
    };

    // Length proximity: 1 – relative measure-count difference (15%).
    let length_sim = match (a.measure_count, b.measure_count) {
        (0, 0) => 1.0,
        (0, _) | (_, 0) => 0.0,
        (ma, mb) => {
            let diff = (ma as f64 - mb as f64).abs();
            (1.0 - diff / ma.max(mb) as f64).max(0.0)
        }
    };

    0.60 * hist_sim + 0.25 * meta_sim + 0.15 * length_sim
}

// ---------------------------------------------------------------------------
// Grouping — union-find single-linkage clustering
// ---------------------------------------------------------------------------

fn find_root(parent: &mut [usize], mut i: usize) -> usize {
    // Two-pass path compression.
    let mut root = i;
    while parent[root] != root {
        root = parent[root];
    }
    while parent[i] != root {
        let next = parent[i];
        parent[i] = root;
        i = next;
    }
    root
}

/// Return groups (each with ≥2 members) of indices into `fps` that exceed `threshold`.
fn find_groups(fps: &[Fingerprint], threshold: f64) -> Vec<Vec<usize>> {
    let n = fps.len();
    let mut parent: Vec<usize> = (0..n).collect();

    for i in 0..n {
        for j in (i + 1)..n {
            if similarity(&fps[i], &fps[j]) >= threshold {
                let ri = find_root(&mut parent, i);
                let rj = find_root(&mut parent, j);
                if ri != rj {
                    parent[rj] = ri;
                }
            }
        }
    }

    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        clusters
            .entry(find_root(&mut parent, i))
            .or_default()
            .push(i);
    }

    let mut groups: Vec<Vec<usize>> = clusters.into_values().filter(|g| g.len() > 1).collect();
    // Sort groups by the smallest path in the group so output is deterministic.
    for g in &mut groups {
        g.sort_by_key(|&i| &fps[i].path);
    }
    groups.sort_by_key(|g| fps[g[0]].path.clone());
    groups
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: &DuplicatesArgs) -> anyhow::Result<()> {
    let threshold = args.threshold.clamp(0.0, 1.0);

    let files = collect_files(&args.dir, args.recursive)?;
    if files.is_empty() {
        eprintln!("No Guitar Pro files found in '{}'.", args.dir);
        return Ok(());
    }

    eprintln!("Scanning {} file(s)…", files.len());

    let mut fps: Vec<Fingerprint> = Vec::with_capacity(files.len());
    for path in &files {
        match Fingerprint::build(path) {
            Ok(fp) => fps.push(fp),
            Err(e) => eprintln!("  skip {}: {}", path.display(), e),
        }
    }

    let groups = find_groups(&fps, threshold);

    if groups.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!("No duplicates found (threshold {:.0}%).", threshold * 100.0);
        }
        return Ok(());
    }

    if args.json {
        print_json(&fps, &groups)?;
    } else {
        print_text(&fps, &groups, threshold);
    }

    if args.delete_keep_first {
        delete_keep_first(&fps, &groups)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// File collection
// ---------------------------------------------------------------------------

fn collect_files(dir: &str, recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let mut stack = vec![PathBuf::from(dir)];

    while let Some(d) = stack.pop() {
        let entries = fs::read_dir(&d)
            .map_err(|e| anyhow::anyhow!("cannot read '{}': {}", d.display(), e))?;
        let mut children: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        children.sort();
        for path in children {
            if path.is_dir() && recursive {
                stack.push(path);
            } else if is_gp_file(&path) {
                result.push(path);
            }
        }
    }

    result.sort();
    Ok(result)
}

fn is_gp_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| GP_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Text output
// ---------------------------------------------------------------------------

fn print_text(fps: &[Fingerprint], groups: &[Vec<usize>], threshold: f64) {
    println!(
        "Found {} duplicate group(s) (threshold {:.0}%):\n",
        groups.len(),
        threshold * 100.0
    );
    for (gi, group) in groups.iter().enumerate() {
        println!("Group {}:", gi + 1);
        for (rank, &idx) in group.iter().enumerate() {
            let label = if rank == 0 { "[keep]" } else { "[dup] " };
            let score = if rank == 0 {
                1.0
            } else {
                similarity(&fps[group[0]], &fps[idx])
            };
            println!(
                "  {}  {:5.1}%  {}",
                label,
                score * 100.0,
                fps[idx].path.display()
            );
        }
        println!();
    }
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct JsonGroup {
    files: Vec<JsonFile>,
}

#[derive(Serialize)]
struct JsonFile {
    path: String,
    similarity: f64,
    keep: bool,
}

fn print_json(fps: &[Fingerprint], groups: &[Vec<usize>]) -> anyhow::Result<()> {
    let output: Vec<JsonGroup> = groups
        .iter()
        .map(|group| JsonGroup {
            files: group
                .iter()
                .enumerate()
                .map(|(rank, &idx)| {
                    let score = if rank == 0 {
                        1.0
                    } else {
                        similarity(&fps[group[0]], &fps[idx])
                    };
                    JsonFile {
                        path: fps[idx].path.to_string_lossy().into_owned(),
                        similarity: (score * 1000.0).round() / 1000.0,
                        keep: rank == 0,
                    }
                })
                .collect(),
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

fn delete_keep_first(fps: &[Fingerprint], groups: &[Vec<usize>]) -> anyhow::Result<()> {
    let to_delete: Vec<&PathBuf> = groups
        .iter()
        .flat_map(|g| g.iter().skip(1).map(|&i| &fps[i].path))
        .collect();

    if to_delete.is_empty() {
        return Ok(());
    }

    eprintln!("\nFiles queued for deletion ({}):", to_delete.len());
    for p in &to_delete {
        eprintln!("  {}", p.display());
    }

    eprint!("\nDelete these {} file(s)? [y/N] ", to_delete.len());
    let stdin = io::stdin();
    let line = stdin.lock().lines().next().unwrap_or(Ok(String::new()))?;

    if line.trim().eq_ignore_ascii_case("y") {
        for p in &to_delete {
            fs::remove_file(p)
                .map_err(|e| anyhow::anyhow!("cannot delete '{}': {}", p.display(), e))?;
            eprintln!("  deleted: {}", p.display());
        }
        eprintln!("Done.");
    } else {
        eprintln!("Aborted — no files were deleted.");
    }

    Ok(())
}
