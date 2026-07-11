# guitarpro

A safe, modular, and high-performance Rust library for parsing and writing Guitar Pro files.

[![Crates.io](https://img.shields.io/crates/v/guitarpro.svg)](https://crates.io/crates/guitarpro)
[![Documentation](https://docs.rs/guitarpro/badge.svg)](https://docs.rs/guitarpro)

## Features

- **Multi-Format Support**:
    - **GP3, GP4, GP5**: High-fidelity reading and writing.
    - **GP6 (.gpx)** and **GP7+ (.gp)**: read and write via GPIF XML.
    - **MuseScore 4.x (.mscz)**: read and write; MSCX ↔ `LoadedScore`
      conversion with `LossReport`. See [Roadmap-mscz](../docs/Roadmap-mscz.md).
    - **MusicXML** (`.xml` / `.musicxml`): score-partwise and timewise.
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

### Reading a MuseScore file (`.mscz`)

```rust
use guitarpro::convert::mscz::mscx_to_loaded_score;
use guitarpro::read_mscz;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = read_mscz("my_awesome_song.mscz")?;
    println!("MuseScore {}", file.mscx.version);
    println!("Title: {}", file.mscx.meta("workTitle").unwrap_or("(untitled)"));

    // Structured conversion to the optimized model
    let outcome = mscx_to_loaded_score(&file.mscx);
    println!(
        "Tracks: {}, timeline entries: {}",
        outcome.score.score.tracks.len(),
        outcome.score.score.timeline.len(),
    );
    if !outcome.report.is_empty() {
        eprintln!("Loss report: {} feature(s) not preserved", outcome.report.distinct());
    }
    Ok(())
}
```
