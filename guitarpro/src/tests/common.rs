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
