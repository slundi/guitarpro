use super::common::read_file;
use crate::model::song::Song;

#[test]
fn test_gp3_duration() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Duration.gp3")))
        .unwrap();
}

#[test]
fn test_gp4_key() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Key.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_key() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Key.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_keysig() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/keysig.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_keysig() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/keysig.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_repeat() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Repeat.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_repeat() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Repeat.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_voices() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Voices.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_dotted_tuplets() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/dotted-tuplets.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_test_irr_tuplet() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/testIrrTuplet.gp4")))
        .unwrap();
}
#[test]
fn test_gp4_tuplet_with_slur() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/tuplet-with-slur.gp4")))
        .unwrap();
}

#[test]
fn test_gp4_rest_centered() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/rest-centered.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_rest_centered() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/rest-centered.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_rse() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/RSE.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_volta() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/volta.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_volta() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/volta.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_volta() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/volta.gp5")))
        .unwrap();
}
