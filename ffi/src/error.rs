//! Thread-local error channel.
//!
//! Every fallible entry point reports failure through its return value (a
//! `NULL` pointer or an empty [`crate::buffer::GpioBuffer`]) and leaves a
//! human-readable message here for [`gpio_last_error`] to retrieve.

use std::cell::RefCell;
use std::ffi::{CString, c_char};
use std::panic::AssertUnwindSafe;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

/// Record `message` as the calling thread's most recent error.
pub(crate) fn set_last_error(message: impl Into<String>) {
    // An interior NUL would silently truncate the C string, so replace the
    // offending bytes rather than dropping the whole diagnostic.
    let sanitized = message.into().replace('\0', "?");
    let Ok(encoded) = CString::new(sanitized) else {
        return;
    };
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(encoded));
}

/// Forget the calling thread's most recent error.
pub(crate) fn clear_last_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
}

/// Run `body`, converting both `Err` and a panic into `fallback` plus a stored
/// error message.
///
/// Unwinding across an `extern "C"` boundary is undefined behaviour, so every
/// entry point funnels through here. The previous error is cleared on entry so
/// a successful call never leaves a stale message behind.
pub(crate) fn guard<T>(fallback: T, body: impl FnOnce() -> Result<T, String>) -> T {
    clear_last_error();
    match std::panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(Ok(value)) => value,
        Ok(Err(message)) => {
            set_last_error(message);
            fallback
        }
        Err(_) => {
            set_last_error("internal panic in guitarpro-ffi (please report this as a bug)");
            fallback
        }
    }
}

/// Return the calling thread's most recent error message, or `NULL` if the
/// last call on this thread succeeded.
///
/// The returned string is owned by the library and stays valid until the next
/// `gpio_*` call on the same thread. Copy it if you need to keep it. Errors are
/// tracked per thread, so a message raised on one thread is never visible from
/// another.
#[unsafe(no_mangle)]
pub extern "C" fn gpio_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| match slot.borrow().as_ref() {
        Some(message) => message.as_ptr(),
        None => std::ptr::null(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Read the current thread-local error back as a Rust string.
    fn current_error() -> Option<String> {
        let pointer = gpio_last_error();
        if pointer.is_null() {
            return None;
        }
        // Safe: the pointer comes straight out of the thread-local `CString`
        // and nothing mutates it between the call above and this read.
        Some(
            unsafe { std::ffi::CStr::from_ptr(pointer) }
                .to_string_lossy()
                .into_owned(),
        )
    }

    #[test]
    fn reports_no_error_initially() {
        clear_last_error();
        assert_eq!(current_error(), None);
    }

    #[test]
    fn stores_and_clears_a_message() {
        set_last_error("boom");
        assert_eq!(current_error().as_deref(), Some("boom"));
        clear_last_error();
        assert_eq!(current_error(), None);
    }

    #[test]
    fn replaces_interior_nul_bytes() {
        set_last_error("before\0after");
        assert_eq!(current_error().as_deref(), Some("before?after"));
        clear_last_error();
    }

    #[test]
    fn guard_passes_through_success_and_clears_previous_error() {
        set_last_error("stale");
        let value = guard(0u32, || Ok(7));
        assert_eq!(value, 7);
        assert_eq!(current_error(), None);
    }

    #[test]
    fn guard_converts_err_into_fallback() {
        let value = guard(-1i32, || Err("bad input".to_string()));
        assert_eq!(value, -1);
        assert_eq!(current_error().as_deref(), Some("bad input"));
        clear_last_error();
    }

    #[test]
    fn guard_converts_panic_into_fallback() {
        // Silence the default hook so the expected panic does not pollute the
        // test output.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let value = guard(-1i32, || panic!("kaboom"));
        std::panic::set_hook(previous);

        assert_eq!(value, -1);
        assert!(
            current_error()
                .is_some_and(|message| message.contains("internal panic in guitarpro-ffi"))
        );
        clear_last_error();
    }

    #[test]
    fn errors_are_isolated_per_thread() {
        set_last_error("main thread");
        let seen_in_other_thread = std::thread::spawn(current_error)
            .join()
            .unwrap_or(Some("join failed".into()));
        assert_eq!(seen_in_other_thread, None);
        assert_eq!(current_error().as_deref(), Some("main thread"));
        clear_last_error();
    }
}
