//! C ABI for the [`guitarpro`] score library.
//!
//! The Rust model — [`guitarpro::Song`], `LoadedScore`, the arena-indexed ID
//! newtypes and the `Song*Ops` traits — never crosses the boundary. Mirroring
//! it in C would mean hand-maintaining a second, lossier copy of the model and
//! would quietly drop the round-trip escape hatches the library works hard to
//! preserve. Instead this crate exposes **data**: scores go in as bytes and
//! come out as bytes, with an opaque handle in between.
//!
//! # Surface
//!
//! | Function | Purpose |
//! |---|---|
//! | `gpio_version` | Library version string |
//! | `gpio_parse` | Bytes → opaque score handle |
//! | `gpio_score_format` | Format a handle was parsed as |
//! | `gpio_score_to_json` | Handle → optimized-model JSON |
//! | `gpio_convert` | Bytes → bytes, one format to another |
//! | `gpio_last_error` | Message for the last failed call |
//! | `gpio_buffer_free` | Release a `GpioBuffer` |
//! | `gpio_score_free` | Release a `GpioScore *` |
//!
//! # Conventions
//!
//! * **Failure** is a `NULL` pointer or a [`GpioBuffer`] whose `data` is
//!   `NULL`. The reason is available from `gpio_last_error` until the next
//!   `gpio_*` call on the same thread.
//! * **Ownership** of every returned buffer and handle passes to the caller,
//!   who must release it with the matching `*_free`. The allocation comes from
//!   Rust, so `free()` must never be used on it.
//! * **Format codes** are passed as `uint32_t` rather than as the enum type so
//!   that an out-of-range value from C is rejected instead of becoming
//!   undefined behaviour. Use the `GPIO_FORMAT_*` constants.
//! * **Panics** are caught at every entry point and reported as errors;
//!   unwinding never crosses the boundary.
//! * **Thread safety**: handles are not synchronized, so do not share one
//!   across threads without external locking. Independent handles on separate
//!   threads are fine, and the error channel is thread-local.
//!
//! # Example
//!
//! ```c
//! GpioScore *score = gpio_parse(data, len, GPIO_FORMAT_AUTO);
//! if (!score) { fprintf(stderr, "%s\n", gpio_last_error()); return 1; }
//!
//! GpioBuffer json = gpio_score_to_json(score);
//! if (json.data) { fwrite(json.data, 1, json.len, stdout); }
//! gpio_buffer_free(json);
//! gpio_score_free(score);
//! ```

mod buffer;
mod error;
mod format;
mod score;

pub use buffer::{GpioBuffer, gpio_buffer_free};
pub use error::gpio_last_error;
pub use format::GpioFormat;
pub use score::{
    GpioScore, gpio_convert, gpio_parse, gpio_score_format, gpio_score_free, gpio_score_to_json,
};

use std::ffi::c_char;

/// This library's version, as a NUL-terminated string (e.g. `"0.1.0"`).
///
/// The pointer refers to static storage: it is always valid and must not be
/// freed.
#[unsafe(no_mangle)]
pub extern "C" fn gpio_version() -> *const c_char {
    // `concat!` appends the terminator at compile time, so no allocation and
    // no fallible `CString::new` are involved.
    const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr().cast::<c_char>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_a_nul_terminated_version() {
        let pointer = gpio_version();
        assert!(!pointer.is_null());
        let version = unsafe { std::ffi::CStr::from_ptr(pointer) }
            .to_str()
            .unwrap_or("");
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
        assert!(
            version.split('.').count() >= 3,
            "expected semver: {version}"
        );
    }
}
