use super::appearance::{
    map_entity_opacity, map_explicit_color, map_layer_opacity, map_line_pattern, map_line_weight,
    MappedColor,
};
use super::diagnostic::DiagnosticAccumulator;
use super::units::apply_units;
use crate::{ImportEntityMapping, ImportError, ImportOutcome};
use cadcodec::entities::EntityCommon;
use cadcodec::{CadDocument, Color, EntityType, Layer, Line, LineType};
use ifccad::ifcdr::{AppearanceId, EntityId, IfcdrEntityRef, LayerId};
use ifccad::package::{
    AppearanceProperty, DrawingLayoutKind, DrawingRef, DrawingRepresentationRef, LayerRef,
};

/// Imports one validated IFCCAD drawing into a cadcodec [`CadDocument`].
///
/// `Import` is named from the `CadDocument` boundary: IFCCAD is the source and
/// the returned CAD document is the destination.
pub fn drawing_to_cad_document(drawing: DrawingRef<'_>) -> Result<ImportOutcome, ImportError> {
    drawing_to_cad_document_with_options(drawing, crate::ImportOptions::default())
}
pub fn drawing_to_cad_document_with_options(
    drawing: DrawingRef<'_>,
    options: crate::ImportOptions,
) -> Result<ImportOutcome, ImportError> {
    let layouts = drawing.layouts().collect::<Vec<_>>();
    let model_layouts = layouts
        .iter()
        .filter(|layout| layout.kind() == DrawingLayoutKind::Model)
        .count();
    if model_layouts != 1 {
        return Err(ImportError::UnsupportedDrawingStructure {
            total_layouts: layouts.len(),
            model_layouts,
        });
    }

    let layout = layouts[0];
    let representation = layout.representation();
    let mut document = CadDocument::new();
    let mut diagnostics = DiagnosticAccumulator::default();
    let mut geometry = crate::ConversionGeometryAssessment::new(
        options.geometry_tolerance,
        representation.resource().unit(),
    )?;
    apply_units(&mut document, representation.resource().unit());
    document.header.plotstyle_mode =
        drawing.plot_style_mode() == ifccad::package::PlotStyleMode::ColorDependent;
    (
        document.header.point_display_mode,
        document.header.point_display_size,
    ) = crate::point_display::to_cad(drawing.point_display());
    let paper_owners = super::layouts::allocate(&mut document, &layouts, &mut diagnostics)?;

    for source in representation.layers() {
        let target = convert_layer(&mut document, source, &mut diagnostics)?;
        if source.name() == "0" {
            let standard =
                document
                    .layers
                    .get_mut("0")
                    .ok_or_else(|| ImportError::InternalInvariant {
                        message: "fresh CadDocument has no standard layer 0".to_owned(),
                    })?;
            standard.flags = target.flags;
            standard.is_plottable = target.is_plottable;
            standard.description = target.description;
            standard.color = target.color;
            standard.line_type = target.line_type;
            standard.line_weight = target.line_weight;
            standard.transparency = target.transparency;
        } else {
            let name = target.name.clone();
            document
                .layers
                .add(target)
                .map_err(|reason| ImportError::LayerInsertion {
                    layer: name,
                    reason,
                })?;
        }
    }

    let scope_id = layout.scope().id();
    let mut owners = super::blocks::allocate(&mut document, representation.resource(), scope_id)?;
    owners.extend(paper_owners);
    let mut block_instances = std::collections::BTreeMap::new();
    let mut block_points = std::collections::BTreeMap::new();
    let mut entity_mapping = ImportEntityMapping::default();
    for (scope_id, owner) in owners {
        for source in representation.resource().entities(scope_id) {
            match source {
                IfcdrEntityRef::Viewport(source) => {
                    if source.view().projection == ifccad::ifcdr::ProjectionMode::Perspective {
                        diagnostics.record(crate::ImportDiagnostic::ViewportUnsupported {
                            entity_id: source.entity_id(),
                            reason: "perspective calibration requires CAD fixtures".into(),
                        });
                        continue;
                    }
                    let clip = source.paper_clip();
                    let clip_handle = clip.boundary_entity_id.and_then(|id| {
                        ifccad::ifcdr::EntityId::new(id)
                            .and_then(|id| entity_mapping.target_handle(id))
                    });
                    if clip.enabled && clip_handle.is_none() {
                        diagnostics.record(crate::ImportDiagnostic::ViewportUnsupported {
                            entity_id: source.entity_id(),
                            reason: "clip boundary could not be mapped".into(),
                        });
                        continue;
                    }
                    let frame = source.frame();
                    let view = source.view();
                    let mut target = cadcodec::entities::Viewport::new();
                    target.center = cadcodec::Vector3::new(frame.center.x(), frame.center.y(), 0.0);
                    target.width = frame.width;
                    target.height = frame.height;
                    target.view_center =
                        cadcodec::Vector3::new(view.center.x(), view.center.y(), 0.0);
                    target.view_target =
                        cadcodec::Vector3::new(view.target.x(), view.target.y(), view.target.z());
                    target.view_direction = cadcodec::Vector3::new(
                        view.direction.x(),
                        view.direction.y(),
                        view.direction.z(),
                    );
                    target.view_height = view.height;
                    target.twist_angle = view.twist;
                    target.lens_length = view.lens_length.unwrap_or(target.lens_length);
                    target.status.front_clipping =
                        view.front_clip.mode != ifccad::ifcdr::FrontClipMode::Disabled;
                    target.status.front_clip_not_at_eye =
                        view.front_clip.mode == ifccad::ifcdr::FrontClipMode::AtDistance;
                    target.front_clip_z = view.front_clip.distance.unwrap_or(0.0);
                    target.status.back_clipping =
                        view.back_clip.mode != ifccad::ifcdr::BackClipMode::Disabled;
                    target.back_clip_z = view.back_clip.distance.unwrap_or(0.0);
                    target.clip_boundary_handle = clip_handle.unwrap_or(cadcodec::Handle::NULL);
                    target.status.is_on = source.view_enabled();
                    target.status.locked = source.view_locked();
                    target.render_mode = match source.render_mode() {
                        ifccad::ifcdr::ViewportRenderMode::TwoDimensional => {
                            cadcodec::entities::ViewportRenderMode::Wireframe2D
                        }
                        ifccad::ifcdr::ViewportRenderMode::Wireframe => {
                            cadcodec::entities::ViewportRenderMode::Wireframe3D
                        }
                        ifccad::ifcdr::ViewportRenderMode::HiddenLine => {
                            cadcodec::entities::ViewportRenderMode::HiddenLine
                        }
                        ifccad::ifcdr::ViewportRenderMode::FlatShadedWithoutEdges => {
                            cadcodec::entities::ViewportRenderMode::FlatShaded
                        }
                        ifccad::ifcdr::ViewportRenderMode::FlatShadedWithEdges => {
                            cadcodec::entities::ViewportRenderMode::FlatShadedWithEdges
                        }
                        ifccad::ifcdr::ViewportRenderMode::SmoothShadedWithoutEdges => {
                            cadcodec::entities::ViewportRenderMode::GouraudShaded
                        }
                        ifccad::ifcdr::ViewportRenderMode::SmoothShadedWithEdges => {
                            cadcodec::entities::ViewportRenderMode::GouraudShadedWithEdges
                        }
                    };
                    target.id = document
                        .entities()
                        .filter_map(|entity| match entity {
                            EntityType::Viewport(viewport) => Some(viewport.id),
                            _ => None,
                        })
                        .max()
                        .unwrap_or(1)
                        .checked_add(1)
                        .ok_or_else(|| ImportError::InternalInvariant {
                            message: "CAD viewport ID range exhausted".into(),
                        })?;
                    for override_row in source.layer_overrides() {
                        if override_row.frozen {
                            if let Some(layer) = representation
                                .layer(override_row.layer_id.into())
                                .and_then(|layer| document.layers.get(layer.name()))
                            {
                                target.frozen_layers.push(layer.handle);
                            }
                        }
                        if override_row.appearance_override_id.is_some() {
                            diagnostics.record(crate::ImportDiagnostic::ViewportUnsupported {
                                entity_id: source.entity_id(),
                                reason: "viewport appearance override".into(),
                            });
                        }
                    }
                    if source.plot_shading_override().is_some() {
                        diagnostics.record(crate::ImportDiagnostic::ViewportUnsupported {
                            entity_id: source.entity_id(),
                            reason: "viewport plot-shading quality override".into(),
                        });
                    }
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Viewport(target),
                    )?;
                }
                IfcdrEntityRef::BlockInstance(source) => {
                    let resource = representation.resource();
                    let Some(ifccad::ifcdr::ScopeRef::BlockDefinition(definition)) =
                        resource.scope(source.definition_scope_id())
                    else {
                        return Err(ImportError::InternalInvariant {
                            message: "validated block target missing".into(),
                        });
                    };
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let (mut target, source_map, target_map, changed) =
                        crate::geometry::blocks::to_cad_instance(
                            source,
                            definition,
                            identity.clone(),
                            &geometry,
                        )?;
                    if changed {
                        diagnostics.record(crate::ImportDiagnostic::BlockParameterizationChanged {
                            source: identity,
                        });
                    }
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Insert(target),
                    )?;
                    block_instances.insert(
                        source.entity_id(),
                        super::blocks::ConvertedInstance {
                            definition: source.definition_scope_id(),
                            owner_scope: scope_id,
                            source: source_map,
                            target: target_map,
                        },
                    );
                }
                IfcdrEntityRef::Point(source) => {
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let position = source.position();
                    geometry.record(identity, 1, 0.0);
                    block_points.insert(
                        source.entity_id(),
                        vec![crate::geometry::blocks::PairedPoint::exact([
                            position.x(),
                            position.y(),
                            position.z(),
                        ])],
                    );
                    let plane = source.placement();
                    let normal = crate::geometry::stored_normal(plane).ok_or_else(|| {
                        ImportError::InternalInvariant {
                            message: "point placement normal cannot be evaluated".into(),
                        }
                    })?;
                    let basis = crate::geometry::cad_plane(normal).ok_or_else(|| {
                        ImportError::InternalInvariant {
                            message: "CAD point frame cannot be evaluated".into(),
                        }
                    })?;
                    let x = plane.x_axis();
                    let dot = |axis: [f64; 3]| x.x() * axis[0] + x.y() * axis[1] + x.z() * axis[2];
                    let mut target = cadcodec::entities::Point::at(cadcodec::Vector3::new(
                        position.x(),
                        position.y(),
                        position.z(),
                    ));
                    target.normal = normal;
                    target.x_axis_angle = dot(basis.v).atan2(dot(basis.u));
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Point(target),
                    )?;
                }
                IfcdrEntityRef::Circle(source) => {
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let (center, normal, angle) =
                        crate::geometry::circular::to_cad_ocs(source.placement(), false)
                            .ok_or_else(|| {
                                geometry.failure(
                                    &identity,
                                    None,
                                    crate::ConversionGeometryStage::TargetConstruction,
                                    crate::ConversionGeometryFailureReason::CadAxisEvaluationFailed,
                                )
                            })?;
                    let mut target = cadcodec::Circle::from_center_radius(center, source.radius());
                    target.normal = normal;
                    let samples = crate::geometry::circular::circle_sample_pairs(
                        source.placement(),
                        source.radius(),
                        angle,
                        &target,
                    )?;
                    let mut maximum = 0.0_f64;
                    let points = samples
                        .into_iter()
                        .enumerate()
                        .map(|(index, (point, squared))| {
                            maximum = maximum.max(geometry.check(&identity, index, &squared)?);
                            Ok(point)
                        })
                        .collect::<Result<Vec<_>, Box<crate::ConversionGeometryFailure>>>()?;
                    geometry.record(identity.clone(), points.len(), maximum);
                    if maximum > 0.0 {
                        diagnostics.record(
                            crate::ImportDiagnostic::GeometryRoundedWithinTolerance {
                                source: identity,
                                max_deviation_upper_bound: maximum,
                            },
                        );
                    }
                    block_points.insert(source.entity_id(), points);
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Circle(target),
                    )?;
                }
                IfcdrEntityRef::Arc(source) => {
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let sweep = source.sweep_parameter();
                    let (center, normal, frame_angle) =
                        crate::geometry::circular::to_cad_ocs(source.placement(), sweep < 0.0)
                            .ok_or_else(|| {
                                geometry.failure(
                                    &identity,
                                    None,
                                    crate::ConversionGeometryStage::TargetConstruction,
                                    crate::ConversionGeometryFailureReason::CadAxisEvaluationFailed,
                                )
                            })?;
                    let start = frame_angle
                        + if sweep < 0.0 {
                            -source.start_parameter()
                        } else {
                            source.start_parameter()
                        };
                    let mut target = cadcodec::Arc::from_center_radius_angles(
                        center,
                        source.radius(),
                        start,
                        start + sweep.abs(),
                    );
                    target.normal = normal;
                    let samples = crate::geometry::circular::arc_sample_pairs(
                        source.placement(),
                        source.radius(),
                        source.start_parameter(),
                        sweep,
                        &target,
                    )?;
                    let mut maximum = 0.0_f64;
                    let points = samples
                        .into_iter()
                        .enumerate()
                        .map(|(index, (point, squared))| {
                            maximum = maximum.max(geometry.check(&identity, index, &squared)?);
                            Ok(point)
                        })
                        .collect::<Result<Vec<_>, Box<crate::ConversionGeometryFailure>>>()?;
                    geometry.record(identity.clone(), points.len(), maximum);
                    if maximum > 0.0 {
                        diagnostics.record(
                            crate::ImportDiagnostic::GeometryRoundedWithinTolerance {
                                source: identity,
                                max_deviation_upper_bound: maximum,
                            },
                        );
                    }
                    block_points.insert(source.entity_id(), points);
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Arc(target),
                    )?;
                }
                IfcdrEntityRef::Ellipse(source) => {
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let mut target = crate::geometry::circular::to_cad_ellipse(
                        source.placement(),
                        source.semi_major_radius(),
                        source.semi_minor_radius(),
                        false,
                    )
                    .ok_or_else(|| {
                        geometry.failure(
                            &identity,
                            None,
                            crate::ConversionGeometryStage::TargetConstruction,
                            crate::ConversionGeometryFailureReason::CadAxisEvaluationFailed,
                        )
                    })?;
                    let samples = crate::geometry::circular::ellipse_sample_pairs(
                        source.placement(),
                        source.semi_major_radius(),
                        source.semi_minor_radius(),
                        0.0,
                        std::f64::consts::TAU,
                        &target,
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
                    geometry.record(identity.clone(), points.len(), maximum);
                    if maximum > 0.0 {
                        diagnostics.record(
                            crate::ImportDiagnostic::GeometryRoundedWithinTolerance {
                                source: identity,
                                max_deviation_upper_bound: maximum,
                            },
                        );
                    }
                    block_points.insert(source.entity_id(), points);
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Ellipse(target),
                    )?;
                }
                IfcdrEntityRef::EllipseArc(source) => {
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let sweep = source.sweep_parameter();
                    let mut target = crate::geometry::circular::to_cad_ellipse(
                        source.placement(),
                        source.semi_major_radius(),
                        source.semi_minor_radius(),
                        sweep < 0.0,
                    )
                    .ok_or_else(|| {
                        geometry.failure(
                            &identity,
                            None,
                            crate::ConversionGeometryStage::TargetConstruction,
                            crate::ConversionGeometryFailureReason::CadAxisEvaluationFailed,
                        )
                    })?;
                    target.start_parameter = if sweep < 0.0 {
                        -source.start_parameter()
                    } else {
                        source.start_parameter()
                    };
                    target.end_parameter = target.start_parameter + sweep.abs();
                    let samples = crate::geometry::circular::ellipse_sample_pairs(
                        source.placement(),
                        source.semi_major_radius(),
                        source.semi_minor_radius(),
                        source.start_parameter(),
                        sweep,
                        &target,
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
                    geometry.record(identity.clone(), points.len(), maximum);
                    if maximum > 0.0 {
                        diagnostics.record(
                            crate::ImportDiagnostic::GeometryRoundedWithinTolerance {
                                source: identity,
                                max_deviation_upper_bound: maximum,
                            },
                        );
                    }
                    block_points.insert(source.entity_id(), points);
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Ellipse(target),
                    )?;
                }
                IfcdrEntityRef::Line(source) => {
                    geometry.record(
                        crate::ConversionEntitySource::IfcdrEntity {
                            resource_id: representation.resource().resource_id().clone(),
                            scope_id,
                            entity_id: source.entity_id(),
                        },
                        2,
                        0.0,
                    );
                    let start = source.start();
                    let end = source.end();
                    block_points.insert(
                        source.entity_id(),
                        [start, end]
                            .map(|p| {
                                crate::geometry::blocks::PairedPoint::exact([p.x(), p.y(), p.z()])
                            })
                            .to_vec(),
                    );
                    let mut target = Line::from_coords(
                        start.x(),
                        start.y(),
                        start.z(),
                        end.x(),
                        end.y(),
                        end.z(),
                    );
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Line(target),
                    )?;
                }
                IfcdrEntityRef::PlanarPolyline(source) => {
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let (mut target, bound, changed) =
                        crate::geometry::to_cad(source, identity.clone(), &mut geometry)?;
                    block_points.insert(
                        source.entity_id(),
                        crate::geometry::blocks::import_polyline_pairs(source, &target),
                    );
                    if changed {
                        diagnostics.record(crate::ImportDiagnostic::PlaneParameterizationChanged {
                            source: identity.clone(),
                        });
                    }
                    if bound > 0.0 {
                        diagnostics.record(
                            crate::ImportDiagnostic::GeometryRoundedWithinTolerance {
                                source: identity,
                                max_deviation_upper_bound: bound,
                            },
                        );
                    }
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::LwPolyline(target),
                    )?;
                }
                IfcdrEntityRef::SpatialPolyline(source) => {
                    let identity = crate::ConversionEntitySource::IfcdrEntity {
                        resource_id: representation.resource().resource_id().clone(),
                        scope_id,
                        entity_id: source.entity_id(),
                    };
                    let mut target = cadcodec::entities::Polyline3D::from_points(
                        source
                            .points()
                            .iter()
                            .map(|p| cadcodec::Vector3::new(p.x(), p.y(), p.z()))
                            .collect(),
                    );
                    target.flags.closed = source.closed();
                    block_points.insert(
                        source.entity_id(),
                        source
                            .points()
                            .iter()
                            .map(|p| {
                                crate::geometry::blocks::PairedPoint::exact([p.x(), p.y(), p.z()])
                            })
                            .collect(),
                    );
                    geometry.record(identity, source.points().len(), 0.0);
                    apply_entity_common(
                        &mut document,
                        representation,
                        &mut target.common,
                        source.entity_id(),
                        source.layer_id(),
                        source.appearance_id(),
                        source.visible(),
                        &mut diagnostics,
                    )?;
                    target.common.owner_handle = owner;
                    add_and_map(
                        &mut document,
                        &mut entity_mapping,
                        source.entity_id(),
                        EntityType::Polyline3D(target),
                    )?;
                }
            }
        }
    }
    super::blocks::assess_occurrences(
        representation.resource(),
        &block_instances,
        &block_points,
        &mut geometry,
        &mut diagnostics,
    )?;

    let diagnostics = diagnostics.finish();
    if options.loss_policy == crate::ConversionLossPolicy::Reject
        && diagnostics.iter().any(|d| {
            !matches!(
                d,
                crate::ImportDiagnostic::GeometryRoundedWithinTolerance { .. }
            )
        })
    {
        return Err(ImportError::LossRejected { diagnostics });
    }
    Ok(
        ImportOutcome::new(document, diagnostics, entity_mapping)
            .with_geometry_assessment(geometry),
    )
}

#[allow(clippy::too_many_arguments)]
fn apply_entity_common(
    document: &mut CadDocument,
    representation: DrawingRepresentationRef<'_>,
    common: &mut EntityCommon,
    entity_id: EntityId,
    layer_id: LayerId,
    appearance_id: AppearanceId,
    visible: bool,
    diagnostics: &mut DiagnosticAccumulator,
) -> Result<(), ImportError> {
    let layer = representation
        .layer(layer_id)
        .ok_or(ImportError::MissingEntityLayer {
            entity_id,
            layer_id,
        })?;
    let appearance =
        representation
            .appearance(appearance_id)
            .ok_or(ImportError::MissingEntityAppearance {
                entity_id,
                appearance_id,
            })?;

    common.layer = layer.name().to_owned();
    common.invisible = !visible;
    match appearance.color() {
        AppearanceProperty::ByLayer => {
            common.color = Color::ByLayer;
            common.color_name = None;
        }
        AppearanceProperty::ByBlock => {
            common.color = Color::ByBlock;
            common.color_name = None;
        }
        AppearanceProperty::Explicit(color) => {
            let mapped = map_explicit_color(color);
            common.color = mapped.color;
            common.color_name = mapped.name;
        }
    }
    common.linetype = map_line_pattern(appearance.line_pattern(), diagnostics);
    ensure_linetype(document, &common.linetype)?;
    common.line_weight = map_line_weight(appearance.line_weight(), diagnostics);
    common.transparency = map_entity_opacity(appearance.opacity());
    Ok(())
}

fn add_and_map(
    document: &mut CadDocument,
    mapping: &mut ImportEntityMapping,
    source_id: EntityId,
    target: EntityType,
) -> Result<(), ImportError> {
    let handle =
        document
            .add_entity(target)
            .map_err(|source| ImportError::CadcodecEntityInsertion {
                entity_id: source_id,
                source,
            })?;
    mapping.insert(source_id, handle);
    Ok(())
}

fn convert_layer(
    document: &mut CadDocument,
    source: LayerRef<'_>,
    diagnostics: &mut DiagnosticAccumulator,
) -> Result<Layer, ImportError> {
    let mut target = Layer::new(source.name());
    target.flags.off = !source.visible();
    target.flags.frozen = source.frozen();
    target.flags.locked = source.locked();
    target.flags.frozen_in_new_viewport = source.frozen_in_new_viewports();
    target.is_plottable = source.plottable();
    target.description = source.description().unwrap_or("").into();

    if let Some(appearance) = source.appearance() {
        let source_color = appearance.color();
        let MappedColor { color, name: _ } = map_explicit_color(source_color);
        target.color = color;
        if let Some(named) = source_color.named() {
            target.color_name = Some(named.name().to_owned());
            target.book_name = Some(named.catalog().to_owned());
        }
        target.line_type = map_line_pattern(
            AppearanceProperty::Explicit(appearance.line_pattern()),
            diagnostics,
        );
        ensure_linetype(document, &target.line_type)?;
        target.line_weight = map_line_weight(
            AppearanceProperty::Explicit(appearance.line_weight()),
            diagnostics,
        );
        target.transparency = map_layer_opacity(appearance.opacity());
    }

    Ok(target)
}

pub(crate) fn ensure_linetype(document: &mut CadDocument, name: &str) -> Result<(), ImportError> {
    if name == "Dashed" && !document.line_types.contains(name) {
        document
            .line_types
            .add(LineType::dashed())
            .map_err(|reason| ImportError::InternalInvariant {
                message: format!("could not insert standard Dashed linetype: {reason}"),
            })?;
    }
    Ok(())
}
