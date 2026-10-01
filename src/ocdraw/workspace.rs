//! Drawing-local saved coordinate systems.

use super::CoordinateFrame3;

#[derive(Clone, Debug, PartialEq)]
pub struct UcsDefinition {
    pub name: String,
    pub frame: CoordinateFrame3,
    pub elevation: f64,
}

impl UcsDefinition {
    pub fn new(name: impl Into<String>, frame: CoordinateFrame3, elevation: f64) -> Self {
        Self {
            name: name.into(),
            frame,
            elevation,
        }
    }
}
