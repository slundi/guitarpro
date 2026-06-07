use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    detail: String,
}

pub struct ApiError(StatusCode, ErrorBody);

impl ApiError {
    pub fn bad_request(error: impl Into<String>, detail: impl Into<String>) -> Self {
        Self(
            StatusCode::BAD_REQUEST,
            ErrorBody {
                error: error.into(),
                detail: detail.into(),
            },
        )
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self(
            StatusCode::NOT_FOUND,
            ErrorBody {
                error: "Not found".into(),
                detail: detail.into(),
            },
        )
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self(
            StatusCode::FORBIDDEN,
            ErrorBody {
                error: "Forbidden".into(),
                detail: detail.into(),
            },
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(self.1)).into_response()
    }
}
