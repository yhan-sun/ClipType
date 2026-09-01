//! Current clipboard text acquisition contract.

use cliptype_core::{ByteCount, NativeByteLimit, SensitiveText};

use crate::NativeError;

/// Content-free result categories for one bounded clipboard acquisition attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    Busy,
    Empty,
    NonText,
    Malformed,
    TooLarge {
        observed: Option<ByteCount>,
        limit: NativeByteLimit,
    },
    Native(NativeError),
}

/// Reads the current text once without history, caching, or hidden retries.
pub trait ClipboardPort: Send + Sync {
    /// Performs one bounded acquisition attempt.
    ///
    /// The adapter must copy native data into owned memory before releasing
    /// native ownership or locks. The application coordinator owns retry timing.
    fn read_current_text(
        &self,
        hard_limit: NativeByteLimit,
    ) -> Result<SensitiveText, ClipboardError>;
}
