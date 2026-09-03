//! macOS menu-bar composition root for ClipType.

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("cliptype status=unsupported_platform platform=macos_required");
}

#[cfg(target_os = "macos")]
fn main() {
    if let Err(error) = macos_host::run() {
        eprintln!("cliptype status=fatal category={}", error.label());
        std::process::exit(1);
    }
}

#[cfg(target_os = "macos")]
mod macos_host {
    use std::{
        cell::RefCell,
        env,
        path::PathBuf,
        rc::Rc,
        sync::{Arc, mpsc},
        time::Duration,
    };

    use cliptype_app::{
        CancelResult, Coordinator, SessionCompletion, SettingsSource, SettingsStore,
        StatusSnapshot, TriggerResult,
    };
    use cliptype_core::{
        HotkeyApplyResult, HotkeyAvailability, HotkeyPlatform, PreparationFailure, ProductSettings,
        TerminalOutcome,
    };
    use cliptype_macos::{
        MacAccessibility, MacClipboard, MacHotkeyController, MacKeyboard, MacMenuEvent,
        MacModifiers, MacPaste, MacStartup, MacStartupStatus, MacStatusItem, MacTarget,
        initialize_application,
    };
    use cliptype_platform::{AccessibilityPermissionPort, AccessibilityPermissionState};
    use cliptype_ui::{SettingsUiEvent, SettingsUiPlatform, SettingsUiSignal, SettingsUiWindow};
    use slint::{Timer, TimerMode};

    const CONTROL_POLL_INTERVAL: Duration = Duration::from_millis(40);

    #[derive(Debug, Clone, Copy)]
    enum HostError {
        SettingsPath,
        SettingsLoad,
        SettingsSave,
        InvalidConfiguration,
        HotkeyRegistration,
        StatusItem,
        SettingsUi,
        EventLoop,
        WorkerShutdownTimeout,
    }

    impl HostError {
        const fn label(self) -> &'static str {
            match self {
                Self::SettingsPath => "settings_path",
                Self::SettingsLoad => "settings_load",
                Self::SettingsSave => "settings_save",
                Self::InvalidConfiguration => "invalid_configuration",
                Self::HotkeyRegistration => "hotkey_registration",
                Self::StatusItem => "status_item",
                Self::SettingsUi => "settings_ui",
                Self::EventLoop => "event_loop",
                Self::WorkerShutdownTimeout => "worker_shutdown_timeout",
            }
        }
    }

    struct Runtime {
        coordinator: Arc<Coordinator>,
        store: SettingsStore,
        startup: MacStartup,
        permission: Rc<MacAccessibility>,
        hotkeys: MacHotkeyController,
        status_item: MacStatusItem,
        ui: SettingsUiSignal,
        menu_events: mpsc::Receiver<MacMenuEvent>,
        ui_events: mpsc::Receiver<SettingsUiEvent>,
        settings: ProductSettings,
        previous_status: Option<StatusSnapshot>,
        quitting: bool,
    }

    pub fn run() -> Result<(), HostError> {
        initialize_application();

        let store = SettingsStore::new(settings_path()?);
        let loaded = store.load().map_err(|_| HostError::SettingsLoad)?;
        let settings = loaded.settings;
        settings
            .hotkeys
            .validate_for(HotkeyPlatform::MacOS)
            .map_err(|_| HostError::InvalidConfiguration)?;
        if loaded.source != SettingsSource::Primary {
            store.save(settings).map_err(|_| HostError::SettingsSave)?;
        }

        let startup = MacStartup;
        let _ = startup.set_enabled(settings.start_at_login);
        let permission = Rc::new(MacAccessibility::default());
        let permission_state = permission.state();

        let coordinator = Arc::new(
            Coordinator::new_product(
                MacClipboard,
                MacTarget,
                MacKeyboard,
                MacModifiers,
                MacPaste,
                settings
                    .runtime_config()
                    .map_err(|_| HostError::InvalidConfiguration)?,
            )
            .map_err(|_| HostError::InvalidConfiguration)?,
        );

        let (menu_tx, menu_rx) = mpsc::channel();
        let hotkeys = MacHotkeyController::new(settings.hotkeys, menu_tx.clone())
            .map_err(|_| HostError::HotkeyRegistration)?;
        let status_item = MacStatusItem::new(menu_tx).map_err(|_| HostError::StatusItem)?;
        status_item.update(
            settings.enabled,
            settings.mode,
            permission_state,
            startup_enabled(startup.status()),
        );

        let (ui_tx, ui_rx) = mpsc::channel();
        let settings_window =
            SettingsUiWindow::new(settings, SettingsUiPlatform::MacOS, permission_state, ui_tx)
                .map_err(|_| HostError::SettingsUi)?;
        let ui = settings_window.signal();
        if env::args_os().any(|argument| argument == "--settings")
            || permission_state != AccessibilityPermissionState::Granted
        {
            settings_window.show().map_err(|_| HostError::SettingsUi)?;
        }

        let runtime = Rc::new(RefCell::new(Runtime {
            coordinator,
            store,
            startup,
            permission,
            hotkeys,
            status_item,
            ui,
            menu_events: menu_rx,
            ui_events: ui_rx,
            settings,
            previous_status: None,
            quitting: false,
        }));

        let timer = Timer::default();
        let timer_runtime = Rc::clone(&runtime);
        timer.start(TimerMode::Repeated, CONTROL_POLL_INTERVAL, move || {
            process_tick(&timer_runtime);
        });

        let event_loop = settings_window.run_event_loop();
        timer.stop();
        let shutdown = runtime.borrow().coordinator.shutdown();
        if shutdown == cliptype_app::ShutdownResult::TimedOut {
            return Err(HostError::WorkerShutdownTimeout);
        }
        event_loop.map_err(|_| HostError::EventLoop)
    }

    fn process_tick(runtime: &Rc<RefCell<Runtime>>) {
        let mut runtime = runtime.borrow_mut();
        while let Ok(event) = runtime.menu_events.try_recv() {
            handle_menu_event(&mut runtime, event);
        }
        while let Ok(event) = runtime.ui_events.try_recv() {
            handle_ui_event(&mut runtime, event);
        }

        let permission = runtime.permission.state();
        let _ = runtime.ui.update_permission(permission);
        runtime.status_item.update(
            runtime.settings.enabled,
            runtime.settings.mode,
            permission,
            startup_enabled(runtime.startup.status()),
        );

        let status = runtime.coordinator.status();
        if runtime.previous_status != Some(status) {
            update_completion_status(&runtime.ui, status);
            runtime.previous_status = Some(status);
        }
    }

    fn handle_menu_event(runtime: &mut Runtime, event: MacMenuEvent) {
        match event {
            MacMenuEvent::Trigger => notify_trigger(&runtime.ui, runtime.coordinator.trigger()),
            MacMenuEvent::Cancel => {
                let result = runtime.coordinator.cancel();
                let _ = runtime.ui.set_status(match result {
                    CancelResult::Requested => "Cancelling the active session…",
                    CancelResult::Idle => "No active session.",
                });
            }
            MacMenuEvent::OpenSettings | MacMenuEvent::About => {
                let _ = runtime.ui.show();
            }
            MacMenuEvent::ToggleEnabled => {
                let mut proposed = runtime.settings;
                proposed.enabled = !proposed.enabled;
                let _ = apply_settings(runtime, proposed);
            }
            MacMenuEvent::ToggleStartup => {
                let mut proposed = runtime.settings;
                proposed.start_at_login = !proposed.start_at_login;
                let _ = apply_settings(runtime, proposed);
            }
            MacMenuEvent::Permission => {
                let _ = runtime.permission.request();
                let _ = runtime.ui.show();
                let _ = runtime.ui.update_permission(runtime.permission.state());
            }
            MacMenuEvent::Quit => {
                if !runtime.quitting {
                    runtime.quitting = true;
                    let _ = runtime.coordinator.cancel();
                    let _ = runtime.ui.shutdown();
                }
            }
        }
    }

    fn handle_ui_event(runtime: &mut Runtime, event: SettingsUiEvent) {
        match event {
            SettingsUiEvent::Apply(proposed) => {
                let _ = apply_settings(runtime, proposed);
            }
            SettingsUiEvent::Probe(pair) => {
                let availability = runtime.hotkeys.probe_pair(pair);
                let _ = runtime.ui.update_probe(availability, availability);
                let _ = runtime.ui.set_status(availability_message(availability));
            }
            SettingsUiEvent::Reset(defaults) => {
                let _ = apply_settings(runtime, defaults);
            }
            SettingsUiEvent::RequestPermission => {
                let result = runtime.permission.request();
                let _ = runtime.ui.set_status(if result.is_ok() {
                    "macOS permission request opened. Approve ClipType in System Settings."
                } else {
                    "The Accessibility permission request could not be opened."
                });
                let _ = runtime.ui.update_permission(runtime.permission.state());
            }
            SettingsUiEvent::OpenPermissionSettings => {
                let result = runtime.permission.open_system_settings();
                let _ = runtime.ui.set_status(if result.is_ok() {
                    "Opened macOS Accessibility settings."
                } else {
                    "Accessibility settings could not be opened."
                });
            }
        }
    }

    fn apply_settings(runtime: &mut Runtime, proposed: ProductSettings) -> bool {
        let proposed = match proposed.validate() {
            Ok(value) => value,
            Err(_) => {
                let _ = runtime.ui.set_status("The proposed settings are invalid.");
                return false;
            }
        };
        if proposed
            .hotkeys
            .validate_for(HotkeyPlatform::MacOS)
            .is_err()
        {
            let _ = runtime
                .ui
                .set_status("The shortcut pair is reserved or unsupported on macOS.");
            return false;
        }
        let Ok(new_runtime) = proposed.runtime_config() else {
            let _ = runtime.ui.set_status("The runtime settings are invalid.");
            return false;
        };
        let Ok(old_runtime) = runtime.settings.runtime_config() else {
            let _ = runtime
                .ui
                .set_status("The current runtime settings are invalid.");
            return false;
        };

        let old = runtime.settings;
        let hotkeys_changed = proposed.hotkeys != old.hotkeys;
        if hotkeys_changed {
            match runtime.hotkeys.replace_pair(proposed.hotkeys) {
                HotkeyApplyResult::Applied => {}
                HotkeyApplyResult::Rejected(availability)
                | HotkeyApplyResult::RolledBack(availability) => {
                    let _ = runtime.ui.update_probe(availability, availability);
                    let _ = runtime.ui.set_status(
                        "The new shortcuts could not be registered; the old pair remains active.",
                    );
                    return false;
                }
            }
        }

        let startup_changed = proposed.start_at_login != old.start_at_login;
        if startup_changed
            && runtime
                .startup
                .set_enabled(proposed.start_at_login)
                .is_err()
        {
            if hotkeys_changed {
                let _ = runtime.hotkeys.replace_pair(old.hotkeys);
            }
            let _ = runtime
                .ui
                .set_status("Start at Login could not be changed.");
            return false;
        }

        if runtime.coordinator.update_config(new_runtime).is_err() {
            if startup_changed {
                let _ = runtime.startup.set_enabled(old.start_at_login);
            }
            if hotkeys_changed {
                let _ = runtime.hotkeys.replace_pair(old.hotkeys);
            }
            let _ = runtime
                .ui
                .set_status("The running configuration could not be updated.");
            return false;
        }

        if runtime.store.save(proposed).is_err() {
            let _ = runtime.coordinator.update_config(old_runtime);
            if startup_changed {
                let _ = runtime.startup.set_enabled(old.start_at_login);
            }
            if hotkeys_changed {
                let _ = runtime.hotkeys.replace_pair(old.hotkeys);
            }
            let _ = runtime.ui.set_status(
                "Settings could not be saved; the previous configuration was restored.",
            );
            return false;
        }

        runtime.settings = proposed;
        runtime.status_item.update(
            proposed.enabled,
            proposed.mode,
            runtime.permission.state(),
            startup_enabled(runtime.startup.status()),
        );
        let _ = runtime.ui.update_settings(proposed);
        let _ = runtime
            .ui
            .update_probe(HotkeyAvailability::Available, HotkeyAvailability::Available);
        let _ = runtime.ui.set_status("Settings applied.");
        true
    }

    fn notify_trigger(ui: &SettingsUiSignal, result: TriggerResult) {
        let message = match result {
            TriggerResult::Started { .. } => "Typing session started.",
            TriggerResult::Busy => "Another typing session is already active.",
            TriggerResult::ShuttingDown => "ClipType is shutting down.",
            TriggerResult::StartFailed => "The typing worker could not be started.",
            TriggerResult::Rejected(PreparationFailure::Disabled) => "ClipType is disabled.",
            TriggerResult::Rejected(PreparationFailure::KnownSecurityRestriction) => {
                "macOS blocked the destination or Accessibility permission is missing."
            }
            TriggerResult::Rejected(_) => "The typing session could not be prepared safely.",
        };
        let _ = ui.set_status(message);
    }

    fn update_completion_status(ui: &SettingsUiSignal, status: StatusSnapshot) {
        let Some(completion) = status.completion else {
            return;
        };
        let message = match completion {
            SessionCompletion::Finished(TerminalOutcome::Completed) => "Typing completed.",
            SessionCompletion::Finished(TerminalOutcome::Cancelled)
            | SessionCompletion::PreparationFailed(PreparationFailure::Cancelled) => {
                "Typing cancelled."
            }
            SessionCompletion::Finished(
                TerminalOutcome::TargetChanged
                | TerminalOutcome::TargetDisappeared
                | TerminalOutcome::TargetEvidenceUnavailable,
            ) => "Destination changed; remaining input was stopped.",
            SessionCompletion::Finished(TerminalOutcome::ClipboardChanged) => {
                "Clipboard changed before paste; nothing further was sent."
            }
            SessionCompletion::Finished(
                TerminalOutcome::ModifierConflict | TerminalOutcome::ModifierSettleTimeout,
            )
            | SessionCompletion::PreparationFailed(PreparationFailure::ModifierSettleTimeout) => {
                "Modifier keys prevented safe input."
            }
            SessionCompletion::Finished(TerminalOutcome::KnownSecurityRestriction)
            | SessionCompletion::PreparationFailed(PreparationFailure::KnownSecurityRestriction) => {
                "macOS denied the input operation. Check Accessibility permission."
            }
            _ => "The input operation stopped safely.",
        };
        let _ = ui.set_status(message);
    }

    const fn availability_message(availability: HotkeyAvailability) -> &'static str {
        match availability {
            HotkeyAvailability::Available => {
                "The operating system accepted both shortcuts. App-local conflicts cannot be fully verified."
            }
            HotkeyAvailability::Conflict => {
                "One or both shortcuts are already registered globally."
            }
            HotkeyAvailability::Reserved => "One or both shortcuts are reserved by macOS.",
            HotkeyAvailability::Unsupported => "One or both shortcuts are unsupported on macOS.",
            HotkeyAvailability::Unknown => "Shortcut availability cannot be fully verified.",
        }
    }

    const fn startup_enabled(status: MacStartupStatus) -> bool {
        matches!(
            status,
            MacStartupStatus::Enabled | MacStartupStatus::RequiresApproval
        )
    }

    fn settings_path() -> Result<PathBuf, HostError> {
        let home = env::var_os("HOME").ok_or(HostError::SettingsPath)?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("ClipType")
            .join("config.toml"))
    }
}
