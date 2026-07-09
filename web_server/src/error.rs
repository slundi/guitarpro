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

    pub fn internal(detail: impl Into<String>) -> Self {
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorBody {
                error: "Internal error".into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    /// The frontend's toast layer parses `{ error, detail }` — freeze the
    /// contract so a rename here can't silently break every error toast.
    async fn body_json(err: ApiError) -> (StatusCode, serde_json::Value) {
        let resp = err.into_response();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn bad_request_body_has_error_and_detail() {
        let (status, json) = body_json(ApiError::bad_request("Parse error", "bad header")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"], "Parse error");
        assert_eq!(json["detail"], "bad header");
    }

    #[tokio::test]
    async fn not_found_body_uses_fixed_error_label() {
        let (status, json) = body_json(ApiError::not_found("Score session not found")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["error"], "Not found");
        assert_eq!(json["detail"], "Score session not found");
    }

    #[tokio::test]
    async fn forbidden_and_internal_carry_the_right_status() {
        let (status, json) = body_json(ApiError::forbidden("outside root")).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(json["error"], "Forbidden");
        assert_eq!(json["detail"], "outside root");

        let (status, json) = body_json(ApiError::internal("kaboom")).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["error"], "Internal error");
        assert_eq!(json["detail"], "kaboom");
    }
}
