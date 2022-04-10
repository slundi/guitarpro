#[macro_use]
extern crate lazy_static;
extern crate clap;
use clap::Parser;
use fraction::ToPrimitive;
use std::path::Path;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
#[path = "song.rs"] mod gp;
mod io;
mod enums;
mod headers;
mod track;
mod measure;
mod effects;
mod key_signature;
mod midi;
mod mix_table;
mod chord;
mod page;
mod rse;
mod note;
mod lyric;
mod beat;

const GUITAR_FILE_MAX_SIZE:usize = 16777216; //16 MB, it should be enough

#[derive(Parser, Debug)]
#[clap(author="slundi", version, about="Read guitar pro files", long_about = None)]
struct Args {
    /// Input file
    #[clap(short='i', long, help="Input file path")] input: String,
}

fn main() {
    let args: Args = Args::parse();
    let f = Path::new(&args.input);
    //check if path OK, file exists and is file
    if !f.exists() || !f.is_file() {panic!("Unable to access file: {}", &args.input);}
    //check file format
    let ext = f.extension().and_then(OsStr::to_str).unwrap_or_else(||{panic!("Cannont get input file extension");}).to_uppercase();
    let size: usize = fs::metadata(&args.input).unwrap_or_else(|_e|{panic!("Unable to get file size")}).len().to_usize().unwrap();
    if size > GUITAR_FILE_MAX_SIZE {panic!("File is too big (bigger than 16 MB)");}
    let f = fs::OpenOptions::new().read(true).open(&args.input).unwrap_or_else(|_error| {
        /*if error.kind() == fs::ErrorKind::NotFound {panic!("File {} was not found", &file);}
        else if error.kind() == fs::ErrorKind::PermissionDenieds {panic!("File {} is unreadable, check permissions", &file);}
        else {panic!("Unknown error while opening {}", &file);}*/
        panic!("Unknown error while opening {}", args.input);
    });
    let mut data: Vec<u8> = Vec::with_capacity(size);
    f.take(u64::from_ne_bytes(size.to_ne_bytes())).read_to_end(&mut data).unwrap_or_else(|_error|{panic!("Unable to read file contents");});
    let mut song: gp::Song = gp::Song::default();
    match ext.as_str() {
        "GP3" | "GP4" | "GP5" => {
            println!("Guitar pro file"); //old Guitar Pro files
            song.read_data(&data);
            println!("Artist: \"{}\"", song.artist);
            println!("Title:  \"{}\"", song.name);
            println!("Album:  \"{}\"", song.album);
            println!("Author: \"{}\"", song.author);
            println!("Date:   \"{}\"", song.date);
            println!("Copyright:   \"{}\"", song.copyright);
            println!("Writer:      \"{}\"", song.writer);
            println!("Transcriber: \"{}\"", song.transcriber);
            println!("Comments:    \"{}\"", song.comments);
            }
        "GPX" => println!("Guitar pro file (new version) is not supported yet"), //new Guitar Pro files
        _ => panic!("Unable to process a {} file (GP1 and GP2 files are not supported)", ext),
    }
}

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
        return data;
    }

    //chords
    #[test]
    fn test_gp3_chord() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Chords.gp3")));
    }
    #[test]
    fn test_gp4_chord() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Chords.gp4")));
    }
    //#[test]
    fn test_gp5_chord() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Chords.gp5")));
    }
    //#[test]
    fn test_gp5_unknown_chord_extension() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Unknown Chords Extension.gp5")));
    }
    //#[test]
    fn test_gp5_chord_without_notes() { //Read chord even if there's no fingering
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/chord_without_notes.gp5")));
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/001_Funky_Guy.gp5")));
    }

    //duration
    #[test]
    fn test_gp3_duration() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Duration.gp3")));
    }

    //effects
    #[test]
    fn test_gp3_effects() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Effects.gp3")));
    }
    #[test]
    fn test_gp4_effects() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Effects.gp4")));
    }
    //#[test]
    fn test_gp5_effects() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Effects.gp5")));
    }

    //harmonics
    #[test]
    fn test_gp3_harmonics() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Harmonics.gp3")));
    }
    #[test]
    fn test_gp4_harmonics() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Harmonics.gp4")));
    }
    //#[test]
    fn test_gp5_harmonics() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Harmonics.gp5")));
    }

    //key
    #[test]
    fn test_gp4_key() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Key.gp4")));
    }
    //#[test]
    fn test_gp5_key() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Key.gp5")));
    }

    //demo

    //repeat
    #[test]
    fn test_gp4_repeat() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Repeat.gp4")));
    }
    //#[test]
    fn test_gp5_repeat() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Repeat.gp5")));
    }

    //RSE
    #[test]
    //#[test]
    fn test_gp5_rse() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/RSE.gp5")));
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Demo v5.gp5")));
    }

    //slides
    #[test]
    fn test_gp4_slides() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Slides.gp4")));
    }
    //#[test]
    fn test_gp5_slides() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Slides.gp5")));
    }

    //strokes
    #[test]
    fn test_gp4_strokes() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Strokes.gp4")));
    }
    //#[test]
    fn test_gp5_strokes() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Strokes.gp5")));
    }

    //vibrato
    #[test]
    fn test_gp4_vibrato() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Vibrato.gp4")));
    }
    //#[test]
    fn test_gp5_vibrato() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Vibrato.gp5")));
    }

    //voices
    //#[test]
    fn test_gp5_voices() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Voices.gp5")));
    }

    //wah
    //#[test]
    fn test_gp5_no_wah() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/No Wah.gp5")));
    }
    //#[test]
    fn test_gp5_wah() {
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Wah.gp5")));
    }
    //#[test]
    fn test_gp5_wah_m() { //Handle gradual wah-wah changes
        let mut song: Song = Song::default();
        song.read_data(&read_file(String::from("test/Wah-m.gp5")));
    }
}
