use crate::base::*;
use regex::Regex;

//GTPFileFormatVersion has 3 attributes : fileFormat(TGFileFormat), verstion(string), versionCode(int)

const VERSION_1_0X: u8 = 10;
const VERSION_2_2X: u8 = 22;
const VERSION_3_00: u8 = 30;
const VERSION_4_0X: u8 = 40;
const VERSION_5_00: u8 = 50;
const VERSION_5_10: u8 = 51;

const GP_BEND_SEMITONE: f32 = 25.0;
const GP_BEND_POSITION: f32 = 60.0;

struct Version {
    data: String,
    number: u8,
    clipboard: bool
}

impl Song {
    pub fn gp_read_data(&mut self, data: &Vec<u8>) {
        let mut seek: usize = 0;
        let version = read_version(data, &mut seek);
        let mut clipboard = Clipboard::default();
        //check for clipboard and read it
        if version.number == VERSION_4_0X && version.clipboard {
            clipboard.start_measure = read_int(data, &mut seek);
            clipboard.stop_measure  = read_int(data, &mut seek);
            clipboard.start_track = read_int(data, &mut seek);
            clipboard.stop_track  = read_int(data, &mut seek);
        }
        if version.number == VERSION_5_00 && version.clipboard {
            clipboard.start_beat = read_int(data, &mut seek);
            clipboard.stop_beat  = read_int(data, &mut seek);
            clipboard.sub_bar_copy = read_int(data, &mut seek) != 0;
        }
        // read GP3 informations
        self.name        = read_int_size_string(data, &mut seek);//.replace("\r", " ").replace("\n", " ").trim().to_owned();
        self.subtitle    = read_int_size_string(data, &mut seek);
        self.artist      = read_int_size_string(data, &mut seek);
        self.album       = read_int_size_string(data, &mut seek);
        self.words       = read_int_size_string(data, &mut seek); //music
        self.copyright   = read_int_size_string(data, &mut seek);
        self.writer      = read_int_size_string(data, &mut seek); //tabbed by
        self.instructions= read_int_size_string(data, &mut seek); //instructions
        //notices
        let nc = read_int(data, &mut seek) as usize;
        if nc >0 {
            for i in 0..nc { 
                println!("  {}\t\t{}",i, read_int_size_string(data, &mut seek));
        }}
        if version.number < VERSION_5_00 {
            self.triplet_feel = if read_bool(data, &mut seek) {TRIPLET_FEEL_EIGHTH} else {TRIPLET_FEEL_NONE};
            //println!("Triplet feel: {}", self.triplet_feel);
            if version.number == VERSION_4_0X {} //read lyrics
            self.tempo = read_int(data, &mut seek) as i16;
            self.key.key = read_int(data, &mut seek) as i8;
            println!("Tempo: {}\t\tKey: {}", self.tempo, self.key.key);
            if version.number == VERSION_4_0X {read_signed_byte(data, &mut seek);} //octave
            self.read_midi_channels(data, &mut seek);
            let measure_count = read_int(data, &mut seek) as usize;
            let track_count = read_int(data, &mut seek) as usize;
            self.read_measure_headers(data, &mut seek, measure_count);
            if version.number == VERSION_4_0X {} //annotate error reading
        }
        //read GP5 information
        if version.number == 50 {
            //self.lyrics = 
            self.read_lyrics(data, &mut seek);
        /*song.masterEffect = self.readRSEMasterEffect()
        song.pageSetup = self.readPageSetup()
        song.tempoName = self.readIntByteSizeString()
        song.tempo = self.readInt()
        song.hideTempo = self.readBool() if self.versionTuple > (5, 0, 0) else False
        song.key = gp.KeySignature((self.readSignedByte(), 0))
        self.readInt()  # octave
        channels = self.readMidiChannels()
        directions = self.readDirections()
        song.masterEffect.reverb = self.readInt()
        measureCount = self.readInt()
        trackCount = self.readInt()
        with self.annotateErrors('reading'):
            self.readMeasureHeaders(song, measureCount, directions)
            self.readTracks(song, trackCount, channels)
            self.readMeasures(song) */
        }
    }

    /// Read lyrics.
    ///
    /// First, read an `i32` that points to the track lyrics are bound to. Then it is followed by 5 lyric lines. Each one consists of
    /// number of starting measure encoded in`i32` and`int-size-string` holding text of the lyric line.
    fn read_lyrics(&mut self, data: &Vec<u8>, seek: &mut usize) -> Lyrics {
        let track = read_int(data, seek) as usize;
        println!("Lyrics for track #{}", track);
        let mut lyrics = Lyrics::default();
        lyrics.lyrics1.insert(read_int(data, seek), read_int_size_string(data, seek));
        lyrics.lyrics2.insert(read_int(data, seek), read_int_size_string(data, seek));
        lyrics.lyrics3.insert(read_int(data, seek), read_int_size_string(data, seek));
        lyrics.lyrics4.insert(read_int(data, seek), read_int_size_string(data, seek));
        lyrics.lyrics5.insert(read_int(data, seek), read_int_size_string(data, seek));
        return lyrics;
    }

    /** Read MIDI channels. Guitar Pro format provides 64 channels (4 MIDI ports by 16 hannels), the channels are stored in this order:
        * port1/channel1
        * port1/channel2
        * ...
        * port1/channel16
        * port2/channel1
        * ...
        * port4/channel16

        Each channel has the following form:
        * Instrument: `int`.
        * Volume: `byte`.
        * Balance: `byte`.
        * Chorus: `byte`.
        * Reverb: `byte`.
        * Phaser: `byte`.
        * Tremolo: `byte`.
        * blank1: `byte`.
        * blank2: `byte`.
     */
    fn read_midi_channels(&mut self, data: &Vec<u8>, seek: &mut usize) {
        for i in 0u8..64u8 {
            let instrument = read_int(data, seek);
            let mut c = MidiChannel::default();
            c.channel = i; c.effect_channel = i;
            c.volume = read_signed_byte(data, seek); c.balance = read_signed_byte(data, seek);
            c.chorus = read_signed_byte(data, seek); c.reverb = read_signed_byte(data, seek); c.phaser = read_signed_byte(data, seek); c.tremolo = read_signed_byte(data, seek);
            c.set_instrument(instrument);
            //println!("Channel: {}\t Volume: {}\tBalance: {}\tInstrument={}, {}", c.channel, c.volume, c.balance, instrument, c.get_instrument());
            self.channels.push(c);
            *seek += 2;
        }
    }

    /// Read measure headers. The *measures* are written one after another, their number have been specified previously.
    fn read_measure_headers(&mut self, data: &Vec<u8>, seek: &mut usize, count: usize) {
        /* previous = None
        for number in range(1, measureCount + 1):
            self._currentMeasureNumber = number
            header, _ = self.readMeasureHeader(number, song, previous)
            song.addMeasureHeader(header)
            previous = header
        self._currentMeasureNumber = None */
        for i in 1..count + 1 {
            self.current_measure_number = i as u16;
            self.read_measure_header(data, seek, i);
        }
        self.current_measure_number = 0;
    }

    /// Read measure header. The first byte is the measure's flags. It lists the data given in the current measure.
    /// - *0x01*: numerator of the key signature
    /// - *0x02*: denominator of the key signature
    /// - *0x04*: beginning of repeat
    /// - *0x08*: end of repeat
    /// - *0x10*: number of alternate ending
    /// - *0x20*: presence of a marker
    /// - *0x40*: tonality of the measure
    /// - *0x80*: presence of a double bar
    ///
    /// Each of these elements is present only if the corresponding bit is a 1.
    /// The different elements are written (if they are present) from lowest to highest bit.
    /// Exceptions are made for the double bar and the beginning of repeat whose sole presence is enough, complementary data is not necessary.
    /// - Numerator of the key signature: :ref:`byte`.
    /// - Denominator of the key signature: :ref:`byte`.
    /// - End of repeat: :ref:`byte`. Number of repeats until the previous beginning of repeat.
    /// - Number of alternate ending: :ref:`byte`.
    /// - Marker: see :meth:`GP3File.readMarker`.
    /// - Tonality of the measure: 2 :ref:`Bytes <byte>`. These values encode a key signature change on the current piece. First byte is key signature root, second is key signature type.
    fn read_measure_header(&mut self, data: &Vec<u8>, seek: &mut usize, number: usize) {
        let flag = read_byte(data, seek);
        let mut mh = MeasureHeader::default();
        mh.number = number as u16;
        mh.start  = 0;
        mh.triplet_feel = self.triplet_feel.clone();
        //we need a previous header for the next 2 flags
        mh.time_signature.numerator = if (flag & 0x01 )== 0x01 && number > 1 {read_signed_byte(data, seek)} else {self.measure_headers[number-1].time_signature.numerator};
        mh.time_signature.denominator = if (flag & 0x02) == 0x02 && number > 1 {read_signed_byte(data, seek)} else {self.measure_headers[number-1].time_signature.denominator};
        mh.repeat_open = (flag & 0x04) == 0x04;
        if (flag & 0x08) == 0x08 {mh.repeat_close = read_signed_byte(data, seek);}
        if (flag & 0x10) == 0x10 {mh.repeat_alternative = self.read_repeat_alternative(data, seek);}
        if (flag & 0x20) == 0x20 {self.read_marker(data, seek, &mut mh);}
        if (flag & 0x40) == 0x40 {
            mh.key_signature.key = read_signed_byte(data, seek);
            mh.key_signature.is_minor = read_signed_byte(data, seek) != 0;
        } else if mh.number > 1 {mh.key_signature = self.measure_headers[number-1].key_signature.clone();}
        mh.double_bar = (flag & 0x80) == 0x80;
        self.measure_headers.push(mh);
    }

    /// Read a marker. The markers are written in two steps:
    /// - first is written an integer equal to the marker's name length + 1
    /// - then a string containing the marker's name. Finally the marker's color is written.
    fn read_marker(&mut self, data: &Vec<u8>, seek: &mut usize, measure_header: &mut MeasureHeader) {
        measure_header.marker.title = read_int_size_string(data, seek);
        measure_header.marker.color = self.read_color(data, seek);
    }

    /// Read a color. Colors are used by `Marker` and `Track`. They consist of 3 consecutive bytes and one blank byte.
    fn read_color(&mut self, data: &Vec<u8>, seek: &mut usize) -> i32 {
        let r = read_byte(data, seek) as i32;
        let g = read_byte(data, seek) as i32;
        let b = read_byte(data, seek) as i32;
        *seek += 1;
        return r * 65536 + g * 256 + b;
    }

    fn read_repeat_alternative(&mut self, data: &Vec<u8>, seek: &mut usize) -> i8 {
        let value = read_byte(data, seek);
        let mut existing_alternative = 0i8;
        for h in self.measure_headers.clone() {
            if h.repeat_open {break;}
            existing_alternative |= h.repeat_alternative;
        }
        return (1 << value) - 1 ^ existing_alternative;
    }
}

struct Clipboard {
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

//reading functions

/// Read a byte and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the read byte as u8
fn read_byte(data: &Vec<u8>, seek: &mut usize ) -> u8 {
    if data.len() < *seek {panic!("End of filee reached");}
    let b = data[*seek] as u8;
    *seek += 1;
    return b;
}

/// Read a signed byte and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the read byte as u8
fn read_signed_byte(data: &Vec<u8>, seek: &mut usize ) -> i8 {
    if data.len() < *seek {panic!("End of file reached");}
    let b = data[*seek] as i8;
    *seek += 1;
    return b;
}

/// Read a boolean and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns boolean value
fn read_bool(data: &Vec<u8>, seek: &mut usize ) -> bool {
    if data.len() < *seek {panic!("End of file reached");}
    let b = data[*seek] as i8;
    *seek += 1;
    return b != 0;
}

/// Read a short and increase the cursor position by 2 (2 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the short value
fn read_short(data: &Vec<u8>, seek: &mut usize ) -> i16 {
    if data.len() < *seek + 1 {panic!("End of file reached");}
    let n = i16::from_le_bytes([data[*seek], data[*seek+1]]);
    *seek += 2;
    return n;
}

/// Read an integer and increase the cursor position by 4 (4 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the integer value
fn read_int(data: &Vec<u8>, seek: &mut usize ) -> i32 {
    if data.len() < *seek + 4 {panic!("End of file reached");}
    let n = i32::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3]]);
    *seek += 4;
    return n;
}

/// Read a float and increase the cursor position by 4 (4 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the float value
fn read_float(data: &Vec<u8>, seek: &mut usize ) -> f32 {
    if data.len() < *seek + 8 {panic!("End of file reached");}
    let n = f32::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3]]);
    *seek += 4;
    return n;
}

/// Read a double and increase the cursor position by 8 (8 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the float value
fn read_double(data: &Vec<u8>, seek: &mut usize ) -> f64 {
    if data.len() >= *seek {panic!("End of file reached");}
    let n = f64::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3], data[*seek+4], data[*seek+5], data[*seek+6], data[*seek+7]]);
    *seek += 8;
    return n;
}

/// Read a string.
fn read_int_size_string(data: &Vec<u8>, seek: &mut usize) -> String {
    let n = read_int(data, seek) as usize;
    //let mut s = 
    //println!("Slice {}", std::str::from_utf8(&data[*seek..*seek+n]).unwrap());
    let parse = String::from_utf8(data[*seek..*seek+n].to_vec());
    if parse.is_err() {panic!("Unable to read string");}
    *seek += n;
    return parse.unwrap();
}

/// Read the file version. It is on the first 30 bytes of the file.
/// * `data` - array of bytes
/// * `seek` - cursor that will be incremented
/// * returns version
fn read_version(data: &Vec<u8>, seek: &mut usize) -> Version {
    let n = data[0] as usize;
    let mut v = Version {data: String::with_capacity(30), number: 0, clipboard: false};
    for i in 1..n+1 {
        let c = data[i];
        if i == 0 {break;} //NULL symbol so we exit
        v.data.push(c as char);
    }
    //println!("Version {} {}", n, s);
    *seek += 31;
    //get the version
    lazy_static! {
        static ref RE: Regex = Regex::new(r"v(\d)\.(\d)").unwrap();
    }
    let cap = RE.captures(&v.data).expect("Cannot extrat version code");
    if      &cap[1] == "3" {v.number = VERSION_3_00;}
    else if &cap[1] == "4" {
        v.clipboard = v.data.starts_with("CLIPBOARD");
        v.number = VERSION_4_0X;
    }
    else if &cap[1] == "5" {
        v.clipboard = v.data.starts_with("CLIPBOARD");
        v.number = VERSION_5_00;
    } //TODO: check subversions?
    return v;
}

//writing functions
