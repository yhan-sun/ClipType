//! Versioned, content-free user settings vocabulary.

use std::{fmt, time::Duration};

use crate::{
    AutoClipboardThreshold, HotkeyPair, InjectionMode, P1Config, ProductConfig, ProductConfigError,
};

pub const SETTINGS_SCHEMA_VERSION: u32 = 2;
pub const MIN_CHARACTERS_PER_SECOND: u16 = 1;
pub const MAX_CHARACTERS_PER_SECOND: u16 = 250;
pub const MAX_JITTER_PERCENT: u8 = 95;
pub const MAX_TYPO_PROBABILITY_PERCENT: u8 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpeedPreset {
    Slow,
    #[default]
    Normal,
    Fast,
    Custom,
}

impl SpeedPreset {
    pub const fn default_characters_per_second(self) -> u16 {
        match self {
            Self::Slow => 8,
            Self::Normal => 40,
            Self::Fast => 120,
            Self::Custom => 40,
        }
    }

    pub const fn keyboard_interval(self) -> Duration {
        interval_for_characters_per_second(self.default_characters_per_second())
    }
}

const fn interval_for_characters_per_second(characters_per_second: u16) -> Duration {
    Duration::from_nanos(1_000_000_000_u64 / characters_per_second as u64)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductSettings {
    pub version: u32,
    pub enabled: bool,
    pub mode: InjectionMode,
    pub auto_clipboard_threshold: AutoClipboardThreshold,
    pub speed: SpeedPreset,
    pub characters_per_second: u16,
    pub jitter_percent: u8,
    pub typo_probability_percent: u8,
    pub notifications: bool,
    pub start_at_login: bool,
    pub hotkeys: HotkeyPair,
}

impl ProductSettings {
    pub fn validate(self) -> Result<Self, SettingsValidationError> {
        if self.version != SETTINGS_SCHEMA_VERSION {
            return Err(SettingsValidationError::UnsupportedVersion(self.version));
        }
        if !(MIN_CHARACTERS_PER_SECOND..=MAX_CHARACTERS_PER_SECOND)
            .contains(&self.characters_per_second)
        {
            return Err(SettingsValidationError::CharactersPerSecondOutOfRange);
        }
        if self.jitter_percent > MAX_JITTER_PERCENT {
            return Err(SettingsValidationError::JitterOutOfRange);
        }
        if self.typo_probability_percent > MAX_TYPO_PROBABILITY_PERCENT {
            return Err(SettingsValidationError::TypoProbabilityOutOfRange);
        }
        self.hotkeys
            .validate()
            .map_err(SettingsValidationError::InvalidHotkeys)?;
        self.runtime_config()
            .map_err(SettingsValidationError::InvalidRuntime)?;
        Ok(self)
    }

    pub fn runtime_config(self) -> Result<ProductConfig, ProductConfigError> {
        ProductConfig {
            enabled: self.enabled,
            mode: self.mode,
            auto_clipboard_threshold: self.auto_clipboard_threshold,
            jitter_percent: self.jitter_percent,
            typo_probability_percent: self.typo_probability_percent,
            safety: P1Config {
                keyboard_interval: interval_for_characters_per_second(self.characters_per_second),
                ..P1Config::default()
            },
        }
        .validate()
    }
}

impl Default for ProductSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_SCHEMA_VERSION,
            enabled: true,
            mode: InjectionMode::Auto,
            auto_clipboard_threshold: AutoClipboardThreshold::new(256)
                .expect("default auto threshold is non-zero"),
            speed: SpeedPreset::Normal,
            characters_per_second: SpeedPreset::Normal.default_characters_per_second(),
            jitter_percent: 0,
            typo_probability_percent: 0,
            notifications: true,
            start_at_login: false,
            hotkeys: HotkeyPair::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsValidationError {
    UnsupportedVersion(u32),
    CharactersPerSecondOutOfRange,
    JitterOutOfRange,
    TypoProbabilityOutOfRange,
    InvalidHotkeys(crate::HotkeyValidationError),
    InvalidRuntime(ProductConfigError),
}

impl fmt::Display for SettingsValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported settings version {version}")
            }
            Self::CharactersPerSecondOutOfRange => write!(
                formatter,
                "characters per second must be between {MIN_CHARACTERS_PER_SECOND} and {MAX_CHARACTERS_PER_SECOND}"
            ),
            Self::JitterOutOfRange => write!(
                formatter,
                "jitter percent must not exceed {MAX_JITTER_PERCENT}"
            ),
            Self::TypoProbabilityOutOfRange => write!(
                formatter,
                "typo probability percent must not exceed {MAX_TYPO_PROBABILITY_PERCENT}"
            ),
            Self::InvalidHotkeys(error) => write!(formatter, "invalid shortcut settings: {error}"),
            Self::InvalidRuntime(error) => write!(formatter, "invalid runtime settings: {error}"),
        }
    }
}

impl std::error::Error for SettingsValidationError {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        MAX_CHARACTERS_PER_SECOND, MAX_JITTER_PERCENT, MAX_TYPO_PROBABILITY_PERCENT,
        ProductSettings, SETTINGS_SCHEMA_VERSION, SettingsValidationError, SpeedPreset,
    };
    use crate::{HotkeyKey, HotkeyPlatform, InjectionMode};

    #[test]
    fn defaults_are_versioned_and_runtime_valid() {
        let settings = ProductSettings::default();
        assert_eq!(settings.version, SETTINGS_SCHEMA_VERSION);
        assert!(settings.validate().is_ok());
        let runtime = settings.runtime_config().expect("runtime config");
        assert_eq!(runtime.enabled, settings.enabled);
        assert_eq!(runtime.mode, settings.mode);
        assert_eq!(
            runtime.auto_clipboard_threshold,
            settings.auto_clipboard_threshold
        );
        assert_eq!(runtime.jitter_percent, settings.jitter_percent);
        assert_eq!(
            runtime.typo_probability_percent,
            settings.typo_probability_percent
        );
        assert_eq!(
            runtime.safety.keyboard_interval,
            Duration::from_nanos(1_000_000_000_u64 / u64::from(settings.characters_per_second))
        );
    }

    #[test]
    fn speed_preset_and_custom_rate_map_to_future_session_config() {
        let settings = ProductSettings {
            mode: InjectionMode::Keyboard,
            speed: SpeedPreset::Custom,
            characters_per_second: 37,
            jitter_percent: 21,
            typo_probability_percent: 3,
            ..ProductSettings::default()
        };
        let runtime = settings.runtime_config().expect("runtime config");
        assert_eq!(
            runtime.safety.keyboard_interval,
            Duration::from_nanos(1_000_000_000_u64 / 37)
        );
        assert_eq!(runtime.jitter_percent, 21);
        assert_eq!(runtime.typo_probability_percent, 3);
    }

    #[test]
    fn typing_bounds_fail_closed() {
        assert_eq!(
            ProductSettings {
                characters_per_second: MAX_CHARACTERS_PER_SECOND + 1,
                ..ProductSettings::default()
            }
            .validate(),
            Err(SettingsValidationError::CharactersPerSecondOutOfRange)
        );
        assert_eq!(
            ProductSettings {
                jitter_percent: MAX_JITTER_PERCENT + 1,
                ..ProductSettings::default()
            }
            .validate(),
            Err(SettingsValidationError::JitterOutOfRange)
        );
        assert_eq!(
            ProductSettings {
                typo_probability_percent: MAX_TYPO_PROBABILITY_PERCENT + 1,
                ..ProductSettings::default()
            }
            .validate(),
            Err(SettingsValidationError::TypoProbabilityOutOfRange)
        );
    }

    #[test]
    fn unknown_versions_fail_closed() {
        let settings = ProductSettings {
            version: SETTINGS_SCHEMA_VERSION + 1,
            ..ProductSettings::default()
        };
        assert_eq!(
            settings.validate(),
            Err(SettingsValidationError::UnsupportedVersion(
                SETTINGS_SCHEMA_VERSION + 1
            ))
        );
    }

    #[test]
    fn defaults_use_reviewed_non_f12_shortcuts() {
        let settings = ProductSettings::default();
        assert_eq!(settings.hotkeys.trigger.key, HotkeyKey::V);
        assert_eq!(settings.hotkeys.cancel.key, HotkeyKey::X);
        assert_eq!(
            settings.hotkeys.trigger.label(HotkeyPlatform::Windows),
            "Ctrl+Alt+Shift+V"
        );
    }
}
