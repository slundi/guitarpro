# score_tool (CLI)

`score_tool` is the command-line interface for `guitarpro`. It inspects and
processes Guitar Pro (`.gp3`, `.gp4`, `.gp5`, `.gpx`, `.gp`) and MuseScore
(`.mscz`) files.

## Installation

From the root project directory:

```bash
cargo build -p cli
```

## Sub-commands

| Command | Purpose |
|---|---|
| `info` | Print metadata, tracks, and timeline |
| `convert` | Convert between formats (gp3/gp4/gp5/gpx/gp/musicxml/mscz/score) |
| `extract` | Extract selected tracks into a new score |
| `duplicates` | Find near-duplicate score files in a directory |
| `repeats` | Analyse repeat structures and simile marks |
| `form` | Detect musical form (verse/chorus/bridge/…) |
| `fingering` | Compute left-hand guitar fingering |
| `mscz list / extract / thumbnail` | Inspect and unpack MSCZ archives |

Run `score_tool <command> --help` for per-command options.

## Usage

```bash
# Basic inspection (any supported format)
cargo run -p cli -- info -i path/to/file.gp5
cargo run -p cli -- info -i path/to/file.mscz

# Convert MSCZ ↔ Guitar Pro / MusicXML
cargo run -p cli -- convert -i song.mscz -o song.musicxml
cargo run -p cli -- convert -i song.gp5  -o song.mscz

# Peek inside a MuseScore archive
cargo run -p cli -- mscz list -i song.mscz
cargo run -p cli -- mscz thumbnail -i song.mscz --out cover.png
cargo run -p cli -- mscz extract -i song.mscz -o ./unpacked
```

## Supported formats

Inputs are auto-detected by extension. Legacy Guitar Pro files (`.gp3` /
`.gp4` / `.gp5`) are additionally probed for the `FICHIER GUITAR PRO`
magic string so files with the wrong extension still parse. MSCZ files
are recognised either by extension or by the ZIP magic + `META-INF/container.xml`
manifest.

Per-format size caps: **16 MB** for legacy GP, **32 MB** for MSCZ.

## Planned Features

- [ ] Export to JSON/CSV for data analysis.
- [ ] Search for specific patterns (chords, sequences).
- [ ] Transposition and tuning adjustment.
