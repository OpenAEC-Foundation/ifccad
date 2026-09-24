use super::appearance::EntityAppearanceError;
use super::conversion::ExportContext;
use super::structure::ModelSpaceInfo;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason,
    SourceStructureProblem,
};
use cadcodec::entities::EntityCommon;
use cadcodec::{
    Arc, CadDocument, Circle, Ellipse, EntityType, Handle, Line, LwPolyline, Point, Vector3,
};
use ifccad::ifcdr::Point2;
use ifccad::package::{
    ArcDefinition, CircleDefinition, DrawingBuilder, EllipseArcDefinition, EllipseDefinition,
    LineDefinition, PlanarPolylineDefinition, PointDefinition, SpatialPolylineDefinition,
    ViewportDefinition, ViewportLayerOverrideDefinition,
};

pub(crate) fn add_entities(
    document: &CadDocument,
    model_space: &ModelSpaceInfo<'_>,
    drawing: &mut DrawingBuilder<'_>,
    context: &mut ExportContext,
) -> Result<Vec<SourceStructureProblem>, super::ExportError> {
    let mut structural_problems = Vec::new();
    for source in super::blocks::ordered_entities(document) {
        let common = source.common();
        if matches!(source, EntityType::Viewport(viewport) if viewport.id == 1 && document.objects.values().any(|object| matches!(object, cadcodec::objects::ObjectType::Layout(layout) if layout.viewport == common.handle)))
        {
            continue;
        }
        if document.block_records.iter().any(|record| {
            record.handle == common.owner_handle
                && match source {
                    EntityType::Block(_) => common.handle == record.block_entity_handle,
                    EntityType::BlockEnd(_) => common.handle == record.block_end_handle,
                    _ => false,
                }
        }) {
            continue;
        }
        let mut common_losses = common_semantic_losses(common);
        if let EntityType::LwPolyline(polyline) = source {
            if polyline.constant_width != 0.0
                || polyline
                    .vertices
                    .iter()
                    .any(|vertex| vertex.start_width != 0.0 || vertex.end_width != 0.0)
            {
                common_losses.push(ExportLossReason::PolylineWidth);
            }
        }
        if let EntityType::Polyline2D(polyline) = source {
            if polyline.start_width != 0.0
                || polyline.end_width != 0.0
                || polyline
                    .vertices
                    .iter()
                    .any(|vertex| vertex.start_width != 0.0 || vertex.end_width != 0.0)
            {
                common_losses.push(ExportLossReason::PolylineWidth);
            }
            let count = polyline
                .vertices
                .iter()
                .filter(|vertex| vertex.id != 0)
                .count();
            if count > 0 {
                common_losses.push(ExportLossReason::PolylineVertexIdentifiers { count });
            }
        }
        if !classify_owner(
            document,
            model_space,
            source,
            common,
            context,
            &mut structural_problems,
            &common_losses,
        ) {
            continue;
        }

        let mut geometry_losses = match source {
            EntityType::Insert(insert)
                if !insert.is_minsert()
                    && insert.row_count == 1
                    && insert.column_count == 1
                    && insert.row_spacing == 0.
                    && insert.column_spacing == 0.
                    && insert.attributes.is_empty()
                    && insert.view_rep_handle.is_none()
                    && document
                        .block_records
                        .get(&insert.block_name)
                        .is_some_and(|record| context.blocks.contains_key(&record.handle)) =>
            {
                Vec::new()
            }
            EntityType::Line(line) => line_losses(line),
            EntityType::Point(point) => point_losses(point),
            EntityType::Circle(circle) => circle_losses(circle),
            EntityType::Arc(arc) => arc_losses(arc),
            EntityType::Ellipse(ellipse) => ellipse_losses(ellipse),
            EntityType::Viewport(viewport)
                if context.paper_scopes.contains_key(&common.owner_handle) =>
            {
                viewport_losses(viewport, document, context)
            }
            EntityType::LwPolyline(polyline) => polyline_losses(polyline),
            EntityType::Polyline2D(polyline) => polyline2d_losses(polyline),
            EntityType::Polyline(polyline) => generic_polyline_losses(polyline),
            EntityType::Polyline3D(polyline) => polyline3d_losses(polyline),
            _ => vec![ExportLossReason::UnsupportedEntityType {
                kind: source.as_entity().entity_type().to_owned(),
            }],
        };
        if !geometry_losses.is_empty() {
            geometry_losses.extend(common_losses);
            record_skipped(source, geometry_losses, context);
            continue;
        }

        let Some(&layer) = context.layer_keys.get(&common.layer.to_lowercase()) else {
            let mut reasons = vec![ExportLossReason::MissingEntityLayer {
                name: common.layer.clone(),
            }];
            reasons.extend(common_losses);
            record_skipped(source, reasons, context);
            continue;
        };
        let appearance = match context.appearances.add_entity_appearance(drawing, common) {
            Ok(appearance) => appearance,
            Err(EntityAppearanceError::Loss(reasons)) => {
                let mut reasons = reasons;
                reasons.extend(common_losses);
                record_skipped(source, reasons, context);
                continue;
            }
            Err(EntityAppearanceError::Build(error)) => return Err(error.into()),
        };

        let mut target = if let Some(&paper) = context.paper_scopes.get(&common.owner_handle) {
            drawing.paper_space(paper)?
        } else if let Some(&definition) = context.blocks.get(&common.owner_handle) {
            drawing.block_definition(definition)?
        } else {
            drawing.model_space()
        };
        let entity_id = match source {
            EntityType::Insert(insert) => {
                let record = document
                    .block_records
                    .get(&insert.block_name)
                    .expect("checked definition");
                let (transform, source_map, target_map) =
                    crate::geometry::blocks::from_cad_instance(
                        insert,
                        record.base_point,
                        context.geometry.as_ref().unwrap(),
                    )?;
                if crate::geometry::stored_normal(transform.placement()) != Some(insert.normal) {
                    common_losses.push(ExportLossReason::SourceNormalNormalized);
                }
                let id = target.add_block_instance(ifccad::package::BlockInstanceDefinition {
                    definition: context.blocks[&record.handle],
                    transform,
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?;
                context.block_instances.insert(
                    common.handle,
                    super::blocks::ConvertedInstance {
                        definition: record.handle,
                        source: source_map,
                        target: target_map,
                    },
                );
                id
            }
            EntityType::Point(point) => {
                let basis = crate::geometry::cad_plane(point.normal)
                    .expect("point normal classified before conversion");
                let (sine, cosine) = point.x_axis_angle.sin_cos();
                let rotated = |u: [f64; 3], v: [f64; 3], a: f64, b: f64| {
                    cadcodec::Vector3::new(
                        a * u[0] + b * v[0],
                        a * u[1] + b * v[1],
                        a * u[2] + b * v[2],
                    )
                };
                let (x, y) = crate::geometry::orthonormal_pair(
                    rotated(basis.u, basis.v, cosine, sine),
                    rotated(basis.u, basis.v, -sine, cosine),
                )
                .expect("finite independent CAD point axes classified before conversion");
                let placement = ifccad::ifcdr::PlanePlacement::try_new(
                    ifccad::ifcdr::Point3::new(
                        point.location.x,
                        point.location.y,
                        point.location.z,
                    ),
                    ifccad::ifcdr::Vector3::new(x.x, x.y, x.z),
                    ifccad::ifcdr::Vector3::new(y.x, y.y, y.z),
                )
                .expect("finite CAD point frame classified before conversion");
                context.block_points.insert(
                    common.handle,
                    vec![crate::geometry::blocks::PairedPoint::exact([
                        point.location.x,
                        point.location.y,
                        point.location.z,
                    ])],
                );
                context.geometry.as_mut().unwrap().record(
                    crate::ConversionEntitySource::CadEntity {
                        handle: common.handle,
                        kind: "POINT".into(),
                    },
                    1,
                    0.0,
                );
                if crate::geometry::stored_normal(placement) != Some(point.normal) {
                    common_losses.push(ExportLossReason::SourceNormalNormalized);
                }
                target.add_point(PointDefinition {
                    placement,
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            EntityType::Circle(circle) => {
                let placement =
                    crate::geometry::circular::from_cad_ocs(circle.center, circle.normal)
                        .expect("classified CAD circle frame");
                let identity = crate::ConversionEntitySource::CadEntity {
                    handle: common.handle,
                    kind: "CIRCLE".into(),
                };
                let geometry = context.geometry.as_mut().unwrap();
                let samples =
                    crate::geometry::circular::export_circle_sample_pairs(circle, placement)
                        .ok_or_else(|| {
                            geometry.failure(
                                &identity,
                                None,
                                crate::ConversionGeometryStage::TargetConstruction,
                                crate::ConversionGeometryFailureReason::TargetCoordinateOutOfRange,
                            )
                        })?;
                let mut maximum = 0.0_f64;
                let points = samples
                    .into_iter()
                    .enumerate()
                    .map(|(index, (point, squared))| {
                        maximum = maximum.max(geometry.check(&identity, index, &squared)?);
                        Ok(point)
                    })
                    .collect::<Result<Vec<_>, Box<crate::ConversionGeometryFailure>>>()?;
                geometry.record(identity, points.len(), maximum);
                context.block_points.insert(common.handle, points);
                if maximum > 0.0 {
                    common_losses.push(ExportLossReason::GeometryRoundedWithinTolerance {
                        max_deviation_upper_bound: maximum,
                    });
                }
                if crate::geometry::stored_normal(placement) != Some(circle.normal) {
                    common_losses.push(ExportLossReason::SourceNormalNormalized);
                }
                target.add_circle(CircleDefinition {
                    placement,
                    radius: circle.radius,
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            EntityType::Arc(arc) => {
                let placement = crate::geometry::circular::from_cad_ocs(arc.center, arc.normal)
                    .expect("classified CAD arc frame");
                let sweep = (arc.end_angle - arc.start_angle).rem_euclid(std::f64::consts::TAU);
                let identity = crate::ConversionEntitySource::CadEntity {
                    handle: common.handle,
                    kind: "ARC".into(),
                };
                let geometry = context.geometry.as_mut().unwrap();
                let samples =
                    crate::geometry::circular::export_arc_sample_pairs(arc, placement, sweep)
                        .ok_or_else(|| {
                            geometry.failure(
                                &identity,
                                None,
                                crate::ConversionGeometryStage::TargetConstruction,
                                crate::ConversionGeometryFailureReason::TargetCoordinateOutOfRange,
                            )
                        })?;
                let mut maximum = 0.0_f64;
                let points = samples
                    .into_iter()
                    .enumerate()
                    .map(|(index, (point, squared))| {
                        maximum = maximum.max(geometry.check(&identity, index, &squared)?);
                        Ok(point)
                    })
                    .collect::<Result<Vec<_>, Box<crate::ConversionGeometryFailure>>>()?;
                geometry.record(identity, points.len(), maximum);
                context.block_points.insert(common.handle, points);
                if maximum > 0.0 {
                    common_losses.push(ExportLossReason::GeometryRoundedWithinTolerance {
                        max_deviation_upper_bound: maximum,
                    });
                }
                if crate::geometry::stored_normal(placement) != Some(arc.normal) {
                    common_losses.push(ExportLossReason::SourceNormalNormalized);
                }
                target.add_arc(ArcDefinition {
                    placement,
                    radius: arc.radius,
                    start_parameter: arc.start_angle,
                    sweep_parameter: sweep,
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            EntityType::Ellipse(ellipse) => {
                let (placement, major, minor) =
                    crate::geometry::circular::from_cad_ellipse(ellipse)
                        .expect("classified CAD ellipse frame");
                let difference = ellipse.end_parameter - ellipse.start_parameter;
                let full = difference == std::f64::consts::TAU;
                let sweep = if full {
                    difference
                } else {
                    difference.rem_euclid(std::f64::consts::TAU)
                };
                let identity = crate::ConversionEntitySource::CadEntity {
                    handle: common.handle,
                    kind: "ELLIPSE".into(),
                };
                let geometry = context.geometry.as_mut().unwrap();
                let samples = crate::geometry::circular::export_ellipse_sample_pairs(
                    ellipse,
                    placement,
                    major,
                    minor,
                    ellipse.start_parameter,
                    sweep,
                )
                .ok_or_else(|| {
                    geometry.failure(
                        &identity,
                        None,
                        crate::ConversionGeometryStage::TargetConstruction,
                        crate::ConversionGeometryFailureReason::TargetCoordinateOutOfRange,
                    )
                })?;
                let mut maximum = 0.0_f64;
                let points = samples
                    .into_iter()
                    .enumerate()
                    .map(|(index, (point, squared))| {
                        maximum = maximum.max(geometry.check(&identity, index, &squared)?);
                        Ok(point)
                    })
                    .collect::<Result<Vec<_>, Box<crate::ConversionGeometryFailure>>>()?;
                geometry.record(identity, points.len(), maximum);
                context.block_points.insert(common.handle, points);
                if maximum > 0.0 {
                    common_losses.push(ExportLossReason::GeometryRoundedWithinTolerance {
                        max_deviation_upper_bound: maximum,
                    });
                }
                if crate::geometry::stored_normal(placement) != Some(ellipse.normal) {
                    common_losses.push(ExportLossReason::SourceNormalNormalized);
                }
                if full {
                    target.add_ellipse(EllipseDefinition {
                        placement,
                        semi_major_radius: major,
                        semi_minor_radius: minor,
                        layer,
                        appearance,
                        visible: !common.invisible,
                    })?
                } else {
                    target.add_ellipse_arc(EllipseArcDefinition {
                        placement,
                        semi_major_radius: major,
                        semi_minor_radius: minor,
                        start_parameter: ellipse.start_parameter,
                        sweep_parameter: sweep,
                        layer,
                        appearance,
                        visible: !common.invisible,
                    })?
                }
            }
            EntityType::Line(line) => target.add_line(LineDefinition {
                start: ifccad::ifcdr::Point3::new(line.start.x, line.start.y, line.start.z),
                end: ifccad::ifcdr::Point3::new(line.end.x, line.end.y, line.end.z),
                layer,
                appearance,
                visible: !common.invisible,
            })?,
            EntityType::Viewport(viewport) => {
                let mut overrides = Vec::new();
                for handle in &viewport.frozen_layers {
                    let Some(source_layer) =
                        document.layers.iter().find(|layer| layer.handle == *handle)
                    else {
                        common_losses.push(ExportLossReason::MissingTarget {
                            kind: "viewport frozen layer".into(),
                            identifier: format!("{handle:?}"),
                        });
                        continue;
                    };
                    if let Some(&layer) = context.layer_keys.get(&source_layer.name.to_lowercase())
                    {
                        overrides.push(ViewportLayerOverrideDefinition {
                            layer,
                            frozen: true,
                            appearance: None,
                        });
                    }
                }
                let render_mode = match viewport.render_mode {
                    cadcodec::entities::ViewportRenderMode::Wireframe2D => {
                        ifccad::ifcdr::ViewportRenderMode::TwoDimensional
                    }
                    cadcodec::entities::ViewportRenderMode::Wireframe3D => {
                        ifccad::ifcdr::ViewportRenderMode::Wireframe
                    }
                    cadcodec::entities::ViewportRenderMode::HiddenLine => {
                        ifccad::ifcdr::ViewportRenderMode::HiddenLine
                    }
                    cadcodec::entities::ViewportRenderMode::FlatShaded => {
                        ifccad::ifcdr::ViewportRenderMode::FlatShadedWithoutEdges
                    }
                    cadcodec::entities::ViewportRenderMode::FlatShadedWithEdges => {
                        ifccad::ifcdr::ViewportRenderMode::FlatShadedWithEdges
                    }
                    cadcodec::entities::ViewportRenderMode::GouraudShaded => {
                        ifccad::ifcdr::ViewportRenderMode::SmoothShadedWithoutEdges
                    }
                    cadcodec::entities::ViewportRenderMode::GouraudShadedWithEdges => {
                        ifccad::ifcdr::ViewportRenderMode::SmoothShadedWithEdges
                    }
                };
                target.add_viewport(ViewportDefinition {
                    frame: ifccad::ifcdr::ViewportFrame {
                        center: Point2::new(viewport.center.x, viewport.center.y),
                        width: viewport.width,
                        height: viewport.height,
                    },
                    view: ifccad::ifcdr::ViewDefinition {
                        center: Point2::new(viewport.view_center.x, viewport.view_center.y),
                        target: ifccad::ifcdr::Point3::new(
                            viewport.view_target.x,
                            viewport.view_target.y,
                            viewport.view_target.z,
                        ),
                        direction: ifccad::ifcdr::Vector3::new(
                            viewport.view_direction.x,
                            viewport.view_direction.y,
                            viewport.view_direction.z,
                        ),
                        height: viewport.view_height,
                        twist: viewport.twist_angle,
                        projection: ifccad::ifcdr::ProjectionMode::Orthographic,
                        lens_length: Some(viewport.lens_length),
                        front_clip: ifccad::ifcdr::FrontClip {
                            mode: if viewport.status.front_clipping {
                                if viewport.status.front_clip_not_at_eye {
                                    ifccad::ifcdr::FrontClipMode::AtDistance
                                } else {
                                    ifccad::ifcdr::FrontClipMode::AtCamera
                                }
                            } else {
                                ifccad::ifcdr::FrontClipMode::Disabled
                            },
                            distance: Some(viewport.front_clip_z),
                        },
                        back_clip: ifccad::ifcdr::BackClip {
                            mode: if viewport.status.back_clipping {
                                ifccad::ifcdr::BackClipMode::AtDistance
                            } else {
                                ifccad::ifcdr::BackClipMode::Disabled
                            },
                            distance: Some(viewport.back_clip_z),
                        },
                    },
                    render_mode,
                    view_enabled: viewport.status.is_on,
                    view_locked: viewport.status.locked,
                    paper_clip: ifccad::ifcdr::PaperClip {
                        enabled: viewport.clip_boundary_handle != Handle::NULL,
                        boundary_entity_id: context
                            .entity_mapping
                            .target_entity_id(viewport.clip_boundary_handle)
                            .map(ifccad::ifcdr::EntityId::get),
                    },
                    plot_shading_override: None,
                    layer,
                    appearance,
                    visible: !common.invisible,
                    layer_overrides: overrides,
                })?
            }
            EntityType::LwPolyline(polyline) => {
                let (placement, bound, normal_changed) =
                    crate::geometry::from_cad(polyline, context.geometry.as_mut().unwrap())?;
                if bound > 0.0 {
                    common_losses.push(ExportLossReason::GeometryRoundedWithinTolerance {
                        max_deviation_upper_bound: bound,
                    });
                }
                if normal_changed {
                    common_losses.push(ExportLossReason::SourceNormalNormalized);
                }
                context.block_points.insert(
                    common.handle,
                    crate::geometry::blocks::polyline_pairs(polyline, placement),
                );
                let count = polyline
                    .vertices
                    .iter()
                    .filter(|v| v.vertex_id != 0)
                    .count();
                if count > 0 {
                    common_losses.push(ExportLossReason::PolylineVertexIdentifiers { count });
                }
                target.add_planar_polyline(PlanarPolylineDefinition {
                    placement,
                    bulges: polyline
                        .vertices
                        .iter()
                        .map(|vertex| vertex.bulge)
                        .collect(),
                    points: polyline
                        .vertices
                        .iter()
                        .map(|vertex| Point2::new(vertex.location.x, vertex.location.y))
                        .collect(),
                    closed: polyline.is_closed,
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            EntityType::Polyline2D(polyline) => {
                let lw = lw_from_polyline2d(polyline);
                let (placement, bound, normal_changed) =
                    crate::geometry::from_cad(&lw, context.geometry.as_mut().unwrap())?;
                if bound > 0.0 {
                    common_losses.push(ExportLossReason::GeometryRoundedWithinTolerance {
                        max_deviation_upper_bound: bound,
                    });
                }
                if normal_changed {
                    common_losses.push(ExportLossReason::SourceNormalNormalized);
                }
                context.block_points.insert(
                    common.handle,
                    crate::geometry::blocks::polyline_pairs(&lw, placement),
                );
                target.add_planar_polyline(PlanarPolylineDefinition {
                    placement,
                    points: lw
                        .vertices
                        .iter()
                        .map(|vertex| Point2::new(vertex.location.x, vertex.location.y))
                        .collect(),
                    bulges: lw.vertices.iter().map(|vertex| vertex.bulge).collect(),
                    closed: polyline.flags.is_closed(),
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            EntityType::Polyline(polyline) => {
                let points = polyline
                    .vertices
                    .iter()
                    .map(|vertex| {
                        ifccad::ifcdr::Point3::new(
                            vertex.location.x,
                            vertex.location.y,
                            vertex.location.z,
                        )
                    })
                    .collect::<Vec<_>>();
                context.block_points.insert(
                    common.handle,
                    polyline
                        .vertices
                        .iter()
                        .map(|vertex| {
                            crate::geometry::blocks::PairedPoint::exact([
                                vertex.location.x,
                                vertex.location.y,
                                vertex.location.z,
                            ])
                        })
                        .collect(),
                );
                context.geometry.as_mut().unwrap().record(
                    crate::ConversionEntitySource::CadEntity {
                        handle: common.handle,
                        kind: "POLYLINE".into(),
                    },
                    points.len(),
                    0.0,
                );
                target.add_spatial_polyline(SpatialPolylineDefinition {
                    points,
                    closed: polyline.flags.is_closed(),
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            EntityType::Polyline3D(polyline) => {
                let points = polyline
                    .vertices
                    .iter()
                    .map(|vertex| {
                        ifccad::ifcdr::Point3::new(
                            vertex.position.x,
                            vertex.position.y,
                            vertex.position.z,
                        )
                    })
                    .collect::<Vec<_>>();
                context.block_points.insert(
                    common.handle,
                    polyline
                        .vertices
                        .iter()
                        .map(|vertex| {
                            crate::geometry::blocks::PairedPoint::exact([
                                vertex.position.x,
                                vertex.position.y,
                                vertex.position.z,
                            ])
                        })
                        .collect(),
                );
                context.geometry.as_mut().unwrap().record(
                    crate::ConversionEntitySource::CadEntity {
                        handle: common.handle,
                        kind: "POLYLINE".into(),
                    },
                    points.len(),
                    0.0,
                );
                target.add_spatial_polyline(SpatialPolylineDefinition {
                    points,
                    closed: polyline.flags.closed,
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            _ => unreachable!("unsupported entity was classified as loss"),
        };
        if let EntityType::Line(line) = source {
            context.block_points.insert(
                common.handle,
                [line.start, line.end]
                    .map(|p| crate::geometry::blocks::PairedPoint::exact([p.x, p.y, p.z]))
                    .to_vec(),
            );
            context.geometry.as_mut().unwrap().record(
                crate::ConversionEntitySource::CadEntity {
                    handle: common.handle,
                    kind: "LINE".into(),
                },
                2,
                0.0,
            );
            if line.normal != Vector3::UNIT_Z {
                common_losses.push(ExportLossReason::UnsupportedNormal);
            }
        }
        if let EntityType::Viewport(viewport) = source {
            common_losses.extend(viewport_deferred_losses(viewport));
        }
        context.entity_mapping.insert(common.handle, entity_id);
        if !common_losses.is_empty() {
            record_diagnostic(
                source,
                ExportAction::PartiallyExported,
                common_losses,
                context,
            );
        }
    }
    Ok(structural_problems)
}

fn classify_owner(
    document: &CadDocument,
    model_space: &ModelSpaceInfo<'_>,
    source: &EntityType,
    common: &EntityCommon,
    context: &mut ExportContext,
    problems: &mut Vec<SourceStructureProblem>,
    common_losses: &[ExportLossReason],
) -> bool {
    if common.owner_handle == model_space.block_handle
        || context.blocks.contains_key(&common.owner_handle)
        || context.paper_scopes.contains_key(&common.owner_handle)
    {
        return true;
    }
    if common.owner_handle == Handle::NULL {
        problems.push(SourceStructureProblem::EntityOwnerMissing {
            entity: common.handle,
        });
    } else if common.owner_handle == document.header.paper_space_block_handle {
        let mut reasons = vec![ExportLossReason::PaperSpaceEntity];
        reasons.extend_from_slice(common_losses);
        record_skipped(source, reasons, context);
    } else if document
        .block_records
        .iter()
        .any(|record| record.handle == common.owner_handle)
    {
        let mut reasons = vec![ExportLossReason::BlockOwnedEntity {
            owner: common.owner_handle,
        }];
        reasons.extend_from_slice(common_losses);
        record_skipped(source, reasons, context);
    } else {
        problems.push(SourceStructureProblem::EntityOwnerUnknown {
            entity: common.handle,
            owner: common.owner_handle,
        });
    }
    false
}

fn record_skipped(
    source: &EntityType,
    reasons: Vec<ExportLossReason>,
    context: &mut ExportContext,
) {
    record_diagnostic(source, ExportAction::Skipped, reasons, context);
}

fn record_diagnostic(
    source: &EntityType,
    action: ExportAction,
    reasons: Vec<ExportLossReason>,
    context: &mut ExportContext,
) {
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::Entity {
            handle: source.common().handle,
            kind: source.as_entity().entity_type().to_owned(),
        },
        action,
        reasons,
    ));
}

fn common_semantic_losses(common: &EntityCommon) -> Vec<ExportLossReason> {
    let mut reasons = Vec::new();
    if common.linetype_scale != 1.0 {
        reasons.push(ExportLossReason::EntityLinetypeScale);
    }
    if common.linetype_handle.is_some() {
        reasons.push(ExportLossReason::EntityLinetypeHandle);
    }
    if !common.extended_data.is_empty() {
        reasons.push(ExportLossReason::EntityExtendedData);
    }
    if common.graphic_data.is_some() {
        reasons.push(ExportLossReason::EntityGraphicData);
    }
    if !common.reactors.is_empty() {
        reasons.push(ExportLossReason::EntityReactors);
    }
    if common.xdictionary_handle.is_some() {
        reasons.push(ExportLossReason::EntityExtensionDictionary);
    }
    if common.color_book_handle.is_some() {
        reasons.push(ExportLossReason::EntityColorBookReference);
    }
    if common.full_visual_style_handle.is_some() {
        reasons.push(ExportLossReason::EntityFullVisualStyle);
    }
    if common.face_visual_style_handle.is_some() {
        reasons.push(ExportLossReason::EntityFaceVisualStyle);
    }
    if common.edge_visual_style_handle.is_some() {
        reasons.push(ExportLossReason::EntityEdgeVisualStyle);
    }
    if common.material_flags != 0 || common.material_handle.is_some() {
        reasons.push(ExportLossReason::EntityMaterial);
    }
    if common.shadow_flags != 0 {
        reasons.push(ExportLossReason::EntityShadowFlags);
    }
    if common.plotstyle_flags != 0 || common.plotstyle_handle.is_some() {
        reasons.push(ExportLossReason::EntityPlotStyle);
    }
    reasons
}

fn line_losses(line: &Line) -> Vec<ExportLossReason> {
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
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if line.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    reasons
}

fn point_losses(point: &Point) -> Vec<ExportLossReason> {
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
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if point.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    if crate::geometry::cad_plane(point.normal).is_none() {
        reasons.push(ExportLossReason::UnsupportedNormal);
    }
    reasons
}

fn circle_losses(circle: &Circle) -> Vec<ExportLossReason> {
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
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if circle.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    if circle.radius <= 0.0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "circle radius".into(),
        });
    }
    if crate::geometry::circular::from_cad_ocs(circle.center, circle.normal).is_none() {
        reasons.push(ExportLossReason::UnsupportedNormal);
    }
    reasons
}

fn arc_losses(arc: &Arc) -> Vec<ExportLossReason> {
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
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if arc.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    if arc.radius <= 0.0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
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
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "arc sweep".into(),
        });
    }
    if crate::geometry::circular::from_cad_ocs(arc.center, arc.normal).is_none() {
        reasons.push(ExportLossReason::UnsupportedNormal);
    }
    reasons
}

fn ellipse_losses(ellipse: &Ellipse) -> Vec<ExportLossReason> {
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
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if crate::geometry::circular::from_cad_ellipse(ellipse).is_none() {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "ellipse frame or axis ratio".into(),
        });
    }
    let difference = ellipse.end_parameter - ellipse.start_parameter;
    let sweep = difference.rem_euclid(std::f64::consts::TAU);
    if !difference.is_finite()
        || difference.abs() > std::f64::consts::TAU
        || (difference != std::f64::consts::TAU && sweep == 0.0)
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "ellipse parameter sweep".into(),
        });
    }
    reasons
}

fn polyline_losses(polyline: &LwPolyline) -> Vec<ExportLossReason> {
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
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if polyline.vertices.len() < 2 {
        reasons.push(ExportLossReason::PolylineTooFewVertices {
            count: polyline.vertices.len(),
        });
    }
    if polyline.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    if polyline.normal.x == 0.0 && polyline.normal.y == 0.0 && polyline.normal.z == 0.0 {
        reasons.push(ExportLossReason::UnsupportedNormal);
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
            reasons.push(ExportLossReason::UnsupportedSemantic {
                name: "bulged zero-length polyline segment".into(),
            });
            break;
        }
    }
    if polyline.plinegen {
        reasons.push(ExportLossReason::PolylinePlinegen);
    }
    reasons
}

fn lw_from_polyline2d(polyline: &cadcodec::entities::Polyline2D) -> LwPolyline {
    let mut lw = LwPolyline::from_points(
        polyline
            .vertices
            .iter()
            .map(|vertex| cadcodec::Vector2::new(vertex.location.x, vertex.location.y))
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

fn polyline2d_losses(polyline: &cadcodec::entities::Polyline2D) -> Vec<ExportLossReason> {
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
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if polyline.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    if polyline.flags.bits() & (2 | 4) != 0
        || polyline.smooth_surface != cadcodec::entities::SmoothSurfaceType::None
        || polyline.vertices.iter().any(|vertex| {
            vertex.flags.bits() & (1 | 2 | 8 | 16) != 0 || vertex.curve_tangent != 0.0
        })
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "polyline fit curve".into(),
        });
    }
    if polyline.flags.bits() & (8 | 16 | 32 | 64) != 0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "polyline mesh or 3D flags".into(),
        });
    }
    if polyline.flags.bits() & 128 != 0 {
        reasons.push(ExportLossReason::PolylinePlinegen);
    }
    if polyline.flags.bits() & !0xff != 0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "polyline flags".into(),
        });
    }
    if polyline
        .vertices
        .iter()
        .any(|vertex| vertex.flags.bits() & !(1 | 2 | 8 | 16) != 0)
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "2D polyline vertex flags".into(),
        });
    }
    if polyline
        .vertices
        .iter()
        .any(|vertex| vertex.location.z != 0.0)
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "2D polyline vertex elevation".into(),
        });
    }
    reasons
}

fn spatial_polyline_losses(
    points: impl IntoIterator<Item = Vector3>,
    flags: u16,
) -> Vec<ExportLossReason> {
    let points = points.into_iter().collect::<Vec<_>>();
    let mut reasons = Vec::new();
    if points.len() < 2 {
        reasons.push(ExportLossReason::PolylineTooFewVertices {
            count: points.len(),
        });
    }
    if points
        .iter()
        .any(|point| ![point.x, point.y, point.z].into_iter().all(f64::is_finite))
    {
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if flags & (2 | 4) != 0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "polyline fit curve".into(),
        });
    }
    if flags & (16 | 32 | 64) != 0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "polyline mesh".into(),
        });
    }
    if flags & !(1 | 8) != 0 && flags & !(1 | 8 | 2 | 4 | 16 | 32 | 64) != 0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "polyline flags".into(),
        });
    }
    reasons
}

fn generic_polyline_losses(polyline: &cadcodec::entities::Polyline) -> Vec<ExportLossReason> {
    let mut reasons = spatial_polyline_losses(
        polyline.vertices.iter().map(|vertex| vertex.location),
        polyline.flags.bits(),
    );
    if polyline
        .vertices
        .iter()
        .any(|vertex| vertex.flags.bits() != 0)
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "3D polyline vertex flags".into(),
        });
    }
    reasons
}

fn polyline3d_losses(polyline: &cadcodec::entities::Polyline3D) -> Vec<ExportLossReason> {
    let mut reasons = spatial_polyline_losses(
        polyline.vertices.iter().map(|vertex| vertex.position),
        polyline.flags.to_bits() as u16,
    );
    if polyline.default_start_width != 0.0 || polyline.default_end_width != 0.0 {
        reasons.push(ExportLossReason::PolylineWidth);
    }
    if !polyline.flags.is_3d
        || polyline.elevation != 0.0
        || polyline.normal != Vector3::UNIT_Z
        || polyline.mesh_m_count != 0
        || polyline.mesh_n_count != 0
        || polyline.smooth_m_density != 0
        || polyline.smooth_n_density != 0
        || polyline.smooth_type != cadcodec::entities::polyline3d::SmoothSurfaceType::None
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "3D polyline source properties".into(),
        });
    }
    if polyline
        .vertices
        .iter()
        .any(|vertex| vertex.flags != 32 || vertex.handle != Handle::NULL || vertex.layer != "0")
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "3D polyline vertex properties".into(),
        });
    }
    reasons
}

fn viewport_losses(
    viewport: &cadcodec::entities::Viewport,
    document: &CadDocument,
    context: &ExportContext,
) -> Vec<ExportLossReason> {
    let mut reasons = Vec::new();
    if viewport.status.perspective {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "perspective viewport needs CAD fixture calibration".into(),
        });
    }
    if viewport.clip_boundary_handle != Handle::NULL {
        let supported = matches!(document.get_entity(viewport.clip_boundary_handle), Some(EntityType::LwPolyline(boundary))
            if boundary.common.owner_handle == viewport.common.owner_handle
                && boundary.is_closed && boundary.vertices.len() >= 3
                && boundary.vertices.iter().all(|vertex| vertex.bulge == 0.0)
                && boundary.elevation == 0.0 && boundary.normal == Vector3::UNIT_Z)
            && context
                .entity_mapping
                .target_entity_id(viewport.clip_boundary_handle)
                .is_some();
        if !supported {
            reasons.push(ExportLossReason::UnsupportedSemantic {
                name: "active viewport clip boundary".into(),
            });
        }
    }
    if viewport.center.z != 0.0 || viewport.view_center.z != 0.0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport paper or DCS z coordinate".into(),
        });
    }
    if viewport.width <= 0.0
        || viewport.height <= 0.0
        || viewport.view_height <= 0.0
        || ![
            viewport.center.x,
            viewport.center.y,
            viewport.width,
            viewport.height,
            viewport.view_center.x,
            viewport.view_center.y,
            viewport.view_target.x,
            viewport.view_target.y,
            viewport.view_target.z,
            viewport.view_direction.x,
            viewport.view_direction.y,
            viewport.view_direction.z,
            viewport.view_height,
            viewport.twist_angle,
            viewport.lens_length,
            viewport.front_clip_z,
            viewport.back_clip_z,
        ]
        .into_iter()
        .all(f64::is_finite)
    {
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    reasons
}

fn viewport_deferred_losses(viewport: &cadcodec::entities::Viewport) -> Vec<ExportLossReason> {
    let baseline = cadcodec::entities::Viewport::new();
    let mut reasons = Vec::new();
    let mut status = viewport.status;
    status.is_on = baseline.status.is_on;
    status.locked = baseline.status.locked;
    status.perspective = baseline.status.perspective;
    status.front_clipping = baseline.status.front_clipping;
    status.back_clipping = baseline.status.back_clipping;
    status.front_clip_not_at_eye = baseline.status.front_clip_not_at_eye;
    if status != baseline.status
        || viewport.snap_base != baseline.snap_base
        || viewport.snap_spacing != baseline.snap_spacing
        || viewport.grid_spacing != baseline.grid_spacing
        || viewport.snap_angle != baseline.snap_angle
        || viewport.circle_sides != baseline.circle_sides
        || viewport.grid_flags != baseline.grid_flags
        || viewport.grid_major != baseline.grid_major
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport workspace snap/grid/display state".into(),
        });
    }
    if viewport.ucs_at_origin != baseline.ucs_at_origin
        || viewport.ucs_per_viewport != baseline.ucs_per_viewport
        || viewport.ucs_icon_visible != baseline.ucs_icon_visible
        || viewport.ucs_origin != baseline.ucs_origin
        || viewport.ucs_x_axis != baseline.ucs_x_axis
        || viewport.ucs_y_axis != baseline.ucs_y_axis
        || viewport.ucs_handle != Handle::NULL
        || viewport.base_ucs_handle != Handle::NULL
        || viewport.ucs_ortho_type != baseline.ucs_ortho_type
        || viewport.elevation != baseline.elevation
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport UCS state".into(),
        });
    }
    if !viewport.style_sheet.is_empty()
        || viewport.shade_plot_mode != baseline.shade_plot_mode
        || viewport.background_handle != Handle::NULL
        || viewport.shade_plot_handle != Handle::NULL
        || viewport.visual_style_handle != Handle::NULL
        || viewport.sun_handle != Handle::NULL
        || viewport.default_lighting != baseline.default_lighting
        || viewport.default_lighting_type != baseline.default_lighting_type
        || viewport.brightness != baseline.brightness
        || viewport.contrast != baseline.contrast
        || viewport.ambient_color != baseline.ambient_color
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport visual and plot state".into(),
        });
    }
    reasons
}
