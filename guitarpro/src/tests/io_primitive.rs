//! Writing GP's binary primitives.

use crate::error::GpResult;
use crate::io::primitive::{read_byte_size_string, write_fixed_byte_size_string, write_version};

#[test]
fn test_write_fixed_byte_size_string() -> GpResult<()> {
    // Non-ASCII: one byte per character in Windows-1252, padded to the
    // full width so the next field starts where a reader expects it.
    let mut out = Vec::new();
    write_fixed_byte_size_string(&mut out, "Flöte", 40);
    assert_eq!(out.len(), 41);
    assert_eq!(&out[..6], &[5, b'F', b'l', 0xf6, b't', b'e']);
    let mut seek = 0;
    assert_eq!(read_byte_size_string(&out, &mut seek, 40)?, "Flöte");
    assert_eq!(seek, 41);

    // Too long for the field: cut to it instead of overflowing.
    let mut out = Vec::new();
    write_fixed_byte_size_string(&mut out, &"x".repeat(30), 22);
    assert_eq!(out.len(), 23);
    assert_eq!(out[0], 22);
    Ok(())
}

#[test]
fn test_write_version() {
    let mut out: Vec<u8> = Vec::with_capacity(32);
    write_version(&mut out, (3, 0, 0));
    let mut expected_result: Vec<u8> = vec![
        0x18, 0x46, 0x49, 0x43, 0x48, 0x49, 0x45, 0x52, 0x20, 0x47, 0x55, 0x49, 0x54, 0x41, 0x52,
        0x20, 0x50, 0x52, 0x4f, 0x20, 0x76, 0x33, 0x2e, 0x30, 0x30,
    ];
    // Padded to the 30-byte field.
    expected_result.resize(31, 0);
    assert_eq!(out, expected_result);
}
