# Guitar Pro Tools

A comprehensive suite of tools for parsing, manipulating, and visualizing Guitar Pro files in Rust.

## Project Structure

This workspace is divided into several crates:

- **[lib](guitarpro/README.md)** (`guitarpro`): The core library to read and write **Guitar Pro** files (GP3, GP4, GP5, GPX, GP7) and **MuseScore** files (MSCZ). It provides a unified data model for musical scores.
- **[cli](cli/README.md)** (`score_tool`): A command-line interface to inspect files, view metadata, and generate ASCII tablatures.
- **web_server**: (Experimental) A web server to search and browse music scores through an API.

## Supported formats

| Format | Read | Write | Notes |
|---|---|---|---|
| `.gp3` / `.gp4` / `.gp5` | ✅ | ✅ | High-fidelity legacy binary |
| `.gpx` (GP6) | ✅ | ✅ | BCFZ/BCFS container |
| `.gp` (GP7+) | ✅ | ✅ | ZIP container with GPIF XML |
| `.mscz` (MuseScore 4.x) | ✅ | ✅ | ZIP container with MSCX XML — see [`docs/Roadmap-mscz.md`](docs/Roadmap-mscz.md) |
| MusicXML (`.xml` / `.musicxml`) | ✅ | ✅ | Score-partwise and timewise |

## Features

- **Multi-format support**: GP3/GP4/GP5, GP6 (`.gpx`), GP7+ (`.gp`), MuseScore (`.mscz`), MusicXML.
- **Rich Data Model**: Exhaustive representation of tracks, measures, beats, notes, and musical effects.
- **ASCII Visualization**: Generate text-based tablatures directly from the CLI.
- **Extensible Architecture**: Module-based design with traits for easy extension.

## Usage

To get started with the CLI, run:

```bash
cargo run -p cli -- --input path/to/song.gp5 --tab
```

Inspect a MuseScore file:

```bash
cargo run -p cli -- info -i path/to/song.mscz
cargo run -p cli -- mscz list -i path/to/song.mscz
```

## Roadmap

### Library
- [x] Refactor core into `model`, `io`, and `audio` modules.
- [x] Comprehensive trait-based API for `Song` operations.
- [x] High-fidelity GP5 parsing.
- [x] Initial support for GP6/7 (.gp/.gpx) formats.
- [x] MuseScore (.mscz) read / write / conversion — see [`docs/Roadmap-mscz.md`](docs/Roadmap-mscz.md).
- [ ] Full RSE (Realistic Sound Engine) data parsing.
- [ ] Export to MIDI/Audio.

### CLI
- [x] Basic metadata inspection.
- [x] ASCII Tablature generation.
- [x] Batch conversion (`score_tool convert`).
- [x] MSCZ archive tooling (`score_tool mscz list / extract / thumbnail`).
- [ ] Advanced search and filtering.

## Documentation

- [Web-server roadmap](docs/Roadmap-web.md)
- [MSCZ (MuseScore) roadmap](docs/Roadmap-mscz.md)

## License

This project is licensed under the MIT License.
