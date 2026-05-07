use crate::model::mnx::MnxId;
use crate::model::mnx::measure::PartMeasure;

/// A chromatic and diatonic interval, used to express transpositions.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/interval/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    /// The number of chromatic half-steps between the two pitches.
    pub half_steps: i32,
    /// The diatonic staff distance between the pitches. For example, the distance from
    /// the bottom E line to the bottom F space is 1.
    pub staff_distance: i32,
}

/// The transposition applied to a part — describes the relationship between a
/// transposing instrument's written pitch and its sounded pitch.
///
/// For example, a B♭ clarinet sounds a major second lower than written:
/// `interval: { half_steps: -2, staff_distance: -1 }`.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/part-transposition/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartTransposition {
    /// The interval that transforms a sounded pitch into the instrument's written pitch.
    pub interval: Interval,
    /// Controls when key signatures "flip" enharmonically to avoid excessive
    /// accidentals. Non-negative values subtract 12 fifths; negative values add 12.
    /// When absent, no flipping is applied.
    pub key_fifths_flip_at: Option<i32>,
    /// When true, this instrument prefers displaying written (transposed) pitches even
    /// in a concert-pitch score. Conventionally applied to piccolo, glockenspiel, and
    /// double bass.
    pub prefers_written_pitches: Option<bool>,
}

/// A component of a percussion kit — defines how a specific drum/cymbal is notated
/// on the kit staff.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/kit-component/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitComponent {
    /// MIDI note number for this kit component (0–127).
    pub midi_note: Option<crate::model::mnx::MidiNumber>,
    /// Staff position at which this kit component is notated.
    pub staff_position: Option<i8>,
    /// Notehead symbol to use for this kit component (SMuFL glyph name).
    pub notehead: Option<String>,
}

/// A musical part — one instrument or voice in the composition.
///
/// The `measures` array must have the same length as the global data's `measures` array,
/// since each entry corresponds to the same measure in the score timeline.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/part/>
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// The measure-by-measure musical content for this part.
    pub measures: Vec<PartMeasure>,
    /// The full name of this part, suitable for display to the left of the first system
    /// (e.g., "Guitar", "Violin I").
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/part-name/>
    pub name: Option<String>,
    /// An abbreviated name for this part, shown on subsequent systems
    /// (e.g., "Gtr.", "Vln. I").
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/part-short-name/>
    pub short_name: Option<String>,
    /// The number of staves this part uses. For example, a piano part uses 2 (grand staff).
    /// Defaults to 1 when absent.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-count/>
    pub staves: Option<u8>,
    /// The SMuFL-compliant font to use when rendering notational objects in this part.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/smufl-font/>
    pub smufl_font: Option<String>,
    /// The instrument's transposition. Specifies the interval that transforms a sounded
    /// pitch into a written pitch.
    pub transposition: Option<PartTransposition>,
    /// Percussion kit component definitions, keyed by user-defined IDs. Events in this
    /// part reference kit component IDs to specify which drum/cymbal is played.
    pub kit: Option<std::collections::HashMap<String, KitComponent>>,
    /// Unique identifier for this part.
    pub id: Option<MnxId>,
}
