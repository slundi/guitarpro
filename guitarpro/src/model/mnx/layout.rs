use crate::model::mnx::MnxId;

/// The visual symbol drawn along the left edge of a staff or staff group.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-symbol/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffSymbol {
    /// No symbol.
    None,
    /// A brace `{` connecting grand-staff systems (e.g., piano).
    Brace,
    /// A bracket connecting groups of similar instruments.
    Bracket,
    /// A square bracket.
    Square,
}

/// How barlines are drawn within a staff group.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-group-barline-style/>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffGroupBarlineStyle {
    /// Each instrument in the group draws its own barline (default).
    Instrument,
    /// Barlines span all staves in the group continuously.
    Continuous,
}

/// A reference to a part's name for display to the left of a staff in a system layout.
///
/// Using a label reference avoids duplicating the part name text in the layout section.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-labelref/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffLabelRef {
    /// The ID of the part whose name should be shown to the left of this staff.
    pub part: MnxId,
    /// When `true`, the short name of the part is used (e.g., "Gtr." instead of "Guitar").
    pub short: Option<bool>,
}

/// Identifies which part (and optionally which staff within that part) provides the
/// musical content for a layout staff.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-source/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffSource {
    /// The ID of the part that contributes to this layout staff.
    pub part: MnxId,
    /// The staff index within the part (for multi-staff parts such as piano). Defaults to 1.
    pub staff: Option<u8>,
}

/// A single staff line in a system layout, representing the visual display of one or
/// more musical parts.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutStaff {
    /// The parts (and their staff indices) that contribute to this staff line.
    pub sources: Vec<StaffSource>,
    /// An optional name to display to the left of this staff.
    pub label: Option<String>,
    /// A reference to a part's name to display to the left of this staff, avoiding
    /// duplication of the name text in the layout.
    pub labelref: Option<StaffLabelRef>,
    /// The visual symbol along the left edge of this staff. Defaults to `None`.
    pub symbol: Option<StaffSymbol>,
}

/// The content of a system layout — either a staff group or an individual staff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemLayoutContent {
    /// A named group of staves, optionally with a bracket or brace symbol.
    Group(StaffGroup),
    /// A single staff line.
    Staff(LayoutStaff),
}

/// A named group of staves in a system layout, optionally enclosed with a bracket or brace.
///
/// Staff groups can be nested to represent complex score layouts (e.g., a bracket
/// containing a piano brace).
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/staff-group/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffGroup {
    /// The nested staff groups and individual staves within this group.
    pub content: Vec<SystemLayoutContent>,
    /// How barlines are drawn within this group. Defaults to `Instrument`.
    pub barline_style: Option<StaffGroupBarlineStyle>,
    /// An optional name displayed to the left of this staff group.
    pub label: Option<String>,
    /// The visual symbol along the left edge of this group. Defaults to `None`.
    pub symbol: Option<StaffSymbol>,
}

/// A system layout definition — describes the arrangement of staves in a system.
///
/// Layouts are defined once and referenced by ID from scores, pages, and systems,
/// allowing a single layout to be reused across multiple systems.
///
/// See: <https://w3c-cg.github.io/mnx/docs/mnx-reference/objects/system-layout/>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemLayout {
    /// The top-level staff groups and staves in this layout.
    pub content: Vec<SystemLayoutContent>,
    /// Unique identifier for this layout, used when referencing it from scores or systems.
    pub id: Option<MnxId>,
}
