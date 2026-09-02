//! Strict versioned settings parsing and recoverable file persistence.

use std::{
    collections::HashSet,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use cliptype_core::{
    AutoClipboardThreshold, HotkeyPreset, InjectionMode, ProductSettings, SettingsValidationError,
    SpeedPreset,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSource {
    Primary,
    Backup,
    Defaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsLoad {
    pub settings: ProductSettings,
    pub source: SettingsSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsError {
    Io(io::ErrorKind),
    Syntax { line: usize },
    DuplicateKey { line: usize },
    UnknownKey { line: usize },
    MissingKey(&'static str),
    InvalidValue { line: usize },
    Validation(SettingsValidationError),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(kind) => write!(formatter, "settings I/O failure: {kind:?}"),
            Self::Syntax { line } => write!(formatter, "settings syntax error at line {line}"),
            Self::DuplicateKey { line } => {
                write!(formatter, "duplicate settings key at line {line}")
            }
            Self::UnknownKey { line } => write!(formatter, "unknown settings key at line {line}"),
            Self::MissingKey(key) => write!(formatter, "missing settings key {key}"),
            Self::InvalidValue { line } => {
                write!(formatter, "invalid settings value at line {line}")
            }
            Self::Validation(error) => write!(formatter, "settings validation failed: {error}"),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<io::Error> for SettingsError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}

/// Persistent settings file plus adjacent recovery files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn backup_path(&self) -> PathBuf {
        adjacent_path(&self.path, "bak")
    }

    pub fn temporary_path(&self) -> PathBuf {
        adjacent_path(&self.path, "tmp")
    }

    pub fn load(&self) -> Result<SettingsLoad, SettingsError> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => match parse_settings(&contents) {
                Ok(settings) => Ok(SettingsLoad {
                    settings,
                    source: SettingsSource::Primary,
                }),
                Err(primary_error) => match fs::read_to_string(self.backup_path()) {
                    Ok(backup) => parse_settings(&backup)
                        .map(|settings| SettingsLoad {
                            settings,
                            source: SettingsSource::Backup,
                        })
                        .map_err(|_| primary_error),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => Err(primary_error),
                    Err(error) => Err(error.into()),
                },
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                match fs::read_to_string(self.backup_path()) {
                    Ok(backup) => parse_settings(&backup).map(|settings| SettingsLoad {
                        settings,
                        source: SettingsSource::Backup,
                    }),
                    Err(backup_error) if backup_error.kind() == io::ErrorKind::NotFound => {
                        Ok(SettingsLoad {
                            settings: ProductSettings::default(),
                            source: SettingsSource::Defaults,
                        })
                    }
                    Err(backup_error) => Err(backup_error.into()),
                }
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, settings: ProductSettings) -> Result<(), SettingsError> {
        let settings = settings.validate().map_err(SettingsError::Validation)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temporary = self.temporary_path();
        let backup = self.backup_path();
        let serialized = serialize_settings(settings);
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(serialized.as_bytes())?;
        file.sync_all()?;
        drop(file);

        remove_if_exists(&backup)?;
        let had_primary = self.path.exists();
        if had_primary {
            fs::rename(&self.path, &backup)?;
        }

        if let Err(error) = fs::rename(&temporary, &self.path) {
            let _ = remove_if_exists(&temporary);
            if had_primary && !self.path.exists() && backup.exists() {
                let _ = fs::rename(&backup, &self.path);
            }
            return Err(error.into());
        }

        Ok(())
    }
}

pub fn serialize_settings(settings: ProductSettings) -> String {
    format!(
        concat!(
            "version = {}\n",
            "enabled = {}\n",
            "mode = \"{}\"\n",
            "auto_clipboard_threshold = {}\n",
            "speed = \"{}\"\n",
            "notifications = {}\n",
            "start_at_login = {}\n",
            "hotkey = \"{}\"\n"
        ),
        settings.version,
        settings.enabled,
        mode_name(settings.mode),
        settings.auto_clipboard_threshold.get(),
        speed_name(settings.speed),
        settings.notifications,
        settings.start_at_login,
        hotkey_name(settings.hotkey),
    )
}

pub fn parse_settings(contents: &str) -> Result<ProductSettings, SettingsError> {
    let mut seen = HashSet::new();
    let mut version = None;
    let mut enabled = None;
    let mut mode = None;
    let mut threshold = None;
    let mut speed = None;
    let mut notifications = None;
    let mut start_at_login = None;
    let mut hotkey = None;

    for (index, raw_line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            return Err(SettingsError::Syntax { line: line_number });
        };
        let key = raw_key.trim();
        let value = raw_value.trim();
        if key.is_empty() || value.is_empty() || value.contains('#') {
            return Err(SettingsError::Syntax { line: line_number });
        }
        if !seen.insert(key.to_owned()) {
            return Err(SettingsError::DuplicateKey { line: line_number });
        }

        match key {
            "version" => version = Some(parse_u32(value, line_number)?),
            "enabled" => enabled = Some(parse_bool(value, line_number)?),
            "mode" => mode = Some(parse_mode(value, line_number)?),
            "auto_clipboard_threshold" => {
                let value = parse_usize(value, line_number)?;
                threshold = Some(
                    AutoClipboardThreshold::new(value)
                        .map_err(|_| SettingsError::InvalidValue { line: line_number })?,
                );
            }
            "speed" => speed = Some(parse_speed(value, line_number)?),
            "notifications" => notifications = Some(parse_bool(value, line_number)?),
            "start_at_login" => start_at_login = Some(parse_bool(value, line_number)?),
            "hotkey" => hotkey = Some(parse_hotkey(value, line_number)?),
            _ => return Err(SettingsError::UnknownKey { line: line_number }),
        }
    }

    ProductSettings {
        version: version.ok_or(SettingsError::MissingKey("version"))?,
        enabled: enabled.ok_or(SettingsError::MissingKey("enabled"))?,
        mode: mode.ok_or(SettingsError::MissingKey("mode"))?,
        auto_clipboard_threshold: threshold
            .ok_or(SettingsError::MissingKey("auto_clipboard_threshold"))?,
        speed: speed.ok_or(SettingsError::MissingKey("speed"))?,
        notifications: notifications.ok_or(SettingsError::MissingKey("notifications"))?,
        start_at_login: start_at_login.ok_or(SettingsError::MissingKey("start_at_login"))?,
        hotkey: hotkey.ok_or(SettingsError::MissingKey("hotkey"))?,
    }
    .validate()
    .map_err(SettingsError::Validation)
}

fn parse_u32(value: &str, line: usize) -> Result<u32, SettingsError> {
    value
        .parse()
        .map_err(|_| SettingsError::InvalidValue { line })
}

fn parse_usize(value: &str, line: usize) -> Result<usize, SettingsError> {
    value
        .parse()
        .map_err(|_| SettingsError::InvalidValue { line })
}

fn parse_bool(value: &str, line: usize) -> Result<bool, SettingsError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(SettingsError::InvalidValue { line }),
    }
}

fn parse_string(value: &str, line: usize) -> Result<&str, SettingsError> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .filter(|value| !value.contains(['"', '\\', '\n', '\r']))
        .ok_or(SettingsError::InvalidValue { line })
}

fn parse_mode(value: &str, line: usize) -> Result<InjectionMode, SettingsError> {
    match parse_string(value, line)? {
        "keyboard" => Ok(InjectionMode::Keyboard),
        "clipboard" => Ok(InjectionMode::Clipboard),
        "auto" => Ok(InjectionMode::Auto),
        _ => Err(SettingsError::InvalidValue { line }),
    }
}

fn parse_speed(value: &str, line: usize) -> Result<SpeedPreset, SettingsError> {
    match parse_string(value, line)? {
        "slow" => Ok(SpeedPreset::Slow),
        "normal" => Ok(SpeedPreset::Normal),
        "fast" => Ok(SpeedPreset::Fast),
        _ => Err(SettingsError::InvalidValue { line }),
    }
}

fn parse_hotkey(value: &str, line: usize) -> Result<HotkeyPreset, SettingsError> {
    match parse_string(value, line)? {
        "ctrl-alt-shift-function" => Ok(HotkeyPreset::CtrlAltShiftFunction),
        "ctrl-alt-function" => Ok(HotkeyPreset::CtrlAltFunction),
        "ctrl-shift-function" => Ok(HotkeyPreset::CtrlShiftFunction),
        _ => Err(SettingsError::InvalidValue { line }),
    }
}

const fn mode_name(mode: InjectionMode) -> &'static str {
    match mode {
        InjectionMode::Keyboard => "keyboard",
        InjectionMode::Clipboard => "clipboard",
        InjectionMode::Auto => "auto",
    }
}

const fn speed_name(speed: SpeedPreset) -> &'static str {
    match speed {
        SpeedPreset::Slow => "slow",
        SpeedPreset::Normal => "normal",
        SpeedPreset::Fast => "fast",
    }
}

const fn hotkey_name(hotkey: HotkeyPreset) -> &'static str {
    match hotkey {
        HotkeyPreset::CtrlAltShiftFunction => "ctrl-alt-shift-function",
        HotkeyPreset::CtrlAltFunction => "ctrl-alt-function",
        HotkeyPreset::CtrlShiftFunction => "ctrl-shift-function",
    }
}

fn adjacent_path(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(format!(".{suffix}"));
    PathBuf::from(value)
}

fn remove_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    use cliptype_core::{
        AutoClipboardThreshold, HotkeyPreset, InjectionMode, ProductSettings, SpeedPreset,
    };

    use super::{SettingsError, SettingsSource, SettingsStore, parse_settings, serialize_settings};

    #[test]
    fn round_trip_is_deterministic_and_content_free() {
        let settings = ProductSettings {
            enabled: false,
            mode: InjectionMode::Clipboard,
            auto_clipboard_threshold: AutoClipboardThreshold::new(513).expect("threshold"),
            speed: SpeedPreset::Fast,
            notifications: false,
            start_at_login: true,
            hotkey: HotkeyPreset::CtrlAltFunction,
            ..ProductSettings::default()
        };
        let serialized = serialize_settings(settings);
        assert_eq!(parse_settings(&serialized), Ok(settings));
        assert_eq!(serialize_settings(settings), serialized);
        assert!(!serialized.contains("clipboard_text"));
        assert!(!serialized.contains("history"));
    }

    #[test]
    fn unknown_duplicate_missing_and_invalid_values_fail_closed() {
        let valid = serialize_settings(ProductSettings::default());
        assert!(matches!(
            parse_settings(&(valid.clone() + "unknown = true\n")),
            Err(SettingsError::UnknownKey { .. })
        ));
        assert!(matches!(
            parse_settings(&(valid.clone() + "enabled = false\n")),
            Err(SettingsError::DuplicateKey { .. })
        ));
        assert!(matches!(
            parse_settings(&valid.replace("mode = \"auto\"\n", "")),
            Err(SettingsError::MissingKey("mode"))
        ));
        assert!(matches!(
            parse_settings(&valid.replace("speed = \"normal\"", "speed = \"warp\"")),
            Err(SettingsError::InvalidValue { .. })
        ));
    }

    #[test]
    fn parser_errors_do_not_echo_untrusted_values() {
        let marker = "SETTINGS_PRIVATE_SENTINEL";
        let error = parse_settings(&format!("mode = \"{marker}\"\n"))
            .expect_err("invalid settings should fail");
        let diagnostic = format!("{error:?} {error}");
        assert!(!diagnostic.contains(marker));
    }

    #[test]
    fn store_defaults_round_trips_and_recovers_from_backup() {
        let directory = unique_test_directory();
        let path = directory.join("config.toml");
        let store = SettingsStore::new(&path);

        let initial = store.load().expect("missing settings use defaults");
        assert_eq!(initial.source, SettingsSource::Defaults);
        store.save(initial.settings).expect("save defaults");
        assert_eq!(
            store.load().expect("load primary").source,
            SettingsSource::Primary
        );

        let changed = ProductSettings {
            mode: InjectionMode::Keyboard,
            ..ProductSettings::default()
        };
        store.save(changed).expect("save changed settings");
        fs::write(&path, "corrupt").expect("corrupt primary fixture");
        let recovered = store.load().expect("recover backup");
        assert_eq!(recovered.source, SettingsSource::Backup);
        assert_eq!(recovered.settings, ProductSettings::default());

        let _ = fs::remove_dir_all(directory);
    }

    fn unique_test_directory() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("cliptype-settings-{}-{nonce}", process::id()))
    }
}
