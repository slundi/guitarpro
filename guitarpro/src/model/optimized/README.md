# Optimized representation

A Rust library for reading, writing, and processing music notation files (Guitar Pro, MusicXML, MNX), optimized for score rendering, MIDI playback, chord extraction, and guitar fingering computation.

## Features

- Parse **Guitar Pro**, **MusicXML 3.0+**, and **MNX** formats
- Render scores and tablatures (designed for use with [AlphaTab](https://www.alphatab.net/))
- Export to **MIDI** with effect-aware reconstruction (clean, overdrive, distortion, …)
- Detect and factorize repeated sections
- Extract chord symbols and guess scales
- Compute optimal guitar fingering positions
- Merge guitar tracks sharing the same measures with different effects
- Display lyrics above any track (not only the vocal track)

## File Format

The library defines two binary file formats serialized with [Postcard](https://github.com/jamesmunns/postcard):

| Extension | Content | Required |
|-----------|---------|----------|
| `.msor` | Canonical score data (notes, rhythm, lyrics, effects) | ✅ |
| `.msorlayout` | User display hints (line breaks, zoom, staff sizes) | ❌ optional |

The layout file embeds a checksum of the song file to detect desync after re-import.

## Architecture Overview

```
Score (canonical, serialized)
    │
    │  build_playback(&score, merge_rules)
    ▼
PlaybackScore (computed, never stored)
    │                    │
    │ to_midi()          │ to_alphatab_data()
    ▼                    ▼
  .mid             score / tab rendering
```

### Score structure

```
Score
├── metadata            artist, album, tempo, key, time signature, chords, scale
├── instruments         arena: Vec<Instrument>  (InstrumentId = u8)
├── staves              arena: Vec<StaffDef>     (StaffId = u8)
├── tracks              arena: Vec<Track>        (TrackId = u8)
│   ├── instrument      InstrumentId
│   ├── staves          Vec<StaffId>
│   └── measures        BTreeMap<MeasureIndex, MeasureData>
│       ├── repeat      Option<MeasureRepeat>    (%, %%)
│       └── voices      HashMap<u8, Voice>
│           └── beats   Vec<Beat>
│               ├── tick_offset, duration
│               ├── events  Vec<BeatEvent>       (EffectEvent, TempoEvent, …)
│               ├── lyric   Option<LyricAnchor>
│               ├── slur    Option<Slur>
│               └── notes   Vec<Note>
│                   ├── pitch, string, fret
│                   ├── tie, techniques
│                   └── articulations, fingering
├── timeline            Vec<MeasureDef>          (shared across all tracks)
│   ├── tempo, time_signature, key_signature
│   ├── marker          Intro, Verse, Chorus, Bridge, …
│   └── navigation      Vec<NavigationEvent>     (repeats, D.C., D.S., Coda, …)
├── lyric_lines         arena: Vec<LyricLine>    (LyricLineId = u8)
├── lyric_projections   Vec<LyricProjection>     (anchor track → display track)
└── display_hints       in separate LayoutFile
```

## Repeat System

Two distinct levels:

- **Local repeats** (`%`, `%%`) — per-track, inside `MeasureData`. Abbreviate notation; resolved dynamically during rendering or MIDI export.
- **Structural repeats** — global, inside `MeasureDef.navigation`. Cover bar repeats (`|:`, `:|`), voltas, Da Capo, Dal Segno, Coda. Applied uniformly across all tracks. Resolved by `Arrangement::resolve() -> Vec<MeasureIndex>`.

## Effect System

Effects are **point events** on a `Beat`, not ranges. Current effect state is computed by scanning backwards to the last `EffectEvent` on that track. This allows mid-measure effect changes and simplifies track merging.

```
Beat.events: Vec<BeatEvent>
    └── EffectEvent { channel: EffectChannel, volume, pan, reverb, chorus, label }
```

## MIDI Export

MIDI is never stored — it is reconstructed from `PlaybackScore`, a computed view:

1. Apply merge rules (e.g. two guitar tracks → one `PlaybackTrack`)
2. Split each track into `PlaybackSegment`s at every `EffectEvent` that changes the MIDI program
3. Emit `program_change` + `control_change` at segment boundaries
4. Emit `note_on` / `note_off` for every non-tied note

## Lyrics

Lyrics are stored at two levels:

- **Global** (`LyricLine` arena): canonical syllables, language, label — shared and independent of any track
- **Rhythmic anchor** (`LyricAnchor` in `Beat`): binds a syllable to a specific beat of a specific track
- **Display projection** (`LyricProjection`): declares on which track to visually render a lyric line — enables showing lyrics above a guitar tab even when the anchor track is a vocal track
