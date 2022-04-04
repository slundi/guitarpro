use crate::beat::*;

/// An enumeration of available clefs
#[derive(Clone)]
pub enum MeasureClef { Treble, Bass, Tenor, Alto }
/// A line break directive: `NONE: no line break`, `BREAK: break line`, `Protect the line from breaking`.
#[derive(Clone)]
pub enum LineBreak { None, Break, Protect }

/// "A measure contains multiple voices of beats
#[derive(Clone)]
pub struct Measure {
    pub track_index: usize,
    pub header_index: usize,
    pub clef: MeasureClef,
    /// Max voice count is 2
    pub voices: Vec<Voice>, 
    pub line_break: LineBreak,
}
impl Default for Measure {fn default() -> Self { Measure {track_index: 0, header_index: 0, clef: MeasureClef::Treble, voices: Vec::with_capacity(2), line_break: LineBreak::None }}}

