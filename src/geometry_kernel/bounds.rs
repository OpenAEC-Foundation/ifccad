use super::{Bounds3d, GeometryRef, GeometryValidationError as Error, PaperFrame, Point2, Point3};

pub fn geometry_bounds(g: GeometryRef<'_>) -> Result<Bounds3d, Error> {
    super::validation::validate_parameters(g)?;
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    let mut union = |a: [f64; 3], b: [f64; 3]| {
        for i in 0..3 {
            min[i] = min[i].min(a[i]);
            max[i] = max[i].max(b[i]);
        }
    };
    match g {
        GeometryRef::Line { start, end } => {
            union(start, start);
            union(end, end);
        }
        GeometryRef::Point { placement } => union(
            placement.origin().components(),
            placement.origin().components(),
        ),
        GeometryRef::Circle { placement, radius } => {
            let b = super::circular_bounds(placement.components(), radius, None)
                .ok_or(Error::EnclosureOutOfRange)?;
            union(b.min().components(), b.max().components());
        }
        GeometryRef::Arc {
            placement,
            radius,
            start,
            sweep,
        } => {
            let b = super::circular_bounds(placement.components(), radius, Some((start, sweep)))
                .ok_or(Error::EnclosureOutOfRange)?;
            union(b.min().components(), b.max().components());
        }
        GeometryRef::Ellipse {
            placement,
            major,
            minor,
            arc,
        } => {
            let b = super::elliptic_bounds(placement.components(), major, minor, arc)
                .ok_or(Error::EnclosureOutOfRange)?;
            union(b.min().components(), b.max().components());
        }
        GeometryRef::SpatialPolyline { vertices, .. } => {
            for v in vertices {
                union(*v, *v);
            }
        }
        GeometryRef::PlanarPolyline {
            placement,
            vertices,
            closed,
        } => {
            for i in 0..if closed {
                vertices.len()
            } else {
                vertices.len() - 1
            } {
                let a = vertices.get(i);
                let b = vertices.get((i + 1) % vertices.len());
                let local = super::bulge_segment_bounds(
                    Point2::new(a[0], a[1]),
                    Point2::new(b[0], b[1]),
                    a[2],
                )
                .ok_or(Error::EnclosureOutOfRange)?;
                for x in [local.min().x(), local.max().x()] {
                    for y in [local.min().y(), local.max().y()] {
                        let b = placement
                            .enclose_point(Point2::new(x, y))
                            .map_err(|_| Error::EnclosureOutOfRange)?;
                        union(b.min().components(), b.max().components());
                    }
                }
            }
        }
    }
    if !min.into_iter().chain(max).all(f64::is_finite) {
        return Err(Error::EnclosureOutOfRange);
    }
    Ok(Bounds3d::new(
        Point3::new(min[0], min[1], min[2]),
        Point3::new(max[0], max[1], max[2]),
    ))
}
pub fn paper_frame_bounds(f: PaperFrame) -> Result<Bounds3d, Error> {
    use super::numeric::{exact, round_down, round_up};
    if !f
        .center
        .into_iter()
        .chain([f.width, f.height])
        .all(f64::is_finite)
        || f.width <= 0.
        || f.height <= 0.
    {
        return Err(Error::InvalidFrame);
    }
    let lo: Result<Vec<_>, _> = (0..2)
        .map(|i| round_down(&(exact(f.center[i]) - exact([f.width, f.height][i]) * exact(0.5))))
        .collect();
    let hi: Result<Vec<_>, _> = (0..2)
        .map(|i| round_up(&(exact(f.center[i]) + exact([f.width, f.height][i]) * exact(0.5))))
        .collect();
    let lo = lo.map_err(|_| Error::InvalidFrame)?;
    let hi = hi.map_err(|_| Error::InvalidFrame)?;
    Ok(Bounds3d::new(
        Point3::new(lo[0], lo[1], 0.),
        Point3::new(hi[0], hi[1], 0.),
    ))
}
