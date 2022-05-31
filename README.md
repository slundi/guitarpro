# Guitarpro

A Rust safe library to parse and write guitar pro files.

[![Tests](https://github.com/slundi/guitarpro/actions/workflows/rust.yml/badge.svg)](https://github.com/slundi/guitarpro/actions/workflows/rust.yml) [![rust-clippy](https://github.com/slundi/guitarpro/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/slundi/guitarpro/actions/workflows/rust-clippy.yml)

It is based on [Perlence/PyGuitarPro](https://github.com/Perlence/PyGuitarPro), [TuxGuitar](http://tuxguitar.com.ar/) and [MuseScore](https://musescore.org) sources.

## usage

```rust
use guitarpro;

fn main() {
    let f = fs::OpenOptions::new().read(true).open("my_awesome_song.gp5").unwrap_or_else(|_error| {
        panic!("Unknown error while opening my_awesome_song.gp5");
    });
    let mut data: Vec<u8> = Vec::with_capacity(size);
    f.take(u64::from_ne_bytes(size.to_ne_bytes())).read_to_end(&mut data).unwrap_or_else(|_error|{panic!("Unable to read file contents");});
    let mut song: guitarpro::Song = gp::Song::default();
    match ext.as_str() {
        "GP3" => song.read_gp3(&data),
        "GP4" => song.read_gp4(&data),
        "GP5" => song.read_gp5(&data),
        "GPX" => println!("Guitar pro file (new version) is not supported yet"), //new Guitar Pro files
        _ => panic!("Unable to process a {} file (GP1 and GP2 files are not supported)", ext),
    }
}
```

## Roadmap

* [ ] Documentation
* [x] Read GP3 files
* [x] Read GP4 files
* [ ] Read GP5 files: almost working, Coda and similar directions are not working, use `test/Demo v5.gp5` to test/fix/pull request/...
* [ ] Read GPX files (version 6)
* [ ] Read GPX files (version 7)
* [ ] Write GP3 files
* [ ] Write GP4 files
* [ ] Write GP5 files
* [ ] Write GPX files (version 6)
* [ ] Write GPX files (version 7)

## About me

I started to play guitar in 2019 and I have a pack of Guitar Pro files and sometimes I get a new files from [Songsterr](https://www.songsterr.com/). I want to write a better documentation but I don't know all the stuffs that I can see on a score ;)

In order to make another software (with advanced search, chord detection, ...), I need a library that is able to parse and write Guitar Pro files.
