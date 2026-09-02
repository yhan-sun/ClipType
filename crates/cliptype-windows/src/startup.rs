//! Per-user start-at-login registration under the current user's Run key.

use core::ffi::c_void;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, path::Path, ptr::null_mut};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "ClipType";

const ERROR_SUCCESS: i32 = 0;
const ERROR_FILE_NOT_FOUND: i32 = 2;
const KEY_QUERY_VALUE: u32 = 0x0001;
const KEY_SET_VALUE: u32 = 0x0002;
const REG_SZ: u32 = 1;
const REG_OPTION_NON_VOLATILE: u32 = 0;
const HKEY_CURRENT_USER_VALUE: isize = -2_147_483_647;

type HKey = *mut c_void;

#[link(name = "advapi32")]
unsafe extern "system" {
    #[link_name = "RegOpenKeyExW"]
    fn reg_open_key_ex_w(
        key: HKey,
        sub_key: *const u16,
        options: u32,
        access: u32,
        result: *mut HKey,
    ) -> i32;

    #[link_name = "RegCreateKeyExW"]
    fn reg_create_key_ex_w(
        key: HKey,
        sub_key: *const u16,
        reserved: u32,
        class: *mut u16,
        options: u32,
        access: u32,
        security: *const c_void,
        result: *mut HKey,
        disposition: *mut u32,
    ) -> i32;

    #[link_name = "RegSetValueExW"]
    fn reg_set_value_ex_w(
        key: HKey,
        value_name: *const u16,
        reserved: u32,
        value_type: u32,
        data: *const u8,
        data_bytes: u32,
    ) -> i32;

    #[link_name = "RegQueryValueExW"]
    fn reg_query_value_ex_w(
        key: HKey,
        value_name: *const u16,
        reserved: *mut u32,
        value_type: *mut u32,
        data: *mut u8,
        data_bytes: *mut u32,
    ) -> i32;

    #[link_name = "RegDeleteValueW"]
    fn reg_delete_value_w(key: HKey, value_name: *const u16) -> i32;

    #[link_name = "RegCloseKey"]
    fn reg_close_key(key: HKey) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupStatus {
    Disabled,
    EnabledMatching,
    EnabledDifferent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupError {
    code: i32,
}

impl StartupError {
    pub const fn code(self) -> i32 {
        self.code
    }
}

impl std::fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Windows startup registry operation failed: {}",
            self.code
        )
    }
}

impl std::error::Error for StartupError {}

#[derive(Debug, Clone)]
pub struct WindowsStartup {
    value_name: Vec<u16>,
}

impl WindowsStartup {
    pub fn new() -> Self {
        Self::with_value_name(VALUE_NAME)
    }

    pub fn status(&self, executable: &Path) -> Result<StartupStatus, StartupError> {
        let Some(existing) = self.read_value()? else {
            return Ok(StartupStatus::Disabled);
        };
        if existing == command_for_executable(executable) {
            Ok(StartupStatus::EnabledMatching)
        } else {
            Ok(StartupStatus::EnabledDifferent)
        }
    }

    pub fn set_enabled(&self, executable: &Path, enabled: bool) -> Result<(), StartupError> {
        if enabled {
            self.write_value(&command_for_executable(executable))
        } else {
            self.delete_value()
        }
    }

    fn with_value_name(name: &str) -> Self {
        Self {
            value_name: wide(name),
        }
    }

    fn read_value(&self) -> Result<Option<Vec<u16>>, StartupError> {
        let run_key = wide(RUN_KEY);
        let mut key = null_mut();
        // SAFETY: static nul-terminated path and writable handle output. The
        // resulting handle is closed by `RegistryKey`.
        let status = unsafe {
            reg_open_key_ex_w(
                current_user(),
                run_key.as_ptr(),
                0,
                KEY_QUERY_VALUE,
                &raw mut key,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        check(status)?;
        let key = RegistryKey(key);

        let mut value_type = 0_u32;
        let mut bytes = 0_u32;
        // SAFETY: null data performs the documented size query.
        let status = unsafe {
            reg_query_value_ex_w(
                key.0,
                self.value_name.as_ptr(),
                null_mut(),
                &raw mut value_type,
                null_mut(),
                &raw mut bytes,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        check(status)?;
        if value_type != REG_SZ || bytes == 0 || !bytes.is_multiple_of(2) {
            return Ok(None);
        }

        let units = usize::try_from(bytes / 2).map_err(|_| StartupError { code: -1 })?;
        let mut buffer = vec![0_u16; units];
        // SAFETY: `buffer` is writable for the size returned by the first query.
        check(unsafe {
            reg_query_value_ex_w(
                key.0,
                self.value_name.as_ptr(),
                null_mut(),
                &raw mut value_type,
                buffer.as_mut_ptr().cast(),
                &raw mut bytes,
            )
        })?;
        if buffer.last() == Some(&0) {
            buffer.pop();
        }
        Ok(Some(buffer))
    }

    fn write_value(&self, value: &[u16]) -> Result<(), StartupError> {
        let run_key = wide(RUN_KEY);
        let mut key = null_mut();
        let mut disposition = 0_u32;
        // SAFETY: static nul-terminated path, null optional class/security, and
        // writable handle/disposition outputs.
        check(unsafe {
            reg_create_key_ex_w(
                current_user(),
                run_key.as_ptr(),
                0,
                null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                std::ptr::null(),
                &raw mut key,
                &raw mut disposition,
            )
        })?;
        let key = RegistryKey(key);
        let bytes = value
            .len()
            .checked_mul(2)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(StartupError { code: -1 })?;
        // SAFETY: `value` is nul-terminated UTF-16 and valid for `bytes`.
        check(unsafe {
            reg_set_value_ex_w(
                key.0,
                self.value_name.as_ptr(),
                0,
                REG_SZ,
                value.as_ptr().cast(),
                bytes,
            )
        })
    }

    fn delete_value(&self) -> Result<(), StartupError> {
        let run_key = wide(RUN_KEY);
        let mut key = null_mut();
        // SAFETY: static nul-terminated path and writable handle output.
        let status = unsafe {
            reg_open_key_ex_w(
                current_user(),
                run_key.as_ptr(),
                0,
                KEY_SET_VALUE,
                &raw mut key,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        check(status)?;
        let key = RegistryKey(key);
        // SAFETY: value name is nul-terminated and the key has set-value rights.
        let status = unsafe { reg_delete_value_w(key.0, self.value_name.as_ptr()) };
        if status == ERROR_FILE_NOT_FOUND {
            Ok(())
        } else {
            check(status)
        }
    }
}

impl Default for WindowsStartup {
    fn default() -> Self {
        Self::new()
    }
}

struct RegistryKey(HKey);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: this guard owns a handle returned by open/create.
            let _ = unsafe { reg_close_key(self.0) };
        }
    }
}

fn command_for_executable(executable: &Path) -> Vec<u16> {
    let mut command = Vec::new();
    command.push('"' as u16);
    command.extend(executable.as_os_str().encode_wide());
    command.push('"' as u16);
    command.extend(OsStr::new(" --background").encode_wide());
    command.push(0);
    command
}

const fn current_user() -> HKey {
    HKEY_CURRENT_USER_VALUE as usize as HKey
}

fn check(status: i32) -> Result<(), StartupError> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(StartupError { code: status })
    }
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{StartupStatus, WindowsStartup, command_for_executable};

    #[test]
    fn executable_command_is_quoted_and_has_background_flag() {
        let command = command_for_executable(Path::new(r"C:\Program Files\ClipType\cliptype.exe"));
        let text = String::from_utf16(&command[..command.len() - 1]).expect("valid command");
        assert_eq!(
            text,
            r#""C:\Program Files\ClipType\cliptype.exe" --background"#
        );
    }

    #[test]
    fn registry_round_trip_uses_an_isolated_value_and_cleans_up() {
        let name = format!("ClipTypeTest-{}", std::process::id());
        let startup = WindowsStartup::with_value_name(&name);
        let executable = Path::new(r"C:\ClipType-Test\cliptype.exe");

        startup
            .set_enabled(executable, false)
            .expect("clean previous test value");
        assert_eq!(
            startup.status(executable).expect("disabled status"),
            StartupStatus::Disabled
        );
        startup
            .set_enabled(executable, true)
            .expect("enable isolated startup value");
        assert_eq!(
            startup.status(executable).expect("matching status"),
            StartupStatus::EnabledMatching
        );
        startup
            .set_enabled(executable, false)
            .expect("cleanup isolated startup value");
    }
}
