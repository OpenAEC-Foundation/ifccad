use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> f64 {
        self.x
    }

    pub fn y(self) -> f64 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds2d {
    pub(crate) min: Point2,
    pub(crate) max: Point2,
}

impl Bounds2d {
    pub fn min(self) -> Point2 {
        self.min
    }

    pub fn max(self) -> Point2 {
        self.max
    }
}

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
pub struct AppearanceId(u32);

impl AppearanceId {
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for AppearanceId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntityId(NonZeroU64);

impl EntityId {
    pub(crate) fn new(value: u64) -> Option<Self> {
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

/// Length unit declared by a validated IFCDR resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IfcdrLengthUnit {
    Unitless,
    Millimetre,
    Centimetre,
    Metre,
    Kilometre,
    Inch,
    Foot,
}
