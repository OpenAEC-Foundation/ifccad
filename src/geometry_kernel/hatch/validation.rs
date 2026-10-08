use super::{
    bounds::local_bounds, endpoints::check_join, resolve_hatch_join_tolerance, HatchBoundary2,
    HatchEdge2, HatchFailureReason as Reason, HatchJoinToleranceRequest,
    HatchValidationError as Error,
};
use crate::geometry_kernel::{
    CoordinateFrame3, GeometryRef, GeometryValidationError, PlanarVertices, Point3, Vector3,
};
use std::collections::BTreeSet;

/// Validates stored parameters, finite local enclosures and every ordered join.
/// Does not certify simplicity, nesting, source agreement or fill evaluation.
pub fn validate_hatch_boundaries<'a>(
    boundaries: impl IntoIterator<Item = &'a HatchBoundary2>,
    join_tolerance: f64,
) -> Result<(), Error> {
    if resolve_hatch_join_tolerance(HatchJoinToleranceRequest::Coordinates(join_tolerance), None)
        .is_err()
    {
        return Err(Error {
            loop_index: None,
            edge_index: None,
            reason: Reason::InvalidTolerance,
        });
    }
    let mut any = false;
    for (index, boundary) in boundaries.into_iter().enumerate() {
        any = true;
        local_bounds(boundary, index)?;
        if let HatchBoundary2::Edges(edges) = boundary {
            for (edge_index, edge) in edges.iter().enumerate() {
                check_join(edge, &edges[(edge_index + 1) % edges.len()], join_tolerance)
                    .map_err(|reason| Error::at(index, Some(edge_index), reason))?;
            }
        }
    }
    if !any {
        return Err(Error {
            loop_index: None,
            edge_index: None,
            reason: Reason::InvalidParameters,
        });
    }
    Ok(())
}

pub(super) fn local_frame(center: [f64; 2], axis: [f64; 2]) -> Result<CoordinateFrame3, Reason> {
    CoordinateFrame3::try_new(
        Point3::new(center[0], center[1], 0.0),
        Vector3::new(axis[0], axis[1], 0.0),
        Vector3::new(-axis[1], axis[0], 0.0),
    )
    .map_err(|_| Reason::InvalidParameters)
}

fn primitive(g: GeometryRef<'_>) -> Result<(), Reason> {
    crate::geometry_kernel::validation::validate_parameters(g).map_err(|e| match e {
        GeometryValidationError::EnclosureOutOfRange => Reason::EnclosureOutOfRange,
        _ => Reason::InvalidParameters,
    })
}

pub(super) fn parameters(boundary: &HatchBoundary2, index: usize) -> Result<(), Error> {
    let result = match boundary {
        HatchBoundary2::Circle { center, radius } => {
            local_frame(*center, [1.0, 0.0]).and_then(|placement| {
                primitive(GeometryRef::Circle {
                    placement,
                    radius: *radius,
                })
            })
        }
        HatchBoundary2::Ellipse {
            center,
            x_axis,
            semi_major_radius,
            semi_minor_radius,
        } => local_frame(*center, *x_axis).and_then(|placement| {
            primitive(GeometryRef::Ellipse {
                placement,
                major: *semi_major_radius,
                minor: *semi_minor_radius,
                arc: None,
            })
        }),
        HatchBoundary2::Polyline { vertices, bulges } => {
            if vertices.len() != bulges.len() {
                Err(Reason::InvalidParameters)
            } else {
                primitive(GeometryRef::PlanarPolyline {
                    placement: CoordinateFrame3::default(),
                    vertices: PlanarVertices::Separate {
                        xy: vertices,
                        bulges,
                    },
                    closed: true,
                })
                .and_then(|()| {
                    let bits = |x: f64| if x == 0.0 { 0 } else { x.to_bits() };
                    let distinct: BTreeSet<_> =
                        vertices.iter().map(|v| (bits(v[0]), bits(v[1]))).collect();
                    if distinct.len()
                        >= if bulges.iter().any(|b| *b != 0.0) {
                            2
                        } else {
                            3
                        }
                    {
                        Ok(())
                    } else {
                        Err(Reason::InvalidParameters)
                    }
                })
            }
        }
        HatchBoundary2::Edges(edges) => {
            if edges.is_empty() {
                Err(Reason::InvalidParameters)
            } else {
                for (edge_index, edge) in edges.iter().enumerate() {
                    let value = match edge {
                        HatchEdge2::Line { start, end } => primitive(GeometryRef::Line {
                            start: [start[0], start[1], 0.0],
                            end: [end[0], end[1], 0.0],
                        }),
                        _ => {
                            let c = edge.curve().expect("arc variant");
                            local_frame(c.center, c.x_axis).and_then(|placement| {
                                primitive(GeometryRef::Ellipse {
                                    placement,
                                    major: c.major,
                                    minor: c.minor,
                                    arc: Some((c.start, c.sweep)),
                                })
                            })
                        }
                    };
                    value.map_err(|reason| Error::at(index, Some(edge_index), reason))?;
                }
                Ok(())
            }
        }
    };
    result.map_err(|reason| Error::at(index, None, reason))
}
