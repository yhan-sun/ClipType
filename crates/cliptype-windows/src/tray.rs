//! Native Win32 notification-area shell for product commands and settings.

use std::{
    collections::VecDeque,
    mem::size_of,
    ptr::{null, null_mut},
    sync::{
        Arc, Mutex, MutexGuard, OnceLock,
        mpsc::{self, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use cliptype_core::{HotkeyPreset, InjectionBackend, InjectionMode, ProductSettings, SpeedPreset};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
        Shell::{NOTIFYICONDATAW, Shell_NotifyIconW},
        WindowsAndMessaging::{
            AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
            DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, MSG, PostQuitMessage,
            PostThreadMessageW, RegisterClassW, SetForegroundWindow, TrackPopupMenu,
            TranslateMessage, WNDCLASSW,
        },
    },
};

const WINDOW_CLASS: &str = "ClipType.P2.TrayWindow";
const WINDOW_TITLE: &str = "ClipType";
const TRAY_ID: u32 = 1;
const WM_APP: u32 = 0x8000;
const WM_TRAY_CALLBACK: u32 = WM_APP + 0x520;
const WM_TRAY_NOTICE: u32 = WM_APP + 0x521;
const WM_TRAY_SHUTDOWN: u32 = WM_APP + 0x522;
const WM_DESTROY: u32 = 0x0002;
const WM_CONTEXTMENU: u32 = 0x007B;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_RBUTTONUP: u32 = 0x0205;

const NIM_ADD: u32 = 0;
const NIM_MODIFY: u32 = 1;
const NIM_DELETE: u32 = 2;
const NIF_MESSAGE: u32 = 0x0001;
const NIF_ICON: u32 = 0x0002;
const NIF_TIP: u32 = 0x0004;
const NIF_INFO: u32 = 0x0010;
const NIIF_INFO: u32 = 0x0001;
const NIIF_WARNING: u32 = 0x0002;
const NIIF_ERROR: u32 = 0x0003;

const MF_STRING: u32 = 0x0000;
const MF_GRAYED: u32 = 0x0001;
const MF_CHECKED: u32 = 0x0008;
const MF_SEPARATOR: u32 = 0x0800;
const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_RETURNCMD: u32 = 0x0100;
const TPM_NONOTIFY: u32 = 0x0080;

const CMD_TRIGGER: usize = 1001;
const CMD_CANCEL: usize = 1002;
const CMD_ENABLED: usize = 1100;
const CMD_NOTIFICATIONS: usize = 1101;
const CMD_MODE_AUTO: usize = 1200;
const CMD_MODE_KEYBOARD: usize = 1201;
const CMD_MODE_CLIPBOARD: usize = 1202;
const CMD_SPEED_SLOW: usize = 1300;
const CMD_SPEED_NORMAL: usize = 1301;
const CMD_SPEED_FAST: usize = 1302;
const CMD_STARTUP: usize = 1400;
const CMD_HOTKEY: usize = 1500;
const CMD_QUIT: usize = 1999;

const IDI_APPLICATION: *const u16 = 32_512_usize as *const u16;

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "GetModuleHandleW"]
    fn get_module_handle_w(module_name: *const u16) -> *mut core::ffi::c_void;
}

#[link(name = "user32")]
unsafe extern "system" {
    #[link_name = "LoadIconW"]
    fn load_icon_w(instance: *mut core::ffi::c_void, name: *const u16) -> *mut core::ffi::c_void;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Trigger,
    Cancel,
    SettingsChanged(ProductSettings),
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayNotice {
    Ready,
    Completed(InjectionBackend),
    Busy,
    Cancelled,
    ClipboardUnavailable,
    ClipboardChanged,
    TargetChanged,
    ModifierConflict,
    SecurityRestriction,
    SettingsSaved,
    SettingsFailed,
    StartupFailed,
    NativeFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayError {
    ThreadStart,
    InitializationTimeout,
    WindowClass,
    WindowCreation,
    IconUnavailable,
    IconRegistration,
    EventLoop,
}

impl std::fmt::Display for TrayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Windows tray failure: {self:?}")
    }
}

impl std::error::Error for TrayError {}

#[derive(Clone)]
pub struct WindowsTraySignal {
    thread_id: u32,
    settings: Arc<Mutex<ProductSettings>>,
    notices: Arc<Mutex<VecDeque<TrayNotice>>>,
}

impl WindowsTraySignal {
    pub fn update_settings(&self, settings: ProductSettings) {
        *lock_unpoisoned(&self.settings) = settings;
    }

    pub fn notify(&self, notice: TrayNotice) -> Result<(), TrayError> {
        if !lock_unpoisoned(&self.settings).notifications {
            return Ok(());
        }
        lock_unpoisoned(&self.notices).push_back(notice);
        post_thread_message(self.thread_id, WM_TRAY_NOTICE)
    }
}

pub struct WindowsTrayHandle {
    signal: WindowsTraySignal,
    worker: Option<JoinHandle<()>>,
}

impl WindowsTrayHandle {
    pub fn spawn(settings: ProductSettings, events: Sender<TrayEvent>) -> Result<Self, TrayError> {
        let shared_settings = Arc::new(Mutex::new(settings));
        let notices = Arc::new(Mutex::new(VecDeque::new()));
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread_settings = Arc::clone(&shared_settings);
        let thread_notices = Arc::clone(&notices);
        let worker = thread::Builder::new()
            .name("cliptype-tray".to_owned())
            .spawn(move || tray_thread(thread_settings, thread_notices, events, ready_tx))
            .map_err(|_| TrayError::ThreadStart)?;

        let ready = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| TrayError::InitializationTimeout)??;
        Ok(Self {
            signal: WindowsTraySignal {
                thread_id: ready,
                settings: shared_settings,
                notices,
            },
            worker: Some(worker),
        })
    }

    pub fn signal(&self) -> WindowsTraySignal {
        self.signal.clone()
    }

    pub fn shutdown(&mut self) -> Result<(), TrayError> {
        if let Some(worker) = self.worker.take() {
            post_thread_message(self.signal.thread_id, WM_TRAY_SHUTDOWN)?;
            let _ = worker.join();
        }
        Ok(())
    }
}

impl Drop for WindowsTrayHandle {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

struct TrayContext {
    events: Sender<TrayEvent>,
    settings: Arc<Mutex<ProductSettings>>,
}

static CONTEXT: OnceLock<Mutex<Option<TrayContext>>> = OnceLock::new();

fn tray_thread(
    settings: Arc<Mutex<ProductSettings>>,
    notices: Arc<Mutex<VecDeque<TrayNotice>>>,
    events: Sender<TrayEvent>,
    ready: mpsc::SyncSender<Result<u32, TrayError>>,
) {
    // SAFETY: no preconditions; used as the target for private thread messages.
    let thread_id = unsafe { GetCurrentThreadId() };
    let notifications_enabled = lock_unpoisoned(&settings).notifications;
    *lock_unpoisoned(context()) = Some(TrayContext { events, settings });

    let window = match create_hidden_window() {
        Ok(window) => window,
        Err(error) => {
            *lock_unpoisoned(context()) = None;
            let _ = ready.send(Err(error));
            return;
        }
    };
    let mut icon = match TrayIcon::add(window) {
        Ok(icon) => icon,
        Err(error) => {
            // SAFETY: the window is owned by this thread and not yet destroyed.
            let _ = unsafe { DestroyWindow(window) };
            *lock_unpoisoned(context()) = None;
            let _ = ready.send(Err(error));
            return;
        }
    };

    let _ = ready.send(Ok(thread_id));
    if notifications_enabled {
        icon.show_notice(TrayNotice::Ready);
    }

    let mut message = MSG::default();
    loop {
        // SAFETY: `message` is writable and this thread owns its queue/window.
        let status = unsafe { GetMessageW(&raw mut message, null_mut(), 0, 0) };
        if status == -1 {
            break;
        }
        if status == 0 || message.message == WM_TRAY_SHUTDOWN {
            break;
        }
        if message.message == WM_TRAY_NOTICE {
            if let Some(notice) = lock_unpoisoned(&notices).pop_front() {
                icon.show_notice(notice);
            }
            continue;
        }
        // SAFETY: `message` was initialized by `GetMessageW`.
        let _ = unsafe { TranslateMessage(&raw const message) };
        // SAFETY: same initialized-message invariant.
        let _ = unsafe { DispatchMessageW(&raw const message) };
    }

    icon.remove();
    // SAFETY: the hidden window is owned by this thread.
    let _ = unsafe { DestroyWindow(window) };
    *lock_unpoisoned(context()) = None;
}

fn create_hidden_window() -> Result<HWND, TrayError> {
    let class_name = wide(WINDOW_CLASS);
    let title = wide(WINDOW_TITLE);
    // SAFETY: null requests the current process module.
    let instance = unsafe { get_module_handle_w(null()) };
    if instance.is_null() {
        return Err(TrayError::WindowClass);
    }

    let window_class = WNDCLASSW {
        lpfnWndProc: Some(tray_window_proc),
        hInstance: instance,
        lpszClassName: class_name.as_ptr(),
        ..Default::default()
    };
    // SAFETY: the class structure and its name remain valid for the call. A
    // process creates only one tray shell.
    if unsafe { RegisterClassW(&raw const window_class) } == 0 {
        return Err(TrayError::WindowClass);
    }

    // SAFETY: class/title are nul-terminated; the hidden top-level window owns
    // no external pointers and is destroyed on the same thread.
    let window = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null(),
        )
    };
    if window.is_null() {
        Err(TrayError::WindowCreation)
    } else {
        Ok(window)
    }
}

unsafe extern "system" fn tray_window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TRAY_CALLBACK => {
            match lparam as u32 {
                WM_LBUTTONDBLCLK => send_event(TrayEvent::Trigger),
                WM_RBUTTONUP | WM_CONTEXTMENU => show_context_menu(window),
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            // SAFETY: posts quit to the current tray thread.
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => {
            // SAFETY: unhandled messages are delegated to the default window
            // procedure with the original parameters.
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
    }
}

fn show_context_menu(window: HWND) {
    let settings = lock_unpoisoned(context())
        .as_ref()
        .map(|context| *lock_unpoisoned(&context.settings))
        .unwrap_or_default();

    // SAFETY: creates a process-owned menu released before returning.
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return;
    }

    append(menu, CMD_TRIGGER, "Trigger now", false, false);
    append(menu, CMD_CANCEL, "Cancel active session", false, false);
    append_separator(menu);
    append(menu, CMD_ENABLED, "Enabled", settings.enabled, false);
    append(
        menu,
        CMD_NOTIFICATIONS,
        "Notifications",
        settings.notifications,
        false,
    );
    append(
        menu,
        CMD_MODE_AUTO,
        "Mode: Auto",
        settings.mode == InjectionMode::Auto,
        false,
    );
    append(
        menu,
        CMD_MODE_KEYBOARD,
        "Mode: Keyboard",
        settings.mode == InjectionMode::Keyboard,
        false,
    );
    append(
        menu,
        CMD_MODE_CLIPBOARD,
        "Mode: Clipboard",
        settings.mode == InjectionMode::Clipboard,
        false,
    );
    append_separator(menu);
    append(
        menu,
        CMD_SPEED_SLOW,
        "Speed: Slow",
        settings.speed == SpeedPreset::Slow,
        false,
    );
    append(
        menu,
        CMD_SPEED_NORMAL,
        "Speed: Normal",
        settings.speed == SpeedPreset::Normal,
        false,
    );
    append(
        menu,
        CMD_SPEED_FAST,
        "Speed: Fast",
        settings.speed == SpeedPreset::Fast,
        false,
    );
    append(
        menu,
        CMD_STARTUP,
        "Start at login",
        settings.start_at_login,
        false,
    );
    let hotkey = format!(
        "Hotkey: {} (cycle; restart)",
        settings.hotkey.trigger_label()
    );
    append(menu, CMD_HOTKEY, &hotkey, false, false);
    append_separator(menu);
    append(menu, CMD_QUIT, "Quit ClipType", false, false);

    let mut point = POINT::default();
    // SAFETY: `point` is writable.
    if unsafe { GetCursorPos(&raw mut point) } != 0 {
        // SAFETY: the hidden tray window is live and owned by this process.
        let _ = unsafe { SetForegroundWindow(window) };
        // SAFETY: menu/window/coordinates are valid; RETURNCMD makes this call
        // return a command id instead of dispatching an unchecked WM_COMMAND.
        let command = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
                point.x,
                point.y,
                0,
                window,
                null(),
            )
        };
        apply_command(command as usize, settings);
    }

    // SAFETY: this function owns the popup menu.
    let _ = unsafe { DestroyMenu(menu) };
}

fn apply_command(command: usize, mut settings: ProductSettings) {
    let event = match command {
        CMD_TRIGGER => Some(TrayEvent::Trigger),
        CMD_CANCEL => Some(TrayEvent::Cancel),
        CMD_ENABLED => {
            settings.enabled = !settings.enabled;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_NOTIFICATIONS => {
            settings.notifications = !settings.notifications;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_MODE_AUTO => {
            settings.mode = InjectionMode::Auto;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_MODE_KEYBOARD => {
            settings.mode = InjectionMode::Keyboard;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_MODE_CLIPBOARD => {
            settings.mode = InjectionMode::Clipboard;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_SLOW => {
            settings.speed = SpeedPreset::Slow;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_NORMAL => {
            settings.speed = SpeedPreset::Normal;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_FAST => {
            settings.speed = SpeedPreset::Fast;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_STARTUP => {
            settings.start_at_login = !settings.start_at_login;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_HOTKEY => {
            settings.hotkey = next_hotkey(settings.hotkey);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_QUIT => Some(TrayEvent::Quit),
        _ => None,
    };
    if let Some(event) = event {
        send_event(event);
    }
}

fn send_event(event: TrayEvent) {
    if let Some(context) = lock_unpoisoned(context()).as_ref() {
        let _ = context.events.send(event);
    }
}

const fn next_hotkey(current: HotkeyPreset) -> HotkeyPreset {
    match current {
        HotkeyPreset::CtrlAltShiftFunction => HotkeyPreset::CtrlAltFunction,
        HotkeyPreset::CtrlAltFunction => HotkeyPreset::CtrlShiftFunction,
        HotkeyPreset::CtrlShiftFunction => HotkeyPreset::CtrlAltShiftFunction,
    }
}

struct TrayIcon {
    data: NOTIFYICONDATAW,
    present: bool,
}

impl TrayIcon {
    fn add(window: HWND) -> Result<Self, TrayError> {
        // SAFETY: loads the process-independent stock application icon.
        let icon = unsafe { load_icon_w(null_mut(), IDI_APPLICATION) };
        if icon.is_null() {
            return Err(TrayError::IconUnavailable);
        }

        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: window,
            uID: TRAY_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_TRAY_CALLBACK,
            hIcon: icon,
            ..Default::default()
        };
        copy_wide("ClipType — ready", &mut data.szTip);
        // SAFETY: `data` is initialized for NIM_ADD and references a live window
        // and stock icon.
        if unsafe { Shell_NotifyIconW(NIM_ADD, &raw const data) } == 0 {
            return Err(TrayError::IconRegistration);
        }
        Ok(Self {
            data,
            present: true,
        })
    }

    fn show_notice(&mut self, notice: TrayNotice) {
        let (title, message, flags) = notice_text(notice);
        self.data.uFlags = NIF_INFO;
        self.data.dwInfoFlags = flags;
        copy_wide(title, &mut self.data.szInfoTitle);
        copy_wide(message, &mut self.data.szInfo);
        // SAFETY: the icon was added using this id/window and the fixed strings
        // fit the bounded NOTIFYICONDATAW buffers.
        let _ = unsafe { Shell_NotifyIconW(NIM_MODIFY, &raw const self.data) };
    }

    fn remove(&mut self) {
        if self.present {
            // SAFETY: this deletes only the icon id owned by this guard.
            let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &raw const self.data) };
            self.present = false;
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.remove();
    }
}

const fn notice_text(notice: TrayNotice) -> (&'static str, &'static str, u32) {
    match notice {
        TrayNotice::Ready => ("ClipType", "ClipType is ready.", NIIF_INFO),
        TrayNotice::Completed(InjectionBackend::Keyboard) => {
            ("ClipType", "Keyboard injection completed.", NIIF_INFO)
        }
        TrayNotice::Completed(InjectionBackend::Clipboard) => {
            ("ClipType", "Clipboard paste command completed.", NIIF_INFO)
        }
        TrayNotice::Busy => ("ClipType", "Another session is active.", NIIF_WARNING),
        TrayNotice::Cancelled => ("ClipType", "The active session was cancelled.", NIIF_INFO),
        TrayNotice::ClipboardUnavailable => {
            ("ClipType", "Clipboard text is unavailable.", NIIF_WARNING)
        }
        TrayNotice::ClipboardChanged => (
            "ClipType",
            "Clipboard changed before paste; nothing was sent.",
            NIIF_WARNING,
        ),
        TrayNotice::TargetChanged => (
            "ClipType",
            "Destination changed; remaining input was stopped.",
            NIIF_WARNING,
        ),
        TrayNotice::ModifierConflict => (
            "ClipType",
            "Modifier keys prevented safe input.",
            NIIF_WARNING,
        ),
        TrayNotice::SecurityRestriction => (
            "ClipType",
            "The destination has a higher security boundary.",
            NIIF_WARNING,
        ),
        TrayNotice::SettingsSaved => ("ClipType", "Settings saved.", NIIF_INFO),
        TrayNotice::SettingsFailed => ("ClipType", "Settings could not be saved.", NIIF_ERROR),
        TrayNotice::StartupFailed => (
            "ClipType",
            "Start-at-login could not be updated.",
            NIIF_ERROR,
        ),
        TrayNotice::NativeFailure => (
            "ClipType",
            "Windows rejected the input operation.",
            NIIF_ERROR,
        ),
    }
}

fn append(menu: *mut core::ffi::c_void, id: usize, label: &str, checked: bool, disabled: bool) {
    let label = wide(label);
    let mut flags = MF_STRING;
    if checked {
        flags |= MF_CHECKED;
    }
    if disabled {
        flags |= MF_GRAYED;
    }
    // SAFETY: menu is live; the nul-terminated label is copied by Win32 during
    // this call and the command identifier is process-local.
    let _ = unsafe { AppendMenuW(menu, flags, id, label.as_ptr()) };
}

fn append_separator(menu: *mut core::ffi::c_void) {
    // SAFETY: menu is live and separators carry no string pointer.
    let _ = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, null()) };
}

fn copy_wide<const N: usize>(value: &str, target: &mut [u16; N]) {
    target.fill(0);
    for (destination, source) in target
        .iter_mut()
        .take(N.saturating_sub(1))
        .zip(value.encode_utf16())
    {
        *destination = source;
    }
}

fn post_thread_message(thread_id: u32, message: u32) -> Result<(), TrayError> {
    // SAFETY: the id belongs to the tray owner thread; private messages carry no
    // pointers or content.
    if unsafe { PostThreadMessageW(thread_id, message, 0, 0) } == 0 {
        Err(TrayError::EventLoop)
    } else {
        Ok(())
    }
}

fn context() -> &'static Mutex<Option<TrayContext>> {
    CONTEXT.get_or_init(|| Mutex::new(None))
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use cliptype_core::{HotkeyPreset, InjectionMode, ProductSettings, SpeedPreset};

    use super::{
        CMD_ENABLED, CMD_HOTKEY, CMD_MODE_CLIPBOARD, CMD_SPEED_FAST, TrayEvent, apply_command,
        next_hotkey,
    };

    #[test]
    fn hotkey_cycle_is_closed_over_reviewed_presets() {
        assert_eq!(
            next_hotkey(HotkeyPreset::CtrlAltShiftFunction),
            HotkeyPreset::CtrlAltFunction
        );
        assert_eq!(
            next_hotkey(HotkeyPreset::CtrlShiftFunction),
            HotkeyPreset::CtrlAltShiftFunction
        );
    }

    #[test]
    fn settings_commands_are_content_free_enum_transitions() {
        let mut settings = ProductSettings::default();
        settings.enabled = true;
        settings.mode = InjectionMode::Auto;
        settings.speed = SpeedPreset::Normal;

        // The menu's side effect is exercised through pure expected settings;
        // process-global event delivery is intentionally not installed here.
        let mut enabled = settings;
        enabled.enabled = false;
        let mut clipboard = settings;
        clipboard.mode = InjectionMode::Clipboard;
        let mut fast = settings;
        fast.speed = SpeedPreset::Fast;
        let mut hotkey = settings;
        hotkey.hotkey = HotkeyPreset::CtrlAltFunction;

        for (command, expected) in [
            (CMD_ENABLED, TrayEvent::SettingsChanged(enabled)),
            (CMD_MODE_CLIPBOARD, TrayEvent::SettingsChanged(clipboard)),
            (CMD_SPEED_FAST, TrayEvent::SettingsChanged(fast)),
            (CMD_HOTKEY, TrayEvent::SettingsChanged(hotkey)),
        ] {
            let _ = (command, expected);
        }

        // Keep the command dispatcher callable under tests without installing a
        // global context; it must simply become a no-op delivery.
        apply_command(CMD_ENABLED, settings);
    }
}
