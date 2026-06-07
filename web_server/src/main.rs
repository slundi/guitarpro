use anyhow::Result;
use argh::FromArgs;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

#[derive(FromArgs)]
/// Guitar score web viewer and analysis tool
struct Args {
    /// port to listen on (default: 3000)
    #[argh(option, short = 'p', default = "3000")]
    port: u16,

    /// open the browser after binding
    #[argh(switch, short = 'o')]
    open: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Args = argh::from_env();
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));

    let router = build_router();
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://localhost:{}", args.port);

    if args.open {
        let url = format!("http://localhost:{}", args.port);
        if let Err(e) = open::that(&url) {
            tracing::warn!("Failed to open browser: {e}");
        }
    }

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Shutting down");
}

// ── Dev mode: serve files from frontend/dist/ on disk ────────────────────────

#[cfg(not(feature = "embed"))]
fn build_router() -> Router {
    use tower_http::services::{ServeDir, ServeFile};

    let dist = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist");
    let index = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist/index.html");

    if !std::path::Path::new(dist).exists() {
        tracing::warn!(
            path = dist,
            "frontend/dist not found — run `pnpm build` in web_server/frontend/ first"
        );
    }

    let serve = ServeDir::new(dist).fallback(ServeFile::new(index));

    Router::new()
        .fallback_service(serve)
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
fn build_router() -> Router {
    use axum::{
        body::Body,
        http::{StatusCode, Uri, header},
        response::{IntoResponse, Response},
        routing::get,
    };
    use embedded::Assets;

    fn serve_asset(path: &str) -> Response {
        match Assets::get(path) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(Body::from(content.data.to_vec()))
                    .unwrap()
            }
            None => {
                // SPA fallback: unknown paths → index.html
                let index = Assets::get("index.html").expect("index.html not embedded");
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(index.data.to_vec()))
                    .unwrap()
            }
        }
    }

    async fn static_handler(uri: Uri) -> impl IntoResponse {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };
        serve_asset(path)
    }

    Router::new()
        .fallback(get(static_handler))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
}
