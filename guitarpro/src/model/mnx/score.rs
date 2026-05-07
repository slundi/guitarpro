use crate::model::mnx::MnxId;

/// A multimeasure rest — a block of consecutive empty measures collapsed into a single
/// rest symbol for readability (commonly used in part scores).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/multimeasure-rest/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultimeasureRest {
    /// The 0-based index of the first measure covered by this multimeasure rest.
    pub start: u32,
    /// The number of measures covered by this multimeasure rest.
    pub duration: u32,
    /// When true, the measure count is displayed above the rest symbol.
    pub label: Option<bool>,
}

/// A system — a single row of staves spanning some measures of the score.
///
/// Systems may override the layout used by their parent score or page.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/system/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct System {
    /// ID of the system layout to use for this system, overriding the score or page
    /// default layout.
    pub layout: Option<MnxId>,
    /// The 0-based measure index where this system begins.
    pub measure: Option<u32>,
}

/// A page in a paginated score, containing system break information.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/page/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// The systems on this page.
    pub systems: Option<Vec<System>>,
    /// ID of the layout to use for all systems on this page, overriding the score
    /// default layout.
    pub layout: Option<MnxId>,
}

/// A score — a particular view of the music, such as a full score or a single-part
/// extract.
///
/// Multiple scores can coexist in one MNX document, each selecting a different subset
/// of parts or using different transpositions.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/score/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    /// The score name, suitable for display when selecting among scores (e.g.,
    /// "Full score", "Flute 1 part").
    ///
    /// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/score-name/>
    pub name: String,
    /// ID of the default layout used for all systems in this score. Individual pages
    /// and systems may override this.
    pub layout: Option<MnxId>,
    /// All multimeasure rests defined for this score.
    pub multimeasure_rests: Option<Vec<MultimeasureRest>>,
    /// Page layout and system break information.
    pub pages: Option<Vec<Page>>,
    /// When true, this score displays written (transposed) pitches rather than sounding
    /// (concert) pitches. Defaults to false when absent.
    pub use_written: Option<bool>,
    /// Unique identifier for this score.
    pub id: Option<MnxId>,
}
