use std::path::Path;
use std::time::Instant;

use axum::Json;
use axum::extract::State;
use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::ScoreSummary;
use crate::error::ApiError;
use crate::state::{AppState, LoadedFile, MAX_FILE_SIZE, SUPPORTED_EXTENSIONS, parse_song};

#[derive(Deserialize)]
pub struct OpenRequest {
    path: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(req): Json<OpenRequest>,
) -> Result<Json<ScoreSummary>, ApiError> {
    let canonical = Path::new(&req.path)
        .canonicalize()
        .map_err(|_| ApiError::bad_request("Invalid path", "File not found or not accessible"))?;

    let canonical_root = state
        .root
        .canonicalize()
        .unwrap_or_else(|_| state.root.clone());
    if !canonical.starts_with(&canonical_root) {
        return Err(ApiError::forbidden(
            "Path is outside the allowed root directory",
        ));
    }

    let ext = canonical
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(ApiError::bad_request(
            "Unsupported format",
            format!("'.{ext}' is not supported; use gp3, gp4, gp5, gp, or gpx"),
        ));
    }

    let bytes = std::fs::read(&canonical)
        .map_err(|e| ApiError::bad_request("Read error", e.to_string()))?;

    if bytes.len() > MAX_FILE_SIZE {
        return Err(ApiError::bad_request(
            "File too large",
            "Maximum allowed size is 16 MB",
        ));
    }

    let song = parse_song(&ext, &bytes)
        .map_err(|e| ApiError::bad_request("Parse error", e.to_string()))?;

    let file_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let track_count = song.tracks.len() as u8;
    let measure_count = song.measure_headers.len() as u16;
    let score = legacy_song_to_loaded_score(&song);

    let id = Uuid::new_v4();
    state.sessions.write().await.insert(
        id,
        LoadedFile {
            bytes,
            song,
            score,
            file_name: file_name.clone(),
            ext,
            last_accessed: Instant::now(),
        },
    );

    Ok(Json(ScoreSummary {
        id,
        name: file_name,
        track_count,
        measure_count,
    }))
}
