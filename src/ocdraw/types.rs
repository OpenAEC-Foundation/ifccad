#![allow(dead_code)]

use std::num::NonZeroU64;

/// Constraint on the exact signed scale factors of every instance of a definition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockScaling {
    #[default]
    Any,
    Uniform,
}

pub use crate::geometry_kernel::{Bounds2d, Point2};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LayerId(u32);

impl LayerId {
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for LayerId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntityId(NonZeroU64);

impl EntityId {
    /// Constructs a nonzero entity identity. Resource validation also checks uniqueness.
    ///
    /// ```
    /// use ocdraw::ocdraw::EntityId;
    /// let existing_id = EntityId::new(42).unwrap();
    /// assert_eq!(existing_id.get(), 42);
    /// assert!(EntityId::new(0).is_none());
    /// ```
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScopeId(u32);

impl ScopeId {
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

pub use crate::geometry_kernel::CoordinateLengthUnit as DrawingLengthUnit;
