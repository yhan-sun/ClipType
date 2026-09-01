//! Thread-affine trigger, cancellation, and shutdown event source contract.

use crate::NativeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandEvent {
    Trigger,
    Cancel,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSourceErrorKind {
    RegistrationConflict,
    InvalidBinding,
    NotRegistered,
    EventLoopStopped,
    NativeFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSourceError {
    pub kind: CommandSourceErrorKind,
    pub native: Option<NativeError>,
}

impl CommandSourceError {
    pub const fn new(kind: CommandSourceErrorKind, native: Option<NativeError>) -> Self {
        Self { kind, native }
    }
}

/// A command event source owned and pumped by one platform-required thread.
///
/// This trait intentionally has no `Send` or `Sync` supertrait. Registration,
/// event retrieval, and teardown must occur on the owning thread. Long-running
/// clipboard or injection work belongs on the application worker, never inside
/// the native message callback.
pub trait CommandEventSource {
    fn register(&mut self) -> Result<(), CommandSourceError>;

    fn next_event(&mut self) -> Result<CommandEvent, CommandSourceError>;

    fn unregister(&mut self) -> Result<(), CommandSourceError>;
}
