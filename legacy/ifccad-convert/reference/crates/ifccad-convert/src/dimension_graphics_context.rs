//! Exact picture frame rebinding and owner-space proof maps.
use crate::IfccadConversionError as Error;
use cad_geometry_convert::geometry::{
    blocks::EvaluatedBlock,
    numeric::{exact, round_nearest},
};
use ifccad::{geometry_kernel::CoordinateFrame3, *};
use num_rational::BigRational as Q;
fn failure(detail: &str) -> Error {
    Error::DimensionPreparation {
        location: "graphics/frame".into(),
        source: cad_dimension::CadDimensionPreparationError::Numeric {
            field: "pictureBlock/frame".into(),
            detail: detail.into(),
        },
    }
}
#[derive(Clone)]
pub(crate) struct PictureMap {
    pub origin: [Q; 3],
    pub axes: [[Q; 3]; 3],
}
impl PictureMap {
    pub fn identity() -> Self {
        Self {
            origin: std::array::from_fn(|_| Q::from_integer(0.into())),
            axes: std::array::from_fn(|i| {
                std::array::from_fn(|j| exact(if i == j { 1. } else { 0. }))
            }),
        }
    }
    pub fn native(p: CoordinateFrame3) -> Self {
        let x = p.x_axis().components().map(exact);
        let y = p.y_axis().components().map(exact);
        let n: [Q; 3] = std::array::from_fn(|i| {
            &x[(i + 1) % 3] * &y[(i + 2) % 3] - &x[(i + 2) % 3] * &y[(i + 1) % 3]
        });
        Self {
            origin: p.origin().components().map(exact),
            axes: std::array::from_fn(|i| [x[i].clone(), y[i].clone(), n[i].clone()]),
        }
    }
    pub fn cad(d: &opencadcodec::entities::Dimension, root_base: [f64; 3]) -> Result<Self, Error> {
        let qualified =
            cad_dimension::prepare_dimension_picture_frame(d, root_base).map_err(|source| {
                Error::DimensionPreparation {
                    location: "graphics/frame".into(),
                    source,
                }
            })?;
        match qualified {
            cad_dimension::CadDimensionPictureFrame::Owner => Ok(Self::identity()),
            cad_dimension::CadDimensionPictureFrame::PlanarOcs(_) => {
                let p = cad_geometry_convert::geometry::cad_plane(d.base().normal)
                    .expect("qualified picture plane");
                Ok(Self {
                    origin: p.n.map(|n| exact(n) * exact(d.base().text_middle_point.z)),
                    axes: std::array::from_fn(|i| [exact(p.u[i]), exact(p.v[i]), exact(p.n[i])]),
                })
            }
        }
    }
    pub fn evaluated(&self) -> EvaluatedBlock {
        EvaluatedBlock {
            origin: self.origin.clone(),
            base: std::array::from_fn(|_| Q::from_integer(0.into())),
            rotation: 0.,
            cosine: std::array::from_fn(|i| {
                [
                    self.axes[i][0].clone(),
                    self.axes[i][1].clone(),
                    Q::from_integer(0.into()),
                ]
            }),
            sine: std::array::from_fn(|_| std::array::from_fn(|_| Q::from_integer(0.into()))),
            constant: std::array::from_fn(|i| {
                [
                    Q::from_integer(0.into()),
                    Q::from_integer(0.into()),
                    self.axes[i][2].clone(),
                ]
            }),
        }
    }
}
pub(crate) struct PictureTransform {
    matrix: [[Q; 3]; 3],
    offset: [Q; 3],
}
impl PictureTransform {
    pub fn new(source: PictureMap, target: PictureMap) -> Result<Self, Error> {
        let a = &target.axes;
        let det = &a[0][0] * (&a[1][1] * &a[2][2] - &a[1][2] * &a[2][1])
            - &a[0][1] * (&a[1][0] * &a[2][2] - &a[1][2] * &a[2][0])
            + &a[0][2] * (&a[1][0] * &a[2][1] - &a[1][1] * &a[2][0]);
        if det == Q::from_integer(0.into()) {
            return Err(failure("singular stored picture frame"));
        }
        let inverse: [[Q; 3]; 3] = std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                (&a[(j + 1) % 3][(i + 1) % 3] * &a[(j + 2) % 3][(i + 2) % 3]
                    - &a[(j + 1) % 3][(i + 2) % 3] * &a[(j + 2) % 3][(i + 1) % 3])
                    / &det
            })
        });
        Ok(Self {
            matrix: std::array::from_fn(|i| {
                std::array::from_fn(|j| (0..3).map(|k| &inverse[i][k] * &source.axes[k][j]).sum())
            }),
            offset: std::array::from_fn(|i| {
                (0..3)
                    .map(|k| &inverse[i][k] * (&source.origin[k] - &target.origin[k]))
                    .sum()
            }),
        })
    }
    pub fn position(&self, v: [f64; 3], translate: bool) -> Result<[f64; 3], Error> {
        if v.iter().any(|x| !x.is_finite()) {
            return Err(failure("nonfinite picture coordinate"));
        }
        let mut out = [0.; 3];
        for (i, o) in out.iter_mut().enumerate() {
            let mut q: Q = (0..3).map(|j| &self.matrix[i][j] * exact(v[j])).sum();
            if translate {
                q += &self.offset[i];
            }
            *o =
                round_nearest(&q).map_err(|_| failure("picture coordinate out of finite range"))?;
        }
        Ok(out)
    }
    fn placement(&self, p: &mut IfccadPlacement) -> Result<(), Error> {
        p.origin = self.position(p.origin, true)?;
        p.x_axis = self.position(p.x_axis, false)?;
        p.y_axis = self.position(p.y_axis, false)?;
        p.coordinate_frame().map_err(Error::CoreValidation)?;
        Ok(())
    }
    pub fn rebind(&self, k: &mut IfccadDimensionGraphicsKind) -> Result<(), Error> {
        use IfccadDimensionGraphicsKind as K;
        match k {
            K::LineSegment { start, end } => {
                *start = self.position(*start, true)?;
                *end = self.position(*end, true)?;
            }
            K::SpatialPolyline { vertices, .. } => {
                for p in vertices {
                    *p = self.position(*p, true)?;
                }
            }
            K::Point { placement }
            | K::Circle { placement, .. }
            | K::Arc { placement, .. }
            | K::Ellipse { placement, .. }
            | K::EllipseArc { placement, .. }
            | K::PlanarPolyline { placement, .. } => self.placement(placement)?,
            K::BlockInstance { transform, .. } => self.placement(&mut transform.placement)?,
            K::Solid(v) | K::Trace(v) => self.placement(&mut v.placement)?,
            K::Hatch(v) => self.placement(&mut v.placement)?,
            K::Text(v) => self.placement(&mut v.placement)?,
            K::MText(v) => self.placement(&mut v.placement)?,
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picture_rebind_inverts_the_stored_basis_instead_of_transposing_it() {
        let p = IfccadPlacement {
            origin: [7., 9., 1.],
            x_axis: [1., 1e-14, 0.],
            y_axis: [0., 1., 0.],
        };
        let map = PictureTransform::new(
            PictureMap::identity(),
            PictureMap::native(p.coordinate_frame().unwrap()),
        )
        .unwrap();
        let mut k = IfccadDimensionGraphicsKind::LineSegment {
            start: [10., 20., 3.],
            end: [12., 20., 3.],
        };
        map.rebind(&mut k).unwrap();
        let IfccadDimensionGraphicsKind::LineSegment { start, end } = k else {
            panic!()
        };
        assert_eq!(start[0], 3.);
        assert_eq!(end[0], 5.);
        assert_eq!(start[1], 10.99999999999997);
        assert_eq!(start[2], 2.);
    }
    #[test]
    fn picture_rebind_preserves_filled_slots_thickness_and_marker_local_geometry() {
        let frame = IfccadPlacement {
            origin: [98., 0., 0.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        };
        let map = PictureTransform::new(
            PictureMap::identity(),
            PictureMap::native(frame.coordinate_frame().unwrap()),
        )
        .unwrap();
        let corners = [[0., 0.], [4., 4.], [0., 4.], [4., 0.]];
        let mut k = IfccadDimensionGraphicsKind::Trace(IfccadFilledQuadrilateral {
            placement: IfccadPlacement {
                origin: [100., 9., 3.],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            corners,
            thickness: -3.,
        });
        map.rebind(&mut k).unwrap();
        let IfccadDimensionGraphicsKind::Trace(v) = k else {
            panic!()
        };
        assert_eq!(v.placement.origin, [2., 9., 3.]);
        assert_eq!(v.corners, corners);
        assert_eq!(v.thickness, -3.);
    }
}
