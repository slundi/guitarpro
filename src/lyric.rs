use std::collections::BTreeMap;

use crate::io::*;


/// Struct to keep lyrics
/// On guitar pro files (gp4 or later), you can have 5 lines of lyrics.
/// It is store on a BTreeMap:
/// * the key is the mesure number
/// * the value is the text. Syntax:
///   * " " (spaces or carry returns): separates the syllables of a word
///   * "+": merge two syllables for the same beat
///   * "\[lorem ipsum...\]": hidden text
#[derive(Clone)]
pub struct Lyrics {
    pub track_choice: u8,
    pub max_line_count: u8,
    pub lyrics1: BTreeMap<u16, String>,
    pub lyrics2: BTreeMap<u16, String>,
    pub lyrics3: BTreeMap<u16, String>,
    pub lyrics4: BTreeMap<u16, String>,
    pub lyrics5: BTreeMap<u16, String>,
}
impl Default for Lyrics {
    fn default() -> Self { Lyrics { track_choice: 0, max_line_count: 5, lyrics1: BTreeMap::new(), lyrics2: BTreeMap::new(), lyrics3: BTreeMap::new(), lyrics4: BTreeMap::new(), lyrics5: BTreeMap::new(), }}
}
impl Lyrics {
    /// Read lyrics.
    ///
    /// First, read an `i32` that points to the track lyrics are bound to. Then it is followed by 5 lyric lines. Each one consists of
    /// number of starting measure encoded in`i32` and`int-size-string` holding text of the lyric line.
    pub fn read(data: &Vec<u8>, seek: &mut usize) -> Lyrics {
        let mut lyrics = Lyrics::default();
        lyrics.track_choice = read_int(data, seek) as u8;
        println!("Lyrics for track #{}", lyrics.track_choice);
        lyrics.lyrics1.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
        lyrics.lyrics2.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
        lyrics.lyrics3.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
        lyrics.lyrics4.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
        lyrics.lyrics5.insert(read_int(data, seek).try_into().unwrap(), read_int_size_string(data, seek));
        return lyrics;
    }
}
