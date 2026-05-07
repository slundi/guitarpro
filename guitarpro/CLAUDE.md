# Library Guide: guitarpro

Dedicated instructions for the core Guitar Pro parsing engine.

## Data Model Hierarchy
`Song → Track → Measure → Voice → Beat → Note`

- **Song**: Tracks, `MeasureHeader` (shared metadata), MIDI, lyrics, page setup.
- **Track**: Contains a `Vec<Measure>`.
- **Voice**: 1–2 per measure; contains `Beat`s.
- **Effects**: `NoteEffect` (bend, slide, etc.) vs. `BeatEffects` (stroke, chord, etc.).

## Parsing Architecture
- **Trait-based API**: Logic is split into traits (e.g., `SongTrackOps`, `SongNoteOps`) implemented on `Song`.
- **Binary Pattern**: Uses `(data: &[u8], seek: &mut usize)` cursor pattern. 
- **IO Primitives**: Use `io::primitive` for all low-level reads (e.g., `read_byte`, `read_short`).
- **Version Branching**: Branch logic on `Song.version.number: (u8, u8, u8)`.
- **Modern Formats**: `.gp` files use ZIP extraction → XML deserialization (`Gpif`) → `Song` conversion.

## Coding Standards
- Tests live in `src/tests.rs` and use files from the `test/` directory.
- Refer to `FILE-STRUCTURE*.md` for binary format specifications.
