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

### Library

* [ ] Documentation
* [x] Read GP3 files
* [x] Read GP4 files
* [ ] Read GP5 files: almost working, Coda and similar directions are not working, use `test/Demo v5.gp5` to test/fix/pull request/...
* [ ] Read GPX files (version 6)
* [ ] Read GPX files (version 7)
* [ ] Read MuseScore files (ZIP + XML)
* [ ] Write GP3 files
* [ ] Write GP4 files
* [ ] Write GP5 files
* [ ] Write GPX files (version 6)
* [ ] Write GPX files (version 7)
* [ ] Write MuseScore files

### CLI

* [ ] `-l` Load bellow parameters from a file (YAML?, JSON?, other?)
* [ ] `-f` Find in files
  * [ ] song information (artist, title, album, ...) with wildcard and regexes
  * [ ] instrument
  * [ ] `-ft <value>` [tuning](https://en.wikipedia.org/wiki/List_of_guitar_tunings) (value example: `E-A-D-g-b-e`, `EADgbe`, ` E2–A2–D3–G3–B3–E4`, )
  * [ ] range (string count, piano keys count, drum elements, ...)
  * [ ] `-f? <value|range>` tempo: is constant, range if variable, min, max, ... (example: `80`, `60-90`, `60>`, `<120` or `0-120`, `60,80,90` or `80,70-85` for a list)
  * [ ] `-fb <beat>`beats
  * [ ] `-fn <notes>` notes (example: `DADDC` anywhere in the track, `|DADDC|` in a mesure, `|A|E|CC|` measures with those notes)
  * [ ] `-fr` Repetitions:
    * [ ] `-frm <1|2|4>` [same mesures](https://musescore.org/en/handbook/4/measure-and-multi-measure-repeats)
    * [ ] `-frs` [repeat signs](https://musescore.org/en/handbook/4/repeat-signs)
    * [ ] `-frv` [voltas](https://musescore.org/en/handbook/4/voltas)
  * [ ] `-fv 0.8` detect verse with a similarity percentage
* [ ] `-x` Extract:
  * [ ] `-xi <format>` above information in various format (CSV, JSON)
  * [ ] `-xt <number|instrument> <format>` tracks
  * [ ] `-xl <format>` lyrics
* [ ] `-c format` Conversion between formats with alerts when information are lost (like GP5 -> GP3)
* [ ] `-r` Replace repetitions 
  * [ ] `m` [same mesures](https://musescore.org/en/handbook/4/measure-and-multi-measure-repeats)
  * [ ] `s` repeat signs and `v` voltas when mesures are the same for all tracks
* [ ] `-t -0.5` Change tuning if possible
* [ ] `-ts x2` Divide/multiply time signatures (I had once an guitar tab that needed to be rewritten by changing the time signature and the beams)
* [ ] `-p` Apply page format parametters (margin, spacing, ...)

## About me

I started to play guitar in 2019 and I have a pack of Guitar Pro files and sometimes I get a new files from [Songsterr](https://www.songsterr.com/). I want to write a better documentation but I don't know all the stuffs that I can see on a score ;)

In order to make another software (with advanced search, chord detection, ...), I need a library that is able to parse and write Guitar Pro files.
