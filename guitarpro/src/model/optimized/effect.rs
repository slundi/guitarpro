use serde::{Deserialize, Serialize};

use crate::model::optimized::note::{NoteValue, Pitch};

/// Point event attached to a Beat. Effect state is resolved by scanning
/// backwards to the last EffectEvent on the same track.
#[derive(Serialize, Deserialize, Debug)]
pub enum BeatEvent {
    Effect(EffectEvent),
    Tempo(TempoEvent),
    // --- dynamic spans ---
    Hairpin(HairpinEvent),
    // --- notation spans ---
    OctaveShift(OctaveShiftEvent),
    Pedal(PedalEvent),
    SpanLine(SpanLineEvent),
    // --- text / performance directions ---
    Words(WordsEvent),
    Rehearsal(RehearsalEvent),
    Metronome(MetronomeEvent),
    // --- playback feel ---
    Swing(SwingEvent),
    // --- instrument re-tuning ---
    Scordatura(ScordaturaEvent),
}

// ---------------------------------------------------------------------------
// Guitar/amp effect state
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct EffectEvent {
    pub channel: EffectChannel,
    pub volume: Option<f32>, // 0.0–1.0, None = unchanged
    pub pan: Option<f32>,    // -1.0 (L) to 1.0 (R), None = unchanged
    pub chorus: Option<ChorusParams>,
    pub reverb: Option<ReverbParams>,
    pub delay: Option<DelayParams>,
    pub wah: Option<bool>,
    pub label: Option<String>, // displayed above staff: "Dist.", "Clean"
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum EffectChannel {
    Clean,
    Crunch,
    Overdrive,
    Distortion,
    Acoustic,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ChorusParams {
    pub mix: f32,
    pub rate: f32,
    pub depth: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ReverbParams {
    pub mix: f32,
    pub decay: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct DelayParams {
    pub mix: f32,
    pub time_ms: u16,
    pub feedback: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct TempoEvent {
    pub bpm: f32,
}

// ---------------------------------------------------------------------------
// Hairpin dynamics (crescendo / decrescendo spans)
// ---------------------------------------------------------------------------

/// A crescendo or decrescendo hairpin span.
///
/// `id` matches a `Start` event to its `Stop` on the same track.
/// Both events are `BeatEvent::Hairpin` attached to the beats where the
/// hairpin opens and closes.
#[derive(Serialize, Deserialize, Debug)]
pub struct HairpinEvent {
    /// Span identifier (1-based, scoped per track). Links start to stop.
    pub id: u8,
    pub kind: HairpinKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum HairpinKind {
    CrescendoStart,
    DecrescendoStart,
    Stop,
}

// ---------------------------------------------------------------------------
// Octave shifts (8va, 8vb, 15ma, 15mb)
// ---------------------------------------------------------------------------

/// An octave-shift span (8va, 8vb, 15ma, 15mb, 22ma, 22mb).
///
/// `id` matches start to stop on the same staff.
#[derive(Serialize, Deserialize, Debug)]
pub struct OctaveShiftEvent {
    /// Span identifier (1-based, scoped per track). Links start to stop.
    pub id: u8,
    pub kind: OctaveShiftKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum OctaveShiftKind {
    Up8Start,    // 8va  — sounds one octave higher than written
    Down8Start,  // 8vb  — sounds one octave lower than written
    Up15Start,   // 15ma — sounds two octaves higher
    Down15Start, // 15mb — sounds two octaves lower
    Up22Start,   // 22ma — sounds three octaves higher
    Down22Start, // 22mb — sounds three octaves lower
    Stop,
}

// ---------------------------------------------------------------------------
// Pedal marks
// ---------------------------------------------------------------------------

/// Piano sustain pedal (or sostenuto) instruction.
///
/// `id` matches start to stop on the same staff.
#[derive(Serialize, Deserialize, Debug)]
pub struct PedalEvent {
    pub id: u8,
    pub kind: PedalKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum PedalKind {
    /// Depress the sustain pedal.
    Start,
    /// Release the sustain pedal.
    Stop,
    /// Release then immediately re-depress (partial release / pedal change).
    Change,
    /// Sostenuto pedal (holds only currently-pressed notes).
    SostenutoStart,
}

// ---------------------------------------------------------------------------
// Bracket / dashes span lines
// ---------------------------------------------------------------------------

/// A dashed or bracketed span line above or below the staff (e.g. `più mosso`
/// with a trailing dashes line, or a bracket around a group of notes).
///
/// `id` matches start to stop on the same staff.
#[derive(Serialize, Deserialize, Debug)]
pub struct SpanLineEvent {
    pub id: u8,
    pub kind: SpanLineKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SpanLineKind {
    /// Solid bracket with a vertical hook at the end.
    BracketStart,
    /// Dashed extension line (used after a text direction).
    DashesStart,
    Stop,
}

// ---------------------------------------------------------------------------
// Text directions
// ---------------------------------------------------------------------------

/// A free-form text direction above or below the staff
/// (e.g. `"Allegro"`, `"con fuoco"`, `"pizz."`).
#[derive(Serialize, Deserialize, Debug)]
pub struct WordsEvent {
    pub text: String,
    pub italic: bool,
    pub bold: bool,
    /// Font size in points. `None` = renderer default.
    pub font_size: Option<f32>,
}

/// A rehearsal mark shown in a box or circle (e.g. `"A"`, `"B"`, `"1"`).
#[derive(Serialize, Deserialize, Debug)]
pub struct RehearsalEvent {
    pub label: String,
    pub enclosure: RehearsalEnclosure,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum RehearsalEnclosure {
    Box,
    Circle,
    None,
}

/// A displayed metronome marking (e.g. `♩ = 120`).
///
/// `bpm` drives playback tempo; `beat_unit` and `dots` control the displayed glyph.
#[derive(Serialize, Deserialize, Debug)]
pub struct MetronomeEvent {
    /// The note value of one displayed beat (left side of the equation).
    pub beat_unit: NoteValue,
    /// Augmentation dots on the beat unit.
    pub beat_unit_dots: u8,
    /// Beats per minute.
    pub bpm: f32,
    /// When `true`, the marking is enclosed in parentheses: `(♩ = 120)`.
    pub parentheses: bool,
}

// ---------------------------------------------------------------------------
// Swing feel
// ---------------------------------------------------------------------------

/// A swing-feel instruction: how pairs of eighth notes should be performed.
#[derive(Serialize, Deserialize, Debug)]
pub struct SwingEvent {
    /// `true` = straight (no swing), `false` = swung.
    pub straight: bool,
    /// Optional explicit ratio `(first, second)` — e.g. `(2, 1)` for a
    /// triplet-eighth feel (long–short). `None` = default/stylistic swing.
    pub ratio: Option<(u8, u8)>,
}

// ---------------------------------------------------------------------------
// Scordatura (string re-tuning)
// ---------------------------------------------------------------------------

/// A string re-tuning instruction: new open-string pitches from this point
/// forward in the part.
#[derive(Serialize, Deserialize, Debug)]
pub struct ScordaturaEvent {
    /// One entry per re-tuned string: `(string_number, new_open_pitch)`.
    /// String numbering follows the instrument definition (1 = highest-pitched).
    pub strings: Vec<(u8, Pitch)>,
}
