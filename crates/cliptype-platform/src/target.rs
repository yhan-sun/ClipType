//! Non-content target and focus evidence contract.

use std::{any::Any, fmt, sync::Arc};

use cliptype_core::{EvidenceStrength, IntegrityRelation};

use crate::NativeError;

/// Safe target metadata that excludes titles and focused-field content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TargetMetadata {
    pub process_id: Option<u32>,
    pub gui_thread_id: Option<u32>,
}

/// Opaque platform evidence plus content-free metadata.
///
/// The token may contain private adapter handles, but callers outside the
/// implementing adapter can neither format nor inspect it accidentally.
#[derive(Clone)]
pub struct TargetEvidence {
    token: Arc<dyn Any + Send + Sync>,
    metadata: TargetMetadata,
    strength: EvidenceStrength,
}

impl TargetEvidence {
    pub fn new<T>(token: T, metadata: TargetMetadata, strength: EvidenceStrength) -> Self
    where
        T: Any + Send + Sync,
    {
        Self {
            token: Arc::new(token),
            metadata,
            strength,
        }
    }

    pub const fn metadata(&self) -> TargetMetadata {
        self.metadata
    }

    pub const fn strength(&self) -> EvidenceStrength {
        self.strength
    }

    /// Allows only an adapter that knows the private token type to inspect it.
    pub fn token<T>(&self) -> Option<&T>
    where
        T: Any + Send + Sync,
    {
        self.token.as_ref().downcast_ref::<T>()
    }
}

impl fmt::Debug for TargetEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TargetEvidence")
            .field("token", &"[OPAQUE]")
            .field("metadata", &self.metadata)
            .field("strength", &self.strength)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCaptureError {
    Unavailable,
    Disappeared,
    Native(NativeError),
}

/// Comparison result under the strongest evidence the adapter can provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetComparison {
    Same,
    Changed,
    Disappeared,
    UnavailableOrAmbiguous,
}

/// Captures and compares target evidence without reading UI content.
pub trait TargetPort: Send + Sync {
    fn capture(&self) -> Result<TargetEvidence, TargetCaptureError>;

    fn compare(&self, expected: &TargetEvidence, observed: &TargetEvidence) -> TargetComparison;

    fn integrity_relation(&self, target: &TargetEvidence) -> IntegrityRelation;
}

#[cfg(test)]
mod tests {
    use cliptype_core::EvidenceStrength;

    use super::{TargetEvidence, TargetMetadata};

    #[test]
    fn opaque_token_is_not_exposed_by_debug() {
        let marker = "PRIVATE_TARGET_TOKEN_913";
        let evidence = TargetEvidence::new(
            marker.to_owned(),
            TargetMetadata {
                process_id: Some(42),
                gui_thread_id: Some(7),
            },
            EvidenceStrength::RenderHostLimited,
        );
        let debug = format!("{evidence:?}");

        assert!(!debug.contains(marker));
        assert!(debug.contains("[OPAQUE]"));
        assert_eq!(evidence.token::<String>().map(String::as_str), Some(marker));
    }
}
