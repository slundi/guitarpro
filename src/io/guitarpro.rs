use crate::base::*;

//GTPFileFormatVersion has 3 attributes : fileFormat(TGFileFormat), verstion(string), versionCode(int)

const VERSION_1_0X: u8 = 10;
const VERSION_2_2X: u8 = 22;
const VERSION_3_00: u8 = 30;
const VERSION_4_0X: u8 = 40;
const VERSION_5_00: u8 = 50;
const VERSION_5_10: u8 = 51;

const GP_BEND_SEMITONE: f32 = 25.0;
const GP_BEND_POSITION: f32 = 60.0;

impl Song {
    pub fn gp_read_data(&mut self, data: &Vec<u8>) {
        let mut seek: usize = 0;
        //version
        let tmp = read_version(data, &mut seek);
        let mut version: u8;
        if tmp == "FICHIER GUITARE PRO v1" || tmp == "FICHIER GUITARE PRO v1.01" || tmp == "FICHIER GUITARE PRO v1.02" || tmp == "FICHIER GUITARE PRO v1.03" || tmp == "FICHIER GUITARE PRO v1.04" {version = VERSION_1_0X}
        else if tmp == "FICHIER GUITARE PRO v2.20" || tmp == "FICHIER GUITARE PRO v2.21" {version = VERSION_2_2X}
        else if tmp == "FICHIER GUITAR PRO v3.00" {version = VERSION_3_00}
        else if tmp == "FICHIER GUITAR PRO v4.00" || tmp == "FICHIER GUITAR PRO v4.06" || tmp == "FICHIER GUITAR PRO L4.06" {version = VERSION_4_0X}
        else if tmp == "FICHIER GUITAR PRO v5.00" {version = VERSION_5_00;}
        else if tmp == "FICHIER GUITAR PRO v5.10" {version = VERSION_5_10;}

        self.name        = read_int_size_string(data, &mut seek);
        read_int_size_string(data, &mut seek); //subtitle
        self.artist      = read_int_size_string(data, &mut seek);
        self.album       = read_int_size_string(data, &mut seek);
        //words
        //music
        self.author      = read_int_size_string(data, &mut seek);
        read_int_size_string(data, &mut seek);
        //self.date        = read_int_size_string(data, &mut seek);
        self.copyright   = read_int_size_string(data, &mut seek);
        self.writer      = read_int_size_string(data, &mut seek);
        read_int_size_string(data, &mut seek); //instructions
        //self.transcriber = read_int_size_string(data, &mut seek);
        //self.comments    = read_int_size_string(data, &mut seek);
    }
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
fn read_version(data: &Vec<u8>, seek: &mut usize) -> String {
    let n = data[0] as usize;
    //if n>
    let mut s = String::with_capacity(30);
    for i in 1..n+1 {
        let c = data[i];
        if i == 0 {break;} //NULL symbol so we exit
        s.push(c as char);
    }
    println!("Version {} {}", n, s);
    *seek += 31;
    return s;
}
