//! Conversion between the MSCZ [`Mscx`](crate::model::mscz::Mscx) view and
//! [`LoadedScore`](crate::model::optimized::LoadedScore).
//!
//! The converter covers:
//! * Score metadata (title, composer, copyright, provenance).
//! * Instrument definitions with tuning (from `<StringData>`) and
//!   `<transposeChromatic>` / `<transposeDiatonic>`; instruments' MuseScore
//!   template, sound and GM program.
//! * Staff definitions (clef + notation/tab display) from `<Part>/<Staff>`.
//! * Global timeline: per-measure `<TimeSig>`, `<KeySig>`, `<Tempo>`,
//!   `<startRepeat>` / `<endRepeat>`, `<Jump>` / `<Marker>` navigation,
//!   barline styles and `<Spanner type="Volta">` endings.
//! * Per-track measure data: voices, beats (Chord/Rest, whole-measure rests
//!   sized to their measure) with duration and augmentation dots, grace
//!   chords, and notes with `<pitch>`, `<string>` and `<fret>`.
//! * Ties via `<Spanner type="Tie">` (chains included), slurs via
//!   `<Spanner type="Slur">` inside the chords they join, hairpins.
//! * Dynamics, chord symbols (`<Harmony>`), fermatas, breath marks,
//!   articulations, ornaments, single-chord tremolos and chord lines.
//! * Clefs (including octave clefs; a change in mid-measure holds from the
//!   next measure on, and clefs the model has no type for are approximated);
//!   transposing staves' written keys and spellings (`<actualKey>`,
//!   `<tpc2>`); noteheads.
//!
//! What the converter sees but can't represent (tuplets, a part's further
//! staves, mid-measure clef positions, approximated clefs, irregular
//! measure lengths, …) is counted in [`LossReport`] so the caller can act
//! on the gap; elements it doesn't know at all are kept only in the raw
//! XML.
//!
//! Not yet implemented:
//! * `<Beam>` grouping and tuplets.
//! * Fret diagrams, lyrics, `<Trill>` and other line spanners.
//! * Track groups (`<PartGroup>` in the MusicXML sense).

pub mod from_optimized;
pub(crate) mod instruments;
pub(crate) mod notation;
pub(crate) mod spelling;
pub mod to_optimized;

pub use crate::convert::loss::LossReport;
pub use from_optimized::loaded_score_to_mscx;
pub use to_optimized::{ConvertOutcome, mscx_to_loaded_score};
