//! Native-neutral global shortcut probing and transactional replacement.

use cliptype_core::{HotkeyApplyResult, HotkeyAvailability, HotkeyPair};

use crate::NativeError;

/// Content-free failure category for the platform shortcut control plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyControlErrorKind {
    EventLoopStopped,
    Timeout,
    NativeFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyControlError {
    pub kind: HotkeyControlErrorKind,
    pub native: Option<NativeError>,
}

impl HotkeyControlError {
    pub const fn new(kind: HotkeyControlErrorKind, native: Option<NativeError>) -> Self {
        Self { kind, native }
    }
}

/// Thread-safe handle to the native global-shortcut owner.
///
/// Implementations may forward work to an OS-owned message loop. A successful
/// probe proves only that the OS accepted a temporary global registration; it
/// cannot prove that an application-local shortcut or low-level hook will not
/// also react.
pub trait HotkeyControlPort: Send + Sync {
    fn current_pair(&self) -> HotkeyPair;

    fn probe_pair(&self, candidate: HotkeyPair) -> Result<HotkeyAvailability, HotkeyControlError>;

    fn replace_pair(&self, candidate: HotkeyPair) -> Result<HotkeyApplyResult, HotkeyControlError>;
}
