use std::path::Path;
use std::time::Instant;

use axum::Json;
use axum::extract::{Multipart, State};
use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;
use uuid::Uuid;

use crate::api::ScoreSummary;
use crate::error::ApiError;
use crate::state::{AppState, LoadedFile, MAX_FILE_SIZE, SUPPORTED_EXTENSIONS, parse_song};

pub async fn handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ScoreSummary>, ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request("Invalid multipart data", e.to_string()))?
    {
        if field.name() == Some("file") {
            file_name = field.file_name().map(str::to_owned);
            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::bad_request("Failed to read upload data", e.to_string()))?;
            file_bytes = Some(data.to_vec());
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        ApiError::bad_request("Missing file field", "Upload must include a `file` field")
    })?;

    let name = file_name.ok_or_else(|| {
        ApiError::bad_request("Missing filename", "The `file` field must have a filename")
    })?;

    if bytes.len() > MAX_FILE_SIZE {
        return Err(ApiError::bad_request(
            "File too large",
            "Maximum allowed size is 16 MB",
        ));
    }

    let ext = Path::new(&name)
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

    let song = parse_song(&ext, &bytes)
        .map_err(|e| ApiError::bad_request("Parse error", e.to_string()))?;
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
            file_name: name.clone(),
            ext,
            last_accessed: Instant::now(),
        },
    );

    Ok(Json(ScoreSummary {
        id,
        name,
        track_count,
        measure_count,
    }))
}
