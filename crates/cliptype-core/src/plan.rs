//! Immutable P1 keyboard-plan construction.

use std::{fmt, slice::Chunks};

use crate::{
    CapabilityState, ConfigError, NormalizationError, NormalizedText, P1Config, SensitiveText,
    normalize_text,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityRequirement {
    UnicodeText,
    LineBreak,
    Tab,
    ModifierObservation,
}

/// Native-neutral capability snapshot consumed by pure planning policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanCapabilities {
    pub unicode_text: CapabilityState,
    pub line_break: CapabilityState,
    pub tab: CapabilityState,
    pub modifier_observation: CapabilityState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    InvalidConfiguration(ConfigError),
    Normalization(NormalizationError),
    CapabilityUnavailable(CapabilityRequirement),
    CapabilityDegraded(CapabilityRequirement),
}

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(error) => write!(formatter, "invalid P1 configuration: {error}"),
            Self::Normalization(error) => write!(formatter, "text normalization failed: {error}"),
            Self::CapabilityUnavailable(requirement) => {
                write!(formatter, "required capability unavailable: {requirement:?}")
            }
            Self::CapabilityDegraded(requirement) => {
                write!(formatter, "degraded capability rejected: {requirement:?}")
            }
        }
    }
}

impl std::error::Error for PlanError {}

/// Immutable normalized text and safety configuration for one P1 session.
pub struct KeyboardPlan {
    text: NormalizedText,
    config: P1Config,
    capabilities: PlanCapabilities,
}

impl KeyboardPlan {
    pub fn text(&self) -> &NormalizedText {
        &self.text
    }

    pub const fn config(&self) -> P1Config {
        self.config
    }

    pub const fn capabilities(&self) -> PlanCapabilities {
        self.capabilities
    }

    /// Returns bounded non-empty slices. P1-08 wraps each slice in `TextBatch`
    /// immediately before calling the platform port.
    pub fn batch_slices(&self) -> Chunks<'_, crate::TextAtom> {
        self.text
            .atoms()
            .chunks(self.config.dispatch_batch_limit.get())
    }
}

impl fmt::Debug for KeyboardPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KeyboardPlan")
            .field("text", &self.text)
            .field("config", &self.config)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

pub fn build_keyboard_plan(
    text: SensitiveText,
    config: P1Config,
    capabilities: PlanCapabilities,
) -> Result<KeyboardPlan, PlanError> {
    let config = config.validate().map_err(PlanError::InvalidConfiguration)?;
    let normalized = normalize_text(text, config).map_err(PlanError::Normalization)?;

    if normalized.contains_scalar() {
        require_available(capabilities.unicode_text, CapabilityRequirement::UnicodeText)?;
    }
    if normalized.contains_line_break() {
        require_available(capabilities.line_break, CapabilityRequirement::LineBreak)?;
    }
    if normalized.contains_tab() {
        require_available(capabilities.tab, CapabilityRequirement::Tab)?;
    }
    require_available(
        capabilities.modifier_observation,
        CapabilityRequirement::ModifierObservation,
    )?;

    Ok(KeyboardPlan {
        text: normalized,
        config,
        capabilities,
    })
}

fn require_available(
    state: CapabilityState,
    requirement: CapabilityRequirement,
) -> Result<(), PlanError> {
    match state {
        CapabilityState::Available => Ok(()),
        CapabilityState::Degraded => Err(PlanError::CapabilityDegraded(requirement)),
        CapabilityState::Unavailable => Err(PlanError::CapabilityUnavailable(requirement)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityRequirement, PlanCapabilities, PlanError, build_keyboard_plan,
    };
    use crate::{CapabilityState, P1Config, SensitiveText};

    fn available() -> PlanCapabilities {
        PlanCapabilities {
            unicode_text: CapabilityState::Available,
            line_break: CapabilityState::Available,
            tab: CapabilityState::Available,
            modifier_observation: CapabilityState::Available,
        }
    }

    #[test]
    fn creates_immutable_bounded_plan() {
        let plan = build_keyboard_plan(
            SensitiveText::new("abcdefghij".to_owned()),
            P1Config::default(),
            available(),
        )
        .expect("capabilities support fixture");
        let lengths: Vec<_> = plan.batch_slices().map(<[crate::TextAtom]>::len).collect();

        assert_eq!(lengths, vec![8, 2]);
        assert_eq!(plan.text().len(), 10);
    }

    #[test]
    fn rejects_unavailable_and_degraded_capabilities() {
        let unavailable = PlanCapabilities {
            unicode_text: CapabilityState::Unavailable,
            ..available()
        };
        assert_eq!(
            build_keyboard_plan(
                SensitiveText::new("a".to_owned()),
                P1Config::default(),
                unavailable,
            )
            .expect_err("Unicode capability is required"),
            PlanError::CapabilityUnavailable(CapabilityRequirement::UnicodeText)
        );

        let degraded = PlanCapabilities {
            modifier_observation: CapabilityState::Degraded,
            ..available()
        };
        assert_eq!(
            build_keyboard_plan(
                SensitiveText::new("a".to_owned()),
                P1Config::default(),
                degraded,
            )
            .expect_err("strict P1 policy rejects degradation"),
            PlanError::CapabilityDegraded(CapabilityRequirement::ModifierObservation)
        );
    }

    #[test]
    fn plan_debug_is_content_free() {
        let marker = "PLAN_PRIVATE_SENTINEL";
        let plan = build_keyboard_plan(
            SensitiveText::new(marker.to_owned()),
            P1Config::default(),
            available(),
        )
        .expect("fixture is supported");

        assert!(!format!("{plan:?}").contains(marker));
    }
}
