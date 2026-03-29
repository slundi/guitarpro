# Guitar Pro Tools

A comprehensive suite of tools for parsing, manipulating, and visualizing Guitar Pro files in Rust.

## Project Structure

This workspace is divided into several crates:

- **[lib](lib/README.md)** (`scorelib`): The core library to read and write **Guitar Pro** files (GP3, GP4, GP5, GPX, GP7) and **MuseScore** files (MSCZ). It provides a unified data model for musical scores.
- **[cli](cli/README.md)** (`score_tool`): A command-line interface to inspect files, view metadata, and generate ASCII tablatures.
- **web_server**: (Experimental) A web server to search and browse music scores through an API.

## Features

- **Multi-format support**: Read GP3, GP4, GP5, and early support for GP6/7 (.gp, .gpx).
- **Rich Data Model**: Exhaustive representation of tracks, measures, beats, notes, and musical effects.
- **ASCII Visualization**: Generate text-based tablatures directly from the CLI.
- **Extensible Architecture**: Module-based design with traits for easy extension.

## Usage

To get started with the CLI, run:

```bash
cargo run -p cli -- --input path/to/song.gp5 --tab
```

## Roadmap

### Library
- [x] Refactor core into `model`, `io`, and `audio` modules.
- [x] Comprehensive trait-based API for `Song` operations.
- [x] High-fidelity GP5 parsing.
- [x] Initial support for GP6/7 (.gp/.gpx) formats.
- [ ] Improved MuseScore (.mscz) support.
- [ ] Full RSE (Realistic Sound Engine) data parsing.
- [ ] Export to MIDI/Audio.

### CLI
- [x] Basic metadata inspection.
- [x] ASCII Tablature generation.
- [ ] Batch conversion tool.
- [ ] Advanced search and filtering.

## License

This project is licensed under the MIT License.
