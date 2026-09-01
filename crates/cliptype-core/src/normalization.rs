//! Clipboard text validation and semantic normalization.

use std::fmt;

use crate::{
    P1Config, SemanticElementCount, SemanticElementLimit, SensitiveText, TabPolicy, TextAtom,
};

/// Content-free normalization failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationError {
    Empty,
    UnsupportedControl { scalar_index: usize },
    TabRejected { scalar_index: usize },
    PayloadTooLarge { limit: SemanticElementLimit },
}

impl fmt::Display for NormalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("clipboard text is empty"),
            Self::UnsupportedControl { scalar_index } => {
                write!(
                    formatter,
                    "unsupported control at scalar index {scalar_index}"
                )
            }
            Self::TabRejected { scalar_index } => {
                write!(formatter, "tab rejected at scalar index {scalar_index}")
            }
            Self::PayloadTooLarge { limit } => {
                write!(formatter, "semantic payload exceeds limit {}", limit.get())
            }
        }
    }
}

impl std::error::Error for NormalizationError {}

/// Owned normalized semantic text with a redacted diagnostic representation.
#[derive(PartialEq, Eq)]
pub struct NormalizedText {
    atoms: Vec<TextAtom>,
}

impl NormalizedText {
    pub fn atoms(&self) -> &[TextAtom] {
        &self.atoms
    }

    pub fn len(&self) -> usize {
        self.atoms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }

    pub fn element_count(&self) -> SemanticElementCount {
        SemanticElementCount::new(self.atoms.len())
    }

    pub fn contains_scalar(&self) -> bool {
        self.atoms
            .iter()
            .any(|atom| matches!(atom, TextAtom::Scalar(_)))
    }

    pub fn contains_line_break(&self) -> bool {
        self.atoms.contains(&TextAtom::LineBreak)
    }

    pub fn contains_tab(&self) -> bool {
        self.atoms.contains(&TextAtom::Tab)
    }
}

impl fmt::Debug for NormalizedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NormalizedText")
            .field("elements", &self.atoms.len())
            .field("content", &"[REDACTED]")
            .finish()
    }
}

/// Converts clipboard text to P1 semantic atoms without changing Unicode
/// normalization form or reordering combining marks.
pub fn normalize_text(
    text: SensitiveText,
    config: P1Config,
) -> Result<NormalizedText, NormalizationError> {
    if text.is_empty() {
        return Err(NormalizationError::Empty);
    }

    let limit = config.total_payload_limit;
    let mut atoms = Vec::with_capacity(text.expose().chars().count().min(limit.get()));
    let mut chars = text.expose().chars().peekable();
    let mut scalar_index = 0_usize;

    while let Some(value) = chars.next() {
        let atom = match value {
            '\r' => {
                if chars.next_if_eq(&'\n').is_some() {
                    scalar_index = scalar_index.saturating_add(1);
                }
                TextAtom::LineBreak
            }
            '\n' => TextAtom::LineBreak,
            '\t' => match config.tab_policy {
                TabPolicy::Allow => TextAtom::Tab,
                TabPolicy::Reject => {
                    return Err(NormalizationError::TabRejected { scalar_index });
                }
            },
            control if control.is_control() => {
                return Err(NormalizationError::UnsupportedControl { scalar_index });
            }
            scalar => TextAtom::scalar(scalar),
        };

        if atoms.len() >= limit.get() {
            return Err(NormalizationError::PayloadTooLarge { limit });
        }
        atoms.push(atom);
        scalar_index = scalar_index.saturating_add(1);
    }

    Ok(NormalizedText { atoms })
}

#[cfg(test)]
mod tests {
    use super::{NormalizationError, normalize_text};
    use crate::{P1Config, SemanticElementLimit, SensitiveText, TabPolicy, TextAtom};

    fn normalize(value: &str) -> Result<Vec<TextAtom>, NormalizationError> {
        normalize_text(SensitiveText::new(value.to_owned()), P1Config::default())
            .map(|text| text.atoms().to_vec())
    }

    #[test]
    fn preserves_ascii_cjk_supplementary_and_combining_order() {
        let source = "A中😀e\u{301}";
        let atoms = normalize(source).expect("fixture is supported");
        let exposed: String = atoms
            .iter()
            .filter_map(|atom| atom.exposed_scalar())
            .collect();

        assert_eq!(exposed, source);
        assert_eq!(atoms.len(), source.chars().count());
    }

    #[test]
    fn normalizes_all_line_endings_to_one_semantic_atom() {
        assert_eq!(
            normalize("a\r\nb\rc\nd"),
            Ok(vec![
                TextAtom::scalar('a'),
                TextAtom::LineBreak,
                TextAtom::scalar('b'),
                TextAtom::LineBreak,
                TextAtom::scalar('c'),
                TextAtom::LineBreak,
                TextAtom::scalar('d'),
            ])
        );
    }

    #[test]
    fn tab_policy_is_explicit() {
        assert_eq!(
            normalize("a\tb"),
            Ok(vec![
                TextAtom::scalar('a'),
                TextAtom::Tab,
                TextAtom::scalar('b')
            ])
        );

        let config = P1Config {
            tab_policy: TabPolicy::Reject,
            ..P1Config::default()
        };
        assert_eq!(
            normalize_text(SensitiveText::new("a\tb".to_owned()), config),
            Err(NormalizationError::TabRejected { scalar_index: 1 })
        );
    }

    #[test]
    fn unsupported_controls_fail_without_exposing_the_character() {
        let marker = "private\u{0000}marker";
        let error = normalize(marker).expect_err("NUL is unsupported");
        let debug = format!("{error:?}");

        assert_eq!(
            error,
            NormalizationError::UnsupportedControl { scalar_index: 7 }
        );
        assert!(!debug.contains(marker));
    }

    #[test]
    fn total_payload_limit_is_enforced_after_line_normalization() {
        let config = P1Config {
            total_payload_limit: SemanticElementLimit::new(2).expect("test limit"),
            ..P1Config::default()
        };

        assert_eq!(
            normalize_text(SensitiveText::new("a\r\nb".to_owned()), config),
            Err(NormalizationError::PayloadTooLarge {
                limit: config.total_payload_limit,
            })
        );
    }
}
