use std::{
    ffi::c_void,
    ptr::{NonNull, null_mut},
    slice,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
};

use cliptype_core::{
    ByteCount, CapabilityState, EvidenceStrength, HotkeyApplyResult, HotkeyAvailability, HotkeyKey,
    HotkeyPair, HotkeyPlatform, HotkeySpec, InjectionMode, IntegrityRelation, NativeByteLimit,
    NativeEventCount, SensitiveText, TextAtom, TextBatch,
};
use cliptype_platform::{
    AccessibilityPermissionPort, AccessibilityPermissionState, ClipboardError, ClipboardPort,
    ClipboardRevision, DispatchResult, KeyboardCapabilities, KeyboardError, KeyboardPort,
    ModifierMask, ModifierObservation, ModifierPort, NativeDispatchCount, NativeError,
    NativeErrorKind, PasteCapabilities, PasteError, PastePort, PermissionActionResult,
    PermissionError, PermissionErrorKind, TargetCaptureError, TargetComparison, TargetEvidence,
    TargetMetadata, TargetPort,
};

const CLIPBOARD_OK: i32 = 0;
const CLIPBOARD_EMPTY: i32 = 1;
const CLIPBOARD_NON_TEXT: i32 = 2;
const CLIPBOARD_MALFORMED: i32 = 3;
const CLIPBOARD_CHANGED: i32 = 4;

const HOTKEY_OK: i32 = 0;
const HOTKEY_CONFLICT: i32 = 1;
const HOTKEY_UNSUPPORTED: i32 = 2;

const FLAG_SHIFT: u64 = 1 << 17;
const FLAG_CONTROL: u64 = 1 << 18;
const FLAG_OPTION: u64 = 1 << 19;
const FLAG_COMMAND: u64 = 1 << 20;

unsafe extern "C" {
    fn ct_macos_initialize_application();
    fn ct_macos_clipboard_change_count() -> i64;
    fn ct_macos_clipboard_copy_utf8(
        out_bytes: *mut *mut u8,
        out_length: *mut usize,
        out_revision: *mut i64,
    ) -> i32;
    fn ct_macos_free(pointer: *mut c_void);
    fn ct_macos_accessibility_trusted() -> i32;
    fn ct_macos_request_accessibility();
    fn ct_macos_open_accessibility_settings() -> i32;
    fn ct_macos_capture_target(
        out_process_id: *mut i32,
        out_window_hash: *mut u64,
        out_window_available: *mut i32,
        out_focus_hash: *mut u64,
        out_focus_available: *mut i32,
        out_render_host_limited: *mut i32,
    ) -> i32;
    fn ct_macos_modifier_flags() -> u64;
    fn ct_macos_secure_input_enabled() -> i32;
    fn ct_macos_post_unicode(units: *const u16, length: usize) -> i32;
    fn ct_macos_post_return() -> i32;
    fn ct_macos_post_tab() -> i32;
    fn ct_macos_post_backspace() -> i32;
    fn ct_macos_post_cursor_right() -> i32;
    fn ct_macos_post_cursor_right_to_line_end() -> i32;
    fn ct_macos_post_paste(expected_revision: i64) -> i32;

    fn ct_macos_hotkey_create(
        callback: extern "C" fn(i32, *mut c_void),
        context: *mut c_void,
    ) -> *mut c_void;
    fn ct_macos_hotkey_register_initial(
        controller: *mut c_void,
        trigger_code: u16,
        trigger_modifiers: u8,
        cancel_code: u16,
        cancel_modifiers: u8,
    ) -> i32;
    fn ct_macos_hotkey_probe_pair(
        controller: *mut c_void,
        trigger_code: u16,
        trigger_modifiers: u8,
        cancel_code: u16,
        cancel_modifiers: u8,
    ) -> i32;
    fn ct_macos_hotkey_replace_pair(
        controller: *mut c_void,
        trigger_code: u16,
        trigger_modifiers: u8,
        cancel_code: u16,
        cancel_modifiers: u8,
    ) -> i32;
    fn ct_macos_hotkey_destroy(controller: *mut c_void);

    fn ct_macos_status_create(
        callback: extern "C" fn(i32, *mut c_void),
        context: *mut c_void,
    ) -> *mut c_void;
    fn ct_macos_status_update(
        controller: *mut c_void,
        enabled: i32,
        mode: i32,
        permission: i32,
        startup: i32,
    );
    fn ct_macos_status_destroy(controller: *mut c_void);

    fn ct_macos_startup_status() -> i32;
    fn ct_macos_set_startup(enabled: i32) -> i32;
}

pub fn initialize_application() {
    // SAFETY: initializes the process-wide AppKit singleton on the main thread.
    unsafe { ct_macos_initialize_application() };
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacClipboard;

impl ClipboardPort for MacClipboard {
    fn read_current_text(
        &self,
        hard_limit: NativeByteLimit,
    ) -> Result<SensitiveText, ClipboardError> {
        let mut pointer = null_mut();
        let mut length = 0_usize;
        let mut revision = -1_i64;
        // SAFETY: all output pointers are valid for the duration of the call.
        let status = unsafe {
            ct_macos_clipboard_copy_utf8(&raw mut pointer, &raw mut length, &raw mut revision)
        };
        match status {
            CLIPBOARD_EMPTY => return Err(ClipboardError::Empty),
            CLIPBOARD_NON_TEXT => return Err(ClipboardError::NonText),
            CLIPBOARD_MALFORMED => return Err(ClipboardError::Malformed),
            CLIPBOARD_CHANGED => return Err(ClipboardError::ChangedDuringRead),
            CLIPBOARD_OK => {}
            _ => {
                return Err(ClipboardError::Native(NativeError::new(
                    NativeErrorKind::ResourceExhausted,
                    None,
                )));
            }
        }

        let Some(pointer) = NonNull::new(pointer) else {
            return Err(ClipboardError::Malformed);
        };
        let measured = ByteCount::new(length);
        if !hard_limit.allows(measured) {
            // SAFETY: the native helper allocated the buffer with malloc.
            unsafe { ct_macos_free(pointer.as_ptr().cast()) };
            return Err(ClipboardError::TooLarge {
                observed: Some(measured),
                limit: hard_limit,
            });
        }

        // SAFETY: the helper returns an allocation of exactly `length` bytes.
        let bytes = unsafe { slice::from_raw_parts(pointer.as_ptr(), length) };
        let decoded = std::str::from_utf8(bytes).map(str::to_owned);
        // SAFETY: copy is complete; release the helper-owned buffer exactly once.
        unsafe { ct_macos_free(pointer.as_ptr().cast()) };
        decoded
            .map(SensitiveText::new)
            .map_err(|_| ClipboardError::Malformed)
    }

    fn current_revision(&self) -> ClipboardRevision {
        // SAFETY: returns content-blind pasteboard sequence metadata.
        let revision = unsafe { ct_macos_clipboard_change_count() };
        u64::try_from(revision)
            .map(ClipboardRevision::Known)
            .unwrap_or(ClipboardRevision::Unavailable)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacModifiers;

impl ModifierPort for MacModifiers {
    fn observe_modifiers(&self) -> ModifierObservation {
        // SAFETY: read-only combined session modifier state.
        let flags = unsafe { ct_macos_modifier_flags() };
        let mut mask = ModifierMask::NONE;
        if flags & FLAG_SHIFT != 0 {
            mask = mask | ModifierMask::SHIFT;
        }
        if flags & FLAG_CONTROL != 0 {
            mask = mask | ModifierMask::CONTROL;
        }
        if flags & FLAG_OPTION != 0 {
            mask = mask | ModifierMask::ALT;
        }
        if flags & FLAG_COMMAND != 0 {
            mask = mask | ModifierMask::WINDOWS;
        }
        if mask.is_empty() {
            ModifierObservation::Clear
        } else {
            ModifierObservation::Held(mask)
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacKeyboard;

impl MacKeyboard {
    fn ready() -> bool {
        // SAFETY: both functions are content-free permission/input-state queries.
        unsafe { ct_macos_accessibility_trusted() != 0 && ct_macos_secure_input_enabled() == 0 }
    }
}

impl KeyboardPort for MacKeyboard {
    fn capabilities(&self) -> KeyboardCapabilities {
        let state = if Self::ready() {
            CapabilityState::Available
        } else {
            CapabilityState::Unavailable
        };
        KeyboardCapabilities {
            unicode_text: state,
            line_break: state,
            tab: state,
            cursor_right: state,
            modifier_observation: CapabilityState::Available,
        }
    }

    fn dispatch(&self, batch: TextBatch<'_>) -> Result<DispatchResult, KeyboardError> {
        if batch.is_empty() {
            return Err(KeyboardError::InvalidBatch);
        }
        let requested_events = batch
            .len()
            .checked_mul(2)
            .and_then(|value| NativeEventCount::try_from(value).ok())
            .ok_or(KeyboardError::InvalidBatch)?;
        let mut accepted_atoms = 0_usize;

        for atom in batch.atoms() {
            let posted = match *atom {
                TextAtom::Scalar(value) => {
                    let mut units = [0_u16; 2];
                    let encoded = value.encode_utf16(&mut units);
                    // SAFETY: `encoded` points into `units`, valid for the call.
                    unsafe { ct_macos_post_unicode(encoded.as_ptr(), encoded.len()) }
                }
                // SAFETY: fixed balanced key events with no borrowed inputs.
                TextAtom::LineBreak => unsafe { ct_macos_post_return() },
                // SAFETY: fixed balanced key events with no borrowed inputs.
                TextAtom::Tab => unsafe { ct_macos_post_tab() },
            };
            if posted == 0 {
                let accepted = NativeEventCount::try_from(accepted_atoms.saturating_mul(2))
                    .unwrap_or(NativeEventCount::new(0));
                if accepted_atoms == 0 {
                    return Ok(DispatchResult::NoneAccepted {
                        requested: requested_events,
                        native: Some(NativeError::new(
                            if Self::ready() {
                                NativeErrorKind::BlockedCauseUnknown
                            } else {
                                NativeErrorKind::PermissionDenied
                            },
                            None,
                        )),
                    });
                }
                return Ok(DispatchResult::ProgressUnknown {
                    counts: NativeDispatchCount {
                        requested: requested_events,
                        accepted,
                    },
                });
            }
            accepted_atoms = accepted_atoms.saturating_add(1);
        }

        Ok(DispatchResult::Complete {
            events: requested_events,
        })
    }

    fn dispatch_backspace(&self) -> Result<DispatchResult, KeyboardError> {
        // SAFETY: fixed balanced Backspace key event.
        if unsafe { ct_macos_post_backspace() } == 0 {
            Ok(DispatchResult::NoneAccepted {
                requested: NativeEventCount::new(2),
                native: Some(NativeError::new(
                    if Self::ready() {
                        NativeErrorKind::BlockedCauseUnknown
                    } else {
                        NativeErrorKind::PermissionDenied
                    },
                    None,
                )),
            })
        } else {
            Ok(DispatchResult::Complete {
                events: NativeEventCount::new(2),
            })
        }
    }

    fn dispatch_cursor_right(&self) -> Result<DispatchResult, KeyboardError> {
        // SAFETY: fixed balanced cursor-right key event.
        if unsafe { ct_macos_post_cursor_right() } == 0 {
            Ok(DispatchResult::NoneAccepted {
                requested: NativeEventCount::new(2),
                native: Some(NativeError::new(
                    if Self::ready() {
                        NativeErrorKind::BlockedCauseUnknown
                    } else {
                        NativeErrorKind::PermissionDenied
                    },
                    None,
                )),
            })
        } else {
            Ok(DispatchResult::Complete {
                events: NativeEventCount::new(2),
            })
        }
    }

    fn dispatch_cursor_right_to_line_end(&self) -> Result<DispatchResult, KeyboardError> {
        // SAFETY: fixed bounded sequence of Right and Command+Right key events.
        if unsafe { ct_macos_post_cursor_right_to_line_end() } == 0 {
            Ok(DispatchResult::NoneAccepted {
                requested: NativeEventCount::new(4),
                native: Some(NativeError::new(
                    if Self::ready() {
                        NativeErrorKind::BlockedCauseUnknown
                    } else {
                        NativeErrorKind::PermissionDenied
                    },
                    None,
                )),
            })
        } else {
            Ok(DispatchResult::Complete {
                events: NativeEventCount::new(4),
            })
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacPaste;

impl PastePort for MacPaste {
    fn capabilities(&self) -> PasteCapabilities {
        let paste_chord = if MacKeyboard::ready() {
            CapabilityState::Available
        } else {
            CapabilityState::Unavailable
        };
        PasteCapabilities {
            paste_chord,
            clipboard_revision_guard: CapabilityState::Available,
        }
    }

    fn dispatch_paste(
        &self,
        expected_revision: ClipboardRevision,
    ) -> Result<DispatchResult, PasteError> {
        let ClipboardRevision::Known(revision) = expected_revision else {
            return Err(PasteError::InvalidRequest);
        };
        let revision = i64::try_from(revision).map_err(|_| PasteError::InvalidRequest)?;
        // SAFETY: helper performs a content-blind revision comparison and posts
        // one balanced Command+V chord.
        match unsafe { ct_macos_post_paste(revision) } {
            -1 => Err(PasteError::ClipboardChanged),
            1 => Ok(DispatchResult::Complete {
                events: NativeEventCount::new(2),
            }),
            _ => Ok(DispatchResult::NoneAccepted {
                requested: NativeEventCount::new(2),
                native: Some(NativeError::new(
                    if MacKeyboard::ready() {
                        NativeErrorKind::BlockedCauseUnknown
                    } else {
                        NativeErrorKind::PermissionDenied
                    },
                    None,
                )),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MacTargetToken {
    process_id: u32,
    window_hash: Option<u64>,
    focus_hash: Option<u64>,
    render_host_limited: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacTarget;

impl TargetPort for MacTarget {
    fn capture(&self) -> Result<TargetEvidence, TargetCaptureError> {
        let mut process_id = 0_i32;
        let mut window_hash = 0_u64;
        let mut window_available = 0_i32;
        let mut focus_hash = 0_u64;
        let mut focus_available = 0_i32;
        let mut render_host_limited = 0_i32;
        // SAFETY: all output pointers are valid and no UI content is requested.
        let captured = unsafe {
            ct_macos_capture_target(
                &raw mut process_id,
                &raw mut window_hash,
                &raw mut window_available,
                &raw mut focus_hash,
                &raw mut focus_available,
                &raw mut render_host_limited,
            )
        };
        if captured == 0 || process_id <= 0 {
            return Err(TargetCaptureError::Unavailable);
        }
        let process_id = u32::try_from(process_id).map_err(|_| TargetCaptureError::Unavailable)?;
        let window_hash = (window_available != 0).then_some(window_hash);
        let focus_hash = (focus_available != 0).then_some(focus_hash);
        let render_host_limited = render_host_limited != 0 && window_hash.is_some();
        Ok(TargetEvidence::new(
            MacTargetToken {
                process_id,
                window_hash,
                focus_hash,
                render_host_limited,
            },
            TargetMetadata {
                process_id: Some(process_id),
                gui_thread_id: None,
            },
            if render_host_limited {
                EvidenceStrength::RenderHostLimited
            } else if focus_hash.is_some() {
                EvidenceStrength::NativeFocusedControl
            } else {
                EvidenceStrength::TopLevelTarget
            },
        ))
    }

    fn compare(&self, expected: &TargetEvidence, observed: &TargetEvidence) -> TargetComparison {
        let (Some(expected), Some(observed)) = (
            expected.token::<MacTargetToken>(),
            observed.token::<MacTargetToken>(),
        ) else {
            return TargetComparison::UnavailableOrAmbiguous;
        };
        if expected.process_id != observed.process_id {
            return TargetComparison::Changed;
        }

        match (expected.window_hash, observed.window_hash) {
            (Some(left), Some(right)) if left != right => return TargetComparison::Changed,
            (Some(_), Some(_)) => {}
            (None, None) => {}
            _ => return TargetComparison::UnavailableOrAmbiguous,
        }

        if expected.render_host_limited {
            // The initial render-host classification selects a process/window
            // comparison policy for the whole session. Monaco may rebuild a
            // focused AX node that temporarily lacks the classification, but a
            // real process/window change was already rejected above.
            return if expected.window_hash.is_some() {
                TargetComparison::Same
            } else {
                TargetComparison::UnavailableOrAmbiguous
            };
        }
        if observed.render_host_limited {
            // A native-control session must not weaken its exact-focus promise
            // merely because a later node looks like a render host.
            return TargetComparison::UnavailableOrAmbiguous;
        }

        match (expected.focus_hash, observed.focus_hash) {
            (Some(left), Some(right)) if left == right => TargetComparison::Same,
            (Some(_), Some(_)) => TargetComparison::Changed,
            (None, None) => TargetComparison::Same,
            _ => TargetComparison::UnavailableOrAmbiguous,
        }
    }

    fn integrity_relation(&self, _target: &TargetEvidence) -> IntegrityRelation {
        IntegrityRelation::KnownNotRestricted
    }
}

#[derive(Debug, Default)]
pub struct MacAccessibility {
    seen_granted: AtomicBool,
}

impl AccessibilityPermissionPort for MacAccessibility {
    fn state(&self) -> AccessibilityPermissionState {
        // SAFETY: content-free trust-state query.
        if unsafe { ct_macos_accessibility_trusted() } != 0 {
            self.seen_granted.store(true, Ordering::Release);
            AccessibilityPermissionState::Granted
        } else if self.seen_granted.load(Ordering::Acquire) {
            AccessibilityPermissionState::Revoked
        } else {
            AccessibilityPermissionState::NotGranted
        }
    }

    fn request(&self) -> Result<PermissionActionResult, PermissionError> {
        if self.state() == AccessibilityPermissionState::Granted {
            return Ok(PermissionActionResult::AlreadyGranted);
        }
        // SAFETY: invokes only Apple's approved asynchronous trust prompt.
        unsafe { ct_macos_request_accessibility() };
        Ok(PermissionActionResult::PromptRequested)
    }

    fn open_system_settings(&self) -> Result<PermissionActionResult, PermissionError> {
        // SAFETY: opens the system-owned Accessibility settings URL.
        if unsafe { ct_macos_open_accessibility_settings() } != 0 {
            Ok(PermissionActionResult::SettingsOpened)
        } else {
            Err(PermissionError::new(
                PermissionErrorKind::NativeFailure,
                Some(NativeError::new(NativeErrorKind::Unknown, None)),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacStartupStatus {
    NotRegistered,
    Enabled,
    RequiresApproval,
    NotFound,
    Unsupported,
    Unknown,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MacStartup;

impl MacStartup {
    pub fn status(self) -> MacStartupStatus {
        // SAFETY: reads the current app-owned SMAppService state.
        match unsafe { ct_macos_startup_status() } {
            0 => MacStartupStatus::NotRegistered,
            1 => MacStartupStatus::Enabled,
            2 => MacStartupStatus::RequiresApproval,
            3 => MacStartupStatus::NotFound,
            4 => MacStartupStatus::Unsupported,
            _ => MacStartupStatus::Unknown,
        }
    }

    pub fn set_enabled(self, enabled: bool) -> Result<MacStartupStatus, NativeError> {
        // SAFETY: changes only `SMAppService.mainApp` for this application.
        if unsafe { ct_macos_set_startup(if enabled { 1 } else { 0 }) } == 0 {
            return Err(NativeError::new(NativeErrorKind::Unknown, None));
        }
        Ok(self.status())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacMenuEvent {
    Trigger,
    Cancel,
    OpenSettings,
    ToggleEnabled,
    ToggleStartup,
    Permission,
    About,
    Quit,
}

fn command_from_native(command: i32) -> Option<MacMenuEvent> {
    match command {
        1 => Some(MacMenuEvent::Trigger),
        2 => Some(MacMenuEvent::Cancel),
        3 => Some(MacMenuEvent::OpenSettings),
        4 => Some(MacMenuEvent::ToggleEnabled),
        5 => Some(MacMenuEvent::ToggleStartup),
        6 => Some(MacMenuEvent::Permission),
        7 => Some(MacMenuEvent::About),
        8 => Some(MacMenuEvent::Quit),
        _ => None,
    }
}

extern "C" fn menu_callback(command: i32, context: *mut c_void) {
    if context.is_null() {
        return;
    }
    // SAFETY: `context` points to a boxed sender retained by the Rust guard.
    let sender = unsafe { &*(context.cast::<Sender<MacMenuEvent>>()) };
    if let Some(event) = command_from_native(command) {
        let _ = sender.send(event);
    }
}

pub struct MacStatusItem {
    handle: NonNull<c_void>,
    callback: NonNull<Sender<MacMenuEvent>>,
}

impl MacStatusItem {
    pub fn new(events: Sender<MacMenuEvent>) -> Result<Self, NativeError> {
        let callback = NonNull::new(Box::into_raw(Box::new(events)))
            .ok_or_else(|| NativeError::new(NativeErrorKind::ResourceExhausted, None))?;
        // SAFETY: callback context remains allocated until after native teardown.
        let handle = unsafe { ct_macos_status_create(menu_callback, callback.as_ptr().cast()) };
        let Some(handle) = NonNull::new(handle) else {
            // SAFETY: native creation failed and never retained the callback.
            drop(unsafe { Box::from_raw(callback.as_ptr()) });
            return Err(NativeError::new(NativeErrorKind::Unknown, None));
        };
        Ok(Self { handle, callback })
    }

    pub fn update(
        &self,
        enabled: bool,
        mode: InjectionMode,
        permission: AccessibilityPermissionState,
        startup: bool,
    ) {
        let mode = match mode {
            InjectionMode::Keyboard => 0,
            InjectionMode::Clipboard => 1,
            InjectionMode::Auto => 2,
            InjectionMode::Code => 3,
        };
        let permission = match permission {
            AccessibilityPermissionState::NotRequired => 0,
            AccessibilityPermissionState::NotRequested => 1,
            AccessibilityPermissionState::NotGranted => 2,
            AccessibilityPermissionState::Granted => 3,
            AccessibilityPermissionState::Revoked => 4,
            AccessibilityPermissionState::Unknown => 5,
        };
        // SAFETY: handle is owned and live for this guard.
        unsafe {
            ct_macos_status_update(
                self.handle.as_ptr(),
                if enabled { 1 } else { 0 },
                mode,
                permission,
                if startup { 1 } else { 0 },
            )
        };
    }
}

impl Drop for MacStatusItem {
    fn drop(&mut self) {
        // SAFETY: native object is destroyed before its callback context.
        unsafe { ct_macos_status_destroy(self.handle.as_ptr()) };
        // SAFETY: the pointer originated from `Box::into_raw` in `new`.
        drop(unsafe { Box::from_raw(self.callback.as_ptr()) });
    }
}

pub struct MacHotkeyController {
    handle: NonNull<c_void>,
    callback: NonNull<Sender<MacMenuEvent>>,
    current: HotkeyPair,
}

impl MacHotkeyController {
    pub fn new(pair: HotkeyPair, events: Sender<MacMenuEvent>) -> Result<Self, HotkeyAvailability> {
        let pair = pair
            .validate_for(HotkeyPlatform::MacOS)
            .map_err(|error| match error {
                cliptype_core::HotkeyValidationError::Reserved => HotkeyAvailability::Reserved,
                _ => HotkeyAvailability::Unsupported,
            })?;
        let callback =
            NonNull::new(Box::into_raw(Box::new(events))).ok_or(HotkeyAvailability::Unknown)?;
        // SAFETY: callback context remains allocated until after native teardown.
        let handle = unsafe { ct_macos_hotkey_create(menu_callback, callback.as_ptr().cast()) };
        let Some(handle) = NonNull::new(handle) else {
            // SAFETY: native creation failed and never retained the callback.
            drop(unsafe { Box::from_raw(callback.as_ptr()) });
            return Err(HotkeyAvailability::Unknown);
        };
        let mut controller = Self {
            handle,
            callback,
            current: pair,
        };
        let (trigger_code, trigger_modifiers) = native_hotkey(pair.trigger)?;
        let (cancel_code, cancel_modifiers) = native_hotkey(pair.cancel)?;
        // SAFETY: controller is live and registrations are performed on the main thread.
        let status = unsafe {
            ct_macos_hotkey_register_initial(
                controller.handle.as_ptr(),
                trigger_code,
                trigger_modifiers,
                cancel_code,
                cancel_modifiers,
            )
        };
        if status != HOTKEY_OK {
            let result = availability_from_status(status);
            // Ensure native state is released before returning.
            controller.current = HotkeyPair::default();
            return Err(result);
        }
        Ok(controller)
    }

    pub const fn current_pair(&self) -> HotkeyPair {
        self.current
    }

    pub fn probe_pair(&self, candidate: HotkeyPair) -> HotkeyAvailability {
        let candidate = match candidate.validate_for(HotkeyPlatform::MacOS) {
            Ok(candidate) => candidate,
            Err(cliptype_core::HotkeyValidationError::Reserved) => {
                return HotkeyAvailability::Reserved;
            }
            Err(_) => return HotkeyAvailability::Unsupported,
        };
        let Ok((trigger_code, trigger_modifiers)) = native_hotkey(candidate.trigger) else {
            return HotkeyAvailability::Unsupported;
        };
        let Ok((cancel_code, cancel_modifiers)) = native_hotkey(candidate.cancel) else {
            return HotkeyAvailability::Unsupported;
        };
        // SAFETY: controller is live and called on the owning main thread.
        availability_from_status(unsafe {
            ct_macos_hotkey_probe_pair(
                self.handle.as_ptr(),
                trigger_code,
                trigger_modifiers,
                cancel_code,
                cancel_modifiers,
            )
        })
    }

    pub fn replace_pair(&mut self, candidate: HotkeyPair) -> HotkeyApplyResult {
        let availability = self.probe_pair(candidate);
        if availability != HotkeyAvailability::Available {
            return HotkeyApplyResult::Rejected(availability);
        }
        let (trigger_code, trigger_modifiers) = match native_hotkey(candidate.trigger) {
            Ok(value) => value,
            Err(error) => return HotkeyApplyResult::Rejected(error),
        };
        let (cancel_code, cancel_modifiers) = match native_hotkey(candidate.cancel) {
            Ok(value) => value,
            Err(error) => return HotkeyApplyResult::Rejected(error),
        };
        // SAFETY: controller is live and called on the owning main thread.
        let status = unsafe {
            ct_macos_hotkey_replace_pair(
                self.handle.as_ptr(),
                trigger_code,
                trigger_modifiers,
                cancel_code,
                cancel_modifiers,
            )
        };
        if status == HOTKEY_OK {
            self.current = candidate;
            HotkeyApplyResult::Applied
        } else {
            HotkeyApplyResult::RolledBack(availability_from_status(status))
        }
    }
}

impl Drop for MacHotkeyController {
    fn drop(&mut self) {
        // SAFETY: controller is owned by this guard and destroyed on main thread.
        unsafe { ct_macos_hotkey_destroy(self.handle.as_ptr()) };
        // SAFETY: the pointer originated from `Box::into_raw` in `new`.
        drop(unsafe { Box::from_raw(self.callback.as_ptr()) });
    }
}

fn availability_from_status(status: i32) -> HotkeyAvailability {
    match status {
        HOTKEY_OK => HotkeyAvailability::Available,
        HOTKEY_CONFLICT => HotkeyAvailability::Conflict,
        HOTKEY_UNSUPPORTED => HotkeyAvailability::Unsupported,
        _ => HotkeyAvailability::Unknown,
    }
}

fn native_hotkey(spec: HotkeySpec) -> Result<(u16, u8), HotkeyAvailability> {
    let code = match spec.key {
        HotkeyKey::A => 0,
        HotkeyKey::S => 1,
        HotkeyKey::D => 2,
        HotkeyKey::F => 3,
        HotkeyKey::H => 4,
        HotkeyKey::G => 5,
        HotkeyKey::Z => 6,
        HotkeyKey::X => 7,
        HotkeyKey::C => 8,
        HotkeyKey::V => 9,
        HotkeyKey::B => 11,
        HotkeyKey::Q => 12,
        HotkeyKey::W => 13,
        HotkeyKey::E => 14,
        HotkeyKey::R => 15,
        HotkeyKey::Y => 16,
        HotkeyKey::T => 17,
        HotkeyKey::Digit1 => 18,
        HotkeyKey::Digit2 => 19,
        HotkeyKey::Digit3 => 20,
        HotkeyKey::Digit4 => 21,
        HotkeyKey::Digit6 => 22,
        HotkeyKey::Digit5 => 23,
        HotkeyKey::Equal => 24,
        HotkeyKey::Digit9 => 25,
        HotkeyKey::Digit7 => 26,
        HotkeyKey::Minus => 27,
        HotkeyKey::Digit8 => 28,
        HotkeyKey::Digit0 => 29,
        HotkeyKey::BracketRight => 30,
        HotkeyKey::O => 31,
        HotkeyKey::U => 32,
        HotkeyKey::BracketLeft => 33,
        HotkeyKey::I => 34,
        HotkeyKey::P => 35,
        HotkeyKey::Enter => 36,
        HotkeyKey::L => 37,
        HotkeyKey::J => 38,
        HotkeyKey::Quote => 39,
        HotkeyKey::K => 40,
        HotkeyKey::Semicolon => 41,
        HotkeyKey::Backslash => 42,
        HotkeyKey::Comma => 43,
        HotkeyKey::Slash => 44,
        HotkeyKey::N => 45,
        HotkeyKey::M => 46,
        HotkeyKey::Period => 47,
        HotkeyKey::Tab => 48,
        HotkeyKey::Space => 49,
        HotkeyKey::Backquote => 50,
        HotkeyKey::Backspace => 51,
        HotkeyKey::Escape => 53,
        HotkeyKey::F17 => 64,
        HotkeyKey::F18 => 79,
        HotkeyKey::F19 => 80,
        HotkeyKey::F20 => 90,
        HotkeyKey::F5 => 96,
        HotkeyKey::F6 => 97,
        HotkeyKey::F7 => 98,
        HotkeyKey::F3 => 99,
        HotkeyKey::F8 => 100,
        HotkeyKey::F9 => 101,
        HotkeyKey::F11 => 103,
        HotkeyKey::F13 => 105,
        HotkeyKey::F16 => 106,
        HotkeyKey::F14 => 107,
        HotkeyKey::F10 => 109,
        HotkeyKey::F12 => 111,
        HotkeyKey::F15 => 113,
        HotkeyKey::Insert => 114,
        HotkeyKey::Home => 115,
        HotkeyKey::PageUp => 116,
        HotkeyKey::Delete => 117,
        HotkeyKey::F4 => 118,
        HotkeyKey::End => 119,
        HotkeyKey::F2 => 120,
        HotkeyKey::PageDown => 121,
        HotkeyKey::F1 => 122,
        HotkeyKey::ArrowLeft => 123,
        HotkeyKey::ArrowRight => 124,
        HotkeyKey::ArrowDown => 125,
        HotkeyKey::ArrowUp => 126,
        HotkeyKey::F21 | HotkeyKey::F22 | HotkeyKey::F23 | HotkeyKey::F24 => {
            return Err(HotkeyAvailability::Unsupported);
        }
    };
    Ok((code, spec.modifiers.bits()))
}

#[cfg(test)]
mod tests {
    use cliptype_core::{EvidenceStrength, HotkeyPlatform, HotkeySpec};
    use cliptype_platform::{TargetComparison, TargetEvidence, TargetMetadata, TargetPort};

    use super::{
        HotkeyAvailability, MacTarget, MacTargetToken, availability_from_status, native_hotkey,
    };

    #[test]
    fn mac_keycodes_cover_reviewed_defaults() {
        for value in ["cmd+alt+shift+v", "cmd+alt+shift+x"] {
            let spec: HotkeySpec = value.parse().expect("reviewed shortcut");
            assert!(spec.validate_for(HotkeyPlatform::MacOS).is_ok());
            assert!(native_hotkey(spec).is_ok());
        }
    }

    #[test]
    fn native_hotkey_status_is_content_free() {
        assert_eq!(availability_from_status(0), HotkeyAvailability::Available);
        assert_eq!(availability_from_status(1), HotkeyAvailability::Conflict);
        assert_eq!(availability_from_status(99), HotkeyAvailability::Unknown);
    }

    #[test]
    fn target_token_contains_only_identity_evidence() {
        let token = MacTargetToken {
            process_id: 42,
            window_hash: Some(5),
            focus_hash: Some(7),
            render_host_limited: true,
        };
        assert_eq!(token.process_id, 42);
        assert_eq!(token.window_hash, Some(5));
        assert_eq!(token.focus_hash, Some(7));
        assert!(token.render_host_limited);
    }

    #[test]
    fn render_host_uses_stable_window_identity_when_focus_node_is_rebuilt() {
        let expected = target(42, Some(5), Some(7), true);
        let rebuilt_focus = target(42, Some(5), Some(8), true);
        let transiently_reclassified = target(42, Some(5), Some(9), false);
        let other_window = target(42, Some(6), Some(8), false);
        let other_process = target(43, Some(5), Some(8), false);

        assert_eq!(
            MacTarget.compare(&expected, &rebuilt_focus),
            TargetComparison::Same
        );
        assert_eq!(
            MacTarget.compare(&expected, &transiently_reclassified),
            TargetComparison::Same
        );
        assert_eq!(
            MacTarget.compare(&expected, &other_window),
            TargetComparison::Changed
        );
        assert_eq!(
            MacTarget.compare(&expected, &other_process),
            TargetComparison::Changed
        );
    }

    #[test]
    fn native_control_still_requires_exact_focus_identity() {
        let expected = target(42, Some(5), Some(7), false);
        let other_control = target(42, Some(5), Some(8), false);
        let other_process = target(43, Some(5), Some(7), false);

        assert_eq!(
            MacTarget.compare(&expected, &other_control),
            TargetComparison::Changed
        );
        assert_eq!(
            MacTarget.compare(&expected, &other_process),
            TargetComparison::Changed
        );
    }

    fn target(
        process_id: u32,
        window_hash: Option<u64>,
        focus_hash: Option<u64>,
        render_host_limited: bool,
    ) -> TargetEvidence {
        TargetEvidence::new(
            MacTargetToken {
                process_id,
                window_hash,
                focus_hash,
                render_host_limited,
            },
            TargetMetadata {
                process_id: Some(process_id),
                gui_thread_id: None,
            },
            if render_host_limited {
                EvidenceStrength::RenderHostLimited
            } else {
                EvidenceStrength::NativeFocusedControl
            },
        )
    }
}
