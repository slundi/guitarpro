use std::collections::HashMap;

use fraction::ToPrimitive;

use crate::{io::*, gp::*, key_signature::*, enums::*};

#[derive(Debug,Clone,PartialEq)]
pub struct Version {
    pub data: String,
    pub number: AppVersion,
    pub clipboard: bool
}

#[derive(Debug,Clone)]
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

#[derive(Debug,Clone)]
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
    pub direction: Option<DirectionSign>,
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
        triplet_feel: TripletFeel::None,
        direction: None,
        key_signature: KeySignature::default(),
        double_bar: false,
        marker: Marker::default(),
        time_signature: TimeSignature {numerator: 4, denominator: Duration::default(), beams: vec![2, 2, 2, 2]},
    }}
}
impl MeasureHeader {
    pub fn length(&self) -> i64 {self.time_signature.numerator.to_i64().unwrap() * self.time_signature.denominator.time().to_i64().unwrap()}
    pub fn end(&self) -> i64 {self.start + self.length()}
}

/// A marker annotation for beats.
#[derive(Debug,Clone)]
pub struct Marker {
    pub title: String,
    pub color: i32,
}
impl Default for Marker {fn default() -> Self { Marker {title: "Section".to_owned(), color: 0xff0000}}}
/// Read a marker. The markers are written in two steps:
/// - first is written an integer equal to the marker's name length + 1
/// - then a string containing the marker's name. Finally the marker's color is written.
fn read_marker(data: &[u8], seek: &mut usize, marker: &mut Marker) {
    marker.title = read_int_size_string(data, seek);
    marker.color = read_color(data, seek);
}

/// This class can store the information about a group of measures which are repeated.
#[derive(Debug,Clone,Default)]
pub struct RepeatGroup {
    /// List of measure header indexes.
    pub measure_headers: Vec<usize>,
    pub closings: Vec<usize>,
    pub openings: Vec<usize>,
    pub is_closed: bool,
}
//impl Default for RepeatGroup {fn default() -> Self { RepeatGroup {measure_headers: Vec::new(), closings: Vec::new(), openings: Vec::new(), is_closed: false, }}}
impl RepeatGroup {
    pub fn add_measure_header(&mut self, measure_header: &MeasureHeader) {
        let index = measure_header.number.to_usize().unwrap();
        if self.openings.is_empty() {self.openings.push(index);} //if not len(self.openings): self.openings.append(h)
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

impl Song {
    /// Read and process version
    pub fn read_version(&mut self, data: &[u8], seek: &mut usize) {
        self.version = read_version_string(data, seek);
        //check for clipboard and read it
        if self.version.number != AppVersion::Version_3_00 && self.version.clipboard { self.read_clipboard(data, seek); }
    }
    fn read_clipboard(&mut self, data: &[u8], seek: &mut usize) -> Clipboard {
        let mut c = Clipboard{start_measure: read_int(data, seek), ..Default::default()};
        c.stop_measure = read_int(data, seek);
        c.start_track = read_int(data, seek);
        c.stop_track = read_int(data, seek);
        if self.version.number == AppVersion::Version_5_00 || self.version.number == AppVersion::Version_5_10 {
            c.start_beat = read_int(data, seek);
            c.stop_beat = read_int(data, seek);
            c.sub_bar_copy = read_int(data, seek) != 0;
        }
        c
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
    pub fn read_measure_header(&mut self, data: &[u8], seek: &mut usize, number: usize) -> u8 {
        //println!("N={}\tmeasure_headers={}", number, song.measure_headers.len());
        let flag = read_byte(data, seek);
        //println!("read_measure_header(), flags: {}", flag);
        let mut mh = MeasureHeader{number: number.to_u16().unwrap(), ..Default::default()};
        mh.start  = 0;
        mh.triplet_feel = self.triplet_feel.clone();
        //we need a previous header for the next 2 flags
        //Numerator of the (key) signature
        if (flag & 0x01 )== 0x01 {mh.time_signature.numerator = read_signed_byte(data, seek);}
        else if number < self.measure_headers.len() {mh.time_signature.numerator = self.measure_headers[number-1].time_signature.numerator;}
        //Denominator of the (key) signature
        if (flag & 0x02) == 0x02 {mh.time_signature.denominator.value = read_signed_byte(data, seek).to_u16().unwrap();}
        else if number < self.measure_headers.len() {mh.time_signature.denominator = self.measure_headers[number-1].time_signature.denominator.clone();}

        mh.repeat_open = (flag & 0x04) == 0x04; //Beginning of repeat
        if (flag & 0x08) == 0x08 {mh.repeat_close = read_signed_byte(data, seek);} //End of repeat
        if (flag & 0x10) == 0x10 {mh.repeat_alternative = if self.version.number == AppVersion::Version_5_00 || self.version.number == AppVersion::Version_5_10 {self.read_repeat_alternative_v5(data, seek)} else {self.read_repeat_alternative(data, seek)};} //Number of alternate endin
        if (flag & 0x20) == 0x20 {read_marker(data, seek, &mut mh.marker);} //Presence of a marker
        if (flag & 0x40) == 0x40 { //Tonality of the measure 
            mh.key_signature.key = read_signed_byte(data, seek);
            mh.key_signature.is_minor = read_signed_byte(data, seek) != 0;
        } else if mh.number > 1 && number < self.measure_headers.len() {mh.key_signature = self.measure_headers[number-1].key_signature.clone();}
        mh.double_bar = (flag & 0x80) == 0x80; //presence of a double bar
        self.measure_headers.push(mh);
        flag
    }
    /// Read measure header. Measure header format in Guitar Pro 5 differs from one if Guitar Pro 3.
    /// 
    /// First, there is a blank byte if measure is not first. Then measure header is read as in GP3's `read_measure_header_v3()`. Then measure header is read as follows:
    /// - Time signature beams: 4 `Bytes <byte>`. Appears If time signature was set, i.e. flags *0x01* and *0x02* are both set.
    /// - Blank `byte` if flag at *0x10* is set.
    /// - Triplet feel: `byte`. See `TripletFeel`.
    pub fn read_measure_header_v5(&mut self, data: &[u8], seek: &mut usize, number: usize, previous: Option<usize>) {
        if previous.is_none() { *seek += 1; } //always
        let flags = self.read_measure_header(data, seek, number);
        let last = self.measure_headers.len()-1;
        let prev = if previous.is_none() {0} else {previous.unwrap()};
        if self.measure_headers[self.measure_headers.len()-1].repeat_close == -1 {self.measure_headers[last].repeat_close -= 1;}
        if (flags & 0x03) == 0x03 {self.measure_headers[last].time_signature.beams = vec![read_byte(data, seek); 4]}
        else {self.measure_headers[last].time_signature.beams = self.measure_headers[prev].time_signature.beams.clone();}
        if (flags & 0x10) == 0 { *seek += 1; } //always
        self.measure_headers[last].triplet_feel = get_triplet_feel(read_signed_byte(data, seek));
    }

    fn read_repeat_alternative(&mut self, data: &[u8], seek: &mut usize) -> i8 {
        //println!("read_repeat_alternative()");
        let value = read_byte(data, seek);
        let mut existing_alternative = 0i8;
        for i in self.measure_headers.len()-1 .. 0 {
            if self.measure_headers[i].repeat_open {break;}
            existing_alternative |= self.measure_headers[i].repeat_alternative;
        }
        ((1 << value) - 1) ^ existing_alternative
    }
    fn read_repeat_alternative_v5(&mut self, data: &[u8], seek: &mut usize) -> i8 {read_byte(data, seek).to_i8().unwrap()}

    /// Read directions.  Directions is a list of 19 `ShortInts <short>` each pointing at the number of measure.
    /// 
    /// Directions are read in the following order:
    /// - Coda
    /// - Double Coda
    /// - Segno
    /// - Segno Segno
    /// - Fine
    /// - Da Capo
    /// - Da Capo al Coda
    /// - Da Capo al Double Coda
    /// - Da Capo al Fine
    /// - Da Segno
    /// - Da Segno al Coda
    /// - Da Segno al Double Coda
    /// - Da Segno al Fine
    /// - Da Segno Segno
    /// - Da Segno Segno al Coda
    /// - Da Segno Segno al Double Coda
    /// - Da Segno Segno al Fine
    /// - Da Coda
    /// - Da Double Coda
    pub fn read_directions(&self, data: &[u8], seek: &mut usize) -> (HashMap<DirectionSign, i16>, HashMap<DirectionSign, i16>) {
        let mut signs: HashMap<DirectionSign, i16> = HashMap::with_capacity(4);
        let mut from_signs: HashMap<DirectionSign, i16> = HashMap::with_capacity(15);
        //signs
        signs.insert(DirectionSign::Coda, read_short(data, seek));
        signs.insert(DirectionSign::DoubleCoda, read_short(data, seek));
        signs.insert(DirectionSign::Segno, read_short(data, seek));
        signs.insert(DirectionSign::SegnoSegno, read_short(data, seek));
        signs.insert(DirectionSign::Fine, read_short(data, seek));
        //from signs
        from_signs.insert(DirectionSign::DaCapo, read_short(data, seek));
        from_signs.insert(DirectionSign::DaCapoAlCoda, read_short(data, seek));
        from_signs.insert(DirectionSign::DaCapoAlDoubleCoda, read_short(data, seek));
        from_signs.insert(DirectionSign::DaCapoAlFine, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegno, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegnoAlCoda, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegnoAlDoubleCoda, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegnoAlFine, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegnoSegno, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegnoSegnoAlCoda, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegnoSegnoAlDoubleCoda, read_short(data, seek));
        from_signs.insert(DirectionSign::DaSegnoSegnoAlFine, read_short(data, seek));
        from_signs.insert(DirectionSign::DaCoda, read_short(data, seek));
        from_signs.insert(DirectionSign::DaDoubleCoda, read_short(data, seek));
        (signs, from_signs)
    }
}
