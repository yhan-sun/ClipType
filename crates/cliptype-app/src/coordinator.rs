//! Native-neutral live coordinator for keyboard, code-keyboard, and
//! current-clipboard paste paths.

use std::{
    collections::VecDeque,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use cliptype_core::{
    AutoClipboardThreshold, CapabilityState, CodeAction, CodePlan, ConfigError, DispatchDecision,
    DispatchObservation, FlowEvent, FlowState, InjectionBackend, InjectionPlan, IntegrityRelation,
    NoInputReason, NormalizationError, P1Config, PlanCapabilities, PlanError, PreparationFailure,
    ProductCapabilities, ProductConfig, ProductConfigError, ProductPlanError, SessionPhase,
    TerminalOutcome, TextAtom, TextBatch, build_injection_plan, classify_dispatch, transition,
};
use cliptype_platform::{
    ClipboardError, ClipboardPort, ClipboardRevision, ClipboardSnapshot, DispatchResult,
    KeyboardCapabilities, KeyboardError, KeyboardPort, ModifierObservation, ModifierPort,
    NativeErrorKind, PasteCapabilities, PasteError, PastePort, TargetCaptureError,
    TargetComparison, TargetEvidence, TargetPort,
};

use crate::CancellationFlag;

static TYPING_SEED_COUNTER: AtomicU64 = AtomicU64::new(0x9E37_79B9_7F4A_7C15);

// Code mode relies on the destination editor's asynchronous auto-pair and
// auto-indent handlers. Keep a bounded gap after every queued action so a
// generated closer exists before the following CursorRight action is posted.
// This is deliberately local to Code mode; normal Keyboard mode retains its
// configured action rate.
const CODE_ACTION_SETTLE_INTERVAL: Duration = Duration::from_millis(8);

// A destination editor may publish an auto-completed pair after accepting the
// opener's synthetic Unicode event. Navigation actions are the only actions
// that depend on that generated state, so give them a larger Code-only barrier
// without slowing ordinary Keyboard, Clipboard, or Auto sessions.
const CODE_NAVIGATION_SETTLE_INTERVAL: Duration = Duration::from_millis(40);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCompletion {
    PreparationFailed(PreparationFailure),
    Finished(TerminalOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub generation: u64,
    pub phase: SessionPhase,
    pub backend: Option<InjectionBackend>,
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
    paste: Arc<dyn PastePort>,
}

struct RuntimeState {
    generation: u64,
    phase: SessionPhase,
    backend: Option<InjectionBackend>,
    completion: Option<SessionCompletion>,
    batches_completed: u32,
    cancellation: Option<Arc<CancellationFlag>>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            generation: 0,
            phase: SessionPhase::Idle,
            backend: None,
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
        state.backend = None;
        state.completion = None;
        state.batches_completed = 0;
        state.cancellation = Some(Arc::clone(&cancellation));
        (state.generation, cancellation)
    }

    fn set_phase(&self, phase: SessionPhase) {
        lock_unpoisoned(&self.state).phase = phase;
    }

    fn set_backend(&self, backend: InjectionBackend) {
        lock_unpoisoned(&self.state).backend = Some(backend);
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
            backend: state.backend,
            completion: state.completion,
            batches_completed: state.batches_completed,
        }
    }
}

/// Owns exactly one live bounded injection session and its worker lifecycle.
pub struct Coordinator {
    ports: SessionPorts,
    config: RwLock<ProductConfig>,
    shared: Arc<SharedRuntime>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl Coordinator {
    /// P1-compatible keyboard-only constructor.
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
        let config = config.validate()?;
        let product = ProductConfig {
            enabled: true,
            mode: cliptype_core::InjectionMode::Keyboard,
            auto_clipboard_threshold: AutoClipboardThreshold::new(256)
                .expect("legacy keyboard threshold is non-zero"),
            jitter_percent: 0,
            typo_probability_percent: 0,
            safety: config,
        };

        Ok(Self::from_validated_ports(
            Arc::new(clipboard),
            Arc::new(target),
            Arc::new(keyboard),
            Arc::new(modifiers),
            Arc::new(UnavailablePaste),
            product,
        ))
    }

    /// P1-compatible keyboard-only constructor using erased ports.
    pub fn from_ports(
        clipboard: Arc<dyn ClipboardPort>,
        target: Arc<dyn TargetPort>,
        keyboard: Arc<dyn KeyboardPort>,
        modifiers: Arc<dyn ModifierPort>,
        config: P1Config,
    ) -> Result<Self, ConfigError> {
        let config = config.validate()?;
        let product = ProductConfig {
            enabled: true,
            mode: cliptype_core::InjectionMode::Keyboard,
            auto_clipboard_threshold: AutoClipboardThreshold::new(256)
                .expect("legacy keyboard threshold is non-zero"),
            jitter_percent: 0,
            typo_probability_percent: 0,
            safety: config,
        };

        Ok(Self::from_validated_ports(
            clipboard,
            target,
            keyboard,
            modifiers,
            Arc::new(UnavailablePaste),
            product,
        ))
    }

    /// Product constructor with keyboard and current-clipboard paste backends.
    pub fn new_product<C, T, K, M, P>(
        clipboard: C,
        target: T,
        keyboard: K,
        modifiers: M,
        paste: P,
        config: ProductConfig,
    ) -> Result<Self, ProductConfigError>
    where
        C: ClipboardPort + 'static,
        T: TargetPort + 'static,
        K: KeyboardPort + 'static,
        M: ModifierPort + 'static,
        P: PastePort + 'static,
    {
        Self::from_product_ports(
            Arc::new(clipboard),
            Arc::new(target),
            Arc::new(keyboard),
            Arc::new(modifiers),
            Arc::new(paste),
            config,
        )
    }

    pub fn from_product_ports(
        clipboard: Arc<dyn ClipboardPort>,
        target: Arc<dyn TargetPort>,
        keyboard: Arc<dyn KeyboardPort>,
        modifiers: Arc<dyn ModifierPort>,
        paste: Arc<dyn PastePort>,
        config: ProductConfig,
    ) -> Result<Self, ProductConfigError> {
        let config = config.validate()?;
        Ok(Self::from_validated_ports(
            clipboard, target, keyboard, modifiers, paste, config,
        ))
    }

    fn from_validated_ports(
        clipboard: Arc<dyn ClipboardPort>,
        target: Arc<dyn TargetPort>,
        keyboard: Arc<dyn KeyboardPort>,
        modifiers: Arc<dyn ModifierPort>,
        paste: Arc<dyn PastePort>,
        config: ProductConfig,
    ) -> Self {
        Self {
            ports: SessionPorts {
                clipboard,
                target,
                keyboard,
                modifiers,
                paste,
            },
            config: RwLock::new(config),
            shared: Arc::new(SharedRuntime::new()),
            worker: Mutex::new(None),
        }
    }

    /// Returns the configuration used by future sessions.
    pub fn config(&self) -> ProductConfig {
        *read_unpoisoned(&self.config)
    }

    /// Updates only future sessions. An active session retains its old snapshot.
    pub fn update_config(&self, config: ProductConfig) -> Result<(), ProductConfigError> {
        let config = config.validate()?;
        *write_unpoisoned(&self.config) = config;
        Ok(())
    }

    pub fn status(&self) -> StatusSnapshot {
        self.shared.snapshot()
    }

    pub fn trigger(&self) -> TriggerResult {
        if self.shared.shutting_down.load(Ordering::Acquire) {
            return TriggerResult::ShuttingDown;
        }

        let config = self.config();
        if !config.enabled {
            return TriggerResult::Rejected(PreparationFailure::Disabled);
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
            config,
            shared: Arc::clone(&self.shared),
            cancellation,
            original_target: target,
            flow,
            typing_seed: next_typing_seed(generation),
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
        self.shutdown_with_timeout(self.config().safety.worker_shutdown_grace)
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
    config: ProductConfig,
    shared: Arc<SharedRuntime>,
    cancellation: Arc<CancellationFlag>,
    original_target: TargetEvidence,
    flow: FlowState,
    typing_seed: u64,
}

fn worker_entry(mut context: SessionContext, shared: Arc<SharedRuntime>) {
    let completion = catch_unwind(AssertUnwindSafe(|| run_session(&mut context))).unwrap_or(
        SessionCompletion::Finished(TerminalOutcome::InternalInvariant),
    );
    shared.finish(completion);
}

fn run_session(context: &mut SessionContext) -> SessionCompletion {
    if context.cancellation.is_requested() {
        return SessionCompletion::PreparationFailed(PreparationFailure::Cancelled);
    }

    let keyboard_capabilities = context.ports.keyboard.capabilities();
    if let Some(failure) = modifier_capability_failure(keyboard_capabilities) {
        return SessionCompletion::PreparationFailed(failure);
    }

    if let Err(completion) = wait_for_modifier_clear(context) {
        return completion;
    }
    if advance(&mut context.flow, FlowEvent::ModifiersSettled).is_err() {
        return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
    }

    let snapshot = match acquire_clipboard(context) {
        Ok(snapshot) => snapshot,
        Err(completion) => return completion,
    };
    if advance(&mut context.flow, FlowEvent::ClipboardAcquired).is_err() {
        return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
    }

    let revision = snapshot.revision();
    let (text, _) = snapshot.into_parts();
    let paste_capabilities = context.ports.paste.capabilities();
    let plan = match build_injection_plan(
        text,
        revision.is_known(),
        context.config,
        product_capabilities(keyboard_capabilities, paste_capabilities),
    ) {
        Ok(plan) => plan,
        Err(error) => return SessionCompletion::PreparationFailed(map_product_plan_error(error)),
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
        return SessionCompletion::PreparationFailed(PreparationFailure::KnownSecurityRestriction);
    }
    if advance(&mut context.flow, FlowEvent::PlanReady).is_err() {
        return SessionCompletion::Finished(TerminalOutcome::InternalInvariant);
    }

    context.shared.set_backend(plan.backend());
    context.shared.set_phase(SessionPhase::Injecting);

    match plan {
        InjectionPlan::Keyboard(plan) => run_keyboard_plan(context, &plan),
        InjectionPlan::Clipboard(_) => run_clipboard_plan(context, revision),
        InjectionPlan::Code(plan) => run_code_plan(context, &plan),
    }
}

fn run_keyboard_plan(
    context: &mut SessionContext,
    plan: &cliptype_core::KeyboardPlan,
) -> SessionCompletion {
    let mut random = TypingRandom::new(context.typing_seed);

    for atom in plan.text().atoms().iter().copied() {
        if let Some(wrong) =
            adjacent_typo(atom, context.config.typo_probability_percent, &mut random)
        {
            if let Err(outcome) = dispatch_timed_action(
                context,
                plan.config(),
                KeyboardAction::Atom(wrong),
                &mut random,
            ) {
                return SessionCompletion::Finished(outcome);
            }
            if let Err(outcome) = dispatch_timed_action(
                context,
                plan.config(),
                KeyboardAction::Backspace,
                &mut random,
            ) {
                return SessionCompletion::Finished(outcome);
            }
        }

        if let Err(outcome) = dispatch_timed_action(
            context,
            plan.config(),
            KeyboardAction::Atom(atom),
            &mut random,
        ) {
            return SessionCompletion::Finished(outcome);
        }
    }

    complete_flow(context)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardAction {
    Atom(TextAtom),
    Backspace,
    CursorRight,
    CursorRightToLineEnd,
}

fn run_code_plan(context: &mut SessionContext, plan: &CodePlan) -> SessionCompletion {
    let mut random = TypingRandom::new(context.typing_seed);
    let mut queue: VecDeque<CodeAction> = plan.actions().iter().copied().collect();

    // This is a strict FIFO action queue. `CursorRight` is already the core
    // planner's decision for the current source closer; we never inspect the
    // destination editor or infer its text.
    while let Some(action) = queue.pop_front() {
        let action = match action {
            CodeAction::Atom(atom) => KeyboardAction::Atom(atom),
            CodeAction::CursorRight => KeyboardAction::CursorRight,
            CodeAction::CursorRightToLineEnd => KeyboardAction::CursorRightToLineEnd,
        };

        if matches!(
            action,
            KeyboardAction::CursorRight | KeyboardAction::CursorRightToLineEnd
        ) && sleep_interruptibly(
            &context.cancellation,
            CODE_NAVIGATION_SETTLE_INTERVAL,
            context.config.safety.modifier_poll_interval,
        ) {
            return SessionCompletion::Finished(TerminalOutcome::Cancelled);
        }

        if let Err(outcome) = dispatch_timed_action(context, plan.config(), action, &mut random) {
            return SessionCompletion::Finished(outcome);
        }

        if !queue.is_empty()
            && sleep_interruptibly(
                &context.cancellation,
                CODE_ACTION_SETTLE_INTERVAL,
                context.config.safety.modifier_poll_interval,
            )
        {
            return SessionCompletion::Finished(TerminalOutcome::Cancelled);
        }
    }

    complete_flow(context)
}

fn dispatch_timed_action(
    context: &mut SessionContext,
    config: P1Config,
    action: KeyboardAction,
    random: &mut TypingRandom,
) -> Result<(), TerminalOutcome> {
    verify_action_preconditions(context)?;

    let delay = jittered_delay(
        context.config.safety.keyboard_interval,
        context.config.jitter_percent,
        random,
    );
    if sleep_interruptibly(
        &context.cancellation,
        delay,
        context.config.safety.modifier_poll_interval,
    ) {
        return Err(TerminalOutcome::Cancelled);
    }

    // The target and physical modifiers can change while the humanized delay
    // elapses, so re-check immediately before every native action.
    verify_action_preconditions(context)?;

    let native = match action {
        KeyboardAction::Atom(atom) => {
            let batch = TextBatch::new(std::slice::from_ref(&atom), config.dispatch_batch_limit)
                .map_err(|_| TerminalOutcome::InternalInvariant)?;
            context
                .ports
                .keyboard
                .dispatch(batch)
                .map_err(map_keyboard_error)?
        }
        KeyboardAction::Backspace => context
            .ports
            .keyboard
            .dispatch_backspace()
            .map_err(map_keyboard_error)?,
        KeyboardAction::CursorRight => context
            .ports
            .keyboard
            .dispatch_cursor_right()
            .map_err(map_keyboard_error)?,
        KeyboardAction::CursorRightToLineEnd => context
            .ports
            .keyboard
            .dispatch_cursor_right_to_line_end()
            .map_err(map_keyboard_error)?,
    };
    accept_dispatch(context, native)
}

fn verify_action_preconditions(context: &SessionContext) -> Result<(), TerminalOutcome> {
    if context.cancellation.is_requested() {
        return Err(TerminalOutcome::Cancelled);
    }
    verify_target(&context.ports, &context.original_target)?;
    if context.ports.modifiers.observe_modifiers() != ModifierObservation::Clear {
        return Err(TerminalOutcome::ModifierConflict);
    }
    Ok(())
}

fn adjacent_typo(
    atom: TextAtom,
    probability_percent: u8,
    random: &mut TypingRandom,
) -> Option<TextAtom> {
    if probability_percent == 0 || random.below(100) >= u64::from(probability_percent) {
        return None;
    }
    let TextAtom::Scalar(value) = atom else {
        return None;
    };
    if !value.is_ascii() {
        return None;
    }

    let lower = value.to_ascii_lowercase();
    let candidates: &[u8] = match lower {
        '1' => b"2q",
        '2' => b"13qw",
        '3' => b"24we",
        '4' => b"35er",
        '5' => b"46rt",
        '6' => b"57ty",
        '7' => b"68yu",
        '8' => b"79ui",
        '9' => b"80io",
        '0' => b"9-op",
        'q' => b"12wa",
        'w' => b"23qesa",
        'e' => b"34wrsd",
        'r' => b"45etdf",
        't' => b"56ryfg",
        'y' => b"67tugh",
        'u' => b"78yihj",
        'i' => b"89uojk",
        'o' => b"90ipkl",
        'p' => b"0-[ol",
        'a' => b"qwsz",
        's' => b"weadzx",
        'd' => b"ersfxc",
        'f' => b"rtdgcv",
        'g' => b"tyfhvb",
        'h' => b"yugjbn",
        'j' => b"uihknm",
        'k' => b"iojlm,",
        'l' => b"opk;.,",
        'z' => b"asx",
        'x' => b"zsdc",
        'c' => b"xdfv",
        'v' => b"cfgb",
        'b' => b"vghn",
        'n' => b"bhjm",
        'm' => b"njk,",
        '-' => b"0=p",
        '=' => b"-[",
        '[' => b"p];",
        ']' => b"[\'",
        ';' => b"lp.'",
        '\'' => b";]/",
        ',' => b"mkl.",
        '.' => b",l;/",
        '/' => b".;'",
        _ => return None,
    };
    let selected = char::from(candidates[random.below(candidates.len() as u64) as usize]);
    let selected = if value.is_ascii_uppercase() {
        selected.to_ascii_uppercase()
    } else {
        selected
    };
    Some(TextAtom::Scalar(selected))
}

fn jittered_delay(base: Duration, jitter_percent: u8, random: &mut TypingRandom) -> Duration {
    if jitter_percent == 0 || base.is_zero() {
        return base;
    }

    let spread = i64::from(jitter_percent);
    let width = u64::try_from(spread.saturating_mul(2).saturating_add(1)).unwrap_or(1);
    let offset = i64::try_from(random.below(width)).unwrap_or(0) - spread;
    let factor = u128::try_from(100_i64.saturating_add(offset)).unwrap_or(1);
    let nanos = base.as_nanos().saturating_mul(factor) / 100;
    Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
}

struct TypingRandom {
    state: u64,
}

impl TypingRandom {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xA076_1D64_78BD_642F
            } else {
                seed
            },
        }
    }

    fn next(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, upper: u64) -> u64 {
        if upper <= 1 { 0 } else { self.next() % upper }
    }
}

fn next_typing_seed(generation: u64) -> u64 {
    let clock = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    clock
        ^ generation.rotate_left(17)
        ^ TYPING_SEED_COUNTER.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed)
}

fn run_clipboard_plan(
    context: &mut SessionContext,
    revision: ClipboardRevision,
) -> SessionCompletion {
    if context.cancellation.is_requested() {
        return SessionCompletion::Finished(TerminalOutcome::Cancelled);
    }
    if let Err(outcome) = verify_target(&context.ports, &context.original_target) {
        return SessionCompletion::Finished(outcome);
    }
    if context.ports.modifiers.observe_modifiers() != ModifierObservation::Clear {
        return SessionCompletion::Finished(TerminalOutcome::ModifierConflict);
    }

    let native = match context.ports.paste.dispatch_paste(revision) {
        Ok(result) => result,
        Err(error) => return SessionCompletion::Finished(map_paste_error(error)),
    };
    if let Err(outcome) = accept_dispatch(context, native) {
        return SessionCompletion::Finished(outcome);
    }

    complete_flow(context)
}

fn accept_dispatch(
    context: &mut SessionContext,
    native: DispatchResult,
) -> Result<(), TerminalOutcome> {
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
                return Err(TerminalOutcome::InternalInvariant);
            }
            context.shared.increment_batches();
            Ok(())
        }
        DispatchDecision::Stop(outcome) => Err(outcome),
    }
}

fn complete_flow(context: &mut SessionContext) -> SessionCompletion {
    if advance(&mut context.flow, FlowEvent::AllBatchesComplete).is_err() {
        SessionCompletion::Finished(TerminalOutcome::InternalInvariant)
    } else {
        SessionCompletion::Finished(TerminalOutcome::Completed)
    }
}

fn modifier_capability_failure(capabilities: KeyboardCapabilities) -> Option<PreparationFailure> {
    match capabilities.modifier_observation {
        CapabilityState::Available => None,
        CapabilityState::Degraded => Some(PreparationFailure::DegradedCapabilityRejected),
        CapabilityState::Unavailable => Some(PreparationFailure::UnsupportedCapability),
    }
}

fn product_capabilities(
    keyboard: KeyboardCapabilities,
    paste: PasteCapabilities,
) -> ProductCapabilities {
    ProductCapabilities {
        keyboard: PlanCapabilities {
            unicode_text: keyboard.unicode_text,
            line_break: keyboard.line_break,
            tab: keyboard.tab,
            cursor_right: keyboard.cursor_right,
            modifier_observation: keyboard.modifier_observation,
        },
        clipboard_paste: paste.paste_chord,
        clipboard_revision_guard: paste.clipboard_revision_guard,
    }
}

fn wait_for_modifier_clear(context: &SessionContext) -> Result<(), SessionCompletion> {
    let Some(deadline) = Instant::now().checked_add(context.config.safety.modifier_settle_timeout)
    else {
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
            context.config.safety.modifier_poll_interval.min(remaining),
            context.config.safety.modifier_poll_interval,
        ) {
            return Err(SessionCompletion::PreparationFailed(
                PreparationFailure::Cancelled,
            ));
        }
    }
}

fn acquire_clipboard(context: &SessionContext) -> Result<ClipboardSnapshot, SessionCompletion> {
    let budget = context.config.safety.clipboard_retry;
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
            .read_current_snapshot(context.config.safety.native_clipboard_limit)
        {
            Ok(snapshot) => return Ok(snapshot),
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
                    context.config.safety.modifier_poll_interval,
                ) {
                    return Err(SessionCompletion::PreparationFailed(
                        PreparationFailure::Cancelled,
                    ));
                }
            }
            Err(error) => {
                return Err(SessionCompletion::PreparationFailed(map_clipboard_error(
                    error,
                )));
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
            | ClipboardError::ChangedDuringRead
            | ClipboardError::Native(cliptype_platform::NativeError { .. })
    ) && match error {
        ClipboardError::Busy | ClipboardError::ChangedDuringRead => true,
        ClipboardError::Native(native) => {
            matches!(native.kind(), NativeErrorKind::TemporarilyUnavailable)
        }
        _ => false,
    }
}

const fn map_clipboard_error(error: ClipboardError) -> PreparationFailure {
    match error {
        ClipboardError::Busy | ClipboardError::ChangedDuringRead | ClipboardError::Native(_) => {
            PreparationFailure::ClipboardUnavailable
        }
        ClipboardError::Empty => PreparationFailure::ClipboardEmpty,
        ClipboardError::NonText => PreparationFailure::ClipboardNonText,
        ClipboardError::Malformed => PreparationFailure::ClipboardMalformed,
        ClipboardError::TooLarge { .. } => PreparationFailure::PayloadTooLarge,
    }
}

const fn map_product_plan_error(error: ProductPlanError) -> PreparationFailure {
    match error {
        ProductPlanError::Disabled => PreparationFailure::Disabled,
        ProductPlanError::InvalidConfiguration(_) => PreparationFailure::InternalInvariant,
        ProductPlanError::Empty => PreparationFailure::ClipboardEmpty,
        ProductPlanError::PayloadTooLarge { .. } => PreparationFailure::PayloadTooLarge,
        ProductPlanError::Keyboard(error) => map_plan_error(error),
        ProductPlanError::ClipboardCapabilityUnavailable => {
            PreparationFailure::UnsupportedCapability
        }
        ProductPlanError::ClipboardCapabilityDegraded => {
            PreparationFailure::DegradedCapabilityRejected
        }
        ProductPlanError::ClipboardRevisionUnavailable => {
            PreparationFailure::ClipboardRevisionUnavailable
        }
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
            NormalizationError::UnsupportedControl { .. } | NormalizationError::TabRejected { .. },
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

const fn map_paste_error(error: PasteError) -> TerminalOutcome {
    match error {
        PasteError::ClipboardChanged => TerminalOutcome::ClipboardChanged,
        PasteError::Native(_) => TerminalOutcome::NativeFailure,
        PasteError::Unsupported | PasteError::InvalidRequest => TerminalOutcome::InternalInvariant,
    }
}

fn verify_target(ports: &SessionPorts, original: &TargetEvidence) -> Result<(), TerminalOutcome> {
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
        TargetComparison::UnavailableOrAmbiguous => Err(TerminalOutcome::TargetEvidenceUnavailable),
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

fn read_unpoisoned<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_unpoisoned<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[derive(Debug, Default)]
struct UnavailablePaste;

impl PastePort for UnavailablePaste {
    fn capabilities(&self) -> PasteCapabilities {
        PasteCapabilities {
            paste_chord: CapabilityState::Unavailable,
            clipboard_revision_guard: CapabilityState::Unavailable,
        }
    }

    fn dispatch_paste(
        &self,
        _expected_revision: ClipboardRevision,
    ) -> Result<DispatchResult, PasteError> {
        Err(PasteError::Unsupported)
    }
}

#[cfg(test)]
mod human_typing_tests {
    use std::time::Duration;

    use cliptype_core::TextAtom;

    use super::{TypingRandom, adjacent_typo, jittered_delay};

    #[test]
    fn jitter_is_bounded_around_every_base_interval() {
        let base = Duration::from_millis(100);
        let mut random = TypingRandom::new(7);

        for _ in 0..1_000 {
            let delay = jittered_delay(base, 20, &mut random);
            assert!(delay >= Duration::from_millis(80));
            assert!(delay <= Duration::from_millis(120));
        }
    }

    #[test]
    fn adjacent_typos_are_ascii_only_and_preserve_case() {
        let mut random = TypingRandom::new(11);
        let lower = adjacent_typo(TextAtom::Scalar('g'), 100, &mut random);
        let mut random = TypingRandom::new(11);
        let upper = adjacent_typo(TextAtom::Scalar('G'), 100, &mut random);

        assert_eq!(
            lower
                .and_then(TextAtom::exposed_scalar)
                .map(|value| value.to_ascii_uppercase()),
            upper.and_then(TextAtom::exposed_scalar)
        );
        let mut random = TypingRandom::new(1);
        assert_eq!(adjacent_typo(TextAtom::Scalar('你'), 25, &mut random), None);
        assert_eq!(adjacent_typo(TextAtom::LineBreak, 25, &mut random), None);
        assert_eq!(adjacent_typo(TextAtom::Tab, 25, &mut random), None);
    }

    #[test]
    fn zero_probability_and_zero_jitter_are_exact() {
        let mut random = TypingRandom::new(3);
        assert_eq!(adjacent_typo(TextAtom::Scalar('a'), 0, &mut random), None);
        assert_eq!(
            jittered_delay(Duration::from_millis(37), 0, &mut random),
            Duration::from_millis(37)
        );
    }
}
