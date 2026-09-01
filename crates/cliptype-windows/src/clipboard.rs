//! Bounded current `CF_UNICODETEXT` acquisition.

use std::{mem::size_of, ptr::null_mut, slice};

use cliptype_core::{ByteCount, NativeByteLimit, SensitiveText};
use cliptype_platform::{ClipboardError, ClipboardPort, NativeError, NativeErrorKind};
use windows_sys::Win32::{
    Foundation::{ERROR_ACCESS_DENIED, GetLastError, HGLOBAL},
    System::{
        DataExchange::{
            CloseClipboard, CountClipboardFormats, GetClipboardData, IsClipboardFormatAvailable,
            OpenClipboard,
        },
        Memory::{GlobalLock, GlobalSize, GlobalUnlock},
    },
};

// `CF_UNICODETEXT` is the stable Win32 standard clipboard-format identifier.
// windows-sys 0.61 keeps this winuser constant outside DataExchange while the
// related clipboard functions remain in that module, so keeping the documented
// numeric identifier local avoids importing an unrelated header module.
const CF_UNICODETEXT: u32 = 13;

/// Windows adapter for one bounded current Unicode clipboard read.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsClipboard;

impl WindowsClipboard {
    pub const fn new() -> Self {
        Self
    }
}

impl ClipboardPort for WindowsClipboard {
    fn read_current_text(
        &self,
        hard_limit: NativeByteLimit,
    ) -> Result<SensitiveText, ClipboardError> {
        let _clipboard = ClipboardGuard::open()?;

        // SAFETY: the clipboard is open on this thread and the format constant
        // is defined by Win32. The call does not transfer ownership.
        if unsafe { IsClipboardFormatAvailable(CF_UNICODETEXT) } == 0 {
            // SAFETY: the clipboard is open and this query has no pointer input.
            return if unsafe { CountClipboardFormats() } == 0 {
                Err(ClipboardError::Empty)
            } else {
                Err(ClipboardError::NonText)
            };
        }

        // SAFETY: the clipboard is open; the returned handle remains owned by
        // the clipboard and is never freed by ClipType.
        let handle = unsafe { GetClipboardData(CF_UNICODETEXT) };
        if handle.is_null() {
            return Err(ClipboardError::Native(last_native_error(
                NativeErrorKind::InvalidData,
            )));
        }
        let global = handle as HGLOBAL;

        // SAFETY: `global` is the clipboard-owned global-memory handle returned
        // for `CF_UNICODETEXT`. No ownership transfer occurs.
        let byte_size = unsafe { GlobalSize(global) };
        let unit_count = validate_allocation_size(byte_size, hard_limit)?;

        let locked = GlobalLockGuard::lock(global)?;
        // SAFETY: `locked.pointer` is non-null while the guard is alive;
        // `unit_count` came from the bounded allocation size, is aligned to
        // `u16`, and cannot exceed the configured hard limit.
        let units = unsafe { slice::from_raw_parts(locked.pointer.cast::<u16>(), unit_count) };
        let text = decode_utf16_units(units)?;

        Ok(text)
    }
}

fn validate_allocation_size(
    byte_size: usize,
    hard_limit: NativeByteLimit,
) -> Result<usize, ClipboardError> {
    let observed = ByteCount::new(byte_size);
    if !hard_limit.allows(observed) {
        return Err(ClipboardError::TooLarge {
            observed: Some(observed),
            limit: hard_limit,
        });
    }
    if byte_size < size_of::<u16>() || !byte_size.is_multiple_of(size_of::<u16>()) {
        return Err(ClipboardError::Malformed);
    }
    if byte_size > isize::MAX as usize {
        return Err(ClipboardError::Malformed);
    }

    Ok(byte_size / size_of::<u16>())
}

fn decode_utf16_units(units: &[u16]) -> Result<SensitiveText, ClipboardError> {
    let terminator = units
        .iter()
        .position(|unit| *unit == 0)
        .ok_or(ClipboardError::Malformed)?;
    if terminator == 0 {
        return Err(ClipboardError::Empty);
    }

    String::from_utf16(&units[..terminator])
        .map(SensitiveText::new)
        .map_err(|_| ClipboardError::Malformed)
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Result<Self, ClipboardError> {
        // SAFETY: null requests clipboard access without assigning an owner.
        if unsafe { OpenClipboard(null_mut()) } == 0 {
            let code = last_error_code();
            return if code == Some(ERROR_ACCESS_DENIED) {
                Err(ClipboardError::Busy)
            } else {
                Err(ClipboardError::Native(NativeError::new(
                    NativeErrorKind::TemporarilyUnavailable,
                    code,
                )))
            };
        }

        Ok(Self)
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        // SAFETY: construction succeeds only after `OpenClipboard`; this guard
        // is neither cloned nor leaked by the adapter and closes on every path.
        let _ = unsafe { CloseClipboard() };
    }
}

struct GlobalLockGuard {
    handle: HGLOBAL,
    pointer: *mut core::ffi::c_void,
}

impl GlobalLockGuard {
    fn lock(handle: HGLOBAL) -> Result<Self, ClipboardError> {
        // SAFETY: the handle came from `GetClipboardData(CF_UNICODETEXT)` and
        // remains valid while the clipboard guard is alive.
        let pointer = unsafe { GlobalLock(handle) };
        if pointer.is_null() {
            return Err(ClipboardError::Native(last_native_error(
                NativeErrorKind::InvalidData,
            )));
        }

        Ok(Self { handle, pointer })
    }
}

impl Drop for GlobalLockGuard {
    fn drop(&mut self) {
        // SAFETY: each successfully constructed guard represents one matching
        // `GlobalLock`. The clipboard still owns the underlying allocation.
        // `GlobalUnlock` returning zero can mean successful final unlock, so
        // cleanup does not interpret the return value here.
        let _ = unsafe { GlobalUnlock(self.handle) };
    }
}

fn last_error_code() -> Option<u32> {
    // SAFETY: `GetLastError` has no pointer or ownership preconditions.
    let code = unsafe { GetLastError() };
    (code != 0).then_some(code)
}

fn last_native_error(kind: NativeErrorKind) -> NativeError {
    NativeError::new(kind, last_error_code())
}

#[cfg(test)]
mod tests {
    use super::{decode_utf16_units, validate_allocation_size};
    use cliptype_core::NativeByteLimit;
    use cliptype_platform::ClipboardError;

    fn nul_terminated(value: &str) -> Vec<u16> {
        value.encode_utf16().chain([0]).collect()
    }

    #[test]
    fn decodes_ascii_cjk_supplementary_and_combining_text() {
        let source = "A中😀e\u{301}";
        let units = nul_terminated(source);
        let text = decode_utf16_units(&units).expect("valid UTF-16 fixture");

        assert_eq!(text.expose(), source);
    }

    #[test]
    fn rejects_empty_missing_terminator_and_invalid_surrogate_data() {
        assert!(matches!(
            decode_utf16_units(&[0]),
            Err(ClipboardError::Empty)
        ));
        assert!(matches!(
            decode_utf16_units(&[b'a' as u16]),
            Err(ClipboardError::Malformed)
        ));
        assert!(matches!(
            decode_utf16_units(&[0xD800, 0]),
            Err(ClipboardError::Malformed)
        ));
    }

    #[test]
    fn allocation_validation_is_bounded_and_aligned() {
        let limit = NativeByteLimit::new(8).expect("test limit");

        assert_eq!(validate_allocation_size(8, limit), Ok(4));
        assert!(matches!(
            validate_allocation_size(1, limit),
            Err(ClipboardError::Malformed)
        ));
        assert!(matches!(
            validate_allocation_size(3, limit),
            Err(ClipboardError::Malformed)
        ));
        assert!(matches!(
            validate_allocation_size(10, limit),
            Err(ClipboardError::TooLarge { .. })
        ));
    }

    #[test]
    fn diagnostics_do_not_expose_fixture_content() {
        let marker = "CLIPBOARD_PRIVATE_SENTINEL_29";
        let units = nul_terminated(marker);
        let text = decode_utf16_units(&units).expect("valid fixture");

        assert!(!format!("{text:?}").contains(marker));
    }
}
