//! Controlled end-to-end evidence for all P2 injection modes.
//!
//! The harness is explicitly opt-in because it writes generated test fixtures
//! to the current clipboard. It never prints fixture or target content.

#[cfg(not(windows))]
fn main() {
    println!("p2_controlled_e2e result=skipped reason=windows_required");
}

#[cfg(windows)]
fn main() {
    match windows_e2e::run_all() {
        Ok(observations) => {
            for observation in observations {
                println!(
                    "p2_controlled_e2e result=ok case={} backend={:?} expected_utf16_units={} observed_utf16_units={} batches={} clipboard_unchanged={}",
                    observation.case,
                    observation.backend,
                    observation.expected_utf16_units,
                    observation.observed_utf16_units,
                    observation.batches,
                    observation.clipboard_unchanged,
                );
            }
        }
        Err(error) => {
            eprintln!(
                "p2_controlled_e2e result=failed category={} expected_utf16_units={} observed_utf16_units={}",
                error.label(),
                error.expected_units(),
                error.observed_units(),
            );
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
mod windows_e2e {
    use std::{
        env,
        mem::size_of,
        ptr::{copy_nonoverlapping, null, null_mut},
        thread,
        time::{Duration, Instant},
    };

    use cliptype_app::{Coordinator, SessionCompletion, TriggerResult, WaitResult};
    use cliptype_core::{
        AutoClipboardThreshold, InjectionBackend, InjectionMode, NativeByteLimit, P1Config,
        ProductConfig, TerminalOutcome,
    };
    use cliptype_platform::ClipboardPort;
    use cliptype_windows::{WindowsClipboard, WindowsKeyboard, WindowsPaste, WindowsTarget};
    use windows_sys::Win32::{
        Foundation::{GlobalFree, HGLOBAL, HWND},
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, DispatchMessageW, GetForegroundWindow,
            GetWindowTextLengthW, GetWindowTextW, MSG, PeekMessageW, SetForegroundWindow,
            ShowWindow, TranslateMessage,
        },
    };

    #[link(name = "user32")]
    unsafe extern "system" {
        #[link_name = "GetFocus"]
        fn get_focus() -> HWND;

        #[link_name = "SetFocus"]
        fn set_focus(window: HWND) -> HWND;

        #[link_name = "UpdateWindow"]
        fn update_window(window: HWND) -> i32;
    }

    const OPT_IN: &str = "CLIPTYPE_RUN_P2_CONTROLLED_E2E";
    const SENTINEL: &str = "CLIPTYPE_P2_E2E_PRIVATE_SENTINEL_9265";

    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;
    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_VSCROLL: u32 = 0x0020_0000;
    const WS_TABSTOP: u32 = 0x0001_0000;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_WANTRETURN: u32 = 0x1000;
    const CW_USEDEFAULT: i32 = i32::MIN;
    const SW_SHOW: i32 = 5;
    const PM_REMOVE: u32 = 0x0001;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Observation {
        pub case: &'static str,
        pub backend: InjectionBackend,
        pub expected_utf16_units: usize,
        pub observed_utf16_units: usize,
        pub batches: u32,
        pub clipboard_unchanged: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum E2eError {
        ExplicitOptInRequired,
        WindowCreation,
        ForegroundFocus,
        ClipboardAllocation,
        ClipboardOpen,
        ClipboardWrite,
        ClipboardRevision,
        ClipboardRead,
        ClipboardChanged,
        CoordinatorConfiguration,
        TriggerRejected,
        CoordinatorTimeout,
        CoordinatorOutcome,
        UnexpectedBackend,
        TargetRead,
        TextMismatch { expected: usize, observed: usize },
    }

    impl E2eError {
        pub const fn label(self) -> &'static str {
            match self {
                Self::ExplicitOptInRequired => "explicit_opt_in_required",
                Self::WindowCreation => "window_creation",
                Self::ForegroundFocus => "foreground_focus",
                Self::ClipboardAllocation => "clipboard_allocation",
                Self::ClipboardOpen => "clipboard_open",
                Self::ClipboardWrite => "clipboard_write",
                Self::ClipboardRevision => "clipboard_revision",
                Self::ClipboardRead => "clipboard_read",
                Self::ClipboardChanged => "clipboard_changed",
                Self::CoordinatorConfiguration => "coordinator_configuration",
                Self::TriggerRejected => "trigger_rejected",
                Self::CoordinatorTimeout => "coordinator_timeout",
                Self::CoordinatorOutcome => "coordinator_outcome",
                Self::UnexpectedBackend => "unexpected_backend",
                Self::TargetRead => "target_read",
                Self::TextMismatch { .. } => "text_mismatch",
            }
        }

        pub const fn expected_units(self) -> usize {
            match self {
                Self::TextMismatch { expected, .. } => expected,
                _ => 0,
            }
        }

        pub const fn observed_units(self) -> usize {
            match self {
                Self::TextMismatch { observed, .. } => observed,
                _ => 0,
            }
        }
    }

    struct Case {
        name: &'static str,
        mode: InjectionMode,
        threshold: usize,
        source: String,
        expected: String,
        backend: InjectionBackend,
    }

    pub fn run_all() -> Result<Vec<Observation>, E2eError> {
        if env::var_os(OPT_IN).as_deref() != Some(std::ffi::OsStr::new("1")) {
            return Err(E2eError::ExplicitOptInRequired);
        }

        let keyboard_source = format!("{SENTINEL}|keyboard|A中😀e\u{301}|line-one\nline-two");
        let keyboard_expected = keyboard_source.replace('\n', "\r\n");
        let clipboard_source = format!("{SENTINEL}|clipboard|A中😀e\u{301}|line-one\r\nline-two");
        let auto_short = format!("{SENTINEL}|auto-short");
        let auto_long = format!("{SENTINEL}|auto-long|{}", "0123456789abcdef".repeat(32));

        let cases = [
            Case {
                name: "keyboard",
                mode: InjectionMode::Keyboard,
                threshold: 256,
                source: keyboard_source,
                expected: keyboard_expected,
                backend: InjectionBackend::Keyboard,
            },
            Case {
                name: "clipboard",
                mode: InjectionMode::Clipboard,
                threshold: 256,
                source: clipboard_source.clone(),
                expected: clipboard_source,
                backend: InjectionBackend::Clipboard,
            },
            Case {
                name: "auto-short",
                mode: InjectionMode::Auto,
                threshold: 256,
                source: auto_short.clone(),
                expected: auto_short,
                backend: InjectionBackend::Keyboard,
            },
            Case {
                name: "auto-long",
                mode: InjectionMode::Auto,
                threshold: 16,
                source: auto_long.clone(),
                expected: auto_long,
                backend: InjectionBackend::Clipboard,
            },
        ];

        cases.into_iter().map(run_case).collect()
    }

    fn run_case(case: Case) -> Result<Observation, E2eError> {
        let window = ControlledWindow::create()?;
        window.activate()?;
        let _clipboard = ClipboardFixture::install(&case.source)?;
        let clipboard = WindowsClipboard::new();
        let revision_before = clipboard.current_revision();
        if !revision_before.is_known() {
            return Err(E2eError::ClipboardRevision);
        }

        let keyboard = WindowsKeyboard::new();
        let coordinator = Coordinator::new_product(
            clipboard,
            WindowsTarget::new(),
            keyboard,
            keyboard,
            WindowsPaste::new(),
            ProductConfig {
                enabled: true,
                mode: case.mode,
                auto_clipboard_threshold: AutoClipboardThreshold::new(case.threshold)
                    .map_err(|_| E2eError::CoordinatorConfiguration)?,
                safety: P1Config {
                    keyboard_interval: Duration::from_millis(1),
                    ..P1Config::default()
                },
            },
        )
        .map_err(|_| E2eError::CoordinatorConfiguration)?;

        if !matches!(coordinator.trigger(), TriggerResult::Started { .. }) {
            return Err(E2eError::TriggerRejected);
        }
        if coordinator.wait_for_idle(Duration::from_secs(8)) != WaitResult::Idle {
            return Err(E2eError::CoordinatorTimeout);
        }

        let status = coordinator.status();
        if status.completion != Some(SessionCompletion::Finished(TerminalOutcome::Completed)) {
            return Err(E2eError::CoordinatorOutcome);
        }
        if status.backend != Some(case.backend) {
            return Err(E2eError::UnexpectedBackend);
        }

        let observed = wait_for_expected_text(window.edit, &case.expected)?;
        let expected_units = case.expected.encode_utf16().count();
        let observed_units = observed.encode_utf16().count();
        if observed != case.expected {
            return Err(E2eError::TextMismatch {
                expected: expected_units,
                observed: observed_units,
            });
        }

        let current = WindowsClipboard::new()
            .read_current_text(NativeByteLimit::new(2 * 1024 * 1024).expect("test limit"))
            .map_err(|_| E2eError::ClipboardRead)?;
        if current.expose() != case.source {
            return Err(E2eError::ClipboardChanged);
        }
        let revision_after = WindowsClipboard::new().current_revision();
        let clipboard_unchanged = revision_before.matches(revision_after);
        if !clipboard_unchanged {
            return Err(E2eError::ClipboardChanged);
        }

        Ok(Observation {
            case: case.name,
            backend: case.backend,
            expected_utf16_units: expected_units,
            observed_utf16_units: observed_units,
            batches: status.batches_completed,
            clipboard_unchanged,
        })
    }

    struct ControlledWindow {
        parent: HWND,
        edit: HWND,
    }

    impl ControlledWindow {
        fn create() -> Result<Self, E2eError> {
            let parent_class = wide("STATIC");
            let edit_class = wide("EDIT");
            let title = wide("ClipType P2 Controlled Target");
            let empty = [0_u16];

            // SAFETY: all strings are nul-terminated for the duration of the
            // calls; built-in window classes require no custom class lifetime.
            let parent = unsafe {
                CreateWindowExW(
                    0,
                    parent_class.as_ptr(),
                    title.as_ptr(),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    640,
                    280,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null(),
                )
            };
            if parent.is_null() {
                return Err(E2eError::WindowCreation);
            }

            // SAFETY: `parent` is live, the class/name buffers are valid, and
            // ownership remains with the parent window after creation.
            let edit = unsafe {
                CreateWindowExW(
                    0,
                    edit_class.as_ptr(),
                    empty.as_ptr(),
                    WS_CHILD
                        | WS_VISIBLE
                        | WS_VSCROLL
                        | WS_TABSTOP
                        | ES_MULTILINE
                        | ES_AUTOVSCROLL
                        | ES_WANTRETURN,
                    12,
                    12,
                    600,
                    220,
                    parent,
                    null_mut(),
                    null_mut(),
                    null(),
                )
            };
            if edit.is_null() {
                // SAFETY: `parent` was created here and remains owned here.
                let _ = unsafe { DestroyWindow(parent) };
                return Err(E2eError::WindowCreation);
            }

            Ok(Self { parent, edit })
        }

        fn activate(&self) -> Result<(), E2eError> {
            for _ in 0..60 {
                // `ShowWindow` reports prior visibility, not success.
                let _ = unsafe { ShowWindow(self.parent, SW_SHOW) };
                // SAFETY: handles are live and owned by this guard/process.
                let _ = unsafe { update_window(self.parent) };
                let _ = unsafe { SetForegroundWindow(self.parent) };
                let _ = unsafe { set_focus(self.edit) };
                pump_messages();

                // SAFETY: query-only opaque-handle calls.
                let foreground = unsafe { GetForegroundWindow() };
                let focus = unsafe { get_focus() };
                if foreground == self.parent && focus == self.edit {
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(E2eError::ForegroundFocus)
        }
    }

    impl Drop for ControlledWindow {
        fn drop(&mut self) {
            // SAFETY: destroying the owned parent also destroys the child.
            let _ = unsafe { DestroyWindow(self.parent) };
        }
    }

    struct ClipboardFixture;

    impl ClipboardFixture {
        fn install(value: &str) -> Result<Self, E2eError> {
            let units: Vec<u16> = value.encode_utf16().chain([0]).collect();
            let bytes = units
                .len()
                .checked_mul(size_of::<u16>())
                .ok_or(E2eError::ClipboardAllocation)?;

            // SAFETY: movable global allocation with checked size.
            let allocation = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) };
            if allocation.is_null() {
                return Err(E2eError::ClipboardAllocation);
            }
            let mut allocation = PendingGlobal::new(allocation);

            // SAFETY: allocation is live and large enough for the units.
            let pointer = unsafe { GlobalLock(allocation.handle) };
            if pointer.is_null() {
                return Err(E2eError::ClipboardAllocation);
            }
            // SAFETY: buffers are valid, non-overlapping and exactly sized.
            unsafe {
                copy_nonoverlapping(units.as_ptr(), pointer.cast::<u16>(), units.len());
                let _ = GlobalUnlock(allocation.handle);
            }

            // SAFETY: null owner and no user data inspection.
            if unsafe { OpenClipboard(null_mut()) } == 0 {
                return Err(E2eError::ClipboardOpen);
            }
            let opened = ClipboardOpenGuard;
            // SAFETY: clipboard is open on this thread.
            if unsafe { EmptyClipboard() } == 0 {
                return Err(E2eError::ClipboardWrite);
            }
            // SAFETY: ownership transfers only on success.
            if unsafe { SetClipboardData(CF_UNICODETEXT, allocation.handle) }.is_null() {
                return Err(E2eError::ClipboardWrite);
            }
            allocation.transferred = true;
            drop(opened);
            Ok(Self)
        }
    }

    impl Drop for ClipboardFixture {
        fn drop(&mut self) {
            // The opt-in test clears only its own generated fixture.
            if unsafe { OpenClipboard(null_mut()) } != 0 {
                let _ = unsafe { EmptyClipboard() };
                let _ = unsafe { CloseClipboard() };
            }
        }
    }

    struct PendingGlobal {
        handle: HGLOBAL,
        transferred: bool,
    }

    impl PendingGlobal {
        const fn new(handle: HGLOBAL) -> Self {
            Self {
                handle,
                transferred: false,
            }
        }
    }

    impl Drop for PendingGlobal {
        fn drop(&mut self) {
            if !self.transferred {
                // SAFETY: ownership has not transferred to the clipboard.
                let _ = unsafe { GlobalFree(self.handle) };
            }
        }
    }

    struct ClipboardOpenGuard;

    impl Drop for ClipboardOpenGuard {
        fn drop(&mut self) {
            // SAFETY: guard exists only after successful OpenClipboard.
            let _ = unsafe { CloseClipboard() };
        }
    }

    fn wait_for_expected_text(window: HWND, expected: &str) -> Result<String, E2eError> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            pump_messages();
            let observed = read_window_text(window)?;
            if observed == expected || Instant::now() >= deadline {
                return Ok(observed);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn read_window_text(window: HWND) -> Result<String, E2eError> {
        // SAFETY: window is the live edit handle owned by ControlledWindow.
        let length = unsafe { GetWindowTextLengthW(window) };
        if length < 0 {
            return Err(E2eError::TargetRead);
        }
        let capacity = usize::try_from(length)
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or(E2eError::TargetRead)?;
        let mut buffer = vec![0_u16; capacity];
        let count = i32::try_from(capacity).map_err(|_| E2eError::TargetRead)?;
        // SAFETY: buffer is writable for count UTF-16 units.
        let copied = unsafe { GetWindowTextW(window, buffer.as_mut_ptr(), count) };
        if copied < 0 {
            return Err(E2eError::TargetRead);
        }
        buffer.truncate(usize::try_from(copied).map_err(|_| E2eError::TargetRead)?);
        String::from_utf16(&buffer).map_err(|_| E2eError::TargetRead)
    }

    fn pump_messages() {
        let mut message = MSG::default();
        // SAFETY: each removed message initializes the buffer before dispatch.
        while unsafe { PeekMessageW(&raw mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
            let _ = unsafe { TranslateMessage(&raw const message) };
            let _ = unsafe { DispatchMessageW(&raw const message) };
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain([0]).collect()
    }
}
