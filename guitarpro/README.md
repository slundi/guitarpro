# guitarpro

A safe, modular, and high-performance Rust library for parsing and writing Guitar Pro files.

[![Crates.io](https://img.shields.io/crates/v/guitarpro.svg)](https://crates.io/crates/guitarpro)
[![Documentation](https://docs.rs/guitarpro/badge.svg)](https://docs.rs/guitarpro)

## Features

- **Multi-Format Support**: 
    - **GP3, GP4, GP5**: High-fidelity reading and writing.
    - **GP7 (.gp)**: Experimental reading support via GPIF XML.
    - **MuseScore (.mscz)**: Basic XML/ZIP parsing.
- **Safety First**: Written in 100% safe Rust to handle untrusted binary files securely.
- **Modular Architecture**: Separated into `model`, `io` (binary primitives), and `audio` (MIDI/GM).
- **Extensible**: Uses a trait-based system to keep the core `Song` model clean while providing rich format-specific functionality.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
guitarpro = "0.1.0"
```

## Quick start

```rust
use guitarpro::model::song::Song;
use guitarpro::model::track::SongTrackOps;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read("my_awesome_song.gp5")?;
    
    let mut song = Song::default();
    // Traits provide the reading capabilities
    song.read_gp5(&data);
    
    println!("Song: {}", song.name);
    for track in &song.tracks {
        println!("Track: {} ({} measures)", track.name, track.measures.len());
    }
    
    Ok(())
}
```
