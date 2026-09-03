//! Native macOS adapters for ClipType.

#![cfg_attr(not(target_os = "macos"), forbid(unsafe_code))]

#[cfg(target_os = "macos")]
mod native;

#[cfg(target_os = "macos")]
pub use native::{
    MacAccessibility, MacClipboard, MacHotkeyController, MacKeyboard, MacMenuEvent, MacModifiers,
    MacPaste, MacStartup, MacStartupStatus, MacStatusItem, MacTarget, initialize_application,
};
