use std::collections::HashSet;
use std::time::Instant;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::Response;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ExtractRequest {
    /// 0-based track indices to keep (or exclude when invert=true)
    tracks: Vec<usize>,
    #[serde(default)]
    invert: bool,
    format: ExtractFormat,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum ExtractFormat {
    Gp5,
    Gpx,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExtractRequest>,
) -> Result<Response, ApiError> {
    // Clone what we need, then release the lock before the (possibly slow) encode.
    let (mut song, file_name) = {
        let mut sessions = state.sessions.write().await;
        let loaded = sessions
            .get_mut(&id)
            .ok_or_else(|| ApiError::not_found("Score session not found"))?;
        loaded.last_accessed = Instant::now();
        (loaded.song.clone(), loaded.file_name.clone())
    };

    let total = song.tracks.len();

    if req.tracks.is_empty() && !req.invert {
        return Err(ApiError::bad_request(
            "No tracks selected",
            "tracks must not be empty when invert is false",
        ));
    }

    for &idx in &req.tracks {
        if idx >= total {
            return Err(ApiError::bad_request(
                "Invalid track index",
                format!("index {idx} out of range (score has {total} track(s), indices 0..{})", total.saturating_sub(1)),
            ));
        }
    }

    let selected: HashSet<usize> = req.tracks.iter().copied().collect();

    let keep: Vec<bool> = (0..total)
        .map(|i| {
            if req.invert {
                !selected.contains(&i)
            } else {
                selected.contains(&i)
            }
        })
        .collect();

    let kept = keep.iter().filter(|&&k| k).count();
    if kept == 0 {
        return Err(ApiError::bad_request(
            "Empty result",
            "The given selection leaves no tracks in the output",
        ));
    }

    // Filter tracks and renumber them 1-based
    let mut new_tracks: Vec<_> = song
        .tracks
        .into_iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, t)| t)
        .collect();
    for (i, track) in new_tracks.iter_mut().enumerate() {
        track.number = (i + 1) as i32;
    }
    song.tracks = new_tracks;

    let (encoded, ext) = match req.format {
        ExtractFormat::Gp5 => (
            song.write((5, 1, 0), None)
                .map_err(|e| ApiError::bad_request("Encode failed", e.to_string()))?,
            "gp5",
        ),
        ExtractFormat::Gpx => (
            song.write_gpx()
                .map_err(|e| ApiError::bad_request("Encode failed", e.to_string()))?,
            "gpx",
        ),
    };

    let stem = file_name
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(&file_name);
    let download_name = format!("{stem}_extracted.{ext}");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{download_name}\""),
        )
        .body(Body::from(encoded))
        .unwrap())
}
