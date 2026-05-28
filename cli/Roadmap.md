# score_tool CLI Roadmap

## Current State

- Read GP3/GP4/GP5/GPX/GP7 files
- Print song metadata (title, artist, album, tracks, …)
- Render first track as ASCII tablature
- Format auto-detection by file extension

The current CLI uses a flat `--action` flag. The roadmap assumes migration to **clap subcommands** as features grow.

---

## Subcommand Architecture

```
score_tool <SUBCOMMAND> [OPTIONS]

Subcommands:
  info        Print metadata and track listing
  tab         Render ASCII tablature
  convert     Convert between formats
  extract     Extract one or more tracks to a new file
  duplicates  Find duplicate/near-duplicate files in a collection
  repeats     Detect repeat structures in a score
  form        Detect musical form (verse/chorus/bridge/…)
  fingering   Compute and annotate guitar fingering on tabs
```

---

## Feature Roadmap

### 1. `info` — Metadata Inspection (refactor of current default)

Expose the current metadata print as a proper subcommand.
Implemented in `cli/src/command_info.rs`.

- [x] `--json` flag: machine-readable output
- [x] Per-track detail: instrument, tuning, string count, measure count, voice count
- [x] Show time signatures, key signatures, tempo map
- [x] Show markers and navigation events (repeats, jumps)

---

### 2. `convert` — Format Conversion

Convert between all supported formats.
Implemented in `cli/src/command_convert.rs`.

**Supported formats:**
- Guitar Pro: GP3, GP4, GP5, GPX, GP7+
- MusicXML (`.xml`, `.musicxml`)
- Optimized Score (`.score` — JSON-serialized `Score` model)

**Usage:**
```
score_tool convert --input song.gp5 --output song.musicxml
score_tool convert --input song.xml --output song.score
score_tool convert --input dir/ --output out_dir/ --to gp5   # batch
```

- [x] Single-file conversion with auto-detected output format from extension
- [x] Explicit `--from` / `--to` format flags for ambiguous cases
- [x] Batch conversion: accept an input directory, convert all matching files
- [x] `--dry-run`: list what would be converted without writing
- [ ] `.score` binary format (currently JSON; migrate to `postcard` or custom binary)

---

### 3. `extract` — Extract Tracks

Produce a new score file containing only a subset of the original tracks.
Implemented in `cli/src/command_extract.rs`.

**Usage:**
```
score_tool extract --input song.gp5 --tracks "Guitar,Bass" --output guitar_bass.gp5
score_tool extract --input song.gp5 --track-index 0,2 --output out.xml
```

- [x] Select tracks by name (substring match, case-insensitive)
- [x] Select tracks by 0-based index
- [x] Output format can differ from input (delegates to `convert` internally)
- [x] Preserve global metadata, tempo map, markers, and time/key signatures
- [x] `--invert`: keep all tracks *except* the selected ones

---

### 4. `duplicates` — Find Duplicate Files

Scan a directory of score files and report probable duplicates.
Implemented in `cli/src/command_duplicates.rs`.

**Usage:**
```
score_tool duplicates --dir ~/tabs/
score_tool duplicates --dir ~/tabs/ --threshold 0.90
```

**Detection strategy (layered):**
1. Exact hash match after normalising metadata (title-independent binary compare)
2. Metadata match: same title + artist + approximate duration
3. Content similarity: compare note/beat sequences across tracks (edit distance or fingerprinting)

- [x] Report groups of duplicates with similarity score
- [x] `--threshold <0..1>`: tune similarity cutoff
- [x] `--json`: machine-readable output
- [x] `--delete-keep-first`: interactive or automatic deduplication (destructive — confirm prompt)
- [x] Recurse into subdirectories with `--recursive`

---

### 5. `repeats` — Detect Repeat Structures

Analyse repeat and simile marks in a score.
Implemented in `cli/src/command_repeats.rs`.

**Two levels of detection:**

#### 5a. Global repeats
All instruments are playing the same repeated section simultaneously (standard repeat barlines, DS al Coda, Da Capo, Coda, Fine, …).

- [x] Parse and list all navigation events (repeat open/close, jump targets) from the score
- [x] Expand the repeat map into a flat play-order sequence of measure ranges
- [x] Report total sounding duration vs. written measure count

#### 5b. Per-instrument simile marks
Single-instrument repeat shorthand within a part.

| Symbol | Meaning |
|--------|---------|
| `%`    | Repeat previous beat |
| `%%`   | Repeat previous two beats |
| `%%%%` | Repeat previous bar |

- [x] Detect measures/beats annotated with simile marks in the optimized model (`MeasureRepeat`, `SimileMark`)
- [x] Report per-track where simile marks appear and what they reference
- [x] `--expand`: emit a version of the score with all simile marks replaced by actual notes (for analysis or export)

---

### 6. `form` — Detect Musical Form

Identify recurring structural sections within each track (verse, chorus, bridge, intro, outro, …) based on note-sequence similarity.
Implemented in `cli/src/command_form.rs`.

**Usage:**
```
score_tool form --input song.gp5
score_tool form --input song.gp5 --track "Rhythm Guitar"
```

**Algorithm outline:**
1. Segment the track into candidate sections (boundary hints: markers, repeat signs, large rests, tempo changes)
2. Build a similarity matrix between all pairs of segments (pitch-class sequence + rhythmic profile)
3. Cluster similar segments → label them A, B, C, … (or user-supplied names like verse/chorus)
4. Detect variations (chorus vs. chorus 2): same harmonic skeleton, different ornaments or dynamics → label A, A', A''

**Output:**
```
Track: Rhythm Guitar
Form:  [Intro A] [Verse B] [Chorus C] [Verse B] [Chorus C] [Bridge D] [Chorus C'] [Outro A']
```

- [x] Measure-range output per section (e.g. `Chorus C: bars 17–24, 33–40, 49–56`)
- [x] `--json`: structured output for downstream processing
- [ ] Integration with `extract`: `--form chorus` extracts only chorus measures

---

### 7. `fingering` — Guitar Fingering Computation

Compute and display left-hand fingering assignments for guitar tab tracks.

**Usage:**
```
score_tool fingering --input song.gp5 --track "Lead Guitar"
score_tool fingering --input song.gp5 --annotate --output annotated.score
```

**Algorithm (using `guitarpro::analysis::fingering`):**
1. For each note, determine fret position and string
2. Apply position-window heuristic: prefer staying in a contiguous 4-fret window
3. Assign fingers 1–4 (index → pinky) minimising total hand movement (dynamic programming on the beat sequence)
4. Handle open strings (finger 0) and muted strings

**Output options:**
- [ ] ASCII tab with finger numbers below each fret number
- [ ] `--annotate`: write finger assignments into the optimized model's `Finger` field and save to `--output`
- [ ] `--position <N>`: force starting position (capo / manual override)
- [ ] Respect existing fingering annotations already present in the source file (GP5 has finger data per note)

---

## Non-Feature Work

- [x] Migrate `main.rs` to proper clap subcommands (breaking the current flat `--action` flag)
- [ ] Add `--quiet` / `--verbose` global flags backed by `tracing`
- [ ] Error reporting: replace `eprintln! + process::exit` with `anyhow` + `thiserror`
- [ ] Integration tests in `cli/tests/` using small fixture files
- [ ] Shell completion generation (`score_tool completions bash/zsh/fish`)
