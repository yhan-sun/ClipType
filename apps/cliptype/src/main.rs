#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! Windows product composition root for ClipType.

#[cfg(not(windows))]
fn main() {
    eprintln!("cliptype status=unsupported_platform platform=windows_required");
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_host::run() {
        eprintln!("cliptype status=fatal category={}", error.label());
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod windows_host {
    use std::{
        env, io,
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver, RecvTimeoutError},
        },
        thread::{self, JoinHandle},
        time::Duration,
    };

    use cliptype_app::{
        CancelResult, Coordinator, SessionCompletion, SettingsSource, SettingsStore,
        ShutdownResult, StatusSnapshot, TriggerResult,
    };
    use cliptype_core::{
        HotkeyApplyResult, HotkeyAvailability, HotkeyPlatform, InjectionBackend,
        PreparationFailure, ProductSettings, TerminalOutcome,
    };
    use cliptype_platform::{AccessibilityPermissionState, HotkeyControlPort};
    use cliptype_ui::{SettingsUiEvent, SettingsUiPlatform, SettingsUiSignal, SettingsUiThread};
    use cliptype_windows::{
        TrayEvent, TrayNotice, WindowsClipboard, WindowsCommandEvent, WindowsCommandSignal,
        WindowsCommandSource, WindowsKeyboard, WindowsPaste, WindowsStartup, WindowsTarget,
        WindowsTrayHandle, WindowsTraySignal,
    };

    const STATUS_POLL_INTERVAL: Duration = Duration::from_millis(25);
    const CONTROL_POLL_INTERVAL: Duration = Duration::from_millis(20);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HostError {
        SettingsPath,
        SettingsLoad,
        SettingsSave,
        InvalidConfiguration,
        CommandRegistration,
        CommandLoop,
        InputThreadStart,
        StatusThreadStart,
        TrayStart,
        SettingsUiStart,
        ProductHandlerStart,
        WorkerShutdownTimeout,
        CommandTeardown,
    }

    impl HostError {
        pub const fn label(self) -> &'static str {
            match self {
                Self::SettingsPath => "settings_path",
                Self::SettingsLoad => "settings_load",
                Self::SettingsSave => "settings_save",
                Self::InvalidConfiguration => "invalid_configuration",
                Self::CommandRegistration => "command_registration",
                Self::CommandLoop => "command_loop",
                Self::InputThreadStart => "input_thread_start",
                Self::StatusThreadStart => "status_thread_start",
                Self::TrayStart => "tray_start",
                Self::SettingsUiStart => "settings_ui_start",
                Self::ProductHandlerStart => "product_handler_start",
                Self::WorkerShutdownTimeout => "worker_shutdown_timeout",
                Self::CommandTeardown => "command_teardown",
            }
        }
    }

    pub fn run() -> Result<(), HostError> {
        let options = HostOptions::from_environment();
        let settings_store = SettingsStore::new(settings_path()?);
        let loaded = settings_store.load().map_err(|_| HostError::SettingsLoad)?;
        let settings = loaded.settings;
        if loaded.source != SettingsSource::Primary {
            settings_store
                .save(settings)
                .map_err(|_| HostError::SettingsSave)?;
        }

        let executable = env::current_exe().map_err(|_| HostError::SettingsPath)?;
        let startup = WindowsStartup::new();
        if startup
            .set_enabled(&executable, settings.start_at_login)
            .is_err()
        {
            eprintln!("cliptype status=warning category=startup_configuration");
        }

        let keyboard = WindowsKeyboard::new();
        let coordinator = Arc::new(
            Coordinator::new_product(
                WindowsClipboard::new(),
                WindowsTarget::new(),
                keyboard,
                keyboard,
                WindowsPaste::new(),
                settings
                    .runtime_config()
                    .map_err(|_| HostError::InvalidConfiguration)?,
            )
            .map_err(|_| HostError::InvalidConfiguration)?,
        );

        settings
            .hotkeys
            .validate_for(HotkeyPlatform::Windows)
            .map_err(|_| HostError::InvalidConfiguration)?;
        let mut commands = WindowsCommandSource::with_pair(settings.hotkeys);
        commands
            .register_commands()
            .map_err(|_| HostError::CommandRegistration)?;
        let command_signal = commands.signal();

        let (mut settings_ui, ui_events, ui_signal) = if options.headless {
            (None, None, None)
        } else {
            let (event_tx, event_rx) = mpsc::channel();
            let ui = SettingsUiThread::spawn(
                settings,
                SettingsUiPlatform::Windows,
                AccessibilityPermissionState::NotRequired,
                event_tx,
            )
            .map_err(|_| HostError::SettingsUiStart)?;
            let signal = ui.signal();
            if options.show_settings {
                let _ = signal.show();
            }
            (Some(ui), Some(event_rx), Some(signal))
        };

        let (mut tray, tray_events, tray_signal) = if options.headless {
            (None, None, None)
        } else {
            let (event_tx, event_rx) = mpsc::channel();
            let tray =
                WindowsTrayHandle::spawn(settings, event_tx).map_err(|_| HostError::TrayStart)?;
            let signal = tray.signal();
            (Some(tray), Some(event_rx), Some(signal))
        };

        let stop_handler = Arc::new(AtomicBool::new(false));
        let product_handler =
            if let (Some(tray_events), Some(ui_events), Some(tray_signal), Some(ui_signal)) = (
                tray_events,
                ui_events,
                tray_signal.clone(),
                ui_signal.clone(),
            ) {
                Some(
                    spawn_product_handler(
                        tray_events,
                        ui_events,
                        Arc::clone(&coordinator),
                        settings_store.clone(),
                        startup,
                        executable,
                        command_signal.clone(),
                        tray_signal,
                        ui_signal,
                        settings,
                        Arc::clone(&stop_handler),
                    )
                    .map_err(|_| HostError::ProductHandlerStart)?,
                )
            } else {
                None
            };

        let stop_monitor = Arc::new(AtomicBool::new(false));
        let monitor = spawn_status_monitor(
            Arc::clone(&coordinator),
            Arc::clone(&stop_monitor),
            tray_signal.clone(),
        )
        .map_err(|_| HostError::StatusThreadStart)?;

        if !options.background && spawn_stdin_shutdown(command_signal.clone()).is_err() {
            stop_monitor.store(true, Ordering::Release);
            stop_handler.store(true, Ordering::Release);
            let _ = monitor.join();
            if let Some(ui) = settings_ui.as_mut() {
                let _ = ui.shutdown();
            }
            if let Some(tray) = tray.as_mut() {
                let _ = tray.shutdown();
            }
            let _ = commands.unregister_commands();
            return Err(HostError::InputThreadStart);
        }

        println!(
            "cliptype status=ready mode={:?} enabled={} cps={} jitter_percent={} typo_percent={} trigger={} cancel={} tray={} settings_ui={} settings_source={:?}",
            settings.mode,
            settings.enabled,
            settings.characters_per_second,
            settings.jitter_percent,
            settings.typo_probability_percent,
            commands.trigger_hotkey(),
            commands.cancel_hotkey(),
            !options.headless,
            !options.headless,
            loaded.source,
        );

        let loop_result = command_loop(&mut commands, &coordinator, tray_signal.as_ref());
        let shutdown_result = coordinator.shutdown();
        stop_monitor.store(true, Ordering::Release);
        stop_handler.store(true, Ordering::Release);
        let _ = monitor.join();
        if let Some(ui) = settings_ui.as_mut() {
            let _ = ui.shutdown();
        }
        if let Some(tray) = tray.as_mut() {
            let _ = tray.shutdown();
        }
        if let Some(handler) = product_handler {
            let _ = handler.join();
        }
        let unregister_result = commands.unregister_commands();

        loop_result?;
        if shutdown_result == ShutdownResult::TimedOut {
            return Err(HostError::WorkerShutdownTimeout);
        }
        unregister_result.map_err(|_| HostError::CommandTeardown)?;

        println!("cliptype status=stopped");
        Ok(())
    }

    fn command_loop(
        commands: &mut WindowsCommandSource,
        coordinator: &Coordinator,
        tray: Option<&WindowsTraySignal>,
    ) -> Result<(), HostError> {
        loop {
            let event = commands
                .wait_for_command()
                .map_err(|_| HostError::CommandLoop)?;
            match event {
                WindowsCommandEvent::Trigger => {
                    let result = coordinator.trigger();
                    println!("cliptype event=trigger result={result:?}");
                    notify_trigger_result(tray, result);
                }
                WindowsCommandEvent::Cancel => {
                    let result = coordinator.cancel();
                    println!("cliptype event=cancel result={result:?}");
                    if result == CancelResult::Idle {
                        notify(tray, TrayNotice::Cancelled);
                    }
                }
                WindowsCommandEvent::Shutdown => return Ok(()),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_product_handler(
        tray_events: Receiver<TrayEvent>,
        ui_events: Receiver<SettingsUiEvent>,
        coordinator: Arc<Coordinator>,
        store: SettingsStore,
        startup: WindowsStartup,
        executable: PathBuf,
        commands: WindowsCommandSignal,
        tray: WindowsTraySignal,
        ui: SettingsUiSignal,
        initial: ProductSettings,
        stop: Arc<AtomicBool>,
    ) -> io::Result<JoinHandle<()>> {
        thread::Builder::new()
            .name("cliptype-product-handler".to_owned())
            .spawn(move || {
                let mut current = initial;
                while !stop.load(Ordering::Acquire) {
                    match tray_events.recv_timeout(CONTROL_POLL_INTERVAL) {
                        Ok(event) => {
                            if handle_tray_event(
                                event,
                                &coordinator,
                                &store,
                                &startup,
                                &executable,
                                &commands,
                                &tray,
                                &ui,
                                &mut current,
                            ) {
                                break;
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                    while let Ok(event) = ui_events.try_recv() {
                        handle_ui_event(
                            event,
                            &coordinator,
                            &store,
                            &startup,
                            &executable,
                            &commands,
                            &tray,
                            &ui,
                            &mut current,
                        );
                    }
                }
            })
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_tray_event(
        event: TrayEvent,
        coordinator: &Coordinator,
        store: &SettingsStore,
        startup: &WindowsStartup,
        executable: &Path,
        commands: &WindowsCommandSignal,
        tray: &WindowsTraySignal,
        ui: &SettingsUiSignal,
        current: &mut ProductSettings,
    ) -> bool {
        match event {
            TrayEvent::Trigger => {
                let result = coordinator.trigger();
                println!("cliptype event=tray_trigger result={result:?}");
                notify_trigger_result(Some(tray), result);
            }
            TrayEvent::Cancel => {
                let result = coordinator.cancel();
                println!("cliptype event=tray_cancel result={result:?}");
            }
            TrayEvent::OpenSettings => {
                let _ = ui.update_settings(*current);
                let _ = ui.show();
            }
            TrayEvent::SettingsChanged(proposed) => {
                let _ = apply_settings(
                    coordinator,
                    store,
                    startup,
                    executable,
                    commands,
                    tray,
                    ui,
                    current,
                    proposed,
                );
            }
            TrayEvent::Quit => {
                let _ = commands.request_shutdown();
                return true;
            }
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_ui_event(
        event: SettingsUiEvent,
        coordinator: &Coordinator,
        store: &SettingsStore,
        startup: &WindowsStartup,
        executable: &Path,
        commands: &WindowsCommandSignal,
        tray: &WindowsTraySignal,
        ui: &SettingsUiSignal,
        current: &mut ProductSettings,
    ) {
        match event {
            SettingsUiEvent::Apply(proposed) => {
                let _ = apply_settings(
                    coordinator,
                    store,
                    startup,
                    executable,
                    commands,
                    tray,
                    ui,
                    current,
                    proposed,
                );
            }
            SettingsUiEvent::Probe(pair) => match commands.probe_pair(pair) {
                Ok(availability) => {
                    let _ = ui.update_probe(availability, availability);
                    let _ = ui.set_status(match availability {
                        HotkeyAvailability::Available => {
                            "The operating system accepted both temporary registrations."
                        }
                        HotkeyAvailability::Conflict => "One or both shortcuts are already in use.",
                        HotkeyAvailability::Reserved => "One or both shortcuts are reserved.",
                        HotkeyAvailability::Unsupported => "One or both shortcuts are unsupported.",
                        HotkeyAvailability::Unknown => {
                            "The operating system could not fully verify this pair."
                        }
                    });
                }
                Err(_) => {
                    let _ =
                        ui.update_probe(HotkeyAvailability::Unknown, HotkeyAvailability::Unknown);
                    let _ = ui.set_status("Shortcut availability could not be checked.");
                }
            },
            SettingsUiEvent::Reset(defaults) => {
                let _ = ui.update_settings(defaults);
                let _ = ui.set_status("Defaults loaded. Choose Apply to save them.");
            }
            SettingsUiEvent::RequestPermission | SettingsUiEvent::OpenPermissionSettings => {
                let _ = ui.set_status("Accessibility permission is not required on Windows.");
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_settings(
        coordinator: &Coordinator,
        store: &SettingsStore,
        startup: &WindowsStartup,
        executable: &Path,
        commands: &WindowsCommandSignal,
        tray: &WindowsTraySignal,
        ui: &SettingsUiSignal,
        current: &mut ProductSettings,
        proposed: ProductSettings,
    ) -> bool {
        let Ok(proposed) = proposed.validate() else {
            settings_failure(tray, ui, "One or more settings are invalid.");
            return false;
        };
        if proposed
            .hotkeys
            .validate_for(HotkeyPlatform::Windows)
            .is_err()
        {
            settings_failure(tray, ui, "The shortcut pair is reserved or unsupported.");
            return false;
        }
        let Ok(runtime) = proposed.runtime_config() else {
            settings_failure(tray, ui, "The runtime settings are invalid.");
            return false;
        };
        let old_runtime = match current.runtime_config() {
            Ok(value) => value,
            Err(_) => {
                settings_failure(tray, ui, "The current runtime settings are invalid.");
                return false;
            }
        };

        let hotkeys_changed = proposed.hotkeys != current.hotkeys;
        if hotkeys_changed {
            match commands.replace_pair(proposed.hotkeys) {
                Ok(HotkeyApplyResult::Applied) => {}
                Ok(HotkeyApplyResult::Rejected(availability))
                | Ok(HotkeyApplyResult::RolledBack(availability)) => {
                    let _ = ui.update_probe(availability, availability);
                    settings_failure(
                        tray,
                        ui,
                        "The new shortcuts could not be applied; the old pair remains active.",
                    );
                    return false;
                }
                Err(_) => {
                    let _ =
                        ui.update_probe(HotkeyAvailability::Unknown, HotkeyAvailability::Unknown);
                    settings_failure(tray, ui, "The shortcut owner did not accept the update.");
                    return false;
                }
            }
        }

        let startup_changed = proposed.start_at_login != current.start_at_login;
        if startup_changed
            && startup
                .set_enabled(executable, proposed.start_at_login)
                .is_err()
        {
            if hotkeys_changed {
                let _ = commands.replace_pair(current.hotkeys);
            }
            settings_failure(tray, ui, "Start at login could not be changed.");
            notify(Some(tray), TrayNotice::StartupFailed);
            return false;
        }

        if coordinator.update_config(runtime).is_err() {
            if startup_changed {
                let _ = startup.set_enabled(executable, current.start_at_login);
            }
            if hotkeys_changed {
                let _ = commands.replace_pair(current.hotkeys);
            }
            settings_failure(tray, ui, "The running configuration could not be updated.");
            return false;
        }

        if store.save(proposed).is_err() {
            let _ = coordinator.update_config(old_runtime);
            if startup_changed {
                let _ = startup.set_enabled(executable, current.start_at_login);
            }
            if hotkeys_changed {
                let _ = commands.replace_pair(current.hotkeys);
            }
            settings_failure(
                tray,
                ui,
                "Settings could not be saved; the previous configuration was restored.",
            );
            return false;
        }

        *current = proposed;
        tray.update_settings(proposed);
        let _ = ui.update_settings(proposed);
        let _ = ui.update_probe(HotkeyAvailability::Available, HotkeyAvailability::Available);
        let _ = ui.set_status("Settings applied.");
        notify(Some(tray), TrayNotice::SettingsSaved);
        println!(
            "cliptype event=settings_saved mode={:?} enabled={} speed={:?} cps={} jitter_percent={} typo_percent={} startup={} hotkeys_live_updated={}",
            proposed.mode,
            proposed.enabled,
            proposed.speed,
            proposed.characters_per_second,
            proposed.jitter_percent,
            proposed.typo_probability_percent,
            proposed.start_at_login,
            hotkeys_changed,
        );
        true
    }

    fn settings_failure(tray: &WindowsTraySignal, ui: &SettingsUiSignal, message: &'static str) {
        notify(Some(tray), TrayNotice::SettingsFailed);
        let _ = ui.set_status(message);
    }

    fn spawn_stdin_shutdown(signal: WindowsCommandSignal) -> io::Result<JoinHandle<()>> {
        thread::Builder::new()
            .name("cliptype-console-shutdown".to_owned())
            .spawn(move || {
                let mut line = String::new();
                let _ = io::stdin().read_line(&mut line);
                let _ = signal.request_shutdown();
            })
    }

    fn spawn_status_monitor(
        coordinator: Arc<Coordinator>,
        stop: Arc<AtomicBool>,
        tray: Option<WindowsTraySignal>,
    ) -> io::Result<JoinHandle<()>> {
        thread::Builder::new()
            .name("cliptype-status".to_owned())
            .spawn(move || {
                let mut previous: Option<StatusSnapshot> = None;
                while !stop.load(Ordering::Acquire) {
                    let current = coordinator.status();
                    if previous != Some(current) {
                        println!(
                            "cliptype status=update generation={} phase={:?} backend={:?} batches={} completion={:?}",
                            current.generation,
                            current.phase,
                            current.backend,
                            current.batches_completed,
                            current.completion,
                        );
                        if previous.and_then(|value| value.completion) != current.completion
                            && let Some(completion) = current.completion
                        {
                            notify_completion(tray.as_ref(), current.backend, completion);
                        }
                        previous = Some(current);
                    }
                    thread::sleep(STATUS_POLL_INTERVAL);
                }
            })
    }

    fn notify_trigger_result(tray: Option<&WindowsTraySignal>, result: TriggerResult) {
        match result {
            TriggerResult::Busy => notify(tray, TrayNotice::Busy),
            TriggerResult::Rejected(PreparationFailure::ClipboardUnavailable)
            | TriggerResult::Rejected(PreparationFailure::ClipboardEmpty)
            | TriggerResult::Rejected(PreparationFailure::ClipboardNonText)
            | TriggerResult::Rejected(PreparationFailure::ClipboardMalformed) => {
                notify(tray, TrayNotice::ClipboardUnavailable);
            }
            TriggerResult::StartFailed => notify(tray, TrayNotice::NativeFailure),
            TriggerResult::Started { .. }
            | TriggerResult::ShuttingDown
            | TriggerResult::Rejected(_) => {}
        }
    }

    fn notify_completion(
        tray: Option<&WindowsTraySignal>,
        backend: Option<InjectionBackend>,
        completion: SessionCompletion,
    ) {
        let notice = match completion {
            SessionCompletion::Finished(TerminalOutcome::Completed) => {
                backend.map(TrayNotice::Completed)
            }
            SessionCompletion::Finished(TerminalOutcome::Cancelled)
            | SessionCompletion::PreparationFailed(PreparationFailure::Cancelled) => {
                Some(TrayNotice::Cancelled)
            }
            SessionCompletion::Finished(TerminalOutcome::ClipboardChanged) => {
                Some(TrayNotice::ClipboardChanged)
            }
            SessionCompletion::Finished(
                TerminalOutcome::TargetChanged
                | TerminalOutcome::TargetDisappeared
                | TerminalOutcome::TargetEvidenceUnavailable,
            ) => Some(TrayNotice::TargetChanged),
            SessionCompletion::Finished(
                TerminalOutcome::ModifierConflict | TerminalOutcome::ModifierSettleTimeout,
            )
            | SessionCompletion::PreparationFailed(PreparationFailure::ModifierSettleTimeout) => {
                Some(TrayNotice::ModifierConflict)
            }
            SessionCompletion::Finished(TerminalOutcome::KnownSecurityRestriction)
            | SessionCompletion::PreparationFailed(PreparationFailure::KnownSecurityRestriction) => {
                Some(TrayNotice::SecurityRestriction)
            }
            SessionCompletion::PreparationFailed(
                PreparationFailure::ClipboardUnavailable
                | PreparationFailure::ClipboardRevisionUnavailable
                | PreparationFailure::ClipboardEmpty
                | PreparationFailure::ClipboardNonText
                | PreparationFailure::ClipboardMalformed,
            ) => Some(TrayNotice::ClipboardUnavailable),
            SessionCompletion::Finished(
                TerminalOutcome::BlockedCauseUnknown
                | TerminalOutcome::PartialInput
                | TerminalOutcome::ProgressUnknown
                | TerminalOutcome::NativeFailure
                | TerminalOutcome::InternalInvariant,
            )
            | SessionCompletion::PreparationFailed(_) => Some(TrayNotice::NativeFailure),
        };
        if let Some(notice) = notice {
            notify(tray, notice);
        }
    }

    fn notify(tray: Option<&WindowsTraySignal>, notice: TrayNotice) {
        if let Some(tray) = tray {
            let _ = tray.notify(notice);
        }
    }

    fn settings_path() -> Result<PathBuf, HostError> {
        let root = env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .ok_or(HostError::SettingsPath)?;
        Ok(PathBuf::from(root).join("ClipType").join("config.toml"))
    }

    struct HostOptions {
        background: bool,
        headless: bool,
        show_settings: bool,
    }

    impl HostOptions {
        fn from_environment() -> Self {
            let mut background = false;
            let mut headless =
                env::var_os("CLIPTYPE_HEADLESS").as_deref() == Some(std::ffi::OsStr::new("1"));
            let mut show_settings = false;
            for argument in env::args_os().skip(1) {
                if argument == "--background" {
                    background = true;
                } else if argument == "--headless" {
                    headless = true;
                } else if argument == "--settings" {
                    show_settings = true;
                }
            }
            Self {
                background,
                headless,
                show_settings,
            }
        }
    }
}
