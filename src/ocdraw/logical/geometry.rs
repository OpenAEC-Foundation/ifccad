//! Geometry values independent of the JSON stream layout.

use super::appearance::EntityAppearance;
use crate::ocdraw::{BlockTransform, CoordinateFrame3};

#[derive(Clone, Debug, PartialEq)]
pub enum EntityGeometry {
    Line {
        start: [f64; 3],
        end: [f64; 3],
    },
    Point {
        placement: CoordinateFrame3,
    },
    Circle {
        placement: CoordinateFrame3,
        radius: f64,
    },
    Arc {
        placement: CoordinateFrame3,
        radius: f64,
        start_parameter: f64,
        sweep_parameter: f64,
    },
    Ellipse {
        placement: CoordinateFrame3,
        semi_major_radius: f64,
        semi_minor_radius: f64,
        arc: Option<(f64, f64)>,
    },
    PlanarPolyline {
        placement: CoordinateFrame3,
        vertices: Vec<[f64; 3]>,
        closed: bool,
    },
    SpatialPolyline {
        vertices: Vec<[f64; 3]>,
        closed: bool,
    },
    BlockInstance {
        definition_scope_id: u32,
        transform: BlockTransform,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingGeometricEntity {
    pub(crate) id: u64,
    pub(crate) layer_id: u32,
    pub(crate) visible: bool,
    pub(crate) appearance: EntityAppearance,
    pub(crate) geometry: EntityGeometry,
}

#[derive(Clone, Debug)]
pub(crate) struct DrawingEntityRecord {
    pub id: u64,
    pub kind: &'static str,
    pub layer_id: u32,
    pub appearance: EntityAppearance,
    pub visible: bool,
    pub geometry: EntityGeometry,
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl DrawingGeometricEntity {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn layer_id(&self) -> u32 {
        self.layer_id
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn appearance(&self) -> &EntityAppearance {
        &self.appearance
    }

    pub fn geometry(&self) -> &EntityGeometry {
        &self.geometry
    }
}
