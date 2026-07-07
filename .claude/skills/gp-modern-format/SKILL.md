---
name: gp-modern-format
description: How the guitarpro crate reads/writes the MODERN Guitar Pro formats — .gpx (GP6, BCFZ/BCFS container) and .gp (GP7+, ZIP container), both wrapping GPIF XML. Use when working on container extraction, the Gpif XML structs, or GPIF↔Song conversion. NOT for legacy .gp3/.gp4/.gp5 binary (see gp-legacy-format).
---

# Modern Guitar Pro formats (.gpx = GP6, .gp = GP7+)

Both modern formats are **containers wrapping the same GPIF XML** (`score.gpif`).
The difference is only the container:

- **`.gp` (GP7+)**: standard **ZIP**, XML at path `Content/score.gpif`
- **`.gpx` (GP6)**: custom **BCFZ**-compressed **BCFS** virtual filesystem, XML file
  named `score.gpif`

XML lib: **`quick-xml` v0.39** with the `serialize` feature (both `de::from_str` and
`se::to_string`). ZIP: **`zip` v8.6**. GP6's BCFZ/BCFS is a hand-rolled parser (no dep).

## Files

| File | Role |
|---|---|
| `guitarpro/src/io/gpx.rs` | container read/write for BOTH `.gp` and `.gpx` |
| `guitarpro/src/io/gpif.rs` | the `Gpif` XML struct tree (serde) |
| `guitarpro/src/io/gpif_import.rs` | `Gpif` → internal `Song` |
| `guitarpro/src/io/gpif_export.rs` | `Song` → GPIF XML string |
| `guitarpro/src/convert/guitarpro/mod.rs` | higher-level conversion helpers |

## Entry points

`guitarpro/src/model/legacy/song.rs`:
- `Song::read_gp(data)` (~L205) → `gpx::read_gp` → `read_gpif`; sets version `(7,0,0)`
- `Song::read_gpx(data)` (~L213) → `gpx::read_gpx` → `read_gpif`; sets version `(6,0,0)`
- `Song::write_gp()` (~L227) and `Song::write_gpx()` (~L222) — round-trip supported

Dispatch is by **file extension** (`cli/src/loader.rs`), not magic bytes. The version
number is hardcoded on read, NOT extracted from the XML `GPVersion`/`GPRevision` attrs.

## Container mechanics

### .gp (ZIP) — `gpx.rs`
- `read_gp()` (~L166): `ZipArchive::new()` on the in-memory buffer, read
  `Content/score.gpif` into a `String`, `quick_xml::de::from_str` → `Gpif`.
- `write_gp_bytes()` (~L143): `ZipWriter` + deflate, GPIF XML at `Content/score.gpif`.

### .gpx (BCFZ/BCFS) — `gpx.rs`
- `read_gpx()` (~L386):
  1. **BCFZ decompress** (~L245): magic `b"BCFZ"`, 4-byte LE uncompressed size, then a
     bit-level `BitStream` (MSB-first). Blocks: literal (flag 0) or LZ77 back-reference
     (flag 1). **Overlap when size>offset** copies byte-by-byte with modular indexing
     (~L276).
  2. **BCFS filesystem parse** (~L317): magic `b"BCFS"`, 0x1000-byte sectors; file
     entries carry name (+0x04), size (+0x8C), block index table (+0x94, 0-terminated).
  3. Find file `score.gpif`, decode UTF-8, `de::from_str` → `Gpif`.
- `write_gpx_bytes()` (~L135): reverse — build GPIF XML, pack into BCFS, compress BCFZ.
- Reference impl noted in code: TuxGuitar.

## The `Gpif` struct — `gpif.rs`

Root `Gpif` (~L3) has ID-referenced, flattened sections (typical of GPIF): each level
holds space-separated ID lists pointing into the next. Top-level fields:

`version`, `revision`, `score`, `master_track`, `tracks`, `master_bars`, `bars`,
`voices`, `beats`, `notes`, `rhythms`.

Relationship chain: `MasterBar.bars` → `Bar.voices` → `Voice.beats` → `Beat.notes` +
`Beat.rhythm`. Import resolves these via HashMaps keyed by ID.

Notable sub-structs: `Score` (metadata), `MasterTrack` (tempo automations),
`Track` (id, name, color RGB, tuning via `properties` in GP6 or `staves` in GP7,
`general_midi`, `transpose`), `MasterBar` (time "4/4", key, repeat, alternate endings,
sections, fermatas), `Beat` (dynamic, grace, tremolo, properties), `Note` (fret/string/
pitch/harmonics via flexible `properties`, tie, vibrato, let_ring, accent, trill),
`Rhythm` (`note_value` e.g. "Quarter", `augmentation_dot`, `primary_tuplet`).

## Conversion `Gpif` → `Song` — `gpif_import.rs`

`SongGpifOps::read_gpif()` (~L209):
- Metadata from `Score` (~L210)
- Tempo from `MasterTrack.automations` where type `"Tempo"` (~L226)
- Builds ID→object HashMaps (~L247)
- MasterBars → measure headers: time sig split from `"n/d"`, key from `accidental_count`,
  repeats, volta bitmask, sections/fermatas (~L256)
- Tracks: tuning (GP6 `properties` / GP7 `staves[0]`, fallback standard EADGBE),
  MIDI (ch 9 = percussion), transpose, per-track measures (~L364)
- `convert_beat()` (~L484): duration from `Rhythm.note_value`, dots, tuplet; dynamic →
  velocity; grace; notes; brush/pickstroke; then `convert_note()` for fret/string,
  bends, slides (flag bitmask), harmonics.

Export mirrors this in `gpif_export.rs` `SongGpifExportOps::write_gpif_xml()` (~L370),
sequentially assigning IDs across bars/voices/beats/notes/rhythms.

## Gotchas

- **`<Fadding>` typo**: GP6 XML misspells "Fading"; the deserializer accommodates it.
- **Harmonic `"Feedback"`** maps to `Pinch` harmonic on import.
- **Marker/section colors** aren't in GPIF — import defaults to red `0xff0000`.
- **Version** isn't read from the XML attrs — it's hardcoded `(6,0,0)`/`(7,0,0)`.
- **Empty tuning** falls back to 6-string EADGBE.
- Tempo parse failure warns and defaults to 120 BPM.

## Tests

`guitarpro/src/tests/gp7.rs` (covers `.gp`), plus `.gpx`/`.gp` fixtures in the
workspace-root `test/` dir.

## When editing

1. Container bug → `gpx.rs`. XML shape mismatch → add/adjust the field in `gpif.rs`
   (serde rename attrs matter — match GP's element/attribute names exactly).
2. Semantic mapping → `gpif_import.rs` / `gpif_export.rs`; keep the two symmetric for
   round-trip.
3. Prefer `GpResult` errors over `unwrap()`/`expect()` (project standard).
