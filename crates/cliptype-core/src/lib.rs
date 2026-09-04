//! Platform-independent domain values and pure product policy for ClipType.
//!
//! This crate contains no operating-system API types. Clipboard text is
//! sensitive by default, native work is bounded, and uncertain platform
//! evidence is represented explicitly rather than collapsed into booleans.

#![forbid(unsafe_code)]

mod cancellation;
mod config;
mod hotkey;
mod limits;
mod normalization;
mod outcome;
mod plan;
mod product;
mod sensitive;
mod settings;
mod state;
mod text;

pub use cancellation::CancellationProbe;
pub use config::{
    ConfigError, ConfigField, FocusPolicy, NewlinePolicy, P1Config, RetryBudget, TabPolicy,
};
pub use hotkey::{
    HotkeyApplyResult, HotkeyAvailability, HotkeyKey, HotkeyModifiers, HotkeyPair,
    HotkeyParseError, HotkeyPlatform, HotkeySpec, HotkeyValidationError,
};
pub use limits::{
    BoundError, BoundKind, ByteCount, DispatchBatchLimit, NativeByteLimit, NativeEventCount,
    RetryAttemptLimit, SemanticElementCount, SemanticElementLimit, Utf16UnitCount,
};
pub use normalization::{NormalizationError, NormalizedText, normalize_text};
pub use outcome::{
    CapabilityState, EvidenceStrength, IntegrityRelation, PreparationFailure, RetryDisposition,
    SessionPhase, TerminalOutcome,
};
pub use plan::{
    CapabilityRequirement, KeyboardPlan, PlanCapabilities, PlanError, build_keyboard_plan,
};
pub use product::{
    AutoClipboardThreshold, ClipboardPlan, CodeAction, CodePlan, InjectionBackend, InjectionMode,
    InjectionPlan, ProductCapabilities, ProductConfig, ProductConfigError, ProductPlanError,
    build_injection_plan,
};
pub use sensitive::SensitiveText;
pub use settings::{
    MAX_CHARACTERS_PER_SECOND, MAX_JITTER_PERCENT, MAX_TYPO_PROBABILITY_PERCENT,
    MIN_CHARACTERS_PER_SECOND, ProductSettings, SETTINGS_SCHEMA_VERSION, SettingsValidationError,
    SpeedPreset,
};
pub use state::{
    DispatchDecision, DispatchObservation, FlowEvent, FlowState, NoInputReason, PreparationStage,
    TransitionError, TriggerDecision, classify_dispatch, decide_trigger, transition,
};
pub use text::{TextAtom, TextBatch, TextBatchError};
