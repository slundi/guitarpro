use axum::Router;
use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use serde::Serialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

mod analysis;
mod extract;
mod files;
mod open;
mod score;
mod upload;

#[derive(Serialize)]
pub struct ScoreSummary {
    pub id: Uuid,
    pub name: String,
    pub track_count: u8,
    pub measure_count: u16,
}

/// Replace characters that would make an invalid or misleading
/// `Content-Disposition` header value. Control bytes (`< 0x20`, `0x7f`) are
/// rejected by the `http` crate and would otherwise turn `Response::builder`
/// into a panic; `"` and `\` would break out of the quoted `filename="…"`
/// parameter. Non-ASCII (UTF-8) is left intact — `http` accepts it as obs-text.
pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || c == '"' || c == '\\' {
                '_'
            } else {
                c
            }
        })
        .collect();
    if cleaned.trim().is_empty() {
        "download".to_string()
    } else {
        cleaned
    }
}

/// Build an `application/octet-stream` attachment response with a safe
/// `Content-Disposition` filename. Shared by the raw/download/extract handlers.
pub fn attachment(bytes: Vec<u8>, filename: &str) -> Result<Response, ApiError> {
    let safe = sanitize_filename(filename);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{safe}\""),
        )
        .body(Body::from(bytes))
        .map_err(|e| ApiError::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn strips_control_chars_and_quotes() {
        assert_eq!(sanitize_filename("foo\nbar.gp5"), "foo_bar.gp5");
        assert_eq!(sanitize_filename("a\"b\\c.gp5"), "a_b_c.gp5");
        assert_eq!(sanitize_filename("tab\there.gp5"), "tab_here.gp5");
    }

    #[test]
    fn keeps_unicode_and_normal_names() {
        assert_eq!(sanitize_filename("Été_solo.gp5"), "Été_solo.gp5");
        assert_eq!(sanitize_filename("song.gp5"), "song.gp5");
    }

    #[test]
    fn falls_back_when_empty() {
        assert_eq!(sanitize_filename(""), "download");
        assert_eq!(sanitize_filename("   "), "download");
    }

    #[test]
    fn produces_a_valid_header_value_for_control_chars() {
        // The whole point: a control char in the name must not panic the builder.
        let safe = sanitize_filename("x\r\n.gp5");
        let value = format!("attachment; filename=\"{safe}\"");
        assert!(axum::http::HeaderValue::try_from(value).is_ok());
    }
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/files", get(files::list))
        .route("/api/duplicates", post(files::duplicates))
        .route("/api/score/upload", post(upload::handler))
        .route("/api/score/open", post(open::handler))
        .route("/api/score/{id}/extract", post(extract::handler))
        .route("/api/score/{id}/raw", get(score::raw))
        .route("/api/score/{id}/download", get(score::download))
        .route("/api/score/{id}/info", get(score::info))
        .route("/api/score/{id}/analysis/repeats", get(analysis::repeats))
        .route("/api/score/{id}/analysis/form", get(analysis::form))
        .route(
            "/api/score/{id}/analysis/fingering",
            get(analysis::fingering),
        )
}
