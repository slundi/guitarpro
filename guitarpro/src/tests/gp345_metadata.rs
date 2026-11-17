use super::common::read_file;
use crate::model::legacy::song::Song;

#[test]
fn test_gp3_copyright() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/copyright.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_copyright() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/copyright.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_copyright() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/copyright.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_tempo() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/tempo.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_tempo() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/tempo.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_tempo() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/tempo.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_capo_fret() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/capo-fret.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_capo_fret() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/capo-fret.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_capo_fret() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/capo-fret.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_dynamic() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/dynamic.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_all_percussion() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/all-percussion.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_beams_stems_ledger_lines() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from(
        "test/beams-stems-ledger-lines.gp5",
    )))
    .unwrap();
}

#[test]
fn test_gp4_fingering() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/fingering.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_fingering() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/fingering.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_fret_diagram() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/fret-diagram.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_fret_diagram() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/fret-diagram.gp5")))
        .unwrap();
}

#[test]
fn guitar_pros_dynamic_levels_are_the_models_markings() {
    use crate::convert::legacy::dynamics::velocity_to_dynamic;
    use crate::model::legacy::effects::{DYNAMIC_LEVELS, level_velocity};
    use crate::model::optimized::beat::Dynamic;

    for level in 0..DYNAMIC_LEVELS.len() {
        let marking = velocity_to_dynamic(level_velocity(level));
        assert!(!matches!(marking, Dynamic::Other(_)), "{marking:?}");
    }
}

/// The optimized model reads a velocity as the marking GPIF writes for it.
#[test]
fn a_velocity_reads_as_the_marking_gpif_writes() {
    use crate::convert::legacy::dynamics::velocity_to_dynamic;
    use crate::model::legacy::effects::nearest_dynamic_name;
    use crate::model::optimized::beat::Dynamic;

    for velocity in [i16::MIN, 0, 15, 23, 24, 87, 95, 96, 103, 104, 127, i16::MAX] {
        assert_eq!(
            velocity_to_dynamic(velocity),
            Dynamic::from_name(&nearest_dynamic_name(velocity).to_ascii_lowercase()),
            "{velocity}"
        );
    }
}

/// GPIF writes the dynamic level nearest a note's velocity, the quieter
/// one between two.
#[test]
fn gpif_writes_the_nearest_dynamic_level() {
    use crate::model::legacy::effects::{level_velocity, nearest_dynamic_name};

    let mf = level_velocity(4);
    let increment = level_velocity(5) - mf;
    assert_eq!(nearest_dynamic_name(mf), "MF");
    assert_eq!(nearest_dynamic_name(mf + 1), "MF");
    assert_eq!(nearest_dynamic_name(mf + increment / 2), "MF");
    assert_eq!(nearest_dynamic_name(mf + increment / 2 + 1), "F");
    assert_eq!(nearest_dynamic_name(i16::MIN), "PPP");
    assert_eq!(nearest_dynamic_name(i16::MAX), "FFF");
}
