//! Foreground-target, focus, and integrity evidence for Windows.

use std::{mem::size_of, ptr::null_mut};

use cliptype_core::{EvidenceStrength, IntegrityRelation};
use cliptype_platform::{
    NativeError, NativeErrorKind, TargetCaptureError, TargetComparison, TargetEvidence,
    TargetMetadata, TargetPort,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, HANDLE, HWND},
    Security::{
        GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TOKEN_MANDATORY_LABEL,
        TOKEN_QUERY, TokenIntegrityLevel,
    },
    System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{
        GUITHREADINFO, GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, IsWindow,
    },
};

/// Windows foreground-target evidence provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsTarget;

impl WindowsTarget {
    pub const fn new() -> Self {
        Self
    }
}

impl TargetPort for WindowsTarget {
    fn capture(&self) -> Result<TargetEvidence, TargetCaptureError> {
        capture_target()
    }

    fn compare(&self, expected: &TargetEvidence, observed: &TargetEvidence) -> TargetComparison {
        let (Some(expected_token), Some(observed_token)) = (
            expected.token::<WindowsTargetToken>(),
            observed.token::<WindowsTargetToken>(),
        ) else {
            return TargetComparison::UnavailableOrAmbiguous;
        };

        if !is_window_alive(expected_token.top_level) {
            return TargetComparison::Disappeared;
        }

        compare_tokens(
            expected_token,
            expected.strength(),
            observed_token,
            observed.strength(),
        )
    }

    fn integrity_relation(&self, target: &TargetEvidence) -> IntegrityRelation {
        let Some(token) = target.token::<WindowsTargetToken>() else {
            return IntegrityRelation::Unknown;
        };

        let current_level = query_current_process_integrity();
        let target_level = query_process_integrity(token.process_id);
        match (current_level, target_level) {
            (Some(current), Some(target)) if target > current => IntegrityRelation::KnownRestricted,
            (Some(_), Some(_)) => IntegrityRelation::KnownNotRestricted,
            _ => IntegrityRelation::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct WindowsTargetToken {
    top_level: usize,
    process_id: u32,
    thread_id: u32,
    active: usize,
    focus: usize,
    capture: usize,
    menu_owner: usize,
    move_size: usize,
    caret: usize,
}

fn capture_target() -> Result<TargetEvidence, TargetCaptureError> {
    // SAFETY: this call has no pointer or ownership preconditions.
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_null() {
        return Err(TargetCaptureError::Unavailable);
    }

    let mut process_id: u32 = 0;
    // SAFETY: `foreground` is an observed window handle and `process_id` is a
    // valid out pointer for the duration of the call.
    let thread_id = unsafe { GetWindowThreadProcessId(foreground, &raw mut process_id) };
    if thread_id == 0 || process_id == 0 {
        return Err(TargetCaptureError::Native(last_native_error(
            NativeErrorKind::TemporarilyUnavailable,
        )));
    }
    if !is_window_alive(foreground as usize) {
        return Err(TargetCaptureError::Disappeared);
    }

    // SAFETY: zero is a valid initial state for `GUITHREADINFO`; Win32 requires
    // only `cbSize` to be initialized before the query.
    let mut info: GUITHREADINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<GUITHREADINFO>() as u32;
    // SAFETY: `thread_id` owns the observed foreground window and `info` is a
    // correctly sized writable structure.
    let detailed = unsafe { GetGUIThreadInfo(thread_id, &raw mut info) } != 0;

    let token = if detailed {
        WindowsTargetToken {
            top_level: foreground as usize,
            process_id,
            thread_id,
            active: info.hwndActive as usize,
            focus: info.hwndFocus as usize,
            capture: info.hwndCapture as usize,
            menu_owner: info.hwndMenuOwner as usize,
            move_size: info.hwndMoveSize as usize,
            caret: info.hwndCaret as usize,
        }
    } else {
        WindowsTargetToken {
            top_level: foreground as usize,
            process_id,
            thread_id,
            active: 0,
            focus: 0,
            capture: 0,
            menu_owner: 0,
            move_size: 0,
            caret: 0,
        }
    };

    let strength = if !detailed {
        EvidenceStrength::Degraded
    } else if info.hwndFocus.is_null() {
        EvidenceStrength::TopLevelTarget
    } else {
        EvidenceStrength::NativeFocusedControl
    };

    Ok(TargetEvidence::new(
        token,
        TargetMetadata {
            process_id: Some(process_id),
            gui_thread_id: Some(thread_id),
        },
        strength,
    ))
}

fn compare_tokens(
    expected: &WindowsTargetToken,
    expected_strength: EvidenceStrength,
    observed: &WindowsTargetToken,
    observed_strength: EvidenceStrength,
) -> TargetComparison {
    if expected.top_level != observed.top_level
        || expected.process_id != observed.process_id
        || expected.thread_id != observed.thread_id
    {
        return TargetComparison::Changed;
    }

    if has_native_focus_detail(expected_strength) && has_native_focus_detail(observed_strength) {
        if expected == observed {
            TargetComparison::Same
        } else {
            TargetComparison::Changed
        }
    } else {
        TargetComparison::Same
    }
}

const fn has_native_focus_detail(strength: EvidenceStrength) -> bool {
    matches!(
        strength,
        EvidenceStrength::NativeFocusedControl | EvidenceStrength::RenderHostLimited
    )
}

fn is_window_alive(value: usize) -> bool {
    let window = value as HWND;
    // SAFETY: `IsWindow` validates an opaque handle value and does not transfer
    // ownership. False is the conservative result for stale/invalid handles.
    unsafe { IsWindow(window) != 0 }
}

fn query_current_process_integrity() -> Option<u32> {
    // SAFETY: returns a process pseudo-handle owned by Windows and not closed by
    // the caller.
    let process = unsafe { GetCurrentProcess() };
    query_token_integrity(process)
}

fn query_process_integrity(process_id: u32) -> Option<u32> {
    if process_id == std::process::id() {
        return query_current_process_integrity();
    }

    // SAFETY: the access mask requests query-only rights, inheritance is false,
    // and the identifier came from the foreground window owner.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    let process = OwnedHandle::new(process)?;
    query_token_integrity(process.get())
}

fn query_token_integrity(process: HANDLE) -> Option<u32> {
    let mut token: HANDLE = null_mut();
    // SAFETY: `process` is a live process handle/pseudo-handle and `token` is a
    // valid out pointer. The resulting token is closed by `OwnedHandle`.
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &raw mut token) } == 0 {
        return None;
    }
    let token = OwnedHandle::new(token)?;

    let mut required: u32 = 0;
    // SAFETY: the null-buffer query is the documented size-discovery call.
    let _ = unsafe {
        GetTokenInformation(
            token.get(),
            TokenIntegrityLevel,
            null_mut(),
            0,
            &raw mut required,
        )
    };
    if required < size_of::<TOKEN_MANDATORY_LABEL>() as u32 {
        return None;
    }

    let byte_count = required as usize;
    let word_count = byte_count.div_ceil(size_of::<usize>());
    let mut storage = vec![0usize; word_count];
    // SAFETY: `storage` is aligned for pointer-containing token structures,
    // writable for at least `required` bytes, and remains alive for all reads.
    if unsafe {
        GetTokenInformation(
            token.get(),
            TokenIntegrityLevel,
            storage.as_mut_ptr().cast(),
            required,
            &raw mut required,
        )
    } == 0
    {
        return None;
    }

    // SAFETY: successful `GetTokenInformation(TokenIntegrityLevel)` initialized
    // at least one `TOKEN_MANDATORY_LABEL` in aligned storage.
    let label = unsafe { &*storage.as_ptr().cast::<TOKEN_MANDATORY_LABEL>() };
    let sid = label.Label.Sid;
    if sid.is_null() {
        return None;
    }

    // SAFETY: `sid` is owned by the token-information buffer and remains valid
    // while `storage` is alive.
    let count = unsafe { GetSidSubAuthorityCount(sid) };
    if count.is_null() {
        return None;
    }
    // SAFETY: the pointer was validated and addresses the SID subauthority
    // count byte.
    let count = u32::from(unsafe { *count });
    if count == 0 {
        return None;
    }

    // SAFETY: `count - 1` is within the SID's validated subauthority range.
    let rid = unsafe { GetSidSubAuthority(sid, count - 1) };
    if rid.is_null() {
        None
    } else {
        // SAFETY: `rid` points into `storage`, which is still alive.
        Some(unsafe { *rid })
    }
}

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn new(handle: HANDLE) -> Option<Self> {
        (!handle.is_null()).then_some(Self(handle))
    }

    const fn get(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the wrapper owns a non-null, closable process or token handle;
        // pseudo-handles are never wrapped.
        let _ = unsafe { CloseHandle(self.0) };
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
    use super::{WindowsTargetToken, compare_tokens};
    use cliptype_core::EvidenceStrength;
    use cliptype_platform::{TargetComparison, TargetEvidence, TargetMetadata};

    fn token(top_level: usize, focus: usize) -> WindowsTargetToken {
        WindowsTargetToken {
            top_level,
            process_id: 41,
            thread_id: 42,
            active: top_level,
            focus,
            capture: 0,
            menu_owner: 0,
            move_size: 0,
            caret: focus,
        }
    }

    #[test]
    fn native_focus_tokens_detect_control_changes() {
        let original = token(10, 11);
        assert_eq!(
            compare_tokens(
                &original,
                EvidenceStrength::NativeFocusedControl,
                &original,
                EvidenceStrength::NativeFocusedControl,
            ),
            TargetComparison::Same
        );
        assert_eq!(
            compare_tokens(
                &original,
                EvidenceStrength::NativeFocusedControl,
                &token(10, 12),
                EvidenceStrength::NativeFocusedControl,
            ),
            TargetComparison::Changed
        );
        assert_eq!(
            compare_tokens(
                &original,
                EvidenceStrength::NativeFocusedControl,
                &token(20, 11),
                EvidenceStrength::NativeFocusedControl,
            ),
            TargetComparison::Changed
        );
    }

    #[test]
    fn degraded_tokens_compare_only_stable_top_level_identity() {
        let original = token(10, 11);
        assert_eq!(
            compare_tokens(
                &original,
                EvidenceStrength::Degraded,
                &token(10, 99),
                EvidenceStrength::NativeFocusedControl,
            ),
            TargetComparison::Same
        );
    }

    #[test]
    fn public_debug_output_redacts_private_window_handles() {
        let private = token(0xDEAD_BEEF, 0xA11C_E001);
        let evidence = TargetEvidence::new(
            private,
            TargetMetadata {
                process_id: Some(41),
                gui_thread_id: Some(42),
            },
            EvidenceStrength::NativeFocusedControl,
        );
        let debug = format!("{evidence:?}");

        assert!(!debug.contains("3735928559"));
        assert!(!debug.contains("2703024129"));
        assert!(debug.contains("process_id"));
    }
}
