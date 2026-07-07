---
name: gp-legacy-format
description: How the guitarpro crate reads/writes the LEGACY binary Guitar Pro formats — .gp3, .gp4, .gp5 (and clipboard variants). Use when adding features, fixing parse/round-trip bugs, or handling version differences in the binary formats. NOT for modern .gp/.gpx (see gp-modern-format) or MusicXML (see musicxml-conversion).
---

# Legacy Guitar Pro binary format (GP3 / GP4 / GP5)

The `.gp3`/`.gp4`/`.gp5` files are a **sequential binary format** with a version
string header, then addressless sequential data. The byte-level layout is
documented in the crate's spec docs — **read those first** for the wire format:

- `guitarpro/FILE-STRUCTURE.md` — headers, MIDI channels, general layout
- `guitarpro/FILE-STRUCTURE-BODY.md` — measures, tracks, beats
- `guitarpro/FILE-STRUCTURE-NOTE.md` — note encoding
- `guitarpro/FILE-STRUCTURE-CHORD-DIAGRAMS.md` — chord diagrams

This skill covers **how the code is wired**, so you edit the right place.

## Entry points

`guitarpro/src/model/legacy/song.rs`:
- `Song::read_gp3()` (~L112), `read_gp4()` (~L150), `read_gp5()` (~L177)
- `Song::write(version: (u8,u8,u8), clipboard: Option<bool>)` (~L260) — one writer,
  branches internally on version. Round-tripping IS supported.

CLI dispatch is by **file extension**, not magic bytes: `cli/src/loader.rs` (~L36–46)
maps `GP3|GP4|GP5` → the legacy readers.

Version detection: `io/primitive.rs` `read_version_string()` (~L203) matches the
30-byte header against the `VERSIONS` table and sets `Song.version.number: (u8,u8,u8)`
plus a `clipboard: bool`. Defaults to `(5,2,0)` if unmatched.

## The cursor / IO pattern

Everything reads through `(data: &[u8], seek: &mut usize)` — `data` is the whole
buffer, `seek` advances after each read. All low-level reads live in
`io/primitive.rs`. Use these, never hand-roll byte math:

| Fn | Bytes | Notes |
|---|---|---|
| `read_byte` / `read_signed_byte` | 1 | u8 / i8 |
| `read_bool` | 1 | non-zero = true |
| `read_short` | 2 | i16, little-endian |
| `read_int` | 4 | i32, little-endian |
| `read_double` | 8 | f64, little-endian |
| `read_byte_size_string` | 1+N | 1-byte length prefix, **Windows-1252** |
| `read_int_size_string` | 4+N | 4-byte length prefix |
| `read_int_byte_size_string` | 4+1+N | hybrid: int(len+1) then byte len |
| `read_color` | 4 | R,G,B + **1 blank byte skipped**; returns `r*65536+g*256+b` |

**String encoding**: Windows-1252 (`encoding_rs::WINDOWS_1252`) with UTF-8 fallback.
Lyric strings are the exception — written directly with no length-prefix convention
(see FILE-STRUCTURE.md).

## Trait-based API

Parsing/writing is split into traits implemented on `Song`, one per domain. Edit the
trait impl for the concern you're touching:

| Trait | Module | Concern |
|---|---|---|
| `SongTrackOps` | `model/legacy/track.rs` | tracks, channels, RSE, v5 display flags |
| `SongMeasureOps` | `model/legacy/measure.rs` | measures & voices |
| `SongHeaderOps` | `model/legacy/headers/io.rs` | measure headers, time sig, repeats, clipboard |
| `SongBeatOps` | `model/legacy/beat/` (`read.rs`/`write.rs`) | beat duration/status, mix table, chord ref |
| `SongNoteOps` | `model/legacy/note/` | pitch, velocity, fret/string, note effects |
| `SongChordOps` | `model/legacy/chord/io.rs` | chord diagrams (v3 vs v4 formats) |
| `SongEffectOps` | `model/legacy/effects/io.rs` | bends, grace, harmonic, slides, trill |
| `SongMixTableOps` | `model/legacy/mix_table/io.rs` | MIDI CC automation, wah |
| `SongLyricOps` | `model/legacy/lyric.rs` | 5 lyric lines per track |
| `SongPageOps` | `model/legacy/page.rs` | page setup (v5 only) |
| `SongRseOps` | `model/legacy/rse.rs` | RSE params (v5 only) |

Read/write are split into `read.rs` / `write.rs` files under each domain where the
domain is large (beat, note).

## Version branching — the core hazard

Branch on `song.version.number`. Real patterns in the code:

- `if song.version.number.0 == 5 { ..v5.. } else { ..legacy.. }` (headers/io.rs repeat alt)
- `if song.version.number == (3,0,0) { v3 } else if .0 == 4 { v4 }` (note effects)
- `if song.version.number < (5,0,0) { .. }` (beat reuse in GP3/4)
- `if self.version.number > (5,0,0) { read_bool } else { false }` (hide_tempo, v5.1+ only)

### Known version differences

| Feature | GP3 | GP4 | GP5 |
|---|---|---|---|
| Clipboard extra data | — | 4 ints | 7 ints (adds beat/subbar) |
| Note effects flags | 1 byte (v3) | 2 bytes (v4) | v4-format flags |
| Grace note | 4 bytes | 4 bytes | 5 bytes (+`is_on_beat`) |
| Beat reuse at same start | yes | yes | no (new beat each read) |
| Track display flags (16-bit) | — | — | yes |
| Page setup / RSE / Directions | — | — | yes |
| Triplet-feel | global bool before lyrics | same | per-measure-header |

### Quirks that bite

- **Color**: 4 bytes = R,G,B + one always-`0x00` blank byte (`read_color` skips it).
- **GP5.0.0 only**: extra blank byte before first track (`track.rs` ~L240).
- **Clipboard variants** exist (`CLIPBOARD GUITAR PRO 4.0`, `CLIPBOARD GP 5.x`) — the
  `clipboard` bool changes header parsing.
- The `VERSIONS` table has a deliberate `// sic` for `(5,2,0)` mapping to the
  `v5.10` header string.

## Tests

Test files live in the workspace-root `test/` dir (e.g. `test/Effects.gp5`). Test
modules are in `guitarpro/src/tests/`:
- `gp345_metadata.rs`, `gp345_structure.rs`, `gp345_chords.rs`,
  `gp345_note_effects.rs`, `gp345_beat_effects.rs`, `gp345_slides.rs`
- `roundtrip.rs` — parse → write → parse fidelity

Pattern: `let mut song = Song::default(); song.read_gp5(&read_file("test/Effects.gp5")).unwrap();`
then assert on fields.

## When adding a feature

1. Confirm the wire layout in the relevant `FILE-STRUCTURE*.md`.
2. Find the owning trait impl above; add reads via `io::primitive`.
3. Branch on `song.version.number` — check whether the field even exists in each version.
4. Mirror the change in the matching `write*` method (keep round-trip green).
5. Add/extend a test in `guitarpro/src/tests/` with a real `test/` fixture.
6. Avoid `unwrap()`/`expect()` — return `GpResult` errors (project standard).
