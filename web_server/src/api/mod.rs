use axum::Router;
use axum::routing::{get, post};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

mod analysis;
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

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/score/upload", post(upload::handler))
        .route("/api/score/open", post(open::handler))
        .route("/api/score/{id}/raw", get(score::raw))
        .route("/api/score/{id}/info", get(score::info))
        .route("/api/score/{id}/analysis/repeats", get(analysis::repeats))
}
