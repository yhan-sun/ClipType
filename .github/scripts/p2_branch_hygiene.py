from pathlib import Path


def replace_once_or_already(text: str, old: str, new: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    return text


settings_path = Path("crates/cliptype-app/src/settings.rs")
settings = settings_path.read_text(encoding="utf-8")
settings = settings.replace("fs::{self, File, OpenOptions}", "fs::{self, OpenOptions}")
settings = settings.replace(
    "ProductSettings, SETTINGS_SCHEMA_VERSION,\n    SettingsValidationError",
    "ProductSettings, SettingsValidationError",
)
settings = replace_once_or_already(
    settings,
    """        remove_if_exists(&backup)?;
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
""",
    """        let primary_is_valid = match fs::read_to_string(&self.path) {
            Ok(contents) => parse_settings(&contents).is_ok(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };
        if primary_is_valid {
            remove_if_exists(&backup)?;
            fs::rename(&self.path, &backup)?;
        } else {
            remove_if_exists(&self.path)?;
        }

        if let Err(error) = fs::rename(&temporary, &self.path) {
            let _ = remove_if_exists(&temporary);
            if primary_is_valid && !self.path.exists() && backup.exists() {
                let _ = fs::rename(&backup, &self.path);
            }
            return Err(error.into());
        }
""",
)
settings = replace_once_or_already(
    settings,
    """        let recovered = store.load().expect("recover backup");
        assert_eq!(recovered.source, SettingsSource::Backup);
        assert_eq!(recovered.settings, ProductSettings::default());

        let _ = fs::remove_dir_all(directory);
""",
    """        let recovered = store.load().expect("recover backup");
        assert_eq!(recovered.source, SettingsSource::Backup);
        assert_eq!(recovered.settings, ProductSettings::default());
        store
            .save(recovered.settings)
            .expect("repair primary without replacing the valid backup");
        let backup = fs::read_to_string(store.backup_path()).expect("valid backup remains");
        assert_eq!(parse_settings(&backup), Ok(ProductSettings::default()));

        let _ = fs::remove_dir_all(directory);
""",
)
settings_path.write_text(settings, encoding="utf-8")

tray_path = Path("crates/cliptype-windows/src/tray.rs")
tray = tray_path.read_text(encoding="utf-8")
if "const CMD_NOTIFICATIONS" not in tray:
    tray = tray.replace(
        "const CMD_ENABLED: usize = 1100;\n",
        "const CMD_ENABLED: usize = 1100;\nconst CMD_NOTIFICATIONS: usize = 1101;\n",
        1,
    )
tray = replace_once_or_already(
    tray,
    """    pub fn notify(&self, notice: TrayNotice) -> Result<(), TrayError> {
        lock_unpoisoned(&self.notices).push_back(notice);
        post_thread_message(self.thread_id, WM_TRAY_NOTICE)
    }
""",
    """    pub fn notify(&self, notice: TrayNotice) -> Result<(), TrayError> {
        if !lock_unpoisoned(&self.settings).notifications {
            return Ok(());
        }
        lock_unpoisoned(&self.notices).push_back(notice);
        post_thread_message(self.thread_id, WM_TRAY_NOTICE)
    }
""",
)
tray = replace_once_or_already(
    tray,
    """    let thread_id = unsafe { GetCurrentThreadId() };
    *lock_unpoisoned(context()) = Some(TrayContext { events, settings });
""",
    """    let thread_id = unsafe { GetCurrentThreadId() };
    let notifications_enabled = lock_unpoisoned(&settings).notifications;
    *lock_unpoisoned(context()) = Some(TrayContext { events, settings });
""",
)
tray = replace_once_or_already(
    tray,
    """    let _ = ready.send(Ok(thread_id));
    icon.show_notice(TrayNotice::Ready);
""",
    """    let _ = ready.send(Ok(thread_id));
    if notifications_enabled {
        icon.show_notice(TrayNotice::Ready);
    }
""",
)
if '"Notifications"' not in tray:
    tray = tray.replace(
        """    append(menu, CMD_ENABLED, "Enabled", settings.enabled, false);
    append(
        menu,
        CMD_MODE_AUTO,
""",
        """    append(menu, CMD_ENABLED, "Enabled", settings.enabled, false);
    append(
        menu,
        CMD_NOTIFICATIONS,
        "Notifications",
        settings.notifications,
        false,
    );
    append(
        menu,
        CMD_MODE_AUTO,
""",
        1,
    )
if "CMD_NOTIFICATIONS =>" not in tray:
    tray = tray.replace(
        """        CMD_ENABLED => {
            settings.enabled = !settings.enabled;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_MODE_AUTO => {
""",
        """        CMD_ENABLED => {
            settings.enabled = !settings.enabled;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_NOTIFICATIONS => {
            settings.notifications = !settings.notifications;
            Some(TrayEvent::SettingsChanged(settings))
        }
        CMD_MODE_AUTO => {
""",
        1,
    )
tray_path.write_text(tray, encoding="utf-8")
