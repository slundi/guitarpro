use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use guitarpro::Song;
use guitarpro::convert::mscz::loaded_score_to_mscx;
use guitarpro::convert::optimized::legacy::legacy_song_to_loaded_score;
use guitarpro::io::mscz::write_mscz;
use guitarpro::model::mscz::{MsczArchive, MsczEntry, MsczFile};
use guitarpro::model::optimized::global::InstrumentKind;
use guitarpro::model::optimized::note::{Pitch, PitchStep};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::attachment;
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
    markers: Vec<MarkerInfo>,
    /// Whether the score bundles a backing-track audio file (GP8 feature).
    has_audio: bool,
    /// SyncPoint anchors (bar → audio frame offset), when present.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sync_points: Vec<SyncPointInfo>,
}

#[derive(Serialize)]
struct SyncPointInfo {
    bar: i32,
    bar_occurrence: i32,
    frame_offset: Option<i64>,
    modified_tempo: Option<f64>,
}

#[derive(Serialize)]
struct MarkerInfo {
    measure: u16,
    title: String,
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

/// What the `raw` endpoint should hand back to the client.
///
/// AlphaTab (frontend renderer) parses GP3–GP7 / GPX / MusicXML but does not
/// understand MSCZ, so serving MSCZ bytes verbatim leaves the viewer blank.
/// For MSCZ sessions we transcode the parsed `Song` to GPX under the same
/// filename stem; other sessions still get their original bytes back.
enum RawPayload {
    Verbatim(Vec<u8>),
    TranscodeMsczToGpx(Box<Song>),
}

pub async fn raw(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let (payload, file_name) = {
        let sessions = state.sessions.read().await;
        let loaded = sessions
            .get(&id)
            .ok_or_else(|| ApiError::not_found("Score session not found"))?;
        loaded.touch();
        if loaded.ext == "mscz" {
            let stem = loaded
                .file_name
                .rsplit_once('.')
                .map(|(s, _)| s)
                .unwrap_or(&loaded.file_name);
            (
                RawPayload::TranscodeMsczToGpx(Box::new(loaded.song.clone())),
                format!("{stem}.gpx"),
            )
        } else {
            (
                RawPayload::Verbatim(loaded.bytes.clone()),
                loaded.file_name.clone(),
            )
        }
    };

    let bytes = match payload {
        RawPayload::Verbatim(b) => b,
        RawPayload::TranscodeMsczToGpx(song) => song
            .write_gpx()
            .map_err(|e| ApiError::internal(format!("MSCZ→GPX transcode failed: {e}")))?,
    };

    attachment(bytes, &file_name)
}

#[derive(Deserialize)]
pub struct DownloadQuery {
    format: DownloadFormat,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum DownloadFormat {
    Gp5,
    Gpx,
    Mscz,
}

pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<DownloadQuery>,
) -> Result<Response, ApiError> {
    let (song, file_name) = {
        let sessions = state.sessions.read().await;
        let loaded = sessions
            .get(&id)
            .ok_or_else(|| ApiError::not_found("Score session not found"))?;
        loaded.touch();
        (loaded.song.clone(), loaded.file_name.clone())
    };

    let (encoded, ext) = match query.format {
        DownloadFormat::Gp5 => (
            song.write((5, 1, 0), None)
                .map_err(|e| ApiError::bad_request("Encode failed", e.to_string()))?,
            "gp5",
        ),
        DownloadFormat::Gpx => (
            song.write_gpx()
                .map_err(|e| ApiError::bad_request("Encode failed", e.to_string()))?,
            "gpx",
        ),
        DownloadFormat::Mscz => (
            encode_song_as_mscz(&song)
                .map_err(|e| ApiError::bad_request("Encode failed", e.to_string()))?,
            "mscz",
        ),
    };

    let stem = file_name
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(&file_name);
    let download_name = format!("{stem}.{ext}");

    attachment(encoded, &download_name)
}

/// Convert a legacy `Song` into MSCZ bytes for the `/download?format=mscz`
/// path. The generated archive contains only `META-INF/container.xml` and
/// `score.mscx` — no thumbnail or style file — because the source Song may
/// have been produced from GP or MusicXML input that never carried those.
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

/// Serve the PNG thumbnail embedded in the source MSCZ archive.
/// Returns 404 if the session was not created from an MSCZ (or the archive
/// carried no thumbnail).
pub async fn thumbnail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let png = {
        let sessions = state.sessions.read().await;
        let loaded = sessions
            .get(&id)
            .ok_or_else(|| ApiError::not_found("Score session not found"))?;
        loaded.touch();
        loaded
            .thumbnail
            .clone()
            .ok_or_else(|| ApiError::not_found("No thumbnail available for this session"))?
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(png))
        .map_err(|e| ApiError::internal(e.to_string()))
}

pub async fn info(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let sessions = state.sessions.read().await;
    let loaded = sessions
        .get(&id)
        .ok_or_else(|| ApiError::not_found("Score session not found"))?;
    loaded.touch();

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

    let markers: Vec<MarkerInfo> = loaded
        .score
        .score
        .timeline
        .iter()
        .filter_map(|md| {
            md.marker.as_ref().map(|m| MarkerInfo {
                measure: md.index.0 + 1, // 1-based
                title: m.label.clone(),
            })
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
        markers,
        has_audio: loaded.song.backing_track_audio.is_some(),
        sync_points: loaded
            .song
            .sync_points
            .iter()
            .map(|sp| SyncPointInfo {
                bar: sp.bar_index,
                bar_occurrence: sp.bar_occurrence,
                frame_offset: sp.frame_offset,
                modified_tempo: sp.modified_tempo,
            })
            .collect(),
    }))
}

/// Serve the embedded backing-track audio (MP3) for a GP7/GP8 session.
/// Returns 404 when the score carries no audio.
pub async fn audio(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let mp3 = {
        let sessions = state.sessions.read().await;
        let loaded = sessions
            .get(&id)
            .ok_or_else(|| ApiError::not_found("Score session not found"))?;
        loaded.touch();
        loaded
            .song
            .backing_track_audio
            .clone()
            .ok_or_else(|| ApiError::not_found("No embedded audio for this session"))?
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(mp3))
        .map_err(|e| ApiError::internal(e.to_string()))
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
