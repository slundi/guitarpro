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
- [x] GP6 (.gpx) support, rewrite tests
- [x] Stabilize GP7 reading.
- [ ] Full chord and rhythm support for GP6/7. (rhythm done; chord diagrams not parsed from GPIF yet)
- [x] Write support for modern formats (.gp, .gpx).
- [x] MusicXML read support (version 1.0 to 4.0) using crate quick-xml
- [x] MusicXML write support
- [ ] MNX read support
- [ ] MNX write support

### Conversion

- [x] `legacy::Song` to `optimized::LoadedScore` in `guitarpro/src/convert/optimized/`
- [x] `musicxml::ScorePartwise` to `optimized::LoadedScore` in `guitarpro/src/convert/optimized/`
- [x] `musicxml::ScoreTimewise` to `optimized::LoadedScore` in `guitarpro/src/convert/optimized/`
- [x] `optimized::LoadedScore` to `legacy::Song` in `guitarpro/src/convert/legacy/`
- [x] `musicxml::ScorePartwise` to `legacy::Song` in `guitarpro/src/convert/guitarpro/`
- [x] `musicxml::ScoreTimewise` to `legacy::Song` in `guitarpro/src/convert/guitarpro/`
- [ ] roundtrip tests: legacy -> musicxml -> legacy (source and end file must be the same in size and bytes)
- [x] roundtrip tests: legacy -> optimized -> legacy
- [x] roundtrip tests: musicxml -> legacy -> musicxml
- [x] roundtrip tests: musicxml -> optimized -> musicxml
- [x] roundtrip tests: optimized -> legacy -> optimized
- [x] roundtrip tests: optimized -> musicxml -> optimized

### Documentation & Tooling

- [ ] Comprehensive documentation of the internal data model.
- [ ] Improved ASCII tablature rendering in the CLI.
- [ ] Fuzz testing for binary parsers.

---

## Analysis Features (`guitarpro/src/analysis/`)

These features are **pure library functions** — no I/O, no presentation. They take model types
(`Track`, `Measure`, `Beat`, …) and return derived information. The CLI, TUI, and web server
each call into them and handle display independently.

Planned module layout:

```
guitarpro/src/analysis/
    mod.rs
    repeats.rs     — measure-repeat detection and structural segmentation
    chords.rs      — pitch-class set → chord name
    fingering.rs   — fret sequence → finger assignments
    scales.rs      — pitch content → scale / key / mode identification
```

---

### Measure Repeats (`repeats.rs`)

**What it does.** Detects consecutive measures that are musically identical and annotates them
with the standard notation shorthand:

| Symbol | Meaning |
|--------|---------|
| `%`    | Repeat the previous measure |
| `%%`   | Repeat the previous two measures |
| `%%%%` | Repeat the previous four measures |

**Why it matters.** Guitar Pro 6/7 files already carry a `simile_mark` field on `Measure`
(populated by the importer when the file was originally written with those symbols). This
function is the *generator* for files that lack them — GP3/4/5, MusicXML, etc. — and for
verifying that existing marks are correct.

**Algorithm sketch.**

1. Compare consecutive measures structurally: same time signature, same number of beats per
   voice, same note durations, same pitches (fret + string, or MIDI pitch when no tab info is
   available).
2. Prefer larger repeat groups (four measures over two, two over one) when multiple groupings
   are valid.
3. Emit `Vec<Option<SimileMark>>` aligned with the track's measure list; `None` means "no
   shorthand applies here."

**Proposed API.**

```rust
pub enum SimileMark { Single, Double, Quad }
pub fn detect_repeats(track: &Track) -> Vec<Option<SimileMark>>
```

**CLI surface.** `score_tool analyze repeats FILE` — prints a compact measure map.
**Render surface.** ASCII tab renderer can substitute `%` / `%%` / `%%%%` for repeated bars.

---

### Chord Detection (`chords.rs`)

**What it does.** Given the notes sounding simultaneously on a beat, identifies the chord
name (root, quality, optional bass note for slash chords).

**Why it matters.** Guitar Pro stores chord diagrams as separately authored objects. Many
files — especially imports from MusicXML or plain tab — contain no chord annotations at all.
This function derives them from the actual notes.

**Algorithm sketch.**

1. Collect the MIDI pitch of every note in the beat (fret + open-string tuning, or explicit
   pitch when available).
2. Reduce to a **pitch-class set** (pitches mod 12, deduplicated).
3. Match against an interval-vector table covering: major, minor, dominant 7th, major 7th,
   minor 7th, sus2, sus4, diminished, augmented, add9, …
4. Try all 12 roots; pick the match with fewest "extra" pitch classes (closest match first).
5. If no root produces a clean match, return the best-effort name with an "?" flag.

**Proposed API.**

```rust
pub enum ChordQuality { Major, Minor, Dom7, Maj7, Min7, Sus2, Sus4, Dim, Aug, Add9, /* … */ }

pub struct ChordName {
    pub root: PitchClass,       // C, C#, D, …
    pub quality: ChordQuality,
    pub bass: Option<PitchClass>, // for slash chords, e.g. G/B
    pub uncertain: bool,
}

pub fn identify_chord(beat: &Beat, strings: &[(i8, i8)]) -> Option<ChordName>
```

**CLI surface.** `score_tool analyze chords FILE` — prints chord names above each beat that
has more than one note.
**Dependency.** Used by `fingering.rs` to seed hand-shape heuristics.

---

### Key / Scale Guessing (`scales.rs`)

**What it does.** Infers the most likely **key** and **scale** (or mode) from the pitch
content of a segment of music — a measure, a phrase, a track, or the whole score.

**Scales covered (planned).** Major, natural/harmonic/melodic minor, pentatonic major/minor,
blues, Dorian, Phrygian, Lydian, Mixolydian, Locrian, whole-tone, diminished.

**Why it matters.** Knowing the key is a prerequisite for:
- Suggesting correct accidentals / enharmonic spellings when converting to notation.
- Transposing intelligently (preserving scale-degree relationships).
- Generating chord names with correct root spellings (F# vs G♭).
- Annotating exported MusicXML with a valid `<key>` element.

**Algorithm sketch.**

1. Collect all pitch classes used in the segment (weighted by duration and/or note count).
2. Score each (root, scale) pair by coverage: what fraction of the scale's expected pitch
   classes appear, and how many unexpected ones are present?
3. Apply a prior that favours common keys (C major, G major, A minor, …) to break ties.
4. Return a ranked list so the caller can present alternatives.

**Proposed API.**

```rust
pub enum Scale {
    Major, NaturalMinor, HarmonicMinor, MelodicMinor,
    PentatonicMajor, PentatonicMinor, Blues,
    Dorian, Phrygian, Lydian, Mixolydian, Locrian,
    WholeTone, Diminished,
}

pub struct KeyGuess {
    pub root: PitchClass,
    pub scale: Scale,
    pub coverage: f32,   // fraction of expected pitch classes found (0.0–1.0)
    pub purity: f32,     // 1 − (unexpected pitch classes / total pitch classes)
}

/// Returns candidates ordered from most to least likely.
pub fn guess_key(segment: &[&Beat], strings: &[(i8, i8)]) -> Vec<KeyGuess>
```

**Dependency.** Used by `chords.rs` to prefer root spellings consistent with the detected
key.

---

### Key Signature / Armor (`scales.rs` or `armature`)

**What it does.** Derives the formal **key signature** (number of sharps or flats, major or
relative minor) to attach to a score or a section, and tracks modulations when the key
changes mid-song.

**Distinction from scale guessing.** Scale guessing is statistical — it looks at the actual
pitch-class distribution. Key signature derivation is normative — it picks the circle-of-fifths
representation that minimises accidentals for the inferred tonal centre, which may differ
from the mode actually used (e.g., a piece in A Dorian shares the D-major key signature even
though its tonal centre is A).

**Algorithm sketch.**

1. Run `guess_key` per section (or per measure, then smooth).
2. Map `(root, scale)` to the canonical key signature: relative major for minor/modal scales,
   then count fifths offset from C.
3. Detect **modulation boundaries**: a run of measures whose best key differs from the
   current key by more than a threshold triggers a key-change annotation.
4. Emit a timeline of `(measure_index, KeySignature)` pairs for use in notation export.

**Proposed API.**

```rust
pub struct KeyChange {
    pub at_measure: usize,
    pub key: crate::model::legacy::key_signature::KeySignature,
}

pub fn detect_key_changes(track: &Track, strings: &[(i8, i8)]) -> Vec<KeyChange>
```

**CLI surface.** `score_tool analyze key FILE` — prints key signature per section.
**Export use.** The MusicXML exporter can use `detect_key_changes` to emit correct `<key>`
elements in `<attributes>` blocks rather than always writing C major.

---

### Finger Assignment (`fingering.rs`)

**What it does.** Suggests which left-hand finger (index = 1, middle = 2, ring = 3,
little = 4) to use for each fretted note in a sequence.

**Why it matters.** Correct fingering reduces hand movement, avoids awkward stretches, and
makes it easier to anticipate upcoming notes. It is the bridge between raw tablature
(string + fret) and playable notation.

**Constraints modelled.**

- **Span limit.** A comfortable reach is typically 4–5 frets; 5+ frets require a position
  shift.
- **One finger per fret rule.** Within a chord, assign one finger per fret (or share when
  two notes are on the same fret — barre).
- **Barre detection.** When multiple strings share the same fret, emit a barre annotation
  instead of individual finger assignments.
- **Position memory.** The hand should stay in the same position between beats unless a
  shift is necessary (minimise total fret-distance travelled).
- **Look-ahead.** When the next measure is available, bias the current position toward the
  first fret of the next phrase to reduce the cost of the transition.

**Algorithm sketch.**

1. For each beat (or chord), collect (string, fret) pairs.
2. Detect barre candidates (two or more notes at the same fret).
3. Assign the remaining fingers greedily from lowest to highest fret, using the chord-shape
   heuristic seeded by `chords.rs` when available.
4. Between consecutive beats, compute a position-shift cost and adjust the assignment if
   staying in position is cheaper than the "ideal" assignment in isolation.
5. One-measure look-ahead: after assigning the current measure, compute the opening position
   of the next measure and retroactively adjust the last beat if it reduces transition cost.

**Proposed API.**

```rust
pub enum FingerRole { Single, BarreAnchor, BarreMember }

pub struct FingerAssignment {
    pub string: i8,
    pub fret: i16,
    pub finger: u8,        // 1–4
    pub role: FingerRole,
    pub position_shift: bool, // true if this note starts a new hand position
}

/// `measures` is a window: current measure + at least one look-ahead if available.
pub fn suggest_fingering(
    measures: &[&Measure],
    strings: &[(i8, i8)],
) -> Vec<Vec<FingerAssignment>>   // one inner Vec per measure
```

**CLI surface.** `score_tool analyze fingering [--lookahead 1] FILE`
**Dependency.** Calls `chords.rs` for hand-shape hints; reads `scales.rs` for barre-chord
position preferences.

---

### Structural Segmentation — Form Detection (`repeats.rs` or `form.rs`)

**What it does.** Identifies large-scale repetition in a score and assigns formal labels
— A, B, A′, C, … — to sections, producing a compact representation of the song's overall
form (e.g. `A B A B A′ C` for a typical verse-chorus-bridge structure).

**Why it matters.** Structural knowledge enables:
- Smarter compression of the tablature (print choruses once with a "D.S." instead of four
  times).
- Meaningful navigation in a TUI or web player ("jump to chorus").
- Detecting which sections a particular guitar part was adapted for.

**Nuances.**

- **Exact repeats** are easy: identical pitch-class + rhythm sequences → same label.
- **Variations (A′)** are harder: same harmonic skeleton, different surface rhythm, melodic
  ornament, or slight lengthening at the end (codetta). This requires a similarity metric,
  not equality.
- **Partial overlaps.** A transition measure may belong to both the end of A and the start of
  B; the algorithm should allow "overlap" zones rather than forcing a hard boundary.
- **Multi-track.** The song form is typically driven by the harmony (chords) and rhythm
  guitar; lead-guitar and melody may diverge. The analysis should run on a designated
  "reference track" (usually the one with the most chords) and the result applied globally.

**Algorithm sketch.**

1. Segment the track into candidate sections by looking for structural boundaries:
   rehearsal marks, double bar-lines, repeat signs, significant key/time-signature changes,
   or long stretches of silence.
2. For each pair of candidate sections, compute a similarity score:
   - Pitch-class overlap (Jaccard on pitch-class sets per measure).
   - Rhythmic similarity (edit distance on beat-duration sequences).
   - Length ratio (sections that differ wildly in length are unlikely to be the same label).
3. Cluster similar sections: exact matches → same base label; similarity above threshold
   (e.g. 0.85) → variation suffix (A → A′, A″, …).
4. Assign labels in order of first appearance: first unique section = A, next = B, etc.
5. Emit a `Vec<SectionLabel>` with start/end measure indices and the label string.

**Proposed API.**

```rust
pub struct SectionLabel {
    pub start_measure: usize,
    pub end_measure: usize,   // exclusive
    pub label: String,        // "A", "B", "A'", "A''", "C", …
    pub similarity: f32,      // vs. the canonical first occurrence of this label
}

pub fn detect_form(
    track: &Track,
    strings: &[(i8, i8)],
    similarity_threshold: f32,   // e.g. 0.85
) -> Vec<SectionLabel>
```

**CLI surface.** `score_tool analyze form [--threshold 0.85] FILE`
**TUI surface.** A section navigator that lets the user jump between A / B / chorus / bridge.
**Export use.** Can populate rehearsal marks in MusicXML `<direction>` elements.
