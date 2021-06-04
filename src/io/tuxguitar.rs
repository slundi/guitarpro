use std::convert::TryInto;

use crate::base::*;

const EXTENSIONS: &str = "tg";
const MIMES: &str = "audio/x-tuxguitar";
const VERSION: &str = "audio/x-tuxguitar";
const TRACK_SOLO:i32 = 0x01;
const TRACK_MUTE:i32 = 0x02;
const TRACK_LYRICS:i32 = 0x04;
const MEASURE_HEADER_TIMESIGNATURE:u16 = 0x01;
const MEASURE_HEADER_TEMPO:u16 = 0x02;
const MEASURE_HEADER_REPEAT_OPEN:u16 = 0x04;
const MEASURE_HEADER_REPEAT_CLOSE:u16 = 0x08;
const MEASURE_HEADER_REPEAT_ALTERNATIVE:u16 = 0x10;
const MEASURE_HEADER_MARKER:u16 = 0x20;
const MEASURE_HEADER_TRIPLET_FEEL:u16 = 0x40;
const MEASURE_CLEF:i32 = 0x01;
const MEASURE_KEYSIGNATURE:i32 = 0x02;
const BEAT_HAS_NEXT:i32 = 0x01;
const BEAT_HAS_STROKE:i32 = 0x02;
const BEAT_HAS_CHORD:i32 = 0x04;
const BEAT_HAS_TEXT:i32 = 0x08;
const BEAT_HAS_VOICE:i32 = 0x10;
const BEAT_HAS_VOICE_CHANGES:i32 = 0x20;
const VOICE_HAS_NOTES:i32 = 0x01;
const VOICE_NEXT_DURATION:i32 = 0x02;
const VOICE_DIRECTION_UP:i32 = 0x04;
const VOICE_DIRECTION_DOWN:i32 = 0x08;
const NOTE_HAS_NEXT:i32 = 0x01;
const NOTE_TIED:i32 = 0x02;
const NOTE_EFFECT:i32 = 0x04;
const NOTE_VELOCITY:i32 = 0x08;
const DURATION_DOTTED:u8 = 0x01;
const DURATION_DOUBLE_DOTTED:u8 = 0x02;
const DURATION_NO_TUPLET:u8 = 0x04;
const EFFECT_BEND:i32 = 0x000001;
const EFFECT_TREMOLO_BAR:i32 = 0x000002;
const EFFECT_HARMONIC:i32 = 0x000004;
const EFFECT_GRACE:i32 = 0x000008;
const EFFECT_TRILL:i32 = 0x000010;
const EFFECT_TREMOLO_PICKING:i32 = 0x000020;
const EFFECT_VIBRATO:i32 = 0x000040;
const EFFECT_DEAD:i32 = 0x000080;
const EFFECT_SLIDE:i32 = 0x000100;
const EFFECT_HAMMER:i32 = 0x000200;
const EFFECT_GHOST:i32 = 0x000400;
const EFFECT_ACCENTUATED:i32 = 0x000800;
const EFFECT_HEAVY_ACCENTUATED:i32 = 0x001000;
const EFFECT_PALM_MUTE:i32 = 0x002000;
const EFFECT_STACCATO:i32 = 0x004000;
const EFFECT_TAPPING:i32 = 0x008000;
const EFFECT_SLAPPING:i32 = 0x010000;
const EFFECT_POPPING:i32 = 0x020000;
const EFFECT_FADE_IN:i32 = 0x040000;
const EFFECT_LET_RING:i32 = 0x080000;
const GRACE_FLAG_DEAD:i32 = 0x01;
const GRACE_FLAG_ON_BEAT:i32 = 0x02;

impl Song {
    pub fn tg_read_data(&mut self, data: &Vec<u8>) {
        let mut seek: usize = 0;
        read_string(data, &mut seek, false); //version
        self.name        = read_string(data, &mut seek, false);
        self.artist      = read_string(data, &mut seek, false);
        self.album       = read_string(data, &mut seek, false);
        self.author      = read_string(data, &mut seek, false);
        self.date        = read_string(data, &mut seek, false);
        self.copyright   = read_string(data, &mut seek, false);
        self.writer      = read_string(data, &mut seek, false);
        self.transcriber = read_string(data, &mut seek, false);
        self.comments    = read_string(data, &mut seek, true);
        //get channels
        let n = data[seek]; seek+=1;
        for _i in 0..n {
            let mut c:Channel = Channel::default();
            c.id      = u16::from_be_bytes([data[seek], data[seek+1]]); seek+=2;
            c.bank    = (data[seek] & 0xff) as u16; seek+=1;
            c.program = (data[seek] & 0xff) as u16; seek+=1;
            c.volume  = (data[seek] & 0xff) as u16; seek+=1;
            c.balance = (data[seek] & 0xff) as u16; seek+=1;
            c.chorus  = (data[seek] & 0xff) as u16; seek+=1;
            c.reverb  = (data[seek] & 0xff) as u16; seek+=1;
            c.phaser  = (data[seek] & 0xff) as u16; seek+=1;
            c.tremolo = (data[seek] & 0xff) as u16; seek+=1;
            c.name    = read_string(data, &mut seek, false);
            //parameters
            let count = u16::from_be_bytes([data[seek], data[seek+1]]); seek+=2;
            println!("\tparameters: {}",count);
            for _j in 0..count {
                let k=read_string(data, &mut seek, false);
                let v=u32::from_be_bytes([data[seek], data[seek+1], data[seek+2], data[seek+3]]); seek+=4;
                c.parameters.insert(k, v);
            }
            self.channels.push(c);
        }
        //get headers
/*
	private int readHeader() throws IOException { return this.dataInputStream.read();}
	
	private int readHeader(int bCount) throws IOException {
		int header = 0;
		for(int i = bCount; i > 0; i --) header += ( readHeader() << ( (8 * i) - 8 ) );
		return header;
	}
*/
        let n = u16::from_be_bytes([data[seek], data[seek+1]]); seek+=2;
        let mut header_start = DURATION_QUARTER_TIME;
        let mut last_header: Option<MeasureHeader> = None;
        //TODO
        /*for i in 0..n {
            println!("i=\t{}", i);
            let mut h:MeasureHeader = MeasureHeader::default();
            let header = u16::from_be_bytes([data[seek], data[seek+1]]); seek+=2;
            h.number = i as i32 + 1;
            h.start = header_start;
            //let header = data[seek]; seek+=1;
            if (header & MEASURE_HEADER_TIMESIGNATURE) != 0 {
                let numerator = data[seek]; seek+=1;
                //duration
                //TODO
                let x=data[seek]; seek+=1;
                let duration_dotted=x & DURATION_DOTTED != 0;
                let duration_double_dotted=x & DURATION_DOUBLE_DOTTED != 0;
                let duration_value=data[seek]; seek+=1;
            }
            else if last_header.is_some() {}
            if (header & MEASURE_HEADER_TEMPO) != 0 {}
            else if last_header.is_some() {}
            h.repeat_open = (header & MEASURE_HEADER_REPEAT_OPEN) != 0;
            if (header & MEASURE_HEADER_REPEAT_CLOSE) != 0 {h.repeat_close = u16::from_be_bytes([data[seek], data[seek+1]]); seek+=2;}
            if (header & MEASURE_HEADER_REPEAT_ALTERNATIVE) != 0 {h.repeat_alternative = data[seek]; seek+=1;}
            if (header & MEASURE_HEADER_MARKER) != 0 {
                //TODO: read marker
            }
            if last_header.is_some() {h.triplet_feel = last_header.unwrap().triplet_feel;}
            else {h.triplet_feel = TRIPLET_FEEL_NONE;}
            if (header & MEASURE_HEADER_TRIPLET_FEEL) != 0{ h.triplet_feel = data[seek]; seek+=1;}
            self.measure_headers.push(h.clone());
            //header_start += h.len();
            last_header = Some(h.clone());
        }
        //get tracks
        let n = data[seek]; seek+=1;
        for _i in 0..n {
            let mut t:Track = Track::default();
            //self.tracks.push(read_track());
        }*/
    }
}

/// Read a string. The first part is the length of the string (mainly on 1 byte). Following is the string (1 char is encoded on 2 bytes)
fn read_string(data: &Vec<u8>, seek: &mut usize, length_is_integer: bool) -> String {
    let mut n: usize = 0;
    if length_is_integer {
        n = i32::from_be_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3]]) as usize;
        *seek+=4;
    } else {
        n = (data[*seek] & 0xff) as usize;
        *seek+=1;
    }
    let mut s: String = String::with_capacity(n);
    for i in 0usize..n {
        s.push(std::char::from_u32(((data[*seek + i * 2] as u32)<<8) + data[*seek + i * 2 + 1] as u32).unwrap_or_else(||{panic!("Cannot read 2-bytes char")}));
    }
    *seek += 2 * n;
    return s;
}
