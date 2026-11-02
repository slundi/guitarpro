//! The opaque score handle and the public C entry points.

use crate::buffer::GpioBuffer;
use crate::error::guard;
use crate::format::{self, GpioFormat};

use guitarpro::Song;

/// A parsed score. Opaque to C: create it with `gpio_parse`, release it with
/// `gpio_score_free`.
pub struct GpioScore {
    song: Song,
    /// The format the bytes were actually read as — the detected one when the
    /// caller passed `GPIO_FORMAT_AUTO`.
    format: GpioFormat,
}

/// Borrow `len` bytes at `data` as a slice.
///
/// # Safety
///
/// When `len` is non-zero, `data` must point to `len` initialized bytes that
/// stay valid and unmutated for the duration of the call.
unsafe fn as_slice<'a>(data: *const u8, len: usize) -> Result<&'a [u8], String> {
    if data.is_null() {
        return Err("data pointer is NULL".to_string());
    }
    if len == 0 {
        // `from_raw_parts` rejects a zero length paired with an arbitrary
        // pointer, and an empty score is an error in every format anyway.
        return Ok(&[]);
    }
    Ok(unsafe { std::slice::from_raw_parts(data, len) })
}

/// Borrow a score handle.
///
/// # Safety
///
/// `score` must be `NULL` or a live handle from `gpio_parse`.
unsafe fn as_score<'a>(score: *const GpioScore) -> Result<&'a GpioScore, String> {
    if score.is_null() {
        return Err("score handle is NULL".to_string());
    }
    Ok(unsafe { &*score })
}

/// Parse `len` bytes at `data` into a score.
///
/// Pass `GPIO_FORMAT_AUTO` as `format` to detect the format from the content,
/// then read it back with `gpio_score_format`.
///
/// Returns `NULL` on failure; call `gpio_last_error` for the reason. Release
/// the handle with `gpio_score_free`.
///
/// # Safety
///
/// `data` must point to `len` readable bytes for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gpio_parse(data: *const u8, len: usize, format: u32) -> *mut GpioScore {
    guard(std::ptr::null_mut(), || {
        let bytes = unsafe { as_slice(data, len) }?;
        let requested = GpioFormat::from_raw(format)
            .ok_or_else(|| format!("unknown input format code {format}"))?;
        let (song, detected) = format::decode(bytes, requested)?;
        Ok(Box::into_raw(Box::new(GpioScore {
            song,
            format: detected,
        })))
    })
}

/// Report the format `score` was parsed as.
///
/// Returns `GPIO_FORMAT_AUTO` (0) when `score` is `NULL`, which is never a
/// valid parsed format.
///
/// # Safety
///
/// `score` must be `NULL` or a live handle from `gpio_parse`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gpio_score_format(score: *const GpioScore) -> u32 {
    guard(GpioFormat::Auto as u32, || {
        Ok(unsafe { as_score(score) }?.format as u32)
    })
}

/// Serialize `score` as JSON in the optimized score model.
///
/// Returns a buffer with `data == NULL` on failure; call `gpio_last_error` for
/// the reason. Release the buffer with `gpio_buffer_free`.
///
/// # Safety
///
/// `score` must be `NULL` or a live handle from `gpio_parse`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gpio_score_to_json(score: *const GpioScore) -> GpioBuffer {
    guard(GpioBuffer::empty(), || {
        let handle = unsafe { as_score(score) }?;
        let bytes = format::encode(&handle.song, GpioFormat::Score)?;
        Ok(GpioBuffer::from_vec(bytes))
    })
}

/// Convert `len` bytes at `data` from `input_format` to `output_format` in one
/// step, without exposing an intermediate handle.
///
/// `input_format` may be `GPIO_FORMAT_AUTO`; `output_format` may not.
///
/// Returns a buffer with `data == NULL` on failure; call `gpio_last_error` for
/// the reason. Release the buffer with `gpio_buffer_free`.
///
/// # Safety
///
/// `data` must point to `len` readable bytes for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gpio_convert(
    data: *const u8,
    len: usize,
    input_format: u32,
    output_format: u32,
) -> GpioBuffer {
    guard(GpioBuffer::empty(), || {
        let bytes = unsafe { as_slice(data, len) }?;
        let source = GpioFormat::from_raw(input_format)
            .ok_or_else(|| format!("unknown input format code {input_format}"))?;
        let target = GpioFormat::from_raw(output_format)
            .ok_or_else(|| format!("unknown output format code {output_format}"))?;
        let (song, _) = format::decode(bytes, source)?;
        Ok(GpioBuffer::from_vec(format::encode(&song, target)?))
    })
}

/// Release a score handle. Passing `NULL` is a no-op; passing the same handle
/// twice is undefined behaviour.
///
/// # Safety
///
/// `score` must be `NULL` or a live handle from `gpio_parse` that has not
/// already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gpio_score_free(score: *mut GpioScore) {
    if score.is_null() {
        return;
    }
    // Dropping a `Song` should not panic, but unwinding out of here would be
    // undefined behaviour, so contain it.
    guard((), || {
        drop(unsafe { Box::from_raw(score) });
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::gpio_buffer_free;
    use crate::error::gpio_last_error;

    fn sample(relative: &str) -> Vec<u8> {
        let path = format!("../{relative}");
        std::fs::read(&path).unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
    }

    fn last_error() -> Option<String> {
        let pointer = gpio_last_error();
        if pointer.is_null() {
            return None;
        }
        Some(
            unsafe { std::ffi::CStr::from_ptr(pointer) }
                .to_string_lossy()
                .into_owned(),
        )
    }

    #[test]
    fn parses_and_reports_the_detected_format() {
        let bytes = sample("test/Chords.gp4");
        let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), GpioFormat::Auto as u32) };
        assert!(!handle.is_null(), "parse failed: {:?}", last_error());
        assert_eq!(unsafe { gpio_score_format(handle) }, GpioFormat::Gp4 as u32);
        assert_eq!(last_error(), None);
        unsafe { gpio_score_free(handle) };
    }

    #[test]
    fn rejects_a_null_data_pointer() {
        let handle = unsafe { gpio_parse(std::ptr::null(), 16, GpioFormat::Gp5 as u32) };
        assert!(handle.is_null());
        assert!(last_error().is_some_and(|message| message.contains("NULL")));
    }

    #[test]
    fn rejects_an_unknown_format_code() {
        let bytes = sample("test/Chords.gp5");
        let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), 999) };
        assert!(handle.is_null());
        assert!(last_error().is_some_and(|message| message.contains("999")));
    }

    #[test]
    fn rejects_an_empty_input() {
        let empty: [u8; 0] = [];
        let handle = unsafe { gpio_parse(empty.as_ptr(), 0, GpioFormat::Auto as u32) };
        assert!(handle.is_null());
        assert!(last_error().is_some());
    }

    #[test]
    fn reports_auto_for_a_null_handle() {
        assert_eq!(
            unsafe { gpio_score_format(std::ptr::null()) },
            GpioFormat::Auto as u32
        );
        assert!(last_error().is_some_and(|message| message.contains("NULL")));
    }

    #[test]
    fn serializes_a_score_to_json() {
        let bytes = sample("test/Chords.gp5");
        let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), GpioFormat::Gp5 as u32) };
        assert!(!handle.is_null(), "parse failed: {:?}", last_error());

        let buffer = unsafe { gpio_score_to_json(handle) };
        assert!(!buffer.data.is_null(), "json failed: {:?}", last_error());
        assert!(buffer.len > 0);

        let json = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) };
        let parsed: serde_json::Value = serde_json::from_slice(json)
            .unwrap_or_else(|error| panic!("output was not valid JSON: {error}"));
        assert!(parsed.is_object());

        unsafe { gpio_buffer_free(buffer) };
        unsafe { gpio_score_free(handle) };
    }

    #[test]
    fn json_on_a_null_handle_yields_an_empty_buffer() {
        let buffer = unsafe { gpio_score_to_json(std::ptr::null()) };
        assert!(buffer.data.is_null());
        assert_eq!(buffer.len, 0);
        assert!(last_error().is_some());
        unsafe { gpio_buffer_free(buffer) };
    }

    #[test]
    fn converts_between_formats_in_one_call() {
        let bytes = sample("test/Chords.gp4");
        let buffer = unsafe {
            gpio_convert(
                bytes.as_ptr(),
                bytes.len(),
                GpioFormat::Auto as u32,
                GpioFormat::MusicXml as u32,
            )
        };
        assert!(!buffer.data.is_null(), "convert failed: {:?}", last_error());

        let out = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) };
        assert!(out.starts_with(b"<?xml"));

        unsafe { gpio_buffer_free(buffer) };
    }

    #[test]
    fn convert_rejects_auto_as_the_target() {
        let bytes = sample("test/Chords.gp5");
        let buffer = unsafe {
            gpio_convert(
                bytes.as_ptr(),
                bytes.len(),
                GpioFormat::Gp5 as u32,
                GpioFormat::Auto as u32,
            )
        };
        assert!(buffer.data.is_null());
        assert!(last_error().is_some_and(|message| message.contains("AUTO")));
        unsafe { gpio_buffer_free(buffer) };
    }

    #[test]
    fn convert_rejects_unknown_format_codes() {
        let bytes = sample("test/Chords.gp5");
        let buffer = unsafe { gpio_convert(bytes.as_ptr(), bytes.len(), 42, 3) };
        assert!(buffer.data.is_null());
        assert!(last_error().is_some_and(|message| message.contains("42")));
        unsafe { gpio_buffer_free(buffer) };

        let buffer = unsafe { gpio_convert(bytes.as_ptr(), bytes.len(), 3, 42) };
        assert!(buffer.data.is_null());
        assert!(last_error().is_some_and(|message| message.contains("42")));
        unsafe { gpio_buffer_free(buffer) };
    }

    #[test]
    fn freeing_a_null_handle_is_a_no_op() {
        unsafe { gpio_score_free(std::ptr::null_mut()) };
    }

    #[test]
    fn a_successful_call_clears_a_previous_error() {
        // Fail once...
        let handle = unsafe { gpio_parse(std::ptr::null(), 8, 0) };
        assert!(handle.is_null());
        assert!(last_error().is_some());

        // ...then succeed: the stale message must not linger.
        let bytes = sample("test/Chords.gp3");
        let handle = unsafe { gpio_parse(bytes.as_ptr(), bytes.len(), GpioFormat::Auto as u32) };
        assert!(!handle.is_null());
        assert_eq!(last_error(), None);
        unsafe { gpio_score_free(handle) };
    }
}
