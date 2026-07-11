//! Conversion between the MSCZ [`Mscx`] view and [`optimized::LoadedScore`].
//!
//! The current implementation covers **structural** conversion:
//! * Score metadata (title, composer, copyright, provenance).
//! * Instrument definitions with tuning (from `<StringData>`) and
//!   `<transposeChromatic>` / `<transposeDiatonic>`.
//! * Staff definitions (clef + notation/tab display) from `<Part>/<Staff>`.
//! * Global timeline: per-measure `<TimeSig>`, `<KeySig>`, `<Tempo>` and
//!   `<startRepeat>` / `<endRepeat>` navigation.
//! * Per-track measure data: voices, beats (Chord/Rest) with duration and
//!   augmentation dots, and notes with `<pitch>`, `<string>` and `<fret>`.
//! * Ties (start/end) via `<Spanner type="Tie">`.
//!
//! Elements that the parser sees but does not fully represent (dynamics,
//! articulations, ornaments, cross-staff notation, lyrics, layout hints,
//! GPX-style guitar techniques, …) are recorded in [`LossReport`] so the
//! caller can act on the gap.
//!
//! Not yet implemented (deferred to later roadmap iterations):
//! * `<Beam>` grouping, tuplets, grace notes.
//! * Chord symbols and fret diagrams.
//! * `<HairPin>`, `<Slur>`, `<Trill>`, `<Volta>` spanners.
//! * Track groups (`<PartGroup>` in the MusicXML sense).

pub mod from_optimized;
pub mod to_optimized;
pub mod validate;

pub use from_optimized::loaded_score_to_mscx;
pub use to_optimized::{ConvertOutcome, mscx_to_loaded_score};
pub use validate::LossReport;
