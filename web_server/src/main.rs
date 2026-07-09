use anyhow::{Context, Result};
use argh::FromArgs;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

mod api;
mod config;
mod error;
mod state;

use config::ServerConfig;
use state::AppState;

const DEFAULT_PORT: u16 = 3000;

#[derive(FromArgs)]
/// Guitar score web viewer and analysis tool
struct Args {
    /// port to listen on (default: 3000, overrides score_server.toml)
    #[argh(option, short = 'p')]
    port: Option<u16>,

    /// suppress auto-opening the browser after binding
    #[argh(switch)]
    no_open: bool,

    /// root directory allowed for /api/score/open (default: $HOME)
    #[argh(option)]
    root: Option<std::path::PathBuf>,

    /// path to a score_server.toml config file (default: search cwd + XDG paths)
    #[argh(option, short = 'c')]
    config: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Args = argh::from_env();

    let file_config = match args.config.as_deref() {
        Some(path) => ServerConfig::load_from(path)
            .with_context(|| format!("Failed to load config from '{}'", path.display()))?,
        None => ServerConfig::load_default().context("Failed to load default config file")?,
    };

    // CLI flag > config file > built-in default.
    let port = args.port.or(file_config.port).unwrap_or(DEFAULT_PORT);
    let auto_open = if args.no_open {
        false
    } else {
        file_config.open.unwrap_or(true)
    };
    let root_input = args.root.or(file_config.root).unwrap_or_else(default_root);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let root = root_input.canonicalize().with_context(|| {
        format!(
            "Root directory '{}' does not exist or is not accessible",
            root_input.display()
        )
    })?;
    tracing::info!(root = %root.display(), "file open root");

    let state = AppState::new(root);
    state.spawn_sweep();

    let router = build_router(state, port);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://localhost:{}", port);

    if auto_open {
        let url = format!("http://localhost:{port}");
        if let Err(e) = open::that(&url) {
            tracing::warn!("Failed to open browser: {e}");
        }
    }

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn default_root() -> std::path::PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}

async fn shutdown_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("failed to install Ctrl+C handler: {e}");
        return;
    }
    tracing::info!("Shutting down");
}

// ── Dev mode: serve files from frontend/dist/ on disk ────────────────────────

#[cfg(not(feature = "embed"))]
fn build_router(state: AppState, _port: u16) -> Router {
    use tower_http::services::{ServeDir, ServeFile};

    let dist = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist");
    let index = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist/index.html");

    if !std::path::Path::new(dist).exists() {
        tracing::warn!(
            path = dist,
            "frontend/dist not found — run `pnpm build` in web_server/frontend/ first"
        );
    }

    api::api_routes()
        .fallback_service(ServeDir::new(dist).fallback(ServeFile::new(index)))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
}

// ── Embed mode: assets bundled into the binary at compile time ────────────────
// Build the frontend first: `pnpm --dir web_server/frontend build`
// Then: `cargo build -p web_server --features embed`

#[cfg(feature = "embed")]
mod embedded {
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "frontend/dist/"]
    pub struct Assets;
}

#[cfg(feature = "embed")]
fn build_router(state: AppState, port: u16) -> Router {
    use axum::{
        http::{HeaderValue, Method, StatusCode, Uri, header},
        response::{IntoResponse, Response},
        routing::get,
    };
    use embedded::Assets;

    fn serve_asset(path: &str) -> Response {
        match Assets::get(path) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                (
                    [(header::CONTENT_TYPE, mime.as_ref().to_string())],
                    content.data.to_vec(),
                )
                    .into_response()
            }
            None => match Assets::get("index.html") {
                Some(index) => (
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8".to_string())],
                    index.data.to_vec(),
                )
                    .into_response(),
                None => (StatusCode::NOT_FOUND, "index.html not embedded").into_response(),
            },
        }
    }

    async fn static_handler(uri: Uri) -> impl IntoResponse {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };
        serve_asset(path)
    }

    let origins: Vec<HeaderValue> = [
        format!("http://localhost:{port}"),
        format!("http://127.0.0.1:{port}"),
    ]
    .into_iter()
    .filter_map(|s| s.parse().ok())
    .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    api::api_routes()
        .fallback(get(static_handler))
        .with_state(state)
        .layer(cors)
        .layer(CompressionLayer::new())
}
