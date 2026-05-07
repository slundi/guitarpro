use super::common::read_file;
use crate::model::song::Song;

#[test]
fn test_gp3_effects() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Effects.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_effects() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Effects.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_effects() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Effects.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Harmonics.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Harmonics.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Harmonics.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_vibrato() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Vibrato.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_vibrato() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/vibrato.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_basic_bend() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/basic-bend.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_ghost_note() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/ghost_note.gp3")))
        .unwrap();
}
#[test]
fn test_gp5_grace() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/grace.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_heavy_accent() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/heavy-accent.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_high_pitch() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/high-pitch.gp3")))
        .unwrap();
}

#[test]
fn test_gp5_tremolos() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/tremolos.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_trill() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/trill.gp4")))
        .unwrap();
}
