use std::{convert::TryInto, str::FromStr};

use crate::base::{Song};

const EXTENSIONS: &str = "tg";
const MIMES: &str = "audio/x-tuxguitar";
const VERSION: &str = "audio/x-tuxguitar";
const TRACK_SOLO:i32 = 0x01;
const TRACK_MUTE:i32 = 0x02;
const TRACK_LYRICS:i32 = 0x04;
const MEASURE_HEADER_TIMESIGNATURE:i32 = 0x01;
const MEASURE_HEADER_TEMPO:i32 = 0x02;
const MEASURE_HEADER_REPEAT_OPEN:i32 = 0x04;
const MEASURE_HEADER_REPEAT_CLOSE:i32 = 0x08;
const MEASURE_HEADER_REPEAT_ALTERNATIVE:i32 = 0x10;
const MEASURE_HEADER_MARKER:i32 = 0x20;
const MEASURE_HEADER_TRIPLET_FEEL:i32 = 0x40;
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
const DURATION_DOTTED:i32 = 0x01;
const DURATION_DOUBLE_DOTTED:i32 = 0x02;
const DURATION_NO_TUPLET:i32 = 0x04;
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
        //version
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        println!("Version: {}", read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek));
        //song's name
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        if n>0 { self.name = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        println!("{} {}", n, self.name);
        //artist
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        if n>0 { self.artist = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        println!("{} {}", n, self.artist);
        //album
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        if n>0 { self.album = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        println!("{} {}", n, self.album);
        //author
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        if n>0 { self.author = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        println!("{} {}", n, self.author);
        //date
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        if n>0 { self.date = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        println!("{} {}", n, self.date);
        //copyright
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        println!("n={}", n);
        if n>0 { self.copyright = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        //writer
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        if n>0 { self.writer = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        //transcriber
        let n = (data[seek] & 0xff) as usize *2; seek+=1;
        if n>0 { self.transcriber = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        //comments
        let n = i32::from_be_bytes(data[seek..seek as usize].try_into().unwrap_or_else(|_e|{panic!("Cannot read comments length")})) as usize * 2; seek+=4;
        println!("n={}", n);
        if n>0 { self.comments = read_unsigned_byte_string(&data[seek..seek+n].to_vec(), &mut seek); }
        //get channels
        let n = data[seek]; seek+=1;
        for _i in 0..n {
            //self.channels.push(read_channel());
        }
        //get headers
        let n = data[seek]; seek+=1;
        for _i in 0..n {
            //self.measure_headers.push(read_measure_header());
        }
        //get tracks
        let n = data[seek]; seek+=1;
        for _i in 0..n {
            //self.tracks.push(read_track());
        }
        println!("{} - {} - {} ({})", self.artist, self.album, self.name, self.date);
    }
}



fn read_RGB_color() {

}

fn read_lyrics() {

}

fn read_byte() -> u8 {
    return 0;
}

fn read_header() {

}

fn read_short() {

}

fn read_unsigned_byte_string(data: &Vec<u8>, seek: &mut usize) -> String {
    *seek+=data.len();
    return String::from_utf8(data.to_vec()).unwrap_or_default();//.unwrap_or_else(|_error|{panic!("Cannot get UTF8 string field {:?}",_error);});
}

fn read_integer_string() -> &'static str {
    return "";
}

fn read_string(data: &Vec<u8>) -> &str {
    return "";//std::str::from_utf8(&data[start..length]).unwrap_or_else(|_error|{panic!("Cannot extract string")});
}