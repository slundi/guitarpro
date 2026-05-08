# CLAUDE.md — optimized music notation architecture reference

This file describes the complete data model of the `optimized` library.
It is intended as a reference for AI assistants and contributors.

---

## Compact ID types

All arena indices use small integer newtypes. `usize` is never used in serialized structures.

```rust
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct InstrumentId(pub u8);   // max 255

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StaffId(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TrackId(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct LyricLineId(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MeasureIndex(pub u16);  // max 65535 measures
```

Arenas are plain `Vec<T>` indexed by `id.0 as usize`. No `SlotMap` in serialized form.

---

## File format

Two file types:
- `.msor` — canonical score (`FileHeader` + `Score`)
- `.msorlayout` — user display hints (`FileHeader` + `LayoutFile`)

---

## Key design decisions

| Decision | Rationale |
|----------|-----------|
| `MeasureDef` is global | Tempo, key, navigation events are shared by all tracks |
| `MeasureData` is per-track | Notes and voices differ per instrument |
| Effects are point events on `Beat` | Avoids redundant range structs; state resolved by backward scan |
| `LayoutFile` is a separate file | Different author (user), different lifecycle, optional |
| `LyricProjection` decouples anchor from display | Lyrics can appear above any track, not just the vocal one |
| `PlaybackScore` is never serialized | MIDI output is always recomputed; merge rules are user-defined |
| `u8`/`u16` IDs instead of `usize` | Compact binary; scores never exceed 255 tracks or 65535 measures |
| `pitch` and `(string, fret)` both stored on `Note` | Tab and notation are redundant but both needed; computed on import |