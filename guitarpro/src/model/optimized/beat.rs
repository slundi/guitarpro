//! Voice, Beat, Duration, and related rhythm/articulation types.

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    effect::BeatEvent,
    global::LyricLineId,
    metadata::ChordSymbol,
    note::{Note, NoteValue},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Voice {
    pub voice_id: u8,
    pub beats: Vec<Beat>,
}

/// A grace note preceding a beat.
///
/// Grace notes are rendered before the beat's `tick_offset` and typically
/// steal time from the following note (appoggiatura) or the preceding one
/// (acciaccatura with `slash = true`).
#[derive(Serialize, Deserialize, Debug)]
pub struct GraceNote {
    pub note: Note,
    /// Acciaccatura (slashed grace note) when `true`; appoggiatura when `false`.
    pub slash: bool,
    /// Fraction of the beat duration to steal for this grace note (0.0–1.0).
    /// `None` = renderer decides based on context and style.
    pub steal_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Beat {
    pub tick_offset: u32,
    pub duration: Duration,
    pub notes: Vec<Note>,
    pub events: Vec<BeatEvent>,
    pub dynamic: Option<Dynamic>,
    pub slur: Option<Slur>,
    pub lyric: Option<LyricAnchor>,
    pub beam_group: Option<u8>,
    pub tuplet: Option<Tuplet>,
    /// Explicit beam begin/continue/end markings per beam level.
    /// Level 1 = eighth-note beam, level 2 = sixteenth, etc.
    /// When empty the renderer infers beaming from duration and context.
    pub beams: Vec<Beam>,
    /// Grace notes that sound immediately before this beat.
    pub grace_notes: Vec<GraceNote>,
    /// When `true`, this beat is a cue (shown smaller, not counted in the part's dynamics).
    pub cue: bool,
    /// Chord symbol that begins on this beat. Rendered above the staff.
    pub chord: Option<ChordSymbol>,
    /// GP-format hint: when `true` this beat is a filler (BeatStatus::Empty, takes 0 time).
    /// Used only in the legacy↔optimized roundtrip; ignored by renderers.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_empty: bool,
    /// GP-format hint: when `true` this beat is a rest (BeatStatus::Rest).
    /// Needed to distinguish rest beats from Normal beats that happen to have no notes.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_rest: bool,
    /// GP beat-level vibrato (flags1 & 0x02 in the beat-effects section).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_vibrato: bool,
    /// GP fade-in articulation (flags1 & 0x10 in the beat-effects section).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_fade_in: bool,
    /// GP arpeggiated stroke: (duration_value, is_up_direction).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_stroke: Option<(u16, bool)>,
    /// GP pick stroke direction: `true` = up, `false` = down.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_pick_stroke: Option<bool>,
    /// GP5 beat display flags2 (octave, beam direction, tuplet bracket, break_secondary, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_beat_flags2: Option<i16>,
    /// GP5 break_secondary value (written after flags2 when flags2 & 0x0800 is set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_break_secondary: Option<u8>,
    /// GP4/5 slap effect: 1=Tapping, 2=Slapping, 3=Popping. `None` = no slap effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_slap_effect: Option<u8>,
    /// GP4/5 rasgueado flag in beat effects.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_rasgueado: bool,
    /// GP beat-level text annotation (fingering numbers, arbitrary strings).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gp_text: String,
    /// GP4/5 mix table change on this beat (tempo change, volume change, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_mix_table: Option<GpMixTableChange>,
    /// GP4/5 tremolo bar (whammy bar) effect on this beat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_tremolo_bar: Option<crate::model::optimized::note::GpBendEffect>,
    /// GP4/5 chord diagram on this beat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_chord: Option<GpChord>,
}

/// GP4/5 chord diagram, preserved for byte-identical roundtrip.
/// Stores the values as written by `write_chord_v4`.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpChord {
    /// Whether the chord uses the "new format" encoding in the GP binary.
    /// `false` = old compact format, `true` = extended format.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub new_format: bool,
    /// String count field (chord `length`) used by the GP binary writer.
    #[serde(default, skip_serializing_if = "crate::model::optimized::is_zero_u8")]
    pub length: u8,
    pub sharp: bool,
    /// PitchClass.value of the chord root.
    pub root: i8,
    /// Encoded chord type (from_chord_type result).
    pub kind: u8,
    /// Encoded chord extension.
    pub extension: u8,
    /// PitchClass.value of the bass note as i32.
    pub bass: i32,
    /// Encoded tonality as i32.
    pub tonality: i32,
    pub add: bool,
    pub name: String,
    pub fifth: u8,
    pub ninth: u8,
    pub eleventh: u8,
    pub first_fret: i32,
    /// String fret values (up to 7; -1 = open/unused).
    pub strings: Vec<i32>,
    /// Barre tuples: (fret, start_string, end_string), up to 5.
    pub barres: Vec<(u8, u8, u8)>,
    /// String omission flags (up to 7).
    pub omissions: Vec<bool>,
    /// Raw fingering values for each string (up to 7).
    pub fingerings: Vec<i8>,
    pub show: bool,
}

/// A single mix table field with a value, a transition duration, and an "all tracks" flag.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpMixTableItem {
    pub value: u8,
    pub duration: u8,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub all_tracks: bool,
}

/// GP4/5 mix table change — changes applied to the mix at the point of this beat.
#[derive(Serialize, Deserialize, Debug)]
pub struct GpMixTableChange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrument: Option<GpMixTableItem>,
    // RSE instrument (GP5 only, often default). Stored as 4-byte ints in the
    // legacy file, so keep 32-bit width — some exports use IDs larger than i16.
    #[serde(default)]
    pub rse_instrument: i32,
    #[serde(default)]
    pub rse_unknown: i32,
    #[serde(default)]
    pub rse_sound_bank: i32,
    #[serde(default)]
    pub rse_effect_number: i32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rse_effect_category: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rse_effect: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<GpMixTableItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<GpMixTableItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chorus: Option<GpMixTableItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverb: Option<GpMixTableItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phaser: Option<GpMixTableItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<GpMixTableItem>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tempo_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tempo: Option<GpMixTableItem>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hide_tempo: bool,
    /// GP5 wah effect: `None` = not present, `Some((value, display))`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wah: Option<(i8, bool)>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub use_rse: bool,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Duration {
    pub base: NoteValue,
    pub dots: u8, // 0, 1, or 2
    pub tuplet: Option<Tuplet>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Tuplet {
    pub actual: u8, // e.g. 3 (triplet plays 3 notes…)
    pub normal: u8, // …in the time of 2
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Dynamic {
    PPP,
    PP,
    P,
    MP,
    MF,
    F,
    FF,
    FFF,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Slur {
    pub slur_id: u8,
    pub kind: SlurKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SlurKind {
    Start,
    End,
}

/// Binds a beat to a specific syllable in a LyricLine.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct LyricAnchor {
    pub lyric_line_id: LyricLineId,
    pub syllable_index: u16, // index into LyricLine.syllables
}

// ---------------------------------------------------------------------------
// Beams
// ---------------------------------------------------------------------------

/// Explicit beam segment for one beam level on this beat.
///
/// Level 1 spans eighth notes, level 2 sixteenth notes, etc.
/// `ForwardHook` and `BackwardHook` represent partial beams (e.g. on dotted rhythms).
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Beam {
    /// Beam level, 1-based (1 = outermost / thickest beam).
    pub level: u8,
    pub kind: BeamKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum BeamKind {
    Begin,
    Continue,
    End,
    ForwardHook,
    BackwardHook,
}
