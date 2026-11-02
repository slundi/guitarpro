//! Owned byte buffer handed across the C boundary.

/// A block of bytes produced by the library and owned by the caller.
///
/// `data` is `NULL` only when the producing call failed — check it before use
/// and read `gpio_last_error` for the reason. A successful call always yields
/// a non-`NULL` `data`, even when `len` is 0.
///
/// Release every buffer with `gpio_buffer_free`. Do not call `free()` on
/// `data`: the allocation comes from Rust's allocator, and `capacity` (not
/// `len`) is what makes deallocation possible.
#[repr(C)]
pub struct GpioBuffer {
    /// Pointer to `len` readable bytes, or `NULL` on failure.
    pub data: *mut u8,
    /// Number of meaningful bytes in `data`.
    pub len: usize,
    /// Allocated capacity. Required by `gpio_buffer_free`; treat as opaque.
    pub capacity: usize,
}

impl GpioBuffer {
    /// The failure value: a `NULL` buffer carrying no bytes.
    pub(crate) fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    /// Transfer ownership of `bytes` to the caller.
    ///
    /// An empty `Vec` still yields a non-`NULL` dangling pointer, which keeps
    /// `data != NULL` a reliable success test on the C side.
    pub(crate) fn from_vec(bytes: Vec<u8>) -> Self {
        let mut bytes = std::mem::ManuallyDrop::new(bytes);
        Self {
            data: bytes.as_mut_ptr(),
            len: bytes.len(),
            capacity: bytes.capacity(),
        }
    }
}

/// Release a buffer returned by this library. Passing a `NULL` buffer is a
/// no-op; passing the same buffer twice is undefined behaviour.
///
/// # Safety
///
/// `buffer` must be a buffer returned by this library and not yet freed, with
/// its `data`, `len` and `capacity` fields unmodified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gpio_buffer_free(buffer: GpioBuffer) {
    if buffer.data.is_null() {
        return;
    }
    // Safe: the fields come from `Vec::into_raw_parts` in `from_vec`, and the
    // contract above forbids tampering with them.
    drop(unsafe { Vec::from_raw_parts(buffer.data, buffer.len, buffer.capacity) });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer_signals_failure() {
        let buffer = GpioBuffer::empty();
        assert!(buffer.data.is_null());
        assert_eq!(buffer.len, 0);
        assert_eq!(buffer.capacity, 0);
        // Freeing the failure value must be harmless.
        unsafe { gpio_buffer_free(buffer) };
    }

    #[test]
    fn round_trips_bytes_through_the_boundary() {
        let buffer = GpioBuffer::from_vec(vec![1, 2, 3, 4]);
        assert!(!buffer.data.is_null());
        assert_eq!(buffer.len, 4);
        assert!(buffer.capacity >= 4);

        let seen = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) }.to_vec();
        assert_eq!(seen, vec![1, 2, 3, 4]);

        unsafe { gpio_buffer_free(buffer) };
    }

    #[test]
    fn empty_vec_still_reports_success() {
        let buffer = GpioBuffer::from_vec(Vec::new());
        assert!(
            !buffer.data.is_null(),
            "data != NULL must remain a valid success check"
        );
        assert_eq!(buffer.len, 0);
        unsafe { gpio_buffer_free(buffer) };
    }

    #[test]
    fn preserves_capacity_beyond_length() {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(b"abc");
        let buffer = GpioBuffer::from_vec(bytes);
        assert_eq!(buffer.len, 3);
        assert!(buffer.capacity >= 64);
        unsafe { gpio_buffer_free(buffer) };
    }
}
