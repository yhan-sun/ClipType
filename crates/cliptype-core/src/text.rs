//! Native-neutral semantic text contracts.

use std::fmt;

use crate::{DispatchBatchLimit, SemanticElementCount};

/// One validated semantic input element.
///
/// `Scalar` stores a Unicode scalar value, but its debug representation is
/// redacted so ordinary logs cannot reveal clipboard text.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextAtom {
    Scalar(char),
    LineBreak,
    Tab,
}

impl TextAtom {
    pub const fn scalar(value: char) -> Self {
        Self::Scalar(value)
    }

    pub const fn exposed_scalar(self) -> Option<char> {
        match self {
            Self::Scalar(value) => Some(value),
            Self::LineBreak | Self::Tab => None,
        }
    }
}

impl fmt::Debug for TextAtom {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(_) => formatter.write_str("Scalar([REDACTED])"),
            Self::LineBreak => formatter.write_str("LineBreak"),
            Self::Tab => formatter.write_str("Tab"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBatchError {
    Empty,
    ExceedsDispatchLimit {
        elements: SemanticElementCount,
        limit: DispatchBatchLimit,
    },
}

/// A non-empty borrowed semantic batch whose size is already bounded.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TextBatch<'a> {
    atoms: &'a [TextAtom],
}

impl<'a> TextBatch<'a> {
    pub fn new(atoms: &'a [TextAtom], limit: DispatchBatchLimit) -> Result<Self, TextBatchError> {
        if atoms.is_empty() {
            return Err(TextBatchError::Empty);
        }

        let elements = SemanticElementCount::new(atoms.len());
        if !limit.allows(elements) {
            return Err(TextBatchError::ExceedsDispatchLimit { elements, limit });
        }

        Ok(Self { atoms })
    }

    pub const fn atoms(self) -> &'a [TextAtom] {
        self.atoms
    }

    pub const fn len(self) -> usize {
        self.atoms.len()
    }

    pub const fn is_empty(self) -> bool {
        self.atoms.is_empty()
    }
}

impl fmt::Debug for TextBatch<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TextBatch")
            .field("elements", &self.atoms.len())
            .field("content", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{TextAtom, TextBatch, TextBatchError};
    use crate::DispatchBatchLimit;

    #[test]
    fn atom_debug_does_not_expose_scalar() {
        let marker = '密';
        let debug = format!("{:?}", TextAtom::scalar(marker));

        assert!(!debug.contains(marker));
        assert_eq!(TextAtom::scalar(marker).exposed_scalar(), Some(marker));
    }

    #[test]
    fn batch_is_non_empty_and_bounded() {
        let limit = DispatchBatchLimit::new(2).expect("test limit");
        let atoms = [TextAtom::scalar('a'), TextAtom::LineBreak];

        assert_eq!(TextBatch::new(&[], limit), Err(TextBatchError::Empty));
        assert_eq!(TextBatch::new(&atoms, limit).map(TextBatch::len), Ok(2));

        let oversized = [TextAtom::scalar('a'), TextAtom::scalar('b'), TextAtom::Tab];
        assert!(matches!(
            TextBatch::new(&oversized, limit),
            Err(TextBatchError::ExceedsDispatchLimit { .. })
        ));
    }

    #[test]
    fn batch_debug_is_content_free() {
        let marker = "PRIVATE_BATCH_MARKER";
        let atoms: Vec<_> = marker.chars().map(TextAtom::scalar).collect();
        let limit = DispatchBatchLimit::new(atoms.len()).expect("test limit");
        let batch = TextBatch::new(&atoms, limit).expect("valid batch");
        let debug = format!("{batch:?}");

        assert!(!debug.contains(marker));
        assert!(debug.contains("[REDACTED]"));
    }
}
