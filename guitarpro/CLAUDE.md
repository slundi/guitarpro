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

## MSCZ (MuseScore compressed archive)

`.mscz` is a **plain ZIP archive** wrapping MuseScore's `.mscx` XML score
representation. See the `gp-mscz-format` skill for a deeper reference; the
crate-side surface lives at:

| Path | Role |
|---|---|
| `src/io/mscz/container.rs` | ZIP read/write, `META-INF/container.xml` manifest, size / entry-count guards |
| `src/io/mscz/parse.rs` | quick-xml streaming parser for the MSCX body |
| `src/model/mscz/mod.rs` | `MsczFile { archive, mscx }`, `MsczArchive`, `MsczEntry` |
| `src/model/mscz/mscx.rs` | `Mscx` AST (envelope, meta tags, parts, staves, measures, voices, beats, notes) |
| `src/convert/mscz/to_optimized.rs` | `mscx_to_loaded_score` (MSCX → `LoadedScore`) |
| `src/convert/mscz/from_optimized.rs` | `loaded_score_to_mscx` (regenerates MSCX from `LoadedScore`) |
| `src/convert/mscz/validate.rs` | `LossReport` — counts observed-but-unmapped MSCX features |

**Public entry points** (re-exported from `lib.rs`):
`read_mscz(path)`, `read_mscz_bytes(&[u8])`, `write_mscz(&MsczFile)`,
`write_mscz_to_path(&MsczFile, path)`, plus the `Mscx` / `MsczArchive` /
`MsczEntry` types.

**Design principles** (see also the `gp-mscz-format` skill):
- The raw MSCX XML is preserved on `Mscx::raw_xml` as the source of truth
  for byte-stable round-trips of unmodified archives; the structured fields
  are best-effort extractors used by higher-level converters.
- Only MuseScore 4.x is supported; 3.x/2.x return `MsczUnsupported`.
- Side files (`.mss`, thumbnails, `audiosettings.json`) are kept verbatim
  inside `MsczArchive.entries` so they survive a full container round-trip.
- Tests live in `src/tests/mscz.rs` (container + parser + archive
  round-trip) and `src/tests/mscz_convert.rs` (MSCX ↔ `LoadedScore`).
