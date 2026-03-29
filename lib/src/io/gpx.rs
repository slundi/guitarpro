use std::io::{Read, Cursor};
use zip::ZipArchive;
use crate::io::gpif::Gpif;
use crate::error::GpResult;
use quick_xml::de::from_str;

/// Reads a .gp (GP7+) file which is a ZIP archive containing 'Content/score.gpif'.
pub fn read_gp(data: &[u8]) -> GpResult<Gpif> {
    let cursor = Cursor::new(data);
    let mut zip = ZipArchive::new(cursor).map_err(|e| format!("Zip error: {}", e))?;

    // Standard path for GP7 files
    let mut file = zip.by_name("Content/score.gpif").map_err(|e| format!("Could not find score.gpif: {}", e))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| format!("Read error: {}", e))?;

    let gpif: Gpif = from_str(&contents).map_err(|e| format!("XML Parse error: {}", e))?;
    Ok(gpif)
}

// ---------------------------------------------------------------------------
// GP6 (.gpx) BCFZ/BCFS container support
// ---------------------------------------------------------------------------

const BCFZ_MAGIC: &[u8; 4] = b"BCFZ";
const BCFS_MAGIC: &[u8; 4] = b"BCFS";
const SECTOR_SIZE: usize = 0x1000;

/// Bit-level reader for BCFZ decompression.
/// Reads bits MSB-first within each byte.
struct BitStream<'a> {
    data: &'a [u8],
    bit_position: usize,
}

impl<'a> BitStream<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitStream { data, bit_position: 0 }
    }

    fn byte_offset(&self) -> usize {
        self.bit_position / 8
    }

    fn is_eof(&self) -> bool {
        self.byte_offset() >= self.data.len()
    }

    /// Read a single bit (MSB-first within the current byte).
    fn read_bit(&mut self) -> u8 {
        let byte_index = self.bit_position / 8;
        let bit_index = 7 - (self.bit_position % 8);
        if byte_index >= self.data.len() {
            return 0;
        }
        self.bit_position += 1;
        (self.data[byte_index] >> bit_index) & 1
    }

    /// Read `count` bits, accumulated big-endian (MSB first).
    fn read_bits(&mut self, count: usize) -> u32 {
        let mut result: u32 = 0;
        for _ in 0..count {
            result = (result << 1) | self.read_bit() as u32;
        }
        result
    }

    /// Read `count` bits, accumulated little-endian (LSB first / "reversed").
    fn read_bits_reversed(&mut self, count: usize) -> u32 {
        let mut result: u32 = 0;
        for i in 0..count {
            result |= (self.read_bit() as u32) << i;
        }
        result
    }
}

/// Decompress a BCFZ-compressed buffer into raw BCFS data.
fn decompress_bcfz(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 8 {
        return Err("BCFZ data too short".to_string());
    }
    if &data[0..4] != BCFZ_MAGIC {
        return Err(format!("Expected BCFZ magic, got {:?}", &data[0..4]));
    }

    let raw_len = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if raw_len < 0 {
        return Err(format!("BCFZ: negative expected length {}", raw_len));
    }
    let expected_len = raw_len as usize;
    let mut output = Vec::with_capacity(expected_len);
    let mut bits = BitStream::new(&data[8..]);

    while output.len() < expected_len && !bits.is_eof() {
        let flag = bits.read_bit();
        if flag == 1 {
            // Back-reference (LZ77-style)
            let word_size = bits.read_bits(4) as usize;
            let offset = bits.read_bits_reversed(word_size) as usize;
            let size = bits.read_bits_reversed(word_size) as usize;
            if offset == 0 || offset > output.len() {
                return Err(format!(
                    "BCFZ: invalid back-reference offset {} (output len {})",
                    offset, output.len()
                ));
            }
            let source_start = output.len() - offset;
            // LZ77 overlapping copy: when size > offset the source overlaps
            // the destination, so we must copy byte-by-byte with modular indexing.
            for i in 0..size {
                let byte = output[source_start + (i % offset)];
                output.push(byte);
            }
        } else {
            // Literal bytes
            let size = bits.read_bits_reversed(2) as usize;
            for _ in 0..size {
                let byte = bits.read_bits(8) as u8;
                output.push(byte);
            }
        }
    }

    output.truncate(expected_len);
    Ok(output)
}

/// A file extracted from a BCFS virtual filesystem.
struct BcfsFile {
    name: String,
    data: Vec<u8>,
}

/// Read the integer at the given offset (little-endian i32).
fn read_le_i32(data: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Parse the BCFS virtual filesystem and extract all files.
///
/// The BCFS format starts with a 4-byte magic ("BCFS"), followed by sector-based data.
/// The Java reference implementation (TuxGuitar) strips the 4-byte magic and then treats
/// the remaining data as a virtual disk with 0x1000-byte sectors.
fn parse_bcfs(data: &[u8]) -> Result<Vec<BcfsFile>, String> {
    if data.len() < 4 {
        return Err("BCFS data too short".to_string());
    }
    if &data[0..4] != BCFS_MAGIC {
        return Err(format!("Expected BCFS magic, got {:?}", &data[0..4]));
    }

    // Strip the 4-byte magic — all sector offsets are relative to this base.
    let disk = &data[4..];
    let mut files = Vec::new();
    let mut sector_offset = SECTOR_SIZE; // Skip sector 0 (header area)

    while sector_offset + 3 < disk.len() {
        let entry_type = read_le_i32(disk, sector_offset);

        if entry_type == 2 {
            // File directory entry — requires at least 0x98 bytes from sector_offset
            if sector_offset + 0x98 > disk.len() {
                sector_offset += SECTOR_SIZE;
                continue;
            }

            let name_start = sector_offset + 4;
            let name_end = (name_start + 127).min(disk.len());
            let name_bytes = &disk[name_start..name_end];
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
            let name = String::from_utf8_lossy(&name_bytes[..name_len]).to_string();

            let file_size = read_le_i32(disk, sector_offset + 0x8C) as usize;

            // Block index table at +0x94, array of i32, terminated by 0
            let mut file_data = Vec::with_capacity(file_size);
            let mut idx_offset = sector_offset + 0x94;
            loop {
                if idx_offset + 4 > sector_offset + SECTOR_SIZE {
                    break;
                }
                let block_idx = read_le_i32(disk, idx_offset);
                if block_idx == 0 {
                    break;
                }
                let block_start = block_idx as usize * SECTOR_SIZE;
                let block_end = (block_start + SECTOR_SIZE).min(disk.len());
                if block_start < disk.len() {
                    file_data.extend_from_slice(&disk[block_start..block_end]);
                }
                idx_offset += 4;
            }

            file_data.truncate(file_size);
            if !name.is_empty() {
                files.push(BcfsFile { name, data: file_data });
            }
        }

        sector_offset += SECTOR_SIZE;
    }

    Ok(files)
}

/// Reads a .gpx (GP6) file which is a BCFZ/BCFS container holding 'score.gpif'.
pub fn read_gpx(data: &[u8]) -> GpResult<Gpif> {
    let decompressed = decompress_bcfz(data)?;
    let files = parse_bcfs(&decompressed)?;

    let score_file = files.iter()
        .find(|f| f.name == "score.gpif")
        .ok_or_else(|| {
            let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
            format!("score.gpif not found in GPX archive. Files found: {:?}", names)
        })?;

    let xml_str = std::str::from_utf8(&score_file.data)
        .map_err(|e| format!("UTF-8 error in score.gpif: {}", e))?;

    let gpif: Gpif = from_str(xml_str)
        .map_err(|e| format!("XML parse error in score.gpif: {}", e))?;

    Ok(gpif)
}
