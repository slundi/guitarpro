# Guitar Pro Parser Documentation

## 1. Introduction

`guitarproparser` is a Rust library (`scorelib`) and CLI tool (`score_tool`) for reading and writing Guitar Pro files (`.gp3`, `.gp4`, `.gp5`). It provides a comprehensive data structure to represent musical partitutres (tablature and notation).

This documentation is designed to be exhaustive for both human developers and AI coding assistants needing to understand the codebase.

## 2. Quick Start

### CLI Tool (`score_tool`)

A CLI is provided to inspect files and generate ASCII tablatures.

**Usage:**
```bash
cargo run -p cli -- --input path/to/file.gp5 --tab
```

**Options:**
- `--input <FILE>`: Path to the Guitar Pro file.
- `--tab` (or `-t`): Visualize the first track as ASCII tablature.

### Library (`scorelib`)

**Dependency:**
Add the library to your `Cargo.toml`:
```toml
[dependencies]
lib = { path = "path/to/guitarproparser/lib" } # Rename 'lib' to 'scorelib' recommended
```

**Basic Usage:**
```rust
use scorelib::gp::Song;
use std::fs;
use fraction::ToPrimitive; // crate 'fraction' is used for durations

fn main() {
    let mut song = Song::default();
    let data = fs::read("clementi.gp5").expect("File not found");
    
    // Auto-detect format via extension usually, but here calling specific reader:
    song.read_gp5(&data);
    
    println!("Song: {}", song.name);
    
    // Iterate tracks
    for track in &song.tracks {
        println!("Track: {} ({} strings)", track.name, track.strings.len());
        // Iterate measures
        for measure in &track.measures {
             // ...
        }
    }
}
```

### Traits and Extensions

The library uses traits to extend `Song` with parsing and writing capabilities. This allows the core `Song` struct to remain clean while providing a large API for different formats and features.

```rust
use scorelib::model::song::Song;
use scorelib::model::track::SongTrackOps;
// ... other trait imports ...

// Now song has .read_tracks(), .write_tracks(), etc.
```

## 3. Architecture & Data Structures

The data model follows a hierarchical structure typical of musical scores.

### Hierarchy
`Song` -> `Track` -> `Measure` -> `Voice` -> `Beat` -> `Note`

### key Structures

#### `Song` (`lib/src/model/song.rs`)
The root object representing the entire file.
- **Metadata**: `name`, `artist`, `album`, `author`, `copyright`, `writer`, `transcriber`, `comments`.
- **Global Properties**: `tempo`, `key`, `version` (GP version tuple).
- **Content**: `tracks` (`Vec<Track>`), `measure_headers` (`Vec<MeasureHeader>`).
- **Channels**: `channels` (`Vec<MidiChannel>`) - MIDI instrument configuration.

#### `Track` (`lib/src/model/track.rs`)
Represents a single instrument (e.g., "Electric Guitar").
- **Identity**: `name`, `color`, `channel_index`.
- **Instrument**: `strings` (`Vec<(i8, i8)>` - string number & midi tuning), `fret_count`, `capo`.
- **Content**: `measures` (`Vec<Measure>`).
- **Settings**: `TrackSettings` (tablature/notation visibility, etc.).

#### `Measure` (`lib/src/model/measure.rs`)
Represents a bar of music for a specific track.
*Note: Global measure info (time signature, key signature, repeat bars) is stored in `Song.measure_headers`.*
- **Structure**: `voices` (`Vec<Voice>`) - usually contains 1 or 2 voices.
- **Properties**: `clef`, `line_break`.
- **Position**: `start` (accumulated time).

#### `Voice` (`lib/src/model/beat.rs`)
A rhythmic container within a measure. GP5 supports up to 2 voices (e.g., Lead + Bass in one staff).
- **Content**: `beats` (`Vec<Beat>`).

#### `Beat` (`lib/src/model/beat.rs`)
A rhythmic unit containing notes.
- **Rhythm**: `duration` (`Duration` struct), `tuplets`.
- **Content**: `notes` (`Vec<Note>`), `text` (lyrics/text above), `effect` (`BeatEffects` - e.g., mix table changes, strokes).
- **Properties**: `status` (Normal, Rest, Empty).

#### `Note` (`lib/src/model/note.rs`)
A single sound event.
- **Pitch**: `value` (fret number 0-99), `string` (string index 1-N).
- **Dynamics**: `velocity` (MIDI velocity).
- **Effects**: `NoteEffect` (bend, slide, hammer, harmonic, vibrato, grace note, etc.).
- **Type**: `kind` (Normal, Tie, Dead, Rest).

## 4. Effects System

Effects are categorized by where they apply:

- **Note Effects** (`lib/src/model/note.rs` -> `NoteEffect`):
  - `bend`: `BendEffect` (points, type).
  - `grace`: `GraceEffect` (fret, duration, transition).
  - `slides`: `Vec<SlideType>`.
  - `harmonic`: `HarmonicEffect` (Natural, Artificial, Tapped, Pinch, Semi).
  - `hammer`/`pull_off`, `palm_mute`, `staccato`, `let_ring`, `vibrato`, `trill`, `tremolo_picking`.

- **Beat Effects** (`lib/src/model/beat.rs` -> `BeatEffects`):
  - `stroke`: Up/Down strums.
  - `mix_table_change`: Tempo, Volume, Pan, Instrument automation changes.
  - `pick_stroke`.

- **Track Effects**: RSE (Realistic Sound Engine) settings, EQ, Humanize.

## 5. Low-Level I/O (`lib/src/io.rs`)

The `io` module provides primitives for reading binary data types used in GP formats:
- `read_byte`, `read_signed_byte`, `read_bool`.
- `read_int` (4 bytes), `read_short` (2 bytes).
- `read_chunk_string`: Pascal-style strings.
- `read_string_byte`: String prefixed by byte length.
- `read_string_int`: String prefixed by int length.

The parsing is sequential. Functions take a `data: &[u8]` slice and a mutable `seek: &mut usize` cursor.

## 6. Supported Formats

| Feature | GP3 (`.gp3`) | GP4 (`.gp4`) | GP5 (`.gp5`) | GP6/GP7 (`.gpx`/`.gp`) |
|---------|--------------|--------------|--------------|-----------------------|
| **Read** | ✅ Full | ✅ Full | ✅ High | ✅ Initial (experimental) |
| **Write** | ⚠️ Partial | ⚠️ Partial | ⚠️ Partial | ❌ Not Implemented |

**Known Limitations in GP5:**
- Complex "Direction" symbols (Segno, Coda) on advanced files may parsing issues.
- RSE (Realistic Sound Engine) data is largely skipped or partially read.

## 7. Development Guide

### Running Tests
The repository contains a suite of integration tests in `lib/src/lib.rs` (module `test`).
```bash
cd lib
cargo test
```
*Note: `test_gp5_demo_complex` and `test_gp3_writing` are currently ignored due to known limitations.*

### Error Handling (Current State)
The parser currently uses `panic!` for errors (e.g., file too short, invalid format).
*Future Plan:* Migrate to `Result<T, ParseError>` for robust error handling.

### Adding Support for New Version
To add parsing for a new version:
1. Define reader methods in `Song` (e.g., `read_gp6`).
2. Implement version-specific logic in `read_track`, `read_measure`, etc., usually switched by `self.version.number`.

## 8. For AI Agents (Context)

When navigating this codebase:
1.  **Entry Point**: `lib/src/lib.rs` exports modules. `lib/src/song.rs` contains the `read_gpX` entry methods.
2.  **Logic Flow**: `read_gpX` -> `read_version` -> `read_info` -> `read_tracks` -> `read_measures`.
3.  **Data Flow**: Measures are stored per track. `song.tracks[t].measures[m]` corresponds to `song.measure_headers[m]`. The header contains timing info (time sig, key sig) shared across all tracks for that measure index.
4.  **One-indexing**: GP formats often use 1-based indexing for user-facing values (strings, tracks). Internally, Vectors are 0-indexed. Be careful with loop bounds.
    - Note strings: `note.string` is often 1-based in GP data.
    - Fret values: 0 is open string, >0 is fret.

### Common Tasks Patterns
- **Iterating Notes**:
  ```rust
  for track in &song.tracks {
      for measure in &track.measures {
          for voice in &measure.voices {
              for beat in &voice.beats {
                  for note in &beat.notes {
                      // Process note
                  }
              }
          }
      }
  }
  ```
