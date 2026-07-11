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

## Part 3 — CLI Integration (`cli/`) ✅

The MSCZ path is wired through `load_song` (`cli/src/loader.rs`), which
detects `.mscz` by extension or by the `PK\x03\x04` magic + presence of
`META-INF/container.xml` and dispatches to
`read_mscz_bytes → mscx_to_loaded_score → loaded_score_to_legacy_song`.
Every existing command (`info`, `convert`, `repeats`, `form`, `fingering`,
`extract`, `duplicates`) therefore accepts `.mscz` inputs without any
per-command change. A new `score_tool mscz` sub-command exposes
container-level tooling. All 94 real-world MuseScore 4.0–4.6 files parse
via `score_tool info`.

### 3.1 Loader ✅

- `cli/src/loader.rs` — `load_song` detects `.mscz` by extension **or**
  by ZIP magic (`PK\x03\x04`) + `META-INF/container.xml` probe, then
  reads via `guitarpro::io::mscz::read_mscz_bytes` and bridges through
  `LoadedScore → Song` for downstream commands
- Per-format size caps: 16 MB for legacy GP, 32 MB for MSCZ

### 3.2 `score_tool info` ✅

- [x] Works via the loader; `MSCZ` is reported as its own format label
  (`MuseScore (MSCZ)`) — verified against the full corpus (94/94 parsed)
- [ ] MSCZ-specific `--verbose` fields (MuseScore version, style hash,
  has-thumbnail flag) — deferred until anyone needs them

### 3.3 `score_tool convert` ✅

- Input auto-detect table includes `.mscz`
- Output `--format` enum accepts `mscz` (`Format::Mscz`)
- End-to-end conversion paths:
  - MSCZ → MSCZ (identity via the archive builder in `command_convert.rs`)
  - MSCZ → MusicXML (via `LoadedScore` → `Song` → `song_to_score_partwise`)
  - MSCZ → GP5 (via the same bridge; note: page-setup defaults from
    `loaded_score_to_legacy_song` are lossy — captured as Part 2 gap)
- [ ] `--report` flag surfacing `LossReport` — deferred; the report is
  already exposed programmatically through `ConvertOutcome.report`

### 3.4 `score_tool repeats` / `form` / `fingering` / `extract` ✅

- Each command runs unchanged on MSCZ inputs thanks to the loader
  bridge; `repeats` verified against real corpus (surfaces `|:`/`:|`
  navigation markers extracted from `<startRepeat>` / `<endRepeat>`)

### 3.5 `score_tool duplicates` ✅

- `GP_EXTENSIONS` widened to include `mscz`; fingerprint hashing
  runs on `Song` (produced by the same loader bridge), so no algorithm
  change was needed

### 3.6 New sub-command: `score_tool mscz` ✅

Implemented in [`cli/src/command_mscz.rs`](../cli/src/command_mscz.rs):

- `score_tool mscz list -i <file>` — dump archive entries with sizes
  (also `--json` for scripting)
- `score_tool mscz extract -i <file> -o <dir>` — expand archive to
  disk with ZIP-slip protection (rejects absolute paths and `..`
  components)
- `score_tool mscz thumbnail -i <file> [--out <png>]` — extract the
  embedded `Thumbnails/thumbnail.png` to disk

---

## Part 4 — Web Server Integration (`web_server/`) ✅

Backend endpoints now accept `.mscz` end to end. The MSCZ path funnels
through `state::session_from_bytes` (shared by both upload and open),
which detects the extension, applies a per-format size cap (16 MB GP /
32 MB MSCZ), reads the archive via `read_mscz_bytes`, converts through
`LoadedScore → Song`, and hangs on to the embedded PNG thumbnail so
downstream endpoints don't have to re-read the archive. The library
target `web_server` was introduced (`src/lib.rs`) so integration tests
can drive the real `api_routes()` router.

Frontend UI polish (4.7) is unchanged — deferred to a later
frontend-focused pass since it doesn't affect the API contract.

### 4.1 Upload / open ✅

- [x] `POST /api/score/upload` — multipart handler accepts `.mscz` (extension
  whitelist extended to include `mscz`)
- [x] `POST /api/score/open` — accepts `.mscz` paths under `--root`
- [x] `LoadedFile` gained a `thumbnail: Option<Vec<u8>>` field so uploaded
  MSCZ archives keep their embedded PNG for later serving
- [x] Per-format size caps: `MAX_FILE_SIZE = 16 MB` (legacy), new
  `MAX_MSCZ_FILE_SIZE = 32 MB` (`max_size_for(ext)` helper)

### 4.2 Raw bytes / rendering

- [x] `GET /api/score/:id/raw` — returns the original MSCZ bytes verbatim
  (attachment download; alphaTab renderers can fetch this URL if they add
  MSCZ support upstream)
- [ ] Server-side render shim (`/api/score/:id/render?format=gp5|musicxml`)
  for browsers that can't load MSCZ directly — deferred; the `download`
  endpoint already fills this role manually
- [ ] Cache of converted bytes — deferred (no measured hit yet)

### 4.3 Metadata & analysis endpoints ✅

- [x] `GET /api/score/:id/info` — unchanged, runs against `LoadedScore`;
  integration test asserts title, tuning and MIDI numbers survive an
  MSCZ upload
- [x] `/api/score/:id/analysis/repeats` verified to return valid JSON on
  an MSCZ session (loader bridge covers all analysis endpoints identically)

### 4.4 Download / export ✅

- [x] `GET /api/score/:id/download?format=mscz` — re-encodes via
  `LoadedScore → MSCX → MSCZ` and streams as an attachment. The archive
  contains `META-INF/container.xml` + `score.mscx`.
- [x] `POST /api/score/:id/extract` — `ExtractFormat::Mscz` added; same
  archive shape as the download path
- [x] Filename: `<stem>.{mscz,gp5,gpx}` with `Content-Disposition: attachment`

### 4.5 File browser ✅

- [x] `GET /api/files` — walker already used `SUPPORTED_EXTENSIONS`; MSCZ
  now shows up automatically alongside GP files
- [x] `GET /api/files/thumbnail?path=<mscz>` — reads the archive on disk
  and serves the embedded PNG. Rejects non-MSCZ paths (`404`) and paths
  outside `--root` (`403`)
- [x] `GET /api/score/:id/thumbnail` — serves the PNG cached from the
  session (populated when the source was `.mscz` and the archive shipped
  one); `404` when unavailable

### 4.6 Duplicate scan ✅

- [x] `POST /api/duplicates` — the fingerprint walker calls `parse_song`,
  which now dispatches `.mscz` to the MSCZ→Song bridge automatically; no
  code change beyond the whitelist extension

### 4.7 UI touch-ups

- [ ] Upload dialog: advertise `.mscz` — deferred (frontend pass)
- [ ] Format selector in the download menu: add "MuseScore (.mscz)" —
  deferred (frontend pass)
- [ ] Toast on load: show a small `MSCZ` badge next to the title —
  deferred (frontend pass)

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

## Part 6 — Documentation ✅

- [`guitarpro/CLAUDE.md`](../guitarpro/CLAUDE.md) — new **MSCZ**
  section mapping every module path (`io/mscz/`, `model/mscz/`,
  `convert/mscz/`) with a two-line description of what each file owns
- [`.claude/skills/gp-mscz-format/SKILL.md`](../.claude/skills/gp-mscz-format/SKILL.md)
  — full skill covering archive layout, MSCX parser structure, MSCX ↔
  `LoadedScore` conversion (both directions), gotchas (0-based string
  numbers, tempo bps↔bpm, ZIP magic vs. `.gp`), and testing conventions
- Root [`README.md`](../README.md) — supported-formats table now
  lists `.mscz`, roadmap tick moved to ✅, docs section links to the
  MSCZ roadmap
- [`guitarpro/README.md`](../guitarpro/README.md) — quick-start snippet
  showing `read_mscz` + `mscx_to_loaded_score` and its `LossReport`
- [`cli/README.md`](../cli/README.md) — sub-command matrix now lists
  `mscz list / extract / thumbnail` and the extension detection notes
- [`docs/Roadmap-web.md`](./Roadmap-web.md) Part 2 cross-links back
  to this document
- `guitarpro/src/tests/mscz_docs.rs` — 10 tests that assert every
  documented file/section exists, the skill carries valid frontmatter,
  and referenced module paths still exist on disk (guards against doc
  drift when files get renamed)

---

## Open questions / Stretch goals

- MuseScore 3 (`<museScore version="3.x">`) support — feature-gated?
- Embedded audio (`.ogg` playback tracks in MSCZ) — expose via
  `/api/score/:id/audio`?
- MuseScore style file (`.mss`) editing — read-only for now; write-through
  support would let the CLI apply house styles across a folder
- Direct alphaTab MSCZ support (upstream) — remove the server-side
  conversion shim once available
