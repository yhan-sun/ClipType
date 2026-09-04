//! Shared, native-compiled ClipType settings window.
//!
//! This crate owns presentation and local recorder state only. Platform
//! registration, persistence, startup, permission, and injection policy remain
//! behind typed application/platform services.

#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use cliptype_core::{
    AutoClipboardThreshold, HotkeyAvailability, HotkeyKey, HotkeyModifiers, HotkeyPair,
    HotkeyPlatform, HotkeySpec, InjectionMode, ProductSettings, SETTINGS_SCHEMA_VERSION,
    SpeedPreset,
};
use cliptype_platform::AccessibilityPermissionState;
use slint::{CloseRequestResponse, ComponentHandle, SharedString, Weak, platform::Key};

slint::include_modules!();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsUiPlatform {
    Windows,
    MacOS,
}

impl SettingsUiPlatform {
    pub const fn hotkey_platform(self) -> HotkeyPlatform {
        match self {
            Self::Windows => HotkeyPlatform::Windows,
            Self::MacOS => HotkeyPlatform::MacOS,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOS => "macOS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutField {
    Trigger,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsUiEvent {
    Apply(ProductSettings),
    Probe(HotkeyPair),
    Reset(ProductSettings),
    RequestPermission,
    OpenPermissionSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsUiError {
    Creation,
    ThreadStart,
    InitializationTimeout,
    EventLoop,
}

impl std::fmt::Display for SettingsUiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "settings UI failure: {self:?}")
    }
}

impl std::error::Error for SettingsUiError {}

pub struct SettingsUiWindow {
    window: ClipTypeSettings,
    platform: SettingsUiPlatform,
}

impl SettingsUiWindow {
    pub fn new(
        settings: ProductSettings,
        platform: SettingsUiPlatform,
        permission: AccessibilityPermissionState,
        events: Sender<SettingsUiEvent>,
    ) -> Result<Self, SettingsUiError> {
        let window = ClipTypeSettings::new().map_err(|_| SettingsUiError::Creation)?;
        set_settings(&window, settings, platform);
        set_permission(&window, permission, platform);
        window.set_platform_name(platform.name().into());
        window.set_version_label(format!("ClipType v{}", env!("CARGO_PKG_VERSION")).into());
        window
            .window()
            .on_close_requested(|| CloseRequestResponse::HideWindow);

        connect_callbacks(&window, platform, events);
        Ok(Self { window, platform })
    }

    pub fn show(&self) -> Result<(), SettingsUiError> {
        self.window.show().map_err(|_| SettingsUiError::EventLoop)
    }

    pub fn hide(&self) -> Result<(), SettingsUiError> {
        self.window.hide().map_err(|_| SettingsUiError::EventLoop)
    }

    pub fn signal(&self) -> SettingsUiSignal {
        SettingsUiSignal {
            weak: self.window.as_weak(),
            platform: self.platform,
        }
    }

    pub fn run_event_loop(self) -> Result<(), SettingsUiError> {
        slint::run_event_loop_until_quit().map_err(|_| SettingsUiError::EventLoop)
    }
}

#[derive(Clone)]
pub struct SettingsUiSignal {
    weak: Weak<ClipTypeSettings>,
    platform: SettingsUiPlatform,
}

impl SettingsUiSignal {
    pub fn show(&self) -> Result<(), SettingsUiError> {
        let weak = self.weak.clone();
        weak.upgrade_in_event_loop(|window| {
            let _ = window.show();
        })
        .map_err(|_| SettingsUiError::EventLoop)
    }

    pub fn update_settings(&self, settings: ProductSettings) -> Result<(), SettingsUiError> {
        let weak = self.weak.clone();
        let platform = self.platform;
        weak.upgrade_in_event_loop(move |window| set_settings(&window, settings, platform))
            .map_err(|_| SettingsUiError::EventLoop)
    }

    pub fn update_permission(
        &self,
        permission: AccessibilityPermissionState,
    ) -> Result<(), SettingsUiError> {
        let weak = self.weak.clone();
        let platform = self.platform;
        weak.upgrade_in_event_loop(move |window| set_permission(&window, permission, platform))
            .map_err(|_| SettingsUiError::EventLoop)
    }

    pub fn update_probe(
        &self,
        trigger: HotkeyAvailability,
        cancel: HotkeyAvailability,
    ) -> Result<(), SettingsUiError> {
        let weak = self.weak.clone();
        weak.upgrade_in_event_loop(move |window| {
            window.set_trigger_status(availability_label(trigger).into());
            window.set_cancel_status(availability_label(cancel).into());
        })
        .map_err(|_| SettingsUiError::EventLoop)
    }

    pub fn set_status(&self, status: &'static str) -> Result<(), SettingsUiError> {
        let weak = self.weak.clone();
        weak.upgrade_in_event_loop(move |window| window.set_status_message(status.into()))
            .map_err(|_| SettingsUiError::EventLoop)
    }

    pub fn shutdown(&self) -> Result<(), SettingsUiError> {
        slint::quit_event_loop().map_err(|_| SettingsUiError::EventLoop)
    }
}

pub struct SettingsUiThread {
    signal: SettingsUiSignal,
    worker: Option<JoinHandle<()>>,
}

impl SettingsUiThread {
    pub fn spawn(
        settings: ProductSettings,
        platform: SettingsUiPlatform,
        permission: AccessibilityPermissionState,
        events: Sender<SettingsUiEvent>,
    ) -> Result<Self, SettingsUiError> {
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("cliptype-settings-ui".to_owned())
            .spawn(move || {
                let window = match SettingsUiWindow::new(settings, platform, permission, events) {
                    Ok(window) => window,
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                        return;
                    }
                };
                let signal = window.signal();
                if ready_tx.send(Ok(signal)).is_err() {
                    return;
                }
                let _ = window.run_event_loop();
            })
            .map_err(|_| SettingsUiError::ThreadStart)?;
        let signal = ready_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| SettingsUiError::InitializationTimeout)??;
        Ok(Self {
            signal,
            worker: Some(worker),
        })
    }

    pub fn signal(&self) -> SettingsUiSignal {
        self.signal.clone()
    }

    pub fn shutdown(&mut self) -> Result<(), SettingsUiError> {
        self.signal.shutdown()?;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        Ok(())
    }
}

impl Drop for SettingsUiThread {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn connect_callbacks(
    window: &ClipTypeSettings,
    platform: SettingsUiPlatform,
    events: Sender<SettingsUiEvent>,
) {
    let weak = window.as_weak();
    let apply_events = events.clone();
    window.on_apply_requested(move || {
        if let Some(window) = weak.upgrade() {
            match settings_from_window(&window, platform) {
                Ok(settings) => {
                    window.set_status_message("Applying settings…".into());
                    let _ = apply_events.send(SettingsUiEvent::Apply(settings));
                }
                Err(message) => window.set_status_message(message.into()),
            }
        }
    });

    let weak = window.as_weak();
    let probe_events = events.clone();
    window.on_probe_requested(move || {
        if let Some(window) = weak.upgrade() {
            match hotkeys_from_window(&window, platform) {
                Ok(pair) => {
                    window.set_trigger_status("Checking…".into());
                    window.set_cancel_status("Checking…".into());
                    let _ = probe_events.send(SettingsUiEvent::Probe(pair));
                }
                Err(message) => window.set_status_message(message.into()),
            }
        }
    });

    let weak = window.as_weak();
    let reset_events = events.clone();
    window.on_reset_requested(move || {
        if let Some(window) = weak.upgrade() {
            let defaults = ProductSettings::default();
            set_settings(&window, defaults, platform);
            let _ = reset_events.send(SettingsUiEvent::Reset(defaults));
        }
    });

    let weak = window.as_weak();
    window.on_shortcut_captured(move |field, text, control, alt, shift, meta| {
        if let Some(window) = weak.upgrade() {
            let field = if field == 0 {
                ShortcutField::Trigger
            } else {
                ShortcutField::Cancel
            };
            match shortcut_from_event(&text, control, alt, shift, meta, platform) {
                Ok(spec) => {
                    let label = spec.label(platform.hotkey_platform());
                    match field {
                        ShortcutField::Trigger => {
                            window.set_trigger_hotkey(label.into());
                            window.set_trigger_status("Not checked".into());
                        }
                        ShortcutField::Cancel => {
                            window.set_cancel_hotkey(label.into());
                            window.set_cancel_status("Not checked".into());
                        }
                    }
                    window.set_status_message(
                        "Shortcut recorded; check availability before Apply.".into(),
                    );
                }
                Err(message) => window.set_status_message(message.into()),
            }
        }
    });

    let weak = window.as_weak();
    window.on_shortcut_cleared(move |field| {
        if let Some(window) = weak.upgrade() {
            if field == 0 {
                window.set_trigger_hotkey("".into());
                window.set_trigger_status("Invalid".into());
            } else {
                window.set_cancel_hotkey("".into());
                window.set_cancel_status("Invalid".into());
            }
            window.set_status_message("A trigger and cancel shortcut are both required.".into());
        }
    });

    let request_events = events.clone();
    window.on_request_permission(move || {
        let _ = request_events.send(SettingsUiEvent::RequestPermission);
    });
    window.on_open_permission_settings(move || {
        let _ = events.send(SettingsUiEvent::OpenPermissionSettings);
    });
}

fn settings_from_window(
    window: &ClipTypeSettings,
    platform: SettingsUiPlatform,
) -> Result<ProductSettings, &'static str> {
    let hotkeys = hotkeys_from_window(window, platform)?;
    let mode = match window.get_mode_index() {
        0 => InjectionMode::Keyboard,
        1 => InjectionMode::Clipboard,
        2 => InjectionMode::Auto,
        3 => InjectionMode::Code,
        _ => return Err("Invalid injection mode."),
    };
    let threshold = usize::try_from(window.get_auto_threshold())
        .ok()
        .and_then(|value| AutoClipboardThreshold::new(value).ok())
        .ok_or("Auto threshold is out of range.")?;
    let characters_per_second = u16::try_from(window.get_characters_per_second())
        .map_err(|_| "Characters per second is out of range.")?;
    let jitter_percent =
        u8::try_from(window.get_jitter_percent()).map_err(|_| "Jitter is out of range.")?;
    let typo_probability_percent =
        u8::try_from(window.get_typo_percent()).map_err(|_| "Typo probability is out of range.")?;
    ProductSettings {
        version: SETTINGS_SCHEMA_VERSION,
        enabled: window.get_enabled_setting(),
        mode,
        auto_clipboard_threshold: threshold,
        speed: SpeedPreset::Custom,
        characters_per_second,
        jitter_percent,
        typo_probability_percent,
        notifications: window.get_notifications_setting(),
        start_at_login: window.get_startup_setting(),
        hotkeys,
    }
    .validate()
    .map_err(|_| "One or more settings are invalid.")
}

fn hotkeys_from_window(
    window: &ClipTypeSettings,
    platform: SettingsUiPlatform,
) -> Result<HotkeyPair, &'static str> {
    let trigger = parse_display_shortcut(&window.get_trigger_hotkey())?;
    let cancel = parse_display_shortcut(&window.get_cancel_hotkey())?;
    HotkeyPair::new(trigger, cancel)
        .validate_for(platform.hotkey_platform())
        .map_err(|_| "The shortcut pair is invalid, duplicate, reserved, or unsupported.")
}

fn parse_display_shortcut(value: &SharedString) -> Result<HotkeySpec, &'static str> {
    let canonical = value
        .as_str()
        .replace('⌃', "ctrl+")
        .replace('⌥', "alt+")
        .replace('⇧', "shift+")
        .replace('⌘', "meta+")
        .replace("Ctrl+", "ctrl+")
        .replace("Alt+", "alt+")
        .replace("Shift+", "shift+")
        .replace("Win+", "meta+")
        .replace(' ', "")
        .to_ascii_lowercase();
    canonical.parse().map_err(|_| "Shortcut text is invalid.")
}

fn shortcut_from_event(
    text: &SharedString,
    control: bool,
    alt: bool,
    shift: bool,
    meta: bool,
    platform: SettingsUiPlatform,
) -> Result<HotkeySpec, &'static str> {
    let key =
        hotkey_key_from_slint(text).ok_or("That key is not supported as a global shortcut.")?;
    let mut modifiers = HotkeyModifiers::empty();
    match platform {
        SettingsUiPlatform::Windows => {
            if control {
                modifiers = modifiers.with(HotkeyModifiers::CONTROL);
            }
            if meta {
                modifiers = modifiers.with(HotkeyModifiers::META);
            }
        }
        SettingsUiPlatform::MacOS => {
            // Slint intentionally maps Command to `control` and physical
            // Control to `meta` on Apple platforms.
            if control {
                modifiers = modifiers.with(HotkeyModifiers::META);
            }
            if meta {
                modifiers = modifiers.with(HotkeyModifiers::CONTROL);
            }
        }
    }
    if alt {
        modifiers = modifiers.with(HotkeyModifiers::ALT);
    }
    if shift {
        modifiers = modifiers.with(HotkeyModifiers::SHIFT);
    }
    HotkeySpec::new(modifiers, key)
        .validate_for(platform.hotkey_platform())
        .map_err(|_| "Shortcut requires a primary modifier and must not be reserved.")
}

fn hotkey_key_from_slint(text: &SharedString) -> Option<HotkeyKey> {
    let value = text.as_str();
    if value.chars().count() == 1 {
        let canonical = value.to_ascii_lowercase();
        if let Ok(key) = canonical.parse() {
            return Some(key);
        }
    }

    const SPECIAL: &[(Key, HotkeyKey)] = &[
        (Key::Tab, HotkeyKey::Tab),
        (Key::Return, HotkeyKey::Enter),
        (Key::Escape, HotkeyKey::Escape),
        (Key::Backspace, HotkeyKey::Backspace),
        (Key::Delete, HotkeyKey::Delete),
        (Key::Insert, HotkeyKey::Insert),
        (Key::Home, HotkeyKey::Home),
        (Key::End, HotkeyKey::End),
        (Key::PageUp, HotkeyKey::PageUp),
        (Key::PageDown, HotkeyKey::PageDown),
        (Key::LeftArrow, HotkeyKey::ArrowLeft),
        (Key::RightArrow, HotkeyKey::ArrowRight),
        (Key::UpArrow, HotkeyKey::ArrowUp),
        (Key::DownArrow, HotkeyKey::ArrowDown),
        (Key::F1, HotkeyKey::F1),
        (Key::F2, HotkeyKey::F2),
        (Key::F3, HotkeyKey::F3),
        (Key::F4, HotkeyKey::F4),
        (Key::F5, HotkeyKey::F5),
        (Key::F6, HotkeyKey::F6),
        (Key::F7, HotkeyKey::F7),
        (Key::F8, HotkeyKey::F8),
        (Key::F9, HotkeyKey::F9),
        (Key::F10, HotkeyKey::F10),
        (Key::F11, HotkeyKey::F11),
        (Key::F12, HotkeyKey::F12),
        (Key::F13, HotkeyKey::F13),
        (Key::F14, HotkeyKey::F14),
        (Key::F15, HotkeyKey::F15),
        (Key::F16, HotkeyKey::F16),
        (Key::F17, HotkeyKey::F17),
        (Key::F18, HotkeyKey::F18),
        (Key::F19, HotkeyKey::F19),
        (Key::F20, HotkeyKey::F20),
        (Key::F21, HotkeyKey::F21),
        (Key::F22, HotkeyKey::F22),
        (Key::F23, HotkeyKey::F23),
        (Key::F24, HotkeyKey::F24),
    ];
    SPECIAL.iter().find_map(|(slint_key, hotkey_key)| {
        let encoded: SharedString = (*slint_key).into();
        (encoded.as_str() == value).then_some(*hotkey_key)
    })
}

fn set_settings(
    window: &ClipTypeSettings,
    settings: ProductSettings,
    platform: SettingsUiPlatform,
) {
    window.set_enabled_setting(settings.enabled);
    window.set_notifications_setting(settings.notifications);
    window.set_startup_setting(settings.start_at_login);
    window.set_mode_index(match settings.mode {
        InjectionMode::Keyboard => 0,
        InjectionMode::Clipboard => 1,
        InjectionMode::Auto => 2,
        InjectionMode::Code => 3,
    });
    window.set_characters_per_second(i32::from(settings.characters_per_second));
    window.set_jitter_percent(i32::from(settings.jitter_percent));
    window.set_typo_percent(i32::from(settings.typo_probability_percent));
    window.set_auto_threshold(
        i32::try_from(settings.auto_clipboard_threshold.get()).unwrap_or(i32::MAX),
    );
    window.set_trigger_hotkey(
        settings
            .hotkeys
            .trigger
            .label(platform.hotkey_platform())
            .into(),
    );
    window.set_cancel_hotkey(
        settings
            .hotkeys
            .cancel
            .label(platform.hotkey_platform())
            .into(),
    );
    window.set_trigger_status("Not checked".into());
    window.set_cancel_status("Not checked".into());
}

fn set_permission(
    window: &ClipTypeSettings,
    permission: AccessibilityPermissionState,
    platform: SettingsUiPlatform,
) {
    window.set_permission_status(permission_label(permission).into());
    window.set_permission_actions_visible(matches!(platform, SettingsUiPlatform::MacOS));
    window.set_capability_status(match platform {
        SettingsUiPlatform::Windows => {
            "Windows global shortcuts and input use normal-user Win32 APIs. Elevated targets remain blocked."
        }
        SettingsUiPlatform::MacOS => {
            "Accessibility permission is required for cross-application synthetic input and detailed destination evidence."
        }
    }.into());
}

const fn permission_label(permission: AccessibilityPermissionState) -> &'static str {
    match permission {
        AccessibilityPermissionState::NotRequired => "Not required",
        AccessibilityPermissionState::NotRequested => "Not requested",
        AccessibilityPermissionState::NotGranted => "Not granted",
        AccessibilityPermissionState::Granted => "Granted",
        AccessibilityPermissionState::Revoked => "Revoked while running",
        AccessibilityPermissionState::Unknown => "Unknown",
    }
}

const fn availability_label(availability: HotkeyAvailability) -> &'static str {
    match availability {
        HotkeyAvailability::Available => "Available",
        HotkeyAvailability::Conflict => "In use",
        HotkeyAvailability::Reserved => "Reserved",
        HotkeyAvailability::Unsupported => "Unsupported",
        HotkeyAvailability::Unknown => "Cannot fully verify",
    }
}

#[cfg(test)]
mod tests {
    use cliptype_core::{HotkeyKey, HotkeyModifiers};
    use slint::{SharedString, platform::Key};

    use super::{SettingsUiPlatform, hotkey_key_from_slint, shortcut_from_event};

    #[test]
    fn printable_and_function_keys_map_to_native_neutral_keys() {
        assert_eq!(
            hotkey_key_from_slint(&SharedString::from("v")),
            Some(HotkeyKey::V)
        );
        let f8: SharedString = Key::F8.into();
        assert_eq!(hotkey_key_from_slint(&f8), Some(HotkeyKey::F8));
    }

    #[test]
    fn windows_recorder_preserves_control_and_meta_meaning() {
        let spec = shortcut_from_event(
            &SharedString::from("v"),
            true,
            true,
            true,
            false,
            SettingsUiPlatform::Windows,
        )
        .expect("valid Windows shortcut");
        assert!(spec.modifiers.contains(HotkeyModifiers::CONTROL));
        assert!(spec.modifiers.contains(HotkeyModifiers::ALT));
        assert!(spec.modifiers.contains(HotkeyModifiers::SHIFT));
        assert!(!spec.modifiers.contains(HotkeyModifiers::META));
    }

    #[test]
    fn macos_recorder_reverses_slint_command_control_mapping() {
        let spec = shortcut_from_event(
            &SharedString::from("v"),
            true,
            true,
            false,
            false,
            SettingsUiPlatform::MacOS,
        )
        .expect("valid macOS shortcut");
        assert!(spec.modifiers.contains(HotkeyModifiers::META));
        assert!(spec.modifiers.contains(HotkeyModifiers::ALT));
        assert!(!spec.modifiers.contains(HotkeyModifiers::CONTROL));
    }
}
