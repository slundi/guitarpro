#[path = "song.rs"] pub mod gp;
mod io;
pub mod enums;
pub mod headers;
pub mod track;
pub mod measure;
pub mod effects;
pub mod key_signature;
pub mod midi;
pub mod mix_table;
pub mod chord;
pub mod page;
pub mod rse;
pub mod note;
pub mod lyric;
pub mod beat;

#[cfg(test)]
mod test {
    use std::{io::Read, fs};
    use fraction::ToPrimitive;
    use crate::gp::Song;

    fn read_file(path: String) -> Vec<u8> {
        let f = fs::OpenOptions::new().read(true).open(&path).expect("Cannot open file");
        let size: usize = fs::metadata(&path).unwrap_or_else(|_e|{panic!("Unable to get file size")}).len().to_usize().unwrap();
        let mut data: Vec<u8> = Vec::with_capacity(size);
        f.take(u64::from_ne_bytes(size.to_ne_bytes())).read_to_end(&mut data).unwrap_or_else(|_error|{panic!("Unable to read file contents");});
        data
    }

    //chords
    #[test]
    fn test_gp3_chord() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Chords.gp3")));
    }
    #[test]
    fn test_gp4_chord() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Chords.gp4")));
    }
    #[test]
    fn test_gp5_chord() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Chords.gp5")));
    }
    #[test]
    fn test_gp5_unknown_chord_extension() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Unknown Chord Extension.gp5")));
    }
    #[test]
    fn test_gp5_chord_without_notes() { //Read chord even if there's no fingering
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/chord_without_notes.gp5")));
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/001_Funky_Guy.gp5")));
    }

    //duration
    #[test]
    fn test_gp3_duration() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Duration.gp3")));
    }

    //effects
    #[test]
    fn test_gp3_effects() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Effects.gp3")));
    }
    #[test]
    fn test_gp4_effects() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Effects.gp4")));
    }
    #[test]
    fn test_gp5_effects() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Effects.gp5")));
    }

    //harmonics
    #[test]
    fn test_gp3_harmonics() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Harmonics.gp3")));
    }
    #[test]
    fn test_gp4_harmonics() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Harmonics.gp4")));
    }
    #[test]
    fn test_gp5_harmonics() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Harmonics.gp5")));
    }

    //key
    #[test]
    fn test_gp4_key() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Key.gp4")));
    }
    #[test]
    fn test_gp5_key() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Key.gp5")));
    }

    //demo

    //repeat
    #[test]
    fn test_gp4_repeat() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Repeat.gp4")));
    }
    #[test]
    fn test_gp5_repeat() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Repeat.gp5")));
    }

    //RSE
    #[test]
    fn test_gp5_rse() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/RSE.gp5")));
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Demo v5.gp5")));
    }

    //slides
    #[test]
    fn test_gp4_slides() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Slides.gp4")));
    }
    #[test]
    fn test_gp5_slides() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Slides.gp5")));
    }

    //strokes
    #[test]
    fn test_gp4_strokes() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Strokes.gp4")));
    }
    #[test]
    fn test_gp5_strokes() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Strokes.gp5")));
    }

    //vibrato
    #[test]
    fn test_gp4_vibrato() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Vibrato.gp4")));
    }

    //voices
    #[test]
    fn test_gp5_voices() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Voices.gp5")));
    }

    //wah
    #[test]
    fn test_gp5_no_wah() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/No Wah.gp5")));
    }
    #[test]
    fn test_gp5_wah() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Wah.gp5")));
    }
    #[test]
    fn test_gp5_wah_m() { //Handle gradual wah-wah changes
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Wah-m.gp5")));
    }

    #[test]
    fn test_gp3_writing() {
        let mut song = Song::default();
        song.read_gp3(&read_file(String::from("test/Chords.gp3")));
        let out = song.write((3,0,0), None);
        song.read_gp3(&out);
    }
}
