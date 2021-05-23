extern crate clap;
use clap::{Arg, App};
use std::path::Path;
use std::ffi::OsStr;
#[path = "base/song.rs"] mod base;

fn main() {
    let matches = App::new("Guitar IO")
    .version("1.0")
    .author("slundi <mail>")
    .about("Read guitar file ")
    .arg(Arg::with_name("input_file")
         .short("i")
         .long("input")
         .value_name("input_file")
         .help("Input file path")
         .takes_value(true)
         .required(true))
    .get_matches();
    let file = matches.value_of("input_file").unwrap_or("default.conf");
    let f = Path::new(&file);
    //check if path OK, file exists and is file
    if !f.exists() || !f.is_file() {panic!("Unable to access file: {}", &file);}
    //check file format
    let ext = f.extension().and_then(OsStr::to_str).unwrap_or_else(||{panic!("Cannont get input file extension");}).to_uppercase();
    match ext.as_str() {
        "TG" => println!("Tux guitar file"),
        "GP2" | "GP3" | "GP4" | "GP5" => println!("Guitar pro file"),
        "GPX" => println!("Guitar pro file (new version)"),
        _ => panic!("Unable to process a {} file", ext),
    }
    println!("File extension is: {}", &ext); //TODO: safe processing
    println!("Value for input: {}", file);

}
