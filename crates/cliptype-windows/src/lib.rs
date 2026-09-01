//! Windows platform adapter boundary for ClipType.
//!
//! Win32 calls and unsafe code are localized in this package and exposed through
//! the native-neutral contracts defined by `cliptype-platform`.

#[cfg(windows)]
mod clipboard;
#[cfg(windows)]
#[allow(clippy::unusual_byte_groupings)]
mod command;
#[cfg(windows)]
mod keyboard;
#[cfg(windows)]
mod target;

#[cfg(windows)]
pub use clipboard::WindowsClipboard;
#[cfg(windows)]
pub use command::{
    CANCEL_HOTKEY, TRIGGER_HOTKEY, WindowsCommandSignal, WindowsCommandSource,
};
#[cfg(windows)]
pub use keyboard::WindowsKeyboard;
#[cfg(windows)]
pub use target::WindowsTarget;
