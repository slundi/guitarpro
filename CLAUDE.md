# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                          # Build entire workspace
cargo build -p lib                   # Build core library only
cargo build -p cli                   # Build CLI tool
cargo test                           # Run all tests
cargo test -- --nocapture            # Run tests with stdout visible
cargo test test_gp5                  # Run tests matching pattern
cargo clippy                         # Lint
cargo run -p cli -- --input file.gp5 --tab   # Run CLI with tablature output
```

Tests live in `lib/src/tests.rs` and parse files from the `test/` directory.

## Architecture

**Workspace crates:**
- `lib` (library name: `scorelib`) — Core parsing library for Guitar Pro files
- `cli` (binary: `score_tool`) — CLI for file inspection and ASCII tablature
- `web_server` (binary: `score_server`) — Experimental web server

**Data model hierarchy:** `Song → Track → Measure → Voice → Beat → Note`

- `Song` holds tracks, measure headers (shared metadata across tracks), MIDI channels, lyrics, and page setup
- Each `Track` has its own `Vec<Measure>`, while `MeasureHeader` metadata is shared at the song level
- `Voice` (1-2 per measure) contains `Beat`s; each `Beat` contains `Note`s
- Effects are split: `NoteEffect` (bend, slide, harmonic, grace) on notes, `BeatEffects` (stroke, chord, mix table) on beats

**Module layout (`lib/src/`):**
- `model/` — All data structures (Song, Track, Measure, Beat, Note, effects, enums, chord, etc.)
- `io/` — Binary I/O primitives (`primitive.rs`), GPIF XML structures (`gpif.rs`), ZIP handling (`gpx.rs`)
- `audio/` — MIDI channel definitions and GM instrument names

**Trait-based parsing API:** Functionality is organized into ~12 traits (e.g., `SongTrackOps`, `SongMeasureOps`, `SongNoteOps`, `SongEffectOps`, `SongChordOps`) implemented on `Song`. Each trait provides `read_*` and `write_*` methods for its domain. All traits are re-exported from `lib.rs`.

**Binary parsing pattern:** All GP3/4/5 parsing uses a `(data: &[u8], seek: &mut usize)` cursor pattern. Low-level reads go through `io::primitive` functions (`read_byte`, `read_int`, `read_short`, `read_int_byte_size_string`, etc.).

**Version branching:** `Song.version.number` is a `(u8, u8, u8)` tuple. Format-specific logic branches on this throughout the parsing code (e.g., GP5 has separate author field, different padding bytes).

**GP7+ (.gp) parsing** uses a different path: ZIP extraction → XML deserialization via `serde`/`quick-xml` into `Gpif` structs → conversion to `Song` model.

## Format Support

- GP3, GP4, GP5: Full read support, partial write support
- GP7 (.gp): Initial read support via GPIF XML
- GP6 (.gpx): Not yet implemented
- Error handling currently uses `panic!()` — no `Result` types yet

## File Structure Documentation

Binary format specs are in `lib/FILE-STRUCTURE*.md` files — useful when modifying parsing logic.
