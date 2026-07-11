use std::path::Path;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::ScoreSummary;
use crate::error::ApiError;
use crate::state::{AppState, SUPPORTED_EXTENSIONS, max_size_for, session_from_bytes};

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

    if !canonical.starts_with(&state.root) {
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
            format!("'.{ext}' is not supported; use gp3, gp4, gp5, gp, gpx, or mscz"),
        ));
    }

    let bytes = std::fs::read(&canonical)
        .map_err(|e| ApiError::bad_request("Read error", e.to_string()))?;

    let cap = max_size_for(&ext);
    if bytes.len() > cap {
        return Err(ApiError::bad_request(
            "File too large",
            format!("Maximum size for .{ext} is {} bytes", cap),
        ));
    }

    let file_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let loaded = session_from_bytes(bytes, file_name.clone())
        .map_err(|e| ApiError::bad_request("Parse error", e.to_string()))?;

    let track_count = u8::try_from(loaded.song.tracks.len())
        .map_err(|_| ApiError::bad_request("Score too large", "Track count exceeds 255"))?;
    let measure_count = u16::try_from(loaded.song.measure_headers.len())
        .map_err(|_| ApiError::bad_request("Score too large", "Measure count exceeds 65535"))?;

    let id = Uuid::new_v4();
    state.insert_session(id, loaded).await;

    Ok(Json(ScoreSummary {
        id,
        name: file_name,
        track_count,
        measure_count,
    }))
}
