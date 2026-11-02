use crate::error::{GpResult, ToPrimitiveGp};
use crate::{io::primitive::*, model::legacy::song::*};

pub const _MAX_LYRICS_LINE_COUNT: u8 = 5;

/// Struct to keep lyrics
/// On guitar pro files (gp4 or later), you can have 5 lines of lyrics.
/// It is store on a BTreeMap:
/// * the key is the measure number. The start measure is 1
/// * the value is the text. Syntax:
///   * " " (spaces or carry returns): separates the syllables of a word
///   * "+": merge two syllables for the same beat
///   * "\[lorem ipsum...\]": hidden text
#[derive(Debug, Clone, Default)]
pub struct Lyrics {
    pub track_choice: u8,
    pub lines: Vec<(u8, u16, String)>,
}
//impl Default for Lyrics { fn default() -> Self { Lyrics { track_choice: 0, line1: BTreeMap::new(), line2: BTreeMap::new(), line3: BTreeMap::new(), line4: BTreeMap::new(), line5: BTreeMap::new(), }}}
impl std::fmt::Display for Lyrics {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::new();
        for l in &self.lines {
            s.push_str(&l.2);
            s.push('\n');
        }
        write!(f, "{}", s.trim().replace(['\n', '\r'], " "))
    }
}

pub trait SongLyricOps {
    fn read_lyrics(&self, data: &[u8], seek: &mut usize) -> GpResult<Lyrics>;
    fn write_lyrics(&self, data: &mut Vec<u8>) -> GpResult<()>;
}

impl SongLyricOps for Song {
    /// Read lyrics.
    ///
    /// First, read an `i32` that points to the track lyrics are bound to. Then it is followed by 5 lyric lines. Each one consists of
    /// number of starting measure encoded in`i32` and`int-size-string` holding text of the lyric line.
    fn read_lyrics(&self, data: &[u8], seek: &mut usize) -> GpResult<Lyrics> {
        // Some files store `-1` here to mean "no lyric track"; the field is
        // purely metadata about which track carries the lyric, so accept any
        // int and clamp into u8.
        let mut lyrics = Lyrics {
            track_choice: read_int(data, seek)? as u8,
            ..Default::default()
        };
        for i in 0..5u8 {
            let starting_measure = read_int(data, seek)?.max(0) as u16;
            lyrics
                .lines
                .push((i, starting_measure, read_int_size_string(data, seek)?));
        }
        Ok(lyrics)
    }
    fn write_lyrics(&self, data: &mut Vec<u8>) -> GpResult<()> {
        write_i32(
            data,
            self.lyrics.track_choice.to_i32_gp("lyrics track_choice")?,
        );
        // The format reserves exactly five lyric slots. `read_lyrics` always
        // fills all five, but a `Song` produced by a converter (MusicXML,
        // MSCZ, the optimized model) carries `Lyrics::default()` with an
        // empty `lines`, so pad the missing slots with the empty line Guitar
        // Pro itself writes for an unused one.
        for index in 0.._MAX_LYRICS_LINE_COUNT as usize {
            let (measure, text) = match self.lyrics.lines.get(index) {
                Some((_, measure, text)) => (*measure, text.as_str()),
                None => (0, ""),
            };
            write_i32(data, measure.to_i32_gp("lyrics line measure")?);
            write_int_size_string(data, text);
        }
        Ok(())
    }
}
