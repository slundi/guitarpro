use super::common::read_file;
use crate::model::song::Song;

#[test]
fn test_gp4_strokes() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Strokes.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_strokes() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Strokes.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_no_wah() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/No Wah.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_wah() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Wah.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_wah_m() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Wah-m.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_brush() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/brush.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_fade_in() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/fade-in.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_fade_in() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/fade-in.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_let_ring() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/let-ring.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_let_ring() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/let-ring.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_palm_mute() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/palm-mute.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_palm_mute() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/palm-mute.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_pick_up_down() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/pick-up-down.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_pick_up_down() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/pick-up-down.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_sforzato() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/sforzato.gp4")))
        .unwrap();
}

#[test]
fn test_gp4_slur() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slur.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slur_notes_effect_mask() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slur-notes-effect-mask.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_tap_slap_pop() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/tap-slap-pop.gp5")))
        .unwrap();
}
