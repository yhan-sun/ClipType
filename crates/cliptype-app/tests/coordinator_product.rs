use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use cliptype_app::{Coordinator, SessionCompletion, TriggerResult, WaitResult};
use cliptype_core::{
    AutoClipboardThreshold, CapabilityState, InjectionBackend, InjectionMode, NativeEventCount,
    PreparationFailure, ProductConfig, SensitiveText, TerminalOutcome,
};
use cliptype_platform::{
    ClipboardError, ClipboardPort, ClipboardRevision, DispatchResult, KeyboardCapabilities,
    KeyboardError, KeyboardPort, ModifierObservation, ModifierPort, PasteCapabilities, PasteError,
    PastePort, TargetCaptureError, TargetComparison, TargetEvidence, TargetMetadata, TargetPort,
};

#[derive(Clone)]
struct RevisionedClipboard {
    text: String,
    revisions: Arc<Mutex<VecDeque<ClipboardRevision>>>,
    reads: Arc<AtomicUsize>,
}

impl RevisionedClipboard {
    fn stable(text: &str, revision: u64, sessions: usize) -> Self {
        let revisions =
            std::iter::repeat_n(ClipboardRevision::Known(revision), sessions * 2).collect();
        Self {
            text: text.to_owned(),
            revisions: Arc::new(Mutex::new(revisions)),
            reads: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn unavailable(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            revisions: Arc::new(Mutex::new(VecDeque::from([
                ClipboardRevision::Unavailable,
                ClipboardRevision::Unavailable,
            ]))),
            reads: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}

impl ClipboardPort for RevisionedClipboard {
    fn read_current_text(
        &self,
        _hard_limit: cliptype_core::NativeByteLimit,
    ) -> Result<SensitiveText, ClipboardError> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        Ok(SensitiveText::new(self.text.clone()))
    }

    fn current_revision(&self) -> ClipboardRevision {
        lock(&self.revisions)
            .pop_front()
            .unwrap_or(ClipboardRevision::Known(1))
    }
}

#[derive(Clone, Default)]
struct StableTarget {
    calls: Arc<AtomicUsize>,
}

impl StableTarget {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl TargetPort for StableTarget {
    fn capture(&self) -> Result<TargetEvidence, TargetCaptureError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(TargetEvidence::new(
            9_u64,
            TargetMetadata {
                process_id: Some(11),
                gui_thread_id: Some(12),
            },
            cliptype_core::EvidenceStrength::NativeFocusedControl,
        ))
    }

    fn compare(&self, expected: &TargetEvidence, observed: &TargetEvidence) -> TargetComparison {
        match (expected.token::<u64>(), observed.token::<u64>()) {
            (Some(left), Some(right)) if left == right => TargetComparison::Same,
            (Some(_), Some(_)) => TargetComparison::Changed,
            _ => TargetComparison::UnavailableOrAmbiguous,
        }
    }

    fn integrity_relation(&self, _target: &TargetEvidence) -> cliptype_core::IntegrityRelation {
        cliptype_core::IntegrityRelation::KnownNotRestricted
    }
}

#[derive(Clone, Default)]
struct CountingKeyboard {
    calls: Arc<AtomicUsize>,
    cursor_right_calls: Arc<AtomicUsize>,
}

impl CountingKeyboard {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }

    fn cursor_right_calls(&self) -> usize {
        self.cursor_right_calls.load(Ordering::Relaxed)
    }
}

impl KeyboardPort for CountingKeyboard {
    fn capabilities(&self) -> KeyboardCapabilities {
        KeyboardCapabilities {
            unicode_text: CapabilityState::Available,
            line_break: CapabilityState::Available,
            tab: CapabilityState::Available,
            cursor_right: CapabilityState::Available,
            modifier_observation: CapabilityState::Available,
        }
    }

    fn dispatch(
        &self,
        _batch: cliptype_core::TextBatch<'_>,
    ) -> Result<DispatchResult, KeyboardError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(DispatchResult::Complete {
            events: NativeEventCount::new(2),
        })
    }

    fn dispatch_cursor_right(&self) -> Result<DispatchResult, KeyboardError> {
        self.cursor_right_calls.fetch_add(1, Ordering::Relaxed);
        Ok(DispatchResult::Complete {
            events: NativeEventCount::new(2),
        })
    }
}

#[derive(Clone)]
struct ScriptedPaste {
    state: Arc<Mutex<PasteState>>,
    capabilities: PasteCapabilities,
}

struct PasteState {
    results: VecDeque<Result<DispatchResult, PasteError>>,
    revisions: Vec<ClipboardRevision>,
}

impl ScriptedPaste {
    fn complete() -> Self {
        Self::new([], available_paste_capabilities())
    }

    fn new(
        results: impl IntoIterator<Item = Result<DispatchResult, PasteError>>,
        capabilities: PasteCapabilities,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(PasteState {
                results: results.into_iter().collect(),
                revisions: Vec::new(),
            })),
            capabilities,
        }
    }

    fn calls(&self) -> usize {
        lock(&self.state).revisions.len()
    }
}

impl PastePort for ScriptedPaste {
    fn capabilities(&self) -> PasteCapabilities {
        self.capabilities
    }

    fn dispatch_paste(
        &self,
        expected_revision: ClipboardRevision,
    ) -> Result<DispatchResult, PasteError> {
        let mut state = lock(&self.state);
        state.revisions.push(expected_revision);
        state
            .results
            .pop_front()
            .unwrap_or(Ok(DispatchResult::Complete {
                events: NativeEventCount::new(4),
            }))
    }
}

#[derive(Clone)]
struct GateModifiers {
    held: Arc<AtomicBool>,
}

impl GateModifiers {
    fn clear() -> Self {
        Self {
            held: Arc::new(AtomicBool::new(false)),
        }
    }

    fn held() -> Self {
        Self {
            held: Arc::new(AtomicBool::new(true)),
        }
    }

    fn release(&self) {
        self.held.store(false, Ordering::Release);
    }
}

impl ModifierPort for GateModifiers {
    fn observe_modifiers(&self) -> ModifierObservation {
        if self.held.load(Ordering::Acquire) {
            ModifierObservation::Held(cliptype_platform::ModifierMask::CONTROL)
        } else {
            ModifierObservation::Clear
        }
    }
}

fn available_paste_capabilities() -> PasteCapabilities {
    PasteCapabilities {
        paste_chord: CapabilityState::Available,
        clipboard_revision_guard: CapabilityState::Available,
    }
}

fn unavailable_paste_capabilities() -> PasteCapabilities {
    PasteCapabilities {
        paste_chord: CapabilityState::Unavailable,
        clipboard_revision_guard: CapabilityState::Unavailable,
    }
}

fn config(mode: InjectionMode, threshold: usize) -> ProductConfig {
    ProductConfig {
        mode,
        auto_clipboard_threshold: AutoClipboardThreshold::new(threshold)
            .expect("non-zero test threshold"),
        safety: cliptype_core::P1Config {
            modifier_poll_interval: Duration::from_millis(1),
            modifier_settle_timeout: Duration::from_secs(1),
            worker_shutdown_grace: Duration::from_secs(1),
            ..cliptype_core::P1Config::default()
        },
        ..ProductConfig::default()
    }
    .validate()
    .expect("test product configuration")
}

fn coordinator(
    clipboard: RevisionedClipboard,
    target: StableTarget,
    keyboard: CountingKeyboard,
    modifiers: GateModifiers,
    paste: ScriptedPaste,
    config: ProductConfig,
) -> Coordinator {
    Coordinator::new_product(clipboard, target, keyboard, modifiers, paste, config)
        .expect("product coordinator")
}

fn start_and_wait(coordinator: &Coordinator) {
    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    assert_eq!(
        coordinator.wait_for_idle(Duration::from_secs(2)),
        WaitResult::Idle
    );
}

#[test]
fn explicit_clipboard_uses_one_revision_guarded_paste_only() {
    let clipboard = RevisionedClipboard::stable("payload", 7, 1);
    let keyboard = CountingKeyboard::default();
    let paste = ScriptedPaste::complete();
    let coordinator = coordinator(
        clipboard.clone(),
        StableTarget::default(),
        keyboard.clone(),
        GateModifiers::clear(),
        paste.clone(),
        config(InjectionMode::Clipboard, 256),
    );

    start_and_wait(&coordinator);

    let status = coordinator.status();
    assert_eq!(clipboard.reads(), 1);
    assert_eq!(keyboard.calls(), 0);
    assert_eq!(paste.calls(), 1);
    assert_eq!(status.backend, Some(InjectionBackend::Clipboard));
    assert_eq!(status.batches_completed, 1);
    assert_eq!(
        status.completion,
        Some(SessionCompletion::Finished(TerminalOutcome::Completed))
    );
}

#[test]
fn code_mode_uses_keyboard_actions_and_skips_auto_pairs() {
    let clipboard = RevisionedClipboard::stable("if (x[0]) { return {}; }", 7, 1);
    let keyboard = CountingKeyboard::default();
    let paste = ScriptedPaste::new([], unavailable_paste_capabilities());
    let coordinator = coordinator(
        clipboard,
        StableTarget::default(),
        keyboard.clone(),
        GateModifiers::clear(),
        paste.clone(),
        config(InjectionMode::Code, 256),
    );

    start_and_wait(&coordinator);

    assert!(keyboard.calls() > 0);
    assert_eq!(keyboard.cursor_right_calls(), 4);
    assert_eq!(paste.calls(), 0);
    assert_eq!(coordinator.status().backend, Some(InjectionBackend::Code));
    assert_eq!(coordinator.status().batches_completed, 24);
}

#[test]
fn code_mode_types_triple_quotes_without_cursor_right() {
    let source = "const doc = \"\"\"hello\"\"\";";
    let clipboard = RevisionedClipboard::stable(source, 8, 1);
    let keyboard = CountingKeyboard::default();
    let paste = ScriptedPaste::new([], unavailable_paste_capabilities());
    let coordinator = coordinator(
        clipboard,
        StableTarget::default(),
        keyboard.clone(),
        GateModifiers::clear(),
        paste,
        config(InjectionMode::Code, 256),
    );

    start_and_wait(&coordinator);

    assert_eq!(keyboard.calls(), source.chars().count());
    assert_eq!(keyboard.cursor_right_calls(), 0);
    assert_eq!(coordinator.status().backend, Some(InjectionBackend::Code));
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(TerminalOutcome::Completed))
    );
}

#[test]
fn code_mode_keeps_markdown_fences_and_skips_pairs_inside_them() {
    let source = "```cpp\nif (x) {\n    return;\n}\n```";
    let clipboard = RevisionedClipboard::stable(source, 9, 1);
    let keyboard = CountingKeyboard::default();
    let paste = ScriptedPaste::new([], unavailable_paste_capabilities());
    let coordinator = coordinator(
        clipboard,
        StableTarget::default(),
        keyboard.clone(),
        GateModifiers::clear(),
        paste,
        config(InjectionMode::Code, 256),
    );

    start_and_wait(&coordinator);

    assert_eq!(keyboard.cursor_right_calls(), 2);
    assert_eq!(keyboard.calls(), source.chars().count() - 6);
    assert_eq!(coordinator.status().backend, Some(InjectionBackend::Code));
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(TerminalOutcome::Completed))
    );
}

#[test]
fn auto_selects_keyboard_below_threshold_and_clipboard_at_threshold() {
    let short_keyboard = CountingKeyboard::default();
    let short_paste = ScriptedPaste::complete();
    let short = coordinator(
        RevisionedClipboard::stable("short", 2, 1),
        StableTarget::default(),
        short_keyboard.clone(),
        GateModifiers::clear(),
        short_paste.clone(),
        config(InjectionMode::Auto, 8),
    );
    start_and_wait(&short);
    assert_eq!(short.status().backend, Some(InjectionBackend::Keyboard));
    assert_ne!(short_keyboard.calls(), 0);
    assert_eq!(short_paste.calls(), 0);

    let long_keyboard = CountingKeyboard::default();
    let long_paste = ScriptedPaste::complete();
    let long = coordinator(
        RevisionedClipboard::stable("12345678", 3, 1),
        StableTarget::default(),
        long_keyboard.clone(),
        GateModifiers::clear(),
        long_paste.clone(),
        config(InjectionMode::Auto, 8),
    );
    start_and_wait(&long);
    assert_eq!(long.status().backend, Some(InjectionBackend::Clipboard));
    assert_eq!(long_keyboard.calls(), 0);
    assert_eq!(long_paste.calls(), 1);
}

#[test]
fn explicit_clipboard_rejects_unavailable_revision_without_fallback() {
    let keyboard = CountingKeyboard::default();
    let paste = ScriptedPaste::complete();
    let coordinator = coordinator(
        RevisionedClipboard::unavailable("payload"),
        StableTarget::default(),
        keyboard.clone(),
        GateModifiers::clear(),
        paste.clone(),
        config(InjectionMode::Clipboard, 8),
    );

    start_and_wait(&coordinator);
    assert_eq!(keyboard.calls(), 0);
    assert_eq!(paste.calls(), 0);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::ClipboardRevisionUnavailable
        ))
    );
}

#[test]
fn changed_clipboard_before_paste_stops_without_retry() {
    let paste = ScriptedPaste::new(
        [Err(PasteError::ClipboardChanged)],
        available_paste_capabilities(),
    );
    let coordinator = coordinator(
        RevisionedClipboard::stable("payload", 7, 1),
        StableTarget::default(),
        CountingKeyboard::default(),
        GateModifiers::clear(),
        paste.clone(),
        config(InjectionMode::Clipboard, 8),
    );

    start_and_wait(&coordinator);
    assert_eq!(paste.calls(), 1);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(
            TerminalOutcome::ClipboardChanged
        ))
    );
}

#[test]
fn progress_unknown_paste_is_attempted_once() {
    let paste = ScriptedPaste::new(
        [Ok(DispatchResult::ProgressUnknown {
            counts: cliptype_platform::NativeDispatchCount {
                requested: NativeEventCount::new(4),
                accepted: NativeEventCount::new(2),
            },
        })],
        available_paste_capabilities(),
    );
    let coordinator = coordinator(
        RevisionedClipboard::stable("payload", 7, 1),
        StableTarget::default(),
        CountingKeyboard::default(),
        GateModifiers::clear(),
        paste.clone(),
        config(InjectionMode::Clipboard, 8),
    );

    start_and_wait(&coordinator);
    assert_eq!(paste.calls(), 1);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(
            TerminalOutcome::ProgressUnknown
        ))
    );
}

#[test]
fn config_update_changes_future_sessions_not_the_active_snapshot() {
    let keyboard = CountingKeyboard::default();
    let paste = ScriptedPaste::complete();
    let modifiers = GateModifiers::held();
    let coordinator = coordinator(
        RevisionedClipboard::stable("payload", 7, 2),
        StableTarget::default(),
        keyboard.clone(),
        modifiers.clone(),
        paste.clone(),
        config(InjectionMode::Keyboard, 8),
    );

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    coordinator
        .update_config(config(InjectionMode::Clipboard, 8))
        .expect("future configuration update");
    modifiers.release();
    assert_eq!(
        coordinator.wait_for_idle(Duration::from_secs(2)),
        WaitResult::Idle
    );
    assert_eq!(
        coordinator.status().backend,
        Some(InjectionBackend::Keyboard)
    );
    assert_ne!(keyboard.calls(), 0);
    assert_eq!(paste.calls(), 0);

    start_and_wait(&coordinator);
    assert_eq!(
        coordinator.status().backend,
        Some(InjectionBackend::Clipboard)
    );
    assert_eq!(paste.calls(), 1);
}

#[test]
fn disabled_mode_rejects_before_target_or_clipboard_work() {
    let clipboard = RevisionedClipboard::stable("private", 7, 1);
    let target = StableTarget::default();
    let coordinator = coordinator(
        clipboard.clone(),
        target.clone(),
        CountingKeyboard::default(),
        GateModifiers::clear(),
        ScriptedPaste::complete(),
        ProductConfig {
            enabled: false,
            ..config(InjectionMode::Auto, 8)
        },
    );

    assert_eq!(
        coordinator.trigger(),
        TriggerResult::Rejected(PreparationFailure::Disabled)
    );
    assert_eq!(clipboard.reads(), 0);
    assert_eq!(target.calls(), 0);
}

#[test]
fn product_status_does_not_expose_plaintext_or_revision_value() {
    let marker = "P2_COORDINATOR_PRIVATE_SENTINEL";
    let coordinator = coordinator(
        RevisionedClipboard::stable(marker, 982_451, 1),
        StableTarget::default(),
        CountingKeyboard::default(),
        GateModifiers::clear(),
        ScriptedPaste::complete(),
        config(InjectionMode::Clipboard, 8),
    );

    start_and_wait(&coordinator);
    let debug = format!("{:?}", coordinator.status());
    assert!(!debug.contains(marker));
    assert!(!debug.contains("982451"));
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
