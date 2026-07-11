//! MSCZ Part 1 tests: container + MSCX AST + round-trip.
//!
//! Fixtures are built programmatically to keep the repository free of
//! copyrighted MuseScore output. The synthetic MSCX exercises every parser
//! branch (envelope version, program version/revision, division, meta tags,
//! parts, part staves, instruments with `StringData`, top-level `Score` staff
//! with measures).

use std::io::Write;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::GpError;
use crate::io::mscz::{read_container, read_mscz_bytes, write_container, write_mscz};
use crate::model::mscz::{MsczArchive, MsczEntry};

// ---------------------------------------------------------------------------
// Fixture builders
// ---------------------------------------------------------------------------

const MSCX_MINIMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.10">
  <programVersion>4.1.1</programVersion>
  <programRevision>e4d1ddf</programRevision>
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">Fixture Piece</metaTag>
    <metaTag name="composer">Jane &amp; John Doe</metaTag>
    <metaTag name="copyright"></metaTag>
    <Part id="1">
      <Staff id="1">
        <StaffType group="tablature">
          <name>tab6StrCommon</name>
          </StaffType>
        <defaultClef>G8vb</defaultClef>
        </Staff>
      <trackName>Guitar</trackName>
      <Instrument id="electric-guitar-tablature">
        <longName>Electric Guitar</longName>
        <shortName>e.g.</shortName>
        <trackName></trackName>
        <transposeDiatonic>0</transposeDiatonic>
        <transposeChromatic>0</transposeChromatic>
        <instrumentId>pluck.guitar.electric</instrumentId>
        <StringData>
          <frets>24</frets>
          <string>40</string>
          <string>45</string>
          <string>50</string>
          <string>55</string>
          <string>59</string>
          <string>64</string>
          </StringData>
        </Instrument>
      </Part>
    <Staff id="1">
      <Measure>
        <voice>
          <Rest><durationType>whole</durationType></Rest>
          </voice>
        </Measure>
      <Measure>
        <voice>
          <Rest><durationType>whole</durationType></Rest>
          </voice>
        </Measure>
      <Measure>
        <voice>
          <Rest><durationType>whole</durationType></Rest>
          </voice>
        </Measure>
      </Staff>
    </Score>
  </museScore>
"#;

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container>
  <rootfiles>
    <rootfile full-path="fixture.mscx"/>
    </rootfiles>
  </container>
"#;

const VIEWSETTINGS_JSON: &str = r#"{"notation":{"viewMode":"page"}}"#;

/// Build a minimal MSCZ archive matching MuseScore 4.x layout.
///
/// Contains:
/// * `META-INF/container.xml` — manifest
/// * `fixture.mscx` — synthetic score XML above
/// * `viewsettings.json` — arbitrary side file preserved verbatim
/// * `Thumbnails/thumbnail.png` — a stub PNG (raw bytes, stored uncompressed)
fn build_minimal_mscz() -> Vec<u8> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);

    let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    writer
        .start_file("META-INF/container.xml", deflate)
        .unwrap();
    writer.write_all(CONTAINER_XML.as_bytes()).unwrap();

    writer.start_file("fixture.mscx", deflate).unwrap();
    writer.write_all(MSCX_MINIMAL.as_bytes()).unwrap();

    writer.start_file("viewsettings.json", deflate).unwrap();
    writer.write_all(VIEWSETTINGS_JSON.as_bytes()).unwrap();

    writer
        .start_file("Thumbnails/thumbnail.png", stored)
        .unwrap();
    // Not a real PNG but the container layer doesn't care.
    writer.write_all(b"\x89PNG\r\n\x1a\nSTUB").unwrap();

    let cursor = writer.finish().unwrap();
    cursor.into_inner()
}

// ---------------------------------------------------------------------------
// Container tests
// ---------------------------------------------------------------------------

#[test]
fn container_reads_all_entries_and_rootfiles() {
    let bytes = build_minimal_mscz();
    let archive = read_container(&bytes).expect("read_container");

    assert_eq!(archive.rootfiles, vec!["fixture.mscx".to_string()]);
    let paths: Vec<&str> = archive
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    assert!(paths.contains(&"META-INF/container.xml"));
    assert!(paths.contains(&"fixture.mscx"));
    assert!(paths.contains(&"viewsettings.json"));
    assert!(paths.contains(&"Thumbnails/thumbnail.png"));
}

#[test]
fn container_lookup_helpers_locate_expected_entries() {
    let bytes = build_minimal_mscz();
    let archive = read_container(&bytes).expect("read_container");

    assert_eq!(archive.mscx_entry().unwrap().path, "fixture.mscx");
    assert_eq!(
        archive.thumbnail_entry().unwrap().path,
        "Thumbnails/thumbnail.png"
    );
    assert_eq!(
        archive.view_settings_entry().unwrap().path,
        "viewsettings.json"
    );
    assert!(archive.style_entry().is_none());
    assert!(archive.audio_settings_entry().is_none());
}

#[test]
fn container_write_then_read_preserves_entries_byte_for_byte() {
    let bytes = build_minimal_mscz();
    let archive = read_container(&bytes).expect("read_container");
    let rewritten = write_container(&archive).expect("write_container");
    let reparsed = read_container(&rewritten).expect("read_container 2");

    assert_eq!(archive, reparsed, "archive must round-trip losslessly");
}

#[test]
fn container_rejects_oversize_input() {
    // Craft input larger than MAX_MSCZ_BYTES.
    let bytes = vec![0u8; crate::io::mscz::MAX_MSCZ_BYTES + 1];
    let error = read_container(&bytes).unwrap_err();
    match error {
        GpError::MsczArchive(_) => {}
        other => panic!("expected MsczArchive error, got {other:?}"),
    }
}

#[test]
fn container_rejects_non_zip_input() {
    let bytes = b"not a zip file".to_vec();
    let error = read_container(&bytes).unwrap_err();
    match error {
        GpError::MsczArchive(_) => {}
        other => panic!("expected MsczArchive error, got {other:?}"),
    }
}

#[test]
fn container_handles_missing_manifest() {
    // Build an archive with no META-INF/container.xml.
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    writer
        .start_file(
            "score.mscx",
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .unwrap();
    writer.write_all(MSCX_MINIMAL.as_bytes()).unwrap();
    let bytes = writer.finish().unwrap().into_inner();

    let archive = read_container(&bytes).expect("read without manifest");
    assert!(archive.rootfiles.is_empty());
    // mscx_entry falls back to any *.mscx not under META-INF/.
    assert_eq!(archive.mscx_entry().unwrap().path, "score.mscx");
}

// ---------------------------------------------------------------------------
// MSCX parse tests
// ---------------------------------------------------------------------------

#[test]
fn mscx_parses_envelope_metadata() {
    let bytes = build_minimal_mscz();
    let file = read_mscz_bytes(&bytes).expect("read_mscz");

    assert_eq!(file.mscx.version, "4.10");
    assert_eq!(file.mscx.program_version.as_deref(), Some("4.1.1"));
    assert_eq!(file.mscx.program_revision.as_deref(), Some("e4d1ddf"));
    assert_eq!(file.mscx.division, Some(480));
}

#[test]
fn mscx_extracts_meta_tags_with_entity_unescape() {
    let bytes = build_minimal_mscz();
    let file = read_mscz_bytes(&bytes).expect("read_mscz");

    assert_eq!(file.mscx.meta("workTitle"), Some("Fixture Piece"));
    // XML entity `&amp;` must be resolved back to `&`.
    assert_eq!(file.mscx.meta("composer"), Some("Jane & John Doe"));
    assert_eq!(file.mscx.meta("copyright"), Some(""));
    assert!(file.mscx.meta("nonexistent").is_none());
}

#[test]
fn mscx_extracts_part_and_instrument_details() {
    let bytes = build_minimal_mscz();
    let file = read_mscz_bytes(&bytes).expect("read_mscz");

    assert_eq!(file.mscx.parts.len(), 1);
    let part = &file.mscx.parts[0];
    assert_eq!(part.id, "1");
    assert_eq!(part.track_name.as_deref(), Some("Guitar"));

    assert_eq!(part.staves.len(), 1);
    let staff = &part.staves[0];
    assert_eq!(staff.id, "1");
    assert_eq!(staff.group.as_deref(), Some("tablature"));
    assert_eq!(staff.type_name.as_deref(), Some("tab6StrCommon"));
    assert_eq!(staff.default_clef.as_deref(), Some("G8vb"));

    let instrument = part.instrument.as_ref().expect("instrument");
    assert_eq!(instrument.id, "electric-guitar-tablature");
    assert_eq!(instrument.long_name.as_deref(), Some("Electric Guitar"));
    assert_eq!(instrument.short_name.as_deref(), Some("e.g."));
    assert_eq!(instrument.transpose_diatonic, Some(0));
    assert_eq!(instrument.transpose_chromatic, Some(0));
    assert_eq!(
        instrument.instrument_id.as_deref(),
        Some("pluck.guitar.electric")
    );

    let string_data = instrument.string_data.as_ref().expect("StringData");
    assert_eq!(string_data.frets, Some(24));
    assert_eq!(string_data.strings, vec![40, 45, 50, 55, 59, 64]);
}

#[test]
fn mscx_counts_measures_per_score_staff() {
    let bytes = build_minimal_mscz();
    let file = read_mscz_bytes(&bytes).expect("read_mscz");

    assert_eq!(file.mscx.measure_counts.len(), 1);
    assert_eq!(file.mscx.measure_counts[0].staff_id, "1");
    assert_eq!(file.mscx.measure_counts[0].measure_count, 3);
}

#[test]
fn mscx_rejects_unsupported_version() {
    let xml = MSCX_MINIMAL.replace(r#"version="4.10""#, r#"version="2.06""#);
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    writer
        .start_file(
            "fixture.mscx",
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .unwrap();
    writer.write_all(xml.as_bytes()).unwrap();
    let bytes = writer.finish().unwrap().into_inner();

    let error = read_mscz_bytes(&bytes).unwrap_err();
    match error {
        GpError::MsczUnsupported { got, supported } => {
            assert_eq!(got, "2.06");
            assert_eq!(supported, "4.x");
        }
        other => panic!("expected MsczUnsupported, got {other:?}"),
    }
}

#[test]
fn mscx_rejects_missing_root_element() {
    let xml = "<?xml version=\"1.0\"?><notMuseScore/>";
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    writer
        .start_file(
            "fixture.mscx",
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .unwrap();
    writer.write_all(xml.as_bytes()).unwrap();
    let bytes = writer.finish().unwrap().into_inner();

    let error = read_mscz_bytes(&bytes).unwrap_err();
    match error {
        GpError::MsczXml(msg) => assert!(msg.contains("museScore"), "message: {msg}"),
        other => panic!("expected MsczXml, got {other:?}"),
    }
}

#[test]
fn mscx_read_bytes_errors_when_no_mscx_present() {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    writer
        .start_file(
            "readme.txt",
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .unwrap();
    writer.write_all(b"hello").unwrap();
    let bytes = writer.finish().unwrap().into_inner();

    let error = read_mscz_bytes(&bytes).unwrap_err();
    match error {
        GpError::MsczArchive(msg) => assert!(msg.contains(".mscx"), "message: {msg}"),
        other => panic!("expected MsczArchive, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// End-to-end round-trip
// ---------------------------------------------------------------------------

#[test]
fn mscz_full_roundtrip_preserves_semantics() {
    let bytes = build_minimal_mscz();
    let file = read_mscz_bytes(&bytes).expect("read_mscz");

    let rewritten = write_mscz(&file).expect("write_mscz");
    let reparsed = read_mscz_bytes(&rewritten).expect("read_mscz 2");

    assert_eq!(file.mscx, reparsed.mscx);
    assert_eq!(file.archive, reparsed.archive);
}

#[test]
fn mscz_write_reflects_raw_xml_mutation() {
    let bytes = build_minimal_mscz();
    let mut file = read_mscz_bytes(&bytes).expect("read_mscz");

    // Simulate a higher-level converter regenerating the XML.
    file.mscx.raw_xml = file.mscx.raw_xml.replace("Fixture Piece", "Renamed Piece");

    let rewritten = write_mscz(&file).expect("write_mscz");
    let reparsed = read_mscz_bytes(&rewritten).expect("read_mscz 2");

    assert_eq!(reparsed.mscx.meta("workTitle"), Some("Renamed Piece"));
}

// ---------------------------------------------------------------------------
// Direct MsczArchive assembly (no round-trip via read)
// ---------------------------------------------------------------------------

#[test]
fn write_container_from_hand_built_archive() {
    let archive = MsczArchive {
        rootfiles: vec!["hand.mscx".to_string()],
        entries: vec![
            MsczEntry {
                path: "META-INF/container.xml".to_string(),
                data: br#"<?xml version="1.0"?><container><rootfiles><rootfile full-path="hand.mscx"/></rootfiles></container>"#.to_vec(),
            },
            MsczEntry {
                path: "hand.mscx".to_string(),
                data: MSCX_MINIMAL.as_bytes().to_vec(),
            },
        ],
    };

    let bytes = write_container(&archive).expect("write_container");
    let reparsed = read_container(&bytes).expect("read_container");
    assert_eq!(reparsed.rootfiles, vec!["hand.mscx".to_string()]);
    assert_eq!(reparsed.mscx_entry().unwrap().path, "hand.mscx");
}
