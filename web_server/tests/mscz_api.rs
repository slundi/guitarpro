//! Web-server integration tests for the MSCZ (MuseScore) path.
//!
//! Each test builds a synthetic MSCZ archive in memory (no copyrighted
//! files committed to the repo), routes an HTTP request through the real
//! `api_routes()` router, and asserts on the response body / status.

use std::io::Write;
use std::path::Path;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use tower::ServiceExt;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use web_server::api::api_routes;
use web_server::state::AppState;

// ---------------------------------------------------------------------------
// Fixture builder
// ---------------------------------------------------------------------------

const MSCX_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.10">
  <programVersion>4.2.0</programVersion>
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">Web Fixture</metaTag>
    <metaTag name="composer">Fixture Composer</metaTag>
    <Part id="1">
      <Staff id="1">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        <defaultClef>G</defaultClef>
        </Staff>
      <trackName>Fixture Guitar</trackName>
      <Instrument id="pluck.guitar.classical">
        <longName>Classical Guitar</longName>
        <shortName>c.g.</shortName>
        <instrumentId>pluck.guitar.classical</instrumentId>
        <StringData>
          <frets>19</frets>
          <string>40</string><string>45</string><string>50</string>
          <string>55</string><string>59</string><string>64</string>
          </StringData>
        </Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <KeySig><accidental>0</accidental></KeySig>
          <Tempo><tempo>2.0</tempo></Tempo>
          <Chord><durationType>quarter</durationType>
            <Note><pitch>52</pitch><tpc>18</tpc></Note>
            </Chord>
          <Rest><durationType>quarter</durationType></Rest>
          <Rest><durationType>half</durationType></Rest>
          </voice>
        </Measure>
      </Staff>
    </Score>
  </museScore>
"#;

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container>
  <rootfiles>
    <rootfile full-path="score.mscx"/>
    </rootfiles>
  </container>
"#;

/// PNG magic bytes + short stub payload. Real PNG structure is not required
/// — the server passes the bytes through unmodified.
const STUB_PNG: &[u8] = b"\x89PNG\r\n\x1a\nfixture-thumbnail";

fn build_mscz(with_thumbnail: bool) -> Vec<u8> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    writer
        .start_file("META-INF/container.xml", deflate)
        .unwrap();
    writer.write_all(CONTAINER_XML.as_bytes()).unwrap();

    writer.start_file("score.mscx", deflate).unwrap();
    writer.write_all(MSCX_FIXTURE.as_bytes()).unwrap();

    if with_thumbnail {
        writer
            .start_file("Thumbnails/thumbnail.png", stored)
            .unwrap();
        writer.write_all(STUB_PNG).unwrap();
    }

    writer.finish().unwrap().into_inner()
}

// ---------------------------------------------------------------------------
// Test scaffolding
// ---------------------------------------------------------------------------

/// Bundle a router + root dir so tests can address paths under `--root`.
fn app_with_root(root: &Path) -> axum::Router {
    let state = AppState::new(root.to_path_buf());
    api_routes().with_state(state)
}

/// Multipart body carrying a single `file` field.
fn multipart_body(boundary: &str, filename: &str, data: &[u8]) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

async fn upload_mscz(app: &axum::Router, filename: &str, bytes: &[u8]) -> serde_json::Value {
    let boundary = "----boundary123";
    let body = multipart_body(boundary, filename, bytes);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/score/upload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "upload should succeed for '{filename}'"
    );
    let raw = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&raw).expect("upload returns valid JSON")
}

async fn get(app: &axum::Router, uri: &str) -> axum::http::Response<Body> {
    let request = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(request).await.unwrap()
}

// ---------------------------------------------------------------------------
// Upload path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_accepts_mscz_and_returns_summary() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let bytes = build_mscz(false);
    let summary = upload_mscz(&app, "fixture.mscz", &bytes).await;

    assert_eq!(summary["name"], "fixture.mscz");
    assert!(summary["id"].is_string());
    assert!(
        summary["track_count"].as_u64().unwrap() >= 1,
        "expected at least one track"
    );
}

#[tokio::test]
async fn upload_rejects_unknown_extension() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let boundary = "----boundary321";
    let body = multipart_body(boundary, "song.exe", b"binary");
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/score/upload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Info + raw endpoints (unchanged code, but need to serve MSCZ sessions)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn info_endpoint_returns_metadata_for_mscz_session() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let bytes = build_mscz(false);
    let summary = upload_mscz(&app, "fixture.mscz", &bytes).await;
    let id = summary["id"].as_str().unwrap();

    let response = get(&app, &format!("/api/score/{id}/info")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["title"], "Web Fixture");
    let tracks = json["tracks"].as_array().unwrap();
    assert!(!tracks.is_empty());
    assert_eq!(tracks[0]["name"], "Fixture Guitar");
    // 6-string classical guitar.
    assert_eq!(tracks[0]["string_count"], 6);
    let tuning = tracks[0]["tuning"].as_array().unwrap();
    assert_eq!(tuning.len(), 6);
    assert_eq!(tuning[0].as_i64(), Some(40));
}

#[tokio::test]
async fn raw_endpoint_returns_source_mscz_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let bytes = build_mscz(false);
    let summary = upload_mscz(&app, "fixture.mscz", &bytes).await;
    let id = summary["id"].as_str().unwrap();

    let response = get(&app, &format!("/api/score/{id}/raw")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let cd = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cd.contains("fixture.mscz"));
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        body.as_ref(),
        bytes.as_slice(),
        "raw bytes must round-trip verbatim"
    );
}

// ---------------------------------------------------------------------------
// Analysis endpoints (verify the loader bridge doesn't break them)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn analysis_repeats_endpoint_works_on_mscz() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let bytes = build_mscz(false);
    let summary = upload_mscz(&app, "fixture.mscz", &bytes).await;
    let id = summary["id"].as_str().unwrap();

    let response = get(&app, &format!("/api/score/{id}/analysis/repeats")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    // The endpoint must at least return valid JSON — repeat-block content
    // depends on the score, but the shape must be parseable.
    let _: serde_json::Value = serde_json::from_slice(&body).expect("valid JSON");
}

// ---------------------------------------------------------------------------
// Download endpoint with format=mscz
// ---------------------------------------------------------------------------

#[tokio::test]
async fn download_format_mscz_returns_a_valid_archive() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let bytes = build_mscz(false);
    let summary = upload_mscz(&app, "fixture.mscz", &bytes).await;
    let id = summary["id"].as_str().unwrap();

    let response = get(&app, &format!("/api/score/{id}/download?format=mscz")).await;
    assert_eq!(response.status(), StatusCode::OK);

    let cd = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cd.contains("fixture.mscz"));

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        &body[..4],
        b"PK\x03\x04",
        "download must produce a ZIP archive"
    );

    // The archive must contain the expected entries.
    let cursor = std::io::Cursor::new(body.to_vec());
    let mut archive = zip::ZipArchive::new(cursor).expect("valid ZIP");
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    names.sort();
    assert!(names.iter().any(|n| n == "META-INF/container.xml"));
    assert!(names.iter().any(|n| n == "score.mscx"));
}

// ---------------------------------------------------------------------------
// Thumbnail endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn score_thumbnail_returns_embedded_png() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let bytes = build_mscz(true);
    let summary = upload_mscz(&app, "fixture.mscz", &bytes).await;
    let id = summary["id"].as_str().unwrap();

    let response = get(&app, &format!("/api/score/{id}/thumbnail")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/png"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), STUB_PNG);
}

#[tokio::test]
async fn score_thumbnail_returns_404_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());

    let bytes = build_mscz(false); // no thumbnail
    let summary = upload_mscz(&app, "fixture.mscz", &bytes).await;
    let id = summary["id"].as_str().unwrap();

    let response = get(&app, &format!("/api/score/{id}/thumbnail")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn files_thumbnail_reads_embedded_png_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    // Note: the file browser rejects paths outside `--root`, so we must
    // place the fixture inside the canonicalised root.
    let mscz_path = root.join("fixture.mscz");
    std::fs::write(&mscz_path, build_mscz(true)).unwrap();
    let app = app_with_root(&root);

    let uri = format!(
        "/api/files/thumbnail?path={}",
        urlencoded(mscz_path.to_string_lossy().as_ref())
    );
    let response = get(&app, &uri).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/png"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), STUB_PNG);
}

#[tokio::test]
async fn files_thumbnail_rejects_non_mscz_path() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let gp5_path = root.join("song.gp5");
    std::fs::write(&gp5_path, b"not really a gp5").unwrap();
    let app = app_with_root(&root);

    let uri = format!(
        "/api/files/thumbnail?path={}",
        urlencoded(gp5_path.to_string_lossy().as_ref())
    );
    let response = get(&app, &uri).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn files_thumbnail_forbids_paths_outside_root() {
    let dir_root = tempfile::tempdir().unwrap();
    let dir_out = tempfile::tempdir().unwrap();
    let mscz_path = dir_out.path().join("outside.mscz");
    std::fs::write(&mscz_path, build_mscz(true)).unwrap();
    let app = app_with_root(dir_root.path());

    let uri = format!(
        "/api/files/thumbnail?path={}",
        urlencoded(mscz_path.to_string_lossy().as_ref())
    );
    let response = get(&app, &uri).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ---------------------------------------------------------------------------
// File browser lists MSCZ files
// ---------------------------------------------------------------------------

#[tokio::test]
async fn files_list_includes_mscz_entries() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("song.mscz"), build_mscz(false)).unwrap();
    std::fs::write(root.join("readme.txt"), b"skip me").unwrap();
    let app = app_with_root(&root);

    let response = get(&app, "/api/files").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let names: Vec<&str> = json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"song.mscz"),
        "expected song.mscz in listing: {names:?}"
    );
    assert!(
        !names.contains(&"readme.txt"),
        "non-score files should be filtered out"
    );
}

// ---------------------------------------------------------------------------
// Shipped-fixture walker (Part 5.4)
// ---------------------------------------------------------------------------
//
// Cross-check: the 7 pre-generated fixtures under `guitarpro/samples/mscz/`
// must all upload cleanly, expose valid info, and reach the analysis
// endpoints. Kept in sync with `guitarpro/src/tests/mscz_fixtures.rs`.

/// The committed fixture names in `guitarpro/samples/mscz/`.
const SHIPPED_FIXTURES: &[&str] = &[
    "simple_monophonic",
    "multi_track_band",
    "alternate_tuning",
    "repeats_and_voltas",
    "empty_score",
    "single_measure",
    "four_voices",
];

fn samples_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("web_server has a workspace parent")
        .join("guitarpro")
        .join("samples")
        .join("mscz")
}

#[tokio::test]
async fn shipped_fixtures_upload_and_expose_info() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());
    let samples = samples_dir();
    let mut missing: Vec<&str> = Vec::new();

    for name in SHIPPED_FIXTURES {
        let path = samples.join(format!("{name}.mscz"));
        if !path.exists() {
            missing.push(name);
            continue;
        }
        let bytes = std::fs::read(&path).expect("read committed fixture");
        let summary = upload_mscz(&app, &format!("{name}.mscz"), &bytes).await;
        let id = summary["id"].as_str().unwrap();

        let response = get(&app, &format!("/api/score/{id}/info")).await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{name}: /info must succeed"
        );

        let response = get(&app, &format!("/api/score/{id}/analysis/repeats")).await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{name}: /analysis/repeats must succeed"
        );
    }

    assert!(
        missing.is_empty(),
        "committed fixtures missing: {missing:?} — run \
         `cargo test -p guitarpro write_mscz_samples_to_disk -- --ignored --exact`"
    );
}

#[tokio::test]
async fn shipped_fixtures_can_be_downloaded_as_mscz() {
    let dir = tempfile::tempdir().unwrap();
    let app = app_with_root(dir.path());
    let samples = samples_dir();

    for name in SHIPPED_FIXTURES {
        let path = samples.join(format!("{name}.mscz"));
        if !path.exists() {
            continue;
        }
        let bytes = std::fs::read(&path).expect("read committed fixture");
        let summary = upload_mscz(&app, &format!("{name}.mscz"), &bytes).await;
        let id = summary["id"].as_str().unwrap();

        let response = get(&app, &format!("/api/score/{id}/download?format=mscz")).await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{name}: /download?format=mscz must succeed"
        );
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            &body[..4],
            b"PK\x03\x04",
            "{name}: download must produce a valid ZIP archive"
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal URL-encoding wrapper. `serde_urlencoded::to_string` handles the
/// full escape table (spaces → `%20`, `/` → `%2F`, etc.).
fn urlencoded(input: &str) -> String {
    serde_urlencoded::to_string([("v", input)])
        .unwrap()
        .strip_prefix("v=")
        .unwrap()
        .to_string()
}
