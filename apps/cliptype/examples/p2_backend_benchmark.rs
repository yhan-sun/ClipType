//! Controlled benchmark for deriving the P2 automatic backend threshold.
//!
//! This opt-in harness measures generated non-secret fixtures against one
//! isolated native Win32 edit target at a time. It prints only element counts,
//! durations and backend categories.

#[cfg(not(windows))]
fn main() {
    println!("p2_backend_benchmark result=skipped reason=windows_required");
}

#[cfg(windows)]
fn main() {
    match benchmark::run() {
        Ok(result) => {
            for row in result.rows {
                println!(
                    "p2_backend_benchmark result=ok elements={} keyboard_us={} clipboard_us={} winner={:?}",
                    row.elements, row.keyboard_us, row.clipboard_us, row.winner,
                );
            }
            println!(
                "p2_backend_benchmark recommendation={} criterion=clipboard_at_least_20_percent_faster",
                result.recommended_threshold,
            );
        }
        Err(error) => {
            eprintln!(
                "p2_backend_benchmark result=failed category={}",
                error.label()
            );
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
mod benchmark {
    use std::{
        env,
        mem::size_of,
        ptr::{copy_nonoverlapping, null, null_mut},
        thread,
        time::{Duration, Instant},
    };

    use cliptype_app::{Coordinator, SessionCompletion, TriggerResult, WaitResult};
    use cliptype_core::{
        AutoClipboardThreshold, InjectionBackend, InjectionMode, P1Config, ProductConfig,
        TerminalOutcome,
    };
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

    const OPT_IN: &str = "CLIPTYPE_RUN_P2_BENCHMARK";
    const LENGTHS: [usize; 4] = [16, 64, 256, 1024];
    const REPETITIONS: usize = 3;
    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;
    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const CW_USEDEFAULT: i32 = i32::MIN;
    const SW_SHOW: i32 = 5;
    const PM_REMOVE: u32 = 0x0001;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Row {
        pub elements: usize,
        pub keyboard_us: u128,
        pub clipboard_us: u128,
        pub winner: InjectionBackend,
    }

    pub struct BenchmarkResult {
        pub rows: Vec<Row>,
        pub recommended_threshold: usize,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BenchmarkError {
        ExplicitOptInRequired,
        WindowCreation,
        ForegroundFocus,
        ClipboardAllocation,
        ClipboardOpen,
        ClipboardWrite,
        CoordinatorConfiguration,
        TriggerRejected,
        CoordinatorTimeout,
        CoordinatorOutcome,
        UnexpectedBackend,
        TargetRead,
        TextMismatch,
    }

    impl BenchmarkError {
        pub const fn label(self) -> &'static str {
            match self {
                Self::ExplicitOptInRequired => "explicit_opt_in_required",
                Self::WindowCreation => "window_creation",
                Self::ForegroundFocus => "foreground_focus",
                Self::ClipboardAllocation => "clipboard_allocation",
                Self::ClipboardOpen => "clipboard_open",
                Self::ClipboardWrite => "clipboard_write",
                Self::CoordinatorConfiguration => "coordinator_configuration",
                Self::TriggerRejected => "trigger_rejected",
                Self::CoordinatorTimeout => "coordinator_timeout",
                Self::CoordinatorOutcome => "coordinator_outcome",
                Self::UnexpectedBackend => "unexpected_backend",
                Self::TargetRead => "target_read",
                Self::TextMismatch => "text_mismatch",
            }
        }
    }

    pub fn run() -> Result<BenchmarkResult, BenchmarkError> {
        if env::var_os(OPT_IN).as_deref() != Some(std::ffi::OsStr::new("1")) {
            return Err(BenchmarkError::ExplicitOptInRequired);
        }

        let mut rows = Vec::with_capacity(LENGTHS.len());
        for elements in LENGTHS {
            let keyboard_us = median(measure(elements, InjectionMode::Keyboard)?);
            let clipboard_us = median(measure(elements, InjectionMode::Clipboard)?);
            rows.push(Row {
                elements,
                keyboard_us,
                clipboard_us,
                winner: if clipboard_us < keyboard_us {
                    InjectionBackend::Clipboard
                } else {
                    InjectionBackend::Keyboard
                },
            });
        }

        let recommended_threshold = rows
            .iter()
            .find(|row| row.clipboard_us.saturating_mul(100) <= row.keyboard_us.saturating_mul(80))
            .map(|row| row.elements)
            .unwrap_or(*LENGTHS.last().expect("benchmark lengths are non-empty"));

        Ok(BenchmarkResult {
            rows,
            recommended_threshold,
        })
    }

    fn measure(elements: usize, mode: InjectionMode) -> Result<Vec<u128>, BenchmarkError> {
        let mut durations = Vec::with_capacity(REPETITIONS);
        for repetition in 0..REPETITIONS {
            let character = char::from(b'a' + u8::try_from(repetition).unwrap_or_default());
            let fixture: String = std::iter::repeat_n(character, elements).collect();
            let _clipboard = ClipboardFixture::install(&fixture)?;
            let window = ControlledWindow::create()?;
            window.activate()?;

            let keyboard = WindowsKeyboard::new();
            let coordinator = Coordinator::new_product(
                WindowsClipboard::new(),
                WindowsTarget::new(),
                keyboard,
                keyboard,
                WindowsPaste::new(),
                ProductConfig {
                    enabled: true,
                    mode,
                    auto_clipboard_threshold: AutoClipboardThreshold::new(256)
                        .map_err(|_| BenchmarkError::CoordinatorConfiguration)?,
                    jitter_percent: 0,
                    typo_probability_percent: 0,
                    safety: P1Config {
                        keyboard_interval: Duration::from_millis(1),
                        ..P1Config::default()
                    },
                },
            )
            .map_err(|_| BenchmarkError::CoordinatorConfiguration)?;

            let started = Instant::now();
            if !matches!(coordinator.trigger(), TriggerResult::Started { .. }) {
                return Err(BenchmarkError::TriggerRejected);
            }
            if coordinator.wait_for_idle(Duration::from_secs(15)) != WaitResult::Idle {
                return Err(BenchmarkError::CoordinatorTimeout);
            }
            let observed = wait_for_expected_text(window.edit, &fixture)?;
            let elapsed = started.elapsed().as_micros();

            let status = coordinator.status();
            if status.completion != Some(SessionCompletion::Finished(TerminalOutcome::Completed)) {
                return Err(BenchmarkError::CoordinatorOutcome);
            }
            let expected_backend = match mode {
                InjectionMode::Keyboard => InjectionBackend::Keyboard,
                InjectionMode::Clipboard => InjectionBackend::Clipboard,
                InjectionMode::Auto => return Err(BenchmarkError::UnexpectedBackend),
            };
            if status.backend != Some(expected_backend) {
                return Err(BenchmarkError::UnexpectedBackend);
            }
            if observed != fixture {
                return Err(BenchmarkError::TextMismatch);
            }
            durations.push(elapsed);
        }
        Ok(durations)
    }

    fn median(mut values: Vec<u128>) -> u128 {
        values.sort_unstable();
        values[values.len() / 2]
    }

    struct ControlledWindow {
        parent: HWND,
        edit: HWND,
    }

    impl ControlledWindow {
        fn create() -> Result<Self, BenchmarkError> {
            let parent_class = wide("STATIC");
            let edit_class = wide("EDIT");
            let title = wide("ClipType P2 Benchmark Target");
            let empty = [0_u16];

            // SAFETY: all strings are nul-terminated for these calls.
            let parent = unsafe {
                CreateWindowExW(
                    0,
                    parent_class.as_ptr(),
                    title.as_ptr(),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    480,
                    180,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null(),
                )
            };
            if parent.is_null() {
                return Err(BenchmarkError::WindowCreation);
            }
            // SAFETY: parent is live and owns the child after creation.
            let edit = unsafe {
                CreateWindowExW(
                    0,
                    edit_class.as_ptr(),
                    empty.as_ptr(),
                    WS_CHILD | WS_VISIBLE | ES_MULTILINE | ES_AUTOVSCROLL,
                    8,
                    8,
                    450,
                    140,
                    parent,
                    null_mut(),
                    null_mut(),
                    null(),
                )
            };
            if edit.is_null() {
                // SAFETY: parent remains owned here.
                let _ = unsafe { DestroyWindow(parent) };
                return Err(BenchmarkError::WindowCreation);
            }
            Ok(Self { parent, edit })
        }

        fn activate(&self) -> Result<(), BenchmarkError> {
            for _ in 0..60 {
                // SAFETY: handles are live and owned by this process.
                let _ = unsafe { ShowWindow(self.parent, SW_SHOW) };
                let _ = unsafe { update_window(self.parent) };
                let _ = unsafe { SetForegroundWindow(self.parent) };
                let _ = unsafe { set_focus(self.edit) };
                pump_messages();
                // SAFETY: query-only opaque handle calls.
                if unsafe { GetForegroundWindow() } == self.parent
                    && unsafe { get_focus() } == self.edit
                {
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(BenchmarkError::ForegroundFocus)
        }
    }

    impl Drop for ControlledWindow {
        fn drop(&mut self) {
            // SAFETY: parent is owned by this guard.
            let _ = unsafe { DestroyWindow(self.parent) };
        }
    }

    struct ClipboardFixture;

    impl ClipboardFixture {
        fn install(value: &str) -> Result<Self, BenchmarkError> {
            let units: Vec<u16> = value.encode_utf16().chain([0]).collect();
            let bytes = units
                .len()
                .checked_mul(size_of::<u16>())
                .ok_or(BenchmarkError::ClipboardAllocation)?;
            // SAFETY: checked movable allocation.
            let allocation = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) };
            if allocation.is_null() {
                return Err(BenchmarkError::ClipboardAllocation);
            }
            let mut allocation = PendingGlobal::new(allocation);
            // SAFETY: allocation is live and sufficiently large.
            let pointer = unsafe { GlobalLock(allocation.handle) };
            if pointer.is_null() {
                return Err(BenchmarkError::ClipboardAllocation);
            }
            // SAFETY: buffers are valid, non-overlapping and exactly sized.
            unsafe {
                copy_nonoverlapping(units.as_ptr(), pointer.cast::<u16>(), units.len());
                let _ = GlobalUnlock(allocation.handle);
            }
            // SAFETY: null owner and no user data inspection.
            if unsafe { OpenClipboard(null_mut()) } == 0 {
                return Err(BenchmarkError::ClipboardOpen);
            }
            let opened = ClipboardOpenGuard;
            // SAFETY: clipboard is open.
            if unsafe { EmptyClipboard() } == 0 {
                return Err(BenchmarkError::ClipboardWrite);
            }
            // SAFETY: success transfers allocation ownership.
            if unsafe { SetClipboardData(CF_UNICODETEXT, allocation.handle) }.is_null() {
                return Err(BenchmarkError::ClipboardWrite);
            }
            allocation.transferred = true;
            drop(opened);
            Ok(Self)
        }
    }

    impl Drop for ClipboardFixture {
        fn drop(&mut self) {
            // The opt-in harness clears only its generated fixture.
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
                // SAFETY: ownership remains with this guard.
                let _ = unsafe { GlobalFree(self.handle) };
            }
        }
    }

    struct ClipboardOpenGuard;

    impl Drop for ClipboardOpenGuard {
        fn drop(&mut self) {
            // SAFETY: guard exists only after successful open.
            let _ = unsafe { CloseClipboard() };
        }
    }

    fn wait_for_expected_text(window: HWND, expected: &str) -> Result<String, BenchmarkError> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            pump_messages();
            let observed = read_window_text(window)?;
            if observed == expected || Instant::now() >= deadline {
                return Ok(observed);
            }
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn read_window_text(window: HWND) -> Result<String, BenchmarkError> {
        // SAFETY: window is the live edit owned by ControlledWindow.
        let length = unsafe { GetWindowTextLengthW(window) };
        if length < 0 {
            return Err(BenchmarkError::TargetRead);
        }
        let capacity = usize::try_from(length)
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or(BenchmarkError::TargetRead)?;
        let mut buffer = vec![0_u16; capacity];
        let count = i32::try_from(capacity).map_err(|_| BenchmarkError::TargetRead)?;
        // SAFETY: buffer is writable for count units.
        let copied = unsafe { GetWindowTextW(window, buffer.as_mut_ptr(), count) };
        if copied < 0 {
            return Err(BenchmarkError::TargetRead);
        }
        buffer.truncate(usize::try_from(copied).map_err(|_| BenchmarkError::TargetRead)?);
        String::from_utf16(&buffer).map_err(|_| BenchmarkError::TargetRead)
    }

    fn pump_messages() {
        let mut message = MSG::default();
        // SAFETY: each removed message initializes the structure before use.
        while unsafe { PeekMessageW(&raw mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
            let _ = unsafe { TranslateMessage(&raw const message) };
            let _ = unsafe { DispatchMessageW(&raw const message) };
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain([0]).collect()
    }
}
