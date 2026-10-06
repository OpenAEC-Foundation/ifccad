//! Primitive preparation returns evaluated source/target pairs, never an accuracy verdict.
use crate::{
    geometry, CadConstructionError, CadGeometryError, GeometryAssessment, GeometryFailure,
    GeometryFailureReason, GeometrySource, GeometryStage, GeometryTolerance,
};
use geometry::{
    blocks::{PairedCurve, PairedPoint},
    circular,
};
use num_rational::BigRational;
use ocdraw::geometry_kernel::{
    validate_geometry, CoordinateFrame3, CoordinateLengthUnit, GeometryRef, OwnedGeometry,
    PlanarVertices, Point3, Vector3,
};
use opencadcodec::{EntityType, Handle};

#[derive(Clone)]
pub struct GeometryPair {
    pub points: Vec<(PairedPoint, BigRational)>,
    pub curves: Vec<PairedCurve>,
}
impl GeometryPair {
    pub fn points(source: &[[f64; 3]], target: &[[f64; 3]]) -> Result<Self, CadPreparationError> {
        if source.len() != target.len()
            || source
                .iter()
                .chain(target)
                .flatten()
                .any(|v| !v.is_finite())
        {
            return Err(CadPreparationError::InvalidGeometry);
        }
        let points = source
            .iter()
            .zip(target)
            .map(|(a, b)| {
                let pair = PairedPoint::new(
                    a.map(geometry::numeric::exact),
                    b.map(geometry::numeric::exact),
                );
                let d2 = pair.squared_deviation().1;
                (pair, d2)
            })
            .collect();
        Ok(Self {
            points,
            curves: Vec::new(),
        })
    }
    fn exact(points: Vec<[f64; 3]>) -> Self {
        Self {
            points: points
                .into_iter()
                .map(|p| (PairedPoint::exact(p), BigRational::from_integer(0.into())))
                .collect(),
            curves: vec![],
        }
    }
    fn curve(
        points: Option<Vec<(PairedPoint, BigRational)>>,
        curve: Option<PairedCurve>,
    ) -> Result<Self, CadPreparationError> {
        Ok(Self {
            points: points.ok_or(CadPreparationError::OutOfRange)?,
            curves: vec![curve.ok_or(CadPreparationError::OutOfRange)?],
        })
    }
}
pub struct PreparedNativeGeometry {
    pub geometry: OwnedGeometry,
    pub pair: GeometryPair,
    pub normal_normalized: bool,
}
pub struct PreparedCadGeometry {
    pub entity: EntityType,
    pub pair: GeometryPair,
    pub parameterization_changed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CadPreparationError {
    #[error("unsupported geometric parameterization")]
    UnsupportedGeometry,
    #[error("invalid CAD primitive geometry")]
    InvalidGeometry,
    #[error("prepared geometry exceeds finite numeric range")]
    OutOfRange,
    #[error("numerical preparation failed at {stage:?}: {reason:?}")]
    Numerical {
        stage: GeometryStage,
        reason: GeometryFailureReason,
    },
}
#[derive(Clone)]
struct PreparationSource;
impl GeometrySource for PreparationSource {
    fn cad_entity(_: Handle, _: String) -> Self {
        Self
    }
    fn occurrence(_: Vec<Self>, _: Self) -> Self {
        Self
    }
}
impl From<Box<GeometryFailure<PreparationSource>>> for CadPreparationError {
    fn from(e: Box<GeometryFailure<PreparationSource>>) -> Self {
        Self::Numerical {
            stage: e.stage,
            reason: e.reason,
        }
    }
}
impl From<CadConstructionError> for CadPreparationError {
    fn from(_: CadConstructionError) -> Self {
        Self::OutOfRange
    }
}
impl From<CadGeometryError<PreparationSource>> for CadPreparationError {
    fn from(e: CadGeometryError<PreparationSource>) -> Self {
        match e {
            CadGeometryError::Geometry(e) => e.into(),
            CadGeometryError::Cad(_) => Self::OutOfRange,
        }
    }
}
fn preparation_assessment() -> GeometryAssessment<PreparationSource> {
    // This permissive construction limit is not exposed as conversion acceptance.
    // Every returned pair must be checked under the caller's actual limit.
    GeometryAssessment::new(
        GeometryTolerance::drawing_units(f64::MAX).unwrap(),
        CoordinateLengthUnit::Unitless,
    )
    .unwrap()
}
fn xyz(v: opencadcodec::Vector3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
fn finite(v: opencadcodec::Vector3) -> bool {
    xyz(v).into_iter().all(f64::is_finite)
}

fn planar_curves(
    native: CoordinateFrame3,
    vertices: &[[f64; 3]],
    closed: bool,
    cad: &opencadcodec::LwPolyline,
    reverse: bool,
) -> Result<Vec<PairedCurve>, CadPreparationError> {
    use geometry::{blocks::Range, numeric::exact};
    let matrix = opencadcodec::types::Matrix3::arbitrary_axis(cad.normal).m;
    if !matrix.iter().flatten().all(|v| v.is_finite()) || vertices.len() != cad.vertices.len() {
        return Err(CadPreparationError::InvalidGeometry);
    }
    let no = native.origin().components().map(exact);
    let nx = native.x_axis().components().map(exact);
    let ny = native.y_axis().components().map(exact);
    let co: [BigRational; 3] = std::array::from_fn(|i| exact(matrix[i][2]) * exact(cad.elevation));
    let cx: [BigRational; 3] = std::array::from_fn(|i| exact(matrix[i][0]));
    let cy: [BigRational; 3] = std::array::from_fn(|i| exact(matrix[i][1]));
    let components = |a: [f64; 3],
                      b: [f64; 3],
                      origin: &[BigRational; 3],
                      x: &[BigRational; 3],
                      y: &[BigRational; 3]| {
        let (sx, sy, ex, ey, bulge) = (
            exact(a[0]),
            exact(a[1]),
            exact(b[0]),
            exact(b[1]),
            exact(a[2]),
        );
        let factor = (exact(1.) / &bulge - &bulge) / exact(4.);
        let center_x = (&sx + &ex) / exact(2.) - (&ey - &sy) * &factor;
        let center_y = (&sy + &ey) / exact(2.) + (&ex - &sx) * factor;
        let rx = &sx - &center_x;
        let ry = &sy - &center_y;
        let center: [BigRational; 3] =
            std::array::from_fn(|i| &origin[i] + &x[i] * &center_x + &y[i] * &center_y);
        let cosine = std::array::from_fn(|i| Range::point(&x[i] * &rx + &y[i] * &ry));
        let sine = std::array::from_fn(|i| Range::point(-&x[i] * &ry + &y[i] * &rx));
        (center, [cosine, sine])
    };
    let mut curves = Vec::new();
    for i in 0..if closed {
        vertices.len()
    } else {
        vertices.len() - 1
    } {
        let a = vertices[i];
        let b = vertices[(i + 1) % vertices.len()];
        if a[2] == 0. {
            continue;
        }
        let v = &cad.vertices[i];
        let w = &cad.vertices[(i + 1) % cad.vertices.len()];
        if v.bulge != a[2] {
            return Err(CadPreparationError::UnsupportedGeometry);
        }
        let (nc, na) = components(a, b, &no, &nx, &ny);
        let (cc, ca) = components(
            [v.location.x, v.location.y, v.bulge],
            [w.location.x, w.location.y, w.bulge],
            &co,
            &cx,
            &cy,
        );
        // A complete circle is a conservative enclosure of the directed segment.
        // It avoids assigning an uncertified rounded atan span to an exact bulge.
        curves.push(if reverse {
            PairedCurve::new(cc, ca, nc, na)
        } else {
            PairedCurve::new(nc, na, cc, ca)
        });
    }
    Ok(curves)
}

pub fn prepare_from_cad(
    entity: &EntityType,
) -> Result<PreparedNativeGeometry, CadPreparationError> {
    let mut normalized = false;
    let (geometry, pair) = match entity {
        EntityType::Line(l) if finite(l.start) && finite(l.end) => (
            OwnedGeometry::Line {
                start: xyz(l.start),
                end: xyz(l.end),
            },
            GeometryPair::exact(vec![xyz(l.start), xyz(l.end)]),
        ),
        EntityType::Point(p) if finite(p.location) && p.x_axis_angle.is_finite() => {
            let basis =
                geometry::cad_plane(p.normal).ok_or(CadPreparationError::InvalidGeometry)?;
            let (s, c) = p.x_axis_angle.sin_cos();
            let rotated = |a: f64, b: f64| {
                opencadcodec::Vector3::new(
                    a * basis.u[0] + b * basis.v[0],
                    a * basis.u[1] + b * basis.v[1],
                    a * basis.u[2] + b * basis.v[2],
                )
            };
            let (x, y) = geometry::orthonormal_pair(rotated(c, s), rotated(-s, c))
                .ok_or(CadPreparationError::InvalidGeometry)?;
            let placement = CoordinateFrame3::try_new(
                Point3::new(p.location.x, p.location.y, p.location.z),
                Vector3::new(x.x, x.y, x.z),
                Vector3::new(y.x, y.y, y.z),
            )
            .map_err(|_| CadPreparationError::InvalidGeometry)?;
            normalized = geometry::stored_normal(placement) != Some(p.normal);
            (
                OwnedGeometry::Point { placement },
                GeometryPair::exact(vec![xyz(p.location)]),
            )
        }
        EntityType::Circle(c) if finite(c.center) && c.radius.is_finite() && c.radius > 0. => {
            let placement = circular::from_cad_ocs(c.center, c.normal)
                .ok_or(CadPreparationError::InvalidGeometry)?;
            normalized = geometry::stored_normal(placement) != Some(c.normal);
            (
                OwnedGeometry::Circle {
                    placement,
                    radius: c.radius,
                },
                GeometryPair::curve(
                    circular::export_circle_sample_pairs(c, placement),
                    circular::export_circle_curve(c, placement),
                )?,
            )
        }
        EntityType::Arc(a)
            if finite(a.center)
                && a.radius.is_finite()
                && a.radius > 0.
                && a.start_angle.is_finite()
                && a.end_angle.is_finite() =>
        {
            let placement = circular::from_cad_ocs(a.center, a.normal)
                .ok_or(CadPreparationError::InvalidGeometry)?;
            let sweep = (a.end_angle - a.start_angle).rem_euclid(std::f64::consts::TAU);
            if !sweep.is_finite() || sweep <= 0. || sweep >= std::f64::consts::TAU {
                return Err(CadPreparationError::InvalidGeometry);
            }
            normalized = geometry::stored_normal(placement) != Some(a.normal);
            (
                OwnedGeometry::Arc {
                    placement,
                    radius: a.radius,
                    start: a.start_angle,
                    sweep,
                },
                GeometryPair::curve(
                    circular::export_arc_sample_pairs(a, placement, sweep),
                    circular::export_arc_curve(a, placement, sweep),
                )?,
            )
        }
        EntityType::Ellipse(e) => {
            if !finite(e.center)
                || !finite(e.major_axis)
                || !e.minor_axis_ratio.is_finite()
                || e.minor_axis_ratio <= 0.
                || e.minor_axis_ratio > 1.
                || !e.start_parameter.is_finite()
                || !e.end_parameter.is_finite()
            {
                return Err(CadPreparationError::InvalidGeometry);
            }
            let (placement, major, minor) =
                circular::from_cad_ellipse(e).ok_or(CadPreparationError::InvalidGeometry)?;
            let difference = e.end_parameter - e.start_parameter;
            let full = difference == std::f64::consts::TAU;
            let sweep = if full {
                difference
            } else {
                difference.rem_euclid(std::f64::consts::TAU)
            };
            if !sweep.is_finite() || sweep <= 0. {
                return Err(CadPreparationError::InvalidGeometry);
            }
            normalized = geometry::stored_normal(placement) != Some(e.normal);
            (
                OwnedGeometry::Ellipse {
                    placement,
                    major,
                    minor,
                    arc: (!full).then_some((e.start_parameter, sweep)),
                },
                GeometryPair::curve(
                    circular::export_ellipse_sample_pairs(
                        e,
                        placement,
                        major,
                        minor,
                        e.start_parameter,
                        sweep,
                    ),
                    circular::export_ellipse_curve(e, placement, major, minor, sweep),
                )?,
            )
        }
        EntityType::LwPolyline(p) => {
            if p.vertices.len() < 2
                || !p.elevation.is_finite()
                || p.vertices.iter().any(|v| {
                    !v.location.x.is_finite() || !v.location.y.is_finite() || !v.bulge.is_finite()
                })
            {
                return Err(CadPreparationError::InvalidGeometry);
            }
            let (placement, _, changed) = geometry::from_cad(p, &mut preparation_assessment())?;
            normalized = changed;
            let points = geometry::blocks::polyline_pairs(p, placement)
                .into_iter()
                .map(|p| {
                    let d = p.squared_deviation().1;
                    (p, d)
                })
                .collect();
            let vertices = p
                .vertices
                .iter()
                .map(|v| [v.location.x, v.location.y, v.bulge])
                .collect::<Vec<_>>();
            let curves = planar_curves(placement, &vertices, p.is_closed, p, true)?;
            (
                OwnedGeometry::PlanarPolyline {
                    placement,
                    vertices,
                    closed: p.is_closed,
                },
                GeometryPair { points, curves },
            )
        }
        EntityType::Polyline2D(p) => {
            if p.vertices.iter().any(|v| v.location.z != 0.) {
                return Err(CadPreparationError::UnsupportedGeometry);
            }
            let mut converted = opencadcodec::LwPolyline::from_points(
                p.vertices
                    .iter()
                    .map(|v| opencadcodec::Vector2::new(v.location.x, v.location.y))
                    .collect(),
            );
            converted.common = p.common.clone();
            converted.normal = p.normal;
            converted.elevation = p.elevation;
            converted.is_closed = p.flags.is_closed();
            for (a, b) in converted.vertices.iter_mut().zip(&p.vertices) {
                a.bulge = b.bulge;
            }
            return prepare_from_cad(&EntityType::LwPolyline(converted));
        }
        EntityType::Polyline3D(p) => {
            let v = p
                .vertices
                .iter()
                .map(|v| xyz(v.position))
                .collect::<Vec<_>>();
            (
                OwnedGeometry::SpatialPolyline {
                    vertices: v.clone(),
                    closed: p.flags.closed,
                },
                GeometryPair::exact(v),
            )
        }
        EntityType::Polyline(p) => {
            let v = p
                .vertices
                .iter()
                .map(|v| xyz(v.location))
                .collect::<Vec<_>>();
            (
                OwnedGeometry::SpatialPolyline {
                    vertices: v.clone(),
                    closed: p.flags.is_closed(),
                },
                GeometryPair::exact(v),
            )
        }
        EntityType::Line(_) | EntityType::Point(_) | EntityType::Circle(_) | EntityType::Arc(_) => {
            return Err(CadPreparationError::InvalidGeometry)
        }
        _ => return Err(CadPreparationError::UnsupportedGeometry),
    };
    validate_geometry(geometry.as_ref()).map_err(|_| CadPreparationError::InvalidGeometry)?;
    Ok(PreparedNativeGeometry {
        geometry,
        pair,
        normal_normalized: normalized,
    })
}

pub fn prepare_to_cad(g: GeometryRef<'_>) -> Result<PreparedCadGeometry, CadPreparationError> {
    validate_geometry(g).map_err(|_| CadPreparationError::InvalidGeometry)?;
    let mut changed = false;
    let (entity, pair) = match g {
        GeometryRef::Line { start, end } => (
            EntityType::Line(opencadcodec::Line::from_coords(
                start[0], start[1], start[2], end[0], end[1], end[2],
            )),
            GeometryPair::exact(vec![start, end]),
        ),
        GeometryRef::Point { placement } => {
            let normal =
                geometry::stored_normal(placement).ok_or(CadPreparationError::OutOfRange)?;
            let basis = geometry::cad_plane(normal).ok_or(CadPreparationError::OutOfRange)?;
            let x = placement.x_axis();
            let o = placement.origin();
            let dot = |a: [f64; 3]| x.x() * a[0] + x.y() * a[1] + x.z() * a[2];
            let mut p = opencadcodec::Point::from_coords(o.x(), o.y(), o.z());
            p.normal = normal;
            p.x_axis_angle = dot(basis.v).atan2(dot(basis.u));
            (
                EntityType::Point(p),
                GeometryPair::exact(vec![o.components()]),
            )
        }
        GeometryRef::Circle { placement, radius } => {
            let (center, normal, phase) =
                circular::to_cad_ocs(placement, false).ok_or(CadPreparationError::OutOfRange)?;
            let mut c = opencadcodec::Circle::from_center_radius(center, radius);
            c.normal = normal;
            let pair = GeometryPair::curve(
                Some(circular::circle_sample_pairs(placement, radius, phase, &c)?),
                circular::import_circle_curve(placement, radius, phase, &c),
            )?;
            (EntityType::Circle(c), pair)
        }
        GeometryRef::Arc {
            placement,
            radius,
            start,
            sweep,
        } => {
            let (center, normal, phase) = circular::to_cad_ocs(placement, sweep < 0.)
                .ok_or(CadPreparationError::OutOfRange)?;
            let target_start = phase + if sweep < 0. { -start } else { start };
            let mut a = opencadcodec::Arc::from_center_radius_angles(
                center,
                radius,
                target_start,
                target_start + sweep.abs(),
            );
            a.normal = normal;
            validate_target_partial_span(a.start_angle, a.end_angle)?;
            let pair = GeometryPair::curve(
                Some(circular::arc_sample_pairs(
                    placement, radius, start, sweep, &a,
                )?),
                circular::import_arc_curve(placement, radius, start, sweep, &a),
            )?;
            (EntityType::Arc(a), pair)
        }
        GeometryRef::Ellipse {
            placement,
            major,
            minor,
            arc,
        } => {
            let (start, sweep) = arc.unwrap_or((0., std::f64::consts::TAU));
            let mut e = circular::to_cad_ellipse(placement, major, minor, sweep < 0.)
                .ok_or(CadPreparationError::OutOfRange)?;
            e.start_parameter = if sweep < 0. { -start } else { start };
            e.end_parameter = e.start_parameter + sweep.abs();
            if arc.is_some() {
                validate_target_partial_span(e.start_parameter, e.end_parameter)?;
            }
            if e.minor_axis_ratio <= 0.
                || !e.minor_axis_ratio.is_finite()
                || ![e.major_axis.x, e.major_axis.y, e.major_axis.z]
                    .into_iter()
                    .all(f64::is_finite)
                || e.major_axis == opencadcodec::Vector3::ZERO
            {
                return Err(unrepresentable_target());
            }
            let pair = GeometryPair::curve(
                circular::ellipse_sample_pairs(placement, major, minor, start, sweep, &e),
                circular::import_ellipse_curve(placement, major, minor, start, sweep, &e),
            )?;
            (EntityType::Ellipse(e), pair)
        }
        GeometryRef::PlanarPolyline {
            placement,
            vertices,
            closed,
        } => {
            let separate;
            let v = match vertices {
                PlanarVertices::Packed(v) => v,
                PlanarVertices::Separate { xy, bulges } => {
                    separate = xy
                        .iter()
                        .zip(bulges)
                        .map(|(p, b)| [p[0], p[1], *b])
                        .collect::<Vec<_>>();
                    &separate
                }
            };
            let (p, _, parameterization) = geometry::to_cad_parts(
                placement,
                v,
                closed,
                PreparationSource,
                &mut preparation_assessment(),
            )?;
            changed = parameterization;
            let count = if closed {
                p.vertices.len()
            } else {
                p.vertices.len() - 1
            };
            for i in 0..count {
                if p.vertices[i].bulge != 0.
                    && p.vertices[i].location == p.vertices[(i + 1) % p.vertices.len()].location
                {
                    return Err(unrepresentable_target());
                }
            }
            let points = geometry::blocks::import_polyline_parts(placement, v, closed, &p)
                .into_iter()
                .map(|p| {
                    let d = p.squared_deviation().1;
                    (p, d)
                })
                .collect();
            let curves = planar_curves(placement, v, closed, &p, false)?;
            (EntityType::LwPolyline(p), GeometryPair { points, curves })
        }
        GeometryRef::SpatialPolyline { vertices, closed } => {
            let mut p = opencadcodec::entities::Polyline3D::from_points(
                vertices
                    .iter()
                    .map(|v| opencadcodec::Vector3::new(v[0], v[1], v[2]))
                    .collect(),
            );
            p.flags.closed = closed;
            (
                EntityType::Polyline3D(p),
                GeometryPair::exact(vertices.to_vec()),
            )
        }
    };
    Ok(PreparedCadGeometry {
        entity,
        pair,
        parameterization_changed: changed,
    })
}

// CAD endpoint/radius representation must still describe a valid curve.
// A generous geometric limit does not authorize zero/full partial spans,
// a zero ellipse ratio or coincident endpoints of a nonzero bulged segment.
fn unrepresentable_target() -> CadPreparationError {
    CadPreparationError::Numerical {
        stage: GeometryStage::TargetConstruction,
        reason: GeometryFailureReason::NumericalProofIncomplete,
    }
}
fn validate_target_partial_span(start: f64, end: f64) -> Result<(), CadPreparationError> {
    let span = end - start;
    if !start.is_finite() || !end.is_finite() || span <= 0. || span >= std::f64::consts::TAU {
        Err(unrepresentable_target())
    } else {
        Ok(())
    }
}
