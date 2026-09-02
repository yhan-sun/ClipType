//! Native-neutral ports and capability contracts for ClipType.
//!
//! Platform adapters implement these traits without leaking operating-system
//! handles, callbacks, or content-bearing diagnostics into application policy.

#![forbid(unsafe_code)]

mod clipboard;
mod command;
mod keyboard;
mod native;
mod paste;
mod target;

pub use clipboard::{ClipboardError, ClipboardPort, ClipboardRevision, ClipboardSnapshot};
pub use command::{CommandEvent, CommandEventSource, CommandSourceError, CommandSourceErrorKind};
pub use keyboard::{
    DispatchResult, KeyboardCapabilities, KeyboardError, KeyboardPort, ModifierMask,
    ModifierObservation, ModifierPort, NativeDispatchCount,
};
pub use native::{NativeError, NativeErrorKind};
pub use paste::{PasteCapabilities, PasteError, PastePort};
pub use target::{
    TargetCaptureError, TargetComparison, TargetEvidence, TargetMetadata, TargetPort,
};
