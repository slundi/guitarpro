use std::path::Path;

use axum::Json;
use axum::extract::{Multipart, State};
use uuid::Uuid;

use crate::api::ScoreSummary;
use crate::error::ApiError;
use crate::state::{AppState, SUPPORTED_EXTENSIONS, max_size_for, session_from_bytes};

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

    let ext = Path::new(&name)
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

    let cap = max_size_for(&ext);
    if bytes.len() > cap {
        return Err(ApiError::bad_request(
            "File too large",
            format!("Maximum size for .{ext} is {} bytes", cap),
        ));
    }

    let loaded = session_from_bytes(bytes, name.clone())
        .map_err(|e| ApiError::bad_request("Parse error", e.to_string()))?;

    let track_count = u8::try_from(loaded.song.tracks.len())
        .map_err(|_| ApiError::bad_request("Score too large", "Track count exceeds 255"))?;
    let measure_count = u16::try_from(loaded.song.measure_headers.len())
        .map_err(|_| ApiError::bad_request("Score too large", "Measure count exceeds 65535"))?;

    let id = Uuid::new_v4();
    state.insert_session(id, loaded).await;

    Ok(Json(ScoreSummary {
        id,
        name,
        track_count,
        measure_count,
    }))
}
