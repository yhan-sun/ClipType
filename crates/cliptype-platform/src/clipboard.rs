//! Current clipboard text acquisition and revision contract.

use std::fmt;

use cliptype_core::{ByteCount, NativeByteLimit, SensitiveText};

use crate::NativeError;

/// Monotonic platform witness for the current clipboard contents.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClipboardRevision {
    Known(u64),
    Unavailable,
}

impl ClipboardRevision {
    pub const fn is_known(self) -> bool {
        matches!(self, Self::Known(_))
    }

    pub const fn matches(self, observed: Self) -> bool {
        match (self, observed) {
            (Self::Known(expected), Self::Known(actual)) => expected == actual,
            _ => false,
        }
    }
}

impl fmt::Debug for ClipboardRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(_) => formatter.write_str("ClipboardRevision::Known([REDACTED])"),
            Self::Unavailable => formatter.write_str("ClipboardRevision::Unavailable"),
        }
    }
}

/// Owned current text plus the content-blind revision observed around its read.
pub struct ClipboardSnapshot {
    text: SensitiveText,
    revision: ClipboardRevision,
}

impl ClipboardSnapshot {
    pub const fn new(text: SensitiveText, revision: ClipboardRevision) -> Self {
        Self { text, revision }
    }

    pub const fn revision(&self) -> ClipboardRevision {
        self.revision
    }

    pub fn text(&self) -> &SensitiveText {
        &self.text
    }

    pub fn into_parts(self) -> (SensitiveText, ClipboardRevision) {
        (self.text, self.revision)
    }
}

impl fmt::Debug for ClipboardSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClipboardSnapshot")
            .field("text", &self.text)
            .field("revision", &self.revision)
            .finish()
    }
}

/// Content-free result categories for one bounded clipboard acquisition attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    Busy,
    ChangedDuringRead,
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

    /// Returns a content-blind revision witness when the platform exposes one.
    fn current_revision(&self) -> ClipboardRevision {
        ClipboardRevision::Unavailable
    }

    /// Reads one snapshot and rejects a known revision change around the read.
    fn read_current_snapshot(
        &self,
        hard_limit: NativeByteLimit,
    ) -> Result<ClipboardSnapshot, ClipboardError> {
        let before = self.current_revision();
        let text = self.read_current_text(hard_limit)?;
        let after = self.current_revision();

        let revision = match (before, after) {
            (ClipboardRevision::Known(expected), ClipboardRevision::Known(observed))
                if expected != observed =>
            {
                return Err(ClipboardError::ChangedDuringRead);
            }
            (ClipboardRevision::Known(_), ClipboardRevision::Known(observed)) => {
                ClipboardRevision::Known(observed)
            }
            _ => ClipboardRevision::Unavailable,
        };

        Ok(ClipboardSnapshot::new(text, revision))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use cliptype_core::{NativeByteLimit, SensitiveText};

    use super::{ClipboardError, ClipboardPort, ClipboardRevision, ClipboardSnapshot};

    struct FakeClipboard {
        revisions: Mutex<Vec<ClipboardRevision>>,
        reads: AtomicUsize,
    }

    impl FakeClipboard {
        fn new(revisions: Vec<ClipboardRevision>) -> Self {
            Self {
                revisions: Mutex::new(revisions),
                reads: AtomicUsize::new(0),
            }
        }
    }

    impl ClipboardPort for FakeClipboard {
        fn read_current_text(
            &self,
            _hard_limit: NativeByteLimit,
        ) -> Result<SensitiveText, ClipboardError> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            Ok(SensitiveText::new(
                "CLIPBOARD_SNAPSHOT_PRIVATE_SENTINEL".to_owned(),
            ))
        }

        fn current_revision(&self) -> ClipboardRevision {
            self.revisions.lock().expect("test mutex").remove(0)
        }
    }

    #[test]
    fn snapshot_keeps_a_stable_revision_and_redacts_diagnostics() {
        let clipboard = FakeClipboard::new(vec![
            ClipboardRevision::Known(7),
            ClipboardRevision::Known(7),
        ]);
        let snapshot = clipboard
            .read_current_snapshot(NativeByteLimit::new(1024).expect("test limit"))
            .expect("stable snapshot");
        let debug = format!("{snapshot:?}");

        assert_eq!(snapshot.revision(), ClipboardRevision::Known(7));
        assert!(!debug.contains("CLIPBOARD_SNAPSHOT_PRIVATE_SENTINEL"));
        assert!(!debug.contains("Known(7)"));
    }

    #[test]
    fn known_change_during_read_fails_closed() {
        let clipboard = FakeClipboard::new(vec![
            ClipboardRevision::Known(7),
            ClipboardRevision::Known(8),
        ]);

        assert!(matches!(
            clipboard.read_current_snapshot(NativeByteLimit::new(1024).expect("test limit")),
            Err(ClipboardError::ChangedDuringRead)
        ));
    }

    #[test]
    fn unavailable_revision_remains_explicit() {
        let snapshot = ClipboardSnapshot::new(
            SensitiveText::new("x".to_owned()),
            ClipboardRevision::Unavailable,
        );

        assert_eq!(snapshot.revision(), ClipboardRevision::Unavailable);
    }
}
