use super::common::read_file;
use crate::model::legacy::song::Song;

#[test]
fn test_gp3_dotted_gliss() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/dotted-gliss.gp3")))
        .unwrap();
}

#[test]
fn test_gp4_slides() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Slides.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slides() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Slides.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_legato_slide() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/legato-slide.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_legato_slide() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/legato-slide.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_shift_slide() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/shift-slide.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_shift_slide() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/shift-slide.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_slide_in_above() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-in-above.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_in_above() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-in-above.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slide_in_below() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-in-below.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_in_below() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-in-below.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slide_out_down() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-out-down.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_out_down() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-out-down.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slide_out_up() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-out-up.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_out_up() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-out-up.gp5")))
        .unwrap();
}
