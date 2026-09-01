//! P1 configuration snapshot and validation.

use std::{fmt, time::Duration};

use crate::{DispatchBatchLimit, NativeByteLimit, RetryAttemptLimit, SemanticElementLimit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPolicy {
    StrictAvailableEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlinePolicy {
    NormalizeToLineBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPolicy {
    Allow,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    DispatchBatchLimit,
    ModifierSettleTimeout,
    ModifierPollInterval,
    ClipboardRetryWindow,
    WorkerShutdownGrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    ZeroDuration(ConfigField),
    BatchExceedsPayload,
    ModifierPollExceedsSettle,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDuration(field) => write!(formatter, "{field:?} must be non-zero"),
            Self::BatchExceedsPayload => {
                formatter.write_str("dispatch batch limit exceeds total payload limit")
            }
            Self::ModifierPollExceedsSettle => {
                formatter.write_str("modifier poll interval exceeds settle timeout")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Bounded retry policy owned by the live coordinator, not native adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryBudget {
    pub attempts: RetryAttemptLimit,
    pub total_window: Duration,
}

impl RetryBudget {
    pub fn new(attempts: RetryAttemptLimit, total_window: Duration) -> Result<Self, ConfigError> {
        if total_window.is_zero() {
            return Err(ConfigError::ZeroDuration(ConfigField::ClipboardRetryWindow));
        }

        Ok(Self {
            attempts,
            total_window,
        })
    }
}

/// Immutable P1 safety and work bounds captured for an injection session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P1Config {
    pub native_clipboard_limit: NativeByteLimit,
    pub total_payload_limit: SemanticElementLimit,
    pub dispatch_batch_limit: DispatchBatchLimit,
    pub keyboard_interval: Duration,
    pub modifier_settle_timeout: Duration,
    pub modifier_poll_interval: Duration,
    pub clipboard_retry: RetryBudget,
    pub worker_shutdown_grace: Duration,
    pub focus_policy: FocusPolicy,
    pub newline_policy: NewlinePolicy,
    pub tab_policy: TabPolicy,
}

impl P1Config {
    pub fn validate(self) -> Result<Self, ConfigError> {
        if self.dispatch_batch_limit.get() > self.total_payload_limit.get() {
            return Err(ConfigError::BatchExceedsPayload);
        }
        if self.modifier_settle_timeout.is_zero() {
            return Err(ConfigError::ZeroDuration(
                ConfigField::ModifierSettleTimeout,
            ));
        }
        if self.modifier_poll_interval.is_zero() {
            return Err(ConfigError::ZeroDuration(ConfigField::ModifierPollInterval));
        }
        if self.modifier_poll_interval > self.modifier_settle_timeout {
            return Err(ConfigError::ModifierPollExceedsSettle);
        }
        if self.worker_shutdown_grace.is_zero() {
            return Err(ConfigError::ZeroDuration(ConfigField::WorkerShutdownGrace));
        }

        RetryBudget::new(
            self.clipboard_retry.attempts,
            self.clipboard_retry.total_window,
        )?;
        Ok(self)
    }
}

impl Default for P1Config {
    fn default() -> Self {
        Self {
            native_clipboard_limit: NativeByteLimit::new(8 * 1024 * 1024)
                .expect("P1 native clipboard limit is non-zero"),
            total_payload_limit: SemanticElementLimit::new(65_536)
                .expect("P1 payload limit is non-zero"),
            dispatch_batch_limit: DispatchBatchLimit::new(8)
                .expect("P1 dispatch batch limit is non-zero"),
            keyboard_interval: Duration::from_millis(1),
            modifier_settle_timeout: Duration::from_millis(750),
            modifier_poll_interval: Duration::from_millis(5),
            clipboard_retry: RetryBudget::new(
                RetryAttemptLimit::new(8).expect("P1 retry attempts are non-zero"),
                Duration::from_millis(80),
            )
            .expect("P1 retry window is non-zero"),
            worker_shutdown_grace: Duration::from_secs(2),
            focus_policy: FocusPolicy::StrictAvailableEvidence,
            newline_policy: NewlinePolicy::NormalizeToLineBreak,
            tab_policy: TabPolicy::Allow,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ConfigError, ConfigField, P1Config};
    use crate::{DispatchBatchLimit, SemanticElementLimit};

    #[test]
    fn default_configuration_is_valid() {
        assert!(P1Config::default().validate().is_ok());
    }

    #[test]
    fn batch_cannot_exceed_total_payload() {
        let config = P1Config {
            total_payload_limit: SemanticElementLimit::new(4).expect("test value"),
            dispatch_batch_limit: DispatchBatchLimit::new(5).expect("test value"),
            ..P1Config::default()
        };

        assert_eq!(config.validate(), Err(ConfigError::BatchExceedsPayload));
    }

    #[test]
    fn modifier_poll_must_be_bounded() {
        let config = P1Config {
            modifier_poll_interval: Duration::ZERO,
            ..P1Config::default()
        };

        assert_eq!(
            config.validate(),
            Err(ConfigError::ZeroDuration(ConfigField::ModifierPollInterval))
        );
    }
}
