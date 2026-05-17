//! Top-level score structure, ID newtypes, instrument and lyric types.

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    display::ScoreDefaults,
    metadata::Metadata,
    note::Pitch,
    timeline::MeasureDef,
    track::{StaffDef, Track},
};

// --- Compact ID newtypes -------------------------------------------------------
// Arenas are plain Vec<T> indexed by id.0 as usize. usize is never stored.

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct InstrumentId(pub u8); // max 255 instruments

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TrackId(pub u8); // max 255 tracks

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StaffId(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct LyricLineId(pub u8);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MeasureIndex(pub u16); // max 65535 measures; Ord needed for BTreeMap keys

// --- Score (top level) --------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Score {
    pub metadata: Metadata,
    pub instruments: Vec<Instrument>, // indexed by InstrumentId
    pub staves: Vec<StaffDef>,        // indexed by StaffId
    pub tracks: Vec<Track>,           // indexed by TrackId
    /// Bracket/brace groupings across tracks (strings section, piano grand staff, etc.).
    pub groups: Vec<PartGroup>,
    pub timeline: Vec<MeasureDef>, // ordered, shared across all tracks
    pub lyric_lines: Vec<LyricLine>, // indexed by LyricLineId
    pub lyric_projections: Vec<LyricProjection>,
    /// Engraver's rendering intent (page size, fonts, line widths).
    /// `None` means the renderer uses its own defaults.
    pub defaults: Option<ScoreDefaults>,
}

// --- Part grouping ------------------------------------------------------------

/// A bracket or brace drawn to the left of a set of tracks, grouping them visually
/// and optionally connecting their barlines.
///
/// Groups may nest (e.g. an outer bracket for "Strings" containing an inner brace
/// for "Violin I + II"). Nesting is expressed by overlapping `tracks` ranges.
#[derive(Serialize, Deserialize, Debug)]
pub struct PartGroup {
    /// Long group name shown at the start of the first system (e.g. `"Strings"`).
    pub label: Option<String>,
    /// Short name shown on subsequent systems (e.g. `"Str."`).
    pub abbreviation: Option<String>,
    /// Symbol drawn to the left of the group.
    pub symbol: GroupSymbol,
    /// Whether barlines are drawn through all staves of the group.
    pub barline: GroupBarline,
    /// Ordered list of tracks included in this group.
    pub tracks: Vec<TrackId>,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum GroupSymbol {
    /// Square bracket — used for orchestral sections (woodwinds, brass, strings).
    Bracket,
    /// Curly brace — used for keyboard instruments (piano, organ, harp).
    Brace,
    /// Square bracket variant (rare; some choral scores).
    Square,
    /// Simple vertical line with no decoration.
    Line,
    /// No symbol — groups only for barline connectivity.
    None,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum GroupBarline {
    /// Barlines are drawn through all staves of the group (standard).
    Yes,
    /// Each staff has its own barlines (independent parts).
    No,
    /// Barlines drawn *between* staves only, not through them (Mensurstrich).
    Mensurstrich,
}

impl Score {
    // fn get_lyrics_for_display(
    //     &self,
    //     display_track: TrackId,
    //     measure_index: MeasureIndex,
    // ) -> Vec<(/*tick*/ usize, &LyricSyllable)> {
    //     self.lyric_projections
    //         .iter()
    //         .filter(|p| p.display_track == display_track)
    //         .flat_map(|p| {
    //             // find LyricAnchor anchor_track for the given measure
    //             let anchor_track = self.tracks[p.anchor_track.0 as usize];
    //             let lyric_line = self.lyric_lines[p.lyric_line_id.0 as usize];
    //             anchor_track.measures[&measure_index]
    //                 .all_beats()
    //                 .filter_map(|beat| {
    //                     beat.lyric
    //                         .as_ref()
    //                         .filter(|a| a.lyric_line_id == p.lyric_line_id)
    //                         .map(|a| (beat.tick_offset, &lyric_line.syllables[a.syllable_index]))
    //                 })
    //         })
    //         .collect()
    // }
}

// --- Transposition ------------------------------------------------------------

/// Written-to-sounding transposition for a transposing instrument
/// (e.g. B♭ clarinet: chromatic = -2, diatonic = -1).
///
/// All values describe the interval from written pitch → sounded pitch.
/// Negative values mean the instrument sounds lower than written.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Transpose {
    /// Diatonic step count (e.g. -1 for a second, -4 for a fifth).
    pub diatonic: Option<i16>,
    /// Exact chromatic semitones (negative = sounds lower, positive = higher).
    pub chromatic: i16,
    /// Additional octave adjustment applied on top of `chromatic`.
    pub octave_change: Option<i8>,
}

// --- Instrument ---------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Instrument {
    /// Full instrument name shown at the start of the first system (e.g. `"Violin I"`).
    pub name: String,
    /// Abbreviated name shown on subsequent systems (e.g. `"Vl. I"`).
    pub abbreviation: Option<String>,
    /// GM or VST sound identifier (e.g. `"strings.violin"`, `"Guitar (clean)"`).
    /// Informational — used to select a default sound patch, not for playback logic.
    pub instrument_sound: Option<String>,

    // --- MIDI playback definition ---
    /// MIDI channel (0–15). Channel 9 is conventionally reserved for percussion.
    pub midi_channel: u8,
    /// MIDI program number (0–127, GM patch).
    pub midi_program: u8,
    /// MIDI bank number (combines CC0 coarse + CC32 fine: `bank = cc0 * 128 + cc32`).
    /// `None` = bank 0 (GM default).
    pub midi_bank: Option<u16>,
    /// Initial volume sent as MIDI CC7 at the start of playback (0.0–1.0).
    /// `None` = MIDI default (1.0 / CC7=127). Overridden beat-by-beat by [`EffectEvent`].
    pub volume: Option<f32>,
    /// Initial stereo pan sent as MIDI CC10 at the start of playback (-1.0 L … 1.0 R).
    /// `None` = center (0.0 / CC10=64). Overridden beat-by-beat by [`EffectEvent`].
    pub pan: Option<f32>,

    pub kind: InstrumentKind,
    /// Written-to-sounded transposition. `None` = concert-pitch (non-transposing).
    pub transpose: Option<Transpose>,
    /// GP-format strings for percussion tracks (tuning values, all-zero in GP5).
    /// Preserved for byte-identical roundtrip; ignored by renderers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gp_strings: Vec<i8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InstrumentKind {
    Pitched,
    Percussion,
    Stringed {
        tuning: Vec<Pitch>, // low → high, e.g. [E2, A2, D3, G3, B3, E4]
        string_count: u8,
        capo: u8,
    },
}

// --- Lyrics -------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct LyricLine {
    pub label: Option<String>,    // "Verse 1", "Chorus", …
    pub language: Option<String>, // BCP47: "en", "fr", …
    pub syllables: Vec<LyricSyllable>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LyricSyllable {
    pub text: String,
    pub hyphen: bool,     // true if more syllables follow in the same word
    pub line_break: bool, // visual line break in lyric display
    /// Elision character printed between this syllable and the next
    /// (e.g. the standard elision tie `"‿"`). `None` = no elision.
    pub elision: Option<String>,
    /// Melisma extender line: draw an underline after this syllable until the
    /// next syllable or the end of the phrase (for held-note syllables).
    pub extend: bool,
    /// The syllable is sung as laughter ("ha", "ha-ha", etc.).
    pub laughing: bool,
    /// The syllable is hummed rather than sung with text.
    pub humming: bool,
}

/// Declares where to visually render a lyric line.
/// `anchor_track` = where LyricAnchors live (usually the vocal track).
/// `display_track` = where to draw the lyrics (can be a guitar tab track).
#[derive(Serialize, Deserialize, Debug)]
pub struct LyricProjection {
    pub lyric_line_id: LyricLineId,
    pub anchor_track: TrackId,
    pub display_track: TrackId,
}
