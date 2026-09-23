use super::appearance::EntityAppearanceError;
use super::conversion::ExportContext;
use super::structure::ModelSpaceInfo;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason,
    SourceStructureProblem,
};
use cadcodec::entities::EntityCommon;
use cadcodec::{CadDocument, EntityType, Handle, Line, LwPolyline, Vector3};
use ifccad::ifcdr::Point2;
use ifccad::package::{
    DrawingBuilder, LineDefinition, PolylineDefinition, ViewportDefinition,
    ViewportLayerOverrideDefinition,
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
            EntityType::Viewport(viewport)
                if context.paper_scopes.contains_key(&common.owner_handle) =>
            {
                viewport_losses(viewport, document, context)
            }
            EntityType::LwPolyline(polyline) => polyline_losses(polyline),
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
                target.add_polyline(PolylineDefinition {
                    placement,
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
    if polyline.vertices.iter().any(|vertex| vertex.bulge != 0.0) {
        reasons.push(ExportLossReason::PolylineBulge);
    }
    if polyline.constant_width != 0.0
        || polyline
            .vertices
            .iter()
            .any(|vertex| vertex.start_width != 0.0 || vertex.end_width != 0.0)
    {
        reasons.push(ExportLossReason::PolylineWidth);
    }
    if polyline.plinegen {
        reasons.push(ExportLossReason::PolylinePlinegen);
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
