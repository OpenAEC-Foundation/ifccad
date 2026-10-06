use super::CoordinateFrame3;

/// Two native storage layouts for the same outgoing-bulge planar vertices.
#[derive(Clone, Copy, Debug)]
pub enum PlanarVertices<'a> {
    Packed(&'a [[f64; 3]]),
    Separate {
        xy: &'a [[f64; 2]],
        bulges: &'a [f64],
    },
}
impl PlanarVertices<'_> {
    pub(crate) fn len(self) -> usize {
        match self {
            Self::Packed(v) => v.len(),
            Self::Separate { xy, .. } => xy.len(),
        }
    }
    pub(crate) fn is_finite(self) -> bool {
        match self {
            Self::Packed(v) => v.iter().flatten().all(|v| v.is_finite()),
            Self::Separate { xy, bulges } => {
                xy.len() == bulges.len() && xy.iter().flatten().chain(bulges).all(|v| v.is_finite())
            }
        }
    }
    pub(crate) fn get(self, i: usize) -> [f64; 3] {
        match self {
            Self::Packed(v) => v[i],
            Self::Separate { xy, bulges } => [xy[i][0], xy[i][1], bulges[i]],
        }
    }
}

/// A borrowed primitive geometry view, without identity, appearance or ownership.
#[derive(Clone, Copy, Debug)]
pub enum GeometryRef<'a> {
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
        start: f64,
        sweep: f64,
    },
    Ellipse {
        placement: CoordinateFrame3,
        major: f64,
        minor: f64,
        arc: Option<(f64, f64)>,
    },
    PlanarPolyline {
        placement: CoordinateFrame3,
        vertices: PlanarVertices<'a>,
        closed: bool,
    },
    SpatialPolyline {
        vertices: &'a [[f64; 3]],
        closed: bool,
    },
}

/// Pure geometry prepared for a native adapter, independent of its storage layout.
#[derive(Clone, Debug, PartialEq)]
pub enum OwnedGeometry {
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
        start: f64,
        sweep: f64,
    },
    Ellipse {
        placement: CoordinateFrame3,
        major: f64,
        minor: f64,
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
}
impl OwnedGeometry {
    pub fn as_ref(&self) -> GeometryRef<'_> {
        match self {
            Self::Line { start, end } => GeometryRef::Line {
                start: *start,
                end: *end,
            },
            Self::Point { placement } => GeometryRef::Point {
                placement: *placement,
            },
            Self::Circle { placement, radius } => GeometryRef::Circle {
                placement: *placement,
                radius: *radius,
            },
            Self::Arc {
                placement,
                radius,
                start,
                sweep,
            } => GeometryRef::Arc {
                placement: *placement,
                radius: *radius,
                start: *start,
                sweep: *sweep,
            },
            Self::Ellipse {
                placement,
                major,
                minor,
                arc,
            } => GeometryRef::Ellipse {
                placement: *placement,
                major: *major,
                minor: *minor,
                arc: *arc,
            },
            Self::PlanarPolyline {
                placement,
                vertices,
                closed,
            } => GeometryRef::PlanarPolyline {
                placement: *placement,
                vertices: PlanarVertices::Packed(vertices),
                closed: *closed,
            },
            Self::SpatialPolyline { vertices, closed } => GeometryRef::SpatialPolyline {
                vertices,
                closed: *closed,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PaperFrame {
    pub center: [f64; 2],
    pub width: f64,
    pub height: f64,
}
