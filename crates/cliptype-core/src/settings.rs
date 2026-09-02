//! Versioned, content-free user settings vocabulary.

use std::{fmt, time::Duration};

use crate::{
    AutoClipboardThreshold, InjectionMode, P1Config, ProductConfig, ProductConfigError,
};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpeedPreset {
    Slow,
    #[default]
    Normal,
    Fast,
}

impl SpeedPreset {
    pub const fn keyboard_interval(self) -> Duration {
        match self {
            Self::Slow => Duration::from_millis(12),
            Self::Normal => Duration::from_millis(5),
            Self::Fast => Duration::from_millis(1),
        }
    }
}

/// Reviewed Windows global-hotkey pairs. Arbitrary user strings are not parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HotkeyPreset {
    #[default]
    CtrlAltShiftFunction,
    CtrlAltFunction,
    CtrlShiftFunction,
}

impl HotkeyPreset {
    pub const fn trigger_label(self) -> &'static str {
        match self {
            Self::CtrlAltShiftFunction => "Ctrl+Alt+Shift+F12",
            Self::CtrlAltFunction => "Ctrl+Alt+F12",
            Self::CtrlShiftFunction => "Ctrl+Shift+F12",
        }
    }

    pub const fn cancel_label(self) -> &'static str {
        match self {
            Self::CtrlAltShiftFunction => "Ctrl+Alt+Shift+F11",
            Self::CtrlAltFunction => "Ctrl+Alt+F11",
            Self::CtrlShiftFunction => "Ctrl+Shift+F11",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductSettings {
    pub version: u32,
    pub enabled: bool,
    pub mode: InjectionMode,
    pub auto_clipboard_threshold: AutoClipboardThreshold,
    pub speed: SpeedPreset,
    pub notifications: bool,
    pub start_at_login: bool,
    pub hotkey: HotkeyPreset,
}

impl ProductSettings {
    pub fn validate(self) -> Result<Self, SettingsValidationError> {
        if self.version != SETTINGS_SCHEMA_VERSION {
            return Err(SettingsValidationError::UnsupportedVersion(self.version));
        }
        self.runtime_config()
            .map_err(SettingsValidationError::InvalidRuntime)?;
        Ok(self)
    }

    pub fn runtime_config(self) -> Result<ProductConfig, ProductConfigError> {
        ProductConfig {
            enabled: self.enabled,
            mode: self.mode,
            auto_clipboard_threshold: self.auto_clipboard_threshold,
            safety: P1Config {
                keyboard_interval: self.speed.keyboard_interval(),
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
            notifications: true,
            start_at_login: false,
            hotkey: HotkeyPreset::CtrlAltShiftFunction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsValidationError {
    UnsupportedVersion(u32),
    InvalidRuntime(ProductConfigError),
}

impl fmt::Display for SettingsValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported settings version {version}")
            }
            Self::InvalidRuntime(error) => write!(formatter, "invalid runtime settings: {error}"),
        }
    }
}

impl std::error::Error for SettingsValidationError {}

#[cfg(test)]
mod tests {
    use super::{
        HotkeyPreset, ProductSettings, SETTINGS_SCHEMA_VERSION, SettingsValidationError,
        SpeedPreset,
    };
    use crate::{InjectionMode, ProductConfig};

    #[test]
    fn defaults_are_versioned_and_runtime_valid() {
        let settings = ProductSettings::default();
        assert_eq!(settings.version, SETTINGS_SCHEMA_VERSION);
        assert!(settings.validate().is_ok());
        let runtime = settings.runtime_config().expect("runtime config");
        assert_eq!(runtime, ProductConfig::default());
    }

    #[test]
    fn speed_and_mode_map_to_future_session_config() {
        let settings = ProductSettings {
            mode: InjectionMode::Clipboard,
            speed: SpeedPreset::Fast,
            ..ProductSettings::default()
        };
        let runtime = settings.runtime_config().expect("runtime config");
        assert_eq!(runtime.mode, InjectionMode::Clipboard);
        assert_eq!(runtime.safety.keyboard_interval, SpeedPreset::Fast.keyboard_interval());
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
    fn hotkey_presets_expose_only_reviewed_pairs() {
        assert_eq!(
            HotkeyPreset::CtrlAltFunction.trigger_label(),
            "Ctrl+Alt+F12"
        );
        assert_eq!(
            HotkeyPreset::CtrlShiftFunction.cancel_label(),
            "Ctrl+Shift+F11"
        );
    }
}
