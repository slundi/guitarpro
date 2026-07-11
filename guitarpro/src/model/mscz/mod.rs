//! MuseScore compressed archive AST (`.mscz`).
//!
//! Layered representation:
//! * [`MsczFile`] — top-level object: parsed [`Mscx`] view plus the raw
//!   [`MsczArchive`] (all archive entries preserved byte-for-byte).
//! * [`Mscx`] — high-level view over `score.mscx`: envelope version,
//!   `metaTag`s, part/staff/instrument summary and per-staff measure counts.
//!   The **source-of-truth is `Mscx::raw_xml`**; the structured fields are
//!   extractors populated by [`crate::io::mscz::parse_mscx`] and are used
//!   by higher-level converters (Part 2 of the roadmap).
//!
//! This split keeps Part 1 honest: round-trips are byte-identical because the
//! raw XML and every non-mscx side file (`score_style.mss`, thumbnails, JSON
//! settings…) are preserved verbatim.

pub mod mscx;

pub use mscx::{Instrument, MetaTag, Mscx, Part, Staff, StaffMeasureCount, StringData};

/// A single entry inside an MSCZ archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsczEntry {
    /// Path inside the archive (forward-slash separated, matching ZIP).
    pub path: String,
    /// Raw bytes for the entry, preserved verbatim.
    pub data: Vec<u8>,
}

/// A parsed MSCZ archive, with all entries kept verbatim.
///
/// Enough state to write out a byte-stable copy without going through the
/// [`Mscx`] AST — used by tests to prove the container layer is lossless.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MsczArchive {
    /// Paths listed under `<rootfile full-path="…"/>` in
    /// `META-INF/container.xml`, in the order they appeared. Empty for
    /// archives that lack a manifest.
    pub rootfiles: Vec<String>,
    /// All entries in the archive, in original ZIP order.
    pub entries: Vec<MsczEntry>,
}

impl MsczArchive {
    /// Return the first entry whose path matches `path`, or `None`.
    pub fn find(&self, path: &str) -> Option<&MsczEntry> {
        self.entries.iter().find(|entry| entry.path == path)
    }

    /// Return the primary `.mscx` entry.
    ///
    /// Preference order:
    /// 1. First `rootfile` from `container.xml` ending in `.mscx`.
    /// 2. Any `*.mscx` entry outside `META-INF/`.
    pub fn mscx_entry(&self) -> Option<&MsczEntry> {
        for path in &self.rootfiles {
            if path.ends_with(".mscx")
                && let Some(entry) = self.find(path)
            {
                return Some(entry);
            }
        }
        self.entries
            .iter()
            .find(|entry| entry.path.ends_with(".mscx") && !entry.path.starts_with("META-INF/"))
    }

    /// Return the style file (`.mss`) if present.
    pub fn style_entry(&self) -> Option<&MsczEntry> {
        self.entries
            .iter()
            .find(|entry| entry.path.ends_with(".mss"))
    }

    /// Return `Thumbnails/thumbnail.png` if present.
    pub fn thumbnail_entry(&self) -> Option<&MsczEntry> {
        self.find("Thumbnails/thumbnail.png")
    }

    /// Return `audiosettings.json` if present.
    pub fn audio_settings_entry(&self) -> Option<&MsczEntry> {
        self.find("audiosettings.json")
    }

    /// Return `viewsettings.json` if present.
    pub fn view_settings_entry(&self) -> Option<&MsczEntry> {
        self.find("viewsettings.json")
    }
}

/// A complete MSCZ document: the raw archive plus the parsed [`Mscx`] view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsczFile {
    /// Preserved archive (every entry retained verbatim).
    pub archive: MsczArchive,
    /// Parsed high-level view of the primary `.mscx` file.
    pub mscx: Mscx,
}
