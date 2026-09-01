//! Windows platform adapters for ClipType.
//!
//! Native APIs and unsafe code remain private to this crate. Public adapters
//! expose the native-neutral contracts from `cliptype-platform`.

#[cfg(windows)]
mod clipboard;

#[cfg(windows)]
pub use clipboard::WindowsClipboard;
