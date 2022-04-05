use fraction::ToPrimitive;

use crate::{io::*, gp::*, key_signature::*, enums::*};

#[derive(Clone)]
pub struct Version {
    pub data: String,
    pub number: u8,
    pub clipboard: bool
}

pub const _VERSION_1_0X: u8 = 10;
pub const _VERSION_2_2X: u8 = 22;
pub const VERSION_3_00: u8 = 30;
pub const VERSION_4_0X: u8 = 40;
pub const VERSION_5_00: u8 = 50;
pub const VERSION_5_10: u8 = 51;

/// Read and process version
pub fn read_version(data: &Vec<u8>, seek: &mut usize, song: &mut Song) {
    song.version = read_version_string(data, seek);
    let mut clipboard = Clipboard::default();
    //check for clipboard and read it
    if song.version.number == VERSION_4_0X && song.version.clipboard {
        clipboard.start_measure = read_int(data, seek);
        clipboard.stop_measure  = read_int(data, seek);
        clipboard.start_track = read_int(data, seek);
        clipboard.stop_track  = read_int(data, seek);
    }
    if song.version.number == VERSION_5_00 && song.version.clipboard {
        clipboard.start_beat = read_int(data, seek);
        clipboard.stop_beat  = read_int(data, seek);
        clipboard.sub_bar_copy = read_int(data, seek) != 0;
    }
}

#[derive(Clone)]
pub struct Clipboard {
    pub start_measure: i32,
    pub stop_measure: i32,
    pub start_track: i32,
    pub stop_track: i32,
    pub start_beat: i32,
    pub stop_beat: i32,
    pub sub_bar_copy: bool
}
impl Default for Clipboard {
	fn default() -> Self { Clipboard {start_measure: 1, stop_measure: 1, start_track: 1, stop_track: 1, start_beat: 1, stop_beat: 1, sub_bar_copy: false} }
}

#[derive(Clone)]
pub struct MeasureHeader {
    pub number: u16,
	pub start: i64,
	pub time_signature: TimeSignature,
	pub tempo: i32,
	pub marker: Marker,
	pub repeat_open: bool,
	pub repeat_alternative: i8,
	pub repeat_close: i8,
	pub triplet_feel: TripletFeel,
    /// Tonality of the measure
    pub key_signature: KeySignature,
    pub double_bar: bool,
}
impl Default for MeasureHeader {
    fn default() -> Self { MeasureHeader {
        number: 1,
        start: DURATION_QUARTER_TIME,
        tempo: 0,
        repeat_open: false,
        repeat_alternative: 0,
        repeat_close: -1,
        triplet_feel: TripletFeel::NONE,
        key_signature: KeySignature::default(),
        double_bar: false,
        marker: Marker::default(),
        time_signature: TimeSignature {numerator: 4, denominator: Duration::default(), beams: vec![2, 2, 2, 2]}, //TODO: denominator
    }}
}
impl MeasureHeader {
    pub fn length(&self) -> i64 {return (self.time_signature.numerator as i64) * (self.time_signature.denominator.time() as i64);}
    pub fn end(&self) -> i64 {return self.start + self.length();}
}

/// Read measure header. The first byte is the measure's flags. It lists the data given in the current measure.
/// 
/// | **Bit 7** | **Bit 6** | **Bit 5** | **Bit 4** | **Bit 3** | **Bit 2** | **Bit 1** | **Bit 0** |
/// |-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
/// | Presence of a double bar  | Tonality of the measure  | Presence of a marker  | Number of alternate ending | End of repeat | Beginning of repeat | Denominator of the (key) signature | Numerator of the (key) signature |
///
/// Each of these elements is present only if the corresponding bit is a 1. The different elements are written (if they are present) from lowest to highest bit.  
/// Exceptions are made for the double bar and the beginning of repeat whose sole presence is enough, complementary data is not necessary.

/// * **Numerator of the (key) signature**: `byte`. Numerator of the (key) signature of the piece
/// * **Denominator of the (key) signature**: `byte`. Denominator of the (key) signature of the piece
/// * **End of repeat**: `byte`. Number of repeats until the previous Beginning of repeat. Nombre de renvoi jusqu'au début de renvoi précédent.
/// * **Number of alternate ending**: `byte`. The number of alternate ending.
/// * **Marker**: The markers are written in two steps:
/// 1) First is written an `integer` equal to the marker's name length + 1
/// 2) a string containing the marker's name. Finally the marker's color is written.
/// * **Tonality of the measure**: `byte`. This value encodes a key (signature) change on the current piece. It is encoded as: `0: C`, `1: G (#)`, `2: D (##)`, `-1: F (b)`, ...
pub fn read_measure_header(data: &Vec<u8>, seek: &mut usize, song: &mut Song, number: usize) {
    //println!("N={}\tmeasure_headers={}", number, song.measure_headers.len());
    let flag = read_byte(data, seek);
    let mut mh = MeasureHeader::default();
    mh.number = number as u16;
    mh.start  = 0;
    mh.triplet_feel = song.triplet_feel.clone();
    //we need a previous header for the next 2 flags
    //Numerator of the (key) signature
    if (flag & 0x01 )== 0x01 {mh.time_signature.numerator = read_signed_byte(data, seek);}
    else if number < song.measure_headers.len() {mh.time_signature.numerator = song.measure_headers[number-1].time_signature.numerator;}
    //Denominator of the (key) signature
    if (flag & 0x02) == 0x02 {mh.time_signature.denominator = read_duration(data, seek, flag);}
    else if number < song.measure_headers.len() {mh.time_signature.denominator = song.measure_headers[number-1].time_signature.denominator.clone();}

    mh.repeat_open = (flag & 0x04) == 0x04; //Beginning of repeat
    if (flag & 0x08) == 0x08 {mh.repeat_close = read_signed_byte(data, seek);} //End of repeat
    if (flag & 0x10) == 0x10 {mh.repeat_alternative = read_repeat_alternative(data, seek, &mut song.measure_headers);} //Number of alternate endin
    if (flag & 0x20) == 0x20 {read_marker(data, seek, &mut mh.marker);} //Presence of a marker
    if (flag & 0x40) == 0x40 { //Tonality of the measure 
        mh.key_signature.key = read_signed_byte(data, seek);
        mh.key_signature.is_minor = read_signed_byte(data, seek) != 0;
    } else if mh.number > 1 && number < song.measure_headers.len() {mh.key_signature = song.measure_headers[number-1].key_signature.clone();}
    mh.double_bar = (flag & 0x80) == 0x80; //presence of a double bar
    song.measure_headers.push(mh);
}

fn read_repeat_alternative(data: &Vec<u8>, seek: &mut usize, measure_headers: &mut Vec<MeasureHeader>) -> i8 {
    let value = read_byte(data, seek);
    let mut existing_alternative = 0i8;
    for i in measure_headers.len()-1 .. 0 {
        if measure_headers[i].repeat_open {break;}
        existing_alternative |= measure_headers[i].repeat_alternative;
    }
    return (1 << value) - 1 ^ existing_alternative;
}

/// A marker annotation for beats.
#[derive(Clone)]
pub struct Marker {
    pub title: String,
    pub color: i32,
}
impl Default for Marker {fn default() -> Self { Marker {title: "Section".to_owned(), color: 0xff0000}}}
/// Read a marker. The markers are written in two steps:
/// - first is written an integer equal to the marker's name length + 1
/// - then a string containing the marker's name. Finally the marker's color is written.
fn read_marker(data: &Vec<u8>, seek: &mut usize, marker: &mut Marker) {
    marker.title = read_int_size_string(data, seek);
    marker.color = read_color(data, seek);
}

/// This class can store the information about a group of measures which are repeated.
#[derive(Clone)]
pub struct RepeatGroup {
    /// List of measure header indexes.
    pub measure_headers: Vec<usize>,
    pub closings: Vec<usize>,
    pub openings: Vec<usize>,
    pub is_closed: bool,
}
impl Default for RepeatGroup {fn default() -> Self { RepeatGroup {
    measure_headers: Vec::new(),
    closings: Vec::new(),
    openings: Vec::new(),
    is_closed: false,
}}}
impl RepeatGroup {
    pub fn add_measure_header(&mut self, measure_header: &MeasureHeader) {
        let index = measure_header.number.to_usize().unwrap();
        if self.openings.len() == 0 {self.openings.push(index);} //if not len(self.openings): self.openings.append(h)
        self.measure_headers.push(index);
        if measure_header.repeat_close > 0 {
            self.closings.push(index);
            self.is_closed = true;
        } else { //A new item after the header was closed? -> repeat alternative, reopens the group
            self.is_closed = false;
            self.openings.push(index);
        }
    }
}
