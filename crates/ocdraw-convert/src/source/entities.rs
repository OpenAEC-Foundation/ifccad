use super::CadToOcdrawLossReason;
use opencadcodec::entities::EntityCommon;
use opencadcodec::{Arc, Circle, Ellipse, Line, LwPolyline, Point, Vector3};

pub(crate) fn common_semantic_losses(common: &EntityCommon) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();

    if !common.extended_data.is_empty() {
        reasons.push(CadToOcdrawLossReason::EntityExtendedData);
    }
    if common.graphic_data.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityGraphicData);
    }
    if !common.reactors.is_empty() {
        reasons.push(CadToOcdrawLossReason::EntityReactors);
    }
    if common.xdictionary_handle.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityExtensionDictionary);
    }
    if common.color_book_handle.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityColorBookReference);
    }
    if common.full_visual_style_handle.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityFullVisualStyle);
    }
    if common.face_visual_style_handle.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityFaceVisualStyle);
    }
    if common.edge_visual_style_handle.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityEdgeVisualStyle);
    }
    if common.material_flags != 0 || common.material_handle.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityMaterial);
    }
    if common.shadow_flags != 0 {
        reasons.push(CadToOcdrawLossReason::EntityShadowFlags);
    }
    if common.plotstyle_flags != 0 || common.plotstyle_handle.is_some() {
        reasons.push(CadToOcdrawLossReason::EntityPlotStyle);
    }
    reasons
}

pub(crate) fn line_losses(line: &Line) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();
    if ![
        line.start.x,
        line.start.y,
        line.start.z,
        line.end.x,
        line.end.y,
        line.end.z,
        line.thickness,
        line.normal.x,
        line.normal.y,
        line.normal.z,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if line.thickness != 0.0 {
        reasons.push(CadToOcdrawLossReason::NonZeroThickness);
    }
    reasons
}

pub(crate) fn point_losses(point: &Point) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();
    if ![
        point.location.x,
        point.location.y,
        point.location.z,
        point.normal.x,
        point.normal.y,
        point.normal.z,
        point.thickness,
        point.x_axis_angle,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if point.thickness != 0.0 {
        reasons.push(CadToOcdrawLossReason::NonZeroThickness);
    }
    if cad_geometry_convert::geometry::cad_plane(point.normal).is_none() {
        reasons.push(CadToOcdrawLossReason::UnsupportedNormal);
    }
    reasons
}

pub(crate) fn circle_losses(circle: &Circle) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();
    if ![
        circle.center.x,
        circle.center.y,
        circle.center.z,
        circle.normal.x,
        circle.normal.y,
        circle.normal.z,
        circle.radius,
        circle.thickness,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if circle.thickness != 0.0 {
        reasons.push(CadToOcdrawLossReason::NonZeroThickness);
    }
    if circle.radius <= 0.0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "circle radius".into(),
        });
    }
    if cad_geometry_convert::geometry::circular::from_cad_ocs(circle.center, circle.normal)
        .is_none()
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedNormal);
    }
    reasons
}

pub(crate) fn arc_losses(arc: &Arc) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();
    if ![
        arc.center.x,
        arc.center.y,
        arc.center.z,
        arc.normal.x,
        arc.normal.y,
        arc.normal.z,
        arc.radius,
        arc.thickness,
        arc.start_angle,
        arc.end_angle,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if arc.thickness != 0.0 {
        reasons.push(CadToOcdrawLossReason::NonZeroThickness);
    }
    if arc.radius <= 0.0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "arc radius".into(),
        });
    }
    let difference = arc.end_angle - arc.start_angle;
    let sweep = difference.rem_euclid(std::f64::consts::TAU);
    if !difference.is_finite()
        || difference.abs() >= std::f64::consts::TAU
        || sweep == 0.0
        || sweep >= std::f64::consts::TAU
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "arc sweep".into(),
        });
    }
    if cad_geometry_convert::geometry::circular::from_cad_ocs(arc.center, arc.normal).is_none() {
        reasons.push(CadToOcdrawLossReason::UnsupportedNormal);
    }
    reasons
}

pub(crate) fn ellipse_losses(ellipse: &Ellipse) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();
    if ![
        ellipse.center.x,
        ellipse.center.y,
        ellipse.center.z,
        ellipse.major_axis.x,
        ellipse.major_axis.y,
        ellipse.major_axis.z,
        ellipse.minor_axis_ratio,
        ellipse.start_parameter,
        ellipse.end_parameter,
        ellipse.normal.x,
        ellipse.normal.y,
        ellipse.normal.z,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if cad_geometry_convert::geometry::circular::from_cad_ellipse(ellipse).is_none() {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "ellipse frame or axis ratio".into(),
        });
    }
    let difference = ellipse.end_parameter - ellipse.start_parameter;
    let sweep = difference.rem_euclid(std::f64::consts::TAU);
    if !difference.is_finite()
        || difference.abs() > std::f64::consts::TAU
        || (difference != std::f64::consts::TAU && sweep == 0.0)
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "ellipse parameter sweep".into(),
        });
    }
    reasons
}

pub(crate) fn polyline_losses(polyline: &LwPolyline) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();
    let finite = [
        polyline.constant_width,
        polyline.elevation,
        polyline.thickness,
        polyline.normal.x,
        polyline.normal.y,
        polyline.normal.z,
    ]
    .into_iter()
    .chain(polyline.vertices.iter().flat_map(|vertex| {
        [
            vertex.location.x,
            vertex.location.y,
            vertex.bulge,
            vertex.start_width,
            vertex.end_width,
        ]
    }))
    .all(f64::is_finite);
    if !finite {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if polyline.vertices.len() < 2 {
        reasons.push(CadToOcdrawLossReason::PolylineTooFewVertices {
            count: polyline.vertices.len(),
        });
    }
    if polyline.thickness != 0.0 {
        reasons.push(CadToOcdrawLossReason::NonZeroThickness);
    }
    if polyline.normal.x == 0.0 && polyline.normal.y == 0.0 && polyline.normal.z == 0.0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedNormal);
    }
    let count = if polyline.is_closed {
        polyline.vertices.len()
    } else {
        polyline.vertices.len().saturating_sub(1)
    };
    for index in 0..count {
        let start = &polyline.vertices[index];
        let end = &polyline.vertices[(index + 1) % polyline.vertices.len()];
        if start.bulge != 0.0 && start.location == end.location {
            reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
                name: "bulged zero-length polyline segment".into(),
            });
            break;
        }
    }

    reasons
}

fn lw_from_polyline2d(polyline: &opencadcodec::entities::Polyline2D) -> LwPolyline {
    let mut lw = LwPolyline::from_points(
        polyline
            .vertices
            .iter()
            .map(|vertex| opencadcodec::Vector2::new(vertex.location.x, vertex.location.y))
            .collect(),
    );
    lw.is_closed = polyline.flags.is_closed();
    lw.elevation = polyline.elevation;
    lw.normal = polyline.normal;
    for (target, source) in lw.vertices.iter_mut().zip(&polyline.vertices) {
        target.bulge = source.bulge;
    }
    lw
}

pub(crate) fn polyline2d_losses(
    polyline: &opencadcodec::entities::Polyline2D,
) -> Vec<CadToOcdrawLossReason> {
    let lw = lw_from_polyline2d(polyline);
    let mut reasons = polyline_losses(&lw);
    if ![
        polyline.start_width,
        polyline.end_width,
        polyline.thickness,
        polyline.elevation,
    ]
    .into_iter()
    .all(f64::is_finite)
        || polyline.vertices.iter().any(|vertex| {
            ![
                vertex.location.x,
                vertex.location.y,
                vertex.location.z,
                vertex.start_width,
                vertex.end_width,
                vertex.bulge,
                vertex.curve_tangent,
            ]
            .into_iter()
            .all(f64::is_finite)
        })
    {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if polyline.thickness != 0.0 {
        reasons.push(CadToOcdrawLossReason::NonZeroThickness);
    }
    if polyline.flags.bits() & (2 | 4) != 0
        || polyline.smooth_surface != opencadcodec::entities::SmoothSurfaceType::None
        || polyline.vertices.iter().any(|vertex| {
            vertex.flags.bits() & (1 | 2 | 8 | 16) != 0 || vertex.curve_tangent != 0.0
        })
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "polyline fit curve".into(),
        });
    }
    if polyline.flags.bits() & (8 | 16 | 32 | 64) != 0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "polyline mesh or 3D flags".into(),
        });
    }

    if polyline.flags.bits() & !0xff != 0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "polyline flags".into(),
        });
    }
    if polyline
        .vertices
        .iter()
        .any(|vertex| vertex.flags.bits() & !(1 | 2 | 8 | 16) != 0)
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "2D polyline vertex flags".into(),
        });
    }
    if polyline
        .vertices
        .iter()
        .any(|vertex| vertex.location.z != 0.0)
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "2D polyline vertex elevation".into(),
        });
    }
    reasons
}

fn spatial_polyline_losses(
    points: impl IntoIterator<Item = Vector3>,
    flags: u16,
) -> Vec<CadToOcdrawLossReason> {
    let points = points.into_iter().collect::<Vec<_>>();
    let mut reasons = Vec::new();
    if points.len() < 2 {
        reasons.push(CadToOcdrawLossReason::PolylineTooFewVertices {
            count: points.len(),
        });
    }
    if points
        .iter()
        .any(|point| ![point.x, point.y, point.z].into_iter().all(f64::is_finite))
    {
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    if flags & (2 | 4) != 0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "polyline fit curve".into(),
        });
    }
    if flags & (16 | 32 | 64) != 0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "polyline mesh".into(),
        });
    }
    if flags & !(1 | 8 | 128) != 0 && flags & !(1 | 8 | 2 | 4 | 16 | 32 | 64 | 128) != 0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "polyline flags".into(),
        });
    }
    reasons
}

pub(crate) fn generic_polyline_losses(
    polyline: &opencadcodec::entities::Polyline,
) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = spatial_polyline_losses(
        polyline.vertices.iter().map(|vertex| vertex.location),
        polyline.flags.bits(),
    );
    if polyline
        .vertices
        .iter()
        .any(|vertex| vertex.flags.bits() != 0)
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "3D polyline vertex flags".into(),
        });
    }
    reasons
}

pub(crate) fn polyline3d_losses(
    polyline: &opencadcodec::entities::Polyline3D,
) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = spatial_polyline_losses(
        polyline.vertices.iter().map(|vertex| vertex.position),
        polyline.flags.to_bits() as u16,
    );
    if polyline.default_start_width != 0.0 || polyline.default_end_width != 0.0 {
        reasons.push(CadToOcdrawLossReason::PolylineWidth);
    }
    if !polyline.flags.is_3d
        || polyline.elevation != 0.0
        || polyline.normal != Vector3::UNIT_Z
        || polyline.mesh_m_count != 0
        || polyline.mesh_n_count != 0
        || polyline.smooth_m_density != 0
        || polyline.smooth_n_density != 0
        || polyline.smooth_type != opencadcodec::entities::polyline3d::SmoothSurfaceType::None
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "3D polyline source properties".into(),
        });
    }
    if polyline
        .vertices
        .iter()
        // Readers allocate ordinary VERTEX handles; those record identities do
        // not change the spatial path or introduce an unsupported property.
        .any(|vertex| vertex.flags != 32 || (!vertex.layer.is_empty() && vertex.layer != "0"))
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "3D polyline vertex properties".into(),
        });
    }
    reasons
}
