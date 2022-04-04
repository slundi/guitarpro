use std::collections::BTreeMap;

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
#[derive(Clone)]
pub struct Lyrics {
    pub track_choice: u8,
    pub line1: BTreeMap<u16, String>,
    pub line2: BTreeMap<u16, String>,
    pub line3: BTreeMap<u16, String>,
    pub line4: BTreeMap<u16, String>,
    pub line5: BTreeMap<u16, String>,
}
impl Default for Lyrics { fn default() -> Self { Lyrics { track_choice: 0, line1: BTreeMap::new(), line2: BTreeMap::new(), line3: BTreeMap::new(), line4: BTreeMap::new(), line5: BTreeMap::new(), }}}
impl Lyrics {
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for l in &self.line1 { s.push_str(l.1); s.push('\n'); }
        for l in &self.line2 { s.push_str(l.1); s.push('\n'); }
        for l in &self.line3 { s.push_str(l.1); s.push('\n'); }
        for l in &self.line4 { s.push_str(l.1); s.push('\n'); }
        for l in &self.line5 { s.push_str(l.1); s.push('\n'); }
        return s.trim().replace('\n', " ").replace('\r', " ");
    }
}
/// Read lyrics.
///
/// First, read an `i32` that points to the track lyrics are bound to. Then it is followed by 5 lyric lines. Each one consists of
/// number of starting measure encoded in`i32` and`int-size-string` holding text of the lyric line.
pub fn read_lyrics(data: &Vec<u8>, seek: &mut usize) -> Lyrics {
    let mut lyrics = Lyrics::default();
    lyrics.track_choice = read_int(data, seek).to_u8().unwrap();
    println!("Lyrics for track #{}", lyrics.track_choice);
    lyrics.line1.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
    lyrics.line2.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
    lyrics.line3.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
    lyrics.line4.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
    lyrics.line5.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
    return lyrics;
}
