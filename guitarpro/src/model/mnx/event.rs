use crate::model::mnx::lyrics::Lyrics;
use crate::model::mnx::note::Note;
use crate::model::mnx::note_value::{NoteValue, NoteValueQuantity};
use crate::model::mnx::{MnxId, Orientation};

/// The visual style of a slur or tie line.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/line-type/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Solid,
    Dashed,
    Dotted,
}

/// The side on which a slur is drawn relative to the notes it spans.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/slur-side/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlurSide {
    Up,
    Down,
}

/// Whether a note stem points up or down.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/stem-direction/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemDirection {
    Up,
    Down,
}

/// The direction of a bow stroke for string instruments.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/bow-direction/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BowDirection {
    Up,
    Down,
}

/// A slur spanning from the containing event to a target event.
///
/// Slurs are encoded on the *first* event — i.e., the event where the slur begins.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/slur/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slur {
    /// The ID of the slur's end event — that is, the last event over which this slur
    /// is rendered.
    pub target: MnxId,
    /// The ID of the specific note at which this slur ends. This note must be within
    /// the event specified by `target`.
    pub end_note: Option<MnxId>,
    /// The visual line style of the slur. When absent, consuming software applies its
    /// own default.
    pub line_type: Option<LineType>,
    /// The slur's side at its starting point. When absent, consuming software determines
    /// this automatically.
    pub side: Option<SlurSide>,
    /// The slur's side at its ending point. Useful when the end side differs from the
    /// start side.
    pub side_end: Option<SlurSide>,
    /// The ID of the specific note at which this slur starts, within the containing event.
    pub start_note: Option<MnxId>,
}

/// A single-note tremolo marking, indicating rapid reiteration of the note.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/tremolo-single/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleNoteTremolo {
    /// The number of beams (slashes) used to notate the tremolo. Must be between 1 and 8.
    pub marks: u8,
}

/// A standard accent articulation (`>`).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/accent/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accent;

/// A soft accent articulation.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/soft-accent/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftAccent;

/// A strong accent articulation (marcato, `^`).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/strong-accent/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongAccent;

/// A breath mark, indicating the performer should take a breath.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/breath-mark/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreathMark {
    /// The visual symbol used for the breath mark (e.g., comma, caesura).
    /// Uses SMuFL glyph names.
    pub symbol: Option<String>,
}

/// Spiccato bowing articulation — notes played with a bouncing bow stroke.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/spiccato/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spiccato;

/// Staccatissimo articulation — notes played as short as possible.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staccatissimo/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Staccatissimo;

/// Staccato articulation — notes played detached.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staccato/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Staccato;

/// Stress marking — an emphasis indicator.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/stress-marking/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StressMarking;

/// Tenuto articulation — notes held for their full duration.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/tenuto/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tenuto;

/// Unstress marking — a de-emphasis indicator.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/unstress-marking/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnstressMarking;

/// A collection of articulation and notation markings that apply to an entire event.
///
/// This is a catch-all container for notations that affect the event as a whole, as
/// distinct from per-note markings.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/event-markings/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMarkings {
    /// Standard accent articulation (`>`).
    pub accent: Option<Accent>,
    /// Bow direction for bowed string instruments.
    pub bow_direction: Option<BowDirection>,
    /// A breath mark — the performer should take a breath here.
    pub breath: Option<BreathMark>,
    /// Soft accent articulation.
    pub soft_accent: Option<SoftAccent>,
    /// Spiccato bowing articulation.
    pub spiccato: Option<Spiccato>,
    /// Staccatissimo — as short as possible.
    pub staccatissimo: Option<Staccatissimo>,
    /// Staccato — detached.
    pub staccato: Option<Staccato>,
    /// Stress marking (emphasis).
    pub stress: Option<StressMarking>,
    /// Strong accent (marcato, `^`).
    pub strong_accent: Option<StrongAccent>,
    /// Tenuto — held for full value.
    pub tenuto: Option<Tenuto>,
    /// Single-note tremolo (rapid reiteration).
    pub tremolo: Option<SingleNoteTremolo>,
    /// Unstress marking (de-emphasis).
    pub unstress: Option<UnstressMarking>,
}

/// A note played on a percussion kit instrument, identified by its kit component.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/kit-note/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitNote {
    /// The kit component ID this note belongs to.
    pub kit_id: MnxId,
}

/// Represents a vertical position on the staff, where 0 is the middle line.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-position/>
pub type StaffPosition = i8;

/// A rest within an event. All fields are optional; an empty object `{}` is valid.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/rest/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rest {
    /// The vertical position of the rest on the staff. When omitted, standard positioning
    /// conventions apply.
    pub staff_position: Option<StaffPosition>,
}

/// A whole-measure rest that occupies an entire sequence.
///
/// When `full_measure` is set on a `Sequence`, the sequence's `content` must be empty.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/full-measure-rest/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullMeasureRest {
    /// The vertical position of the rest on the staff. When omitted, standard positioning
    /// conventions apply.
    pub staff_position: Option<StaffPosition>,
}

/// The category of grace note, controlling how it interacts with surrounding events.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/grace-type/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraceType {
    /// The grace notes delay the onset of the next non-grace event.
    MakeTime,
    /// The grace notes occupy time starting at the expected onset of the next non-grace
    /// event, delaying it and shortening its duration.
    StealFollowing,
    /// The grace notes occupy time ending before the expected onset of the next non-grace
    /// event, shortening the preceding event's duration.
    ///
    /// This is the default when `grace_type` is not specified.
    StealPrevious,
}

/// Controls whether a tuplet's ratio or note value is displayed.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/tuplet-display-setting/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TupletDisplaySetting {
    /// Display the inner (notated) quantity.
    Inner,
    /// Display both inner and outer quantities.
    Both,
    /// Display neither.
    None,
}

/// A three-state flag used in contexts where a feature can be explicitly enabled,
/// disabled, or left to automatic determination.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/yes-no-auto/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YesNoAuto {
    Yes,
    No,
    Auto,
}

/// A spacer element in a sequence that occupies rhythmic time without producing a note
/// or rest glyph. Useful for alignment and padding purposes.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/space/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    /// The duration occupied by this space in the sequence.
    pub duration: NoteValue,
}

/// A tuplet — a group of notes that occupies a different amount of time than their
/// notated values would normally suggest (e.g., a triplet of quarter notes in the space
/// of two quarter notes).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/tuplet/>
#[derive(Debug, Clone, PartialEq)]
pub struct Tuplet {
    /// The events and nested structures within this tuplet, in order.
    pub content: Vec<SequenceContent>,
    /// The notated duration of the enclosed content (e.g., three quarter notes for a
    /// quarter-note triplet). This is the "inner" ratio value.
    pub inner: NoteValueQuantity,
    /// How much time the entire tuplet occupies in its parent sequence (e.g., two quarter
    /// notes for a quarter-note triplet). This is the "outer" ratio value.
    pub outer: NoteValueQuantity,
    /// Controls whether a bracket is drawn over the tuplet. Defaults to "auto" (bracket
    /// shown only when notes are not beamed).
    pub bracket: Option<YesNoAuto>,
    /// Controls display of the quantity ratio number(s). Defaults to `Inner`.
    pub show_number: Option<TupletDisplaySetting>,
    /// Controls display of the note value unit. Defaults to `None`.
    pub show_value: Option<TupletDisplaySetting>,
    /// The default orientation of notes within this tuplet. Inherited from ancestors
    /// when not specified.
    pub orient: Option<Orientation>,
    /// Staff assignment, primarily for cross-staff keyboard notation overrides.
    pub staff: Option<u8>,
}

/// A multi-note tremolo — rapid alternation between two or more notes.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/multi-note-tremolo/>
#[derive(Debug, Clone, PartialEq)]
pub struct MultiNoteTremolo {
    /// The events within the tremolo. Typically exactly two events.
    /// Each event's duration represents the displayed notehead value.
    pub content: Vec<Event>,
    /// The number of beams used to notate the tremolo. Must be between 1 and 8.
    pub marks: u8,
    /// How much time the entire tremolo occupies in its containing sequence.
    /// The `multiple` must equal the total number of events within the tremolo.
    pub outer: NoteValueQuantity,
    /// The performed duration of each individual event. Usually derived automatically,
    /// but may be specified explicitly.
    pub individual_duration: Option<NoteValue>,
}

/// A grace note group — one or more un-metered ornament notes preceding a main event.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/grace/>
#[derive(Debug, Clone, PartialEq)]
pub struct Grace {
    /// The ornament events. Each event's `duration` represents the *displayed* notehead
    /// value (e.g., eighth note grace notes use `"eighth"`), not a performed duration.
    pub content: Vec<Event>,
    /// How the grace notes interact with surrounding events in terms of performed timing.
    /// Defaults to `StealPrevious` when not specified.
    pub grace_type: Option<GraceType>,
    /// Whether a diagonal slash is drawn through the grace note stem(s).
    /// Defaults to true when not specified.
    pub slash: Option<bool>,
    /// Optional CSS-style color for rendering.
    pub color: Option<String>,
}

/// The content items that can appear in a sequence or tuplet.
///
/// This enum corresponds to the polymorphic array described in the MNX spec for
/// sequence and tuplet content.
#[derive(Debug, Clone, PartialEq)]
pub enum SequenceContent {
    /// A metered musical event (notes, chord, or rest).
    Event(Event),
    /// An un-metered group of grace notes.
    Grace(Grace),
    /// A tuplet grouping with an irregular rhythmic ratio.
    Tuplet(Box<Tuplet>),
    /// A rhythmic spacer that occupies time without a visible glyph.
    Space(Space),
    /// A rapid alternation between two or more notes.
    MultiNoteTremolo(MultiNoteTremolo),
}

/// A metered musical event — a chord (one or more notes), a rest, or both.
///
/// Each event occupies a specific rhythmic duration within its sequence. An event
/// without `notes` and with `rest` is a rest; an event with `notes` is a chord
/// (or single note when `notes` has one element).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/event/>
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// The rhythmic duration of this event.
    pub duration: NoteValue,
    /// The notes sounded during this event. When absent (and `rest` is set), this is
    /// a rest event.
    pub notes: Option<Vec<Note>>,
    /// Rest notation for this event. An event may have both notes and a rest only in
    /// special cases (e.g., cue notes above a rest).
    pub rest: Option<Rest>,
    /// Articulation and notation markings applying to the entire event.
    pub markings: Option<EventMarkings>,
    /// Slurs beginning at this event.
    pub slurs: Option<Vec<Slur>>,
    /// Lyrics assigned to this event.
    pub lyrics: Option<Lyrics>,
    /// A fermata placed over or under this event.
    pub fermata: Option<crate::model::mnx::global::Fermata>,
    /// Percussion kit notes within this event.
    pub kit_notes: Option<Vec<KitNote>>,
    /// Overrides the default stem orientation for this event.
    pub orient: Option<Orientation>,
    /// Whether the stem points up or down for the note(s) in this event.
    pub stem_direction: Option<StemDirection>,
    /// Overrides the default staff assignment for this event.
    pub staff: Option<u8>,
    /// Unique identifier for this event, referenced by slurs and beams.
    pub id: Option<MnxId>,
}
