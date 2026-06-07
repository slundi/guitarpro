use std::time::Instant;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use guitarpro::model::optimized::global::InstrumentKind;
use guitarpro::model::optimized::note::{Pitch, PitchStep};
use serde::Serialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
struct ScoreInfo {
    title: String,
    artist: Option<String>,
    album: Option<String>,
    tempo: f32,
    time_signature: TimeSigInfo,
    tracks: Vec<TrackInfo>,
}

#[derive(Serialize)]
struct TimeSigInfo {
    numerator: u8,
    denominator: u8,
}

#[derive(Serialize)]
struct TrackInfo {
    index: u8,
    name: String,
    string_count: u8,
    /// MIDI note numbers, low string first (e.g. [40,45,50,55,59,64] for standard guitar)
    tuning: Vec<i16>,
}

pub async fn raw(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let bytes;
    let file_name;
    {
        let mut sessions = state.sessions.write().await;
        let loaded = sessions
            .get_mut(&id)
            .ok_or_else(|| ApiError::not_found("Score session not found"))?;
        loaded.last_accessed = Instant::now();
        bytes = loaded.bytes.clone();
        file_name = loaded.file_name.clone();
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{file_name}\""),
        )
        .body(Body::from(bytes))
        .unwrap())
}

pub async fn info(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let mut sessions = state.sessions.write().await;
    let loaded = sessions
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("Score session not found"))?;
    loaded.last_accessed = Instant::now();

    let meta = &loaded.score.score.metadata;
    let instruments = &loaded.score.score.instruments;
    let score_tracks = &loaded.score.score.tracks;

    let tracks: Vec<TrackInfo> = score_tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let inst = &instruments[track.instrument.0 as usize];
            let (string_count, tuning) = match &inst.kind {
                InstrumentKind::Stringed {
                    string_count,
                    tuning,
                    ..
                } => (*string_count, tuning.iter().map(pitch_to_midi).collect()),
                _ => (0u8, vec![]),
            };
            TrackInfo {
                index: i as u8,
                name: track.name.clone(),
                string_count,
                tuning,
            }
        })
        .collect();

    Ok(Json(ScoreInfo {
        title: meta.title.clone(),
        artist: meta.artist.clone(),
        album: meta.album.clone(),
        tempo: meta.master_tempo,
        time_signature: TimeSigInfo {
            numerator: meta.time_signature.numerator,
            denominator: meta.time_signature.denominator,
        },
        tracks,
    }))
}

fn pitch_to_midi(p: &Pitch) -> i16 {
    let semitone: i16 = match p.step {
        PitchStep::C => 0,
        PitchStep::D => 2,
        PitchStep::E => 4,
        PitchStep::F => 5,
        PitchStep::G => 7,
        PitchStep::A => 9,
        PitchStep::B => 11,
    };
    12 * (p.octave as i16 + 1) + semitone + p.alter as i16
}
