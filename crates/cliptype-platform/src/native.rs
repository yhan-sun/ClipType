//! Content-free native error representation.

/// Broad native failure category suitable for policy and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeErrorKind {
    PermissionDenied,
    TemporarilyUnavailable,
    InvalidData,
    ResourceExhausted,
    Unsupported,
    BlockedCauseUnknown,
    Unknown,
}

/// Native error category plus an optional numeric operating-system code.
///
/// Human messages from platform APIs are deliberately not stored here because
/// they can accidentally contain target or user data on some integration paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeError {
    kind: NativeErrorKind,
    code: Option<u32>,
}

impl NativeError {
    pub const fn new(kind: NativeErrorKind, code: Option<u32>) -> Self {
        Self { kind, code }
    }

    pub const fn kind(self) -> NativeErrorKind {
        self.kind
    }

    pub const fn code(self) -> Option<u32> {
        self.code
    }
}
