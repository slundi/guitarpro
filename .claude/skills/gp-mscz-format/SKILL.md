---
name: gp-mscz-format
description: How the guitarpro crate reads/writes the MuseScore compressed archive format (.mscz) — a ZIP wrapping the .mscx XML score. Use when working on the container layer, the Mscx AST, the MSCX↔LoadedScore converter, or the LossReport. NOT for Guitar Pro formats (see gp-modern-format / gp-legacy-format) or MusicXML (see musicxml-conversion).
---

# MuseScore compressed archive (`.mscz`)

MSCZ is a **plain ZIP archive** wrapping a single MuseScore XML score
(`.mscx`) plus optional side files. Unlike `.gp` (GP7+) — which is also a
ZIP — the MSCZ archive carries an OPC-style `META-INF/container.xml`
manifest naming the root MSCX file. That manifest is what distinguishes an
MSCZ from a `.gp` archive at the byte level.

XML lib: **`quick-xml` v0.41** with event-driven streaming (no serde
deserialize — the MSCX schema is too irregular). ZIP: **`zip` v8.6** (same
crate as `.gp`).

## Archive layout

```
score.mscz  (ZIP)
├── META-INF/
│   └── container.xml           — <rootfile full-path="…"/> manifest
├── <name>.mscx                 — the score body (MuseScore XML)
├── <name>.mss                  — optional style file (raw XML fragments)
├── Thumbnails/
│   └── thumbnail.png           — preview image (rendered by MuseScore)
├── audiosettings.json          — mixer / plugin config
├── viewsettings.json           — viewport / zoom
└── (optional) embedded fonts, images, soundfont overrides
```

Every entry is preserved verbatim on `MsczArchive.entries`. The raw MSCX
XML is *also* preserved on `Mscx::raw_xml` — this is the source of truth
for byte-stable round-trips of unmodified archives.

## Files (crate paths)

| File | Role |
|---|---|
| `guitarpro/src/io/mscz/container.rs` | ZIP read/write, `container.xml` manifest parsing, `MAX_MSCZ_BYTES`/`MAX_MSCZ_ENTRIES` guards |
| `guitarpro/src/io/mscz/parse.rs` | streaming MSCX parser (`parse_mscx`, `write_mscx`) |
| `guitarpro/src/io/mscz/mod.rs` | `read_mscz` / `read_mscz_bytes` / `write_mscz` / `write_mscz_to_path` |
| `guitarpro/src/model/mscz/mod.rs` | `MsczFile { archive, mscx }`, `MsczArchive`, `MsczEntry` |
| `guitarpro/src/model/mscz/mscx.rs` | full AST (`Mscx`, `MetaTag`, `Part`, `Staff`, `Instrument`, `MscxStaff`, `MscxMeasure`, `MscxVoice`, `MscxBeat`, `MscxNote`, …) |
| `guitarpro/src/convert/mscz/to_optimized.rs` | `mscx_to_loaded_score` → `ConvertOutcome { score, report }` |
| `guitarpro/src/convert/mscz/from_optimized.rs` | `loaded_score_to_mscx` (regenerates a MuseScore 4.10 MSCX from the optimized model) |
| `guitarpro/src/convert/mscz/validate.rs` | `LossReport` (sorted `BTreeMap<String, u32>`) |

## Entry points

```rust
// Read
let file: MsczFile = guitarpro::read_mscz("score.mscz")?;
let file: MsczFile = guitarpro::read_mscz_bytes(&bytes)?;

// Traverse
file.archive.entries   // every ZIP entry, byte-identical
file.archive.mscx_entry()      // primary .mscx (via manifest rootfile)
file.archive.thumbnail_entry() // Thumbnails/thumbnail.png
file.mscx.meta("workTitle")    // metaTag lookup

// Convert
use guitarpro::convert::mscz::{mscx_to_loaded_score, loaded_score_to_mscx};
let outcome = mscx_to_loaded_score(&file.mscx);
let regenerated_mscx = loaded_score_to_mscx(&outcome.score);

// Write
let bytes: Vec<u8> = guitarpro::write_mscz(&file)?;
```

## Container mechanics

### Read (`container.rs::read_container`)

1. Reject inputs > `MAX_MSCZ_BYTES` (32 MB) — before touching the ZIP crate
2. `ZipArchive::new`; reject archives with > `MAX_MSCZ_ENTRIES` (256) entries
3. For each non-directory entry, keep `MsczEntry { path, data: Vec<u8> }`
4. If `META-INF/container.xml` is present, parse the `<rootfile>` list into
   `MsczArchive.rootfiles` (empty otherwise — the fallback picks any
   `*.mscx` outside `META-INF/`)

### Write (`container.rs::write_container`)

- Entries emitted in `archive.entries` order (deterministic)
- Compression per extension: `Deflated` for XML/JSON/text, `Stored` for
  PNG/JPEG (already compressed — deflate would inflate). This avoids the
  pathological expansion MuseScore's thumbnails sometimes trigger.
- ZIP timestamps come from the `zip` crate defaults — byte equality with
  MuseScore-produced archives is not promised; `read → write → read`
  stability *is* what round-trip tests exercise.

## MSCX parser (`parse.rs`)

Hand-rolled quick-xml event streaming — a serde-based deserializer would
choke on MuseScore's irregular schema (elements that can be either empty
tags, self-closed, or open/close depending on version).

**Version gate**: `<museScore version="X.Y">` must have major = 4. Others
return `GpError::MsczUnsupported { got, supported: "4.x" }`.

**Parser structure**: a top-level state machine tracks `depth_in_score`
and `current_part`, then dispatches into recursive sub-parsers:

- `parse_score_staff` — consumes `<Score>/<Staff>` (master staves with
  measures), producing `MscxStaff { staff_id, measures }`
- `parse_measure` — `<Measure>` including `<startRepeat/>`,
  `<endRepeat>N</endRepeat>`, and `<voice>` children
- `parse_voice` — pushes `<TimeSig>`, `<KeySig>`, `<Tempo>` onto the
  parent measure once (first voice wins), and collects Chord/Rest beats
- `parse_chord` / `parse_rest` — `<durationType>` + optional `<dots>` +
  child `<Note>`s
- `parse_note` — `<pitch>`, `<tpc>`, `<string>`, `<fret>`, and
  `<Spanner type="Tie">` (Start when `<next>` is present, End when `<prev>`
  is present)

**Entity handling**: `read_text` decodes standard XML entities (`&amp;`,
`&lt;`, `&gt;`, `&quot;`, `&apos;`) and numeric character references
(`&#65;`, `&#x41;`). Custom DTD entities cause `GpError::MsczXml`.

## MSCX AST layers

The AST is **layered** so unimplemented features don't block round-trip:

- **`raw_xml`**: the source-of-truth string — used by `write_mscx` and by
  `mscz_archive.mscx_entry()`. Left untouched for unmodified archives so a
  round-trip is byte-stable.
- **Envelope**: `version`, `program_version`, `program_revision`, `division`
- **Header**: `meta_tags: Vec<MetaTag>` (workTitle, composer, copyright,
  arranger, lyricist, movementTitle, creationDate, source, …)
- **Instruments**: `parts: Vec<Part>` — each with `staves`, `instrument`
  (long/short name, transpose, `StringData` for guitars)
- **Body**: `score_staves: Vec<MscxStaff>` — each with `measures`, and each
  measure with time/key/tempo/repeats plus voices with beats and notes
- **Legacy field**: `measure_counts: Vec<StaffMeasureCount>` — a cheap
  summary computed from `score_staves`; kept for callers that only want the
  measure count without walking the whole body

## MSCX ↔ LoadedScore conversion

### `mscx_to_loaded_score` (`to_optimized.rs`)

Populates a `LoadedScore`:

- `metaTag`s → `Metadata { title, composer, copyright, work,
  identification, movement_number, artist, album, year }`; unknown tags
  are silently ignored
- `<Part>/<Instrument>` → `Instrument` — long name, abbreviation,
  `instrument_sound` (MuseScore's `<instrumentId>`), `<transposeChromatic>`
  + `<transposeDiatonic>`. `<StringData>` → `InstrumentKind::Stringed { tuning }`;
  `drum.*` instrumentId → `InstrumentKind::Percussion`
- `<Part>/<Staff>/<defaultClef>` + `<StaffType group="…">` → `StaffDef`
  (`Clef::{Treble, Bass, Alto, Tenor, Percussion, Tab}`, `StaffDisplay::{Notation, Tab}`)
- Per `<Measure>`: `<TimeSig>` / `<KeySig>` / `<Tempo>` → `MeasureDef`
  (first measure always announces the initial signatures; subsequent
  measures announce only on change)
- Repeat marks → `NavigationEvent`:
  `<startRepeat/>` → `JumpKind::RepeatOpen`,
  `<endRepeat>N</endRepeat>` → `JumpKind::RepeatClose { repeat_count: N }`
- `<Chord>` → `Beat` with `Duration { base, dots }` and `notes` populated;
  `<Rest>` → `Beat { gp_rest: true, notes: [] }`
- `<Note>`: MIDI `<pitch>` → `Pitch` (via `midi_to_pitch`); `<string>` is
  MuseScore's 0-based encoding, we store 1-based on `Note.string`; `<fret>`
  is copied verbatim; `<Spanner type="Tie">` → `TieType::{Start, End}`

Anything not mapped is recorded in the returned `LossReport`:

```rust
let outcome = mscx_to_loaded_score(&file.mscx);
for (name, count) in outcome.report.iter() {
    eprintln!("{name}: {count} occurrence(s) lost");
}
```

### `loaded_score_to_mscx` (`from_optimized.rs`)

Generates a MuseScore 4.10 MSCX from the optimized model:

1. Emit envelope: `<?xml…?>`, `<museScore version="4.10">`,
   `<programVersion>`, `<programRevision>`, `<Score>`, `<Division>`
2. metaTags from `Metadata` (title, composer, copyright, movementNumber,
   creationDate — pulled from `Identification.encoding_date`)
3. One `<Part>` per track: `<Staff>` (clef, `<StaffType group>` chosen
   from `StaffDisplay`), `<trackName>`, `<Instrument>` with
   `<longName>`, `<shortName>`, `<instrumentId>`, `<transposeChromatic>`,
   `<transposeDiatonic>`, `<StringData>` (MIDI numbers via `pitch_to_midi`)
4. One master `<Staff id>` per track with `<Measure>` children:
   `<startRepeat/>` / `<endRepeat>` from navigation, `<TimeSig>` /
   `<KeySig>` (via `fifths_from_root`) / `<Tempo>` on the primary voice,
   then `<Chord>` / `<Rest>` with `<durationType>` and `<dots>`, and
   `<Note>` children with `<pitch>` / `<string>` (converted back to 0-based)
   / `<fret>` / `<Spanner type="Tie">`

Ordering is deterministic (parts, measures, voices sorted by id) so the
regenerated file is stable across runs.

## Gotchas

- **`.gp` also uses ZIP**: distinguish MSCZ from `.gp` by probing for
  `META-INF/container.xml` (see `cli/src/loader.rs::is_mscz_archive`).
  Extension alone is not always reliable.
- **String number encoding**: MuseScore stores `<string>` as 0-based (top
  string = 0); the optimized `Note.string` is 1-based. The converters
  translate in both directions.
- **Repeat count**: `<endRepeat>N</endRepeat>` may appear as an empty tag
  (`<endRepeat/>`); the parser treats that as `Some(2)` (MuseScore's
  default when the count is omitted).
- **Time / key / tempo announce**: the first measure always emits the
  initial signatures even if the source MSCX didn't (the optimized model
  requires them for a valid baseline).
- **Tempo units**: MSCX stores tempo as beats-per-second (float), but the
  optimized model wants BPM — we multiply by 60 on import, divide by 60
  on export.
- **`<Spanner type="Tie">`**: emit with `<next/>` for `TieType::Start` and
  `<prev/>` for `TieType::End`. MuseScore fills in the actual target IDs
  when it re-opens the file.
- **Empty MSCX files**: rejected with `"missing <museScore> root element"`
  — we never construct a partial `Mscx` from a malformed input.
- **MuseScore 3.x / 2.x**: rejected with `MsczUnsupported { got, supported: "4.x" }`.
  Adding 3.x support means feature-gating (proposed `mscz3` cargo feature).
- **`.mss` style files**: preserved byte-for-byte in the archive but not
  parsed as structured `Style` (yet).
- **The `mscz.*` misc-store convention** was reserved for MuseScore-specific
  fields that don't map cleanly to the optimized model (mirroring `gp.*`),
  but is currently unused — the raw XML on `Mscx::raw_xml` covers the
  same need for consumers that don't mutate the score.

## Tests

- `guitarpro/src/tests/mscz.rs` — container read/write, MSCX AST parsing,
  entity unescape, error paths, `MsczFile` round-trip (Part 1)
- `guitarpro/src/tests/mscz_convert.rs` — MSCX ↔ `LoadedScore`: metadata,
  instruments/tunings, timeline, notes with pitch/string/fret/ties,
  `LossReport`, multi-part scores, repeat navigation (Part 2)
- `cli/tests/mscz_cli.rs` — full CLI paths (`info`, `convert`, `repeats`,
  `mscz list/extract/thumbnail`, duplicates walker) end-to-end (Part 3)
- `web_server/tests/mscz_api.rs` — HTTP surface: upload / info / raw /
  analysis / download / thumbnail / file browser (Part 4)

Fixtures are **built programmatically** (no copyrighted MuseScore files
committed to the repo). During development, the parser has been smoke-
tested against a 94-file real-world corpus of MuseScore 4.0 – 4.6 exports
(469 k notes / 361 k beats parsed without failures).

## When editing

1. **Container / ZIP bug** → `src/io/mscz/container.rs`. Keep the write
   path's entry ordering deterministic; tests assert byte-stable
   read → write → read.
2. **XML shape change** (new MuseScore version tag, added element) →
   extend the AST in `src/model/mscz/mscx.rs`, then wire the parser branch
   in `src/io/mscz/parse.rs`. Unknown tags should be silently ignored, not
   fatal.
3. **Semantic mapping** → `src/convert/mscz/{to,from}_optimized.rs`; keep
   the two symmetric and add a corresponding `report.note(...)` call for
   anything you observe but don't preserve.
4. **New error condition** → extend `GpError` in `src/error.rs`. Reserve
   `MsczArchive` for ZIP / container problems, `MsczXml` for XML content
   problems, and `MsczUnsupported` for version / feature gates.
5. Prefer `GpResult` errors over `unwrap()` / `expect()` (project
   standard). No `expect()` at all in the MSCZ crate.
