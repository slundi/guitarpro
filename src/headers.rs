use std::collections::HashMap;

use fraction::ToPrimitive;

use crate::{io::*, gp::*, key_signature::*, enums::*};

#[derive(Debug,Clone,PartialEq)]
pub struct Version {
    pub data: String,
    pub number: (u8, u8, u8),
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
	pub marker: Option<Marker>,
	pub repeat_open: bool,
	pub repeat_alternative: u8,
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
        marker: None,
        time_signature: TimeSignature {numerator: 4, denominator: Duration::default(), beams: vec![2, 2, 2, 2]},
    }}
}
impl MeasureHeader {
    pub(crate) fn length(&self) -> i64 {self.time_signature.numerator.to_i64().unwrap() * self.time_signature.denominator.time().to_i64().unwrap()}
    pub(crate) fn end(&self) -> i64 {self.start + self.length()}
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
fn read_marker(data: &[u8], seek: &mut usize) -> Marker {
    let mut marker = Marker{title: read_int_size_string(data, seek), ..Default::default()};
    marker.color = read_color(data, seek);
    marker
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
/*impl RepeatGroup {
    pub(crate) fn add_measure_header(&mut self, measure_header: &MeasureHeader) {
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
}*/

impl Song {
    fn add_measure_header(&mut self, header: MeasureHeader) {
        // if the group is closed only the next upcoming header can reopen the group in case of a repeat alternative, so we remove the current group
        //TODO: if header.repeat_open or self.current_repeat_group.is_closed && header.repeat_alternative <= 0 {self.current_repeat_group = RepeatGroup::default();}
        self.measure_headers.push(header);
    }

    pub(crate) fn read_clipboard(&mut self, data: &[u8], seek: &mut usize) -> Option<Clipboard> {
        if !self.version.clipboard {return None;}
        let mut c = Clipboard{start_measure: read_int(data, seek), ..Default::default()};
        c.stop_measure = read_int(data, seek);
        c.start_track = read_int(data, seek);
        c.stop_track = read_int(data, seek);
        if self.version.number.0 == 5 {
            c.start_beat = read_int(data, seek);
            c.stop_beat = read_int(data, seek);
            c.sub_bar_copy = read_int(data, seek) != 0;
        }
        println!("read_clipboard(): {:?}", c);
        Some(c)
    }

    /// Read measure headers. The *measures* are written one after another, their number have been specified previously.
    /// * `measure_count`: number of measures to expect.
    pub(crate) fn read_measure_headers(&mut self, data: &[u8], seek: &mut usize, measure_count: usize) {
        //println!("read_measure_headers()");
        let mut previous: Option<MeasureHeader> = None;
        for i in 1..measure_count + 1  {
            let r: (MeasureHeader, u8) = self.read_measure_header(data, seek, i, previous);
            previous = Some(r.0.clone());
            self.measure_headers.push(r.0); //TODO: use add_measure_header
        }
    }

    pub(crate) fn read_measure_headers_v5(&mut self, data: &[u8], seek: &mut usize, measure_count: usize, directions: &(HashMap<DirectionSign, i16>, HashMap<DirectionSign, i16>)) {
        //println!("read_measure_headers_v5()");
        let mut previous: Option<MeasureHeader> = None;
        for i in 1..measure_count + 1  {
            let r: (MeasureHeader, u8) = self.read_measure_header_v5(data, seek, i, previous);
            previous = Some(r.0.clone());
            self.measure_headers.push(r.0); //TODO: use add_measure_header
        }
        for s in &directions.0 { if s.1 > &-1 {self.measure_headers[s.1.to_usize().unwrap() - 1].direction = Some(*s.0);} }
        for s in &directions.1 { if s.1 > &-1 {self.measure_headers[s.1.to_usize().unwrap() - 1].direction = Some(*s.0);} }
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
    pub(crate) fn read_measure_header(&mut self, data: &[u8], seek: &mut usize, number: usize, previous: Option<MeasureHeader>) -> (MeasureHeader, u8) {
        let flag = read_byte(data, seek);
        //println!("read_measure_header(), flags: {} \t N: {} \t Measure header count: {}", flag, number, self.measure_headers.len());
        let mut mh = MeasureHeader{number: number.to_u16().unwrap(), ..Default::default()};
        mh.start  = 0;
        mh.triplet_feel = self.triplet_feel;
        //we need a previous header for the next 2 flags
        //Numerator of the (key) signature
        if (flag & 0x01 )== 0x01 {mh.time_signature.numerator = read_signed_byte(data, seek);}
        else if number > 1 {mh.time_signature.numerator = previous.clone().unwrap().time_signature.numerator;}
        //Denominator of the (key) signature
        if (flag & 0x02) == 0x02 {mh.time_signature.denominator.value = read_signed_byte(data, seek).to_u16().unwrap();}
        else if number > 1 {mh.time_signature.denominator = previous.clone().unwrap().time_signature.denominator;}

        mh.repeat_open = (flag & 0x04) == 0x04; //Beginning of repeat
        if (flag & 0x08) == 0x08 {mh.repeat_close = read_signed_byte(data, seek);} //End of repeat
        if (flag & 0x10) == 0x10 {mh.repeat_alternative = if self.version.number.0 == 5 {self.read_repeat_alternative_v5(data, seek)} else {self.read_repeat_alternative(data, seek)};} //Number of alternate ending
        if (flag & 0x20) == 0x20 {mh.marker = Some(read_marker(data, seek));} //Presence of a marker
        if (flag & 0x40) == 0x40 { //Tonality of the measure 
            mh.key_signature.key      = read_signed_byte(data, seek);
            mh.key_signature.is_minor = read_signed_byte(data, seek) != 0;
        } else if mh.number > 1 {mh.key_signature = previous.unwrap().key_signature;}
        mh.double_bar = (flag & 0x80) == 0x80; //presence of a double bar
        (mh, flag)
    }
    /// Read measure header. Measure header format in Guitar Pro 5 differs from one if Guitar Pro 3.
    /// 
    /// First, there is a blank byte if measure is not first. Then measure header is read as in GP3's `read_measure_header_v3()`. Then measure header is read as follows:
    /// - Time signature beams: 4 `Bytes <byte>`. Appears If time signature was set, i.e. flags *0x01* and *0x02* are both set.
    /// - Blank `byte` if flag at *0x10* is set.
    /// - Triplet feel: `byte`. See `TripletFeel`.
    pub(crate) fn read_measure_header_v5(&mut self, data: &[u8], seek: &mut usize, number: usize, previous: Option<MeasureHeader>) -> (MeasureHeader,u8) {
        if previous.is_some() { *seek += 1; } //always
        let r = self.read_measure_header(data, seek, number, previous.clone());
        let mut mh = r.0;
        let flags = r.1;
        //println!("read_measure_header_v5(), flags: {}", flags);
        if mh.repeat_close > -1 {mh.repeat_close -= 1;}
        if (flags & 0x03) == 0x03 {
            for i in 0..4 {mh.time_signature.beams[i] = read_byte(data, seek);}
        } else {mh.time_signature.beams = previous.unwrap().time_signature.beams;};
        if (flags & 0x10) == 0 { *seek += 1; } //always 0
        mh.triplet_feel = get_triplet_feel(read_byte(data, seek).to_i8().unwrap());
        //println!("################################### {:?}", mh.triplet_feel);
        (mh, flags)
    }

    fn read_repeat_alternative(&mut self, data: &[u8], seek: &mut usize) -> u8 {
        //println!("read_repeat_alternative()");
        let value = read_byte(data, seek).to_u16().unwrap();
        let mut existing_alternative = 0u16;
        for i in (0..self.measure_headers.len()).rev() {
            if self.measure_headers[i].repeat_open {break;}
            existing_alternative |= self.measure_headers[i].repeat_alternative.to_u16().unwrap();
        }
        //println!("read_repeat_alternative(), value:  {}, existing_alternative: {}", value, existing_alternative);
        //println!("read_repeat_alternative(), return: {}", ((1 << value) - 1) ^ existing_alternative);
        (((1 << value) - 1) ^ existing_alternative).to_u8().unwrap()
    }
    fn read_repeat_alternative_v5(&mut self, data: &[u8], seek: &mut usize) -> u8 {read_byte(data, seek)}

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
    pub(crate) fn read_directions(&self, data: &[u8], seek: &mut usize) -> (HashMap<DirectionSign, i16>, HashMap<DirectionSign, i16>) {
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

    pub(crate) fn write_measure_headers(&self, data: &mut Vec<u8>, version: &(u8,u8,u8)) {
        let mut previous: Option<usize> = None;
        for i in 0..self.measure_headers.len() {
            //self.current_measure_number = Some(self.tracks[0].measures[i].number);
            self.write_measure_header(data, i, previous, version);
            previous = Some(i);
        }
    }

    fn write_measure_header(&self, data: &mut Vec<u8>, header: usize, previous: Option<usize>, version: &(u8,u8,u8)) {
        //pack measure header flags
        let mut flags: u8 = 0x00;
        if let Some(p) = previous {
            if self.measure_headers[header].time_signature.numerator != self.measure_headers[p].time_signature.numerator {flags |= 0x01;}
            if self.measure_headers[header].time_signature.denominator.value != self.measure_headers[p].time_signature.denominator.value {flags |= 0x02;}
        } else {
            flags |= 0x01;
            flags |= 0x02;
            if self.measure_headers[header].repeat_open {flags |= 0x04;}
            if self.measure_headers[header].repeat_close > -1 {flags |= 0x08;}
            if self.measure_headers[header].repeat_alternative > 0 {flags |= 0x10;}
            if self.measure_headers[header].marker.is_some() {flags |= 0x20;}
        }
        if version.0 >= 4 {
            if previous.is_none() {flags |= 0x40;}
            else if let Some(p) = previous {if self.measure_headers[header].key_signature ==  self.measure_headers[p].key_signature {flags |= 0x40;}}
            if self.measure_headers[header].double_bar {flags |= 0x80;}
        }
        if version.0 >= 5 {
            if let Some(p) = previous {
                if self.measure_headers[header].time_signature != self.measure_headers[p].time_signature {flags |= 0x03;}
                write_placeholder_default(data, 1);
            }
        }
        //end pack
        //write measure header values
        write_byte(data, flags);
        if (flags & 0x01) == 0x01 {write_signed_byte(data, self.measure_headers[header].time_signature.numerator);}
        if (flags & 0x02) == 0x02 {write_signed_byte(data, self.measure_headers[header].time_signature.denominator.value.to_i8().unwrap());}
        if (flags & 0x08) == 0x08 {write_signed_byte(data, if version.0 < 5 {self.measure_headers[header].repeat_close} else {self.measure_headers[header].repeat_close + 1});}
        if (flags & 0x10) == 0x10 { //write repeat alternative
            if version.0 ==5 {write_byte(data, self.measure_headers[header].repeat_alternative);}
            else {
                let mut first_one = false;
                let mut ra:u8 = 0;
                for i in 0u8..9-self.measure_headers[header].repeat_alternative.leading_zeros().to_u8().unwrap() {
                    ra = i;
                    if (self.measure_headers[header].repeat_alternative & 1 << i) > 0 {first_one = true;}
                    else if first_one {break;}
                }
                write_byte(data, ra);
            }
        }
        if (flags & 0x20) == 0x20 { //write marker
            if let Some(marker) = &self.measure_headers[header].marker {
                write_int_byte_size_string(data, &marker.title);
                write_color(data, marker.color);
            }
        }
        if version.0 >= 4 {
            write_signed_byte(data, self.measure_headers[header].key_signature.key);
            write_signed_byte(data, if self.measure_headers[header].key_signature.is_minor {1} else {0});
        }
        if version.0 >= 5 {
            if (flags & 0x03) == 0x03 {
                for i in 0..self.measure_headers[header].time_signature.beams.len() {write_byte(data, self.measure_headers[header].time_signature.beams[i]);}
            }
            if (flags & 0x10) == 0x10 {write_placeholder_default(data, 1);}
            write_byte(data, from_triplet_feel(self.measure_headers[header].triplet_feel));
        }
    }

    pub(crate) fn write_clipboard(&self, data: &mut Vec<u8>, version: &(u8,u8,u8)) {
        if let Some(c) = &self.clipboard {
            write_i32(data, c.start_measure.to_i32().unwrap());
            write_i32(data, c.stop_measure.to_i32().unwrap());
            write_i32(data, c.start_track.to_i32().unwrap());
            write_i32(data, c.stop_track.to_i32().unwrap());
            if version.0 == 5 {
                write_i32(data, c.start_beat.to_i32().unwrap());
                write_i32(data, c.stop_beat.to_i32().unwrap());
                write_i32(data, if c.sub_bar_copy {1} else {0});
            }
        }
    }
    pub(crate) fn write_directions(&self, data: &mut Vec<u8>) {
        let mut map: HashMap<DirectionSign, i16>= HashMap::with_capacity(19);
        for i in 1..self.measure_headers.len() {
            if let Some(d) = &self.measure_headers[i].direction { map.insert(d.clone(), i.to_i16().unwrap()); }
        }
        let order: Vec<DirectionSign> = vec![DirectionSign::Coda, DirectionSign::DoubleCoda, DirectionSign::Segno, DirectionSign::SegnoSegno, DirectionSign::Fine,
                                             DirectionSign::DaCapo,
                                             DirectionSign::DaCapoAlCoda,
                                             DirectionSign::DaCapoAlDoubleCoda,
                                             DirectionSign::DaCapoAlFine,
                                             DirectionSign::DaSegno,
                                             DirectionSign::DaSegnoAlCoda,
                                             DirectionSign::DaSegnoAlDoubleCoda,
                                             DirectionSign::DaSegnoAlFine,
                                             DirectionSign::DaSegnoSegno,
                                             DirectionSign::DaSegnoSegnoAlCoda,
                                             DirectionSign::DaSegnoSegnoAlDoubleCoda,
                                             DirectionSign::DaSegnoSegnoAlFine,
                                             DirectionSign::DaCoda,
                                             DirectionSign::DaDoubleCoda];
        for d in order {
            let x = map.get(&d);
            if let Some(dir) = x {write_i16(data, *dir);}
            else {write_i16(data, -1);}
        }
    }
}
