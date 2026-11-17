use crate::model::legacy::song::Song;
use fraction::ToPrimitive;
use std::{fs, io::Read};

pub fn read_file(path: String) -> Vec<u8> {
    let test_path = if path.starts_with("test/") {
        format!("../{}", path)
    } else {
        format!("../test/{}", path)
    };
    let f = fs::OpenOptions::new()
        .read(true)
        .open(&test_path)
        .unwrap_or_else(|e| panic!("Cannot open file '{}': {}", test_path, e));
    let size: usize = fs::metadata(&test_path)
        .unwrap_or_else(|e| panic!("Unable to get file size for '{}': {}", test_path, e))
        .len()
        .to_usize()
        .unwrap();
    let mut data: Vec<u8> = Vec::with_capacity(size);
    f.take(size as u64)
        .read_to_end(&mut data)
        .unwrap_or_else(|e| panic!("Unable to read file contents from '{}': {}", test_path, e));
    data
}

pub fn read_gpx(filename: &str) -> Song {
    let mut song = Song::default();
    song.read_gpx(&read_file(String::from(filename))).unwrap();
    song
}

pub fn read_gp7(filename: &str) -> Song {
    let mut song = Song::default();
    song.read_gp(&read_file(String::from(filename))).unwrap();
    song
}

/// A MusicXML (partwise) fixture from `test/`, without its DTD declaration,
/// which quick-xml can't parse.
/// A test's result: any error fails it.
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn read_musicxml(filename: &str) -> TestResult<crate::model::musicxml::ScorePartwise> {
    let raw = String::from_utf8(read_file(String::from(filename)))?;
    let raw = match raw.find("<!DOCTYPE") {
        Some(start) => match raw[start..].find('>') {
            Some(end) => format!("{}{}", &raw[..start], &raw[start + end + 1..]),
            None => raw,
        },
        None => raw,
    };
    Ok(quick_xml::de::from_str(&raw)?)
}

/// `score` saved as before saves had a format version: loading it runs the
/// migrations of older forms.
pub fn unversioned_save(
    score: &crate::model::optimized::global::Score,
) -> serde_json::Result<String> {
    let mut value = serde_json::to_value(score)?;
    if let Some(object) = value.as_object_mut() {
        object.remove("format_version");
    }
    serde_json::to_string(&value)
}
