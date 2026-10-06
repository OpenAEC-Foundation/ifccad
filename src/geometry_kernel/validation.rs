use super::{geometry_bounds, paper_frame_bounds, GeometryRef, PaperFrame};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GeometryValidationError {
    #[error("geometry needs finite coordinates and valid primitive parameters")]
    InvalidGeometry,
    #[error("geometry has no finite conservative enclosure")]
    EnclosureOutOfRange,
    #[error("paper frame needs positive finite dimensions and finite enclosure")]
    InvalidFrame,
    #[error("active boundary requires a circle, full ellipse or closed planar contour")]
    UnsupportedBoundary,
    #[error("entire boundary must lie in Paper Z=0 inside the frame")]
    BoundaryOutsideFrame,
}

pub(crate) fn validate_parameters(g: GeometryRef<'_>) -> Result<(), GeometryValidationError> {
    let good = match g {
        GeometryRef::Line { start, end } => start.into_iter().chain(end).all(f64::is_finite),
        GeometryRef::Point { .. } => true,
        GeometryRef::Circle { radius, .. } => radius.is_finite() && radius > 0.,
        GeometryRef::Arc {
            radius,
            start,
            sweep,
            ..
        } => radius.is_finite() && radius > 0. && arc_valid(start, sweep),
        GeometryRef::Ellipse {
            major, minor, arc, ..
        } => {
            major.is_finite()
                && minor.is_finite()
                && minor > 0.
                && major >= minor
                && arc.is_none_or(|(s, w)| arc_valid(s, w))
        }
        GeometryRef::PlanarPolyline {
            vertices, closed, ..
        } => {
            vertices.len() >= 2
                && vertices.is_finite()
                && (0..if closed {
                    vertices.len()
                } else {
                    vertices.len() - 1
                })
                    .all(|i| {
                        let a = vertices.get(i);
                        let b = vertices.get((i + 1) % vertices.len());
                        a[2] == 0. || a[0] != b[0] || a[1] != b[1]
                    })
        }
        GeometryRef::SpatialPolyline { vertices, .. } => {
            vertices.len() >= 2 && vertices.iter().flatten().all(|v| v.is_finite())
        }
    };
    good.then_some(())
        .ok_or(GeometryValidationError::InvalidGeometry)
}
fn arc_valid(start: f64, sweep: f64) -> bool {
    start.is_finite() && sweep.is_finite() && sweep != 0. && sweep.abs() < std::f64::consts::TAU
}
pub fn validate_geometry(g: GeometryRef<'_>) -> Result<(), GeometryValidationError> {
    geometry_bounds(g).map(|_| ())
}
pub fn validate_paper_boundary(
    f: PaperFrame,
    g: GeometryRef<'_>,
) -> Result<(), GeometryValidationError> {
    paper_frame_bounds(f)?;
    let eligible = match g {
        GeometryRef::Circle { .. } | GeometryRef::Ellipse { arc: None, .. } => true,
        GeometryRef::PlanarPolyline {
            vertices,
            closed: true,
            ..
        } if vertices.is_finite() => {
            let bits = |v: f64| if v == 0. { 0 } else { v.to_bits() };
            let distinct: BTreeSet<_> = (0..vertices.len())
                .map(|i| {
                    let v = vertices.get(i);
                    (bits(v[0]), bits(v[1]))
                })
                .collect();
            distinct.len()
                >= if (0..vertices.len()).any(|i| vertices.get(i)[2] != 0.) {
                    2
                } else {
                    3
                }
        }
        _ => false,
    };
    if !eligible {
        return Err(GeometryValidationError::UnsupportedBoundary);
    }
    validate_geometry(g)?;
    if super::clip_containment::enclosed_by_frame(f, g) {
        Ok(())
    } else {
        Err(GeometryValidationError::BoundaryOutsideFrame)
    }
}
