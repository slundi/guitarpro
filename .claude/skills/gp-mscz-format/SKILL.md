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
| `guitarpro/src/convert/loss.rs` | `LossReport` (sorted `BTreeMap<String, u32>`), re-exported as `convert::mscz::LossReport` |
| `guitarpro/src/convert/mscz/spelling.rs` | tonal pitch classes (`<tpc>`/`<tpc2>`), written pitch and key of a transposing staff |
| `guitarpro/src/convert/mscz/instruments.rs` | MuseScore `<Instrument id>` + `<instrumentId>` sound from `instrument_sound` or the GM program |

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
- `parse_voice` — pushes `<TimeSig>`, `<KeySig>` (`concertKey` and
  `mode`; the written `actualKey` is not read), `<Tempo>` onto the parent measure
  once (first voice wins), the first voice's `<Clef>`s (before its beats:
  `start_clef`, after its last beat: `end_clef`, between beats: counted
  in `mid_clefs`), and collects Chord/Rest beats; a voice-level
  `<location>` becomes the next beat's `gap_before`, and voice-level
  `<Spanner>`s are skipped
- `parse_chord` / `parse_rest` — `<durationType>` + optional `<dots>` +
  child `<Note>`s; each of a chord's `<Spanner type="Slur">`s becomes an
  `MscxSlurEnd` (start with `<next>`, end with `<prev>`) with the other
  end's relative `<location>` (`measures`, `fractions`, `voices`)
- `parse_note` — `<pitch>`, `<tpc>`, `<head>`, `<string>`,
  `<fret>`, and `<Spanner type="Tie">` (a start when `<next>` is present, an
  end when `<prev>` is present; a chain's middle note has both)
- `parse_instrument` — also the first `<Channel>`'s `<program value>`
  (later channels are alternative sounds and are skipped)

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
  `instrument_sound` (MuseScore's `<instrumentId>`), `midi_program` (first
  `<Channel>`), `<transposeChromatic>` + `<transposeDiatonic>`.
  `<StringData>` → `InstrumentKind::Stringed { tuning }`;
  `drum.*` instrumentId → `InstrumentKind::Percussion`
- `<Part>/<Staff>/<defaultClef>` + `<StaffType group="…">` → `StaffDef`
  (`Clef::{Treble, Bass, Alto, Tenor, Percussion, Tab, Treble8vb, Treble8va,
  Bass8vb, Bass8va}`, `StaffDisplay::{Notation, Tab}`); a `<Clef>` at a
  measure's start, or at the end of the measure before, →
  `MeasureData.clef` (the first measure's header clef only when it differs
  from the staff's); a clef between two beats can't be held and is
  reported as `Clef (mid-measure)`, one after the last measure's last beat
  as `Clef (end of score)`
- Per `<Measure>`: `<TimeSig>` / `<KeySig>` / `<Tempo>` → `MeasureDef`
  (first measure always announces the initial signatures; subsequent
  measures announce only on change)
- Repeat marks → `NavigationEvent`:
  `<startRepeat/>` → `JumpKind::RepeatOpen`,
  `<endRepeat>N</endRepeat>` → `JumpKind::RepeatClose { repeat_count: N }`
- `<Chord>` → `Beat` with `Duration { base, dots }` and `notes` populated;
  `<Rest>` → `Beat { gp_rest: true, notes: [] }`
- `<Note>`: MIDI `<pitch>` → `Pitch`, spelled by `<tpc>` (sharps via
  `midi_to_pitch` without one) — always the **sounding** pitch, the written
  one follows from the instrument's `Transpose`; `<head>` →
  `Notehead`; `<string>` is MuseScore's 0-based encoding, we store 1-based
  on `Note.string`; `<fret>` is copied verbatim; `<Spanner type="Tie">` →
  `TieType::{Start, End, Continue}` (`Continue` = both ends)
- Chord slurs → `Beat.slur`. Ends pair with starts by their `<location>`,
  so nested slurs and slurs between voices stay apart (an end matching no
  location falls back to its voice's innermost open slur).
  `SlurKind::EndStart` marks a chord that ends one slur and starts
  another, which takes over the ending slur's id (so the id names both);
  ids are unique among open slurs and reused once closed. Grace chords
  sort before their main chord at the same tick; a slur touching one is
  paired, then dropped whole. Reported: `Slur (unpaired end)`, `Slur
  (unpaired start)`, `Slur (several on one chord)` (a chord holds at most
  one start and one end) and `Slur (on a grace chord)`
- `<KeySig>` → the timeline's **concert** key via
  `KeySignature::from_fifths` (`<concertKey>` and `<mode>`; major without
  one). A modal key's root is the mode's tonic, as in every converter
- Voltas → barline `Ending`s, which `Arrangement::resolve` plays by pass
- Staff clef: `<defaultClef>`, else the `<Instrument>`'s `<clef>` for that
  staff (`staff` attribute, 1-based)
- Reported as well: `Tremolo (two-chord)`, `Tremolo (unknown)`,
  `ChordLine (unknown)`, `Dynamic (velocity)` (one MuseScore wouldn't
  give the dynamic by default), `Dynamic (voice assignment)`
- Dynamics: the writer writes a staff's dynamics for their own voice only
  (`<voiceAssignment>currentVoiceOnly</voiceAssignment>`) when they're in
  several voices, as the model gives each such voice its own level, else
  for all voices; one assigned otherwise on import is reported
- Grace chords take no time and wait for the next normal chord — after-
  graces (`grace8after`…) too, since MuseScore writes all of a chord's
  graces before it; graces with no chord after them follow the last one

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
3. One `<Part>` per track: `<Staff>` (`<StaffType group>` chosen from
   `StaffDisplay`, `<defaultClef>` unless treble), `<trackName>`,
   `<Instrument id>` (MuseScore template, see `instruments.rs`) with
   `<longName>`, `<shortName>`, `<instrumentId>` (the sound MuseScore 4
   plays — without one a part plays as a piano), `<transposeChromatic>`,
   `<transposeDiatonic>`, `<StringData>` (MIDI numbers via `pitch_to_midi`),
   `<clef>`, `<Channel><program value>`
4. One master `<Staff id>` per track with `<Measure>` children:
   `<startRepeat/>` / `<endRepeat>` from navigation; on the primary voice a
   header `<Clef>` (first measure: its `MeasureData.clef`, else the
   staff's; never on tab staves), `<TimeSig>` / `<KeySig>` (`concertKey`
   via `KeySignature::fifths` from tonic and mode, `actualKey` on
   transposing staves, and `<mode>`) / `<Tempo>`, and at its end a
   `<Clef>` for the next measure's `MeasureData.clef`, the `<BarLine>`
   (none on an `<endRepeat>` measure — MuseScore draws that barline, an
   explicit one would replace it) and volta ends, after a `<location>`
   past any gap when the voice is shorter than the measure; then grace
   chords (after-graces too) before their `<Chord>` / `<Rest>` with
   `<dots>` and `<durationType>`, chord slurs as `<Spanner type="Slur">`
   located by the other end's chord — an `End` or `EndStart` closes the
   open slur with its id, an `EndStart` then opens the next under it; a
   slur touching a beat written as `<Rest>` is dropped — and `<Note>`
   children with `<pitch>` / `<tpc>` /
   `<tpc2>` (transposing staves, when the written note is spellable) /
   `<head>` / `<string>` (converted back to
   0-based) / `<fret>` / `<Spanner type="Tie">`

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
- **`<dots>` before `<durationType>`**: MuseScore applies the dots when it
  reads the duration type, so a chord whose `<dots>` comes after its
  `<durationType>` is timed undotted (still drawn dotted) and MuseScore
  fills the measure up with rests.
- **`<Spanner type="Tie">`**: `plan_ties` pairs each tied note with the
  next note of its pitch in its voice that is tied from before. The start
  is written with a `<Tie>` and a `<next>` location of the end, the end
  with a `<prev>` location of the start (as for slurs); a `Continue` note
  gets both. MuseScore pairs the ends by these locations, so an unpaired
  tie end is dropped.
- **Spanner locations**: a slur's start (`<next>`) and end (`<prev>`)
  elements are paired by their relative `<location>` (`measures`,
  `fractions` of a whole note, `voices`) — they must point at where the
  other chord actually starts.
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
  `LossReport`, multi-part scores, repeat navigation, programs,
  transposing staves, clefs, slurs and noteheads (Part 2)
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
