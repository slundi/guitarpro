# score_tool (CLI)

`score_tool` is the command-line interface for `scorelib`. It allows you to quickly inspect Guitar Pro files, view their metadata, and visualize them as ASCII tablature.

## Installation

From the root project directory:

```bash
cargo build -p cli
```

## Usage

```bash
# Basic inspection (metadata only)
cargo run -p cli -- --input path/to/file.gp5

# Generate ASCII tablature for the first track
cargo run -p cli -- --input path/to/file.gp5 --tab
```

## Options

- `--input <FILE>` (or `-i`): **(Required)** Path to the Guitar Pro file (.gp3, .gp4, .gp5, .gp).
- `--tab` (or `-t`): Display the first track as ASCII tablature in the terminal.

## Current Infrastructure

The CLI currently supports:
- **Metadata extraction**: Title, Artist, Album, Author, Version, etc.
- **ASCII Rendering**: Responsive text-based tablature alignment.
- **Format Auto-detection**: Based on file extension.

## Planned Features

- [ ] Batch processing of directories.
- [ ] Export to JSON/CSV for data analysis.
- [ ] Search for specific patterns (chords, sequences).
- [ ] Transposition and tuning adjustment.
- [ ] Conversion between Guitar Pro versions.
