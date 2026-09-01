//! Thread-safe cancellation state for one live injection session.

use std::sync::atomic::{AtomicBool, Ordering};

use cliptype_core::CancellationProbe;

/// One-way cancellation flag shared by the command surface and worker.
#[derive(Debug, Default)]
pub struct CancellationFlag {
    requested: AtomicBool,
}

impl CancellationFlag {
    pub const fn new() -> Self {
        Self {
            requested: AtomicBool::new(false),
        }
    }

    pub fn request(&self) {
        self.requested.store(true, Ordering::Release);
    }

    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }
}

impl CancellationProbe for CancellationFlag {
    fn is_cancelled(&self) -> bool {
        self.is_requested()
    }
}
