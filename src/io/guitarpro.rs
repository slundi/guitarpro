use crate::base::*;


const VERSION_1_0X: u8 = 10;
const VERSION_2_2X: u8 = 22;
const VERSION_3_00: u8 = 30;
const VERSION_4_0X: u8 = 40;
const VERSION_5_00: u8 = 50;
const VERSION_5_10: u8 = 51;

impl Song {
    pub fn gp_read_data(&mut self, data: &Vec<u8>) {
        let mut seek: usize = 0;
        //version
        let mut version: u8;
        let tmp = read_string(data, &mut seek, false);
        if tmp == "FICHIER GUITARE PRO v1" || tmp == "FICHIER GUITARE PRO v1.01" || tmp == "FICHIER GUITARE PRO v1.02" || tmp == "FICHIER GUITARE PRO v1.03" || tmp == "FICHIER GUITARE PRO v1.04" {version = VERSION_1_0X}
        else if tmp == "FICHIER GUITARE PRO v2.20" || tmp == "FICHIER GUITARE PRO v2.21" {version = VERSION_2_2X}
        else if tmp == "FICHIER GUITAR PRO v3.00" {version = VERSION_3_00}
        else if tmp == "FICHIER GUITAR PRO v4.00" || tmp == "FICHIER GUITAR PRO v4.06" || tmp == "FICHIER GUITAR PRO L4.06" {version = VERSION_4_0X}
        else if tmp == "FICHIER GUITAR PRO v5.00" {version = VERSION_5_00;}
        else if tmp == "FICHIER GUITAR PRO v5.10" {version = VERSION_5_10;}

        /*self.name        = read_string(data, &mut seek, false);
        self.artist      = read_string(data, &mut seek, false);
        self.album       = read_string(data, &mut seek, false);
        self.author      = read_string(data, &mut seek, false);
        self.date        = read_string(data, &mut seek, false);
        self.copyright   = read_string(data, &mut seek, false);
        self.writer      = read_string(data, &mut seek, false);
        self.transcriber = read_string(data, &mut seek, false);
        self.comments    = read_string(data, &mut seek, true);*/
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
        s.push(data[*seek + i * 2] as char);
    }
    *seek += n;
    return s;
}
