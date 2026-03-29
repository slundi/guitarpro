# Ideas

## CLI

* [ ] `-l` Load bellow parameters from a file (YAML?, JSON?, other?)
* [ ] rework CLI with commands (find, info, export, edit, check, repair, convert, ...) and options (global ones: verbose, help, json to display output result?)

### Data Extraction & Search

* [ ] `-f` find in files

#### Metadata Filtering

* Current: Search by artist, title, album.
* Improvement: Add version detection. Being able to filter for specifically .gp5 vs .gpx files is critical because the underlying data structures (binary vs. XML) are fundamentally different.

* [ ] song information (artist, title, album, ...) with wildcard and regexes

#### Musical content search

* Current: Search by tuning, tempo, and note sequences.
* Improvement: Fuzzy Note Matching. Instead of just matching |A|E|CC|, allow a "transposition-aware" search that finds the same interval pattern regardless of the starting key.


* [ ] instrument
* [ ] `-ft <value>` [tuning](https://en.wikipedia.org/wiki/List_of_guitar_tunings) (value example: `E-A-D-g-b-e`, `EADgbe`, ` E2–A2–D3–G3–B3–E4`, )
* [ ] range (string count, piano keys count, drum elements, ...)
* [ ] `-f? <value|range>` tempo: is constant, range if variable, min, max, ... (example: `80`, `60-90`, `60>`, `<120` or `0-120`, `60,80,90` or `80,70-85` for a list)
* [ ] `-fb <beat>`beats
* [ ] `-fn <notes>` notes (example: `DADDC` anywhere in the track, `|DADDC|` in a mesure, `|A|E|CC|` measures with those notes)
* [ ] `-fr` Repetitions:
* [ ] `-frm <1|2|4>` [same mesures](https://musescore.org/en/handbook/4/measure-and-multi-measure-repeats)
* [ ] `-frs` [repeat signs](https://musescore.org/en/handbook/4/repeat-signs)
* [ ] `-frv` [voltas](https://musescore.org/en/handbook/4/voltas)
  * [ ] `-fv 0.8` detect verse with a similarity percentage

#### Extraction formats

* Current: Export to CSV and JSON.
* Improvement: Add MusicXML Export. This is the industry standard for interoperability between different notation softwares (like MuseScore or Sibelius) and is often used as a "bridge" for comparing files.

* [ ] `-x` Extract:
  * [ ] `-xi <format>` above information in various format (CSV, JSON)
  * [ ] `-xt <number|instrument> <format>` tracks
  * [ ] `-xl <format>` lyrics
* [ ] `-c format` Conversion between formats with alerts when information are lost (like GP5 -> GP3)
* [ ] `-r` Replace repetitions 
  * [ ] `m` [same mesures](https://musescore.org/en/handbook/4/measure-and-multi-measure-repeats)
  * [ ] `s` repeat signs and `v` voltas when mesures are the same for all tracks
* [ ] `-t -0.5` Change tuning if possible
* [ ] `-ts x2` Divide/multiply time signatures (I had once an guitar tab that needed to be rewritten by changing the time signature and the beams)
* [ ] `-p` Apply page format parametters (margin, spacing, ...)

## Core Library Architecture (Structural Improvements)

To make your it capable of "writing newer ones" reliably, the internal structure needs to be highly organized.

* Unified Object Model (UOM):
  * Idea: Instead of separate logic for every version, create a single "Master Score" object.
  * Reasoning: A GP3 file and a GP8 file both have "Tracks" and "Measures." If you map them to a shared model, you can read a GP3 file and save it as a GP5 (or vice versa) without rewriting your entire logic.
* Validation Engine:
  * Idea: Implement a "Sanity Checker" before writing.
  * Details: Ensure that the total duration of notes in a measure matches the Time Signature. Check that no notes are placed on string 7 if the track is defined as a 6-string guitar.
* Lossless Round-tripping:
  * Idea: A testing suite where you Read -> Write -> Read and ensure the data remains identical.
  * Critical Data: Many libraries lose "non-musical" data like Page Layout (margins, font sizes) or Stylesheet settings. Keeping these intact is what makes a library "Pro."

## Notation & Guitar Techniques (The "Deep" Data)

Guitar Pro's complexity lies in its "Effects" byte-maps. To fully support "writing" files, the library must handle these accurately.

* Automation Events:
  * Current: Tempo is mentioned.
  * Improvement: Support Master Track Automations. This includes volume swells, pan changes, and tempo ramps (accelerando/ritardando) that happen mid-song.
* Bends and Whammy Points:
  * Idea: Treat Bends as a collection of "Graph Points."
  * Details: A bend isn't just a "full" or "half" tag; it’s a series of points (time vs. pitch). If you don't write these points correctly, the bend will either not sound or look broken in the Guitar Pro editor.
* Chord Library Management:
  * Idea: Don't just store chord names; store the Fingering Object.
  * Details: Guitar Pro files contain a specific "Chord Table" at the end of the file. Properly writing this table is what allows the "Chord Diagrams" to appear at the top of the sheet music.

## Complexity

| Category | Feature | Complexity |
|:---------|:--------|:-----------|
| CLI | Regex Search for Artist/Title | Low |
| CLI | Note Sequence Search (-fn) | Medium |
| Lib | GP3/4/5 Binary Parser | Medium |
| Lib | GPX (XML) Compressed Parser | High |
| Lib | Bend/Whammy Graph Writer | High |
| Extraction | JSON/CSV Dump | Low |
| Extraction | MusicXML Converter | Medium |
