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
