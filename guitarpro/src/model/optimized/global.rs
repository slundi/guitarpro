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

/// The form a [`Score`] is saved in now ([`Score::format_version`]).
/// Saves without one (0) are from before it was kept: loading migrates
/// them to this form, telling their older forms apart by what they hold,
/// as nothing else says. A save with one is in its form: loading
/// migrates nothing; one newer than this is refused.
///
/// It starts at 2: a binary save from before it was kept starts with an
/// `Option` tag (0 or 1), which a binary reader would take for the version.
pub const SCORE_FORMAT_VERSION: u32 = 2;

#[derive(Serialize, Debug)]
pub struct Score {
    /// The form it's in: [`SCORE_FORMAT_VERSION`] for one made or loaded
    /// now. First, so a binary (positional) reader can check it before
    /// decoding the rest, which it can't migrate.
    pub format_version: u32,
    pub metadata: Metadata,
    pub instruments: Vec<Instrument>, // indexed by InstrumentId
    pub staves: Vec<StaffDef>,        // indexed by StaffId
    pub tracks: Vec<Track>,           // indexed by TrackId
    /// Bracket/brace groupings across tracks (strings section, piano grand staff, etc.).
    pub groups: Vec<PartGroup>,
    /// Ordered, shared across all tracks.
    pub timeline: Vec<MeasureDef>,
    pub lyric_lines: Vec<LyricLine>, // indexed by LyricLineId
    pub lyric_projections: Vec<LyricProjection>,
    /// Engraver's rendering intent (page size, fonts, line widths).
    /// `None` means the renderer uses its own defaults.
    pub defaults: Option<ScoreDefaults>,
}

impl Score {
    /// Whether it's from a GP file: only the GP import fills GP's own
    /// measure fields ([`MeasureDef::has_gp_fields`]) and gives notes their
    /// own velocity ([`Note::gp_velocity`](crate::model::optimized::note::Note::gp_velocity)).
    /// One stripped of all of those can't be told from another format's.
    /// Loading an older save migrates what only GP's import left in an
    /// older form by it.
    pub(crate) fn is_gp_sourced(&self) -> bool {
        is_gp_sourced(&self.timeline, &self.tracks)
    }
}

/// [`Score::is_gp_sourced`], of its parts.
fn is_gp_sourced(timeline: &[MeasureDef], tracks: &[Track]) -> bool {
    timeline.iter().any(MeasureDef::has_gp_fields)
        || tracks
            .iter()
            .flat_map(|track| track.measures.values())
            .flat_map(|data| data.voices.values())
            .flat_map(|voice| &voice.beats)
            .flat_map(|beat| &beat.notes)
            .any(|note| note.gp_velocity.is_some())
}

/// [`Score`] as serialized, by any form of an older save.
#[derive(Deserialize)]
struct SerializedScore {
    /// 0 for a save from before it was kept.
    #[serde(default)]
    format_version: u32,
    metadata: Metadata,
    instruments: Vec<Instrument>,
    staves: Vec<StaffDef>,
    tracks: Vec<Track>,
    groups: Vec<PartGroup>,
    #[serde(deserialize_with = "crate::model::optimized::timeline::deserialize_timeline")]
    timeline: crate::model::optimized::timeline::SerializedTimeline,
    lyric_lines: Vec<LyricLine>,
    lyric_projections: Vec<LyricProjection>,
    defaults: Option<ScoreDefaults>,
}

/// A binary (positional) save can't be migrated, nor told from an older
/// form but by its version: only one in the current form is read. A
/// self-describing one (JSON) of any version up to the current is (1 is
/// the current form there: only binary saves' enum indices changed).
impl<'de> Deserialize<'de> for Score {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let serialized = if deserializer.is_human_readable() {
            SerializedScore::deserialize(deserializer)?
        } else {
            deserializer.deserialize_struct("SerializedScore", SERIALIZED_FIELDS, BinaryScore)?
        };
        Score::try_from(serialized).map_err(serde::de::Error::custom)
    }
}

/// [`SerializedScore`]'s fields, in order.
const SERIALIZED_FIELDS: &[&str] = &[
    "format_version",
    "metadata",
    "instruments",
    "staves",
    "tracks",
    "groups",
    "timeline",
    "lyric_lines",
    "lyric_projections",
    "defaults",
];

/// Reads a binary (positional) save: its version first, refusing one not
/// in the current form before decoding the rest, which would read an
/// older layout as this one's (failing on some field, or reading garbage).
struct BinaryScore;

impl<'de> serde::de::Visitor<'de> for BinaryScore {
    type Value = SerializedScore;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a score")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        use serde::de::Error;

        // The next field, `at` its place (for the error when it's missing).
        macro_rules! field {
            ($at:expr, $ty:ty) => {
                seq.next_element::<$ty>()?
                    .ok_or_else(|| A::Error::invalid_length($at, &self))?
            };
        }
        let format_version = field!(0, u32);
        if format_version != SCORE_FORMAT_VERSION {
            return Err(A::Error::custom(format!(
                "binary score format version {format_version} isn't this library's ({SCORE_FORMAT_VERSION})"
            )));
        }
        Ok(SerializedScore {
            format_version,
            metadata: field!(1, Metadata),
            instruments: field!(2, Vec<Instrument>),
            staves: field!(3, Vec<StaffDef>),
            tracks: field!(4, Vec<Track>),
            groups: field!(5, Vec<PartGroup>),
            timeline: field!(6, TimelineField).0,
            lyric_lines: field!(7, Vec<LyricLine>),
            lyric_projections: field!(8, Vec<LyricProjection>),
            defaults: field!(9, Option<ScoreDefaults>),
        })
    }
}

/// [`SerializedScore::timeline`], read on its own.
struct TimelineField(crate::model::optimized::timeline::SerializedTimeline);

impl<'de> Deserialize<'de> for TimelineField {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::model::optimized::timeline::deserialize_timeline(deserializer).map(TimelineField)
    }
}

impl TryFrom<SerializedScore> for Score {
    type Error = String;

    fn try_from(serialized: SerializedScore) -> Result<Self, Self::Error> {
        if serialized.format_version > SCORE_FORMAT_VERSION {
            return Err(format!(
                "score format version {} is newer than this library's ({SCORE_FORMAT_VERSION})",
                serialized.format_version
            ));
        }
        let tracks = serialized.tracks;
        // Where the score is from, worked out once (it walks every note)
        // and only if a migration needs it.
        let mut gp_sourced: Option<bool> = None;
        // An older timeline's voltas are read by where the score is from.
        let timeline = serialized.timeline.into_timeline(|timeline| {
            *gp_sourced.get_or_insert_with(|| is_gp_sourced(timeline, &tracks))
        });
        let mut score = Score {
            metadata: serialized.metadata,
            instruments: serialized.instruments,
            staves: serialized.staves,
            tracks,
            groups: serialized.groups,
            timeline,
            lyric_lines: serialized.lyric_lines,
            lyric_projections: serialized.lyric_projections,
            defaults: serialized.defaults,
            format_version: serialized.format_version,
        };
        // Only an unversioned save can be in an older form (its timeline's
        // `volta`s are migrated above: see `SerializedTimeline`).
        if score.format_version == 0 {
            crate::model::directions::migrate_direction_signs(&mut score);
            // The migrations change nothing that tells where it's from.
            let gp_sourced = gp_sourced.unwrap_or_else(|| score.is_gp_sourced());
            if gp_sourced {
                crate::convert::optimized::legacy::migrate_tie_starts(&mut score);
                crate::convert::optimized::legacy::migrate_dynamics(&mut score);
            }
            score.format_version = SCORE_FORMAT_VERSION;
        }
        Ok(score)
    }
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

impl Transpose {
    /// The same transposition with `octave_change` folded into the
    /// chromatic and diatonic steps (a bass clarinet's −2 and one octave
    /// down: −14 semitones), as formats without an octave field write it.
    pub fn folded(&self) -> Transpose {
        let octaves = i16::from(self.octave_change.unwrap_or(0));
        Transpose {
            diatonic: self.diatonic.map(|steps| steps + 7 * octaves),
            chromatic: self.chromatic + 12 * octaves,
            octave_change: None,
        }
    }
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
