//! CAD preparation and optional surface evidence for ordered filled descriptors.
use super::{blocks::PairedPoint, numeric::exact};
use crate::{
    CadPreparationError as Error, GeometryPair, PreparedCadGeometry, PreparedNativeGeometry,
};
use ifccad::geometry_kernel::{
    filled_quadrilateral_surface_cover, CoordinateFrame3, FilledQuadrilateralSurfaceCover as Cover,
    GeometryRef, OwnedGeometry,
};
use num_rational::BigRational as Q;
use opencadcodec::entities::Solid;
use opencadcodec::EntityType;

/// Common original-slot cells; None means unassessed surface, never invalid geometry.
pub fn common_filled_surface_cover(
    a: &[[f64; 2]; 4],
    b: &[[f64; 2]; 4],
) -> Option<Vec<[usize; 3]>> {
    for i in 0..4 {
        for j in 0..i {
            if (a[i] == a[j]) != (b[i] == b[j]) {
                return None;
            }
        }
    }
    match (
        filled_quadrilateral_surface_cover(a)?,
        filled_quadrilateral_surface_cover(b)?,
    ) {
        (Cover::Triangle { boundary: x }, Cover::Triangle { boundary: y }) if x == y => {
            Some(vec![x])
        }
        (
            Cover::Quadrilateral {
                admissible_diagonals: x,
            },
            Cover::Quadrilateral {
                admissible_diagonals: y,
            },
        ) => {
            if x[0] && y[0] {
                Some(vec![[0, 1, 3], [0, 3, 2]])
            } else if x[1] && y[1] {
                Some(vec![[0, 1, 2], [1, 3, 2]])
            } else {
                None
            }
        }
        _ => None,
    }
}

fn raw(s: &Solid) -> [opencadcodec::Vector3; 4] {
    [
        s.first_corner,
        s.second_corner,
        s.third_corner,
        s.fourth_corner,
    ]
}
fn native_points(p: CoordinateFrame3, c: &[[f64; 2]; 4], h: f64) -> Vec<[Q; 3]> {
    let o = p.origin().components().map(exact);
    let u = p.x_axis().components().map(exact);
    let v = p.y_axis().components().map(exact);
    let n: [Q; 3] = std::array::from_fn(|i| {
        &u[(i + 1) % 3] * &v[(i + 2) % 3] - &u[(i + 2) % 3] * &v[(i + 1) % 3]
    });
    [0., h]
        .into_iter()
        .flat_map(|z| {
            c.iter()
                .map(|c| {
                    std::array::from_fn(|i| {
                        &o[i] + &u[i] * exact(c[0]) + &v[i] * exact(c[1]) + &n[i] * exact(z)
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn cad_points(s: &Solid) -> Result<Vec<[Q; 3]>, Error> {
    // Evaluate the actual codec OCS, not the reconstructed native frame.
    let m = opencadcodec::types::Matrix3::arbitrary_axis(s.normal).m;
    if !m.iter().flatten().all(|v| v.is_finite()) {
        return Err(Error::InvalidGeometry);
    }
    Ok([0., s.thickness]
        .into_iter()
        .flat_map(|h| {
            raw(s)
                .into_iter()
                .map(|c| {
                    std::array::from_fn(|i| {
                        exact(m[i][0]) * exact(c.x)
                            + exact(m[i][1]) * exact(c.y)
                            + exact(m[i][2]) * (exact(c.z) + exact(h))
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect())
}
fn pair(a: Vec<[Q; 3]>, b: Vec<[Q; 3]>) -> GeometryPair {
    GeometryPair {
        bands: vec![],
        points: a
            .into_iter()
            .zip(b)
            .map(|(a, b)| {
                let p = PairedPoint::new(a, b);
                let d = p.squared_deviation().1;
                (p, d)
            })
            .collect(),
        curves: vec![],
    }
}
pub(crate) fn from_cad(s: &Solid) -> Result<PreparedNativeGeometry, Error> {
    let c = raw(s);
    if !c
        .iter()
        .all(|c| [c.x, c.y, c.z].into_iter().all(f64::is_finite))
        || !s.thickness.is_finite()
    {
        return Err(Error::InvalidGeometry);
    }
    let basis = super::cad_plane(s.normal).ok_or(Error::InvalidGeometry)?;
    if c.iter().any(|c| c.z != s.first_corner.z) {
        return Err(Error::UnsupportedGeometry);
    }
    let p = super::circular::from_cad_ocs(
        opencadcodec::Vector3::new(0., 0., s.first_corner.z),
        s.normal,
    )
    .ok_or(Error::OutOfRange)?;
    let corners = c.map(|c| [c.x, c.y]);
    let geometry = OwnedGeometry::FilledQuadrilateral {
        placement: p,
        corners,
        thickness: s.thickness,
    };
    ifccad::geometry_kernel::validate_geometry(geometry.as_ref()).map_err(|_| Error::OutOfRange)?;
    Ok(PreparedNativeGeometry {
        geometry,
        pair: pair(cad_points(s)?, native_points(p, &corners, s.thickness)),
        normal_normalized: [s.normal.x, s.normal.y, s.normal.z] != basis.n,
    })
}
pub(crate) fn to_cad(
    p: CoordinateFrame3,
    c: &[[f64; 2]; 4],
    h: f64,
) -> Result<PreparedCadGeometry, Error> {
    ifccad::geometry_kernel::validate_geometry(GeometryRef::FilledQuadrilateral {
        placement: p,
        corners: c,
        thickness: h,
    })
    .map_err(|_| Error::InvalidGeometry)?;
    let vertices = c.map(|[x, y]| [x, y, 0.]);
    let (poly, _, changed) = super::to_cad_parts(
        p,
        &vertices,
        false,
        crate::prepare::PreparationSource,
        &mut crate::prepare::preparation_assessment(),
    )?;
    let v: Vec<_> = poly
        .vertices
        .iter()
        .map(|v| opencadcodec::Vector3::new(v.location.x, v.location.y, poly.elevation))
        .collect();
    let s = Solid::new(v[0], v[1], v[2], v[3])
        .with_normal(poly.normal)
        .with_thickness(h);
    let pair = pair(native_points(p, c, h), cad_points(&s)?);
    Ok(PreparedCadGeometry {
        entity: EntityType::Solid(s),
        pair,
        parameterization_changed: changed,
    })
}
