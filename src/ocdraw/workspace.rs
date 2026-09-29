//! Drawing-local saved coordinate systems.

use super::{CoordinateFrame3, Point3, Vector3};
use serde_json::{json, Value};

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

    pub(crate) fn to_json(&self, id: u32) -> Value {
        let origin = self.frame.origin();
        let x = self.frame.x_axis();
        let y = self.frame.y_axis();
        json!({
            "ucsId": id,
            "name": self.name,
            "frame": {
                "origin": {"x": origin.x(), "y": origin.y(), "z": origin.z()},
                "X": {"x": x.x(), "y": x.y(), "z": x.z()},
                "Y": {"x": y.x(), "y": y.y(), "z": y.z()}
            },
            "elevation": self.elevation
        })
    }
}

pub(crate) fn frame_from_json(value: &Value) -> Option<CoordinateFrame3> {
    let point = &value["origin"];
    let x = &value["X"];
    let y = &value["Y"];
    CoordinateFrame3::try_new(
        Point3::new(
            point["x"].as_f64()?,
            point["y"].as_f64()?,
            point["z"].as_f64()?,
        ),
        Vector3::new(x["x"].as_f64()?, x["y"].as_f64()?, x["z"].as_f64()?),
        Vector3::new(y["x"].as_f64()?, y["y"].as_f64()?, y["z"].as_f64()?),
    )
    .ok()
}
