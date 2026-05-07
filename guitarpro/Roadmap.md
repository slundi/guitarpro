# Roadmap

This document outlines the development stages and future goals of the project.

### Core Architecture
- [x] Refactor into `model`, `io`, and `audio` modules.
- [x] Convert `impl Song` blocks into specialized traits.
- [x] Improve GP5 parsing (complex directions).
- [x] Better error management: Replace `expect`/`unwrap` with `thiserror`.
- [ ] Transition model toward [MNX](https://w3c-cg.github.io/mnx/docs/) compatibility.

### Format Support
- [x] GP3, GP4, GP5: Stable reading/writing.
- [ ] GP6 (.gpx) support, rewrite tests
- [ ] Stabilize GP7 reading.
- [ ] Full chord and rhythm support for GP6/7.
- [ ] Write support for modern formats (.gp, .gpx).
- [ ] MusicXML read support (version 1.0 to 4.0) using crate quick-xml
- [ ] MusicXML write support
- [ ] MNX read support
- [ ] MNX write support

### Documentation & Tooling
- [ ] Comprehensive documentation of the internal data model.
- [ ] Improved ASCII tablature rendering in the CLI.
- [ ] Fuzz testing for binary parsers.
