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
        self.name        = read_int_size_string(data, &mut seek);
        self.subtitle    = read_int_size_string(data, &mut seek);
        self.artist      = read_int_size_string(data, &mut seek);
        self.album       = read_int_size_string(data, &mut seek);
        self.words       = read_int_size_string(data, &mut seek); //music
        self.copyright   = read_int_size_string(data, &mut seek);
        self.author      = read_int_size_string(data, &mut seek);
        self.writer      = read_int_size_string(data, &mut seek); //tabbed by
        self.instructions= read_int_size_string(data, &mut seek); //instructions
        let nc = read_int(data, &mut seek) as usize;
        println!("Note count: {}", nc);
        for i in 0..nc { //notices
            println!("  {}\t\t{}",i, read_int_size_string(data, &mut seek));
        }
        //read GP4 information
        if version.number == 40 {

        }
        //read GP5 information
        if version.number == 50 {
            
        }
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


/// Read a byte and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the read byte as u8
fn read_byte(data: &Vec<u8>, seek: &mut usize ) -> i8 {
    if data.len() < *seek {panic!("End of filee reached");}
    let b = data[*seek] as i8;
    *seek += 1;
    return b;
}

/// Read a signed byte and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the read byte as u8
fn read_signed_byte(data: &Vec<u8>, seek: &mut usize ) -> u8 {
    if data.len() < *seek {panic!("End of file reached");}
    let b = data[*seek] as u8;
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
