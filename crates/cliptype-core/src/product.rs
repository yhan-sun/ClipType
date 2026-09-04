//! P2 product configuration and pure backend-selection policy.

use std::{fmt, num::NonZeroUsize};

use crate::{
    CapabilityState, ConfigError, KeyboardPlan, P1Config, PlanCapabilities, PlanError,
    SemanticElementCount, SemanticElementLimit, SensitiveText, TabPolicy, build_keyboard_plan,
};

/// User-visible injection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InjectionMode {
    Keyboard,
    Clipboard,
    #[default]
    Auto,
    /// Paste the current clipboard as one guarded code block so editor
    /// auto-pair and auto-indent handlers do not process each character.
    Code,
}

/// Backend selected immutably for one session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionBackend {
    Keyboard,
    Clipboard,
    Code,
}

/// Non-zero semantic-element threshold at which `auto` prefers clipboard paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AutoClipboardThreshold(NonZeroUsize);

impl AutoClipboardThreshold {
    pub fn new(value: usize) -> Result<Self, ProductConfigError> {
        NonZeroUsize::new(value)
            .map(Self)
            .ok_or(ProductConfigError::ZeroAutoClipboardThreshold)
    }

    pub const fn get(self) -> usize {
        self.0.get()
    }
}

impl TryFrom<usize> for AutoClipboardThreshold {
    type Error = ProductConfigError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Validated P2 product configuration. Active sessions retain a copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductConfig {
    pub enabled: bool,
    pub mode: InjectionMode,
    pub auto_clipboard_threshold: AutoClipboardThreshold,
    pub jitter_percent: u8,
    pub typo_probability_percent: u8,
    pub safety: P1Config,
}

impl ProductConfig {
    pub fn validate(self) -> Result<Self, ProductConfigError> {
        self.safety
            .validate()
            .map_err(ProductConfigError::InvalidSafety)?;
        if self.jitter_percent > crate::MAX_JITTER_PERCENT {
            return Err(ProductConfigError::JitterOutOfRange);
        }
        if self.typo_probability_percent > crate::MAX_TYPO_PROBABILITY_PERCENT {
            return Err(ProductConfigError::TypoProbabilityOutOfRange);
        }
        Ok(self)
    }

    pub fn keyboard_only(safety: P1Config) -> Result<Self, ProductConfigError> {
        Self {
            enabled: true,
            mode: InjectionMode::Keyboard,
            auto_clipboard_threshold: AutoClipboardThreshold::new(256)?,
            jitter_percent: 0,
            typo_probability_percent: 0,
            safety,
        }
        .validate()
    }
}

impl Default for ProductConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: InjectionMode::Auto,
            auto_clipboard_threshold: AutoClipboardThreshold::new(256)
                .expect("P2 auto clipboard threshold is non-zero"),
            jitter_percent: 0,
            typo_probability_percent: 0,
            safety: P1Config::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductConfigError {
    ZeroAutoClipboardThreshold,
    JitterOutOfRange,
    TypoProbabilityOutOfRange,
    InvalidSafety(ConfigError),
}

impl fmt::Display for ProductConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroAutoClipboardThreshold => {
                formatter.write_str("auto clipboard threshold must be non-zero")
            }
            Self::JitterOutOfRange => formatter.write_str("jitter percent is out of range"),
            Self::TypoProbabilityOutOfRange => {
                formatter.write_str("typo probability percent is out of range")
            }
            Self::InvalidSafety(error) => {
                write!(formatter, "invalid safety configuration: {error}")
            }
        }
    }
}

impl std::error::Error for ProductConfigError {}

/// Native-neutral capability snapshot consumed by P2 pure planning policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductCapabilities {
    pub keyboard: PlanCapabilities,
    pub clipboard_paste: CapabilityState,
    pub clipboard_revision_guard: CapabilityState,
}

/// Content-free clipboard plan. The source remains the current OS clipboard.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ClipboardPlan {
    elements: SemanticElementCount,
}

impl ClipboardPlan {
    pub const fn element_count(self) -> SemanticElementCount {
        self.elements
    }
}

impl fmt::Debug for ClipboardPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClipboardPlan")
            .field("elements", &self.elements.get())
            .field("content", &"[REDACTED]")
            .finish()
    }
}

/// Immutable backend-specific plan for one P2 session.
pub enum InjectionPlan {
    Keyboard(KeyboardPlan),
    Clipboard(ClipboardPlan),
    Code(ClipboardPlan),
}

impl InjectionPlan {
    pub const fn backend(&self) -> InjectionBackend {
        match self {
            Self::Keyboard(_) => InjectionBackend::Keyboard,
            Self::Clipboard(_) => InjectionBackend::Clipboard,
            Self::Code(_) => InjectionBackend::Code,
        }
    }

    pub fn element_count(&self) -> SemanticElementCount {
        match self {
            Self::Keyboard(plan) => plan.text().element_count(),
            Self::Clipboard(plan) | Self::Code(plan) => plan.element_count(),
        }
    }
}

impl fmt::Debug for InjectionPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyboard(plan) => formatter.debug_tuple("Keyboard").field(plan).finish(),
            Self::Clipboard(plan) => formatter.debug_tuple("Clipboard").field(plan).finish(),
            Self::Code(plan) => formatter.debug_tuple("Code").field(plan).finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductPlanError {
    Disabled,
    InvalidConfiguration(ProductConfigError),
    Empty,
    PayloadTooLarge { limit: SemanticElementLimit },
    Keyboard(PlanError),
    ClipboardCapabilityUnavailable,
    ClipboardCapabilityDegraded,
    ClipboardRevisionUnavailable,
}

impl fmt::Display for ProductPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => formatter.write_str("ClipType is disabled"),
            Self::InvalidConfiguration(error) => {
                write!(formatter, "invalid configuration: {error}")
            }
            Self::Empty => formatter.write_str("clipboard text is empty"),
            Self::PayloadTooLarge { limit } => {
                write!(formatter, "semantic payload exceeds limit {}", limit.get())
            }
            Self::Keyboard(error) => write!(formatter, "keyboard plan unavailable: {error}"),
            Self::ClipboardCapabilityUnavailable => {
                formatter.write_str("clipboard paste capability unavailable")
            }
            Self::ClipboardCapabilityDegraded => {
                formatter.write_str("degraded clipboard paste capability rejected")
            }
            Self::ClipboardRevisionUnavailable => {
                formatter.write_str("clipboard revision evidence unavailable")
            }
        }
    }
}

impl std::error::Error for ProductPlanError {}

/// Selects exactly one backend and creates an immutable plan.
///
/// Explicit modes never fall back. `Auto` may select clipboard paste only when
/// both paste dispatch and revision guarding are fully available for the
/// current snapshot.
pub fn build_injection_plan(
    text: SensitiveText,
    revision_available: bool,
    config: ProductConfig,
    capabilities: ProductCapabilities,
) -> Result<InjectionPlan, ProductPlanError> {
    let config = config
        .validate()
        .map_err(ProductPlanError::InvalidConfiguration)?;
    if !config.enabled {
        return Err(ProductPlanError::Disabled);
    }

    let elements = inspect_elements(&text, config.safety.total_payload_limit)?;
    match config.mode {
        InjectionMode::Keyboard => build_keyboard(text, config, capabilities),
        InjectionMode::Clipboard => {
            require_clipboard(capabilities, revision_available)?;
            Ok(InjectionPlan::Clipboard(ClipboardPlan { elements }))
        }
        InjectionMode::Code => {
            require_clipboard(capabilities, revision_available)?;
            Ok(InjectionPlan::Code(ClipboardPlan { elements }))
        }
        InjectionMode::Auto => {
            let clipboard_available = clipboard_is_available(capabilities, revision_available);
            let keyboard_available =
                keyboard_is_available(&text, config.safety, capabilities.keyboard);
            // Text outside ASCII is safer through the already-current,
            // revision-guarded paste path. This covers CJK, emoji, combining
            // marks, and mixed Unicode text even when the payload is short.
            let prefers_clipboard = !text.expose().is_ascii();

            if clipboard_available
                && (prefers_clipboard
                    || elements.get() >= config.auto_clipboard_threshold.get()
                    || !keyboard_available)
            {
                Ok(InjectionPlan::Clipboard(ClipboardPlan { elements }))
            } else {
                build_keyboard(text, config, capabilities)
            }
        }
    }
}

fn build_keyboard(
    text: SensitiveText,
    config: ProductConfig,
    capabilities: ProductCapabilities,
) -> Result<InjectionPlan, ProductPlanError> {
    build_keyboard_plan(text, config.safety, capabilities.keyboard)
        .map(InjectionPlan::Keyboard)
        .map_err(ProductPlanError::Keyboard)
}

fn require_clipboard(
    capabilities: ProductCapabilities,
    revision_available: bool,
) -> Result<(), ProductPlanError> {
    match capabilities.clipboard_paste {
        CapabilityState::Unavailable => {
            return Err(ProductPlanError::ClipboardCapabilityUnavailable);
        }
        CapabilityState::Degraded => {
            return Err(ProductPlanError::ClipboardCapabilityDegraded);
        }
        CapabilityState::Available => {}
    }
    match capabilities.clipboard_revision_guard {
        CapabilityState::Unavailable => {
            return Err(ProductPlanError::ClipboardRevisionUnavailable);
        }
        CapabilityState::Degraded => {
            return Err(ProductPlanError::ClipboardCapabilityDegraded);
        }
        CapabilityState::Available => {}
    }
    if !revision_available {
        return Err(ProductPlanError::ClipboardRevisionUnavailable);
    }
    Ok(())
}

fn clipboard_is_available(capabilities: ProductCapabilities, revision_available: bool) -> bool {
    revision_available
        && capabilities.clipboard_paste == CapabilityState::Available
        && capabilities.clipboard_revision_guard == CapabilityState::Available
}

fn keyboard_is_available(
    text: &SensitiveText,
    config: P1Config,
    capabilities: PlanCapabilities,
) -> bool {
    let mut contains_scalar = false;
    let mut contains_line_break = false;
    let mut contains_tab = false;
    let mut chars = text.expose().chars().peekable();

    while let Some(value) = chars.next() {
        match value {
            '\r' => {
                let _ = chars.next_if_eq(&'\n');
                contains_line_break = true;
            }
            '\n' => contains_line_break = true,
            '\t' => {
                if config.tab_policy == TabPolicy::Reject {
                    return false;
                }
                contains_tab = true;
            }
            control if control.is_control() => return false,
            _ => contains_scalar = true,
        }
    }

    (!contains_scalar || capabilities.unicode_text == CapabilityState::Available)
        && (!contains_line_break || capabilities.line_break == CapabilityState::Available)
        && (!contains_tab || capabilities.tab == CapabilityState::Available)
        && capabilities.modifier_observation == CapabilityState::Available
}

fn inspect_elements(
    text: &SensitiveText,
    limit: SemanticElementLimit,
) -> Result<SemanticElementCount, ProductPlanError> {
    if text.is_empty() {
        return Err(ProductPlanError::Empty);
    }

    let mut count = 0_usize;
    let mut chars = text.expose().chars().peekable();
    while let Some(value) = chars.next() {
        if value == '\r' {
            let _ = chars.next_if_eq(&'\n');
        }
        if count >= limit.get() {
            return Err(ProductPlanError::PayloadTooLarge { limit });
        }
        count = count.saturating_add(1);
    }

    Ok(SemanticElementCount::new(count))
}

#[cfg(test)]
mod tests {
    use super::{
        AutoClipboardThreshold, InjectionBackend, InjectionMode, InjectionPlan,
        ProductCapabilities, ProductConfig, ProductPlanError, build_injection_plan,
    };
    use crate::{CapabilityState, P1Config, PlanCapabilities, SensitiveText};

    fn capabilities() -> ProductCapabilities {
        ProductCapabilities {
            keyboard: PlanCapabilities {
                unicode_text: CapabilityState::Available,
                line_break: CapabilityState::Available,
                tab: CapabilityState::Available,
                modifier_observation: CapabilityState::Available,
            },
            clipboard_paste: CapabilityState::Available,
            clipboard_revision_guard: CapabilityState::Available,
        }
    }

    fn config(mode: InjectionMode, threshold: usize) -> ProductConfig {
        ProductConfig {
            mode,
            auto_clipboard_threshold: AutoClipboardThreshold::new(threshold)
                .expect("test threshold"),
            ..ProductConfig::default()
        }
    }

    #[test]
    fn explicit_modes_never_fall_back() {
        let unavailable_clipboard = ProductCapabilities {
            clipboard_paste: CapabilityState::Unavailable,
            ..capabilities()
        };
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("hello".to_owned()),
                true,
                config(InjectionMode::Clipboard, 8),
                unavailable_clipboard,
            ),
            Err(ProductPlanError::ClipboardCapabilityUnavailable)
        ));

        let unavailable_keyboard = ProductCapabilities {
            keyboard: PlanCapabilities {
                unicode_text: CapabilityState::Unavailable,
                ..capabilities().keyboard
            },
            ..capabilities()
        };
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("hello".to_owned()),
                true,
                config(InjectionMode::Keyboard, 8),
                unavailable_keyboard,
            ),
            Err(ProductPlanError::Keyboard(_))
        ));
    }

    #[test]
    fn code_mode_uses_guarded_paste_for_short_ascii_code() {
        let unavailable_keyboard = ProductCapabilities {
            keyboard: PlanCapabilities {
                unicode_text: CapabilityState::Unavailable,
                line_break: CapabilityState::Unavailable,
                tab: CapabilityState::Unavailable,
                modifier_observation: CapabilityState::Unavailable,
            },
            ..capabilities()
        };
        let plan = build_injection_plan(
            SensitiveText::new("if (items[0] == '{') {\n\treturn {};\n}".to_owned()),
            true,
            config(InjectionMode::Code, 256),
            unavailable_keyboard,
        )
        .expect("Code mode only needs the guarded paste capabilities");

        assert_eq!(plan.backend(), InjectionBackend::Code);
        assert!(matches!(plan, InjectionPlan::Code(_)));
    }

    #[test]
    fn code_mode_requires_a_known_clipboard_revision() {
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("fn main() {}".to_owned()),
                false,
                config(InjectionMode::Code, 256),
                capabilities(),
            ),
            Err(ProductPlanError::ClipboardRevisionUnavailable)
        ));
    }

    #[test]
    fn auto_uses_keyboard_below_threshold_and_clipboard_at_threshold() {
        let short = build_injection_plan(
            SensitiveText::new("short".to_owned()),
            true,
            config(InjectionMode::Auto, 8),
            capabilities(),
        )
        .expect("short keyboard plan");
        assert_eq!(short.backend(), InjectionBackend::Keyboard);

        let long = build_injection_plan(
            SensitiveText::new("12345678".to_owned()),
            true,
            config(InjectionMode::Auto, 8),
            capabilities(),
        )
        .expect("threshold clipboard plan");
        assert_eq!(long.backend(), InjectionBackend::Clipboard);
    }

    #[test]
    fn auto_uses_clipboard_when_keyboard_text_is_ineligible() {
        let plan = build_injection_plan(
            SensitiveText::new("a\u{0007}b".to_owned()),
            true,
            config(InjectionMode::Auto, 100),
            capabilities(),
        )
        .expect("clipboard can deliver text without emitting the control as a key");
        assert_eq!(plan.backend(), InjectionBackend::Clipboard);
    }

    #[test]
    fn auto_prefers_revision_guarded_clipboard_for_short_unicode_text() {
        for text in [
            "你好，世界",
            "繁體中文測試",
            "こんにちは世界",
            "안녕하세요",
            "中文与 ASCII 123 混合",
            "😀👍",
            "e\u{0301}",
        ] {
            let plan = build_injection_plan(
                SensitiveText::new(text.to_owned()),
                true,
                config(InjectionMode::Auto, 256),
                capabilities(),
            )
            .expect("short Unicode text should use the guarded paste path");
            assert_eq!(plan.backend(), InjectionBackend::Clipboard, "{text:?}");
        }
    }

    #[test]
    fn auto_can_use_unicode_keyboard_only_when_clipboard_guard_is_unavailable() {
        let capabilities = ProductCapabilities {
            clipboard_paste: CapabilityState::Unavailable,
            clipboard_revision_guard: CapabilityState::Unavailable,
            ..capabilities()
        };
        let plan = build_injection_plan(
            SensitiveText::new("你好".to_owned()),
            false,
            config(InjectionMode::Auto, 256),
            capabilities,
        )
        .expect("Auto retains the Unicode keyboard fallback when paste is unavailable");
        assert_eq!(plan.backend(), InjectionBackend::Keyboard);
    }

    #[test]
    fn clipboard_requires_revision_evidence() {
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("hello".to_owned()),
                false,
                config(InjectionMode::Clipboard, 8),
                capabilities(),
            ),
            Err(ProductPlanError::ClipboardRevisionUnavailable)
        ));
    }

    #[test]
    fn plan_debug_is_content_free() {
        let marker = "PRODUCT_PLAN_PRIVATE_SENTINEL";
        let plan = build_injection_plan(
            SensitiveText::new(marker.to_owned()),
            true,
            config(InjectionMode::Clipboard, 8),
            capabilities(),
        )
        .expect("clipboard plan");
        let debug = format!("{plan:?}");

        assert!(!debug.contains(marker));
        assert!(matches!(plan, InjectionPlan::Clipboard(_)));
    }

    #[test]
    fn threshold_must_be_non_zero() {
        assert!(AutoClipboardThreshold::new(0).is_err());
        assert!(ProductConfig::keyboard_only(P1Config::default()).is_ok());
    }
}
