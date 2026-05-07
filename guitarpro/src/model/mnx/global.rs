use crate::model::mnx::lyrics::LyricsGlobal;
use crate::model::mnx::note_value::{NoteValue, RhythmicPosition};
use crate::model::mnx::{MnxId, Orientation};

/// The number of fifths distance from a key signature with no accidentals (C major / A minor).
///
/// Positive values indicate sharps (e.g., 1 = G major, 2 = D major), negative values
/// indicate flats (e.g., -1 = F major, -2 = B♭ major).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/fifths/>
pub type Fifths = i8;

/// The top number of a time signature — the number of beats per measure.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/time-signature-unit/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSignatureUnit {
    /// Whole note unit (1).
    Whole,
    /// Half note unit (2).
    Half,
    /// Quarter note unit (4).
    Quarter,
    /// Eighth note unit (8).
    Eighth,
    /// Sixteenth note unit (16).
    Sixteenth,
    /// 32nd note unit (32).
    ThirtySecond,
    /// 64th note unit (64).
    SixtyFourth,
    /// 128th note unit (128).
    OneHundredTwentyEighth,
}

/// A special glyph to display instead of numeric top/bottom numbers for certain conventional
/// time signatures.
///
/// Even when `display` is set, `count` and `unit` must still be encoded (e.g.,
/// `display: Common` requires `count: 4, unit: Quarter`).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/time-signature-display/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSignatureDisplay {
    /// Common time — displayed as a `C` symbol (equivalent to 4/4).
    Common,
    /// Cut time (alla breve) — displayed as a `¢` symbol (equivalent to 2/2).
    Cut,
}

/// A time signature, specifying the number of beats and the beat unit.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/time/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSignature {
    /// The number of beats per measure (top number of the time signature).
    pub count: u32,
    /// The beat unit (bottom number of the time signature).
    pub unit: TimeSignatureUnit,
    /// A special glyph such as common time or cut time. When provided, this overrides
    /// the numeric display, but `count` and `unit` are still required.
    pub display: Option<TimeSignatureDisplay>,
}

/// A key signature.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/key/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySignature {
    /// The number of fifths distance from a key signature with no accidentals (C major).
    /// Positive = sharps, negative = flats.
    pub fifths: Fifths,
    /// Optional CSS-style color to use when rendering this key signature.
    pub color: Option<String>,
}

/// The visual style of a barline.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/barline-type/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarlineType {
    /// A standard single barline.
    Regular,
    /// A dashed barline.
    Dashed,
    /// A dotted barline.
    Dotted,
    /// Two light lines together — the conventional "double barline."
    Double,
    /// A light line followed by a heavy line — conventionally used at the end of a score.
    Final,
    /// A single heavy barline.
    Heavy,
    /// Two heavy barlines.
    HeavyHeavy,
    /// A heavy line followed by a light line.
    HeavyLight,
    /// No barline (invisible).
    NoBarline,
    /// A short barline that doesn't span the full height of the staff.
    Short,
    /// A tick barline at the top of the staff.
    Tick,
}

/// A barline at the end of a measure.
///
/// The default barline type is `Final` for the last measure and `Regular` for all others.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/barline/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Barline {
    /// The visual style of this barline.
    pub barline_type: BarlineType,
}

/// A tempo marking — asserts a specific number of beats per minute.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/tempo/>
#[derive(Debug, Clone, PartialEq)]
pub struct Tempo {
    /// The note value that occurs at `bpm` times per minute (e.g., a quarter note at
    /// 120 bpm means 120 quarter notes per minute).
    pub value: NoteValue,
    /// The number of times per minute that `value` should occur.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/bpm/>
    pub bpm: f64,
    /// Where within the measure this tempo marking begins. Defaults to the start of the
    /// measure when not provided.
    pub location: Option<RhythmicPosition>,
}

/// How long a fermata extends the duration of a note or rest.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/fermata-duration/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FermataDuration {
    /// Consuming software determines the duration according to its own algorithms.
    Auto,
    /// A very short pause.
    VeryShort,
    /// A short pause.
    Short,
    /// A normal-length pause.
    Normal,
    /// A long pause.
    Long,
    /// A very long pause.
    VeryLong,
    /// No effect on playback — the fermata is displayed but does not extend duration.
    None,
}

/// The visual symbol used for a fermata.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/fermata-symbol/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FermataSymbol {
    /// A standard curved fermata.
    Normal,
    /// A square fermata.
    Square,
    /// An angled fermata.
    Angled,
    /// A double-dotted fermata.
    DoubleDot,
    /// A double square fermata.
    DoubleSquare,
    /// A double angled fermata.
    DoubleAngled,
    /// A half-curve fermata.
    HalfCurve,
    /// A curlew (open) fermata.
    Curlew,
}

/// A three-state directional value used for fermata pointing direction.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/up-down-auto/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpDownAuto {
    Up,
    Down,
    Auto,
}

/// A fermata — a hold placed over or under a note, chord, or rest.
///
/// All fields are optional; an empty `Fermata {}` uses default values for all attributes.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/fermata/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fermata {
    /// How long the fermata pause lasts. Defaults to `Auto` when not specified.
    pub duration: Option<FermataDuration>,
    /// The vertical orientation of this fermata's symbol relative to the staff.
    /// Defaults to `Auto` when not specified.
    pub orient: Option<Orientation>,
    /// The direction the fermata symbol points. Standard fermatas above the staff use
    /// `Up`. Defaults to `Auto` when not specified.
    pub pointing: Option<UpDownAuto>,
    /// The visual symbol used for this fermata. Defaults to `Normal` when not specified.
    pub symbol: Option<FermataSymbol>,
}

/// A volta bracket (alternate ending) starting at a measure.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/ending/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ending {
    /// The duration of this ending, measured as a number of measures.
    pub duration: u32,
    /// The volta numbers displayed on the bracket (e.g., `[1]`, `[2]`, `[1, 2]`).
    pub numbers: Option<Vec<u32>>,
    /// Whether the ending bracket is left open at the right side (i.e., no closing hook).
    pub open: Option<bool>,
    /// Optional CSS-style color for rendering this bracket.
    pub color: Option<String>,
}

/// Repeat barline at the start of a repeated section.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/repeat-start/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeatStart;

/// Repeat barline at the end of a repeated section.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/repeat-end/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeatEnd {
    /// The number of times this section should be repeated. Defaults to 2 (play once,
    /// repeat once) when not specified.
    pub times: Option<u32>,
}

/// The type of navigation jump instruction.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/jump-type/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpType {
    /// D.S. al Fine — jump back to the segno and play until the Fine marking.
    DsAlFine,
    /// Segno — a jump to a segno symbol (e.g., D.C. al Segno).
    Segno,
}

/// A navigation jump instruction (e.g., D.S. al Fine, D.C.).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/jump/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jump {
    /// The location within the measure where this jump is positioned.
    pub location: RhythmicPosition,
    /// The type of jump instruction.
    pub jump_type: JumpType,
}

/// A segno symbol (𝄋), marking the target of a D.S. jump.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/segno/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segno {
    /// Unique identifier for this segno, allowing jumps to target it.
    pub id: Option<MnxId>,
}

/// A Fine marking, indicating the end of the piece during a D.S. al Fine.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/fine/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fine;

/// Global notation data for a single measure — shared across all parts.
///
/// An array of these objects forms the backbone of the score timeline; its length
/// determines the total number of measures in the piece.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/measure-global/>
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureGlobal {
    /// The barline at the end of this measure. Defaults to `Final` for the last measure
    /// and `Regular` for all others.
    pub barline: Option<Barline>,
    /// A volta bracket (alternate ending) beginning at this measure.
    pub ending: Option<Ending>,
    /// A fermata aligned with the end barline of this measure.
    pub fermata: Option<Fermata>,
    /// A Fine marking in this measure.
    pub fine: Option<Fine>,
    /// A jump instruction (D.S., D.C., etc.) in this measure.
    pub jump: Option<Jump>,
    /// A key signature change beginning at this measure.
    pub key: Option<KeySignature>,
    /// The visual label for this measure. Not required to be unique in the document.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/measure-number/>
    pub number: Option<String>,
    /// A repeat ending barline at the end of this measure.
    pub repeat_end: Option<RepeatEnd>,
    /// A repeat start barline at the beginning of this measure.
    pub repeat_start: Option<RepeatStart>,
    /// A segno symbol in this measure.
    pub segno: Option<Segno>,
    /// Tempo markings applicable to this measure.
    pub tempos: Option<Vec<Tempo>>,
    /// A time signature change beginning at this measure.
    pub time: Option<TimeSignature>,
    /// Unique identifier for this measure global object.
    pub id: Option<MnxId>,
}

/// The global data object — contains notation data shared by all parts, organized by measure.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/global/>
#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    /// An array of per-measure global objects. Its length defines the total number of
    /// measures in the score.
    pub measures: Vec<MeasureGlobal>,
    /// Global lyrics metadata (line ordering, labels, etc.).
    pub lyrics: Option<LyricsGlobal>,
    /// User-defined sound definitions (keyed by user-chosen IDs).
    ///
    /// Each entry maps a user-defined sound ID to a vendor-specific dictionary of
    /// sound properties (e.g., MIDI program, sample references). The inner map uses
    /// string keys and string values as a lowest-common-denominator representation;
    /// consuming software should interpret values according to the vendor extension.
    pub sounds:
        Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
}
