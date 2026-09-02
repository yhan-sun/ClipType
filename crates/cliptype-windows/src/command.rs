//! Thread-owned Windows trigger and cancellation command source.

use core::ffi::c_void;
use std::{marker::PhantomData, ptr::null_mut, rc::Rc};

use cliptype_core::HotkeyPreset;
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
const MOD_NOREPEAT: u32 = 0x4000;

const VK_F11: u32 = 0x7A;
const VK_F12: u32 = 0x7B;

const PM_NOREMOVE: u32 = 0;
const WM_QUIT: u32 = 0x0012;
const WM_HOTKEY: u32 = 0x0312;
const WM_APP: u32 = 0x8000;
const WM_CLIPTYPE_SHUTDOWN: u32 = WM_APP + 0x4354;
const ERROR_HOTKEY_ALREADY_REGISTERED: u32 = 1409;

/// Human-readable default trigger binding retained for P1 compatibility.
pub const TRIGGER_HOTKEY: &str = "Ctrl+Alt+Shift+F12";
/// Human-readable default cancel binding retained for P1 compatibility.
pub const CANCEL_HOTKEY: &str = "Ctrl+Alt+Shift+F11";

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
    preset: HotkeyPreset,
    registered: bool,
    _thread_affine: PhantomData<Rc<()>>,
}

impl WindowsCommandSource {
    pub fn new() -> Self {
        Self::with_preset(HotkeyPreset::default())
    }

    pub fn with_preset(preset: HotkeyPreset) -> Self {
        // SAFETY: this call has no pointer or ownership preconditions.
        let owner_thread_id = unsafe { GetCurrentThreadId() };
        Self {
            owner_thread_id,
            preset,
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

    pub const fn preset(&self) -> HotkeyPreset {
        self.preset
    }

    pub const fn trigger_hotkey(&self) -> &'static str {
        self.preset.trigger_label()
    }

    pub const fn cancel_hotkey(&self) -> &'static str {
        self.preset.cancel_label()
    }

    /// Changes the reviewed preset only while no native registration exists.
    pub fn set_preset(&mut self, preset: HotkeyPreset) -> Result<(), CommandSourceError> {
        self.ensure_owner_thread()?;
        if self.registered {
            return Err(CommandSourceError::new(
                CommandSourceErrorKind::RegistrationConflict,
                None,
            ));
        }
        self.preset = preset;
        Ok(())
    }

    fn modifiers(&self) -> u32 {
        preset_modifiers(self.preset)
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

        self.create_message_queue();
        let modifiers = self.modifiers();

        // SAFETY: null HWND creates thread-owned registrations. The ids are
        // process-local constants, the modifiers are explicit, and F11/F12 are
        // valid virtual-key values.
        if unsafe { register_hot_key(null_mut(), TRIGGER_ID, modifiers, VK_F12) } == 0 {
            return Err(registration_error());
        }

        // SAFETY: same invariant as the trigger registration.
        if unsafe { register_hot_key(null_mut(), CANCEL_ID, modifiers, VK_F11) } == 0 {
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

const fn preset_modifiers(preset: HotkeyPreset) -> u32 {
    let modifiers = match preset {
        HotkeyPreset::CtrlAltShiftFunction => MOD_CONTROL | MOD_ALT | MOD_SHIFT,
        HotkeyPreset::CtrlAltFunction => MOD_CONTROL | MOD_ALT,
        HotkeyPreset::CtrlShiftFunction => MOD_CONTROL | MOD_SHIFT,
    };
    modifiers | MOD_NOREPEAT
}

const fn decode_message(message: u32, wparam: usize) -> Option<CommandEvent> {
    match (message, wparam as i32) {
        (WM_HOTKEY, TRIGGER_ID) => Some(CommandEvent::Trigger),
        (WM_HOTKEY, CANCEL_ID) => Some(CommandEvent::Cancel),
        (WM_CLIPTYPE_SHUTDOWN, _) => Some(CommandEvent::Shutdown),
        _ => None,
    }
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

    use cliptype_core::HotkeyPreset;
    use cliptype_platform::{CommandEvent, CommandEventSource};

    use super::{
        CANCEL_ID, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, TRIGGER_ID, WM_CLIPTYPE_SHUTDOWN,
        WM_HOTKEY, WindowsCommandSource, decode_message, preset_modifiers,
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
    fn reviewed_presets_map_to_explicit_no_repeat_modifiers() {
        assert_eq!(
            preset_modifiers(HotkeyPreset::CtrlAltShiftFunction),
            MOD_CONTROL | MOD_ALT | MOD_SHIFT | MOD_NOREPEAT
        );
        assert_eq!(
            preset_modifiers(HotkeyPreset::CtrlAltFunction),
            MOD_CONTROL | MOD_ALT | MOD_NOREPEAT
        );
        assert_eq!(
            preset_modifiers(HotkeyPreset::CtrlShiftFunction),
            MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT
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
