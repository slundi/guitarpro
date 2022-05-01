use fraction::ToPrimitive;

use crate::io::*;

pub const _MAX_LYRICS_LINE_COUNT: u8 = 5;

/// Struct to keep lyrics
/// On guitar pro files (gp4 or later), you can have 5 lines of lyrics.
/// It is store on a BTreeMap:
/// * the key is the mesure number. The start mesure is 1
/// * the value is the text. Syntax:
///   * " " (spaces or carry returns): separates the syllables of a word
///   * "+": merge two syllables for the same beat
///   * "\[lorem ipsum...\]": hidden text
#[derive(Debug,Clone,Default)]
pub struct Lyrics {
    pub track_choice: u8,
    pub lines: Vec<(u8, u16, String)>,
}
//impl Default for Lyrics { fn default() -> Self { Lyrics { track_choice: 0, line1: BTreeMap::new(), line2: BTreeMap::new(), line3: BTreeMap::new(), line4: BTreeMap::new(), line5: BTreeMap::new(), }}}
impl std::fmt::Display for Lyrics {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::new();
        for l in &self.lines { s.push_str(&l.2); s.push('\n'); }
        write!(f, "{}", s.trim().replace('\n', " ").replace('\r', " "))
    }
}

impl crate::gp::Song {
    /// Read lyrics.
    ///
    /// First, read an `i32` that points to the track lyrics are bound to. Then it is followed by 5 lyric lines. Each one consists of
    /// number of starting measure encoded in`i32` and`int-size-string` holding text of the lyric line.
    pub(crate) fn read_lyrics(&self, data: &[u8], seek: &mut usize) -> Lyrics {
        let mut lyrics = Lyrics{track_choice: read_int(data, seek).to_u8().unwrap(), ..Default::default()};
        for i in 0..5u8 {
            let starting_measure = read_int(data, seek).to_u16().unwrap();
            lyrics.lines.push((i, starting_measure, read_int_size_string(data, seek)));
        }
        lyrics
    }
    pub(crate) fn write_lyrics(&self, data: &mut Vec<u8>) {
        write_i32(data, self.lyrics.track_choice.to_i32().unwrap());
        for i in 0..5 {
            write_i32(data, self.lyrics.lines[i].1.to_i32().unwrap());
            write_int_byte_size_string(data, &self.lyrics.lines[i].2);
        }
    }
}