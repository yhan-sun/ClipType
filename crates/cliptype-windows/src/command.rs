//! Thread-owned Windows trigger and cancellation command source.

use core::ffi::c_void;
use std::{marker::PhantomData, ptr::null_mut, rc::Rc};

use cliptype_core::{HotkeyKey, HotkeyPair, HotkeyPlatform, HotkeySpec};
use cliptype_platform::{
    CommandEvent, CommandEventSource, CommandSourceError, CommandSourceErrorKind, NativeError,
    NativeErrorKind,
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

const TRIGGER_ID: i32 = 0x4354_01;
const CANCEL_ID: i32 = 0x4354_02;

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
const ERROR_HOTKEY_ALREADY_REGISTERED: u32 = 1409;

/// Human-readable default trigger binding.
pub const TRIGGER_HOTKEY: &str = "Ctrl+Alt+Shift+V";
/// Human-readable default cancel binding.
pub const CANCEL_HOTKEY: &str = "Ctrl+Alt+Shift+X";

/// Cross-thread signal handle for controlled host shutdown.
///
/// It posts only ClipType's private shutdown message. It does not capture or
/// synthesize user keystrokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsCommandSignal {
    owner_thread_id: u32,
}

impl WindowsCommandSignal {
    pub fn request_shutdown(self) -> Result<(), CommandSourceError> {
        // SAFETY: the thread id belongs to the source's owner, the message is in
        // the private application range, and neither parameter carries pointers.
        if unsafe { PostThreadMessageW(self.owner_thread_id, WM_CLIPTYPE_SHUTDOWN, 0, 0) } == 0 {
            Err(last_command_error(CommandSourceErrorKind::EventLoopStopped))
        } else {
            Ok(())
        }
    }
}

/// Thread-affine `RegisterHotKey`/`GetMessageW` event source.
///
/// `Rc` in the marker deliberately makes this type neither `Send` nor `Sync`.
/// Construction, registration, message pumping, and teardown therefore remain
/// on one Windows message-queue owner thread.
#[derive(Debug)]
pub struct WindowsCommandSource {
    owner_thread_id: u32,
    pair: HotkeyPair,
    registered: bool,
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
            pair,
            registered: false,
            _thread_affine: PhantomData,
        }
    }

    pub const fn signal(&self) -> WindowsCommandSignal {
        WindowsCommandSignal {
            owner_thread_id: self.owner_thread_id,
        }
    }

    pub const fn is_registered(&self) -> bool {
        self.registered
    }

    pub const fn pair(&self) -> HotkeyPair {
        self.pair
    }

    pub fn trigger_hotkey(&self) -> String {
        self.pair.trigger.label(HotkeyPlatform::Windows)
    }

    pub fn cancel_hotkey(&self) -> String {
        self.pair.cancel.label(HotkeyPlatform::Windows)
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
        self.pair = pair;
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
        // `PeekMessageW` causes Windows to create the current thread's message
        // queue even when no matching message is currently available.
        let mut message = MSG::default();
        // SAFETY: `message` is a valid writable structure; null selects all
        // windows owned by this thread, and PM_NOREMOVE preserves any message.
        let _ = unsafe { PeekMessageW(&raw mut message, null_mut(), 0, 0, PM_NOREMOVE) };
    }

    fn unregister_best_effort(&mut self) -> Result<(), CommandSourceError> {
        self.ensure_owner_thread()?;
        if !self.registered {
            return Err(CommandSourceError::new(
                CommandSourceErrorKind::NotRegistered,
                None,
            ));
        }

        // SAFETY: these ids were registered with a null HWND on this same owner
        // thread. Both calls are attempted so one failure cannot skip cleanup of
        // the other registration.
        let trigger_removed = unsafe { unregister_hot_key(null_mut(), TRIGGER_ID) } != 0;
        // SAFETY: same ownership invariant as the trigger registration.
        let cancel_removed = unsafe { unregister_hot_key(null_mut(), CANCEL_ID) } != 0;
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

        self.pair
            .validate_for(HotkeyPlatform::Windows)
            .map_err(|_| invalid_binding())?;
        let trigger = translate_hotkey(self.pair.trigger)?;
        let cancel = translate_hotkey(self.pair.cancel)?;
        self.create_message_queue();

        // SAFETY: null HWND creates thread-owned registrations. The ids are
        // process-local constants and translated values are validated.
        if unsafe { register_hot_key(null_mut(), TRIGGER_ID, trigger.0, trigger.1) } == 0 {
            return Err(registration_error());
        }

        // SAFETY: same invariant as the trigger registration.
        if unsafe { register_hot_key(null_mut(), CANCEL_ID, cancel.0, cancel.1) } == 0 {
            // SAFETY: the trigger registration succeeded on this owner thread.
            let _ = unsafe { unregister_hot_key(null_mut(), TRIGGER_ID) };
            return Err(registration_error());
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
            // in this message-pump method.
            let status = unsafe { GetMessageW(&raw mut message, null_mut(), 0, 0) };
            if status == -1 {
                return Err(last_command_error(CommandSourceErrorKind::NativeFailure));
            }
            if status == 0 || message.message == WM_QUIT {
                return Ok(CommandEvent::Shutdown);
            }

            if let Some(event) = decode_message(message.message, message.wParam) {
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

const fn decode_message(message: u32, wparam: usize) -> Option<CommandEvent> {
    match (message, wparam as i32) {
        (WM_HOTKEY, TRIGGER_ID) => Some(CommandEvent::Trigger),
        (WM_HOTKEY, CANCEL_ID) => Some(CommandEvent::Cancel),
        (WM_CLIPTYPE_SHUTDOWN, _) => Some(CommandEvent::Shutdown),
        _ => None,
    }
}

const fn invalid_binding() -> CommandSourceError {
    CommandSourceError::new(CommandSourceErrorKind::InvalidBinding, None)
}

fn registration_error() -> CommandSourceError {
    // SAFETY: `GetLastError` has no pointer or ownership preconditions.
    let code = unsafe { GetLastError() };
    let kind = if code == ERROR_HOTKEY_ALREADY_REGISTERED {
        CommandSourceErrorKind::RegistrationConflict
    } else {
        CommandSourceErrorKind::NativeFailure
    };
    CommandSourceError::new(
        kind,
        Some(NativeError::new(
            NativeErrorKind::TemporarilyUnavailable,
            (code != 0).then_some(code),
        )),
    )
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

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use cliptype_core::{HotkeyKey, HotkeyPair, HotkeyPlatform};
    use cliptype_platform::{CommandEvent, CommandEventSource};

    use super::{
        CANCEL_ID, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, TRIGGER_ID, WM_CLIPTYPE_SHUTDOWN,
        WM_HOTKEY, WindowsCommandSource, decode_message, translate_hotkey,
    };

    #[test]
    fn maps_only_owned_messages_to_typed_commands() {
        assert_eq!(
            decode_message(WM_HOTKEY, TRIGGER_ID as usize),
            Some(CommandEvent::Trigger)
        );
        assert_eq!(
            decode_message(WM_HOTKEY, CANCEL_ID as usize),
            Some(CommandEvent::Cancel)
        );
        assert_eq!(
            decode_message(WM_CLIPTYPE_SHUTDOWN, 0),
            Some(CommandEvent::Shutdown)
        );
        assert_eq!(decode_message(WM_HOTKEY, 999), None);
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
    fn owner_queue_receives_cross_thread_shutdown_and_cleans_up() {
        let mut source = WindowsCommandSource::new();
        source.register().expect("development hotkeys register");
        let signal = source.signal();
        let worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            signal.request_shutdown()
        });

        assert_eq!(
            source.next_event().expect("owner receives private message"),
            CommandEvent::Shutdown
        );
        worker
            .join()
            .expect("signal worker does not panic")
            .expect("shutdown message posts");
        source.unregister().expect("hotkeys unregister");
        assert!(!source.is_registered());
    }
}
