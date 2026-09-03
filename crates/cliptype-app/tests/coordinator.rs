use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use cliptype_app::{
    CancelResult, Coordinator, SessionCompletion, ShutdownResult, TriggerResult, WaitResult,
};
use cliptype_core::{
    CapabilityState, DispatchBatchLimit, IntegrityRelation, NativeEventCount, P1Config,
    PreparationFailure, RetryAttemptLimit, RetryBudget, SemanticElementLimit, SensitiveText,
    SessionPhase, TerminalOutcome, TextAtom,
};
use cliptype_platform::{
    ClipboardError, ClipboardPort, DispatchResult, KeyboardCapabilities, KeyboardError,
    KeyboardPort, ModifierMask, ModifierObservation, ModifierPort, NativeDispatchCount,
    TargetCaptureError, TargetComparison, TargetEvidence, TargetMetadata, TargetPort,
};

#[derive(Clone, Default)]
struct Trace(Arc<Mutex<Vec<&'static str>>>);

impl Trace {
    fn push(&self, value: &'static str) {
        lock(&self.0).push(value);
    }

    fn snapshot(&self) -> Vec<&'static str> {
        lock(&self.0).clone()
    }
}

#[derive(Clone)]
struct FakeClipboard {
    state: Arc<Mutex<ClipboardState>>,
    trace: Trace,
}

struct ClipboardState {
    text: String,
    busy_remaining: usize,
    calls: usize,
}

impl FakeClipboard {
    fn new(text: &str, busy_remaining: usize, trace: Trace) -> Self {
        Self {
            state: Arc::new(Mutex::new(ClipboardState {
                text: text.to_owned(),
                busy_remaining,
                calls: 0,
            })),
            trace,
        }
    }

    fn calls(&self) -> usize {
        lock(&self.state).calls
    }
}

impl ClipboardPort for FakeClipboard {
    fn read_current_text(
        &self,
        _hard_limit: cliptype_core::NativeByteLimit,
    ) -> Result<SensitiveText, ClipboardError> {
        self.trace.push("clipboard");
        let mut state = lock(&self.state);
        state.calls = state.calls.saturating_add(1);
        if state.busy_remaining > 0 {
            state.busy_remaining -= 1;
            Err(ClipboardError::Busy)
        } else {
            Ok(SensitiveText::new(state.text.clone()))
        }
    }
}

#[derive(Clone)]
struct FakeTarget {
    state: Arc<Mutex<TargetState>>,
    trace: Trace,
}

struct TargetState {
    steps: VecDeque<Result<u64, TargetCaptureError>>,
    fallback: u64,
    integrity: IntegrityRelation,
    calls: usize,
}

impl FakeTarget {
    fn stable(token: u64, trace: Trace) -> Self {
        Self::sequence(
            [Ok(token)],
            token,
            IntegrityRelation::KnownNotRestricted,
            trace,
        )
    }

    fn sequence(
        steps: impl IntoIterator<Item = Result<u64, TargetCaptureError>>,
        fallback: u64,
        integrity: IntegrityRelation,
        trace: Trace,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(TargetState {
                steps: steps.into_iter().collect(),
                fallback,
                integrity,
                calls: 0,
            })),
            trace,
        }
    }

    fn calls(&self) -> usize {
        lock(&self.state).calls
    }
}

impl TargetPort for FakeTarget {
    fn capture(&self) -> Result<TargetEvidence, TargetCaptureError> {
        self.trace.push("target");
        let mut state = lock(&self.state);
        state.calls = state.calls.saturating_add(1);
        let fallback = state.fallback;
        let token = state.steps.pop_front().unwrap_or(Ok(fallback))?;
        Ok(TargetEvidence::new(
            token,
            TargetMetadata {
                process_id: Some(100),
                gui_thread_id: Some(200),
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

    fn integrity_relation(&self, _target: &TargetEvidence) -> IntegrityRelation {
        lock(&self.state).integrity
    }
}

#[derive(Clone)]
struct FakeModifier {
    state: Arc<Mutex<ModifierState>>,
}

struct ModifierState {
    observations: VecDeque<ModifierObservation>,
    fallback: ModifierObservation,
}

impl FakeModifier {
    fn clear() -> Self {
        Self::sequence([], ModifierObservation::Clear)
    }

    fn held() -> Self {
        Self::sequence([], ModifierObservation::Held(ModifierMask::CONTROL))
    }

    fn sequence(
        observations: impl IntoIterator<Item = ModifierObservation>,
        fallback: ModifierObservation,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ModifierState {
                observations: observations.into_iter().collect(),
                fallback,
            })),
        }
    }
}

impl ModifierPort for FakeModifier {
    fn observe_modifiers(&self) -> ModifierObservation {
        let mut state = lock(&self.state);
        let fallback = state.fallback;
        state.observations.pop_front().unwrap_or(fallback)
    }
}

#[derive(Clone)]
struct FakeKeyboard {
    state: Arc<Mutex<KeyboardState>>,
    trace: Trace,
}

struct KeyboardState {
    output: String,
    calls: usize,
    results: VecDeque<DispatchResult>,
}

impl FakeKeyboard {
    fn complete(trace: Trace) -> Self {
        Self::with_results([], trace)
    }

    fn with_results(results: impl IntoIterator<Item = DispatchResult>, trace: Trace) -> Self {
        Self {
            state: Arc::new(Mutex::new(KeyboardState {
                output: String::new(),
                calls: 0,
                results: results.into_iter().collect(),
            })),
            trace,
        }
    }

    fn output(&self) -> String {
        lock(&self.state).output.clone()
    }

    fn calls(&self) -> usize {
        lock(&self.state).calls
    }
}

impl KeyboardPort for FakeKeyboard {
    fn capabilities(&self) -> KeyboardCapabilities {
        KeyboardCapabilities {
            unicode_text: CapabilityState::Available,
            line_break: CapabilityState::Available,
            tab: CapabilityState::Available,
            modifier_observation: CapabilityState::Available,
        }
    }

    fn dispatch(
        &self,
        batch: cliptype_core::TextBatch<'_>,
    ) -> Result<DispatchResult, KeyboardError> {
        self.trace.push("dispatch");
        let mut state = lock(&self.state);
        state.calls = state.calls.saturating_add(1);
        for atom in batch.atoms() {
            match atom {
                TextAtom::Scalar(value) => state.output.push(*value),
                TextAtom::LineBreak => state.output.push('\n'),
                TextAtom::Tab => state.output.push('\t'),
            }
        }

        Ok(state
            .results
            .pop_front()
            .unwrap_or(DispatchResult::Complete {
                events: NativeEventCount::new(1),
            }))
    }
}

fn config(batch: usize) -> P1Config {
    P1Config {
        total_payload_limit: SemanticElementLimit::new(256).expect("test payload bound"),
        dispatch_batch_limit: DispatchBatchLimit::new(batch).expect("test batch bound"),
        keyboard_interval: Duration::from_millis(1),
        modifier_settle_timeout: Duration::from_millis(100),
        modifier_poll_interval: Duration::from_millis(1),
        clipboard_retry: RetryBudget::new(
            RetryAttemptLimit::new(4).expect("test retry count"),
            Duration::from_millis(20),
        )
        .expect("test retry window"),
        worker_shutdown_grace: Duration::from_secs(1),
        ..P1Config::default()
    }
    .validate()
    .expect("test configuration")
}

fn wait(coordinator: &Coordinator) {
    assert_eq!(
        coordinator.wait_for_idle(Duration::from_secs(2)),
        WaitResult::Idle
    );
}

#[test]
fn multi_batch_unicode_path_is_ordered_and_content_correct() {
    let trace = Trace::default();
    let clipboard = FakeClipboard::new("A中😀e\u{301}\r\nB\tC", 0, trace.clone());
    let target = FakeTarget::stable(1, trace.clone());
    let keyboard = FakeKeyboard::complete(trace.clone());
    let coordinator = Coordinator::new(
        clipboard,
        target,
        keyboard.clone(),
        FakeModifier::clear(),
        config(2),
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { generation: 1 }
    ));
    wait(&coordinator);

    let status = coordinator.status();
    assert_eq!(status.phase, SessionPhase::Idle);
    assert_eq!(
        status.completion,
        Some(SessionCompletion::Finished(TerminalOutcome::Completed))
    );
    assert_eq!(status.batches_completed, 9);
    assert_eq!(keyboard.output(), "A中😀e\u{301}\nB\tC");

    let trace = trace.snapshot();
    assert_eq!(trace.first(), Some(&"target"));
    assert!(
        trace
            .iter()
            .position(|value| *value == "clipboard")
            .is_some_and(|position| position > 0)
    );
}

#[test]
fn rapid_second_trigger_is_busy_and_cancel_releases_the_slot() {
    let trace = Trace::default();
    let clipboard = FakeClipboard::new("payload", 0, trace.clone());
    let target = FakeTarget::stable(1, trace.clone());
    let keyboard = FakeKeyboard::complete(trace);
    let coordinator = Coordinator::new(
        clipboard.clone(),
        target.clone(),
        keyboard,
        FakeModifier::held(),
        config(2),
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    assert_eq!(coordinator.trigger(), TriggerResult::Busy);
    assert_eq!(coordinator.cancel(), CancelResult::Requested);
    wait(&coordinator);

    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::Cancelled
        ))
    );
    assert_ne!(target.calls(), 0);
    assert_eq!(clipboard.calls(), 0);
}

#[test]
fn transient_clipboard_busy_retries_within_budget() {
    let trace = Trace::default();
    let clipboard = FakeClipboard::new("retry", 2, trace.clone());
    let target = FakeTarget::stable(1, trace.clone());
    let keyboard = FakeKeyboard::complete(trace);
    let coordinator = Coordinator::new(
        clipboard.clone(),
        target,
        keyboard,
        FakeModifier::clear(),
        config(2),
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    wait(&coordinator);

    assert_eq!(clipboard.calls(), 3);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(TerminalOutcome::Completed))
    );
}

#[test]
fn clipboard_retry_exhaustion_is_a_preparation_failure() {
    let trace = Trace::default();
    let clipboard = FakeClipboard::new("never read", usize::MAX, trace.clone());
    let target = FakeTarget::stable(1, trace.clone());
    let keyboard = FakeKeyboard::complete(trace);
    let mut retry_attempt_config = config(2);
    // This case verifies the attempt bound, not the independent wall-clock
    // bound. A wide window prevents slower CI hosts from legitimately
    // exhausting the time budget before the fourth attempt.
    retry_attempt_config.clipboard_retry = RetryBudget::new(
        RetryAttemptLimit::new(4).expect("test retry count"),
        Duration::from_secs(1),
    )
    .expect("test retry window");
    let coordinator = Coordinator::new(
        clipboard.clone(),
        target,
        keyboard,
        FakeModifier::clear(),
        retry_attempt_config,
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    wait(&coordinator);

    assert_eq!(clipboard.calls(), 4);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::ClipboardUnavailable
        ))
    );
}

#[test]
fn target_change_before_first_dispatch_aborts_without_input() {
    let trace = Trace::default();
    let target = FakeTarget::sequence(
        [Ok(1), Ok(2)],
        2,
        IntegrityRelation::KnownNotRestricted,
        trace.clone(),
    );
    let keyboard = FakeKeyboard::complete(trace.clone());
    let coordinator = Coordinator::new(
        FakeClipboard::new("payload", 0, trace),
        target,
        keyboard.clone(),
        FakeModifier::clear(),
        config(2),
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    wait(&coordinator);

    assert_eq!(keyboard.calls(), 0);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(TerminalOutcome::TargetChanged))
    );
}

#[test]
fn partial_native_result_stops_without_retry() {
    let trace = Trace::default();
    let counts = NativeDispatchCount {
        requested: NativeEventCount::new(4),
        accepted: NativeEventCount::new(2),
    };
    let keyboard = FakeKeyboard::with_results([DispatchResult::Partial { counts }], trace.clone());
    let coordinator = Coordinator::new(
        FakeClipboard::new("abcd", 0, trace.clone()),
        FakeTarget::stable(1, trace),
        keyboard.clone(),
        FakeModifier::clear(),
        config(2),
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    wait(&coordinator);

    assert_eq!(keyboard.calls(), 1);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(TerminalOutcome::PartialInput))
    );
}

#[test]
fn controlled_shutdown_cancels_active_work_and_rejects_new_triggers() {
    let trace = Trace::default();
    let coordinator = Coordinator::new(
        FakeClipboard::new("payload", 0, trace.clone()),
        FakeTarget::stable(1, trace.clone()),
        FakeKeyboard::complete(trace),
        FakeModifier::held(),
        config(2),
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    assert_eq!(coordinator.shutdown(), ShutdownResult::Complete);
    assert_eq!(coordinator.trigger(), TriggerResult::ShuttingDown);
    assert_eq!(coordinator.status().phase, SessionPhase::Idle);
}

#[test]
fn status_and_results_do_not_expose_clipboard_plaintext() {
    let marker = "CLIPTYPE_PRIVATE_STATUS_SENTINEL_704";
    let trace = Trace::default();
    let coordinator = Coordinator::new(
        FakeClipboard::new(marker, 0, trace.clone()),
        FakeTarget::stable(1, trace.clone()),
        FakeKeyboard::complete(trace),
        FakeModifier::clear(),
        config(8),
    )
    .expect("coordinator");

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    wait(&coordinator);

    assert!(!format!("{:?}", coordinator.status()).contains(marker));
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
