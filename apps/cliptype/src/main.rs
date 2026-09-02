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
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver},
        },
        thread::{self, JoinHandle},
        time::Duration,
    };

    use cliptype_app::{
        CancelResult, Coordinator, SessionCompletion, SettingsSource, SettingsStore,
        ShutdownResult, StatusSnapshot, TriggerResult,
    };
    use cliptype_core::{InjectionBackend, PreparationFailure, ProductSettings, TerminalOutcome};
    use cliptype_windows::{
        TrayEvent, TrayNotice, WindowsClipboard, WindowsCommandEvent, WindowsCommandSignal,
        WindowsCommandSource, WindowsKeyboard, WindowsPaste, WindowsStartup, WindowsTarget,
        WindowsTrayHandle, WindowsTraySignal,
    };

    const STATUS_POLL_INTERVAL: Duration = Duration::from_millis(25);

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
        TrayHandlerStart,
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
                Self::TrayHandlerStart => "tray_handler_start",
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

        let mut commands = WindowsCommandSource::with_preset(settings.hotkey);
        commands
            .register_commands()
            .map_err(|_| HostError::CommandRegistration)?;
        let command_signal = commands.signal();

        let (mut tray, tray_handler, tray_signal) = if options.headless {
            (None, None, None)
        } else {
            let (event_tx, event_rx) = mpsc::channel();
            let tray =
                WindowsTrayHandle::spawn(settings, event_tx).map_err(|_| HostError::TrayStart)?;
            let signal = tray.signal();
            let handler = spawn_tray_handler(
                event_rx,
                Arc::clone(&coordinator),
                settings_store.clone(),
                startup,
                executable,
                command_signal,
                signal.clone(),
                settings,
            )
            .map_err(|_| HostError::TrayHandlerStart)?;
            (Some(tray), Some(handler), Some(signal))
        };

        let stop_monitor = Arc::new(AtomicBool::new(false));
        let monitor = spawn_status_monitor(
            Arc::clone(&coordinator),
            Arc::clone(&stop_monitor),
            tray_signal.clone(),
        )
        .map_err(|_| HostError::StatusThreadStart)?;

        if !options.background && spawn_stdin_shutdown(command_signal).is_err() {
            stop_monitor.store(true, Ordering::Release);
            let _ = monitor.join();
            if let Some(tray) = tray.as_mut() {
                let _ = tray.shutdown();
            }
            let _ = commands.unregister_commands();
            return Err(HostError::InputThreadStart);
        }

        println!(
            "cliptype status=ready mode={:?} enabled={} trigger={} cancel={} tray={} settings_source={:?}",
            settings.mode,
            settings.enabled,
            commands.trigger_hotkey(),
            commands.cancel_hotkey(),
            !options.headless,
            loaded.source,
        );

        let loop_result = command_loop(&mut commands, &coordinator, tray_signal.as_ref());
        let shutdown_result = coordinator.shutdown();
        stop_monitor.store(true, Ordering::Release);
        let _ = monitor.join();
        if let Some(tray) = tray.as_mut() {
            let _ = tray.shutdown();
        }
        if let Some(handler) = tray_handler {
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
    fn spawn_tray_handler(
        events: Receiver<TrayEvent>,
        coordinator: Arc<Coordinator>,
        store: SettingsStore,
        startup: WindowsStartup,
        executable: PathBuf,
        shutdown: WindowsCommandSignal,
        tray: WindowsTraySignal,
        initial: ProductSettings,
    ) -> io::Result<JoinHandle<()>> {
        thread::Builder::new()
            .name("cliptype-tray-handler".to_owned())
            .spawn(move || {
                let mut current = initial;
                while let Ok(event) = events.recv() {
                    match event {
                        TrayEvent::Trigger => {
                            let result = coordinator.trigger();
                            println!("cliptype event=tray_trigger result={result:?}");
                            notify_trigger_result(Some(&tray), result);
                        }
                        TrayEvent::Cancel => {
                            let result = coordinator.cancel();
                            println!("cliptype event=tray_cancel result={result:?}");
                        }
                        TrayEvent::SettingsChanged(proposed) => {
                            if apply_settings(
                                &coordinator,
                                &store,
                                &startup,
                                &executable,
                                &tray,
                                current,
                                proposed,
                            ) {
                                current = proposed;
                            }
                        }
                        TrayEvent::Quit => {
                            let _ = shutdown.request_shutdown();
                            break;
                        }
                    }
                }
            })
    }

    fn apply_settings(
        coordinator: &Coordinator,
        store: &SettingsStore,
        startup: &WindowsStartup,
        executable: &PathBuf,
        tray: &WindowsTraySignal,
        current: ProductSettings,
        proposed: ProductSettings,
    ) -> bool {
        let Ok(proposed) = proposed.validate() else {
            notify(Some(tray), TrayNotice::SettingsFailed);
            return false;
        };
        let startup_changed = proposed.start_at_login != current.start_at_login;
        if startup_changed
            && startup
                .set_enabled(executable, proposed.start_at_login)
                .is_err()
        {
            notify(Some(tray), TrayNotice::StartupFailed);
            return false;
        }
        if store.save(proposed).is_err() {
            if startup_changed {
                let _ = startup.set_enabled(executable, current.start_at_login);
            }
            notify(Some(tray), TrayNotice::SettingsFailed);
            return false;
        }
        let Ok(runtime) = proposed.runtime_config() else {
            notify(Some(tray), TrayNotice::SettingsFailed);
            return false;
        };
        if coordinator.update_config(runtime).is_err() {
            notify(Some(tray), TrayNotice::SettingsFailed);
            return false;
        }
        tray.update_settings(proposed);
        notify(Some(tray), TrayNotice::SettingsSaved);
        println!(
            "cliptype event=settings_saved mode={:?} enabled={} speed={:?} startup={} hotkey_restart_required={}",
            proposed.mode,
            proposed.enabled,
            proposed.speed,
            proposed.start_at_login,
            proposed.hotkey != current.hotkey,
        );
        true
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
                        if previous.and_then(|value| value.completion) != current.completion {
                            if let Some(completion) = current.completion {
                                notify_completion(tray.as_ref(), current.backend, completion);
                            }
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
    }

    impl HostOptions {
        fn from_environment() -> Self {
            let mut background = false;
            let mut headless =
                env::var_os("CLIPTYPE_HEADLESS").as_deref() == Some(std::ffi::OsStr::new("1"));
            for argument in env::args_os().skip(1) {
                if argument == "--background" {
                    background = true;
                } else if argument == "--headless" {
                    headless = true;
                }
            }
            Self {
                background,
                headless,
            }
        }
    }
}
