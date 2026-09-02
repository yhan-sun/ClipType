//! Content-blind current-clipboard paste dispatch contract.

use cliptype_core::{CapabilityState, RetryDisposition};

use crate::{ClipboardRevision, DispatchResult, NativeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasteCapabilities {
    pub paste_chord: CapabilityState,
    pub clipboard_revision_guard: CapabilityState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteError {
    Unsupported,
    ClipboardChanged,
    InvalidRequest,
    Native(NativeError),
}

/// Emits one bounded paste command for the already-current clipboard.
///
/// Implementations must not overwrite the clipboard. The expected revision is
/// checked immediately before dispatch when the capability is available.
pub trait PastePort: Send + Sync {
    fn capabilities(&self) -> PasteCapabilities;

    fn dispatch_paste(
        &self,
        expected_revision: ClipboardRevision,
    ) -> Result<DispatchResult, PasteError>;
}

impl PasteError {
    pub const fn retry_disposition(self) -> RetryDisposition {
        let _ = self;
        RetryDisposition::Never
    }
}

#[cfg(test)]
mod tests {
    use cliptype_core::RetryDisposition;

    use super::PasteError;

    #[test]
    fn paste_failures_are_never_retried_as_idempotent() {
        assert_eq!(
            PasteError::ClipboardChanged.retry_disposition(),
            RetryDisposition::Never
        );
        assert_eq!(
            PasteError::InvalidRequest.retry_disposition(),
            RetryDisposition::Never
        );
    }
}
