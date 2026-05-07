use std::collections::HashMap;

/// The position of a lyric syllable within a word.
///
/// Used to determine hyphenation and vocal synthesis behavior.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/event-lyric-line-type/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLyricLineType {
    /// The syllable begins a multi-syllable word.
    Start,
    /// The syllable is in the middle of a multi-syllable word.
    Middle,
    /// The syllable ends a multi-syllable word.
    End,
    /// The syllable constitutes an entire word on its own.
    Whole,
}

/// A single lyric syllable assigned to a specific lyric line of an event.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/event-lyric-line/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLyricLine {
    /// The text of the syllable. May be an empty string when a lyric line is skipped
    /// for a given verse.
    pub text: String,
    /// The syllable's position within its word. Defaults to `Whole` when not provided.
    pub lyric_type: Option<EventLyricLineType>,
}

/// The lyric content for a single sung event, typically corresponding to a single syllable.
///
/// The `lines` map uses user-defined keys (e.g., `"1"`, `"2"`, `"A"`) to identify
/// individual lyric lines (verses).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/lyrics/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lyrics {
    /// A mapping from lyric line IDs to syllable data for this event.
    pub lines: Option<HashMap<String, EventLyricLine>>,
}

/// Global metadata for a single lyric line.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/lyric-line-metadata/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLineMetadata {
    /// An optional human-readable label for this lyric line (e.g., "Verse 1", "Chorus").
    pub label: Option<String>,
}

/// Global lyrics data for the entire score, providing ordering and metadata for lyric lines.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/lyrics-global/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricsGlobal {
    /// An object mapping lyric line IDs to their global metadata.
    pub line_metadata: Option<HashMap<String, LyricLineMetadata>>,
    /// An ordered list of all lyric line IDs used in this document, from top to bottom
    /// as they should appear visually.
    pub line_order: Option<Vec<String>>,
}
