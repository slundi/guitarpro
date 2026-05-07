use crate::model::mnx::event::{FullMeasureRest, SequenceContent};

/// A sequence of musical content belonging to a single voice within a measure.
///
/// A part-measure contains one or more sequences, each representing a distinct voice
/// (e.g., voice 1 and voice 2 on the same staff). No two sequences in a measure may
/// share the same `voice` identifier.
///
/// When `full_measure` is set, `content` must be empty — the sequence represents a
/// whole-measure rest.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/sequence/>
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    /// The ordered musical content of this sequence: events, grace notes, tuplets,
    /// spaces, and multi-note tremolos.
    pub content: Vec<SequenceContent>,
    /// When present, designates this sequence as a whole-measure rest. `content` must
    /// be empty in this case.
    pub full_measure: Option<FullMeasureRest>,
    /// The default orientation (stem direction) for content in this sequence.
    /// When absent, consuming software determines orientation automatically.
    pub orient: Option<crate::model::mnx::Orientation>,
    /// The default staff assignment for this sequence's content. Defaults to 1 (the first
    /// staff) when absent.
    pub staff: Option<u8>,
    /// An identifier for the voice this sequence belongs to. No two sequences in the same
    /// measure may share a voice identifier.
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/voice-name/>
    pub voice: Option<String>,
    /// Unique identifier for this sequence.
    pub id: Option<crate::model::mnx::MnxId>,
}
