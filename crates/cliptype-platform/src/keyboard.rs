//! Bounded semantic keyboard dispatch and modifier contracts.

use std::{fmt, ops::BitOr};

use cliptype_core::{CapabilityState, NativeEventCount, RetryDisposition, TextBatch};

use crate::NativeError;

/// Modifier keys relevant to safe synthetic input dispatch.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ModifierMask(u8);

impl ModifierMask {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const WINDOWS: Self = Self(1 << 3);

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for ModifierMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl fmt::Debug for ModifierMask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModifierMask")
            .field("bits", &self.bits())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierObservation {
    Clear,
    Held(ModifierMask),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardCapabilities {
    pub unicode_text: CapabilityState,
    pub line_break: CapabilityState,
    pub tab: CapabilityState,
    pub cursor_right: CapabilityState,
    pub modifier_observation: CapabilityState,
}

/// Native requested and accepted event counts, never clipboard content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeDispatchCount {
    pub requested: NativeEventCount,
    pub accepted: NativeEventCount,
}

/// Result of exactly one bounded native dispatch call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchResult {
    Complete {
        events: NativeEventCount,
    },
    NoneAccepted {
        requested: NativeEventCount,
        native: Option<NativeError>,
    },
    Partial {
        counts: NativeDispatchCount,
    },
    ProgressUnknown {
        counts: NativeDispatchCount,
    },
}

impl DispatchResult {
    /// P1 never retries a native text dispatch, including a zero result.
    pub const fn retry_disposition(self) -> RetryDisposition {
        let _ = self;
        RetryDisposition::Never
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardError {
    UnsupportedSemanticElement,
    InvalidBatch,
    ModifierStateUnavailable,
    Native(NativeError),
}

/// Bounded modifier observation used by the coordinator's settle/check loop.
pub trait ModifierPort: Send + Sync {
    fn observe_modifiers(&self) -> ModifierObservation;
}

/// Dispatches one already-validated, already-bounded semantic text batch.
///
/// Product orchestration treats each scalar, line break, Tab, simulated wrong
/// key, corrective Backspace, and corrected key as a separate paced action.
/// Cancellation, target evidence, and modifier state are therefore checked at
/// every action boundary rather than only once for a multi-character batch.
pub trait KeyboardPort: Send + Sync {
    fn capabilities(&self) -> KeyboardCapabilities;

    fn dispatch(&self, batch: TextBatch<'_>) -> Result<DispatchResult, KeyboardError>;

    /// Dispatches exactly one corrective Backspace action.
    ///
    /// The default keeps existing test/fake ports source-compatible. Product
    /// implementations that enable typo correction must override it. The
    /// coordinator applies the same delay, jitter, cancellation, focus, and
    /// modifier checks to this action as it does to ordinary text actions.
    fn dispatch_backspace(&self) -> Result<DispatchResult, KeyboardError> {
        Err(KeyboardError::UnsupportedSemanticElement)
    }

    /// Dispatches exactly one cursor-right action used to pass over an editor's
    /// already-generated closing delimiter or quote in Code mode.
    fn dispatch_cursor_right(&self) -> Result<DispatchResult, KeyboardError> {
        Err(KeyboardError::UnsupportedSemanticElement)
    }

    /// Crosses an existing editor-generated line boundary and moves to that
    /// line's end using right-arrow navigation. Code mode uses this for a
    /// matching closer that auto-indent placed on its own line.
    fn dispatch_cursor_right_to_line_end(&self) -> Result<DispatchResult, KeyboardError> {
        Err(KeyboardError::UnsupportedSemanticElement)
    }
}

#[cfg(test)]
mod tests {
    use cliptype_core::{NativeEventCount, RetryDisposition};

    use super::{DispatchResult, ModifierMask, NativeDispatchCount};

    #[test]
    fn modifier_mask_composes_without_external_bitflags() {
        let held = ModifierMask::CONTROL | ModifierMask::SHIFT;

        assert!(held.contains(ModifierMask::CONTROL));
        assert!(held.contains(ModifierMask::SHIFT));
        assert!(!held.contains(ModifierMask::ALT));
    }

    #[test]
    fn partial_and_unknown_results_are_never_retryable() {
        let counts = NativeDispatchCount {
            requested: NativeEventCount::new(4),
            accepted: NativeEventCount::new(3),
        };

        assert_eq!(
            DispatchResult::Partial { counts }.retry_disposition(),
            RetryDisposition::Never
        );
        assert_eq!(
            DispatchResult::ProgressUnknown { counts }.retry_disposition(),
            RetryDisposition::Never
        );
    }
}
