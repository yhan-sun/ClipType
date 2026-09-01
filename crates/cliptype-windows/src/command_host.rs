//! Host-facing wrappers around the thread-affine command-source contract.

use cliptype_platform::{CommandEvent, CommandEventSource, CommandSourceError};

use crate::command::WindowsCommandSource;

impl WindowsCommandSource {
    /// Registers the P1 development trigger and cancellation hotkeys.
    pub fn register_commands(&mut self) -> Result<(), CommandSourceError> {
        <Self as CommandEventSource>::register(self)
    }

    /// Blocks on the owning Windows message queue until a typed command arrives.
    pub fn wait_for_command(&mut self) -> Result<CommandEvent, CommandSourceError> {
        <Self as CommandEventSource>::next_event(self)
    }

    /// Removes both P1 development hotkeys on the owning thread.
    pub fn unregister_commands(&mut self) -> Result<(), CommandSourceError> {
        <Self as CommandEventSource>::unregister(self)
    }
}
