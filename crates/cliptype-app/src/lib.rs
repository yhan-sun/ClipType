//! Native-neutral application orchestration for ClipType.
//!
//! The coordinator owns one live bounded session, cancellation, port ordering,
//! retry budgets, and content-free status. Platform message loops and native API
//! calls remain outside this package.

#![forbid(unsafe_code)]

mod cancellation;
mod coordinator;
mod settings;

pub use cancellation::CancellationFlag;
pub use coordinator::{
    CancelResult, Coordinator, SessionCompletion, ShutdownResult, StatusSnapshot, TriggerResult,
    WaitResult,
};
pub use settings::{
    SettingsError, SettingsLoad, SettingsSource, SettingsStore, parse_settings, serialize_settings,
};
