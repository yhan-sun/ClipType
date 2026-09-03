//! Thread-owned Windows trigger/cancellation source with live shortcut updates.

use core::ffi::c_void;
use std::{
    collections::VecDeque,
    marker::PhantomData,
    ptr::null_mut,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard, mpsc},
    time::Duration,
};

use cliptype_core::{
    HotkeyApplyResult, HotkeyAvailability, HotkeyKey, HotkeyPair, HotkeyPlatform, HotkeySpec,
    HotkeyValidationError,
};
use cliptype_platform::{
    CommandEvent, CommandEventSource, CommandSourceError, CommandSourceErrorKind,
    HotkeyControlError, HotkeyControlErrorKind, HotkeyControlPort, NativeError, NativeErrorKind,
};
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{GetMessageW, MSG, PeekMessageW, PostThreadMessageW},
};

#[link(name = "user32")]
unsafe extern "system" {
    #[link_name = "RegisterHotKey"]
    fn register_hot_key(hwnd: *mut c_void, id: i32, modifiers: u32, key: u32) -> i32;

    #[link_name = "UnregisterHotKey"]
    fn unregister_hot_key(hwnd: *mut c_void, id: i32) -> i32;
}

// Application hotkey identifiers must remain in 0x0000..=0xBFFF. Separate
// primary/secondary ids let a changed binding be secured before its old binding
// is released.
const TRIGGER_PRIMARY_ID: i32 = 0x4351;
const TRIGGER_SECONDARY_ID: i32 = 0x4352;
const CANCEL_PRIMARY_ID: i32 = 0x4353;
const CANCEL_SECONDARY_ID: i32 = 0x4354;
const PROBE_TRIGGER_ID: i32 = 0x4355;
const PROBE_CANCEL_ID: i32 = 0x4356;

const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;

const PM_NOREMOVE: u32 = 0;
const WM_QUIT: u32 = 0x0012;
const WM_HOTKEY: u32 = 0x0312;
const WM_APP: u32 = 0x8000;
const WM_CLIPTYPE_SHUTDOWN: u32 = WM_APP + 0x4354;
const WM_CLIPTYPE_CONTROL: u32 = WM_APP + 0x4355;
const ERROR_HOTKEY_ALREADY_REGISTERED: u32 = 1409;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(3);

/// Human-readable default trigger binding.
pub const TRIGGER_HOTKEY: &str = "Ctrl+Alt+Shift+V";
/// Human-readable default cancel binding.
pub const CANCEL_HOTKEY: &str = "Ctrl+Alt+Shift+X";

type ControlResult<T> = Result<T, HotkeyControlError>;

#[derive(Debug)]
enum ControlRequest {
    Probe {
        candidate: HotkeyPair,
        response: mpsc::SyncSender<ControlResult<HotkeyAvailability>>,
    },
    Replace {
        candidate: HotkeyPair,
        response: mpsc::SyncSender<ControlResult<HotkeyApplyResult>>,
    },
}

#[derive(Debug)]
struct SharedControl {
    pair: Mutex<HotkeyPair>,
    queue: Mutex<VecDeque<ControlRequest>>,
    operation: Mutex<()>,
}

impl SharedControl {
    fn new(pair: HotkeyPair) -> Self {
        Self {
            pair: Mutex::new(pair),
            queue: Mutex::new(VecDeque::new()),
            operation: Mutex::new(()),
        }
    }
}

/// Cross-thread handle to the message-loop owner.
///
/// Requests contain only shortcut values and content-free result channels. No
/// clipboard text, focused content, or arbitrary key events cross this control
/// plane.
#[derive(Clone)]
pub struct WindowsCommandSignal {
    owner_thread_id: u32,
    shared: Arc<SharedControl>,
}

impl std::fmt::Debug for WindowsCommandSignal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WindowsCommandSignal")
            .field("owner_thread_id", &self.owner_thread_id)
            .finish_non_exhaustive()
    }
}

impl WindowsCommandSignal {
    pub fn request_shutdown(&self) -> Result<(), CommandSourceError> {
        post_private_message(self.owner_thread_id, WM_CLIPTYPE_SHUTDOWN).map_err(|error| {
            command_error_from_control(error, CommandSourceErrorKind::EventLoopStopped)
        })
    }

    fn request_probe(&self, candidate: HotkeyPair) -> ControlResult<HotkeyAvailability> {
        let _operation = lock_unpoisoned(&self.shared.operation);
        let (sender, receiver) = mpsc::sync_channel(1);
        lock_unpoisoned(&self.shared.queue).push_back(ControlRequest::Probe {
            candidate,
            response: sender,
        });
        if let Err(error) = post_private_message(self.owner_thread_id, WM_CLIPTYPE_CONTROL) {
            let _ = lock_unpoisoned(&self.shared.queue).pop_back();
            return Err(error);
        }
        receiver
            .recv_timeout(CONTROL_TIMEOUT)
            .map_err(|_| HotkeyControlError::new(HotkeyControlErrorKind::Timeout, None))?
    }

    fn request_replace(&self, candidate: HotkeyPair) -> ControlResult<HotkeyApplyResult> {
        let _operation = lock_unpoisoned(&self.shared.operation);
        let (sender, receiver) = mpsc::sync_channel(1);
        lock_unpoisoned(&self.shared.queue).push_back(ControlRequest::Replace {
            candidate,
            response: sender,
        });
        if let Err(error) = post_private_message(self.owner_thread_id, WM_CLIPTYPE_CONTROL) {
            let _ = lock_unpoisoned(&self.shared.queue).pop_back();
            return Err(error);
        }
        receiver
            .recv_timeout(CONTROL_TIMEOUT)
            .map_err(|_| HotkeyControlError::new(HotkeyControlErrorKind::Timeout, None))?
    }
}

impl HotkeyControlPort for WindowsCommandSignal {
    fn current_pair(&self) -> HotkeyPair {
        *lock_unpoisoned(&self.shared.pair)
    }

    fn probe_pair(&self, candidate: HotkeyPair) -> Result<HotkeyAvailability, HotkeyControlError> {
        self.request_probe(candidate)
    }

    fn replace_pair(&self, candidate: HotkeyPair) -> Result<HotkeyApplyResult, HotkeyControlError> {
        self.request_replace(candidate)
    }
}

/// Thread-affine `RegisterHotKey`/`GetMessageW` event source.
///
/// `Rc` in the marker deliberately makes this type neither `Send` nor `Sync`.
/// Construction, registration, message pumping, replacement, and teardown
/// remain on one Windows message-queue owner thread.
#[derive(Debug)]
pub struct WindowsCommandSource {
    owner_thread_id: u32,
    shared: Arc<SharedControl>,
    registered: bool,
    trigger_id: i32,
    cancel_id: i32,
    _thread_affine: PhantomData<Rc<()>>,
}

impl WindowsCommandSource {
    pub fn new() -> Self {
        Self::with_pair(HotkeyPair::default())
    }

    pub fn with_pair(pair: HotkeyPair) -> Self {
        // SAFETY: this call has no pointer or ownership preconditions.
        let owner_thread_id = unsafe { GetCurrentThreadId() };
        Self {
            owner_thread_id,
            shared: Arc::new(SharedControl::new(pair)),
            registered: false,
            trigger_id: TRIGGER_PRIMARY_ID,
            cancel_id: CANCEL_PRIMARY_ID,
            _thread_affine: PhantomData,
        }
    }

    pub fn signal(&self) -> WindowsCommandSignal {
        WindowsCommandSignal {
            owner_thread_id: self.owner_thread_id,
            shared: Arc::clone(&self.shared),
        }
    }

    pub const fn is_registered(&self) -> bool {
        self.registered
    }

    pub fn pair(&self) -> HotkeyPair {
        *lock_unpoisoned(&self.shared.pair)
    }

    pub fn trigger_hotkey(&self) -> String {
        self.pair().trigger.label(HotkeyPlatform::Windows)
    }

    pub fn cancel_hotkey(&self) -> String {
        self.pair().cancel.label(HotkeyPlatform::Windows)
    }

    /// Changes the pair only while no native registration exists.
    pub fn set_pair(&mut self, pair: HotkeyPair) -> Result<(), CommandSourceError> {
        self.ensure_owner_thread()?;
        if self.registered {
            return Err(CommandSourceError::new(
                CommandSourceErrorKind::RegistrationConflict,
                None,
            ));
        }
        pair.validate_for(HotkeyPlatform::Windows)
            .map_err(|_| invalid_binding())?;
        *lock_unpoisoned(&self.shared.pair) = pair;
        Ok(())
    }

    fn ensure_owner_thread(&self) -> Result<(), CommandSourceError> {
        // SAFETY: this call has no pointer or ownership preconditions.
        let current = unsafe { GetCurrentThreadId() };
        if current == self.owner_thread_id {
            Ok(())
        } else {
            Err(CommandSourceError::new(
                CommandSourceErrorKind::EventLoopStopped,
                None,
            ))
        }
    }

    fn create_message_queue(&self) {
        let mut message = MSG::default();
        // SAFETY: `message` is a valid writable structure; null selects all
        // windows owned by this thread, and PM_NOREMOVE preserves any message.
        let _ = unsafe { PeekMessageW(&raw mut message, null_mut(), 0, 0, PM_NOREMOVE) };
    }

    fn process_control_requests(&mut self) {
        loop {
            let request = lock_unpoisoned(&self.shared.queue).pop_front();
            let Some(request) = request else {
                return;
            };
            match request {
                ControlRequest::Probe {
                    candidate,
                    response,
                } => {
                    let result = self.probe_pair_on_owner(candidate);
                    let _ = response.send(result);
                }
                ControlRequest::Replace {
                    candidate,
                    response,
                } => {
                    let result = self.replace_pair_on_owner(candidate);
                    let _ = response.send(result);
                }
            }
        }
    }

    fn probe_pair_on_owner(&mut self, candidate: HotkeyPair) -> ControlResult<HotkeyAvailability> {
        self.ensure_owner_thread()
            .map_err(control_error_from_command)?;
        let candidate = match validate_candidate(candidate) {
            Ok(candidate) => candidate,
            Err(availability) => return Ok(availability),
        };
        if !self.registered {
            return Ok(HotkeyAvailability::Unknown);
        }
        let current = self.pair();
        if candidate == current {
            return Ok(HotkeyAvailability::Available);
        }
        if is_cross_swap(current, candidate) {
            return Ok(HotkeyAvailability::Unsupported);
        }

        let mut trigger_registered = false;
        if candidate.trigger != current.trigger {
            let translated =
                translate_hotkey(candidate.trigger).map_err(control_error_from_command)?;
            match register_id(PROBE_TRIGGER_ID, translated) {
                Ok(()) => trigger_registered = true,
                Err(availability) => return Ok(availability),
            }
        }

        if candidate.cancel != current.cancel {
            let translated =
                translate_hotkey(candidate.cancel).map_err(control_error_from_command)?;
            if let Err(availability) = register_id(PROBE_CANCEL_ID, translated) {
                if trigger_registered {
                    unregister_id(PROBE_TRIGGER_ID);
                }
                return Ok(availability);
            }
        }

        if trigger_registered {
            unregister_id(PROBE_TRIGGER_ID);
        }
        if candidate.cancel != current.cancel {
            unregister_id(PROBE_CANCEL_ID);
        }
        Ok(HotkeyAvailability::Available)
    }

    fn replace_pair_on_owner(&mut self, candidate: HotkeyPair) -> ControlResult<HotkeyApplyResult> {
        self.ensure_owner_thread()
            .map_err(control_error_from_command)?;
        let candidate = match validate_candidate(candidate) {
            Ok(candidate) => candidate,
            Err(availability) => return Ok(HotkeyApplyResult::Rejected(availability)),
        };
        if !self.registered {
            return Err(HotkeyControlError::new(
                HotkeyControlErrorKind::EventLoopStopped,
                None,
            ));
        }
        let current = self.pair();
        if candidate == current {
            return Ok(HotkeyApplyResult::Applied);
        }
        // `RegisterHotKey` cannot own the same chord under two ids. A direct
        // trigger/cancel swap therefore cannot satisfy the no-gap transaction;
        // reject it while retaining the active pair.
        if is_cross_swap(current, candidate) {
            return Ok(HotkeyApplyResult::Rejected(HotkeyAvailability::Unsupported));
        }

        let trigger_changed = candidate.trigger != current.trigger;
        let cancel_changed = candidate.cancel != current.cancel;
        let next_trigger_id = alternate_trigger_id(self.trigger_id);
        let next_cancel_id = alternate_cancel_id(self.cancel_id);

        let mut new_trigger_registered = false;
        if trigger_changed {
            let translated =
                translate_hotkey(candidate.trigger).map_err(control_error_from_command)?;
            if let Err(availability) = register_id(next_trigger_id, translated) {
                return Ok(HotkeyApplyResult::RolledBack(availability));
            }
            new_trigger_registered = true;
        }

        if cancel_changed {
            let translated =
                translate_hotkey(candidate.cancel).map_err(control_error_from_command)?;
            if let Err(availability) = register_id(next_cancel_id, translated) {
                if new_trigger_registered {
                    unregister_id(next_trigger_id);
                }
                return Ok(HotkeyApplyResult::RolledBack(availability));
            }
        }

        let mut trigger_old_removed = false;
        if trigger_changed {
            trigger_old_removed = unregister_id_checked(self.trigger_id);
        }
        let mut cancel_old_removed = false;
        if cancel_changed {
            cancel_old_removed = unregister_id_checked(self.cancel_id);
        }

        if (trigger_changed && !trigger_old_removed) || (cancel_changed && !cancel_old_removed) {
            if trigger_changed {
                unregister_id(next_trigger_id);
            }
            if cancel_changed {
                unregister_id(next_cancel_id);
            }
            let mut rollback_ok = true;
            if trigger_old_removed {
                rollback_ok &= register_id(
                    self.trigger_id,
                    translate_hotkey(current.trigger).map_err(control_error_from_command)?,
                )
                .is_ok();
            }
            if cancel_old_removed {
                rollback_ok &= register_id(
                    self.cancel_id,
                    translate_hotkey(current.cancel).map_err(control_error_from_command)?,
                )
                .is_ok();
            }
            if rollback_ok {
                return Ok(HotkeyApplyResult::RolledBack(HotkeyAvailability::Unknown));
            }
            self.registered = false;
            return Err(last_control_error(HotkeyControlErrorKind::NativeFailure));
        }

        if trigger_changed {
            self.trigger_id = next_trigger_id;
        }
        if cancel_changed {
            self.cancel_id = next_cancel_id;
        }
        *lock_unpoisoned(&self.shared.pair) = candidate;
        Ok(HotkeyApplyResult::Applied)
    }

    fn unregister_best_effort(&mut self) -> Result<(), CommandSourceError> {
        self.ensure_owner_thread()?;
        if !self.registered {
            return Err(CommandSourceError::new(
                CommandSourceErrorKind::NotRegistered,
                None,
            ));
        }

        let trigger_removed = unregister_id_checked(self.trigger_id);
        let cancel_removed = unregister_id_checked(self.cancel_id);
        self.registered = false;

        if trigger_removed && cancel_removed {
            Ok(())
        } else {
            Err(last_command_error(CommandSourceErrorKind::NativeFailure))
        }
    }
}

impl Default for WindowsCommandSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandEventSource for WindowsCommandSource {
    fn register(&mut self) -> Result<(), CommandSourceError> {
        self.ensure_owner_thread()?;
        if self.registered {
            return Err(CommandSourceError::new(
                CommandSourceErrorKind::RegistrationConflict,
                None,
            ));
        }

        let pair = self.pair();
        pair.validate_for(HotkeyPlatform::Windows)
            .map_err(|_| invalid_binding())?;
        let trigger = translate_hotkey(pair.trigger)?;
        let cancel = translate_hotkey(pair.cancel)?;
        self.create_message_queue();

        register_id(self.trigger_id, trigger).map_err(availability_to_command_error)?;
        if let Err(availability) = register_id(self.cancel_id, cancel) {
            unregister_id(self.trigger_id);
            return Err(availability_to_command_error(availability));
        }

        self.registered = true;
        Ok(())
    }

    fn next_event(&mut self) -> Result<CommandEvent, CommandSourceError> {
        self.ensure_owner_thread()?;
        if !self.registered {
            return Err(CommandSourceError::new(
                CommandSourceErrorKind::NotRegistered,
                None,
            ));
        }

        loop {
            let mut message = MSG::default();
            // SAFETY: `message` is writable for the duration of this call; null
            // HWND selects the current thread queue. Policy work is not executed
            // in native callbacks.
            let status = unsafe { GetMessageW(&raw mut message, null_mut(), 0, 0) };
            if status == -1 {
                return Err(last_command_error(CommandSourceErrorKind::NativeFailure));
            }
            if status == 0 || message.message == WM_QUIT {
                return Ok(CommandEvent::Shutdown);
            }
            if message.message == WM_CLIPTYPE_CONTROL {
                self.process_control_requests();
                continue;
            }
            if let Some(event) = self.decode_message(message.message, message.wParam) {
                return Ok(event);
            }
        }
    }

    fn unregister(&mut self) -> Result<(), CommandSourceError> {
        self.unregister_best_effort()
    }
}

impl Drop for WindowsCommandSource {
    fn drop(&mut self) {
        if self.registered {
            let _ = self.unregister_best_effort();
        }
    }
}

fn validate_candidate(pair: HotkeyPair) -> Result<HotkeyPair, HotkeyAvailability> {
    pair.validate_for(HotkeyPlatform::Windows)
        .map_err(|error| match error {
            HotkeyValidationError::Reserved => HotkeyAvailability::Reserved,
            HotkeyValidationError::Unsupported => HotkeyAvailability::Unsupported,
            HotkeyValidationError::MissingPrimaryModifier
            | HotkeyValidationError::DuplicatePair => HotkeyAvailability::Unsupported,
        })
}

fn is_cross_swap(current: HotkeyPair, candidate: HotkeyPair) -> bool {
    (candidate.trigger == current.cancel && candidate.trigger != current.trigger)
        || (candidate.cancel == current.trigger && candidate.cancel != current.cancel)
}

const fn alternate_trigger_id(current: i32) -> i32 {
    if current == TRIGGER_PRIMARY_ID {
        TRIGGER_SECONDARY_ID
    } else {
        TRIGGER_PRIMARY_ID
    }
}

const fn alternate_cancel_id(current: i32) -> i32 {
    if current == CANCEL_PRIMARY_ID {
        CANCEL_SECONDARY_ID
    } else {
        CANCEL_PRIMARY_ID
    }
}

fn translate_hotkey(spec: HotkeySpec) -> Result<(u32, u32), CommandSourceError> {
    spec.validate_for(HotkeyPlatform::Windows)
        .map_err(|_| invalid_binding())?;
    let mut modifiers = MOD_NOREPEAT;
    if spec.modifiers.control() {
        modifiers |= MOD_CONTROL;
    }
    if spec.modifiers.alt() {
        modifiers |= MOD_ALT;
    }
    if spec.modifiers.shift() {
        modifiers |= MOD_SHIFT;
    }
    if spec.modifiers.meta() {
        modifiers |= MOD_WIN;
    }
    Ok((modifiers, virtual_key(spec.key)))
}

const fn virtual_key(key: HotkeyKey) -> u32 {
    match key {
        HotkeyKey::A => 0x41,
        HotkeyKey::B => 0x42,
        HotkeyKey::C => 0x43,
        HotkeyKey::D => 0x44,
        HotkeyKey::E => 0x45,
        HotkeyKey::F => 0x46,
        HotkeyKey::G => 0x47,
        HotkeyKey::H => 0x48,
        HotkeyKey::I => 0x49,
        HotkeyKey::J => 0x4A,
        HotkeyKey::K => 0x4B,
        HotkeyKey::L => 0x4C,
        HotkeyKey::M => 0x4D,
        HotkeyKey::N => 0x4E,
        HotkeyKey::O => 0x4F,
        HotkeyKey::P => 0x50,
        HotkeyKey::Q => 0x51,
        HotkeyKey::R => 0x52,
        HotkeyKey::S => 0x53,
        HotkeyKey::T => 0x54,
        HotkeyKey::U => 0x55,
        HotkeyKey::V => 0x56,
        HotkeyKey::W => 0x57,
        HotkeyKey::X => 0x58,
        HotkeyKey::Y => 0x59,
        HotkeyKey::Z => 0x5A,
        HotkeyKey::Digit0 => 0x30,
        HotkeyKey::Digit1 => 0x31,
        HotkeyKey::Digit2 => 0x32,
        HotkeyKey::Digit3 => 0x33,
        HotkeyKey::Digit4 => 0x34,
        HotkeyKey::Digit5 => 0x35,
        HotkeyKey::Digit6 => 0x36,
        HotkeyKey::Digit7 => 0x37,
        HotkeyKey::Digit8 => 0x38,
        HotkeyKey::Digit9 => 0x39,
        HotkeyKey::F1 => 0x70,
        HotkeyKey::F2 => 0x71,
        HotkeyKey::F3 => 0x72,
        HotkeyKey::F4 => 0x73,
        HotkeyKey::F5 => 0x74,
        HotkeyKey::F6 => 0x75,
        HotkeyKey::F7 => 0x76,
        HotkeyKey::F8 => 0x77,
        HotkeyKey::F9 => 0x78,
        HotkeyKey::F10 => 0x79,
        HotkeyKey::F11 => 0x7A,
        HotkeyKey::F12 => 0x7B,
        HotkeyKey::F13 => 0x7C,
        HotkeyKey::F14 => 0x7D,
        HotkeyKey::F15 => 0x7E,
        HotkeyKey::F16 => 0x7F,
        HotkeyKey::F17 => 0x80,
        HotkeyKey::F18 => 0x81,
        HotkeyKey::F19 => 0x82,
        HotkeyKey::F20 => 0x83,
        HotkeyKey::F21 => 0x84,
        HotkeyKey::F22 => 0x85,
        HotkeyKey::F23 => 0x86,
        HotkeyKey::F24 => 0x87,
        HotkeyKey::Space => 0x20,
        HotkeyKey::Tab => 0x09,
        HotkeyKey::Enter => 0x0D,
        HotkeyKey::Escape => 0x1B,
        HotkeyKey::Backspace => 0x08,
        HotkeyKey::Insert => 0x2D,
        HotkeyKey::Delete => 0x2E,
        HotkeyKey::Home => 0x24,
        HotkeyKey::End => 0x23,
        HotkeyKey::PageUp => 0x21,
        HotkeyKey::PageDown => 0x22,
        HotkeyKey::ArrowLeft => 0x25,
        HotkeyKey::ArrowUp => 0x26,
        HotkeyKey::ArrowRight => 0x27,
        HotkeyKey::ArrowDown => 0x28,
        HotkeyKey::Minus => 0xBD,
        HotkeyKey::Equal => 0xBB,
        HotkeyKey::BracketLeft => 0xDB,
        HotkeyKey::BracketRight => 0xDD,
        HotkeyKey::Backslash => 0xDC,
        HotkeyKey::Semicolon => 0xBA,
        HotkeyKey::Quote => 0xDE,
        HotkeyKey::Comma => 0xBC,
        HotkeyKey::Period => 0xBE,
        HotkeyKey::Slash => 0xBF,
        HotkeyKey::Backquote => 0xC0,
    }
}

impl WindowsCommandSource {
    const fn decode_message(&self, message: u32, wparam: usize) -> Option<CommandEvent> {
        if message == WM_CLIPTYPE_SHUTDOWN {
            return Some(CommandEvent::Shutdown);
        }
        if message != WM_HOTKEY {
            return None;
        }
        if wparam as i32 == self.trigger_id {
            Some(CommandEvent::Trigger)
        } else if wparam as i32 == self.cancel_id {
            Some(CommandEvent::Cancel)
        } else {
            None
        }
    }
}

fn register_id(id: i32, translated: (u32, u32)) -> Result<(), HotkeyAvailability> {
    // SAFETY: null HWND creates a thread-owned registration and all arguments
    // are validated scalar values.
    if unsafe { register_hot_key(null_mut(), id, translated.0, translated.1) } != 0 {
        return Ok(());
    }
    // SAFETY: no pointer preconditions.
    let code = unsafe { GetLastError() };
    if code == ERROR_HOTKEY_ALREADY_REGISTERED {
        Err(HotkeyAvailability::Conflict)
    } else {
        Err(HotkeyAvailability::Unknown)
    }
}

fn unregister_id(id: i32) {
    // SAFETY: best-effort cleanup of an application-owned id on the owner thread.
    let _ = unsafe { unregister_hot_key(null_mut(), id) };
}

fn unregister_id_checked(id: i32) -> bool {
    // SAFETY: removes an application-owned id on the owner thread.
    let result = unsafe { unregister_hot_key(null_mut(), id) };
    result != 0
}

fn availability_to_command_error(availability: HotkeyAvailability) -> CommandSourceError {
    let kind = match availability {
        HotkeyAvailability::Conflict | HotkeyAvailability::Reserved => {
            CommandSourceErrorKind::RegistrationConflict
        }
        HotkeyAvailability::Unsupported => CommandSourceErrorKind::InvalidBinding,
        HotkeyAvailability::Available | HotkeyAvailability::Unknown => {
            CommandSourceErrorKind::NativeFailure
        }
    };
    CommandSourceError::new(kind, None)
}

const fn invalid_binding() -> CommandSourceError {
    CommandSourceError::new(CommandSourceErrorKind::InvalidBinding, None)
}

fn last_command_error(kind: CommandSourceErrorKind) -> CommandSourceError {
    // SAFETY: `GetLastError` has no pointer or ownership preconditions.
    let code = unsafe { GetLastError() };
    CommandSourceError::new(
        kind,
        Some(NativeError::new(
            NativeErrorKind::Unknown,
            (code != 0).then_some(code),
        )),
    )
}

fn last_control_error(kind: HotkeyControlErrorKind) -> HotkeyControlError {
    // SAFETY: `GetLastError` has no pointer or ownership preconditions.
    let code = unsafe { GetLastError() };
    HotkeyControlError::new(
        kind,
        Some(NativeError::new(
            NativeErrorKind::Unknown,
            (code != 0).then_some(code),
        )),
    )
}

fn post_private_message(thread_id: u32, message: u32) -> Result<(), HotkeyControlError> {
    // SAFETY: private thread message carries no pointer or content.
    if unsafe { PostThreadMessageW(thread_id, message, 0, 0) } == 0 {
        Err(last_control_error(HotkeyControlErrorKind::EventLoopStopped))
    } else {
        Ok(())
    }
}

fn control_error_from_command(error: CommandSourceError) -> HotkeyControlError {
    HotkeyControlError::new(
        match error.kind {
            CommandSourceErrorKind::EventLoopStopped | CommandSourceErrorKind::NotRegistered => {
                HotkeyControlErrorKind::EventLoopStopped
            }
            CommandSourceErrorKind::RegistrationConflict
            | CommandSourceErrorKind::InvalidBinding
            | CommandSourceErrorKind::NativeFailure => HotkeyControlErrorKind::NativeFailure,
        },
        error.native,
    )
}

fn command_error_from_control(
    error: HotkeyControlError,
    fallback: CommandSourceErrorKind,
) -> CommandSourceError {
    CommandSourceError::new(
        match error.kind {
            HotkeyControlErrorKind::EventLoopStopped | HotkeyControlErrorKind::Timeout => fallback,
            HotkeyControlErrorKind::NativeFailure => CommandSourceErrorKind::NativeFailure,
        },
        error.native,
    )
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use cliptype_core::{
        HotkeyApplyResult, HotkeyAvailability, HotkeyKey, HotkeyPair, HotkeyPlatform,
    };
    use cliptype_platform::{CommandEvent, CommandEventSource, HotkeyControlPort};

    use super::{
        CANCEL_PRIMARY_ID, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, TRIGGER_PRIMARY_ID,
        WM_CLIPTYPE_SHUTDOWN, WM_HOTKEY, WindowsCommandSource, translate_hotkey,
    };

    #[test]
    fn maps_only_owned_messages_to_typed_commands() {
        let source = WindowsCommandSource::new();
        assert_eq!(
            source.decode_message(WM_HOTKEY, TRIGGER_PRIMARY_ID as usize),
            Some(CommandEvent::Trigger)
        );
        assert_eq!(
            source.decode_message(WM_HOTKEY, CANCEL_PRIMARY_ID as usize),
            Some(CommandEvent::Cancel)
        );
        assert_eq!(
            source.decode_message(WM_CLIPTYPE_SHUTDOWN, 0),
            Some(CommandEvent::Shutdown)
        );
        assert_eq!(source.decode_message(WM_HOTKEY, 999), None);
    }

    #[test]
    fn default_pair_maps_to_explicit_no_repeat_modifiers_and_vk() {
        let pair = HotkeyPair::default();
        assert_eq!(
            translate_hotkey(pair.trigger).expect("trigger translation"),
            (MOD_CONTROL | MOD_ALT | MOD_SHIFT | MOD_NOREPEAT, 0x56)
        );
        assert_eq!(pair.trigger.key, HotkeyKey::V);
        assert_eq!(
            pair.cancel.label(HotkeyPlatform::Windows),
            "Ctrl+Alt+Shift+X"
        );
    }

    #[test]
    fn owner_queue_handles_live_probe_replace_and_shutdown() {
        let mut source = WindowsCommandSource::new();
        source.register().expect("default hotkeys register");
        let signal = source.signal();
        let worker_signal = signal.clone();
        let replacement = HotkeyPair::new(
            "ctrl+alt+shift+b".parse().expect("trigger"),
            "ctrl+alt+shift+n".parse().expect("cancel"),
        );
        let worker = thread::spawn(move || {
            assert_eq!(
                worker_signal.probe_pair(replacement),
                Ok(HotkeyAvailability::Available)
            );
            assert_eq!(
                worker_signal.replace_pair(replacement),
                Ok(HotkeyApplyResult::Applied)
            );
            thread::sleep(Duration::from_millis(20));
            worker_signal.request_shutdown()
        });

        assert_eq!(
            source
                .next_event()
                .expect("owner receives private shutdown"),
            CommandEvent::Shutdown
        );
        worker
            .join()
            .expect("signal worker does not panic")
            .expect("shutdown message posts");
        assert_eq!(source.pair(), replacement);
        source.unregister().expect("hotkeys unregister");
        assert!(!source.is_registered());
        assert_eq!(signal.current_pair(), replacement);
    }

    #[test]
    fn direct_trigger_cancel_swap_is_rejected_without_churn() {
        let current = HotkeyPair::default();
        let swapped = HotkeyPair::new(current.cancel, current.trigger);
        assert!(super::is_cross_swap(current, swapped));
    }
}
