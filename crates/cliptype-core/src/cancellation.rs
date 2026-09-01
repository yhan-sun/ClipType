//! Cancellation observation contract.

/// A native-neutral, non-blocking cancellation observation seam.
///
/// Implementations used by the live coordinator must make this call cheap and
/// safe from the injection worker. The trait deliberately does not choose an
/// async runtime or platform event primitive.
pub trait CancellationProbe: Send + Sync {
    /// Returns `true` once cancellation has been requested.
    fn is_cancelled(&self) -> bool;
}
