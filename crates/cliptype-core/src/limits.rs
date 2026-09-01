//! Strongly typed native and semantic work bounds.

use std::{fmt, num::NonZeroUsize};

/// Identifies the unit and purpose of a rejected bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundKind {
    NativeClipboardBytes,
    SemanticElements,
    DispatchElements,
    RetryAttempts,
    NativeEvents,
}

/// A content-free bound construction or conversion error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundError {
    Zero(BoundKind),
    ExceedsPlatformU32 { kind: BoundKind, value: usize },
}

impl fmt::Display for BoundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero(kind) => write!(formatter, "{kind:?} must be non-zero"),
            Self::ExceedsPlatformU32 { kind, value } => {
                write!(formatter, "{kind:?} value {value} exceeds u32")
            }
        }
    }
}

impl std::error::Error for BoundError {}

macro_rules! define_non_zero_bound {
    ($name:ident, $kind:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(NonZeroUsize);

        impl $name {
            pub fn new(value: usize) -> Result<Self, BoundError> {
                NonZeroUsize::new(value)
                    .map(Self)
                    .ok_or(BoundError::Zero($kind))
            }

            pub const fn get(self) -> usize {
                self.0.get()
            }
        }

        impl TryFrom<usize> for $name {
            type Error = BoundError;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

define_non_zero_bound!(NativeByteLimit, BoundKind::NativeClipboardBytes);
define_non_zero_bound!(SemanticElementLimit, BoundKind::SemanticElements);
define_non_zero_bound!(DispatchBatchLimit, BoundKind::DispatchElements);
define_non_zero_bound!(RetryAttemptLimit, BoundKind::RetryAttempts);

/// A measured number of bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteCount(usize);

impl ByteCount {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

/// A measured number of UTF-16 code units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf16UnitCount(usize);

impl Utf16UnitCount {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

/// A measured number of normalized semantic text elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticElementCount(usize);

impl SemanticElementCount {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

/// A native event count accepted by platform APIs using a `u32` count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativeEventCount(u32);

impl NativeEventCount {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<usize> for NativeEventCount {
    type Error = BoundError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value)
            .map(Self)
            .map_err(|_| BoundError::ExceedsPlatformU32 {
                kind: BoundKind::NativeEvents,
                value,
            })
    }
}

impl NativeByteLimit {
    pub const fn allows(self, count: ByteCount) -> bool {
        count.get() <= self.get()
    }
}

impl SemanticElementLimit {
    pub const fn allows(self, count: SemanticElementCount) -> bool {
        count.get() <= self.get()
    }
}

impl DispatchBatchLimit {
    pub const fn allows(self, count: SemanticElementCount) -> bool {
        count.get() <= self.get()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundError, BoundKind, ByteCount, DispatchBatchLimit, NativeByteLimit, NativeEventCount,
        SemanticElementCount,
    };

    #[test]
    fn zero_bounds_fail_closed() {
        assert_eq!(
            NativeByteLimit::new(0),
            Err(BoundError::Zero(BoundKind::NativeClipboardBytes))
        );
        assert_eq!(
            DispatchBatchLimit::new(0),
            Err(BoundError::Zero(BoundKind::DispatchElements))
        );
    }

    #[test]
    fn native_event_conversion_is_checked() {
        assert_eq!(
            NativeEventCount::try_from(3_usize).map(NativeEventCount::get),
            Ok(3)
        );

        if usize::BITS > u32::BITS {
            let too_large = (u32::MAX as usize) + 1;
            assert_eq!(
                NativeEventCount::try_from(too_large),
                Err(BoundError::ExceedsPlatformU32 {
                    kind: BoundKind::NativeEvents,
                    value: too_large,
                })
            );
        }
    }

    #[test]
    fn limits_compare_only_matching_units() {
        let byte_limit = NativeByteLimit::new(8).expect("test constant is non-zero");
        let batch_limit = DispatchBatchLimit::new(2).expect("test constant is non-zero");

        assert!(byte_limit.allows(ByteCount::new(8)));
        assert!(!byte_limit.allows(ByteCount::new(9)));
        assert!(batch_limit.allows(SemanticElementCount::new(2)));
        assert!(!batch_limit.allows(SemanticElementCount::new(3)));
    }
}
