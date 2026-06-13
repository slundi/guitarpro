use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use guitarpro::Song;
use guitarpro::model::optimized::LoadedScore;
use tokio::sync::RwLock;
use uuid::Uuid;

pub const MAX_FILE_SIZE: usize = 16 * 1024 * 1024; // 16 MB
pub const SUPPORTED_EXTENSIONS: &[&str] = &["gp3", "gp4", "gp5", "gp", "gpx"];

const SESSION_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour inactivity
const SWEEP_INTERVAL: Duration = Duration::from_secs(300); // check every 5 minutes

pub struct LoadedFile {
    pub bytes: Vec<u8>,
    pub song: Song,
    pub score: LoadedScore,
    pub file_name: String,
    #[expect(dead_code)] // used in Part 8 (format-aware re-encoding)
    pub ext: String,
    pub last_accessed: Instant,
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

    pub fn spawn_sweep(&self) {
        let sessions = Arc::clone(&self.sessions);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
            ticker.tick().await; // discard the immediate first tick
            loop {
                ticker.tick().await;
                let mut map = sessions.write().await;
                let before = map.len();
                map.retain(|_, v| v.last_accessed.elapsed() < SESSION_TIMEOUT);
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
        other => anyhow::bail!("unsupported extension: .{other}"),
    }
    Ok(song)
}
