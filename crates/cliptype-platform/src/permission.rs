//! Native-neutral permission state used by platform onboarding UI.

use crate::NativeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityPermissionState {
    NotRequired,
    NotRequested,
    NotGranted,
    Granted,
    Revoked,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionActionResult {
    PromptRequested,
    SettingsOpened,
    AlreadyGranted,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionErrorKind {
    NativeFailure,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionError {
    pub kind: PermissionErrorKind,
    pub native: Option<NativeError>,
}

impl PermissionError {
    pub const fn new(kind: PermissionErrorKind, native: Option<NativeError>) -> Self {
        Self { kind, native }
    }
}

/// Platform permission surface. The UI may ask for current state and may invoke
/// explicit system-approved onboarding actions; it never bypasses consent.
pub trait AccessibilityPermissionPort: Send + Sync {
    fn state(&self) -> AccessibilityPermissionState;

    fn request(&self) -> Result<PermissionActionResult, PermissionError>;

    fn open_system_settings(&self) -> Result<PermissionActionResult, PermissionError>;
}
