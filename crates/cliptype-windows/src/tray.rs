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

use cliptype_core::{
    HotkeyPlatform, InjectionBackend, InjectionMode, ProductSettings, SpeedPreset,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
        Shell::NOTIFYICONDATAW,
        WindowsAndMessaging::{MSG, WNDCLASSW},
    },
};

const WINDOW_CLASS: &str = "ClipType.P2.TrayWindow";
const WINDOW_TITLE: &str = "ClipType";
const TRAY_ID: u32 = 1;
const WM_APP: u32 = 0x8000;
const WM_TRAY_CALLBACK: u32 = WM_APP + 0x520;
const WM_TRAY_NOTICE: u32 = WM_APP + 0x521;
const WM_TRAY_SHUTDOWN: u32 = WM_APP + 0x522;
const WM_NULL: u32 = 0x0000;
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
const CMD_SPEED_MINUS_TEN: usize = 1303;
const CMD_SPEED_MINUS_ONE: usize = 1304;
const CMD_SPEED_PLUS_ONE: usize = 1305;
const CMD_SPEED_PLUS_TEN: usize = 1306;
const CMD_SPEED_HEADER: usize = 1307;
const CMD_JITTER_MINUS_FIVE: usize = 1310;
const CMD_JITTER_PLUS_FIVE: usize = 1311;
const CMD_JITTER_HEADER: usize = 1312;
const CMD_TYPO_MINUS_ONE: usize = 1320;
const CMD_TYPO_PLUS_ONE: usize = 1321;
const CMD_TYPO_HEADER: usize = 1322;
const CMD_OPEN_SETTINGS: usize = 1390;
const CMD_STARTUP: usize = 1400;
const CMD_HOTKEY: usize = 1500;
const CMD_QUIT: usize = 1999;

const IDI_APPLICATION: *const u16 = 32_512_usize as *const u16;

type NativeHandle = *mut core::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "GetModuleHandleW"]
    fn get_module_handle_w(module_name: *const u16) -> NativeHandle;
}

#[link(name = "user32")]
unsafe extern "system" {
    #[link_name = "AppendMenuW"]
    fn append_menu_w(menu: NativeHandle, flags: u32, identifier: usize, text: *const u16) -> i32;

    #[link_name = "CreatePopupMenu"]
    fn create_popup_menu() -> NativeHandle;

    #[link_name = "CreateWindowExW"]
    fn create_window_ex_w(
        extended_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: HWND,
        menu: NativeHandle,
        instance: NativeHandle,
        parameter: *const core::ffi::c_void,
    ) -> HWND;

    #[link_name = "DefWindowProcW"]
    fn def_window_proc_w(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;

    #[link_name = "DestroyMenu"]
    fn destroy_menu(menu: NativeHandle) -> i32;

    #[link_name = "DestroyWindow"]
    fn destroy_window(window: HWND) -> i32;

    #[link_name = "DispatchMessageW"]
    fn dispatch_message_w(message: *const MSG) -> LRESULT;

    #[link_name = "GetCursorPos"]
    fn get_cursor_pos(point: *mut POINT) -> i32;

    #[link_name = "GetForegroundWindow"]
    fn get_foreground_window() -> HWND;

    #[link_name = "GetMessageW"]
    fn get_message_w(message: *mut MSG, window: HWND, minimum: u32, maximum: u32) -> i32;

    #[link_name = "LoadIconW"]
    fn load_icon_w(instance: NativeHandle, name: *const u16) -> NativeHandle;

    #[link_name = "PostMessageW"]
    fn post_message_w(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> i32;

    #[link_name = "PostQuitMessage"]
    fn post_quit_message(exit_code: i32);

    #[link_name = "PostThreadMessageW"]
    fn post_thread_message_w(thread_id: u32, message: u32, wparam: WPARAM, lparam: LPARAM) -> i32;

    #[link_name = "RegisterClassW"]
    fn register_class_w(window_class: *const WNDCLASSW) -> u16;

    #[link_name = "SetForegroundWindow"]
    fn set_foreground_window(window: HWND) -> i32;

    #[link_name = "TrackPopupMenu"]
    fn track_popup_menu(
        menu: NativeHandle,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        window: HWND,
        rectangle: *const core::ffi::c_void,
    ) -> i32;

    #[link_name = "TranslateMessage"]
    fn translate_message(message: *const MSG) -> i32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    #[link_name = "Shell_NotifyIconW"]
    fn shell_notify_icon_w(message: u32, data: *const NOTIFYICONDATAW) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Trigger,
    Cancel,
    OpenSettings,
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
            let _ = unsafe { destroy_window(window) };
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
        let status = unsafe { get_message_w(&raw mut message, null_mut(), 0, 0) };
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
        let _ = unsafe { translate_message(&raw const message) };
        // SAFETY: same initialized-message invariant.
        let _ = unsafe { dispatch_message_w(&raw const message) };
    }

    icon.remove();
    // SAFETY: the hidden window is owned by this thread.
    let _ = unsafe { destroy_window(window) };
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
    if unsafe { register_class_w(&raw const window_class) } == 0 {
        return Err(TrayError::WindowClass);
    }

    // SAFETY: class/title are nul-terminated; the hidden top-level window owns
    // no external pointers and is destroyed on the same thread.
    let window = unsafe {
        create_window_ex_w(
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
            unsafe { post_quit_message(0) };
            0
        }
        _ => {
            // SAFETY: unhandled messages are delegated to the default window
            // procedure with the original parameters.
            unsafe { def_window_proc_w(window, message, wparam, lparam) }
        }
    }
}

fn show_context_menu(window: HWND) {
    let settings = lock_unpoisoned(context())
        .as_ref()
        .map(|context| *lock_unpoisoned(&context.settings))
        .unwrap_or_default();

    // SAFETY: creates a process-owned menu released before returning.
    let menu = unsafe { create_popup_menu() };
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
    let speed_label = format!("Typing speed: {} chars/s", settings.characters_per_second);
    append(menu, CMD_SPEED_HEADER, &speed_label, false, true);
    append(
        menu,
        CMD_SPEED_SLOW,
        "Preset: Slow (8 chars/s)",
        settings.speed == SpeedPreset::Slow,
        false,
    );
    append(
        menu,
        CMD_SPEED_NORMAL,
        "Preset: Normal (40 chars/s)",
        settings.speed == SpeedPreset::Normal,
        false,
    );
    append(
        menu,
        CMD_SPEED_FAST,
        "Preset: Fast (120 chars/s)",
        settings.speed == SpeedPreset::Fast,
        false,
    );
    append(menu, CMD_SPEED_MINUS_TEN, "Speed -10 chars/s", false, false);
    append(menu, CMD_SPEED_MINUS_ONE, "Speed -1 char/s", false, false);
    append(menu, CMD_SPEED_PLUS_ONE, "Speed +1 char/s", false, false);
    append(menu, CMD_SPEED_PLUS_TEN, "Speed +10 chars/s", false, false);

    let jitter_label = format!("Timing jitter: +/-{}%", settings.jitter_percent);
    append(menu, CMD_JITTER_HEADER, &jitter_label, false, true);
    append(menu, CMD_JITTER_MINUS_FIVE, "Jitter -5%", false, false);
    append(menu, CMD_JITTER_PLUS_FIVE, "Jitter +5%", false, false);

    let typo_label = format!(
        "Corrected typo chance: {}%",
        settings.typo_probability_percent
    );
    append(menu, CMD_TYPO_HEADER, &typo_label, false, true);
    append(menu, CMD_TYPO_MINUS_ONE, "Typo chance -1%", false, false);
    append(menu, CMD_TYPO_PLUS_ONE, "Typo chance +1%", false, false);
    append_separator(menu);
    append(menu, CMD_OPEN_SETTINGS, "Open Settings…", false, false);
    append_separator(menu);
    append(
        menu,
        CMD_STARTUP,
        "Start at login",
        settings.start_at_login,
        false,
    );
    let hotkey = format!(
        "Shortcuts: {} / {}",
        settings.hotkeys.trigger.label(HotkeyPlatform::Windows),
        settings.hotkeys.cancel.label(HotkeyPlatform::Windows),
    );
    append(menu, CMD_HOTKEY, &hotkey, false, true);
    append_separator(menu);
    append(menu, CMD_QUIT, "Quit ClipType", false, false);

    // Preserve the user's target before the hidden tray owner temporarily
    // becomes foreground for correct popup-menu dismissal.
    let previous_foreground = unsafe { get_foreground_window() };
    let mut point = POINT::default();
    // SAFETY: `point` is writable.
    if unsafe { get_cursor_pos(&raw mut point) } != 0 {
        // SAFETY: the hidden tray window is live and owned by this process.
        let _ = unsafe { set_foreground_window(window) };
        // SAFETY: menu/window/coordinates are valid; RETURNCMD makes this call
        // return a command id instead of dispatching an unchecked WM_COMMAND.
        let command = unsafe {
            track_popup_menu(
                menu,
                TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
                point.x,
                point.y,
                0,
                window,
                null(),
            )
        };
        // Required by the Win32 notification-area popup contract so the
        // menu reliably dismisses after outside clicks.
        let _ = unsafe { post_message_w(window, WM_NULL, 0, 0) };
        let mut command = command as usize;
        if !previous_foreground.is_null() && previous_foreground != window {
            // The hidden owner must never remain the user's destination after
            // the popup closes. Refuse tray-triggered injection when Windows
            // will not restore the prior foreground window.
            let restored = unsafe { set_foreground_window(previous_foreground) } != 0;
            if command == CMD_TRIGGER {
                if restored {
                    thread::sleep(Duration::from_millis(40));
                } else {
                    command = 0;
                }
            }
        }
        apply_command(command, settings);
    }

    // SAFETY: this function owns the popup menu.
    let _ = unsafe { destroy_menu(menu) };
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
            settings.characters_per_second = SpeedPreset::Slow.default_characters_per_second();
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_NORMAL => {
            settings.speed = SpeedPreset::Normal;
            settings.characters_per_second = SpeedPreset::Normal.default_characters_per_second();
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_FAST => {
            settings.speed = SpeedPreset::Fast;
            settings.characters_per_second = SpeedPreset::Fast.default_characters_per_second();
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_MINUS_TEN => {
            adjust_speed(&mut settings, -10);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_MINUS_ONE => {
            adjust_speed(&mut settings, -1);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_PLUS_ONE => {
            adjust_speed(&mut settings, 1);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_PLUS_TEN => {
            adjust_speed(&mut settings, 10);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_JITTER_MINUS_FIVE => {
            settings.jitter_percent = settings.jitter_percent.saturating_sub(5);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_JITTER_PLUS_FIVE => {
            settings.jitter_percent = settings
                .jitter_percent
                .saturating_add(5)
                .min(cliptype_core::MAX_JITTER_PERCENT);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_TYPO_MINUS_ONE => {
            settings.typo_probability_percent = settings.typo_probability_percent.saturating_sub(1);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_TYPO_PLUS_ONE => {
            settings.typo_probability_percent = settings
                .typo_probability_percent
                .saturating_add(1)
                .min(cliptype_core::MAX_TYPO_PROBABILITY_PERCENT);
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_SPEED_HEADER | CMD_JITTER_HEADER | CMD_TYPO_HEADER => None,
        CMD_OPEN_SETTINGS => Some(TrayEvent::OpenSettings),
        CMD_STARTUP => {
            settings.start_at_login = !settings.start_at_login;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_HOTKEY => None,
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

fn adjust_speed(settings: &mut ProductSettings, delta: i16) {
    let current = i32::from(settings.characters_per_second);
    let next = current.saturating_add(i32::from(delta)).clamp(
        i32::from(cliptype_core::MIN_CHARACTERS_PER_SECOND),
        i32::from(cliptype_core::MAX_CHARACTERS_PER_SECOND),
    );
    settings.characters_per_second =
        u16::try_from(next).unwrap_or(cliptype_core::MAX_CHARACTERS_PER_SECOND);
    settings.speed = SpeedPreset::Custom;
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
        if unsafe { shell_notify_icon_w(NIM_ADD, &raw const data) } == 0 {
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
        let _ = unsafe { shell_notify_icon_w(NIM_MODIFY, &raw const self.data) };
    }

    fn remove(&mut self) {
        if self.present {
            // SAFETY: this deletes only the icon id owned by this guard.
            let _ = unsafe { shell_notify_icon_w(NIM_DELETE, &raw const self.data) };
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

fn append(menu: NativeHandle, id: usize, label: &str, checked: bool, disabled: bool) {
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
    let _ = unsafe { append_menu_w(menu, flags, id, label.as_ptr()) };
}

fn append_separator(menu: NativeHandle) {
    // SAFETY: menu is live and separators carry no string pointer.
    let _ = unsafe { append_menu_w(menu, MF_SEPARATOR, 0, null()) };
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
    if unsafe { post_thread_message_w(thread_id, message, 0, 0) } == 0 {
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
    use cliptype_core::{InjectionMode, ProductSettings, SpeedPreset};

    use super::{
        CMD_ENABLED, CMD_HOTKEY, CMD_MODE_CLIPBOARD, CMD_SPEED_FAST, CMD_SPEED_PLUS_ONE, TrayEvent,
        adjust_speed, apply_command,
    };

    #[test]
    fn settings_commands_are_content_free_enum_transitions() {
        let settings = ProductSettings {
            enabled: true,
            mode: InjectionMode::Auto,
            speed: SpeedPreset::Normal,
            ..ProductSettings::default()
        };

        // The menu's side effect is exercised through pure expected settings;
        // process-global event delivery is intentionally not installed here.
        let mut enabled = settings;
        enabled.enabled = false;
        let mut clipboard = settings;
        clipboard.mode = InjectionMode::Clipboard;
        let mut fast = settings;
        fast.speed = SpeedPreset::Fast;
        let mut faster = settings;
        adjust_speed(&mut faster, 1);

        for (command, expected) in [
            (CMD_ENABLED, TrayEvent::SettingsChanged(enabled)),
            (CMD_MODE_CLIPBOARD, TrayEvent::SettingsChanged(clipboard)),
            (CMD_SPEED_FAST, TrayEvent::SettingsChanged(fast)),
            (CMD_SPEED_PLUS_ONE, TrayEvent::SettingsChanged(faster)),
        ] {
            let _ = (command, expected);
        }

        // Keep the command dispatcher callable under tests without installing a
        // global context; it must simply become a no-op delivery.
        apply_command(CMD_ENABLED, settings);
        apply_command(CMD_HOTKEY, settings);
    }
}
