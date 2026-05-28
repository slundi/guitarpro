use std::collections::{BTreeSet, HashMap};

use clap::Args;
use guitarpro::{MeasureHeader, NoteType};
use serde::Serialize;

use crate::loader::load_song;

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct FormArgs {
    /// Input score file
    #[arg(short, long)]
    pub input: String,

    /// Print results as JSON
    #[arg(long)]
    pub json: bool,

    /// Restrict analysis to tracks whose name contains this substring (case-insensitive)
    #[arg(long)]
    pub track: Option<String>,

    /// Similarity threshold for coarse clustering, 0.0–1.0 (default: 0.75)
    #[arg(long, default_value_t = 0.75)]
    pub threshold: f64,

    /// Similarity threshold above which a section is treated as an exact (non-variant)
    /// repetition of its cluster exemplar, 0.0–1.0 (default: 0.90)
    #[arg(long, default_value_t = 0.90)]
    pub variant_threshold: f64,

    /// Minimum bars per auto-detected section (default: 2)
    #[arg(long, default_value_t = 2)]
    pub min_section: usize,

    /// Fixed-window size for fallback segmentation when no markers or repeats are
    /// present (default: 0 = auto)
    #[arg(long, default_value_t = 0)]
    pub window: usize,
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Section {
    /// 0-based measure index, inclusive
    start: usize,
    /// 0-based measure index, exclusive
    end: usize,
    /// Marker title at the start bar, if any
    name: Option<String>,
}

#[derive(Debug)]
struct SectionFp {
    /// Pitch-class histogram, normalized so values sum to 1.0
    pitch_class: [f64; 12],
    note_count: usize,
    len: usize,
}

#[derive(Debug)]
struct LabeledSection {
    start: usize,
    end: usize,
    name: Option<String>,
    /// Cluster letter + optional prime(s), e.g. "A", "B", "A'", "A''"
    label: String,
}

impl LabeledSection {
    /// 1-based bar range (start, end_inclusive)
    fn bar_range(&self) -> (usize, usize) {
        (self.start + 1, self.end)
    }
}

struct TrackForm {
    track_name: String,
    sections: Vec<LabeledSection>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: &FormArgs) -> anyhow::Result<()> {
    let threshold = args.threshold.clamp(0.0, 1.0);
    let variant_threshold = args.variant_threshold.clamp(threshold, 1.0);

    let (song, _fmt) = load_song(&args.input)?;
    let headers = &song.measure_headers;

    if headers.is_empty() {
        println!("No measures found.");
        return Ok(());
    }

    let sections = segment(headers, args.min_section, args.window);

    let mut track_forms: Vec<TrackForm> = Vec::new();

    for track in &song.tracks {
        if args
            .track
            .as_deref()
            .is_some_and(|f| !track.name.to_lowercase().contains(&f.to_lowercase()))
        {
            continue;
        }

        let fps: Vec<SectionFp> = sections
            .iter()
            .map(|s| build_fingerprint(track, s.start, s.end))
            .collect();

        let sims = similarity_matrix(&fps);
        let labeled = assign_labels(&sections, &fps, &sims, threshold, variant_threshold);

        track_forms.push(TrackForm {
            track_name: track.name.clone(),
            sections: labeled,
        });
    }

    if track_forms.is_empty() {
        println!("No matching tracks found.");
        return Ok(());
    }

    if args.json {
        print_json(&args.input, &track_forms)?;
    } else {
        print_text(&args.input, &track_forms, headers.len());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Segmentation
// ---------------------------------------------------------------------------

fn segment(headers: &[MeasureHeader], min_len: usize, window: usize) -> Vec<Section> {
    let n = headers.len();
    if n == 0 {
        return vec![];
    }

    let mut break_set: BTreeSet<usize> = BTreeSet::new();
    break_set.insert(0);
    break_set.insert(n);

    // Strategy 1: markers
    let mut has_markers = false;
    for (i, h) in headers.iter().enumerate() {
        if h.marker.is_some() && i > 0 {
            break_set.insert(i);
            has_markers = true;
        }
    }

    // Strategy 2: repeat signs (when no markers)
    let has_repeats = headers.iter().any(|h| h.repeat_open || h.repeat_close >= 0);
    if !has_markers && has_repeats {
        for (i, h) in headers.iter().enumerate() {
            if h.repeat_open && i > 0 {
                break_set.insert(i);
            }
            if h.repeat_close >= 0 && i + 1 < n {
                break_set.insert(i + 1);
            }
        }
    }

    let mut breaks: Vec<usize> = break_set.into_iter().collect();

    // Strategy 3: fixed-window fallback when no segmentation was found
    if breaks.len() <= 2 {
        let w = if window > 0 {
            window
        } else {
            // auto window: ~4 sections, min 4 bars
            (n / 4).clamp(4, 16)
        };
        breaks = (0..=n).step_by(w).collect();
        if *breaks.last().unwrap() != n {
            breaks.push(n);
        }
        breaks.dedup();
    }

    // Merge sections shorter than min_len into their successor
    breaks = merge_short(&breaks, min_len, n);

    build_sections(&breaks, headers)
}

/// Merge any segment [a,b) with b-a < min_len into the next segment.
fn merge_short(breaks: &[usize], min_len: usize, total: usize) -> Vec<usize> {
    if breaks.len() <= 2 {
        return breaks.to_vec();
    }
    let min_len = min_len.max(1);
    let mut out: Vec<usize> = vec![0];
    // Collect lengths
    for w in breaks.windows(2) {
        let len = w[1] - w[0];
        if len < min_len {
            // skip this break (merge into previous or next)
            continue;
        }
        out.push(w[1]);
    }
    if *out.last().unwrap() != total {
        out.push(total);
    }
    out.dedup();
    if out.len() <= 1 {
        return vec![0, total];
    }
    out
}

fn build_sections(breaks: &[usize], headers: &[MeasureHeader]) -> Vec<Section> {
    breaks
        .windows(2)
        .map(|w| {
            let start = w[0];
            let end = w[1];
            let name = headers[start].marker.as_ref().map(|m| clean_str(&m.title));
            Section { start, end, name }
        })
        .collect()
}

/// Strip ASCII control characters from a string (GP marker titles include a length prefix byte).
fn clean_str(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_owned()
}

// ---------------------------------------------------------------------------
// Fingerprint
// ---------------------------------------------------------------------------

fn build_fingerprint(track: &guitarpro::Track, start: usize, end: usize) -> SectionFp {
    let mut pc = [0u32; 12];
    let mut note_count = 0usize;

    for mi in start..end.min(track.measures.len()) {
        for voice in &track.measures[mi].voices {
            for beat in &voice.beats {
                for note in &beat.notes {
                    if note.kind == NoteType::Rest || note.kind == NoteType::Tie {
                        continue;
                    }
                    let midi = if track.percussion_track {
                        note.value
                    } else {
                        let s = note.string as usize;
                        if s > 0 && s <= track.strings.len() {
                            note.value + track.strings[s - 1].1 as i16
                        } else {
                            note.value
                        }
                    };
                    if midi >= 0 {
                        pc[(midi % 12) as usize] += 1;
                        note_count += 1;
                    }
                }
            }
        }
    }

    let total: u32 = pc.iter().sum();
    let pitch_class = if total > 0 {
        pc.map(|c| c as f64 / total as f64)
    } else {
        [0.0f64; 12]
    };

    SectionFp {
        pitch_class,
        note_count,
        len: end - start,
    }
}

// ---------------------------------------------------------------------------
// Similarity
// ---------------------------------------------------------------------------

fn fp_similarity(a: &SectionFp, b: &SectionFp) -> f64 {
    // Empty sections: identical if both empty, dissimilar otherwise.
    if a.note_count == 0 || b.note_count == 0 {
        return if a.note_count == b.note_count {
            1.0
        } else {
            0.0
        };
    }

    // Pitch-class cosine similarity
    let dot: f64 = a
        .pitch_class
        .iter()
        .zip(b.pitch_class.iter())
        .map(|(x, y)| x * y)
        .sum();
    let mag_a: f64 = a.pitch_class.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.pitch_class.iter().map(|x| x * x).sum::<f64>().sqrt();
    let pc_sim = if mag_a * mag_b > 0.0 {
        dot / (mag_a * mag_b)
    } else {
        0.0
    };

    // Length similarity (1 – relative difference)
    let la = a.len as f64;
    let lb = b.len as f64;
    let len_sim = 1.0 - (la - lb).abs() / la.max(lb).max(1.0);

    0.80 * pc_sim + 0.20 * len_sim
}

fn similarity_matrix(fps: &[SectionFp]) -> Vec<Vec<f64>> {
    let n = fps.len();
    let mut m = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        m[i][i] = 1.0;
        for j in (i + 1)..n {
            let s = fp_similarity(&fps[i], &fps[j]);
            m[i][j] = s;
            m[j][i] = s;
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Clustering and label assignment
// ---------------------------------------------------------------------------

fn find_root(parent: &mut [usize], mut i: usize) -> usize {
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

/// Union-find clustering; returns the root/cluster-id for each element.
fn cluster(sims: &[Vec<f64>], threshold: f64) -> Vec<usize> {
    let n = sims.len();
    let mut parent: Vec<usize> = (0..n).collect();
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        for j in (i + 1)..n {
            if sims[i][j] >= threshold {
                let ri = find_root(&mut parent, i);
                let rj = find_root(&mut parent, j);
                if ri != rj {
                    parent[rj] = ri;
                }
            }
        }
    }
    (0..n).map(|i| find_root(&mut parent, i)).collect()
}

fn cluster_label(idx: usize) -> String {
    // A-Z, then AA, AB, …
    if idx < 26 {
        char::from(b'A' + idx as u8).to_string()
    } else {
        format!(
            "{}{}",
            char::from(b'A' + (idx / 26 - 1) as u8),
            char::from(b'A' + (idx % 26) as u8)
        )
    }
}

fn assign_labels(
    sections: &[Section],
    _fps: &[SectionFp],
    sims: &[Vec<f64>],
    threshold: f64,
    variant_threshold: f64,
) -> Vec<LabeledSection> {
    let n = sections.len();
    if n == 0 {
        return vec![];
    }

    // --- Coarse clustering ---
    let coarse = cluster(sims, threshold);

    // Assign base letters in order of first appearance.
    let mut root_to_letter: HashMap<usize, usize> = HashMap::new();
    let mut letter_counter = 0usize;
    for &root in &coarse {
        root_to_letter.entry(root).or_insert_with(|| {
            let l = letter_counter;
            letter_counter += 1;
            l
        });
    }

    // --- Variant detection within each coarse cluster ---
    // For each cluster, track sub-groups using variant_threshold.
    // A new sub-group is started when sim to every previous sub-group exemplar < variant_threshold.
    // variant_group[i] = index of sub-group within its cluster (0 = exact, 1 = first variant, …)
    let mut variant_group: Vec<usize> = vec![0; n];

    // cluster_root → list of (exemplar_section_idx, sub_group_idx) in encounter order
    let mut cluster_subgroups: HashMap<usize, Vec<usize>> = HashMap::new();

    for i in 0..n {
        let root = coarse[i];
        let subgroups = cluster_subgroups.entry(root).or_default();

        // Check against each sub-group's exemplar
        let mut found = None;
        for (sg_idx, &exemplar) in subgroups.iter().enumerate() {
            if sims[exemplar][i] >= variant_threshold {
                found = Some(sg_idx);
                break;
            }
        }
        if let Some(sg_idx) = found {
            variant_group[i] = sg_idx;
        } else {
            let sg_idx = subgroups.len();
            subgroups.push(i); // this section is the exemplar of the new sub-group
            variant_group[i] = sg_idx;
        }
    }

    // --- Build final labels ---
    sections
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let root = coarse[i];
            let base_idx = root_to_letter[&root];
            let base = cluster_label(base_idx);
            let vg = variant_group[i];
            let label = if vg == 0 {
                base
            } else {
                format!("{}{}", base, "'".repeat(vg.min(3)))
            };
            LabeledSection {
                start: s.start,
                end: s.end,
                name: s.name.clone(),
                label,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Text output
// ---------------------------------------------------------------------------

fn print_text(path: &str, track_forms: &[TrackForm], total_bars: usize) {
    println!("=== Form Detection: {} ===\n", path);

    for tf in track_forms {
        let secs = &tf.sections;
        println!("Track: {}  ({} bars)", tf.track_name, total_bars);

        // Form line
        let form_str: Vec<String> = secs
            .iter()
            .map(|s| match &s.name {
                Some(n) => format!("[{} {}]", n, s.label),
                None => format!("[{}]", s.label),
            })
            .collect();
        println!("Form:  {}", form_str.join("  "));
        println!();

        // Section table: collect occurrences per label
        let mut label_info: Vec<(String, Vec<(usize, usize)>)> = Vec::new();
        let mut label_idx: HashMap<String, usize> = HashMap::new();
        for s in secs {
            let range = s.bar_range();
            if let Some(&idx) = label_idx.get(&s.label) {
                label_info[idx].1.push(range);
            } else {
                label_idx.insert(s.label.clone(), label_info.len());
                label_info.push((s.label.clone(), vec![range]));
            }
        }

        let label_width = label_info.iter().map(|(l, _)| l.len()).max().unwrap_or(1);
        let bar_digits = format!("{}", total_bars).len();

        for (label, ranges) in &label_info {
            let ranges_str: Vec<String> = ranges
                .iter()
                .map(|(a, b)| {
                    if a == b {
                        format!("bar {:>w$}", a, w = bar_digits)
                    } else {
                        format!("bars {:>w$}–{:<w$}", a, b, w = bar_digits)
                    }
                })
                .collect();
            println!(
                "  {:<width$}  {}",
                label,
                ranges_str.join(",  "),
                width = label_width
            );
        }
        println!();
    }
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct JsonOutput {
    file: String,
    tracks: Vec<JsonTrack>,
}

#[derive(Serialize)]
struct JsonTrack {
    name: String,
    form: String,
    sections: Vec<JsonSection>,
}

#[derive(Serialize)]
struct JsonSection {
    label: String,
    bar_start: usize,
    bar_end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

fn print_json(path: &str, track_forms: &[TrackForm]) -> anyhow::Result<()> {
    let out = JsonOutput {
        file: path.to_owned(),
        tracks: track_forms
            .iter()
            .map(|tf| {
                let form_str: Vec<String> = tf
                    .sections
                    .iter()
                    .map(|s| match &s.name {
                        Some(n) => format!("[{} {}]", n, s.label),
                        None => format!("[{}]", s.label),
                    })
                    .collect();
                JsonTrack {
                    name: tf.track_name.clone(),
                    form: form_str.join(" "),
                    sections: tf
                        .sections
                        .iter()
                        .map(|s| {
                            let (a, b) = s.bar_range();
                            JsonSection {
                                label: s.label.clone(),
                                bar_start: a,
                                bar_end: b,
                                name: s.name.clone(),
                            }
                        })
                        .collect(),
                }
            })
            .collect(),
    };
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
