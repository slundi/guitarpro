use super::common::read_file;
use crate::model::legacy::song::Song;

#[test]
fn test_gp3_chord() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Chords.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_chord() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Chords.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_chord() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Chords.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_unknown_chord_extension() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Unknown Chord Extension.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_chord_without_notes() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/chord_without_notes.gp5")))
        .unwrap();
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/001_Funky_Guy.gp5")))
        .unwrap();
}
