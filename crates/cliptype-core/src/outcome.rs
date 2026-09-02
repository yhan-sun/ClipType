//! Capability, evidence, and session outcome vocabulary.

/// Availability of a platform capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityState {
    Available,
    Degraded,
    Unavailable,
}

/// Strength of non-content target evidence exposed by an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceStrength {
    TopLevelTarget,
    NativeFocusedControl,
    RenderHostLimited,
    Degraded,
}

/// Evidence-based relationship between ClipType and a target's integrity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityRelation {
    KnownRestricted,
    KnownNotRestricted,
    Unknown,
}

/// Live session phase exposed to a content-free status surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Idle,
    Preparing,
    Injecting,
    Cancelling,
}

/// Content-free failures that can occur before native dispatch begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationFailure {
    Disabled,
    Busy,
    ClipboardUnavailable,
    ClipboardRevisionUnavailable,
    ClipboardEmpty,
    ClipboardNonText,
    ClipboardMalformed,
    PayloadTooLarge,
    UnsupportedCapability,
    DegradedCapabilityRejected,
    TargetUnavailable,
    ModifierSettleTimeout,
    KnownSecurityRestriction,
    Cancelled,
    InternalInvariant,
}

/// Terminal result of one bounded injection attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalOutcome {
    Completed,
    Cancelled,
    ClipboardChanged,
    TargetChanged,
    TargetDisappeared,
    TargetEvidenceUnavailable,
    ModifierConflict,
    ModifierSettleTimeout,
    KnownSecurityRestriction,
    BlockedCauseUnknown,
    PartialInput,
    ProgressUnknown,
    NativeFailure,
    InternalInvariant,
}

/// ClipType never retries synthetic input after an observed native result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDisposition {
    Never,
}

impl TerminalOutcome {
    pub const fn retry_disposition(self) -> RetryDisposition {
        let _ = self;
        RetryDisposition::Never
    }
}

#[cfg(test)]
mod tests {
    use super::{RetryDisposition, TerminalOutcome};

    #[test]
    fn partial_and_unknown_input_are_never_retryable() {
        assert_eq!(
            TerminalOutcome::PartialInput.retry_disposition(),
            RetryDisposition::Never
        );
        assert_eq!(
            TerminalOutcome::ProgressUnknown.retry_disposition(),
            RetryDisposition::Never
        );
        assert_eq!(
            TerminalOutcome::ClipboardChanged.retry_disposition(),
            RetryDisposition::Never
        );
    }
}
