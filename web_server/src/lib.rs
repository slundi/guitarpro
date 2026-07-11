//! Library-facing entry points for `web_server`, exposed so integration
//! tests under `tests/` can spin up the API router directly.

pub mod api;
pub mod config;
pub mod error;
pub mod state;
