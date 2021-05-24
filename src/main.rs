extern crate clap;
use clap::{Arg, App};
use std::path::Path;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
#[path = "base/song.rs"] mod base;
#[path = "io/tuxguitar.rs"] mod tg;

const GUITAR_FILE_MAX_SIZE:usize = 16777216; //16 MB, it should be enough

fn main() {
    let matches = App::new("Guitar IO").version("1.0").author("slundi <mail>").about("Read guitar file ")
    .arg(
         Arg::with_name("input_file").takes_value(true).required(true).short("i").long("input").value_name("input_file").help("Input file path")
    ).get_matches();
    let file = matches.value_of("input_file").unwrap_or("default.conf");
    let f = Path::new(&file);
    //check if path OK, file exists and is file
    if !f.exists() || !f.is_file() {panic!("Unable to access file: {}", &file);}
    //check file format
    let ext = f.extension().and_then(OsStr::to_str).unwrap_or_else(||{panic!("Cannont get input file extension");}).to_uppercase();
    let size: usize = fs::metadata(file).unwrap_or_else(|e|{panic!("Unable to get file size")}).len() as usize;
    if size > GUITAR_FILE_MAX_SIZE {panic!("File is too big (bigger than 16 MB)");}
    let f = fs::OpenOptions::new().read(true).open(file).unwrap_or_else(|_error| {
        /*if error.kind() == fs::ErrorKind::NotFound {panic!("File {} was not found", &file);}
        else if error.kind() == fs::ErrorKind::PermissionDenieds {panic!("File {} is unreadable, check permissions", &file);}
        else {panic!("Unknown error while opening {}", &file);}*/
        panic!("Unknown error while opening {}", &file);
    });
    let mut data = vec![0u8; size];
    f.take(size as u64).read_to_end(&mut data).unwrap_or_else(|_error|{panic!("Unable to read file contents");});
    match ext.as_str() {
        "TG" => println!("Tux guitar file"),
        "GP2" | "GP3" | "GP4" | "GP5" => println!("Guitar pro file"),
        "GPX" => println!("Guitar pro file (new version)"),
        _ => panic!("Unable to process a {} file", ext),
    }
    
}
