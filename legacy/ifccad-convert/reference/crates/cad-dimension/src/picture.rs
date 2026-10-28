//! Qualification of an existing CAD cache, never generation of display geometry.
use crate::{invalid, unsupported, CadDimensionPreparationError};
use cad_geometry_convert::geometry::{
    cad_plane,
    numeric::{exact, round_nearest},
};
use ifccad::geometry_kernel::{CoordinateFrame3, Point3, Vector3};
use opencadcodec::{entities::Dimension, Vector3 as V};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CadDimensionPictureFrame {
    Owner,
    PlanarOcs(CoordinateFrame3),
}
pub fn prepare_dimension_picture_frame(
    dimension: &Dimension,
    root_base: [f64; 3],
) -> Result<CadDimensionPictureFrame, CadDimensionPreparationError> {
    let b = dimension.base();
    if root_base.into_iter().any(|x| !x.is_finite()) {
        return Err(invalid("pictureBlock/basePoint", "nonfinite root base"));
    }
    if root_base != [0.; 3]
        || b.insertion_point != V::ZERO
        || b.insertion_scale != V::new(1., 1., 1.)
        || b.insertion_rotation != 0.
    {
        return Err(unsupported(
            "pictureBlock/frame",
            "nonzero root base or cloned picture transform",
        ));
    }
    let axis = cad_plane(b.normal)
        .ok_or_else(|| invalid("pictureBlock/normal", "invalid picture normal"))?;
    if axis.n[0] == 0. && axis.n[1] == 0. && axis.n[2] > 0. {
        return Ok(CadDimensionPictureFrame::Owner);
    }
    let h = b.text_middle_point.z;
    if !h.is_finite() {
        return Err(invalid(
            "pictureBlock/elevation",
            "nonfinite cache elevation",
        ));
    }
    let mut o = [0.; 3];
    for (i, component) in o.iter_mut().enumerate() {
        *component = round_nearest(&(exact(axis.n[i]) * exact(h))).map_err(|_| {
            crate::numeric("pictureBlock/elevation", "cache origin out of finite range")
        })?;
    }
    let v = |p: [f64; 3]| Vector3::new(p[0], p[1], p[2]);
    let frame = CoordinateFrame3::try_new(Point3::new(o[0], o[1], o[2]), v(axis.u), v(axis.v))
        .map_err(|_| invalid("pictureBlock/frame", "invalid cache axes"))?;
    Ok(CadDimensionPictureFrame::PlanarOcs(frame))
}
