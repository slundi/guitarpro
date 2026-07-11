//! CLI integration tests for the MSCZ path (Part 3 of the MSCZ roadmap).
//!
//! Fixtures are built programmatically to avoid checking copyrighted
//! MuseScore files into the repository. The synthetic MSCX is small but
//! rich enough to exercise every CLI branch: metadata (title/composer),
//! part with `<StringData>` tuning, and two measures with notes so
//! `repeats`, `info`, and `convert` all produce meaningful output.

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

// ---------------------------------------------------------------------------
// Fixture: synthetic MSCZ
// ---------------------------------------------------------------------------

const MSCX_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.10">
  <programVersion>4.2.0</programVersion>
  <programRevision>fixture</programRevision>
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">CLI Fixture</metaTag>
    <metaTag name="composer">Test Composer</metaTag>
    <metaTag name="copyright">Public Domain</metaTag>
    <Part id="1">
      <Staff id="1">
        <StaffType group="pitched"><name>stdNormal</name></StaffType>
        <defaultClef>G</defaultClef>
        </Staff>
      <trackName>Test Guitar</trackName>
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
      <Measure>
        <startRepeat/>
        <voice>
          <Chord><durationType>quarter</durationType>
            <Note><pitch>55</pitch></Note>
            </Chord>
          <Rest><durationType>quarter</durationType></Rest>
          <Rest><durationType>half</durationType></Rest>
          </voice>
        </Measure>
      <Measure>
        <endRepeat>2</endRepeat>
        <voice>
          <Chord><durationType>quarter</durationType>
            <Note><pitch>60</pitch></Note>
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

/// Build a full MSCZ archive around [`MSCX_FIXTURE`], including a stub
/// PNG thumbnail so the `mscz thumbnail` sub-command has something to
/// extract.
fn build_mscz() -> Vec<u8> {
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

    writer
        .start_file("Thumbnails/thumbnail.png", stored)
        .unwrap();
    writer.write_all(b"\x89PNG\r\n\x1a\nSTUB").unwrap();

    writer.finish().unwrap().into_inner()
}

fn write_fixture(dir: &std::path::Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, build_mscz()).unwrap();
    path
}

fn score_tool() -> Command {
    // Cargo populates this env var to the built binary for integration tests.
    Command::new(env!("CARGO_BIN_EXE_score_tool"))
}

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

// ---------------------------------------------------------------------------
// info
// ---------------------------------------------------------------------------

#[test]
fn info_reads_mscz_and_prints_metadata() {
    let dir = tempdir();
    let path = write_fixture(dir.path(), "fixture.mscz");

    let out = score_tool()
        .arg("info")
        .arg("-i")
        .arg(&path)
        .output()
        .expect("score_tool info");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("MuseScore (MSCZ)"),
        "expected format label: {stdout}"
    );
    assert!(stdout.contains("CLI Fixture"), "expected title: {stdout}");
    assert!(
        stdout.contains("Test Composer"),
        "expected composer: {stdout}"
    );
    assert!(
        stdout.contains("Test Guitar"),
        "expected track name: {stdout}"
    );
}

#[test]
fn info_json_output_includes_track_and_timeline() {
    let dir = tempdir();
    let path = write_fixture(dir.path(), "fixture.mscz");

    let out = score_tool()
        .arg("info")
        .arg("-i")
        .arg(&path)
        .arg("--json")
        .output()
        .expect("score_tool info --json");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");
    assert_eq!(json["format"], "MSCZ");
    assert_eq!(json["title"], "CLI Fixture");
    assert_eq!(json["track_count"], 1);
    // 3 measures in the fixture (one has repeat_open, one repeat_close).
    assert_eq!(json["measure_count"], 3);
}

// ---------------------------------------------------------------------------
// convert
// ---------------------------------------------------------------------------

#[test]
fn convert_mscz_to_mscz_produces_valid_archive() {
    let dir = tempdir();
    let src = write_fixture(dir.path(), "source.mscz");
    let dst = dir.path().join("out.mscz");

    let out = score_tool()
        .arg("convert")
        .arg("-i")
        .arg(&src)
        .arg("-o")
        .arg(&dst)
        .output()
        .expect("score_tool convert");

    assert!(
        out.status.success(),
        "convert failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dst.exists(), "output MSCZ was not created");

    // Verify the produced file is a readable MSCZ.
    let read = score_tool()
        .arg("info")
        .arg("-i")
        .arg(&dst)
        .output()
        .expect("info on output");
    assert!(read.status.success(), "cannot read produced MSCZ");
    let stdout = String::from_utf8_lossy(&read.stdout);
    assert!(stdout.contains("MuseScore (MSCZ)"));
    assert!(stdout.contains("CLI Fixture"));
}

#[test]
fn convert_mscz_to_musicxml_writes_xml() {
    let dir = tempdir();
    let src = write_fixture(dir.path(), "source.mscz");
    let dst = dir.path().join("out.musicxml");

    let out = score_tool()
        .arg("convert")
        .arg("-i")
        .arg(&src)
        .arg("-o")
        .arg(&dst)
        .output()
        .expect("score_tool convert to musicxml");

    assert!(
        out.status.success(),
        "convert failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dst.exists());
    let contents = std::fs::read_to_string(&dst).unwrap();
    assert!(
        contents.starts_with("<?xml"),
        "expected XML output: {contents:.80}"
    );
    assert!(contents.contains("score-partwise") || contents.contains("ScorePartwise"));
}

#[test]
fn convert_rejects_unknown_input_format() {
    let dir = tempdir();
    let bogus = dir.path().join("nope.xyz");
    std::fs::write(&bogus, b"not a score file").unwrap();
    let dst = dir.path().join("out.mscz");

    let out = score_tool()
        .arg("convert")
        .arg("-i")
        .arg(&bogus)
        .arg("-o")
        .arg(&dst)
        .output()
        .expect("score_tool convert");
    assert!(!out.status.success(), "should refuse unknown extension");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("input format"), "stderr: {stderr}");
}

// ---------------------------------------------------------------------------
// repeats (structural analysis works through the loader bridge)
// ---------------------------------------------------------------------------

#[test]
fn repeats_reads_mscz_and_finds_repeat_block() {
    let dir = tempdir();
    let path = write_fixture(dir.path(), "fixture.mscz");

    let out = score_tool()
        .arg("repeats")
        .arg("-i")
        .arg(&path)
        .output()
        .expect("score_tool repeats");

    assert!(
        out.status.success(),
        "repeats failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Fixture has `|: bar 2 :|×2` which should surface as a repeat block.
    assert!(
        stdout.contains("|:") || stdout.to_lowercase().contains("repeat"),
        "expected repeat markers in output: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// mscz sub-command
// ---------------------------------------------------------------------------

#[test]
fn mscz_list_prints_archive_entries() {
    let dir = tempdir();
    let path = write_fixture(dir.path(), "fixture.mscz");

    let out = score_tool()
        .args(["mscz", "list", "-i"])
        .arg(&path)
        .output()
        .expect("mscz list");
    assert!(
        out.status.success(),
        "mscz list failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("score.mscx"));
    assert!(stdout.contains("Thumbnails/thumbnail.png"));
    assert!(stdout.contains("META-INF/container.xml"));
}

#[test]
fn mscz_list_json_is_valid() {
    let dir = tempdir();
    let path = write_fixture(dir.path(), "fixture.mscz");

    let out = score_tool()
        .args(["mscz", "list", "-i"])
        .arg(&path)
        .arg("--json")
        .output()
        .expect("mscz list --json");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = json.as_array().expect("expected JSON array");
    let paths: Vec<&str> = arr.iter().map(|v| v["path"].as_str().unwrap()).collect();
    assert!(paths.contains(&"score.mscx"));
    assert!(paths.contains(&"META-INF/container.xml"));
}

#[test]
fn mscz_extract_writes_all_entries() {
    let dir = tempdir();
    let path = write_fixture(dir.path(), "fixture.mscz");
    let out_dir = dir.path().join("unpack");

    let out = score_tool()
        .args(["mscz", "extract", "-i"])
        .arg(&path)
        .args(["-o"])
        .arg(&out_dir)
        .output()
        .expect("mscz extract");
    assert!(
        out.status.success(),
        "mscz extract failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(out_dir.join("score.mscx").exists());
    assert!(out_dir.join("META-INF").join("container.xml").exists());
    assert!(out_dir.join("Thumbnails").join("thumbnail.png").exists());
    let mscx = std::fs::read_to_string(out_dir.join("score.mscx")).unwrap();
    assert!(mscx.contains("workTitle"));
}

#[test]
fn mscz_extract_refuses_path_traversal() {
    // Craft an archive with a `../` entry name.
    let dir = tempdir();
    let malicious = dir.path().join("evil.mscz");

    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer
        .start_file("META-INF/container.xml", deflate)
        .unwrap();
    writer.write_all(CONTAINER_XML.as_bytes()).unwrap();
    writer.start_file("../escape.txt", deflate).unwrap();
    writer.write_all(b"hi").unwrap();
    writer.start_file("score.mscx", deflate).unwrap();
    writer.write_all(MSCX_FIXTURE.as_bytes()).unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    std::fs::write(&malicious, bytes).unwrap();

    let out_dir = dir.path().join("unpack");
    let out = score_tool()
        .args(["mscz", "extract", "-i"])
        .arg(&malicious)
        .args(["-o"])
        .arg(&out_dir)
        .output()
        .expect("mscz extract");
    assert!(!out.status.success(), "traversal path should be rejected");
    // The escape file must not exist on disk.
    assert!(!dir.path().join("escape.txt").exists());
}

#[test]
fn mscz_thumbnail_writes_png_bytes() {
    let dir = tempdir();
    let path = write_fixture(dir.path(), "fixture.mscz");
    let png = dir.path().join("thumb.png");

    let out = score_tool()
        .args(["mscz", "thumbnail", "-i"])
        .arg(&path)
        .args(["--out"])
        .arg(&png)
        .output()
        .expect("mscz thumbnail");
    assert!(
        out.status.success(),
        "mscz thumbnail failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let data = std::fs::read(&png).unwrap();
    assert!(data.starts_with(b"\x89PNG"), "expected PNG magic");
}

// ---------------------------------------------------------------------------
// duplicates walker includes .mscz files
// ---------------------------------------------------------------------------

#[test]
fn duplicates_walker_picks_up_mscz_files() {
    let dir = tempdir();
    write_fixture(dir.path(), "one.mscz");
    write_fixture(dir.path(), "two.mscz");

    let out = score_tool()
        .args(["duplicates", "-d"])
        .arg(dir.path())
        .output()
        .expect("score_tool duplicates");
    assert!(
        out.status.success(),
        "duplicates failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Two identical fixtures should be flagged as one duplicate group.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Either the scanner logs the file count on stderr, or the stdout
    // reports "duplicate group(s)".
    assert!(
        stderr.contains("Scanning 2 file") || stdout.contains("duplicate"),
        "expected scan to include both mscz files. stdout: {stdout}, stderr: {stderr}"
    );
}
