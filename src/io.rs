use fraction::ToPrimitive;

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

/// Read a float and increase the cursor position by 4 (4 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the float value
pub(crate) fn read_float(data: &[u8], seek: &mut usize ) -> f32 {
    if data.len() < *seek + 8 {panic!("End of file reached");}
    let n = f32::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3]]);
    *seek += 4;
    n
}

/// Read a double and increase the cursor position by 8 (8 little-endian bytes)
/// * `data` - array of bytes
/// * `seek` - start position to read
/// * returns the float value
pub(crate) fn read_double(data: &[u8], seek: &mut usize ) -> f64 {
    if data.len() >= *seek {panic!("End of file reached");}
    let n = f64::from_le_bytes([data[*seek], data[*seek+1], data[*seek+2], data[*seek+3], data[*seek+4], data[*seek+5], data[*seek+6], data[*seek+7]]);
    *seek += 8;
    n
}

/// Read length of the string stored in 1 integer and followed by character bytes.
pub(crate) fn read_int_size_string(data: &[u8], seek: &mut usize) -> String {
    let n = read_int(data, seek).to_usize().unwrap();
    let parse = std::str::from_utf8(&data[*seek..*seek+n]);
    if parse.is_err() {panic!("Unable to read string");}
    *seek += n;
    parse.unwrap().to_string()
}

/// Read length of the string increased by 1 and stored in 1 integer followed by length of the string in 1 byte and finally followed by character bytes.
pub(crate) fn read_int_byte_size_string(data: &[u8], seek: &mut usize) -> String {
    //TODO: read_int_size_string is used instead, but it should be fixed
    let s = read_int(data, seek).to_usize().unwrap() - 1;
    let n = read_byte(data, seek).to_usize().unwrap();
    let count = if s > 0 {s} else {n};
    //println!("read_byte_size_string: n={}", n);
    let parse = std::str::from_utf8(&data[*seek..*seek+count]);
    if parse.is_err() {panic!("Unable to read string");}
    *seek += count;
    String::new()
}

/// Read length of the string stored in 1 byte and followed by character bytes.
pub(crate) fn read_byte_size_string(data: &[u8], seek: &mut usize, size: Option<usize>) -> String {
    let n = read_byte(data, seek).to_usize().unwrap();
    if size.is_none() {
        //println!("read_byte_size_string: n={}", n);
        let parse = std::str::from_utf8(&data[*seek..*seek+n]);
        if parse.is_err() {panic!("Unable to read string");}
        *seek += n;
        parse.unwrap().to_string()
    } else {
        let s = size.unwrap();
        let l = if n>s {s} else {n};
        //println!("read_byte_size_string: n={}", n);
        let parse = std::str::from_utf8(&data[*seek..*seek+l]);
        if parse.is_err() {panic!("Unable to read string");}
        *seek += s;
        parse.unwrap().to_string()
    }
}

const VERSIONS: [((u8,u8,u8), bool, &str); 10] = [((3, 0, 0), false, "FICHIER GUITAR PRO v3.00"),

                                                  ((4, 0, 0), false, "FICHIER GUITAR PRO v4.00"),
                                                  ((4, 0, 6), false, "FICHIER GUITAR PRO v4.06"),
                                                  ((4, 0, 6), true, "CLIPBOARD GUITAR PRO 4.0 [c6]"),

                                                  ((5, 0, 0), false, "FICHIER GUITAR PRO v5.00"),
                                                  ((5, 1, 0), false, "FICHIER GUITAR PRO v5.10"),
                                                  ((5, 2, 0), false, "FICHIER GUITAR PRO v5.10"),  // sic
                                                  ((5, 0, 0), true, "CLIPBOARD GP 5.0"),
                                                  ((5, 1, 0), true, "CLIPBOARD GP 5.1"),
                                                  ((5, 2, 0), true, "CLIPBOARD GP 5.2")];

/// Read the file version. It is on the first 31 bytes (1st byte is the real length, the following 30 bytes contain the version string) of the file.
/// * `data` - array of bytes
/// * `seek` - cursor that will be incremented
/// * returns version
pub(crate) fn read_version_string(data: &[u8], seek: &mut usize) -> crate::headers::Version {
    let mut v = crate::headers::Version {data: read_byte_size_string(data, seek, Some(30)), number: (5,2,0), clipboard: false};
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
pub(crate) fn write_u32(data: &mut Vec<u8>, value: u32) {data.extend(value.to_le_bytes());}
pub(crate) fn write_i16(data: &mut Vec<u8>, value: i16) {data.extend(value.to_le_bytes());}
pub(crate) fn write_u16(data: &mut Vec<u8>, value: u16) {data.extend(value.to_le_bytes());}
pub(crate) fn write_f32(data: &mut Vec<u8>, value: f32) {data.extend(value.to_le_bytes());}
pub(crate) fn write_f64(data: &mut Vec<u8>, value: f64) {data.extend(value.to_le_bytes());}
pub(crate) fn write_byte_string(data: &mut Vec<u8>, value: &str) {
    write_byte(data, value.len().to_u8().unwrap());
    data.extend(value.as_bytes());
}

pub(crate) fn write_version(data: &mut Vec<u8>, version: (u8,u8,u8)) {
    for v in VERSIONS {
        if version == v.0 {
            write_byte_string(data, v.2);
            write_placeholder_default(data, 30 - v.2.len());
            break;
        }
    }
}
