//! Minimal Windows composition root for the P1 development vertical slice.

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
        io,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread::{self, JoinHandle},
        time::Duration,
    };

    use cliptype_app::{Coordinator, ShutdownResult, StatusSnapshot};
    use cliptype_windows::{
        CANCEL_HOTKEY, TRIGGER_HOTKEY, WindowsClipboard, WindowsCommandEvent,
        WindowsCommandSignal, WindowsCommandSource, WindowsKeyboard, WindowsTarget,
    };

    const STATUS_POLL_INTERVAL: Duration = Duration::from_millis(25);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HostError {
        InvalidConfiguration,
        CommandRegistration,
        CommandLoop,
        InputThreadStart,
        StatusThreadStart,
        WorkerShutdownTimeout,
        CommandTeardown,
    }

    impl HostError {
        pub const fn label(self) -> &'static str {
            match self {
                Self::InvalidConfiguration => "invalid_configuration",
                Self::CommandRegistration => "command_registration",
                Self::CommandLoop => "command_loop",
                Self::InputThreadStart => "input_thread_start",
                Self::StatusThreadStart => "status_thread_start",
                Self::WorkerShutdownTimeout => "worker_shutdown_timeout",
                Self::CommandTeardown => "command_teardown",
            }
        }
    }

    pub fn run() -> Result<(), HostError> {
        let keyboard = WindowsKeyboard::new();
        let coordinator = Arc::new(
            Coordinator::new(
                WindowsClipboard::new(),
                WindowsTarget::new(),
                keyboard,
                keyboard,
                Default::default(),
            )
            .map_err(|_| HostError::InvalidConfiguration)?,
        );

        let mut commands = WindowsCommandSource::new();
        commands
            .register_commands()
            .map_err(|_| HostError::CommandRegistration)?;

        let stop_monitor = Arc::new(AtomicBool::new(false));
        let monitor = spawn_status_monitor(Arc::clone(&coordinator), Arc::clone(&stop_monitor))
            .map_err(|_| HostError::StatusThreadStart)?;
        if spawn_stdin_shutdown(commands.signal()).is_err() {
            stop_monitor.store(true, Ordering::Release);
            let _ = monitor.join();
            let _ = commands.unregister_commands();
            return Err(HostError::InputThreadStart);
        }

        println!(
            "cliptype status=ready trigger={TRIGGER_HOTKEY} cancel={CANCEL_HOTKEY} quit=Enter"
        );

        let loop_result = command_loop(&mut commands, &coordinator);
        let shutdown_result = coordinator.shutdown();
        stop_monitor.store(true, Ordering::Release);
        let _ = monitor.join();
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
    ) -> Result<(), HostError> {
        loop {
            let event = commands
                .wait_for_command()
                .map_err(|_| HostError::CommandLoop)?;
            match event {
                WindowsCommandEvent::Trigger => {
                    println!("cliptype event=trigger result={:?}", coordinator.trigger());
                }
                WindowsCommandEvent::Cancel => {
                    println!("cliptype event=cancel result={:?}", coordinator.cancel());
                }
                WindowsCommandEvent::Shutdown => return Ok(()),
            }
        }
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
    ) -> io::Result<JoinHandle<()>> {
        thread::Builder::new()
            .name("cliptype-status".to_owned())
            .spawn(move || {
                let mut previous: Option<StatusSnapshot> = None;
                while !stop.load(Ordering::Acquire) {
                    let current = coordinator.status();
                    if previous != Some(current) {
                        println!(
                            "cliptype status=update generation={} phase={:?} batches={} completion={:?}",
                            current.generation,
                            current.phase,
                            current.batches_completed,
                            current.completion
                        );
                        previous = Some(current);
                    }
                    thread::sleep(STATUS_POLL_INTERVAL);
                }
            })
    }
}
