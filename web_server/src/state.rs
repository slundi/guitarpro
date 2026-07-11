use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use guitarpro::Song;
use guitarpro::convert::legacy::loaded_score_to_legacy_song;
use guitarpro::convert::mscz::mscx_to_loaded_score;
use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;
use guitarpro::io::mscz::read_mscz_bytes;
use guitarpro::model::optimized::LoadedScore;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Legacy per-file cap for GP3/GP4/GP5/GP/GPX inputs.
pub const MAX_FILE_SIZE: usize = 16 * 1024 * 1024; // 16 MB
/// MSCZ archives bundle thumbnails and (optionally) soundfont overrides, so
/// they need a larger cap than GP inputs.
pub const MAX_MSCZ_FILE_SIZE: usize = 32 * 1024 * 1024; // 32 MB
pub const SUPPORTED_EXTENSIONS: &[&str] = &["gp3", "gp4", "gp5", "gp", "gpx", "mscz"];

/// Return the byte cap for a given lowercase extension. Unknown extensions
/// fall back to the smaller [`MAX_FILE_SIZE`] to stay conservative.
pub fn max_size_for(ext: &str) -> usize {
    if ext.eq_ignore_ascii_case("mscz") {
        MAX_MSCZ_FILE_SIZE
    } else {
        MAX_FILE_SIZE
    }
}

/// Upper bound on concurrently retained sessions. Each session holds the raw
/// bytes (≤16 MB) plus the parsed model, so an unbounded map lets a client
/// exhaust memory well before the inactivity sweep fires. When full, the least
/// recently accessed session is evicted to make room.
pub const MAX_SESSIONS: usize = 64;

const SESSION_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour inactivity
const SWEEP_INTERVAL: Duration = Duration::from_secs(300); // check every 5 minutes

pub struct LoadedFile {
    pub bytes: Vec<u8>,
    pub song: Song,
    pub score: LoadedScore,
    pub file_name: String,
    #[cfg_attr(not(test), allow(dead_code))] // read in tests + Part 8 (format-aware re-encoding)
    pub ext: String,
    /// Preserved MSCZ thumbnail (embedded PNG), if the source was `.mscz`
    /// and the archive contained one. Enables `GET /api/score/:id/thumbnail`
    /// without re-reading the archive.
    pub thumbnail: Option<Vec<u8>>,
    /// Interior-mutable so read-only handlers can refresh it under a shared
    /// read lock instead of taking the global write lock for the whole request.
    last_accessed: Mutex<Instant>,
}

impl LoadedFile {
    pub fn new(
        bytes: Vec<u8>,
        song: Song,
        score: LoadedScore,
        file_name: String,
        ext: String,
    ) -> Self {
        Self {
            bytes,
            song,
            score,
            file_name,
            ext,
            thumbnail: None,
            last_accessed: Mutex::new(Instant::now()),
        }
    }

    /// Construct a session with an embedded thumbnail (MSCZ path).
    pub fn with_thumbnail(mut self, thumbnail: Option<Vec<u8>>) -> Self {
        self.thumbnail = thumbnail;
        self
    }

    /// Mark the session as accessed now. Callable through a shared reference.
    pub fn touch(&self) {
        if let Ok(mut guard) = self.last_accessed.lock() {
            *guard = Instant::now();
        }
    }

    /// Time since the last access. A poisoned lock is treated as freshly
    /// accessed so a wedged session is not swept away prematurely.
    pub fn age(&self) -> Duration {
        self.last_accessed
            .lock()
            .map(|g| g.elapsed())
            .unwrap_or(Duration::ZERO)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<RwLock<HashMap<Uuid, LoadedFile>>>,
    pub root: std::path::PathBuf,
}

impl AppState {
    pub fn new(root: std::path::PathBuf) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            root,
        }
    }

    /// Insert a session, evicting the least recently accessed one first if the
    /// store is at capacity. Bounds total memory regardless of client activity.
    pub async fn insert_session(&self, id: Uuid, file: LoadedFile) {
        let mut map = self.sessions.write().await;
        if map.len() >= MAX_SESSIONS
            && let Some(oldest) = map.iter().max_by_key(|(_, v)| v.age()).map(|(k, _)| *k)
        {
            map.remove(&oldest);
        }
        map.insert(id, file);
    }

    pub fn spawn_sweep(&self) {
        let sessions = Arc::clone(&self.sessions);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
            ticker.tick().await; // discard the immediate first tick
            loop {
                ticker.tick().await;
                let mut map = sessions.write().await;
                let before = map.len();
                map.retain(|_, v| v.age() < SESSION_TIMEOUT);
                let removed = before - map.len();
                if removed > 0 {
                    tracing::info!(removed, active = map.len(), "session sweep");
                }
            }
        });
    }
}

pub fn parse_song(ext: &str, data: &[u8]) -> Result<Song> {
    let mut song = Song::default();
    match ext.to_uppercase().as_str() {
        "GP3" => song.read_gp3(data)?,
        "GP4" => song.read_gp4(data)?,
        "GP5" => song.read_gp5(data)?,
        "GP" => song.read_gp(data)?,
        "GPX" => song.read_gpx(data)?,
        "MSCZ" => {
            song = parse_mscz(data)?.song;
        }
        other => anyhow::bail!("unsupported extension: .{other}"),
    }
    Ok(song)
}

/// Bundle produced by [`parse_mscz`].
pub struct MsczSession {
    pub song: Song,
    pub score: LoadedScore,
    pub thumbnail: Option<Vec<u8>>,
}

/// Parse MSCZ bytes into a `Song` + `LoadedScore` + optional embedded PNG
/// thumbnail. Bridges MSCX → LoadedScore (via the guitarpro converter) and
/// LoadedScore → Song so downstream handlers keep working on the legacy
/// model.
pub fn parse_mscz(data: &[u8]) -> Result<MsczSession> {
    let file = read_mscz_bytes(data).map_err(|e| anyhow::anyhow!("MSCZ parse: {e}"))?;
    let thumbnail = file
        .archive
        .thumbnail_entry()
        .map(|entry| entry.data.clone());
    let outcome = mscx_to_loaded_score(&file.mscx);
    let song = loaded_score_to_legacy_song(&outcome.score);
    Ok(MsczSession {
        song,
        score: outcome.score,
        thumbnail,
    })
}

/// Turn raw bytes + a source filename into a session-ready [`LoadedFile`].
///
/// Centralises the parsing pipeline shared by the upload and open handlers:
/// picks the correct parser from the extension, enforces the per-format size
/// cap, and — for MSCZ inputs — hangs on to the embedded thumbnail so the
/// `/api/score/:id/thumbnail` endpoint can serve it without re-reading the
/// archive.
pub fn session_from_bytes(bytes: Vec<u8>, file_name: String) -> Result<LoadedFile> {
    let ext = Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        anyhow::bail!("unsupported extension '.{ext}'; use gp3, gp4, gp5, gp, gpx, or mscz");
    }
    if bytes.len() > max_size_for(&ext) {
        anyhow::bail!(
            "file size {} exceeds cap {} for .{ext}",
            bytes.len(),
            max_size_for(&ext)
        );
    }

    if ext == "mscz" {
        let session = parse_mscz(&bytes)?;
        return Ok(
            LoadedFile::new(bytes, session.song, session.score, file_name, ext)
                .with_thumbnail(session.thumbnail),
        );
    }

    let song = parse_song(&ext, &bytes)?;
    let score = legacy_song_to_loaded_score(&song);
    Ok(LoadedFile::new(bytes, song, score, file_name, ext))
}

/// Load, validate, parse, and convert a GP file from disk into a [`LoadedFile`].
///
/// Used at startup by `main` to pre-load a file passed on the CLI, and mirrors
/// the pipeline that the `/api/score/upload` and `/api/score/open` handlers run
/// on their own inputs. Enforces the same extension whitelist and size cap.
pub fn load_file_from_disk(path: &Path) -> Result<LoadedFile> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        anyhow::bail!(
            "'{}': unsupported extension '.{ext}'; use gp3, gp4, gp5, gp, gpx, or mscz",
            path.display()
        );
    }

    let bytes =
        std::fs::read(path).with_context(|| format!("Failed to read '{}'", path.display()))?;

    let cap = max_size_for(&ext);
    if bytes.len() > cap {
        anyhow::bail!(
            "'{}': {} bytes exceeds the {cap}-byte limit for .{ext}",
            path.display(),
            bytes.len(),
        );
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    session_from_bytes(bytes, file_name)
        .with_context(|| format!("Failed to parse '{}'", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;

    fn dummy_file() -> LoadedFile {
        let song = Song::default();
        let score = legacy_song_to_loaded_score(&song);
        LoadedFile::new(
            Vec::new(),
            song,
            score,
            "dummy.gp5".to_string(),
            "gp5".to_string(),
        )
    }

    // ── load_file_from_disk (Part 9.4 CLI preload) ────────────────────────────

    /// `LoadedFile` deliberately doesn't implement `Debug` (its `Song` field is
    /// noisy), so `.unwrap_err()` won't compile. Small helper that panics with
    /// a readable message when the Result is `Ok`.
    fn expect_err<T>(r: Result<T>) -> anyhow::Error {
        match r {
            Ok(_) => panic!("expected an error, got Ok"),
            Err(e) => e,
        }
    }

    fn workspace_test_dir() -> std::path::PathBuf {
        // web_server → workspace root → test/
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test")
    }

    #[test]
    fn load_file_from_disk_rejects_unsupported_extension() {
        let dir = std::env::temp_dir().join(format!("ws_load_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("song.txt");
        std::fs::write(&path, b"not a score").unwrap();

        let err = expect_err(load_file_from_disk(&path));
        assert!(
            err.to_string().contains("unsupported extension"),
            "unexpected error: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_file_from_disk_rejects_missing_extension() {
        let dir = std::env::temp_dir().join(format!("ws_load_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("no_extension");
        std::fs::write(&path, b"bytes").unwrap();

        let err = expect_err(load_file_from_disk(&path));
        assert!(err.to_string().contains("unsupported extension"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_file_from_disk_rejects_oversized() {
        let dir = std::env::temp_dir().join(format!("ws_load_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("huge.gp5");
        // 16 MB + 1 byte of arbitrary content — the size check should reject
        // before any parser is asked to look at it.
        std::fs::write(&path, vec![0u8; MAX_FILE_SIZE + 1]).unwrap();

        let err = expect_err(load_file_from_disk(&path));
        assert!(err.to_string().contains("exceeds"), "unexpected: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_file_from_disk_missing_file_bubbles_up_io_error() {
        let path = std::env::temp_dir().join(format!("ws_missing_{}.gp5", Uuid::new_v4()));
        assert!(!path.exists());
        let err = expect_err(load_file_from_disk(&path));
        assert!(
            err.to_string().contains("Failed to read"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn load_file_from_disk_parses_real_gp5_fixture() {
        let path = workspace_test_dir().join("001_Funky_Guy.gp5");
        if !path.is_file() {
            eprintln!("skipping — fixture missing: {}", path.display());
            return;
        }
        let loaded = load_file_from_disk(&path).expect("real GP5 fixture must load");
        assert_eq!(loaded.ext, "gp5");
        assert_eq!(loaded.file_name, "001_Funky_Guy.gp5");
        assert!(!loaded.bytes.is_empty());
        assert!(
            !loaded.song.tracks.is_empty(),
            "expected at least one track"
        );
    }

    #[tokio::test]
    async fn insert_session_evicts_when_over_capacity() {
        let state = AppState::new(std::path::PathBuf::from("/"));

        let mut ids = Vec::new();
        for _ in 0..MAX_SESSIONS {
            let id = Uuid::new_v4();
            ids.push(id);
            state.insert_session(id, dummy_file()).await;
        }
        assert_eq!(state.sessions.read().await.len(), MAX_SESSIONS);

        // One more insert must evict, not grow past the cap.
        let extra = Uuid::new_v4();
        state.insert_session(extra, dummy_file()).await;

        let map = state.sessions.read().await;
        assert_eq!(map.len(), MAX_SESSIONS, "capacity must hold steady");
        assert!(map.contains_key(&extra), "newest session must survive");
        assert!(
            !map.contains_key(&ids[0]),
            "least recently accessed session must be evicted"
        );
    }
}
