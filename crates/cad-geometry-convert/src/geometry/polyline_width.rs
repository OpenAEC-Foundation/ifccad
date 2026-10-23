//! Complete local segment-band certificates; general CAD join surfaces are separate.
use super::{
    blocks::{EvaluatedBlock, PairedCurve, PairedPoint, Range},
    numeric::{exact, sqrt_interval},
};
use crate::{CadPreparationError as Error, GeometryFailureReason, GeometryStage};
use num_rational::BigRational as Q;
use num_traits::{Signed, Zero};

#[derive(Clone, Debug)]
pub struct PolylineBandOperands {
    pub origin: [Q; 3],
    pub x_axis: [Q; 3],
    pub y_axis: [Q; 3],
    /// Packed local XY and outgoing signed bulge, including dormant final record.
    pub vertices: Vec<[f64; 3]>,
    pub effective_widths: Vec<[f64; 2]>,
    pub closed: bool,
}
#[derive(Clone)]
enum Proof {
    Straight(Vec<PairedPoint>),
    Circular {
        centerline: Box<PairedCurve>,
        normals: Box<PairedCurve>,
        source_width: Q,
        width_difference: Q,
    },
}
/// Retains both original symbolic operands and ordered occurrence maps.
#[derive(Clone)]
pub struct PairedPolylineBand {
    index: usize,
    source: std::sync::Arc<PolylineBandOperands>,
    target: std::sync::Arc<PolylineBandOperands>,
    source_maps: Vec<EvaluatedBlock>,
    target_maps: Vec<EvaluatedBlock>,
    proof: Proof,
}
fn unavailable() -> Error {
    Error::Numerical {
        stage: GeometryStage::DeviationAssessment,
        reason: GeometryFailureReason::DeviationBoundOutOfRange,
    }
}
fn root_inverse(square: &Q) -> Result<Range, Error> {
    let (lo, hi) = sqrt_interval(square).ok_or_else(unavailable)?;
    if lo <= 0. {
        return Err(unavailable());
    }
    Ok(Range::hull(exact(1.) / exact(hi), exact(1.) / exact(lo)))
}
fn normal_pair(a: [Q; 3], asq: &Q, b: [Q; 3], bsq: &Q) -> Result<PairedPoint, Error> {
    let ar = root_inverse(asq)?;
    let br = root_inverse(bsq)?;
    let target = std::array::from_fn(|i| br.scale(&b[i]));
    let residual = if asq == bsq {
        std::array::from_fn(|i| ar.scale(&(&a[i] - &b[i])))
    } else {
        std::array::from_fn(|i| ar.scale(&a[i]).add(&br.scale(&-&b[i])))
    };
    Ok(PairedPoint::from_residual_ranges(target, residual))
}
impl PolylineBandOperands {
    fn point(&self, xy: &[Q; 2]) -> [Q; 3] {
        std::array::from_fn(|i| {
            &self.origin[i] + &self.x_axis[i] * &xy[0] + &self.y_axis[i] * &xy[1]
        })
    }
    fn vector(&self, xy: &[Q; 2]) -> [Q; 3] {
        std::array::from_fn(|i| &self.x_axis[i] * &xy[0] + &self.y_axis[i] * &xy[1])
    }
    fn segment(&self, index: usize) -> ([Q; 2], [Q; 2]) {
        let a = self.vertices[index];
        let b = self.vertices[(index + 1) % self.vertices.len()];
        ([exact(a[0]), exact(a[1])], [exact(b[0]), exact(b[1])])
    }
    fn arc(&self, index: usize) -> ([Q; 3], [[Q; 3]; 2], Q) {
        let (a, b) = self.segment(index);
        let bulge = exact(self.vertices[index][2]);
        let factor = (exact(1.) / &bulge - &bulge) / exact(4.);
        let center = [
            (&a[0] + &b[0]) / exact(2.) - (&b[1] - &a[1]) * &factor,
            (&a[1] + &b[1]) / exact(2.) + (&b[0] - &a[0]) * factor,
        ];
        let radial = [&a[0] - &center[0], &a[1] - &center[1]];
        let sign = if bulge.is_negative() {
            exact(-1.)
        } else {
            exact(1.)
        };
        let sine = [-&radial[1] * &sign, &radial[0] * &sign];
        let radius_squared = &radial[0] * &radial[0] + &radial[1] * &radial[1];
        (
            self.point(&center),
            [self.vector(&radial), self.vector(&sine)],
            radius_squared,
        )
    }
}
impl PairedPolylineBand {
    pub fn segment_index(&self) -> usize {
        self.index
    }
    fn new(
        a: std::sync::Arc<PolylineBandOperands>,
        b: std::sync::Arc<PolylineBandOperands>,
        index: usize,
    ) -> Result<Self, Error> {
        if a.vertices.len() != b.vertices.len() || a.closed != b.closed || index >= a.vertices.len()
        {
            return Err(Error::InvalidGeometry);
        }
        if a.vertices[index][2] != b.vertices[index][2] {
            return Err(Error::UnsupportedGeometry);
        }
        let (sa, sb) = a.segment(index);
        let (ta, tb) = b.segment(index);
        let sd = [&sb[0] - &sa[0], &sb[1] - &sa[1]];
        let td = [&tb[0] - &ta[0], &tb[1] - &ta[1]];
        if td.iter().all(Zero::is_zero) {
            return Err(Error::Numerical {
                stage: GeometryStage::TargetConstruction,
                reason: GeometryFailureReason::TargetCoordinateOutOfRange,
            });
        }
        let proof = if a.vertices[index][2] == 0. {
            let asq = &sd[0] * &sd[0] + &sd[1] * &sd[1];
            let bsq = &td[0] * &td[0] + &td[1] * &td[1];
            let n = normal_pair(
                a.vector(&[-sd[1].clone(), sd[0].clone()]),
                &asq,
                b.vector(&[-td[1].clone(), td[0].clone()]),
                &bsq,
            )?;
            let mut corners = Vec::with_capacity(4);
            // A bilinear straight strip attains its convex squared-norm maximum at a parameter-box corner.
            for (k, ap, bp) in [(0, &sa, &ta), (1, &sb, &tb)] {
                let center = PairedPoint::new(a.point(ap), b.point(bp));
                for sign in [-0.5, 0.5] {
                    corners.push(PairedPoint::offset_pair(
                        &center,
                        &n,
                        &(exact(a.effective_widths[index][k]) * exact(sign)),
                        &(exact(b.effective_widths[index][k]) * exact(sign)),
                    ));
                }
            }
            Proof::Straight(corners)
        } else {
            let (ac, aa, asq) = a.arc(index);
            let (bc, ba, bsq) = b.arc(index);
            let ranged = |v: [Q; 3]| v.map(Range::point);
            let centerline = PairedCurve::new(
                ac,
                [ranged(aa[0].clone()), ranged(aa[1].clone())],
                bc,
                [ranged(ba[0].clone()), ranged(ba[1].clone())],
            );
            let normal_sign = if a.vertices[index][2] < 0. {
                exact(1.)
            } else {
                exact(-1.)
            };
            let x = normal_pair(
                aa[0].clone().map(|v| v * &normal_sign),
                &asq,
                ba[0].clone().map(|v| v * &normal_sign),
                &bsq,
            )?;
            let y = normal_pair(
                aa[1].clone().map(|v| v * &normal_sign),
                &asq,
                ba[1].clone().map(|v| v * &normal_sign),
                &bsq,
            )?;
            let normals = PairedCurve::from_paired_coefficients(
                PairedPoint::new(
                    std::array::from_fn(|_| Q::zero()),
                    std::array::from_fn(|_| Q::zero()),
                ),
                [x, y],
            );
            let source_width =
                exact(a.effective_widths[index][0].max(a.effective_widths[index][1]));
            let width_difference = (exact(a.effective_widths[index][0])
                - exact(b.effective_widths[index][0]))
            .abs()
            .max((exact(a.effective_widths[index][1]) - exact(b.effective_widths[index][1])).abs());
            Proof::Circular {
                centerline: Box::new(centerline),
                normals: Box::new(normals),
                source_width,
                width_difference,
            }
        };
        Ok(Self {
            index,
            source: a,
            target: b,
            source_maps: vec![],
            target_maps: vec![],
            proof,
        })
    }
    pub fn apply(&mut self, source: &EvaluatedBlock, target: &EvaluatedBlock) {
        match &mut self.proof {
            Proof::Straight(corners) => {
                for p in corners {
                    p.apply(source, target)
                }
            }
            Proof::Circular {
                centerline,
                normals,
                ..
            } => {
                centerline.apply(source, target);
                normals.apply(&source.vector_map(), &target.vector_map());
            }
        }
        self.source_maps.push(source.clone());
        self.target_maps.push(target.clone());
    }
    pub fn squared_deviation(&self) -> Option<(Q, Q)> {
        match &self.proof {
            Proof::Straight(corners) => Some(
                corners
                    .iter()
                    .map(PairedPoint::squared_deviation)
                    .fold((Q::zero(), Q::zero()), |(l, u), (a, b)| {
                        (l.max(a), u.max(b))
                    }),
            ),
            Proof::Circular {
                centerline,
                normals,
                source_width,
                width_difference,
            } => {
                let c = exact(sqrt_interval(&centerline.squared_deviation()?.1)?.1);
                let n = exact(sqrt_interval(&normals.squared_deviation()?.1)?.1);
                let w = if width_difference.is_zero() {
                    Q::zero()
                } else {
                    width_difference
                        * exact(sqrt_interval(&normals.target_coefficient_upper_squared())?.1)
                        / exact(2.)
                };
                let upper = c + source_width * n / exact(2.) + w;
                Some((Q::zero(), &upper * &upper))
            }
        }
    }
    fn combine(a: &Self, b: &Self, target_side: bool) -> Result<Self, Error> {
        if a.index != b.index {
            return Err(Error::InvalidGeometry);
        }
        let mut pair = Self::new(
            a.source.clone(),
            if target_side {
                b.target.clone()
            } else {
                b.source.clone()
            },
            a.index,
        )?;
        let right = if target_side {
            &b.target_maps
        } else {
            &b.source_maps
        };
        let identity = EvaluatedBlock::identity();
        for i in 0..a.source_maps.len().max(right.len()) {
            pair.apply(
                a.source_maps.get(i).unwrap_or(&identity),
                right.get(i).unwrap_or(&identity),
            );
        }
        Ok(pair)
    }
    pub(crate) fn from_prepared_source_and_target(a: &Self, b: &Self) -> Result<Self, Error> {
        Self::combine(a, b, true)
    }
    pub(crate) fn from_prepared_sources(a: &Self, b: &Self) -> Result<Self, Error> {
        Self::combine(a, b, false)
    }
}
pub fn pair_polyline_bands(
    source: &PolylineBandOperands,
    target: &PolylineBandOperands,
) -> Result<Vec<PairedPolylineBand>, Error> {
    let n = source.vertices.len();
    if n < 2 || target.vertices.len() != n || source.closed != target.closed {
        return Err(Error::InvalidGeometry);
    }
    for side in [source, target] {
        if side.effective_widths.len() != n
            || side.vertices.iter().flatten().any(|v| !v.is_finite())
            || side
                .effective_widths
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || *v < 0.)
        {
            return Err(Error::InvalidGeometry);
        }
    }
    let count = if source.closed { n } else { n - 1 };
    let mut bands = Vec::new();
    let source = std::sync::Arc::new(source.clone());
    let target = std::sync::Arc::new(target.clone());
    for i in 0..count {
        if source.vertices[i][2] != target.vertices[i][2] {
            return Err(Error::UnsupportedGeometry);
        }
        if source.effective_widths[i]
            .iter()
            .chain(&target.effective_widths[i])
            .all(|v| *v == 0.)
        {
            continue;
        }
        let (a, b) = source.segment(i);
        if a == b {
            continue;
        }
        bands.push(PairedPolylineBand::new(source.clone(), target.clone(), i)?);
    }
    Ok(bands)
}

pub(crate) fn native_operands(
    placement: ifccad::geometry_kernel::CoordinateFrame3,
    vertices: &[[f64; 3]],
    widths: &ifccad::geometry_kernel::PlanarPolylineWidths,
    closed: bool,
) -> Result<PolylineBandOperands, Error> {
    Ok(PolylineBandOperands {
        origin: placement.origin().components().map(exact),
        x_axis: placement.x_axis().components().map(exact),
        y_axis: placement.y_axis().components().map(exact),
        vertices: vertices.to_vec(),
        effective_widths: widths
            .effective_widths(vertices.len())
            .map_err(|_| Error::InvalidGeometry)?,
        closed,
    })
}
/// Interpret supported authored CAD width controls without inferring absence from equality.
pub fn from_cad_polyline_widths(
    entity: &opencadcodec::EntityType,
) -> Result<ifccad::geometry_kernel::PlanarPolylineWidths, Error> {
    use ifccad::geometry_kernel::{PlanarPolylineWidths as W, PolylineVertexWidthOverride as O};
    let (widths, n) = match entity {
        opencadcodec::EntityType::Polyline2D(p) => (
            W::Defaulted {
                default_start_width: p.start_width,
                default_end_width: p.end_width,
                overrides: p
                    .vertices
                    .iter()
                    .map(|v| O {
                        start: v.start_width,
                        end: v.end_width,
                    })
                    .collect(),
            },
            p.vertices.len(),
        ),
        opencadcodec::EntityType::LwPolyline(p) => {
            if !p.constant_width.is_finite()
                || p.constant_width < 0.
                || p.vertices.iter().any(|v| {
                    !v.start_width.is_finite()
                        || !v.end_width.is_finite()
                        || v.start_width < 0.
                        || v.end_width < 0.
                })
            {
                return Err(Error::InvalidGeometry);
            }
            let pairs = p
                .vertices
                .iter()
                .map(|v| [v.start_width, v.end_width])
                .collect::<Vec<_>>();
            let variable = pairs.iter().flatten().any(|v| v.to_bits() != 0);
            let widths = if p.constant_width.to_bits() != 0 {
                if variable {
                    return Err(Error::UnsupportedGeometry);
                }
                W::Constant {
                    width: p.constant_width,
                }
            } else if variable {
                W::PerVertex { widths: pairs }
            } else {
                W::default()
            };
            (widths, p.vertices.len())
        }
        _ => return Err(Error::UnsupportedGeometry),
    };
    widths.validate(n).map_err(|_| Error::InvalidGeometry)?;
    Ok(widths)
}
pub(crate) fn cad_operands(p: &opencadcodec::LwPolyline) -> Result<PolylineBandOperands, Error> {
    let m = opencadcodec::types::Matrix3::arbitrary_axis(p.normal).m;
    if !m.iter().flatten().all(|v| v.is_finite()) || !p.elevation.is_finite() {
        return Err(Error::InvalidGeometry);
    }
    let widths = from_cad_polyline_widths(&opencadcodec::EntityType::LwPolyline(p.clone()))?;
    Ok(PolylineBandOperands {
        origin: std::array::from_fn(|i| exact(m[i][2]) * exact(p.elevation)),
        x_axis: std::array::from_fn(|i| exact(m[i][0])),
        y_axis: std::array::from_fn(|i| exact(m[i][1])),
        vertices: p
            .vertices
            .iter()
            .map(|v| [v.location.x, v.location.y, v.bulge])
            .collect(),
        effective_widths: widths
            .effective_widths(p.vertices.len())
            .map_err(|_| Error::InvalidGeometry)?,
        closed: p.is_closed,
    })
}
pub(crate) fn apply_lw_widths(
    widths: &ifccad::geometry_kernel::PlanarPolylineWidths,
    p: &mut opencadcodec::LwPolyline,
) -> Result<(), Error> {
    use ifccad::geometry_kernel::PlanarPolylineWidths as W;
    let pairs = widths
        .effective_widths(p.vertices.len())
        .map_err(|_| Error::InvalidGeometry)?;
    p.constant_width = match widths {
        W::Constant { width } => *width,
        _ => 0.,
    };
    for (v, pair) in p.vertices.iter_mut().zip(pairs) {
        [v.start_width, v.end_width] = if matches!(widths, W::Constant { .. }) {
            [0.; 2]
        } else {
            pair
        };
    }
    Ok(())
}
pub(crate) fn entity_for_widths(
    widths: &ifccad::geometry_kernel::PlanarPolylineWidths,
    p: opencadcodec::LwPolyline,
) -> Result<opencadcodec::EntityType, Error> {
    use ifccad::geometry_kernel::PlanarPolylineWidths as W;
    use opencadcodec::entities::{Polyline2D, PolylineFlags, Vertex2D};
    if let W::Defaulted {
        default_start_width,
        default_end_width,
        overrides,
    } = widths
    {
        let mut out = Polyline2D::new();
        out.common = p.common;
        out.normal = p.normal;
        out.elevation = p.elevation;
        out.thickness = p.thickness;
        out.flags =
            PolylineFlags::from_bits(u16::from(p.is_closed) | if p.plinegen { 128 } else { 0 });
        out.start_width = *default_start_width;
        out.end_width = *default_end_width;
        out.vertices = p
            .vertices
            .iter()
            .zip(overrides)
            .map(|(v, w)| {
                let mut out =
                    Vertex2D::new(opencadcodec::Vector3::new(v.location.x, v.location.y, 0.))
                        .with_bulge(v.bulge);
                out.start_width = w.start;
                out.end_width = w.end;
                out
            })
            .collect();
        Ok(opencadcodec::EntityType::Polyline2D(out))
    } else {
        Ok(opencadcodec::EntityType::LwPolyline(p))
    }
}
