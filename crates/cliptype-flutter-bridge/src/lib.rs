//! Fixed, content-free C ABI for the Flutter macOS shell.
//!
//! The bridge owns the Rust application coordinator and settings store. Swift
//! owns AppKit, the status item, global shortcut registration, permission
//! presentation, and the Flutter channels. Clipboard and target plaintext
//! never cross this boundary.

#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(target_os = "macos")]
use std::slice;
use std::{
    ffi::{c_char, c_void},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
};

const CT_OK: i32 = 0;
const CT_INVALID: i32 = 1;
const CT_NATIVE_FAILURE: i32 = 2;
#[cfg(target_os = "macos")]
const CT_BUSY: i32 = 3;
#[cfg(target_os = "macos")]
const CT_SHUTTING_DOWN: i32 = 4;
#[cfg(target_os = "macos")]
const CT_REJECTED: i32 = 5;

#[cfg(target_os = "macos")]
const CT_HOTKEY_AVAILABLE: i32 = 0;
#[cfg(target_os = "macos")]
const CT_HOTKEY_RESERVED: i32 = 2;
const CT_HOTKEY_UNSUPPORTED: i32 = 3;
#[cfg(target_os = "macos")]
const CT_HOTKEY_UNKNOWN: i32 = 4;

#[cfg(target_os = "macos")]
const MAX_HOTKEY_BYTES: usize = 63;

/// Numeric snapshot exchanged with Swift. All fields are bounded and
/// content-free. Enum values are documented in `cliptype_bridge.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CtBridgeState {
    pub enabled: i32,
    pub notifications: i32,
    pub start_at_login: i32,
    pub mode: i32,
    pub characters_per_second: u16,
    pub jitter_percent: u8,
    pub typo_probability_percent: u8,
    pub auto_clipboard_threshold: u32,
    pub generation: u64,
    pub phase: i32,
    pub backend: i32,
    pub completion: i32,
    pub batches_completed: u32,
}

#[cfg(target_os = "macos")]
mod imp {
    use std::{
        env,
        sync::{Arc, Mutex, MutexGuard, atomic::AtomicBool},
    };

    use cliptype_app::{Coordinator, SettingsStore};
    use cliptype_core::ProductSettings;

    pub struct BridgeRuntime {
        pub coordinator: Arc<Coordinator>,
        pub store: SettingsStore,
        pub settings: Mutex<ProductSettings>,
        pub shutting_down: AtomicBool,
    }

    impl BridgeRuntime {
        pub fn new() -> Result<Self, ()> {
            let home = env::var_os("HOME").ok_or(())?;
            let path = std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("ClipType")
                .join("config.toml");
            let store = SettingsStore::new(path);
            let loaded = store.load().map_err(|_| ())?;
            let settings = loaded.settings.validate().map_err(|_| ())?;
            if loaded.source != cliptype_app::SettingsSource::Primary {
                store.save(settings).map_err(|_| ())?;
            }
            let runtime = settings.runtime_config().map_err(|_| ())?;
            let coordinator = Coordinator::new_product(
                cliptype_macos::MacClipboard,
                cliptype_macos::MacTarget,
                cliptype_macos::MacKeyboard,
                cliptype_macos::MacModifiers,
                cliptype_macos::MacPaste,
                runtime,
            )
            .map_err(|_| ())?;

            Ok(Self {
                coordinator: Arc::new(coordinator),
                store,
                settings: Mutex::new(settings),
                shutting_down: AtomicBool::new(false),
            })
        }

        pub fn settings(&self) -> ProductSettings {
            lock_unpoisoned(&self.settings).to_owned()
        }

        pub fn save(&self, settings: ProductSettings) -> Result<(), ()> {
            let settings = settings.validate().map_err(|_| ())?;
            let old = self.settings();
            let old_runtime = old.runtime_config().map_err(|_| ())?;
            let new_runtime = settings.runtime_config().map_err(|_| ())?;
            self.coordinator
                .update_config(new_runtime)
                .map_err(|_| ())?;
            if self.store.save(settings).is_err() {
                let _ = self.coordinator.update_config(old_runtime);
                return Err(());
            }
            *lock_unpoisoned(&self.settings) = settings;
            Ok(())
        }
    }

    fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        match mutex.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    pub struct BridgeRuntime;
}

use imp::BridgeRuntime;

fn catch_code<T>(fallback: i32, callback: impl FnOnce() -> Result<T, i32>) -> i32 {
    match catch_unwind(AssertUnwindSafe(callback)) {
        Ok(Ok(_)) => CT_OK,
        Ok(Err(code)) => code,
        Err(_) => fallback,
    }
}

#[cfg(target_os = "macos")]
fn hotkey_code(error: cliptype_core::HotkeyValidationError) -> i32 {
    match error {
        cliptype_core::HotkeyValidationError::Reserved => CT_HOTKEY_RESERVED,
        cliptype_core::HotkeyValidationError::Unsupported => CT_HOTKEY_UNSUPPORTED,
        cliptype_core::HotkeyValidationError::MissingPrimaryModifier
        | cliptype_core::HotkeyValidationError::DuplicatePair => CT_INVALID,
    }
}

#[cfg(target_os = "macos")]
fn bounded_utf8<'a>(pointer: *const c_char) -> Result<&'a str, i32> {
    if pointer.is_null() {
        return Err(CT_INVALID);
    }
    // SAFETY: the C ABI requires a valid NUL-terminated string. Reading one
    // byte at a time stops at the terminator and never scans beyond the
    // documented bound.
    for index in 0..=MAX_HOTKEY_BYTES {
        let byte = unsafe { *pointer.cast::<u8>().add(index) };
        if byte == 0 {
            // SAFETY: the preceding bounded scan found the terminator at
            // `index`, so the first `index` bytes belong to this C string.
            let bytes = unsafe { slice::from_raw_parts(pointer.cast::<u8>(), index) };
            return std::str::from_utf8(bytes).map_err(|_| CT_INVALID);
        }
        if index == MAX_HOTKEY_BYTES {
            return Err(CT_INVALID);
        }
    }
    Err(CT_INVALID)
}

#[cfg(target_os = "macos")]
fn parse_hotkeys(
    trigger: *const c_char,
    cancel: *const c_char,
) -> Result<cliptype_core::HotkeyPair, i32> {
    use std::str::FromStr;

    let trigger =
        cliptype_core::HotkeySpec::from_str(bounded_utf8(trigger)?).map_err(
            |error| match error {
                cliptype_core::HotkeyParseError::Validation(validation) => hotkey_code(validation),
                _ => CT_INVALID,
            },
        )?;
    let cancel =
        cliptype_core::HotkeySpec::from_str(bounded_utf8(cancel)?).map_err(
            |error| match error {
                cliptype_core::HotkeyParseError::Validation(validation) => hotkey_code(validation),
                _ => CT_INVALID,
            },
        )?;
    cliptype_core::HotkeyPair::new(trigger, cancel)
        .validate_for(cliptype_core::HotkeyPlatform::MacOS)
        .map_err(hotkey_code)
}

#[cfg(target_os = "macos")]
fn settings_from_args(args: SettingsArgs) -> Result<cliptype_core::ProductSettings, i32> {
    let mode = match args.mode {
        0 => cliptype_core::InjectionMode::Keyboard,
        1 => cliptype_core::InjectionMode::Clipboard,
        2 => cliptype_core::InjectionMode::Auto,
        3 => cliptype_core::InjectionMode::Code,
        _ => return Err(CT_INVALID),
    };
    let threshold = cliptype_core::AutoClipboardThreshold::new(
        usize::try_from(args.auto_clipboard_threshold).map_err(|_| CT_INVALID)?,
    )
    .map_err(|_| CT_INVALID)?;
    let hotkeys = parse_hotkeys(args.trigger, args.cancel)?;
    let settings = cliptype_core::ProductSettings {
        version: cliptype_core::SETTINGS_SCHEMA_VERSION,
        enabled: args.enabled != 0,
        mode,
        auto_clipboard_threshold: threshold,
        speed: cliptype_core::SpeedPreset::Custom,
        characters_per_second: args.characters_per_second,
        jitter_percent: args.jitter_percent,
        typo_probability_percent: args.typo_probability_percent,
        notifications: args.notifications != 0,
        start_at_login: args.start_at_login != 0,
        hotkeys,
    };
    settings.validate().map_err(|error| match error {
        cliptype_core::SettingsValidationError::InvalidHotkeys(error) => hotkey_code(error),
        _ => CT_INVALID,
    })
}

#[cfg(target_os = "macos")]
struct SettingsArgs {
    enabled: i32,
    notifications: i32,
    start_at_login: i32,
    mode: i32,
    characters_per_second: u16,
    jitter_percent: u8,
    typo_probability_percent: u8,
    auto_clipboard_threshold: u32,
    trigger: *const c_char,
    cancel: *const c_char,
}

#[cfg(target_os = "macos")]
fn phase_code(phase: cliptype_core::SessionPhase) -> i32 {
    match phase {
        cliptype_core::SessionPhase::Idle => 0,
        cliptype_core::SessionPhase::Preparing => 1,
        cliptype_core::SessionPhase::Injecting => 2,
        cliptype_core::SessionPhase::Cancelling => 3,
    }
}

#[cfg(target_os = "macos")]
fn backend_code(backend: Option<cliptype_core::InjectionBackend>) -> i32 {
    match backend {
        Some(cliptype_core::InjectionBackend::Keyboard) => 0,
        Some(cliptype_core::InjectionBackend::Clipboard) => 1,
        Some(cliptype_core::InjectionBackend::Code) => 2,
        None => -1,
    }
}

#[cfg(target_os = "macos")]
fn completion_code(completion: Option<cliptype_app::SessionCompletion>) -> i32 {
    match completion {
        None => 0,
        Some(cliptype_app::SessionCompletion::PreparationFailed(
            cliptype_core::PreparationFailure::Cancelled,
        ))
        | Some(cliptype_app::SessionCompletion::Finished(
            cliptype_core::TerminalOutcome::Cancelled,
        )) => 2,
        Some(cliptype_app::SessionCompletion::Finished(
            cliptype_core::TerminalOutcome::Completed,
        )) => 1,
        Some(cliptype_app::SessionCompletion::Finished(
            cliptype_core::TerminalOutcome::TargetChanged
            | cliptype_core::TerminalOutcome::TargetDisappeared
            | cliptype_core::TerminalOutcome::TargetEvidenceUnavailable,
        )) => 3,
        Some(cliptype_app::SessionCompletion::Finished(
            cliptype_core::TerminalOutcome::ClipboardChanged,
        )) => 4,
        Some(cliptype_app::SessionCompletion::PreparationFailed(
            cliptype_core::PreparationFailure::KnownSecurityRestriction,
        ))
        | Some(cliptype_app::SessionCompletion::Finished(
            cliptype_core::TerminalOutcome::KnownSecurityRestriction,
        )) => 5,
        Some(_) => 6,
    }
}

fn fill_state(runtime: &BridgeRuntime, output: &mut CtBridgeState) -> Result<(), i32> {
    #[cfg(target_os = "macos")]
    {
        let settings = runtime.settings();
        let status = runtime.coordinator.status();
        *output = CtBridgeState {
            enabled: i32::from(settings.enabled),
            notifications: i32::from(settings.notifications),
            start_at_login: i32::from(settings.start_at_login),
            mode: match settings.mode {
                cliptype_core::InjectionMode::Keyboard => 0,
                cliptype_core::InjectionMode::Clipboard => 1,
                cliptype_core::InjectionMode::Auto => 2,
                cliptype_core::InjectionMode::Code => 3,
            },
            characters_per_second: settings.characters_per_second,
            jitter_percent: settings.jitter_percent,
            typo_probability_percent: settings.typo_probability_percent,
            auto_clipboard_threshold: u32::try_from(settings.auto_clipboard_threshold.get())
                .map_err(|_| CT_NATIVE_FAILURE)?,
            generation: status.generation,
            phase: phase_code(status.phase),
            backend: backend_code(status.backend),
            completion: completion_code(status.completion),
            batches_completed: status.batches_completed,
        };
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (runtime, output);
        Err(CT_NATIVE_FAILURE)
    }
}

/// Creates the one Rust runtime owned by the one Flutter process.
#[unsafe(no_mangle)]
pub extern "C" fn ct_bridge_create() -> *mut c_void {
    #[cfg(target_os = "macos")]
    {
        let runtime = catch_unwind(AssertUnwindSafe(|| {
            cliptype_macos::initialize_application();
            BridgeRuntime::new()
        }))
        .ok()
        .and_then(Result::ok);
        runtime
            .map(Box::new)
            .map(Box::into_raw)
            .map_or(ptr::null_mut(), |pointer| pointer.cast::<c_void>())
    }
    #[cfg(not(target_os = "macos"))]
    {
        ptr::null_mut()
    }
}

/// Destroys the runtime after its bounded coordinator shutdown has completed.
#[unsafe(no_mangle)]
pub extern "C" fn ct_bridge_destroy(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the pointer was returned by `ct_bridge_create` and is
        // consumed exactly once by this function.
        let runtime = unsafe { Box::from_raw(handle.cast::<BridgeRuntime>()) };
        #[cfg(target_os = "macos")]
        {
            runtime
                .shutting_down
                .store(true, std::sync::atomic::Ordering::Release);
            let _ = runtime.coordinator.shutdown();
        }
        drop(runtime);
    }));
}

/// Reads the current content-free Rust settings/session snapshot.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ct_bridge_get_state(handle: *mut c_void, output: *mut CtBridgeState) -> i32 {
    if handle.is_null() || output.is_null() {
        return CT_INVALID;
    }
    catch_code(CT_NATIVE_FAILURE, || {
        // SAFETY: callers must pass a live bridge handle and writable output
        // storage for the duration of this synchronous call.
        let runtime = unsafe { &*handle.cast::<BridgeRuntime>() };
        // SAFETY: null was rejected above and the caller owns this output.
        let output = unsafe { &mut *output };
        fill_state(runtime, output)
    })
}

/// Copies one canonical shortcut into caller-owned bounded storage.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ct_bridge_get_hotkey(
    handle: *mut c_void,
    trigger: i32,
    output: *mut c_char,
    capacity: usize,
) -> i32 {
    if handle.is_null() || output.is_null() || capacity == 0 {
        return CT_INVALID;
    }
    catch_code(CT_NATIVE_FAILURE, || {
        #[cfg(target_os = "macos")]
        {
            // SAFETY: callers must pass a live bridge handle and writable
            // storage of at least `capacity` bytes.
            let runtime = unsafe { &*handle.cast::<BridgeRuntime>() };
            let settings = runtime.settings();
            let value = if trigger != 0 {
                settings.hotkeys.trigger.canonical()
            } else {
                settings.hotkeys.cancel.canonical()
            };
            let bytes = value.as_bytes();
            if bytes.len().saturating_add(1) > capacity {
                return Err(CT_INVALID);
            }
            // SAFETY: the capacity check guarantees both the copied bytes and
            // terminating NUL fit in caller-owned storage.
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), output.cast::<u8>(), bytes.len());
                *output.add(bytes.len()) = 0;
            }
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (handle, trigger, output, capacity);
            Err::<(), _>(CT_NATIVE_FAILURE)
        }
    })
}

/// Validates a complete pair using the Rust/macOS policy before OS probing.
#[unsafe(no_mangle)]
pub extern "C" fn ct_bridge_validate_hotkeys(trigger: *const c_char, cancel: *const c_char) -> i32 {
    #[cfg(target_os = "macos")]
    {
        match catch_unwind(AssertUnwindSafe(|| parse_hotkeys(trigger, cancel))) {
            Ok(Ok(_)) => CT_HOTKEY_AVAILABLE,
            Ok(Err(code)) => code,
            Err(_) => CT_HOTKEY_UNKNOWN,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (trigger, cancel);
        CT_HOTKEY_UNSUPPORTED
    }
}

/// Applies validated product settings and persists them atomically.
#[unsafe(no_mangle)]
pub extern "C" fn ct_bridge_save_settings(
    handle: *mut c_void,
    enabled: i32,
    notifications: i32,
    start_at_login: i32,
    mode: i32,
    characters_per_second: u16,
    jitter_percent: u8,
    typo_probability_percent: u8,
    auto_clipboard_threshold: u32,
    trigger: *const c_char,
    cancel: *const c_char,
) -> i32 {
    if handle.is_null() {
        return CT_INVALID;
    }
    #[cfg(target_os = "macos")]
    {
        return match catch_unwind(AssertUnwindSafe(|| {
            let settings = settings_from_args(SettingsArgs {
                enabled,
                notifications,
                start_at_login,
                mode,
                characters_per_second,
                jitter_percent,
                typo_probability_percent,
                auto_clipboard_threshold,
                trigger,
                cancel,
            })?;
            // SAFETY: callers must pass a live bridge handle for this
            // synchronous update.
            let runtime = unsafe { &*handle.cast::<BridgeRuntime>() };
            runtime.save(settings).map_err(|_| CT_NATIVE_FAILURE)
        })) {
            Ok(Ok(())) => CT_OK,
            Ok(Err(code)) => code,
            Err(_) => CT_NATIVE_FAILURE,
        };
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (
            enabled,
            notifications,
            start_at_login,
            mode,
            characters_per_second,
            jitter_percent,
            typo_probability_percent,
            auto_clipboard_threshold,
            trigger,
            cancel,
        );
        CT_NATIVE_FAILURE
    }
}

/// Starts one explicit session; the coordinator owns clipboard/input policy.
#[unsafe(no_mangle)]
pub extern "C" fn ct_bridge_trigger(handle: *mut c_void) -> i32 {
    if handle.is_null() {
        return CT_INVALID;
    }
    #[cfg(target_os = "macos")]
    {
        catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: callers must pass a live bridge handle.
            let runtime = unsafe { &*handle.cast::<BridgeRuntime>() };
            if runtime
                .shutting_down
                .load(std::sync::atomic::Ordering::Acquire)
            {
                return CT_SHUTTING_DOWN;
            }
            match runtime.coordinator.trigger() {
                cliptype_app::TriggerResult::Started { .. } => CT_OK,
                cliptype_app::TriggerResult::Busy => CT_BUSY,
                cliptype_app::TriggerResult::ShuttingDown => CT_SHUTTING_DOWN,
                cliptype_app::TriggerResult::Rejected(_) => CT_REJECTED,
                cliptype_app::TriggerResult::StartFailed => CT_NATIVE_FAILURE,
            }
        }))
        .unwrap_or(CT_NATIVE_FAILURE)
    }
    #[cfg(not(target_os = "macos"))]
    {
        CT_NATIVE_FAILURE
    }
}

/// Requests cancellation at the next bounded coordinator boundary.
#[unsafe(no_mangle)]
pub extern "C" fn ct_bridge_cancel(handle: *mut c_void) -> i32 {
    if handle.is_null() {
        return CT_INVALID;
    }
    #[cfg(target_os = "macos")]
    {
        catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: callers must pass a live bridge handle.
            let runtime = unsafe { &*handle.cast::<BridgeRuntime>() };
            match runtime.coordinator.cancel() {
                cliptype_app::CancelResult::Requested => CT_OK,
                cliptype_app::CancelResult::Idle => CT_REJECTED,
            }
        }))
        .unwrap_or(CT_NATIVE_FAILURE)
    }
    #[cfg(not(target_os = "macos"))]
    {
        CT_NATIVE_FAILURE
    }
}

/// Marks the coordinator as shutting down and waits within its configured bound.
#[unsafe(no_mangle)]
pub extern "C" fn ct_bridge_shutdown(handle: *mut c_void) -> i32 {
    if handle.is_null() {
        return CT_INVALID;
    }
    #[cfg(target_os = "macos")]
    {
        catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: callers must pass a live bridge handle.
            let runtime = unsafe { &*handle.cast::<BridgeRuntime>() };
            runtime
                .shutting_down
                .store(true, std::sync::atomic::Ordering::Release);
            match runtime.coordinator.shutdown() {
                cliptype_app::ShutdownResult::Complete => CT_OK,
                cliptype_app::ShutdownResult::TimedOut => CT_NATIVE_FAILURE,
            }
        }))
        .unwrap_or(CT_NATIVE_FAILURE)
    }
    #[cfg(not(target_os = "macos"))]
    {
        CT_NATIVE_FAILURE
    }
}
