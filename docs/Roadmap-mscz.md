# MSCZ File Format Roadmap

Support for **MuseScore's compressed score format** (`.mscz`) in the `guitarpro`
crate, with round-trip conversions to the internal model, CLI integration for
inspection/conversion, and web-server upload/download endpoints.

MSCZ is a **ZIP archive** wrapping MuseScore's XML score representation
(MSCX). It typically contains:

```
score.mscz  (ZIP)
├── META-INF/
│   └── container.xml           — manifest pointing at the root .mscx
├── <score>.mscx                — main score XML (MuseScore's format)
├── <score>.mss                 — optional style file
├── Thumbnails/
│   └── thumbnail.png           — preview image
├── audiosettings.json          — mixer / playback state
├── viewsettings.json           — layout / zoom state
└── (optional) embedded fonts, images, soundfont overrides
```

The `.mscx` XML sits at the same abstraction level as GPIF (modern Guitar Pro)
or MusicXML: it's the authoritative score representation. The archive layer
adds thumbnails, per-part styles and playback state.

MuseScore 4 versions the XML as `<museScore version="4.x">`. The roadmap
targets **MuseScore 4.x** first; MuseScore 3 (`.mscz` with `<museScore
version="3.x">`) is a stretch goal.

---

## Architecture Overview

```
┌───────────────────────────────────────────────────────────┐
│  MSCZ archive (ZIP)                                       │
│  ┌────────────────────────┐  ┌────────────────────────┐   │
│  │  container.xml         │  │  <score>.mscx (XML)    │   │
│  │  audiosettings.json    │  │  <score>.mss (style)   │   │
│  │  thumbnail.png         │  │  embedded assets       │   │
│  └────────────────────────┘  └────────┬───────────────┘   │
└───────────────────────────────────────┼───────────────────┘
                                        │
┌───────────────────────────────────────▼───────────────────┐
│  guitarpro::io::mscz                                      │
│  ┌────────────────────────────────────────────────────┐   │
│  │  Container extraction (zip crate)                  │   │
│  ├────────────────────────────────────────────────────┤   │
│  │  MSCX XML parser  (quick-xml)                      │   │
│  ├────────────────────────────────────────────────────┤   │
│  │  Mscx AST  ──►  optimized::LoadedScore             │   │
│  │  optimized::LoadedScore  ──►  Mscx AST             │   │
│  └────────────────────────────────────────────────────┘   │
│                          │                                │
│  Sits next to gpx / gpif / musicxml under src/io/         │
└───────────────────────────────────────────────────────────┘
```

Design principles (aligned with the existing GPIF / MusicXML modules):
- `src/io/mscz/` owns container (un)packing, side-file JSON, thumbnails
- `src/model/mscz/` owns the pure `Mscx` AST (strongly typed structs, no I/O)
- `src/convert/mscz/` owns `Mscx ↔ optimized::LoadedScore` conversion
- Preserve unknown / non-lossy XML fragments in a `misc` store, mirroring the
  strategy used for GP7 (`gp.*` keys) so round-trip stays lossless

---

## Part 1 — Library: Archive & AST ✅

### 1.1 Container extraction

- Add `zip` dependency (already used transitively for `.gp` in `io/gpx.rs`;
  reuse the same crate/version)
- `src/io/mscz/mod.rs` — module entry
- `src/io/mscz/container.rs` — `read_container(bytes) -> MsczArchive`
  - Enumerate ZIP entries
  - Parse `META-INF/container.xml` to locate the root `.mscx`
  - Return an in-memory struct with `mscx_xml: String`, `style_xml:
    Option<String>`, `audio_settings: Option<Vec<u8>>`, `view_settings:
    Option<Vec<u8>>`, `thumbnail_png: Option<Vec<u8>>`, `extras:
    HashMap<String, Vec<u8>>`
- `src/io/mscz/container.rs` — `write_container(archive) -> Vec<u8>`
  - Deterministic entry order for byte-stable round-trips
  - Reproduces `META-INF/container.xml` with the correct rootfile path
- File-size / entry-count guards (reject archives > 32 MB or > 256 entries
  by default; configurable via loader constants — mirrors CLI's GP limits)

### 1.2 MSCX AST (`src/model/mscz/`)

- `mod.rs` — top-level `Mscx { version, program, score }` + `MsczFile`
  (AST + side files)
- `score.rs` — `Score`, `Part`, `Staff`, `Instrument`, `PartList`
- `measure.rs` — `Measure`, `Voice`, `Chord`, `Rest`, `BarLine`,
  `TimeSig`, `KeySig`, `Clef`, `Tempo`
- `note.rs` — `Note`, `Pitch`, `Tie`, `Tuplet`, `Beam`, `Articulation`,
  `Fingering`, `Bend`
- `layout.rs` — `LayoutBreak`, `Spacer`, `SystemDistance`, `PageBreak`
- `style.rs` — `Style` (parsed `.mss` fragments; kept as a struct only for
  fields we understand, rest as raw XML in a `misc` map)
- `metadata.rs` — `metaTag` entries (`workTitle`, `composer`, `arranger`,
  `lyricist`, `copyright`, …)
- Rich AST for guitar-relevant elements: `StringData` (tuning), `FretDiagram`,
  `HarmonicMark`, `PalmMute`, `LetRing`, `Vibrato`, `Slide`, `Tremolo`

### 1.3 XML parser / serializer

- `src/io/mscz/parse.rs` — `parse_mscx(xml: &str) -> Result<Mscx>`
  using `quick-xml` (already in the tree)
- `src/io/mscz/write.rs` — `write_mscx(&Mscx) -> String`
  - Match MuseScore's whitespace/indent style so third-party diff tools stay
    readable
  - Preserve attribute order in round-trips
- Handle unknown tags by capturing them into a `unknown: Vec<XmlFragment>`
  on each parent struct, mirroring GP7's approach

### 1.4 Public entry points

- `guitarpro::io::mscz::read_mscz(path)` → `MsczFile`
- `guitarpro::io::mscz::read_mscz_bytes(&[u8])` → `MsczFile`
- `guitarpro::io::mscz::write_mscz(&MsczFile) -> Vec<u8>`
- Re-export from `guitarpro::lib.rs` alongside `read_song` /
  `read_musicxml`

### 1.5 Error handling

- Extend `guitarpro::error::Error` with `Mscz(MsczError)`
- `MsczError` variants: `Zip`, `MissingContainer`, `MissingRootFile`,
  `Xml`, `UnsupportedVersion { got, supported }`, `UnknownElement { tag,
  line }`, `TooLarge`, `TooManyEntries`
- Follow the crate rule: no `expect()` / `unwrap()`; all failures return
  `Result`

---

## Part 2 — Round-trip Conversions ✅ (structural subset)

Delivered: MSCX ↔ `LoadedScore` for the **structural** subset — metadata,
instruments/tunings, staff definitions, per-measure signatures / tempo /
repeats, and note content (pitch/string/fret/ties). The converter runs
against the 94-file real-world MuseScore 4.0 – 4.6 corpus (469 k notes
across 361 k beats) without failures, and every observed-but-not-mapped
feature is tallied in a `LossReport`.

Deeper coverage (dynamics, articulations, ornaments, chord symbols,
tuplets, beam groups, spanners, `.mss` styles) is scoped as follow-up
work inside this Part rather than a separate roadmap Part.

### 2.1 MSCX ↔ optimized model ✅ (subset)

- `src/convert/mscz/mod.rs` — module entry
- `src/convert/mscz/to_optimized.rs` — `mscx_to_loaded_score(&Mscx) ->
  ConvertOutcome { score, report }`
  - Parts → `Instrument` (name, abbreviation, `instrument_sound`,
    transpose, `Stringed { tuning }` / `Percussion` detection)
  - Part `Staff` → `StaffDef` (clef + notation/tab display)
  - Score-level `Staff` `<Measure>` → per-track `MeasureData` with voices,
    beats (Chord/Rest), notes (`<pitch>` → `Pitch`, `<string>` → 1-based,
    `<fret>`, `<Spanner type="Tie">` → `TieType`)
  - `metaTag`s → `Metadata` (title, composer, copyright, encoding date,
    `Work { number, title }`, `Identification { creators, encoding_software,
    encoding_date, source }`)
  - `<TimeSig>` / `<KeySig>` / `<Tempo>` → `MeasureDef.time_signature`,
    `.key_signature`, `.tempo`; first measure always announces the initial
    triple, subsequent measures announce only on change
  - `<startRepeat/>` / `<endRepeat>N</endRepeat>` → `NavigationEvent`
    entries with `JumpKind::RepeatOpen` / `RepeatClose { repeat_count }`
- `src/convert/mscz/from_optimized.rs` —
  `loaded_score_to_mscx(&LoadedScore) -> Mscx`
  - Emits `<museScore version="4.10">` envelope, `metaTag`s, one `<Part>`
    per track with Staff/Instrument/StringData, and one master `<Staff>`
    per track with Measure/voice/Chord/Rest/Note
  - Deterministic ordering (parts, measures, voices sorted by id)
- Deferred (tracked in `LossReport`):
  - `<Beam>` grouping, tuplets, grace notes
  - `<HairPin>`, `<Slur>`, `<Trill>`, `<Volta>` spanners
  - Chord symbols, fret diagrams, dynamics, articulations, ornaments
  - MuseScore layout hints (`<LayoutBreak>`, `<vspacerFixed>`, etc.)
  - `mscz.*` misc-store keys are unused so far — the raw XML on
    `Mscx::raw_xml` already provides a lossless fallback for consumers
    that don't mutate the score

### 2.2 MSCZ ↔ Guitar Pro (via optimized)

- [x] Real-world smoke: 94/94 files in the local corpus parse and convert
  cleanly to `LoadedScore` (469 379 notes, 361 655 beats)
- [ ] Add MSCZ tests to `src/tests/roundtrip_optimized.rs` (or a sibling file
  `roundtrip_mscz.rs`) modeled after the existing GP3/4/5/7/GPX tables
- [ ] Collect an anonymized corpus of MSCZ test files under
  `guitarpro/samples/mscz/` (a mix of MuseScore 4.0 – 4.6 exports)
- [ ] MSCZ → Song → MSCZ byte-stable round-trip (or documented tolerances)
- [ ] GP5 → MSCZ → GP5 semantic round-trip (notes, timing, tunings survive)
- [ ] GPX/GP7 → MSCZ → GPX/GP7 semantic round-trip

### 2.3 MSCZ ↔ MusicXML

- [x] Cross-format works implicitly through `LoadedScore` — both
  `mscx_to_loaded_score` and the existing `loaded_score_to_score_partwise`
  are wired
- [ ] Add a MusicXML → MSCZ → MusicXML round-trip test to
  `roundtrip_musicxml.rs` (or the equivalent existing file)

### 2.4 Version compatibility ✅

- Detect MSCZ version from `<museScore version="X.Y">`; support 4.x
  first, gate 3.x behind a future `mscz3` cargo feature
- Emit `MsczUnsupported { got, supported }` for MuseScore 3.x and 2.x
- `.mss` style files: preserved byte-for-byte in `MsczArchive.entries`
  for round-trip; not yet parsed as structured `Style`

### 2.5 Loss report ✅

- [x] `guitarpro::convert::mscz::LossReport` — sorted `BTreeMap<String, u32>`
  of MSCX features the converter observed but did not map to `LoadedScore`,
  returned as `ConvertOutcome.report`
- [ ] Wire into CLI `convert --report` and web-server `/api/score/:id/lossreport`
  (Parts 3 / 4)

---

## Part 3 — CLI Integration (`cli/`)

### 3.1 Loader

- [ ] `cli/src/loader.rs` — extend `load_score` to detect `.mscz` by extension
  and magic bytes (`PK\x03\x04` + `container.xml` presence) and dispatch to
  `guitarpro::io::mscz::read_mscz`
- [ ] Add MSCZ to the 16 MB size limit constant (or bump to 32 MB — MSCZ files
  can be larger due to embedded fonts/audio)

### 3.2 `score_tool info` ✅ (via loader)

- [ ] Should Just Work once the loader handles MSCZ — verify with a sample
  file that title, tracks, tuning and tempo print correctly
- [ ] Add MSCZ-specific fields to `--verbose`: MuseScore version, style
  fingerprint hash, has-thumbnail flag

### 3.3 `score_tool convert`

- [ ] Extend the input auto-detect table with `.mscz`
- [ ] Extend the output `--format` enum with `mscz`
- [ ] `--report` prints the loss report from § 2.5
- [ ] End-to-end tests: `gp5 → mscz`, `mscz → gp5`, `mscz → musicxml`,
  `musicxml → mscz`, `mscz → mscz` (identity)

### 3.4 `score_tool repeats` / `form` / `fingering` / `extract`

- [ ] Each command already runs on `LoadedScore`; verify each one against
  an MSCZ input and add one integration test per command using a curated
  MSCZ fixture

### 3.5 `score_tool duplicates`

- [ ] Extend the file walker to include `*.mscz` alongside `*.gp*`
- [ ] Duplicate-similarity hashing runs on the optimized model, so no
  algorithm changes are required — just widen the input glob

### 3.6 New sub-command: `score_tool mscz`

- [ ] `score_tool mscz list <file>` — dump archive entry list with sizes
- [ ] `score_tool mscz extract <file> <dir>` — expand archive to disk
  (useful when debugging conversion issues without unzipping manually)
- [ ] `score_tool mscz thumbnail <file> --out <png>` — extract the embedded
  thumbnail

---

## Part 4 — Web Server Integration (`web_server/`)

### 4.1 Upload / open

- [ ] `POST /api/score/upload` — accept `.mscz` in the multipart handler
  (extend the extension whitelist)
- [ ] `POST /api/score/open` — accept `.mscz` paths under `--root`
- [ ] `LoadedFile` in `state.rs` already stores raw bytes + `LoadedScore`;
  no shape change needed
- [ ] Bump the per-file size cap to 32 MB (or make it format-specific:
  16 MB GP*, 32 MB MSCZ)

### 4.2 Raw bytes / rendering

- [ ] `GET /api/score/:id/raw` — return MSCZ bytes with
  `Content-Type: application/vnd.recordare.musicxml` … or preferably
  `application/x-musescore` (check what alphaTab expects)
- [ ] **alphaTab does not natively load MSCZ.** Two options:
  - **(a)** Server-side conversion: emit an on-the-fly GP5 or MusicXML
    stream on request (`GET /api/score/:id/render?format=gp5`) and point
    alphaTab at that URL. Recommended default.
  - **(b)** Wait for alphaTab to add MSCZ support (upstream tracking issue);
    document as a known limitation until then.
- [ ] Cache the converted bytes on the `LoadedFile` so repeat renders don't
  reconvert

### 4.3 Metadata & analysis endpoints

- [ ] `GET /api/score/:id/info` — no change; runs against `LoadedScore`
- [ ] All existing `/api/score/:id/analysis/*` endpoints already work on
  `LoadedScore` — verify with MSCZ fixtures and add one integration test
  per analysis endpoint

### 4.4 Download / export

- [ ] `GET /api/score/:id/download?format=mscz` — re-encode via
  `write_mscz` and stream as `application/x-musescore`
- [ ] Add `mscz` to the extract endpoint's `format` enum
  (`POST /api/score/:id/extract`)
- [ ] Filename: `<original>.mscz` with disposition `attachment`

### 4.5 File browser

- [ ] `GET /api/files` — extend the glob to include `*.mscz` alongside
  `*.gp*`
- [ ] Thumbnail preview: when an entry is `.mscz`, expose
  `GET /api/files/thumbnail?path=<mscz>` returning the embedded PNG
  (falls back to a placeholder if the archive has none). Nice-to-have
  polish for the sidebar.

### 4.6 Duplicate scan

- [ ] `POST /api/duplicates` — include `.mscz` in the walker; similarity
  scoring already runs on `LoadedScore` so no changes needed

### 4.7 UI touch-ups

- [ ] Upload dialog: advertise `.mscz` in the accepted-extensions list
- [ ] Format selector in the download menu: add "MuseScore (.mscz)"
- [ ] Toast on load: show a small `MSCZ` badge next to the title in the
  status bar (mirrors the existing GP4/GP5 badges)

---

## Part 5 — Testing & Quality Gate

### 5.1 Fixtures

- [ ] Curate `guitarpro/samples/mscz/` with:
  - Simple monophonic guitar piece
  - Multi-track band arrangement (drums + bass + guitar + vocals)
  - Complex classical piano (chord voicings, tuplets, ties, cross-staff)
  - Guitar-specific: bends, slides, harmonics, palm mute, let ring,
    fret diagrams, capo, alternate tunings
  - Repeat structures: voltas, D.C. al Fine, D.S. al Coda, simile marks
  - Edge cases: empty score, single-measure score, 4-voice measure

### 5.2 Round-trip test matrix

- [ ] `roundtrip_mscz.rs` — MSCZ → LoadedScore → MSCZ byte or semantic
  equality across the fixture set, matching the `GP3: 13/13 ✓` reporting
  style in the existing roundtrip tests
- [ ] Cross-format round trips added to the existing files:
  - `roundtrip_musicxml.rs`: musicxml → mscz → musicxml
  - `roundtrip_optimized.rs`: gp7 → mscz → gp7

### 5.3 CLI tests

- [ ] `cli/tests/mscz_convert.rs` — every conversion path exercised end to end
- [ ] Regression suite: for each fixture, `info`, `repeats`, `form`,
  `fingering`, `extract`, `duplicates` all run without error

### 5.4 Web server tests

- [ ] `web_server/tests/mscz_endpoints.rs` — upload, info, analysis, download,
  file-browser listing, thumbnail preview
- [ ] `axum::test` harness reusing the existing test helpers

### 5.5 Coverage & lint

- [ ] `just coverage-check` stays above 85% including the new modules
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Part 6 — Documentation

- [ ] `guitarpro/CLAUDE.md` — add an MSCZ section describing the `io::mscz`
  and `model::mscz` layout (mirrors the existing GPX / MusicXML sections)
- [ ] Add a `gp-mscz-format` skill under `.claude/skills/` documenting the
  archive layout, MSCX schema quirks, and misc-store keys (`mscz.*`)
- [ ] `README.md` — list `.mscz` in the supported-formats section
- [ ] `docs/Roadmap-web.md` — cross-link this document from Part 2 (File
  Loading)

---

## Open questions / Stretch goals

- MuseScore 3 (`<museScore version="3.x">`) support — feature-gated?
- Embedded audio (`.ogg` playback tracks in MSCZ) — expose via
  `/api/score/:id/audio`?
- MuseScore style file (`.mss`) editing — read-only for now; write-through
  support would let the CLI apply house styles across a folder
- Direct alphaTab MSCZ support (upstream) — remove the server-side
  conversion shim once available
