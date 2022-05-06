use fraction::ToPrimitive;
use encoding_rs::*;

//reading functions

/// Read a byte and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the read byte as u8
pub(crate) fn read_byte(data: &[u8], seek: &mut usize ) -> u8 {
    if data.len() < *seek {panic!("End of filee reached");}
    let b = data[*seek];
    *seek += 1;
    b
}

/// Read a signed byte and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the read byte as u8
pub(crate) fn read_signed_byte(data: &[u8], seek: &mut usize ) -> i8 {
    if data.len() < *seek {panic!("End of file reached");}
    let b = data[*seek] as i8;
    *seek += 1;
    b
}

/// Read a boolean and increase the cursor position by 1
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns boolean value
pub(crate) fn read_bool(data: &[u8], seek: &mut usize ) -> bool {
    if data.len() < *seek {panic!("End of file reached");}
    let b = data[*seek];
    *seek += 1;
    b != 0
}

/// Read a short and increase the cursor position by 2 (2 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the short value
pub(crate) fn read_short(data: &[u8], seek: &mut usize ) -> i16 {
    if data.len() < *seek + 2 {panic!("End of file reached");}
    let n = i16::from_le_bytes([data[*seek], data[*seek+1]]);
    *seek += 2;
    n
}

/// Read an integer and increase the cursor position by 4 (4 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the integer value
pub(crate) fn read_int(data: &[u8], seek: &mut usize ) -> i32 {
    if data.len() < *seek + 4 {panic!("End of file reached");}
    let n = i32::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3]]);
    *seek += 4;
    n
}

/*/// Read a float and increase the cursor position by 4 (4 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the float value
pub(crate) fn read_float(data: &[u8], seek: &mut usize ) -> f32 {
    let n = f32::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3]]);
    *seek += 4;
    n
}*/

/// Read a double and increase the cursor position by 8 (8 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the float value
pub(crate) fn read_double(data: &[u8], seek: &mut usize ) -> f64 {
    let n = f64::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3], data[*seek+4], data[*seek+5], data[*seek+6], data[*seek+7]]);
    *seek += 8;
    n
}

/// Read length of the string stored in 1 integer and followed by character bytes.
pub(crate) fn read_int_size_string(data: &[u8], seek: &mut usize) -> String {
    let size = read_int(data, seek).to_usize().unwrap();
    read_string(data, seek, size, None)
}

/// Read length of the string increased by 1 and stored in 1 integer followed by length of the string in 1 byte and finally followed by character bytes.
pub(crate) fn read_int_byte_size_string(data: &[u8], seek: &mut usize) -> String {
    let s = (read_int(data, seek) - 1).to_usize().unwrap();
    read_byte_size_string(data, seek, s)
}

/// Read length of the string stored in 1 byte and followed by character bytes.
/// * `size`: string length that we should attempt to read.
pub(crate) fn read_byte_size_string(data: &[u8], seek: &mut usize, size: usize) -> String {
    //println!("read_int_byte_size_string(), size={}", size);
    let length = read_byte(data, seek).to_usize().unwrap();
    read_string(data, seek, size, Some(length))
}

/// Read a string
/// * `size`:   real string length
/// * `length`: optionnal provided length (in case of blank chars after the string)
fn read_string(data: &[u8], seek: &mut usize, size: usize, length: Option<usize>) -> String {
    //println!("read_string(), size={} \t length={:?}", size, length);
    let length = length.unwrap_or(size);
    //let count = if size > 0 {size} else {length};
    let (cow, _encoding_used, had_errors) = WINDOWS_1252.decode(&data[*seek..*seek+length]);
    if had_errors {
        let parse = std::str::from_utf8(&data[*seek..*seek+length]);
        if parse.is_err() {panic!("Unable to read string");}
        *seek += size;
        return parse.unwrap().to_string();
    }
    *seek += size;
    (&cow).to_string()
}

pub const VERSIONS: [((u8,u8,u8), bool, &str); 10] = [((3, 0, 0), false, "FICHIER GUITAR PRO v3.00"),

                                                      ((4, 0, 0), false, "FICHIER GUITAR PRO v4.00"),
                                                      ((4, 0, 6), false, "FICHIER GUITAR PRO v4.06"),
                                                      ((4, 0, 6), true,  "CLIPBOARD GUITAR PRO 4.0 [c6]"),

                                                      ((5, 0, 0), false, "FICHIER GUITAR PRO v5.00"),
                                                      ((5, 1, 0), false, "FICHIER GUITAR PRO v5.10"),
                                                      ((5, 2, 0), false, "FICHIER GUITAR PRO v5.10"),  // sic
                                                      ((5, 0, 0), true,  "CLIPBOARD GP 5.0"),
                                                      ((5, 1, 0), true,  "CLIPBOARD GP 5.1"),
                                                      ((5, 2, 0), true,  "CLIPBOARD GP 5.2")];

/// Read the file version. It is on the first 31 bytes (1st byte is the real length, the following 30 bytes contain the version string) of the file.
/// * `data` - array of bytes
/// * `seek` - cursor that will be incremented
/// * returns version
pub(crate) fn read_version_string(data: &[u8], seek: &mut usize) -> crate::headers::Version {
    let mut v = crate::headers::Version {data: read_byte_size_string(data, seek, 30), number: (5,2,0), clipboard: false};
    //println!("Version {} {}", n, s);
    //get the version
    for x in VERSIONS {
        if v.data == x.2 {
            v.number = x.0;
            v.clipboard = x.1;
            break;
        }
    }
    //println!("########################## Version: {:?}", v);
    v
}

/// Read a color. Colors are used by `Marker` and `Track`. They consist of 3 consecutive bytes and one blank byte.
pub(crate) fn read_color(data: &[u8], seek: &mut usize) -> i32 {
    let r = read_byte(data, seek).to_i32().unwrap();
    let g = read_byte(data, seek).to_i32().unwrap();
    let b = read_byte(data, seek).to_i32().unwrap();
    *seek += 1;
    r * 65536 + g * 256 + b
}

//writing functions
fn write_placeholder(data: &mut Vec<u8>, count: usize, byte: u8) { for _ in 0..count {data.push(byte);} }

pub(crate) fn write_placeholder_default(data: &mut Vec<u8>, count: usize) {write_placeholder(data, count, 0x00);}
pub(crate) fn write_byte(data: &mut Vec<u8>, value: u8) {data.push(value);}
pub(crate) fn write_signed_byte(data: &mut Vec<u8>, value: i8) {data.extend(value.to_le_bytes());}
pub(crate) fn write_bool(data: &mut Vec<u8>, value: bool) {data.push(if value {0x01} else {0x00});}
pub(crate) fn write_i32(data: &mut Vec<u8>, value: i32) {data.extend(value.to_le_bytes());}
//pub(crate) fn write_u32(data: &mut Vec<u8>, value: u32) {data.extend(value.to_le_bytes());}
pub(crate) fn write_i16(data: &mut Vec<u8>, value: i16) {data.extend(value.to_le_bytes());}
//pub(crate) fn write_u16(data: &mut Vec<u8>, value: u16) {data.extend(value.to_le_bytes());}
//pub(crate) fn write_f32(data: &mut Vec<u8>, value: f32) {data.extend(value.to_le_bytes());}
pub(crate) fn write_f64(data: &mut Vec<u8>, value: f64) {data.extend(value.to_le_bytes());}
pub(crate) fn write_color(data: &mut Vec<u8>, value: i32) {
    let r: u8 = ((value & 0xff0000) >> 16).to_u8().unwrap();
    let g: u8 = ((value & 0x00ff00) >> 8).to_u8().unwrap();
    let b: u8 = (value & 0x0000ff).to_u8().unwrap();
    write_byte(data, r);
    write_byte(data, g);
    write_byte(data, b);
    write_placeholder_default(data, 1);
}
pub(crate) fn write_byte_size_string(data: &mut Vec<u8>, value: &str) {
    write_byte(data, value.chars().count().to_u8().unwrap());
    data.extend(value.as_bytes());
}
pub(crate) fn write_int_size_string(data: &mut Vec<u8>, value: &str) {
    let count = value.chars().count();
    write_i32(data, count.to_i32().unwrap()+1);
    data.extend(value.as_bytes());
}

pub(crate) fn write_int_byte_size_string(data: &mut Vec<u8>, value: &str) {
    let count = value.chars().count();
    write_i32(data, count.to_i32().unwrap()+1); //write_i32( (value.getBytes(charset).length + 1) );
    write_byte(data, count.to_u8().unwrap());
    data.extend(value.as_bytes());
}

pub(crate) fn write_version(data: &mut Vec<u8>, version: (u8,u8,u8)) {
    for v in VERSIONS {
        if version == v.0 {
            write_byte_size_string(data, v.2);
            write_placeholder_default(data, 30 - v.2.len());
            break;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::io::*;

    #[test]
    fn test_read_byte_size_string() {
        let data: Vec<u8> = vec![0x18,0x46,0x49,0x43,0x48,0x49,0x45,0x52,
                                 0x20,0x47,0x55,0x49,0x54,0x41,0x52,0x20,
                                 0x50,0x52,0x4f,0x20,0x76,0x33,0x2e,0x30,
                                 0x30];
        let mut seek = 0usize;
        assert_eq!(read_byte_size_string(&data, &mut seek, 30), "FICHIER GUITAR PRO v3.00");
    }

    #[test]
    fn test_read_int_size_string() {
        let data: Vec<u8> = vec![0x08,0x00,0x00,0x00,   0x25,0x41,0x52,0x54,0x49,0x53,0x54,0x25];
        let mut seek = 0usize;
        assert_eq!(read_int_size_string(&data, &mut seek), "%ARTIST%");
    }

    #[test]
    fn test_read_int_byte_size_string() {
        let data: Vec<u8> = vec![0x09,0x00,0x00,0x00,   0x08,   0x25,0x41,0x52,0x54,0x49,0x53,0x54,0x25];
        let mut seek = 0usize;
        assert_eq!(read_int_byte_size_string(&data, &mut seek), "%ARTIST%");
    }

    #[test]
    fn test_write_byte_size_string() {
        let mut out: Vec<u8> = Vec::with_capacity(32);
        write_byte_size_string(&mut out, "FICHIER GUITAR PRO v3.00");
        let expected_result: Vec<u8> = vec![0x18,0x46,0x49,0x43,0x48,0x49,0x45,0x52,
                                            0x20,0x47,0x55,0x49,0x54,0x41,0x52,0x20,
                                            0x50,0x52,0x4f,0x20,0x76,0x33,0x2e,0x30,
                                            0x30];
        assert_eq!(out, expected_result);
    }
    #[test]
    fn test_write_int_size_string() {
        let mut out: Vec<u8> = Vec::with_capacity(16);
        write_int_size_string(&mut out, "%ARTIST%");
        let expected_result: Vec<u8> = vec![0x09,0x00,0x00,0x00,   0x08,0x25,0x41,0x52,0x54,0x49,0x53,0x54,0x25];
        assert_eq!(out, expected_result);
    }
    #[test]
    fn test_write_int_byte_size_string() {
        let mut out: Vec<u8> = Vec::with_capacity(16);
        write_int_byte_size_string(&mut out, "%ARTIST%");
        let expected_result: Vec<u8> = vec![0x09,0x00,0x00,0x00,   0x08,   0x25,0x41,0x52,0x54,0x49,0x53,0x54,0x25];
        assert_eq!(out, expected_result);
    }
}
