//! MSCX (MuseScore XML) high-level AST.
//!
//! Only the elements that Part 1 of the MSCZ roadmap cares about are strongly
//! typed: envelope version, meta tags, part / staff / instrument summary and
//! per-staff measure counts. Everything else lives in `raw_xml`, which stays
//! the source-of-truth for byte-identical round-trips.
//!
//! As converters land in Part 2, more of this structure will be populated
//! directly from the XML and eventually the raw-XML field can be dropped.

/// The parsed high-level view of a `score.mscx` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mscx {
    /// The full MSCX XML content, preserved verbatim.
    ///
    /// This is what `write_mscx` reserializes. Structural mutations should
    /// be performed by higher-level converters that regenerate this string.
    pub raw_xml: String,

    /// `<museScore version="X.Y">` attribute, e.g. `"4.10"`.
    pub version: String,

    /// `<programVersion>` element, e.g. `"4.1.1"`.
    pub program_version: Option<String>,

    /// `<programRevision>` element (git commit hash).
    pub program_revision: Option<String>,

    /// `<Division>` — MIDI-style ticks per quarter note.
    pub division: Option<u32>,

    /// All `<metaTag name="…">value</metaTag>` entries, in order.
    pub meta_tags: Vec<MetaTag>,

    /// One entry per top-level `<Part>` element under `<Score>`.
    pub parts: Vec<Part>,

    /// Per-`<Staff id="…">` measure count, extracted from
    /// `<Score>/<Staff id="…">` blocks. Vec index corresponds to the order
    /// staves appeared. Empty when the parser could not find any staff.
    pub measure_counts: Vec<StaffMeasureCount>,
}

impl Mscx {
    /// Retrieve a meta tag by name (case-sensitive), returning its value.
    pub fn meta(&self, name: &str) -> Option<&str> {
        self.meta_tags
            .iter()
            .find(|tag| tag.name == name)
            .map(|tag| tag.value.as_str())
    }
}

/// One `<metaTag name="…">value</metaTag>` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaTag {
    pub name: String,
    pub value: String,
}

/// A `<Part id="…">` block under `<Score>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    /// `id` attribute (numeric string in MuseScore 4.x).
    pub id: String,
    /// `<trackName>` element on the part, if present.
    pub track_name: Option<String>,
    /// One entry per `<Staff id="…">` under the part.
    pub staves: Vec<Staff>,
    /// The `<Instrument>` block for this part, if present.
    pub instrument: Option<Instrument>,
}

/// A `<Staff id="…">` block under `<Part>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staff {
    /// `id` attribute.
    pub id: String,
    /// `<StaffType group="…">` attribute (e.g. `pitched`, `tablature`).
    pub group: Option<String>,
    /// `<StaffType>/<name>` (e.g. `stdNormal`, `tab6StrCommon`).
    pub type_name: Option<String>,
    /// `<defaultClef>` value (e.g. `G8vb`).
    pub default_clef: Option<String>,
}

/// A `<Part>/<Instrument>` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    /// `id` attribute on the `<Instrument>` element.
    pub id: String,
    /// `<longName>`.
    pub long_name: Option<String>,
    /// `<shortName>`.
    pub short_name: Option<String>,
    /// `<trackName>` inside `<Instrument>` (often empty).
    pub track_name: Option<String>,
    /// `<transposeDiatonic>`.
    pub transpose_diatonic: Option<i8>,
    /// `<transposeChromatic>`.
    pub transpose_chromatic: Option<i8>,
    /// `<instrumentId>` (e.g. `pluck.guitar.electric`).
    pub instrument_id: Option<String>,
    /// `<StringData>` block, if the instrument declares one.
    pub string_data: Option<StringData>,
}

/// A `<StringData>` block on an instrument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringData {
    /// `<frets>` value.
    pub frets: Option<u8>,
    /// One MIDI note per `<string>` child, in file order (low string first).
    pub strings: Vec<u8>,
}

/// Per-staff measure count captured from `<Score>` streaming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffMeasureCount {
    /// `id` attribute on the `<Staff>` element under `<Score>`.
    pub staff_id: String,
    /// Number of `<Measure>` children encountered.
    pub measure_count: u32,
}
