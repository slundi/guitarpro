use std::collections::HashMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use std::{fs, io};

use axum::Json;
use axum::extract::{Query, State};
use axum::response::sse::{Event, Sse};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::error::ApiError;
use crate::state::{AppState, SUPPORTED_EXTENSIONS, parse_song};
use guitarpro::NoteType;

// ── File browser ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListQuery {
    path: Option<String>,
}

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    path: String,
    size: u64,
    modified: u64,
    is_dir: bool,
}

#[derive(Serialize)]
pub struct ListResponse {
    current: String,
    entries: Vec<FileEntry>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse>, ApiError> {
    let base = match query.path {
        Some(ref p) if !p.is_empty() => {
            let p = Path::new(p)
                .canonicalize()
                .map_err(|_| ApiError::bad_request("Invalid path", "Path not found"))?;
            if !p.starts_with(&state.root) {
                return Err(ApiError::forbidden(
                    "Path is outside the allowed root directory",
                ));
            }
            p
        }
        _ => state
            .root
            .canonicalize()
            .map_err(|e| ApiError::bad_request("Root unavailable", e.to_string()))?,
    };

    if !base.is_dir() {
        return Err(ApiError::bad_request(
            "Not a directory",
            "Path is not a directory",
        ));
    }

    let mut dirs: Vec<FileEntry> = Vec::new();
    let mut files: Vec<FileEntry> = Vec::new();

    let read_dir = fs::read_dir(&base)
        .map_err(|e| ApiError::bad_request("Cannot read directory", e.to_string()))?;

    for entry_result in read_dir {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };

        // Skip hidden entries
        if name_str.starts_with('.') {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let path_str = entry.path().to_string_lossy().into_owned();
        let size = metadata.len();
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if metadata.is_dir() {
            dirs.push(FileEntry {
                name: name_str,
                path: path_str,
                size,
                modified,
                is_dir: true,
            });
        } else if is_gp_file(&entry.path()) {
            files.push(FileEntry {
                name: name_str,
                path: path_str,
                size,
                modified,
                is_dir: false,
            });
        }
    }

    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));
    dirs.extend(files);

    Ok(Json(ListResponse {
        current: base.to_string_lossy().into_owned(),
        entries: dirs,
    }))
}

fn is_gp_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

// ── Duplicate scanner ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DupRequest {
    dir: String,
    #[serde(default = "default_threshold")]
    threshold: f64,
    #[serde(default)]
    recursive: bool,
}

fn default_threshold() -> f64 {
    0.85
}

#[derive(Serialize, Clone)]
pub struct DupFile {
    path: String,
    name: String,
    similarity: f64,
}

#[derive(Serialize, Clone)]
pub struct DupGroup {
    files: Vec<DupFile>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum SseMsg {
    Progress {
        file: String,
        current: usize,
        total: usize,
    },
    Result {
        groups: Vec<DupGroup>,
    },
    Error {
        message: String,
    },
}

pub async fn duplicates(
    State(state): State<AppState>,
    Json(req): Json<DupRequest>,
) -> Result<Sse<ReceiverStream<Result<Event, Infallible>>>, ApiError> {
    let dir_path = Path::new(&req.dir)
        .canonicalize()
        .map_err(|_| ApiError::bad_request("Invalid directory", "Directory not found"))?;

    if !dir_path.starts_with(&state.root) {
        return Err(ApiError::forbidden(
            "Path is outside the allowed root directory",
        ));
    }

    if !dir_path.is_dir() {
        return Err(ApiError::bad_request(
            "Not a directory",
            "Path is not a directory",
        ));
    }

    let threshold = req.threshold.clamp(0.0, 1.0);
    let recursive = req.recursive;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(64);

    tokio::task::spawn_blocking(move || {
        run_dup_scan(dir_path, threshold, recursive, &tx);
    });

    Ok(Sse::new(ReceiverStream::new(rx)))
}

fn send_event(tx: &mpsc::Sender<Result<Event, Infallible>>, msg: &SseMsg) {
    if let Ok(json) = serde_json::to_string(msg) {
        let event = Event::default().data(json);
        let _ = tx.blocking_send(Ok(event));
    }
}

fn run_dup_scan(
    dir: PathBuf,
    threshold: f64,
    recursive: bool,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) {
    let files = match collect_files(&dir, recursive) {
        Ok(f) => f,
        Err(e) => {
            send_event(
                tx,
                &SseMsg::Error {
                    message: e.to_string(),
                },
            );
            return;
        }
    };

    let total = files.len();
    let mut fps: Vec<Fingerprint> = Vec::with_capacity(total);

    for (i, path) in files.iter().enumerate() {
        send_event(
            tx,
            &SseMsg::Progress {
                file: path.to_string_lossy().into_owned(),
                current: i + 1,
                total,
            },
        );

        match Fingerprint::build(path) {
            Ok(fp) => fps.push(fp),
            Err(_) => {
                // skip unparsable files silently
            }
        }
    }

    let groups = find_groups(&fps, threshold);

    let dup_groups: Vec<DupGroup> = groups
        .iter()
        .map(|group| DupGroup {
            files: group
                .iter()
                .enumerate()
                .map(|(rank, &idx)| {
                    let score = if rank == 0 {
                        1.0
                    } else {
                        similarity(&fps[group[0]], &fps[idx])
                    };
                    let path = fps[idx].path.to_string_lossy().into_owned();
                    let name = fps[idx]
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.clone());
                    DupFile {
                        path,
                        name,
                        similarity: (score * 1000.0).round() / 1000.0,
                    }
                })
                .collect(),
        })
        .collect();

    send_event(tx, &SseMsg::Result { groups: dup_groups });
}

// ── Fingerprint ───────────────────────────────────────────────────────────────

struct Fingerprint {
    path: PathBuf,
    content_hash: u64,
    note_hist: [u32; 128],
    measure_count: usize,
    title_norm: String,
    artist_norm: String,
}

impl Fingerprint {
    fn build(path: &Path) -> anyhow::Result<Self> {
        let bytes = fs::read(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let song = parse_song(&ext, &bytes)?;

        let measure_count = song.measure_headers.len();
        let title_norm = song.name.trim().to_lowercase();
        let artist_norm = song.artist.trim().to_lowercase();

        let mut note_hist = [0u32; 128];
        let mut hash_bytes: Vec<u8> = Vec::new();

        for track in &song.tracks {
            for measure in &track.measures {
                for voice in &measure.voices {
                    for beat in &voice.beats {
                        let mut midi_notes: Vec<u8> = beat
                            .notes
                            .iter()
                            .filter(|n| n.kind != NoteType::Rest && n.kind != NoteType::Tie)
                            .filter_map(|n| {
                                let midi = if track.percussion_track {
                                    n.value
                                } else {
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
                        hash_bytes.push(0xFE);

                        for &p in &midi_notes {
                            note_hist[p as usize] += 1;
                        }
                    }
                }
                hash_bytes.push(0xFD);
            }
            hash_bytes.push(0xFC);
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

fn fnv1a(data: &[u8]) -> u64 {
    const OFFSET: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    data.iter()
        .fold(OFFSET, |h, &b| (h ^ b as u64).wrapping_mul(PRIME))
}

// ── Similarity ────────────────────────────────────────────────────────────────

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

fn similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    if a.content_hash == b.content_hash {
        return 1.0;
    }

    let hist_sim = histogram_similarity(&a.note_hist, &b.note_hist);

    let meta_sim = if !a.title_norm.is_empty()
        && a.title_norm == b.title_norm
        && a.artist_norm == b.artist_norm
    {
        1.0_f64
    } else {
        0.0
    };

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

// ── Grouping — union-find single-linkage clustering ───────────────────────────

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
    for g in &mut groups {
        g.sort_by_key(|&i| &fps[i].path);
    }
    groups.sort_by_key(|g| fps[g[0]].path.clone());
    groups
}

// ── File collection ───────────────────────────────────────────────────────────

fn collect_files(dir: &Path, recursive: bool) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(d) = stack.pop() {
        let entries = fs::read_dir(&d)?;
        let mut children: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        children.sort();
        for path in children {
            // Descend only into *real* directories. `is_dir()` follows symlinks,
            // so a symlink pointing at an ancestor would otherwise make the scan
            // loop forever; skip symlinked directories to break such cycles.
            if recursive && path.is_dir() && !path.is_symlink() {
                stack.push(path);
            } else if is_gp_file(&path) {
                result.push(path);
            }
        }
    }

    result.sort();
    Ok(result)
}

#[cfg(all(test, unix))]
mod tests {
    use super::collect_files;
    use std::fs;

    #[test]
    fn recursive_scan_terminates_on_symlink_cycle() {
        // Unique scratch dir under the system temp directory.
        let root = std::env::temp_dir().join(format!("ws_dup_{}", uuid::Uuid::new_v4()));
        let sub = root.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("song.gp5"), b"not a real score").unwrap();

        // A symlink pointing back at the root would loop forever if followed.
        std::os::unix::fs::symlink(&root, root.join("loop")).unwrap();

        // Must return (not hang) and find the single real gp file exactly once.
        let found = collect_files(&root, true).unwrap();
        let cleanup = fs::remove_dir_all(&root);

        assert_eq!(found.len(), 1, "cycle must not inflate or hang the scan");
        assert!(found[0].ends_with("sub/song.gp5"));
        cleanup.unwrap();
    }
}
