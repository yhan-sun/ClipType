//! Platform-independent domain values and pure P1 contracts for ClipType.
//!
//! This crate contains no operating-system API types. Clipboard text is
//! sensitive by default, native work is bounded, and uncertain platform
//! evidence is represented explicitly rather than collapsed into booleans.

#![forbid(unsafe_code)]

mod cancellation;
mod config;
mod limits;
mod outcome;
mod sensitive;
mod text;

pub use cancellation::CancellationProbe;
pub use config::{
    ConfigError, ConfigField, FocusPolicy, NewlinePolicy, P1Config, RetryBudget, TabPolicy,
};
pub use limits::{
    BoundError, BoundKind, ByteCount, DispatchBatchLimit, NativeByteLimit, NativeEventCount,
    RetryAttemptLimit, SemanticElementCount, SemanticElementLimit, Utf16UnitCount,
};
pub use outcome::{
    CapabilityState, EvidenceStrength, IntegrityRelation, PreparationFailure, RetryDisposition,
    SessionPhase, TerminalOutcome,
};
pub use sensitive::SensitiveText;
pub use text::{TextAtom, TextBatch, TextBatchError};
