use crate::model::mnx::MnxId;
use crate::model::mnx::note_value::RhythmicPosition;
use crate::model::mnx::sequence::Sequence;

/// The sign used for a clef symbol.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/clef-sign/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClefSign {
    /// G clef (treble clef) — typically placed at staff position -2.
    G,
    /// F clef (bass clef) — typically placed at staff position 2.
    F,
    /// C clef (alto/tenor clef) — may be placed at various staff positions.
    C,
}

/// The number of octaves a clef transposes its associated pitches.
///
/// 0 means no transposition (the default). Negative values mean the written pitch is
/// higher than sounded; positive values mean it is lower.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/ottava-amount-or-zero/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OttavaAmountOrZero {
    /// Three octaves lower than sounded (written pitch is 3 octaves higher).
    MinusThree,
    /// Two octaves lower than sounded.
    MinusTwo,
    /// One octave lower than sounded — rendered with an "8vb" symbol.
    MinusOne,
    /// No octave transposition.
    Zero,
    /// One octave higher than sounded — rendered with an "8va" symbol.
    PlusOne,
    /// Two octaves higher than sounded — rendered with a "15ma" symbol.
    PlusTwo,
    /// Three octaves higher than sounded — rendered with a "22" symbol.
    PlusThree,
}

/// The number of octaves for an ottava marking (excluding zero).
///
/// Negative values mean the written pitch is higher than sounded; positive values mean
/// the written pitch is lower.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/ottava-amount/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OttavaAmount {
    /// The written pitch is one octave higher than sounded ("8vb").
    MinusOne,
    /// The written pitch is two octaves higher than sounded ("15mb").
    MinusTwo,
    /// The written pitch is three octaves higher than sounded ("22").
    MinusThree,
    /// The written pitch is one octave lower than sounded ("8va").
    PlusOne,
    /// The written pitch is two octaves lower than sounded ("15ma").
    PlusTwo,
    /// The written pitch is three octaves lower than sounded ("22").
    PlusThree,
}

/// A clef symbol with its placement on the staff.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/clef/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clef {
    /// The clef sign (G, F, or C).
    pub sign: ClefSign,
    /// The staff position at which the clef is drawn. For example, a standard G clef is
    /// drawn at position -2.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-position/>
    pub staff_position: i8,
    /// Optional CSS-style color for rendering this clef and its children.
    pub color: Option<String>,
    /// A specific SMuFL glyph name for rendering this clef (e.g., `"gClefArrowUp"`).
    /// Use sparingly — prefer `octave` and `show_octave` for transposing clefs.
    pub glyph: Option<String>,
    /// The number of octaves by which the sounded pitches normally indicated by this
    /// clef should be transposed. Default is zero (no transposition).
    pub octave: Option<OttavaAmountOrZero>,
    /// Whether the clef's glyph displays an octave offset visually (such as an "8" at
    /// the top or bottom of a G clef). Defaults to true when `octave` is non-zero.
    pub show_octave: Option<bool>,
}

/// A clef placed at a specific rhythmic position within a measure.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/positioned-clef/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedClef {
    /// The clef symbol.
    pub clef: Clef,
    /// The position within the measure where this clef appears. When absent, the clef
    /// is assumed to be at the start of the measure.
    pub position: Option<RhythmicPosition>,
    /// The staff this clef appears on in multi-staff parts. Defaults to 1 when absent.
    pub staff: Option<u8>,
}

/// The direction of a beam hook (a partial beam connecting a beamed note group).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/beam-hook-direction/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamHookDirection {
    /// The hook extends to the right.
    Right,
    /// The hook extends to the left.
    Left,
    /// Consuming software determines direction automatically.
    Auto,
}

/// A beam connecting a group of notes, or a nested sub-beam within a beam group.
///
/// Beams are encoded on the first measure in which they appear. Beams that span
/// multiple measures are also encoded only once.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/beam/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Beam {
    /// IDs of the events that comprise this beam, in order of their position in the beam.
    pub events: Vec<MnxId>,
    /// Secondary (nested) beams within this beam group, for subdivided beam levels.
    pub beams: Option<Vec<Beam>>,
    /// When this beam represents a beam hook, indicates whether it extends right or left.
    /// Defaults to `Auto`.
    pub direction: Option<BeamHookDirection>,
}

/// A textual dynamic marking directing the performer's volume (e.g., "pp", "mf", "ff").
///
/// The `value` field uses standard dynamic marking strings. Common values include:
/// `"ppp"`, `"pp"`, `"p"`, `"mp"`, `"mf"`, `"f"`, `"ff"`, `"fff"`, `"sfz"`, `"fp"`.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/dynamic/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dynamic {
    /// The position of this dynamic marking within the measure.
    pub position: RhythmicPosition,
    /// The dynamic marking text (e.g., `"pp"`, `"mf"`, `"sfz"`).
    pub value: String,
    /// A specific SMuFL glyph to use when rendering this dynamic.
    pub glyph: Option<String>,
    /// The staff this dynamic applies to. When absent, applies to all staves.
    pub staff: Option<u8>,
    /// The voice this dynamic applies to. When absent, applies to all voices.
    pub voice: Option<String>,
}

/// An ottava (8va / 8vb) marking spanning one or more events.
///
/// The pitches of affected notes are encoded as their sounded pitch; the ottava only
/// affects visual display.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/ottava/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ottava {
    /// The location of the first event affected by this ottava.
    pub position: RhythmicPosition,
    /// The location of the last event affected by this ottava.
    pub end: crate::model::mnx::note_value::MeasureRhythmicPosition,
    /// The type of octave shift.
    pub value: OttavaAmount,
    /// The visual orientation of the ottava marking.
    pub orient: Option<crate::model::mnx::Orientation>,
    /// The staff this ottava applies to. Defaults to 1 when absent.
    pub staff: Option<u8>,
    /// The voice this ottava applies to. When absent, applies to all voices.
    pub voice: Option<String>,
}

/// The musical content of a single measure within a single part.
///
/// Contains sequences of note events, clef changes, dynamics, beams, and ottava markings.
/// Its index within the part's `measures` array must correspond to the same index in
/// the global measures array.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/part-measure/>
#[derive(Debug, Clone, PartialEq)]
pub struct PartMeasure {
    /// The sequences of events (voices) in this measure. At least one sequence is required.
    pub sequences: Vec<Sequence>,
    /// All beams that begin in this measure. Beams spanning multiple measures are encoded
    /// only in the first measure where they appear.
    pub beams: Option<Vec<Beam>>,
    /// Clef changes positioned within this measure.
    pub clefs: Option<Vec<PositionedClef>>,
    /// Dynamic markings in this measure.
    pub dynamics: Option<Vec<Dynamic>>,
    /// Ottava markings that begin in this measure. Multi-measure ottavas are encoded in
    /// the first measure where they appear.
    pub ottavas: Option<Vec<Ottava>>,
    /// Unique identifier for this part-measure.
    pub id: Option<MnxId>,
}
