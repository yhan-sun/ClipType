//! Sensitive clipboard text wrapper.

use std::fmt;

/// Clipboard text whose ordinary diagnostic representation is redacted.
///
/// The wrapper intentionally does not implement `Clone`, `Display`, or any
/// serialization trait. Callers must explicitly opt into accessing plaintext.
/// This minimizes accidental copies and logging, but does not claim guaranteed
/// memory erasure from the process, operating system, or target application.
pub struct SensitiveText(String);

impl SensitiveText {
    /// Wraps owned clipboard text.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Exposes plaintext to the narrow code path that must process or inject it.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the owned plaintext.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns the UTF-8 storage size without exposing content.
    pub fn len_bytes(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the wrapped text is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for SensitiveText {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for SensitiveText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SensitiveText")
            .field("utf8_bytes", &self.0.len())
            .field("content", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::SensitiveText;

    #[test]
    fn debug_is_content_free() {
        let marker = "CLIPTYPE_PRIVATE_SENTINEL_7d4a";
        let text = SensitiveText::new(marker.to_owned());
        let debug = format!("{text:?}");

        assert!(!debug.contains(marker));
        assert!(debug.contains("[REDACTED]"));
        assert_eq!(text.expose(), marker);
    }
}
