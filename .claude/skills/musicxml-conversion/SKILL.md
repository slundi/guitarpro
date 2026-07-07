---
name: musicxml-conversion
description: How the guitarpro crate converts between its internal Song model and MusicXML (score-partwise, plain XML). Bidirectional — Song→MusicXML export and MusicXML→Song import. Use when working on MusicXML mapping, adding technique/notation coverage, or fixing round-trip issues. For the Guitar Pro formats themselves see gp-legacy-format / gp-modern-format.
---

# MusicXML conversion (bidirectional)

Converts the internal `Song` model ↔ **MusicXML 4.0, score-partwise, plain XML**
(not compressed `.mxl`). XML lib: **`quick-xml` v0.39** (`de::from_str` for import,
`se::to_string` for export). Currently **CLI-only** — no web_server endpoint exposes it.

## Directions & entry points

- **Export** `Song → MusicXML`: `convert/musicxml/mod.rs` `song_to_score_partwise(&Song)
  -> ScorePartwise` (~L34). Serialized to XML bytes with an XML header in
  `cli/src/command_convert.rs` (~L413).
- **Import** `MusicXML → Song`: `convert/guitarpro/mod.rs` `musicxml_to_legacy_song(
  &ScorePartwise) -> Song` (~L42). Parsed via `quick_xml::de::from_str` in
  `command_convert.rs` (~L406).
- The `ScorePartwise` struct is defined in `model/musicxml/mod.rs` (~L127).
- CLI auto-detects `.xml` / `.musicxml` extensions (`command_convert.rs` ~L52).

## The critical constant: DIVISIONS

`DIVISIONS = DURATION_QUARTER_TIME` (= 960 ticks/quarter), so legacy tick values map
1:1 to MusicXML `<divisions>` — this is what makes round-trip faithful. Defined in
`convert/musicxml/mod.rs` (~L19). Don't change one side without the other.

## File responsibilities (export side)

| File | Handles |
|---|---|
| `convert/musicxml/note.rs` | Voice → notes; `<backup>` to rewind cursor between voices; MIDI→pitch; rests; chord flag (`is_chord = note_idx>0`); dotted/tuplet timing |
| `convert/musicxml/measure.rs` | first-measure `<attributes>` (clef, key fifths+mode, time sig, staff count = notation + TAB, tunings); tempo `<direction>/<metronome>` (first measure of first track); repeat barlines |
| `convert/musicxml/notations.rs` | GP effects → `<notations>`: fret/string, hammer-on, bend (`bend-alter`), let-ring, palm-mute (`half-muted`), natural harmonic, slides, vibrato (`wavy-line`), articulations (staccato/accent/strong-accent) |
| `convert/musicxml/metadata.rs` | `<identification>` creators (artist→composer, author→arranger, words→lyricist, writer→transcriber), rights, `<miscellaneous>` fields; `<part-list>` per track (P1, P2, …; MIDI program 0→1-based; volume/pan scaling; percussion) |
| `convert/musicxml/helpers.rs` | `midi_to_pitch` (sharps; MIDI 60=C4; octave = midi/12 − 1); `duration_to_divisions`; `duration_to_note_type` (1→Whole … 128→128th) |

Import (`convert/guitarpro/mod.rs`) inverts all of the above — metadata (~L63),
channels (~L113), tempo from `<sound tempo>` (~L150), headers/key/time/repeat (~L177),
tunings from `<staff-tuning>` (~L357), voices via `<backup>` grouping (~L433), notes
preferring explicit `<technical><fret>/<string>` over pitch-to-fret (~L545).

## Conventions that must stay symmetric

- **String order**: GP strings run high→low pitch; MusicXML staff tunings run line 1 =
  lowest. Both directions `.rev()` — keep them consistent (`measure.rs` ~L62,
  `guitarpro/mod.rs` ~L379).
- **Octave**: MIDI 60 = C4, octave = `midi/12 − 1`.
- **Chords**: export sets `is_chord` for note index > 0; import groups notes at same
  voice+onset.
- **Polyphony**: multiple voices encoded sequentially with `<backup>` cursor rewind.
- **Pitch spelling**: uses sharps (alter = +1), never flats.
- **Bend**: 100 bend units = 1 semitone → `bend-alter` in semitone quarters.

## Known limitations (as of this writing — verify before relying)

- Voice count effectively **capped at 2** per measure (`note.rs` ~L25).
- **Grace notes skipped** on import (`guitarpro/mod.rs` ~L457).
- **Pull-offs not encoded** (`technical.pull_off` always `None`); only hammer-on.
- Sparse articulations (staccato/accent/strong-accent only); **no dynamics/velocity**,
  no trills/turns, no forward ties/slurs (only tie-stop), no `.mxl`.
- Percussion is marked unpitched but not deeply GM-drum-mapped.

## Tests

`guitarpro/src/tests/roundtrip_musicxml_legacy.rs` covers both directions.

## When editing

1. Adding a technique → touch `notations.rs` (export) AND the note-import path in
   `convert/guitarpro/mod.rs` so round-trip holds; add to the roundtrip test.
2. Timing changes → respect `DIVISIONS`; use the `helpers.rs` duration fns.
3. Match MusicXML element/attribute names exactly (serde rename); it's score-partwise.
4. Prefer `GpResult`/errors over `unwrap()`/`expect()` (project standard).
