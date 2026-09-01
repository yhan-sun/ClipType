//! Native-neutral live coordinator for the P1 keyboard injection path.

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Condvar, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use cliptype_core::{
    CapabilityState, ConfigError, DispatchDecision, DispatchObservation, FlowEvent, FlowState,
    IntegrityRelation, NoInputReason, NormalizationError, P1Config, PlanCapabilities, PlanError,
    PreparationFailure, SensitiveText, SessionPhase, TerminalOutcome, TextBatch,
    build_keyboard_plan, classify_dispatch, transition,
};
use cliptype_platform::{
    ClipboardError, ClipboardPort, DispatchResult, KeyboardCapabilities, KeyboardError,
    KeyboardPort, ModifierObservation, ModifierPort, NativeErrorKind, TargetCaptureError,
    TargetComparison, TargetEvidence, TargetPort,
};

use crate::CancellationFlag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCompletion {
    PreparationFailed(PreparationFailure),
    Finished(TerminalOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub generation: u64,
    pub phase: SessionPhase,
    pub completion: Option<SessionCompletion>,
    pub batches_completed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerResult {
    Started { generation: u64 },
    Busy,
    ShuttingDown,
    Rejected(PreparationFailure),
    StartFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelResult {
    Requested,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitResult {
    Idle,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownResult {
    Complete,
    TimedOut,
}

#[derive(Clone)]
struct SessionPorts {
    clipboard: Arc<dyn ClipboardPort>,
    target: Arc<dyn TargetPort>,
    keyboard: Arc<dyn KeyboardPort>,
    modifiers: Arc<dyn ModifierPort>,
}

struct RuntimeState {
    generation: u64,
    phase: SessionPhase,
    completion: Option<SessionCompletion>,
    batches_completed: u32,
    cancellation: Option<Arc<CancellationFlag>>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            generation: 0,
            phase: SessionPhase::Idle,
            completion: None,
            batches_completed: 0,
            cancellation: None,
        }
    }
}

struct SharedRuntime {
    active: AtomicBool,
    shutting_down: AtomicBool,
    state: Mutex<RuntimeState>,
    idle: Condvar,
}

impl SharedRuntime {
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            shutting_down: AtomicBool::new(false),
            state: Mutex::new(RuntimeState::default()),
            idle: Condvar::new(),
        }
    }

    fn begin_session(&self) -> (u64, Arc<CancellationFlag>) {
        let cancellation = Arc::new(CancellationFlag::new());
        let mut state = lock_unpoisoned(&self.state);
        state.generation = state.generation.saturating_add(1);
        state.phase = SessionPhase::Preparing;
        state.completion = None;
        state.batches_completed = 0;
        state.cancellation = Some(Arc::clone(&cancellation));
        (state.generation, cancellation)
    }

    fn set_phase(&self, phase: SessionPhase) {
        lock_unpoisoned(&self.state).phase = phase;
    }

    fn increment_batches(&self) {
        let mut state = lock_unpoisoned(&self.state);
        state.batches_completed = state.batches_completed.saturating_add(1);
    }

    fn request_cancel(&self) -> CancelResult {
        let mut state = lock_unpoisoned(&self.state);
        let Some(cancellation) = state.cancellation.clone() else {
            return CancelResult::Idle;
        };

        cancellation.request();
        state.phase = SessionPhase::Cancelling;
        CancelResult::Requested
    }

    fn finish(&self, completion: SessionCompletion) {
        {
            let mut state = lock_unpoisoned(&self.state);
            state.phase = SessionPhase::Idle;
            state.completion = Some(completion);
            state.cancellation = None;
            self.active.store(false, Ordering::Release);
        }
        self.idle.notify_all();
    }

    fn snapshot(&self) -> StatusSnapshot {
        let state = lock_unpoisoned(&self.state);
        StatusSnapshot {
            generation: state.generation,
            phase: state.phase,
            completion: state.completion,
            batches_completed: state.batches_completed,
        }
    }
}

/// Owns exactly one live P1 injection session and its worker lifecycle.
pub struct Coordinator {
    ports: SessionPorts,
    config: P1Config,
    shared: Arc<SharedRuntime>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl Coordinator {
    pub fn new<C, T, K, M>(
        clipboard: C,
        target: T,
        keyboard: K,
        modifiers: M,
        config: P1Config,
    ) -> Result<Self, ConfigError>
    where
        C: ClipboardPort + 'static,
        T: TargetPort + 'static,
        K: KeyboardPort + 'static,
        M: ModifierPort + 'static,
    {
        Self::from_ports(
            Arc::new(clipboard),
            Arc::new(target),
            Arc::new(keyboard),
            Arc::new(modifiers),
            config,
        )
    }

    pub fn from_ports(
        clipboard: Arc<dyn ClipboardPort>,
        target: Arc<dyn TargetPort>,
        keyboard: Arc<dyn KeyboardPort>,
        modifiers: Arc<dyn ModifierPort>,
        config: P1Config,
    ) -> Result<Self, ConfigError> {
        let config = config.validate()?;
        Ok(Self {
            ports: SessionPorts {
                clipboard,
                target,
                keyboard,
                modifiers,
            },
            config,
            shared: Arc::new(SharedRuntime::new()),
            worker: Mutex::new(None),
        })
    }

    pub const fn config(&self) -> P1Config {
        self.config
    }

    pub fn status(&self) -> StatusSnapshot {
        self.shared.snapshot()
    }

    pub fn trigger(&self) -> TriggerResult {
        if self.shared.shutting_down.load(Ordering::Acquire) {
            return TriggerResult::ShuttingDown;
        }

        let mut worker_slot = lock_unpoisoned(&self.worker);
        if let Some(previous) = worker_slot.take() {
            if self.shared.active.load(Ordering::Acquire) {
                *worker_slot = Some(previous);
                return TriggerResult::Busy;
            }
            let _ = previous.join();
        }

        if self
            .shared
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return TriggerResult::Busy;
        }
        if self.shared.shutting_down.load(Ordering::Acquire) {
            self.shared.active.store(false, Ordering::Release);
            self.shared.idle.notify_all();
            return TriggerResult::ShuttingDown;
        }

        let (generation, cancellation) = self.shared.begin_session();
        let target = match self.ports.target.capture() {
            Ok(target) => target,
            Err(_) => {
                self.shared.finish(SessionCompletion::PreparationFailed(
                    PreparationFailure::TargetUnavailable,
                ));
                return TriggerResult::Rejected(PreparationFailure::TargetUnavailable);
            }
        };

        let flow = match transition(FlowState::Idle, FlowEvent::TriggerAccepted)
            .and_then(|state| transition(state, FlowEvent::TargetCaptured))
        {
            Ok(flow) => flow,
            Err(_) => {
                self.shared.finish(SessionCompletion::PreparationFailed(
                    PreparationFailure::InternalInvariant,
                ));
                return TriggerResult::Rejected(PreparationFailure::InternalInvariant);
            }
        };

        let context = SessionContext {
            ports: self.ports.clone(),
            config: self.config,
            shared: Arc::clone(&self.shared),
            cancellation,
            original_target: target,
            flow,
        };
        let shared = Arc::clone(&self.shared);
        let spawn = thread::Builder::new()
            .name("cliptype-injection".to_owned())
            .spawn(move || worker_entry(context, shared));

        match spawn {
            Ok(worker) => {
                *worker_slot = Some(worker);
                TriggerResult::Started { generation }
            }
            Err(_) => {
                self.shared.finish(SessionCompletion::PreparationFailed(
                    PreparationFailure::InternalInvariant,
                ));
                TriggerResult::StartFailed
            }
        }
    }

    pub fn cancel(&self) -> CancelResult {
        self.shared.request_cancel()
    }

    pub fn wait_for_idle(&self, timeout: Duration) -> WaitResult {
        let Some(deadline) = Instant::now().checked_add(timeout) else {
            return WaitResult::TimedOut;
        };
        let mut state = lock_unpoisoned(&self.shared.state);

        while self.shared.active.load(Ordering::Acquire) {
            let now = Instant::now();
            if now >= deadline {
                return WaitResult::TimedOut;
            }
            let remaining = deadline.saturating_duration_since(now);
            let (next_state, result) = match self.shared.idle.wait_timeout(state, remaining) {
                Ok(value) => value,
                Err(poisoned) => poisoned.into_inner(),
            };
            state = next_state;
            if result.timed_out() && self.shared.active.load(Ordering::Acquire) {
                return WaitResult::TimedOut;
            }
        }
        drop(state);
        self.join_completed_worker();
        WaitResult::Idle
    }

    pub fn shutdown(&self) -> ShutdownResult {
        self.shutdown_with_timeout(self.config.worker_shutdown_grace)
    }

    pub fn shutdown_with_timeout(&self, timeout: Duration) -> ShutdownResult {
        self.shared.shutting_down.store(true, Ordering::Release);
        let _ = self.cancel();
        match self.wait_for_idle(timeout) {
            WaitResult::Idle => ShutdownResult::Complete,
            WaitResult::TimedOut => ShutdownResult::TimedOut,
        }
    }

    fn join_completed_worker(&self) {
        if self.shared.active.load(Ordering::Acquire) {
            return;
        }
        if let Some(worker) = lock_unpoisoned(&self.worker).take() {
            let _ = worker.join();
        }
    }
}

impl Drop for Coordinator {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

struct SessionContext {
    ports: SessionPorts,
    config: P1Config,
    shared: Arc<SharedRuntime>,
    cancellation: Arc<CancellationFlag>,
    original_target: TargetEvidence,
    flow: FlowState,
}

fn worker_entry(mut context: SessionContext, shared: Arc<SharedRuntime>) {
    let completion = catch_unwind(AssertUnwindSafe(|| run_session(&mut context)))
        .unwrap_or(SessionCompletion::Finished(
            TerminalOutcome::InternalInvariant,
        ));
    shared.finish(completion);
}

fn run_session(context: &mut SessionContext) -> SessionCompletion {
    if context.cancellation.is_requested() {
        return SessionCompletion::PreparationFailed(PreparationFailure::Cancelled);
    }

    let capabilities = context.ports.keyboard.capabilities();
    if let Some(failure) = modifier_capability_failure(capabilities) {
        return SessionCompletion::PreparationFailed(failure);
    }

    if let Err(completion) = wait_for_modifier_clear(context) {
        return completion;
    }
    if advance(&mut context.flow, FlowEvent::ModifiersSettled).is_err() {
        return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
    }

    let text = match acquire_clipboard(context) {
        Ok(text) => text,
        Err(completion) => return completion,
    };
    if advance(&mut context.flow, FlowEvent::ClipboardAcquired).is_err() {
        return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
    }

    let plan = match build_keyboard_plan(
        text,
        context.config,
        plan_capabilities(capabilities),
    ) {
        Ok(plan) => plan,
        Err(error) => return SessionCompletion::PreparationFailed(map_plan_error(error)),
    };
    if context.cancellation.is_requested() {
        return SessionCompletion::PreparationFailed(PreparationFailure::Cancelled);
    }
    if context
        .ports
        .target
        .integrity_relation(&context.original_target)
        == IntegrityRelation::KnownRestricted
    {
        return SessionCompletion::PreparationFailed(
            PreparationFailure::KnownSecurityRestriction,
        );
    }
    if advance(&mut context.flow, FlowEvent::PlanReady).is_err() {
        return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
    }
    context.shared.set_phase(SessionPhase::Injecting);

    let batch_count = plan.batch_slices().len();
    for (index, atoms) in plan.batch_slices().enumerate() {
        if context.cancellation.is_requested() {
            return SessionCompletion::Finished(TerminalOutcome::Cancelled);
        }
        if let Err(outcome) = verify_target(&context.ports, &context.original_target) {
            return SessionCompletion::Finished(outcome);
        }
        match context.ports.modifiers.observe_modifiers() {
            ModifierObservation::Clear => {}
            ModifierObservation::Held(_) | ModifierObservation::Unknown => {
                return SessionCompletion::Finished(TerminalOutcome::ModifierConflict);
            }
        }

        let batch = match TextBatch::new(atoms, plan.config().dispatch_batch_limit) {
            Ok(batch) => batch,
            Err(_) => {
                return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
            }
        };
        let native = match context.ports.keyboard.dispatch(batch) {
            Ok(result) => result,
            Err(error) => return SessionCompletion::Finished(map_keyboard_error(error)),
        };
        let observation = dispatch_observation(
            native,
            context
                .ports
                .target
                .integrity_relation(&context.original_target),
        );
        match classify_dispatch(observation) {
            DispatchDecision::Continue => {
                if advance(&mut context.flow, FlowEvent::BatchAccepted).is_err() {
                    return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
                }
                context.shared.increment_batches();
            }
            DispatchDecision::Stop(outcome) => return SessionCompletion::Finished(outcome),
        }

        if index + 1 < batch_count
            && sleep_interruptibly(
                &context.cancellation,
                context.config.keyboard_interval,
                context.config.modifier_poll_interval,
            )
        {
            return SessionCompletion::Finished(TerminalOutcome::Cancelled);
        }
    }

    if advance(&mut context.flow, FlowEvent::AllBatchesComplete).is_err() {
        return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
    }
    SessionCompletion::Finished(TerminalOutcome::Completed)
}

fn modifier_capability_failure(capabilities: KeyboardCapabilities) -> Option<PreparationFailure> {
    match capabilities.modifier_observation {
        CapabilityState::Available => None,
        CapabilityState::Degraded => Some(PreparationFailure::DegradedCapabilityRejected),
        CapabilityState::Unavailable => Some(PreparationFailure::UnsupportedCapability),
    }
}

fn plan_capabilities(capabilities: KeyboardCapabilities) -> PlanCapabilities {
    PlanCapabilities {
        unicode_text: capabilities.unicode_text,
        line_break: capabilities.line_break,
        tab: capabilities.tab,
        modifier_observation: capabilities.modifier_observation,
    }
}

fn wait_for_modifier_clear(context: &SessionContext) -> Result<(), SessionCompletion> {
    let Some(deadline) = Instant::now().checked_add(context.config.modifier_settle_timeout) else {
        return Err(SessionCompletion::PreparationFailed(
            PreparationFailure::InternalInvariant,
        ));
    };

    loop {
        if context.cancellation.is_requested() {
            return Err(SessionCompletion::PreparationFailed(
                PreparationFailure::Cancelled,
            ));
        }
        if context.ports.modifiers.observe_modifiers() == ModifierObservation::Clear {
            return Ok(());
        }
        if let Err(outcome) = verify_target(&context.ports, &context.original_target) {
            return Err(SessionCompletion::Finished(outcome));
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(SessionCompletion::PreparationFailed(
                PreparationFailure::ModifierSettleTimeout,
            ));
        }
        if sleep_interruptibly(
            &context.cancellation,
            context.config.modifier_poll_interval.min(remaining),
            context.config.modifier_poll_interval,
        ) {
            return Err(SessionCompletion::PreparationFailed(
                PreparationFailure::Cancelled,
            ));
        }
    }
}

fn acquire_clipboard(context: &SessionContext) -> Result<SensitiveText, SessionCompletion> {
    let budget = context.config.clipboard_retry;
    let attempts = budget.attempts.get();
    let divisor = u32::try_from(attempts).unwrap_or(u32::MAX);
    let retry_pause = budget.total_window / divisor;
    let Some(deadline) = Instant::now().checked_add(budget.total_window) else {
        return Err(SessionCompletion::PreparationFailed(
            PreparationFailure::InternalInvariant,
        ));
    };

    for attempt in 0..attempts {
        if context.cancellation.is_requested() {
            return Err(SessionCompletion::PreparationFailed(
                PreparationFailure::Cancelled,
            ));
        }

        match context
            .ports
            .clipboard
            .read_current_text(context.config.native_clipboard_limit)
        {
            Ok(text) => return Ok(text),
            Err(error) if clipboard_error_is_retryable(error) && attempt + 1 < attempts => {
                if let Err(outcome) = verify_target(&context.ports, &context.original_target) {
                    return Err(SessionCompletion::Finished(outcome));
                }
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                if sleep_interruptibly(
                    &context.cancellation,
                    retry_pause.min(remaining),
                    context.config.modifier_poll_interval,
                ) {
                    return Err(SessionCompletion::PreparationFailed(
                        PreparationFailure::Cancelled,
                    ));
                }
            }
            Err(error) => {
                return Err(SessionCompletion::PreparationFailed(
                    map_clipboard_error(error),
                ));
            }
        }
    }

    Err(SessionCompletion::PreparationFailed(
        PreparationFailure::ClipboardUnavailable,
    ))
}

const fn clipboard_error_is_retryable(error: ClipboardError) -> bool {
    matches!(
        error,
        ClipboardError::Busy
            | ClipboardError::Native(cliptype_platform::NativeError { .. })
    ) && match error {
        ClipboardError::Busy => true,
        ClipboardError::Native(native) => {
            matches!(native.kind(), NativeErrorKind::TemporarilyUnavailable)
        }
        _ => false,
    }
}

const fn map_clipboard_error(error: ClipboardError) -> PreparationFailure {
    match error {
        ClipboardError::Busy | ClipboardError::Native(_) => {
            PreparationFailure::ClipboardUnavailable
        }
        ClipboardError::Empty => PreparationFailure::ClipboardEmpty,
        ClipboardError::NonText => PreparationFailure::ClipboardNonText,
        ClipboardError::Malformed => PreparationFailure::ClipboardMalformed,
        ClipboardError::TooLarge { .. } => PreparationFailure::PayloadTooLarge,
    }
}

const fn map_plan_error(error: PlanError) -> PreparationFailure {
    match error {
        PlanError::InvalidConfiguration(_) => PreparationFailure::InternalInvariant,
        PlanError::Normalization(NormalizationError::Empty) => PreparationFailure::ClipboardEmpty,
        PlanError::Normalization(NormalizationError::PayloadTooLarge { .. }) => {
            PreparationFailure::PayloadTooLarge
        }
        PlanError::Normalization(
            NormalizationError::UnsupportedControl { .. }
            | NormalizationError::TabRejected { .. },
        )
        | PlanError::CapabilityUnavailable(_) => PreparationFailure::UnsupportedCapability,
        PlanError::CapabilityDegraded(_) => PreparationFailure::DegradedCapabilityRejected,
    }
}

const fn map_keyboard_error(error: KeyboardError) -> TerminalOutcome {
    match error {
        KeyboardError::ModifierStateUnavailable => TerminalOutcome::ModifierConflict,
        KeyboardError::Native(_) => TerminalOutcome::NativeFailure,
        KeyboardError::UnsupportedSemanticElement | KeyboardError::InvalidBatch => {
            TerminalOutcome::InternalInvariant
        }
    }
}

fn verify_target(
    ports: &SessionPorts,
    original: &TargetEvidence,
) -> Result<(), TerminalOutcome> {
    let observed = match ports.target.capture() {
        Ok(observed) => observed,
        Err(TargetCaptureError::Disappeared) => return Err(TerminalOutcome::TargetDisappeared),
        Err(TargetCaptureError::Unavailable | TargetCaptureError::Native(_)) => {
            return Err(TerminalOutcome::TargetEvidenceUnavailable);
        }
    };

    match ports.target.compare(original, &observed) {
        TargetComparison::Same => Ok(()),
        TargetComparison::Changed => Err(TerminalOutcome::TargetChanged),
        TargetComparison::Disappeared => Err(TerminalOutcome::TargetDisappeared),
        TargetComparison::UnavailableOrAmbiguous => {
            Err(TerminalOutcome::TargetEvidenceUnavailable)
        }
    }
}

fn dispatch_observation(
    result: DispatchResult,
    integrity: IntegrityRelation,
) -> DispatchObservation {
    match result {
        DispatchResult::Complete { .. } => DispatchObservation::CompleteBatch,
        DispatchResult::Partial { .. } => DispatchObservation::Partial,
        DispatchResult::ProgressUnknown { .. } => DispatchObservation::ProgressUnknown,
        DispatchResult::NoneAccepted { native, .. } => {
            if integrity == IntegrityRelation::KnownRestricted {
                DispatchObservation::NoEvents(NoInputReason::KnownSecurityRestriction)
            } else if native.is_some_and(|error| {
                matches!(
                    error.kind(),
                    NativeErrorKind::BlockedCauseUnknown | NativeErrorKind::PermissionDenied
                )
            }) {
                DispatchObservation::NoEvents(NoInputReason::BlockedCauseUnknown)
            } else {
                DispatchObservation::NoEvents(NoInputReason::NativeFailure)
            }
        }
    }
}

fn advance(flow: &mut FlowState, event: FlowEvent) -> Result<(), ()> {
    match transition(*flow, event) {
        Ok(next) => {
            *flow = next;
            Ok(())
        }
        Err(_) => Err(()),
    }
}

fn sleep_interruptibly(
    cancellation: &CancellationFlag,
    duration: Duration,
    quantum: Duration,
) -> bool {
    let Some(deadline) = Instant::now().checked_add(duration) else {
        return true;
    };
    while !cancellation.is_requested() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        thread::sleep(remaining.min(quantum));
    }
    true
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
