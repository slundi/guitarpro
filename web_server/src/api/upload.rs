use std::path::Path;

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
    let track_count = u8::try_from(song.tracks.len())
        .map_err(|_| ApiError::bad_request("Score too large", "Track count exceeds 255"))?;
    let measure_count = u16::try_from(song.measure_headers.len())
        .map_err(|_| ApiError::bad_request("Score too large", "Measure count exceeds 65535"))?;
    let score = legacy_song_to_loaded_score(&song);

    let id = Uuid::new_v4();
    state
        .insert_session(id, LoadedFile::new(bytes, song, score, name.clone(), ext))
        .await;

    Ok(Json(ScoreSummary {
        id,
        name,
        track_count,
        measure_count,
    }))
}
