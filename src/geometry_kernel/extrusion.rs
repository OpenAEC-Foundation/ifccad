//! Signed primitive sweeps; no CAD runtime or surface tessellation.
use super::{
    numeric::{exact, round_down, round_up},
    Bounds3d, GeometryRef, GeometryValidationError as Error, Point3,
};
use num_rational::BigRational as Q;
use num_traits::Signed;

fn sweep(g: GeometryRef<'_>) -> Option<(f64, [Q; 3])> {
    let (h, p) = match g {
        GeometryRef::Line {
            thickness,
            extrusion_direction,
            ..
        } => return Some((thickness, extrusion_direction.map(exact))),
        GeometryRef::Point {
            thickness,
            placement,
            ..
        }
        | GeometryRef::Circle {
            thickness,
            placement,
            ..
        }
        | GeometryRef::Arc {
            thickness,
            placement,
            ..
        }
        | GeometryRef::PlanarPolyline {
            thickness,
            placement,
            ..
        } => (thickness, placement),
        _ => return None,
    };
    let x = p.x_axis().components().map(exact);
    let y = p.y_axis().components().map(exact);
    Some((
        h,
        std::array::from_fn(|i| {
            &x[(i + 1) % 3] * &y[(i + 2) % 3] - &x[(i + 2) % 3] * &y[(i + 1) % 3]
        }),
    ))
}
pub(super) fn validate(g: GeometryRef<'_>) -> Result<(), Error> {
    if let GeometryRef::Line {
        extrusion_direction,
        ..
    } = g
    {
        if !extrusion_direction.into_iter().all(f64::is_finite) {
            return Err(Error::InvalidGeometry);
        }
        let d = extrusion_direction.map(exact);
        let squared: Q = d.iter().map(|v| v * v).sum();
        if (squared - exact(1.)).abs() > exact(1e-12) {
            return Err(Error::InvalidGeometry);
        }
    }
    if sweep(g).is_some_and(|(h, _)| !h.is_finite()) {
        return Err(Error::InvalidGeometry);
    }
    Ok(())
}
pub(super) fn extrude_bounds(base: Bounds3d, g: GeometryRef<'_>) -> Result<Bounds3d, Error> {
    validate(g)?;
    let Some((h, d)) = sweep(g) else {
        return Ok(base);
    };
    if h == 0. {
        return Ok(base);
    }
    let mut lo = base.min().components();
    let mut hi = base.max().components();
    for i in 0..3 {
        let shift = &d[i] * exact(h);
        if shift.is_negative() {
            lo[i] = round_down(&(exact(lo[i]) + shift)).map_err(|_| Error::EnclosureOutOfRange)?
        } else {
            hi[i] = round_up(&(exact(hi[i]) + shift)).map_err(|_| Error::EnclosureOutOfRange)?
        }
    }
    Ok(Bounds3d::new(
        Point3::new(lo[0], lo[1], lo[2]),
        Point3::new(hi[0], hi[1], hi[2]),
    ))
}
