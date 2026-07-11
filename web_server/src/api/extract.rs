use std::collections::HashSet;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::Response;
use guitarpro::convert::mscz::loaded_score_to_mscx;
use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;
use guitarpro::io::mscz::write_mscz;
use guitarpro::model::mscz::{MsczArchive, MsczEntry, MsczFile};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::attachment;
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
    Mscz,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExtractRequest>,
) -> Result<Response, ApiError> {
    // Clone what we need, then release the lock before the (possibly slow) encode.
    let (mut song, file_name) = {
        let sessions = state.sessions.read().await;
        let loaded = sessions
            .get(&id)
            .ok_or_else(|| ApiError::not_found("Score session not found"))?;
        loaded.touch();
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
                format!(
                    "index {idx} out of range (score has {total} track(s), indices 0..{})",
                    total.saturating_sub(1)
                ),
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
        ExtractFormat::Mscz => (
            encode_song_as_mscz(&song)
                .map_err(|e| ApiError::bad_request("Encode failed", e.to_string()))?,
            "mscz",
        ),
    };

    let stem = file_name
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(&file_name);
    let download_name = format!("{stem}_extracted.{ext}");

    attachment(encoded, &download_name)
}

/// Repackage a legacy `Song` as an MSCZ archive with a minimal
/// `META-INF/container.xml` + `score.mscx`. Mirrors the helper used by the
/// `/download?format=mscz` endpoint so both paths produce identical output
/// shape.
fn encode_song_as_mscz(song: &guitarpro::Song) -> anyhow::Result<Vec<u8>> {
    let loaded = legacy_song_to_loaded_score(song);
    let mscx = loaded_score_to_mscx(&loaded);
    let manifest =
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container><rootfiles><rootfile full-path=\"score.mscx\"/></rootfiles></container>\n";
    let archive = MsczArchive {
        rootfiles: vec!["score.mscx".to_string()],
        entries: vec![
            MsczEntry {
                path: "META-INF/container.xml".to_string(),
                data: manifest.to_vec(),
            },
            MsczEntry {
                path: "score.mscx".to_string(),
                data: mscx.raw_xml.as_bytes().to_vec(),
            },
        ],
    };
    let file = MsczFile { archive, mscx };
    write_mscz(&file).map_err(|e| anyhow::anyhow!("MSCZ write failed: {e}"))
}
