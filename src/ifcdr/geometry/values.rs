/// A position in three-dimensional coordinates. Consumers validate finiteness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Point3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub const fn x(self) -> f64 {
        self.x
    }
    pub const fn y(self) -> f64 {
        self.y
    }
    pub const fn z(self) -> f64 {
        self.z
    }
    pub(crate) fn components(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

/// A direction, distinct from a position; this type alone does not imply unit length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Vector3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub const fn x(self) -> f64 {
        self.x
    }
    pub const fn y(self) -> f64 {
        self.y
    }
    pub const fn z(self) -> f64 {
        self.z
    }
    pub(crate) fn components(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

/// Finite axis-aligned bounds in one owning coordinate scope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds3d {
    pub(crate) min: Point3,
    pub(crate) max: Point3,
}

impl Bounds3d {
    pub const fn min(self) -> Point3 {
        self.min
    }
    pub const fn max(self) -> Point3 {
        self.max
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateAxis {
    X,
    Y,
    Z,
}

impl CoordinateAxis {
    pub(crate) const ALL: [Self; 3] = [Self::X, Self::Y, Self::Z];
}
