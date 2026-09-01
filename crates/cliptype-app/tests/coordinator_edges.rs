use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use cliptype_app::{Coordinator, SessionCompletion, TriggerResult, WaitResult};
use cliptype_core::{
    CapabilityState, DispatchBatchLimit, EvidenceStrength, IntegrityRelation, NativeEventCount,
    P1Config, PreparationFailure, RetryAttemptLimit, RetryBudget, SemanticElementLimit,
    SensitiveText, TerminalOutcome,
};
use cliptype_platform::{
    ClipboardError, ClipboardPort, DispatchResult, KeyboardCapabilities, KeyboardError,
    KeyboardPort, ModifierMask, ModifierObservation, ModifierPort, NativeDispatchCount,
    NativeError, NativeErrorKind, TargetCaptureError, TargetComparison, TargetEvidence,
    TargetMetadata, TargetPort,
};

#[derive(Clone)]
struct ScriptedClipboard {
    state: Arc<Mutex<ClipboardState>>,
}

struct ClipboardState {
    text: String,
    failures: VecDeque<ClipboardError>,
    calls: usize,
}

impl ScriptedClipboard {
    fn new(text: &str, failures: impl IntoIterator<Item = ClipboardError>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ClipboardState {
                text: text.to_owned(),
                failures: failures.into_iter().collect(),
                calls: 0,
            })),
        }
    }

    fn calls(&self) -> usize {
        lock(&self.state).calls
    }
}

impl ClipboardPort for ScriptedClipboard {
    fn read_current_text(
        &self,
        _hard_limit: cliptype_core::NativeByteLimit,
    ) -> Result<SensitiveText, ClipboardError> {
        let mut state = lock(&self.state);
        state.calls = state.calls.saturating_add(1);
        if let Some(error) = state.failures.pop_front() {
            Err(error)
        } else {
            Ok(SensitiveText::new(state.text.clone()))
        }
    }
}

#[derive(Clone)]
struct ScriptedTarget {
    state: Arc<Mutex<TargetState>>,
}

struct TargetState {
    captures: VecDeque<Result<u64, TargetCaptureError>>,
    fallback: Result<u64, TargetCaptureError>,
    integrity: IntegrityRelation,
}

impl ScriptedTarget {
    fn new(
        captures: impl IntoIterator<Item = Result<u64, TargetCaptureError>>,
        fallback: Result<u64, TargetCaptureError>,
        integrity: IntegrityRelation,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(TargetState {
                captures: captures.into_iter().collect(),
                fallback,
                integrity,
            })),
        }
    }

    fn stable(token: u64) -> Self {
        Self::new(
            [Ok(token)],
            Ok(token),
            IntegrityRelation::KnownNotRestricted,
        )
    }
}

impl TargetPort for ScriptedTarget {
    fn capture(&self) -> Result<TargetEvidence, TargetCaptureError> {
        let mut state = lock(&self.state);
        let fallback = state.fallback;
        let token = state.captures.pop_front().unwrap_or(fallback)?;
        Ok(TargetEvidence::new(
            token,
            TargetMetadata {
                process_id: Some(7),
                gui_thread_id: Some(11),
            },
            EvidenceStrength::NativeFocusedControl,
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
struct ScriptedModifiers {
    state: Arc<Mutex<ModifierState>>,
}

struct ModifierState {
    values: VecDeque<ModifierObservation>,
    fallback: ModifierObservation,
}

impl ScriptedModifiers {
    fn new(
        values: impl IntoIterator<Item = ModifierObservation>,
        fallback: ModifierObservation,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ModifierState {
                values: values.into_iter().collect(),
                fallback,
            })),
        }
    }

    fn clear() -> Self {
        Self::new([], ModifierObservation::Clear)
    }
}

impl ModifierPort for ScriptedModifiers {
    fn observe_modifiers(&self) -> ModifierObservation {
        let mut state = lock(&self.state);
        let fallback = state.fallback;
        state.values.pop_front().unwrap_or(fallback)
    }
}

#[derive(Clone)]
struct ScriptedKeyboard {
    state: Arc<Mutex<KeyboardState>>,
    capabilities: KeyboardCapabilities,
}

struct KeyboardState {
    results: VecDeque<DispatchResult>,
    calls: usize,
}

impl ScriptedKeyboard {
    fn new(
        results: impl IntoIterator<Item = DispatchResult>,
        capabilities: KeyboardCapabilities,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(KeyboardState {
                results: results.into_iter().collect(),
                calls: 0,
            })),
            capabilities,
        }
    }

    fn complete() -> Self {
        Self::new([], available_capabilities())
    }

    fn calls(&self) -> usize {
        lock(&self.state).calls
    }
}

impl KeyboardPort for ScriptedKeyboard {
    fn capabilities(&self) -> KeyboardCapabilities {
        self.capabilities
    }

    fn dispatch(
        &self,
        _batch: cliptype_core::TextBatch<'_>,
    ) -> Result<DispatchResult, KeyboardError> {
        let mut state = lock(&self.state);
        state.calls = state.calls.saturating_add(1);
        Ok(state
            .results
            .pop_front()
            .unwrap_or(DispatchResult::Complete {
                events: NativeEventCount::new(2),
            }))
    }
}

fn available_capabilities() -> KeyboardCapabilities {
    KeyboardCapabilities {
        unicode_text: CapabilityState::Available,
        line_break: CapabilityState::Available,
        tab: CapabilityState::Available,
        modifier_observation: CapabilityState::Available,
    }
}

fn config(batch: usize) -> P1Config {
    P1Config {
        total_payload_limit: SemanticElementLimit::new(64).expect("payload bound"),
        dispatch_batch_limit: DispatchBatchLimit::new(batch).expect("batch bound"),
        keyboard_interval: Duration::from_millis(5),
        modifier_settle_timeout: Duration::from_millis(30),
        modifier_poll_interval: Duration::from_millis(1),
        clipboard_retry: RetryBudget::new(
            RetryAttemptLimit::new(20).expect("retry bound"),
            Duration::from_millis(100),
        )
        .expect("retry window"),
        worker_shutdown_grace: Duration::from_secs(1),
        ..P1Config::default()
    }
    .validate()
    .expect("valid config")
}

fn coordinator(
    clipboard: ScriptedClipboard,
    target: ScriptedTarget,
    keyboard: ScriptedKeyboard,
    modifiers: ScriptedModifiers,
    config: P1Config,
) -> Coordinator {
    Coordinator::new(clipboard, target, keyboard, modifiers, config).expect("coordinator")
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
fn failed_initial_target_capture_releases_the_session_slot() {
    let target = ScriptedTarget::new(
        [Err(TargetCaptureError::Unavailable), Ok(1)],
        Ok(1),
        IntegrityRelation::KnownNotRestricted,
    );
    let coordinator = coordinator(
        ScriptedClipboard::new("text", []),
        target,
        ScriptedKeyboard::complete(),
        ScriptedModifiers::clear(),
        config(2),
    );

    assert_eq!(
        coordinator.trigger(),
        TriggerResult::Rejected(PreparationFailure::TargetUnavailable)
    );
    start_and_wait(&coordinator);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(TerminalOutcome::Completed))
    );
}

#[test]
fn held_modifier_times_out_without_releasing_user_keys() {
    let coordinator = coordinator(
        ScriptedClipboard::new("text", []),
        ScriptedTarget::stable(1),
        ScriptedKeyboard::complete(),
        ScriptedModifiers::new([], ModifierObservation::Held(ModifierMask::CONTROL)),
        config(2),
    );

    start_and_wait(&coordinator);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::ModifierSettleTimeout
        ))
    );
}

#[test]
fn cancellation_interrupts_clipboard_retry_waits() {
    let clipboard = ScriptedClipboard::new("text", std::iter::repeat_n(ClipboardError::Busy, 20));
    let coordinator = coordinator(
        clipboard.clone(),
        ScriptedTarget::stable(1),
        ScriptedKeyboard::complete(),
        ScriptedModifiers::clear(),
        config(2),
    );

    assert!(matches!(
        coordinator.trigger(),
        TriggerResult::Started { .. }
    ));
    thread::sleep(Duration::from_millis(10));
    let _ = coordinator.cancel();
    assert_eq!(
        coordinator.wait_for_idle(Duration::from_secs(2)),
        WaitResult::Idle
    );

    assert!(clipboard.calls() < 20);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::Cancelled
        ))
    );
}

#[test]
fn oversized_semantic_payload_fails_before_dispatch() {
    let keyboard = ScriptedKeyboard::complete();
    let coordinator = coordinator(
        ScriptedClipboard::new("abcdef", []),
        ScriptedTarget::stable(1),
        keyboard.clone(),
        ScriptedModifiers::clear(),
        P1Config {
            total_payload_limit: SemanticElementLimit::new(3).expect("small limit"),
            dispatch_batch_limit: DispatchBatchLimit::new(2).expect("small batch"),
            ..config(2)
        }
        .validate()
        .expect("valid small-limit config"),
    );

    start_and_wait(&coordinator);
    assert_eq!(keyboard.calls(), 0);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::PayloadTooLarge
        ))
    );
}

#[test]
fn degraded_required_capability_fails_closed() {
    let mut capabilities = available_capabilities();
    capabilities.modifier_observation = CapabilityState::Degraded;
    let keyboard = ScriptedKeyboard::new([], capabilities);
    let coordinator = coordinator(
        ScriptedClipboard::new("text", []),
        ScriptedTarget::stable(1),
        keyboard.clone(),
        ScriptedModifiers::clear(),
        config(2),
    );

    start_and_wait(&coordinator);
    assert_eq!(keyboard.calls(), 0);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::DegradedCapabilityRejected
        ))
    );
}

#[test]
fn target_change_between_batches_stops_later_dispatch() {
    let target = ScriptedTarget::new(
        [Ok(1), Ok(1), Ok(2)],
        Ok(2),
        IntegrityRelation::KnownNotRestricted,
    );
    let keyboard = ScriptedKeyboard::complete();
    let coordinator = coordinator(
        ScriptedClipboard::new("ab", []),
        target,
        keyboard.clone(),
        ScriptedModifiers::clear(),
        config(1),
    );

    start_and_wait(&coordinator);
    assert_eq!(keyboard.calls(), 1);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(TerminalOutcome::TargetChanged))
    );
}

#[test]
fn unavailable_focus_evidence_aborts_under_strict_policy() {
    let target = ScriptedTarget::new(
        [Ok(1), Err(TargetCaptureError::Unavailable)],
        Err(TargetCaptureError::Unavailable),
        IntegrityRelation::KnownNotRestricted,
    );
    let keyboard = ScriptedKeyboard::complete();
    let coordinator = coordinator(
        ScriptedClipboard::new("text", []),
        target,
        keyboard.clone(),
        ScriptedModifiers::clear(),
        config(2),
    );

    start_and_wait(&coordinator);
    assert_eq!(keyboard.calls(), 0);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(
            TerminalOutcome::TargetEvidenceUnavailable
        ))
    );
}

#[test]
fn modifier_conflict_between_batches_stops_later_dispatch() {
    let keyboard = ScriptedKeyboard::complete();
    let modifiers = ScriptedModifiers::new(
        [
            ModifierObservation::Clear,
            ModifierObservation::Clear,
            ModifierObservation::Held(ModifierMask::SHIFT),
        ],
        ModifierObservation::Held(ModifierMask::SHIFT),
    );
    let coordinator = coordinator(
        ScriptedClipboard::new("ab", []),
        ScriptedTarget::stable(1),
        keyboard.clone(),
        modifiers,
        config(1),
    );

    start_and_wait(&coordinator);
    assert_eq!(keyboard.calls(), 1);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(
            TerminalOutcome::ModifierConflict
        ))
    );
}

#[test]
fn progress_unknown_stops_after_one_native_attempt() {
    let counts = NativeDispatchCount {
        requested: NativeEventCount::new(4),
        accepted: NativeEventCount::new(1),
    };
    let keyboard = ScriptedKeyboard::new(
        [DispatchResult::ProgressUnknown { counts }],
        available_capabilities(),
    );
    let coordinator = coordinator(
        ScriptedClipboard::new("abcd", []),
        ScriptedTarget::stable(1),
        keyboard.clone(),
        ScriptedModifiers::clear(),
        config(2),
    );

    start_and_wait(&coordinator);
    assert_eq!(keyboard.calls(), 1);
    assert_eq!(
        coordinator.status().completion,
        Some(SessionCompletion::Finished(
            TerminalOutcome::ProgressUnknown
        ))
    );
}

#[test]
fn blocked_unknown_and_known_integrity_restriction_remain_distinct() {
    let blocked = ScriptedKeyboard::new(
        [DispatchResult::NoneAccepted {
            requested: NativeEventCount::new(2),
            native: Some(NativeError::new(NativeErrorKind::BlockedCauseUnknown, None)),
        }],
        available_capabilities(),
    );
    let unknown_coordinator = coordinator(
        ScriptedClipboard::new("a", []),
        ScriptedTarget::stable(1),
        blocked.clone(),
        ScriptedModifiers::clear(),
        config(1),
    );
    start_and_wait(&unknown_coordinator);
    assert_eq!(blocked.calls(), 1);
    assert_eq!(
        unknown_coordinator.status().completion,
        Some(SessionCompletion::Finished(
            TerminalOutcome::BlockedCauseUnknown
        ))
    );

    let restricted_target = ScriptedTarget::new([Ok(1)], Ok(1), IntegrityRelation::KnownRestricted);
    let restricted_keyboard = ScriptedKeyboard::complete();
    let restricted_coordinator = coordinator(
        ScriptedClipboard::new("a", []),
        restricted_target,
        restricted_keyboard.clone(),
        ScriptedModifiers::clear(),
        config(1),
    );
    start_and_wait(&restricted_coordinator);
    assert_eq!(restricted_keyboard.calls(), 0);
    assert_eq!(
        restricted_coordinator.status().completion,
        Some(SessionCompletion::PreparationFailed(
            PreparationFailure::KnownSecurityRestriction
        ))
    );
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
