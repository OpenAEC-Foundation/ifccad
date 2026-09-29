//! Direct CadDocument to standalone drawing conversion while the old package route is retired.

use crate::export::{
    direct_arc_losses, direct_circle_losses, direct_common_losses, direct_document_losses,
    direct_ellipse_losses, direct_entity, direct_generic_spatial_polyline_losses, direct_layer,
    direct_legacy_planar_polyline_losses, direct_line_losses, direct_planar_polyline_losses,
    direct_point_losses, direct_spatial_polyline_losses, inspect_markers, inspect_model_space,
    inspect_references, ordered_entities, with_recovered_model_space_handle, ExportAction,
    ExportDiagnostic, ExportDiagnosticSource, ExportLossReason, ExportOptions,
    SourceStructureProblem,
};
use crate::ConversionLossPolicy;
use cadcodec::objects::ObjectType;
use cadcodec::{CadDocument, EntityType, Handle};
use ocdraw::drawing::{
    ArcDefinition, BlockDefinition, BlockInstanceDefinition, CircleDefinition, CoordinateFrame3,
    DrawingBuildError, DrawingBuilder, DrawingOptions, EllipseDefinition, EncodedDrawing,
    LayoutRect, LayoutSettings, LineDefinition, PlanarPolylineDefinition, PlotStyleMode, Point3,
    PointDefinition, SpatialPolylineDefinition, UcsDefinition, Vector3,
};
use std::collections::BTreeMap;

pub(crate) const UNIT_TOKENS: [&str; 25] = [
    "unitless",
    "in",
    "ft",
    "mi",
    "mm",
    "cm",
    "m",
    "km",
    "microin",
    "mil",
    "yd",
    "angstrom",
    "nm",
    "um",
    "dm",
    "dam",
    "hm",
    "Gm",
    "au",
    "ly",
    "pc",
    "usSurveyFoot",
    "usSurveyInch",
    "usSurveyYard",
    "usSurveyMile",
];

pub struct DirectExportOutcome {
    drawing: EncodedDrawing,
    diagnostics: Vec<ExportDiagnostic>,
    entity_mapping: BTreeMap<Handle, u64>,
}

impl DirectExportOutcome {
    pub fn drawing(&self) -> &EncodedDrawing {
        &self.drawing
    }
    pub fn diagnostics(&self) -> &[ExportDiagnostic] {
        &self.diagnostics
    }
    pub fn entity_mapping(&self) -> &BTreeMap<Handle, u64> {
        &self.entity_mapping
    }
    pub fn into_drawing(self) -> EncodedDrawing {
        self.drawing
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DirectExportError {
    #[error("CAD source structure is invalid")]
    InvalidSourceStructure {
        problems: Vec<SourceStructureProblem>,
    },
    #[error("conversion loss was rejected")]
    LossRejected { diagnostics: Vec<ExportDiagnostic> },
    #[error(transparent)]
    DrawingBuild(#[from] DrawingBuildError),
}

fn loss(
    source: ExportDiagnosticSource,
    action: ExportAction,
    reasons: Vec<ExportLossReason>,
    diagnostics: &mut Vec<ExportDiagnostic>,
) {
    if !reasons.is_empty() {
        diagnostics.push(ExportDiagnostic::loss(source, action, reasons));
    }
}

pub fn cad_document_to_drawing(
    document: &CadDocument,
    options: ExportOptions,
) -> Result<DirectExportOutcome, DirectExportError> {
    let drawing_id = if document.header.fingerprint_guid.is_empty() {
        "drawing-main"
    } else {
        &document.header.fingerprint_guid
    };
    cad_document_to_drawing_with_id(document, drawing_id, options)
}

pub fn cad_document_to_drawing_with_id(
    document: &CadDocument,
    drawing_id: &str,
    options: ExportOptions,
) -> Result<DirectExportOutcome, DirectExportError> {
    let recovered = with_recovered_model_space_handle(document);
    let document = recovered.as_ref();
    let model = inspect_model_space(document)
        .map_err(|problems| DirectExportError::InvalidSourceStructure { problems })?;
    let mut problems = inspect_markers(document);
    problems.extend(inspect_references(document));
    if !problems.is_empty() {
        return Err(DirectExportError::InvalidSourceStructure { problems });
    }
    let mut diagnostics = direct_document_losses(document);
    let unit = usize::try_from(document.header.insertion_units)
        .ok()
        .and_then(|code| UNIT_TOKENS.get(code))
        .copied();
    if unit.is_none() {
        loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.insertion_units".into(),
            },
            ExportAction::PartiallyExported,
            vec![ExportLossReason::UnsupportedUnit {
                code: document.header.insertion_units,
            }],
            &mut diagnostics,
        );
    }
    let mut drawing =
        DrawingBuilder::new(DrawingOptions::new(drawing_id, unit.unwrap_or("unitless")))?;
    for source in document.ucss.iter() {
        let frame = CoordinateFrame3::try_new(
            Point3::new(source.origin.x, source.origin.y, source.origin.z),
            Vector3::new(source.x_axis.x, source.x_axis.y, source.x_axis.z),
            Vector3::new(source.y_axis.x, source.y_axis.y, source.y_axis.z),
        );
        if let Ok(frame) = frame {
            drawing.add_ucs_definition(UcsDefinition::new(
                &source.name,
                frame,
                source.elevation,
            ))?;
        } else {
            loss(
                ExportDiagnosticSource::DocumentField {
                    name: format!("ucs.{}", source.name),
                },
                ExportAction::Skipped,
                vec![ExportLossReason::UnsupportedSemantic {
                    name: "invalid UCS frame".into(),
                }],
                &mut diagnostics,
            );
        }
        if source.ortho_view_type != 0
            || source.ortho_type != 0
            || source.named_ucs_handle != Handle::NULL
            || source.base_ucs_handle != Handle::NULL
            || source.xref_reference
            || source.xref_dependent
        {
            loss(
                ExportDiagnosticSource::DocumentField {
                    name: format!("ucs.{}", source.name),
                },
                ExportAction::PartiallyExported,
                vec![ExportLossReason::UnsupportedSemantic {
                    name: "UCS orthographic or external-reference settings".into(),
                }],
                &mut diagnostics,
            );
        }
    }
    drawing.set_model_layout_name(model.layout_name);
    drawing.set_plot_style_mode(if document.header.plotstyle_mode {
        PlotStyleMode::ColorDependent
    } else {
        PlotStyleMode::Named
    });

    let mut paper_scopes = BTreeMap::new();
    for object in document.objects.values() {
        if let ObjectType::Layout(layout) = object {
            let normalized = crate::direct_layout::normalized_plot_integers(layout);
            for field in crate::direct_layout::unrepresented_fields(&normalized) {
                loss(
                    ExportDiagnosticSource::DocumentField {
                        name: format!("layout.{}.{}", layout.name, field),
                    },
                    ExportAction::PartiallyExported,
                    vec![ExportLossReason::UnsupportedSemantic { name: field.into() }],
                    &mut diagnostics,
                );
            }
            let id = if layout.block_record == model.block_handle {
                0
            } else {
                let id = drawing.add_paper_layout(&layout.name)?;
                paper_scopes.insert(layout.block_record, id);
                id
            };
            let limits = if layout.min_limits != layout.max_limits {
                Some(LayoutRect {
                    min_x: layout.min_limits.0,
                    min_y: layout.min_limits.1,
                    max_x: layout.max_limits.0,
                    max_y: layout.max_limits.1,
                })
            } else {
                None
            };
            let settings = LayoutSettings {
                limits,
                limits_checking: layout.flags & 2 != 0,
                paper_space_linetype_scaling: document.header.paper_space_linetype_scaling,
                plot_settings: match crate::direct_layout::plot_from_cad(&normalized, id == 0) {
                    Ok(settings) => settings,
                    Err(reason) => {
                        loss(
                            ExportDiagnosticSource::DocumentField {
                                name: format!("layout.{}.plot", layout.name),
                            },
                            ExportAction::PartiallyExported,
                            vec![ExportLossReason::UnsupportedSemantic {
                                name: reason.into(),
                            }],
                            &mut diagnostics,
                        );
                        None
                    }
                },
            };
            if drawing.set_layout_settings(id, settings).is_err() {
                loss(
                    ExportDiagnosticSource::DocumentField {
                        name: format!("layout.{}.limits", layout.name),
                    },
                    ExportAction::PartiallyExported,
                    vec![ExportLossReason::UnsupportedSemantic {
                        name: "invalid layout limits".into(),
                    }],
                    &mut diagnostics,
                );
            }
        }
    }
    let mut block_scopes = BTreeMap::new();
    let mut block_names = BTreeMap::new();
    for record in document.block_records.iter() {
        if record.handle == model.block_handle || paper_scopes.contains_key(&record.handle) {
            continue;
        }
        let mut definition = BlockDefinition::new(&record.name);
        definition.base_point = [
            record.base_point.x,
            record.base_point.y,
            record.base_point.z,
        ];
        definition.description = record.description.clone();
        definition.anonymous = record.flags.anonymous;
        definition.explodable = record.explodable;
        definition.uniform_scaling = record.scale_uniformly;
        if let Ok(code) = usize::try_from(record.units) {
            if let Some(unit) = UNIT_TOKENS.get(code) {
                definition.insertion_unit = (*unit).into();
            }
        }
        let scope = drawing.add_block_definition(definition)?;
        block_scopes.insert(record.handle, scope);
        block_names.insert(record.name.to_lowercase(), scope);
    }

    let mut layer_ids = BTreeMap::new();
    for layer in document.layers.iter() {
        match direct_layer(layer) {
            Ok((definition, reasons)) => {
                let id = drawing.add_layer(definition)?;
                layer_ids.insert(layer.name.to_lowercase(), id);
                loss(
                    ExportDiagnosticSource::Layer {
                        name: layer.name.clone(),
                    },
                    ExportAction::PartiallyExported,
                    reasons,
                    &mut diagnostics,
                );
            }
            Err(reasons) => loss(
                ExportDiagnosticSource::Layer {
                    name: layer.name.clone(),
                },
                ExportAction::Skipped,
                reasons,
                &mut diagnostics,
            ),
        }
    }
    if let Some(&layer_id) = layer_ids.get(&document.header.current_layer_name.to_lowercase()) {
        drawing.set_current_layer(layer_id);
    } else if !document.header.current_layer_name.is_empty() {
        loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.current_layer_name".into(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::MissingTarget {
                kind: "layer".into(),
                identifier: document.header.current_layer_name.clone(),
            }],
            &mut diagnostics,
        );
    }
    if document.header.show_model_space {
        drawing.set_active_layout(0);
    } else if let Some(&layout_id) = paper_scopes.get(&document.header.paper_space_block_handle) {
        drawing.set_active_layout(layout_id);
    } else {
        loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.paper_space_block_handle".into(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::MissingTarget {
                kind: "paper layout".into(),
                identifier: document.header.paper_space_block_handle.to_string(),
            }],
            &mut diagnostics,
        );
    }

    if let Some(display) = crate::point_display::direct_from_cad(
        document.header.point_display_mode,
        document.header.point_display_size,
    ) {
        drawing.set_point_display(display)?;
    } else {
        loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.point_display".into(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::UnsupportedHeaderField {
                name: "point_display".into(),
            }],
            &mut diagnostics,
        );
    }
    let mut entity_mapping = BTreeMap::new();
    for entity in ordered_entities(document) {
        let common = entity.common();
        if matches!(entity, EntityType::Block(_) | EntityType::BlockEnd(_))
            && document.block_records.iter().any(|record| {
                record.handle == common.owner_handle
                    && (record.block_entity_handle == common.handle
                        || record.block_end_handle == common.handle)
            })
        {
            continue;
        }
        let source = ExportDiagnosticSource::Entity {
            handle: common.handle,
            kind: entity.as_entity().entity_type().into(),
        };
        if !matches!(
            entity,
            EntityType::Line(_)
                | EntityType::Point(_)
                | EntityType::Circle(_)
                | EntityType::Arc(_)
                | EntityType::Ellipse(_)
                | EntityType::Insert(_)
                | EntityType::LwPolyline(_)
                | EntityType::Polyline2D(_)
                | EntityType::Polyline(_)
                | EntityType::Polyline3D(_)
        ) {
            loss(
                source,
                ExportAction::Skipped,
                vec![ExportLossReason::UnsupportedEntityType {
                    kind: entity.as_entity().entity_type().into(),
                }],
                &mut diagnostics,
            );
            continue;
        }
        let mut reasons = direct_common_losses(common);
        match entity {
            EntityType::Line(line) => reasons.extend(direct_line_losses(line)),
            EntityType::Point(point) => {
                reasons.extend(direct_point_losses(point));
                if point.normal != cadcodec::Vector3::new(0.0, 0.0, 1.0)
                    || point.x_axis_angle != 0.0
                {
                    reasons.push(ExportLossReason::UnsupportedNormal);
                }
            }
            EntityType::Circle(circle) => {
                reasons.extend(direct_circle_losses(circle));
                if circle.normal != cadcodec::Vector3::new(0.0, 0.0, 1.0) {
                    reasons.push(ExportLossReason::UnsupportedNormal);
                }
            }
            EntityType::Arc(arc) => {
                reasons.extend(direct_arc_losses(arc));
                if arc.normal != cadcodec::Vector3::new(0.0, 0.0, 1.0) {
                    reasons.push(ExportLossReason::UnsupportedNormal);
                }
            }
            EntityType::Ellipse(ellipse) => {
                reasons.extend(direct_ellipse_losses(ellipse));
                if ellipse.normal != cadcodec::Vector3::new(0.0, 0.0, 1.0)
                    || ellipse.major_axis.z != 0.0
                {
                    reasons.push(ExportLossReason::UnsupportedNormal);
                }
            }
            EntityType::LwPolyline(polyline) => {
                reasons.extend(direct_planar_polyline_losses(polyline));
                if polyline.normal != cadcodec::Vector3::UNIT_Z
                    || polyline.elevation != 0.0
                    || polyline.constant_width != 0.0
                    || polyline.vertices.iter().any(|vertex| {
                        vertex.start_width != 0.0
                            || vertex.end_width != 0.0
                            || vertex.vertex_id != 0
                    })
                    || (!polyline.is_closed
                        && polyline
                            .vertices
                            .last()
                            .is_some_and(|vertex| vertex.bulge != 0.0))
                {
                    reasons.push(ExportLossReason::UnsupportedSemantic {
                        name: "polyline placement, width, identifiers, or trailing bulge".into(),
                    });
                }
            }
            EntityType::Polyline3D(polyline) => {
                reasons.extend(direct_spatial_polyline_losses(polyline));
            }
            EntityType::Polyline2D(polyline) => {
                reasons.extend(direct_legacy_planar_polyline_losses(polyline));
                if polyline.normal != cadcodec::Vector3::UNIT_Z
                    || polyline.elevation != 0.0
                    || polyline.start_width != 0.0
                    || polyline.end_width != 0.0
                    || polyline
                        .vertices
                        .iter()
                        .any(|vertex| vertex.start_width != 0.0 || vertex.end_width != 0.0)
                    || (!polyline.flags.is_closed()
                        && polyline
                            .vertices
                            .last()
                            .is_some_and(|vertex| vertex.bulge != 0.0))
                {
                    reasons.push(ExportLossReason::UnsupportedSemantic {
                        name: "2D polyline placement, width, or trailing bulge".into(),
                    });
                }
            }
            EntityType::Polyline(polyline) => {
                reasons.extend(direct_generic_spatial_polyline_losses(polyline))
            }
            EntityType::Insert(insert) => {
                if !block_names.contains_key(&insert.block_name.to_lowercase()) {
                    reasons.push(ExportLossReason::MissingTarget {
                        kind: "block definition".into(),
                        identifier: insert.block_name.clone(),
                    });
                }
                if insert.normal != cadcodec::Vector3::new(0.0, 0.0, 1.0)
                    || insert.rotation != 0.0
                    || insert.x_scale() != 1.0
                    || insert.y_scale() != 1.0
                    || insert.z_scale() != 1.0
                    || insert.column_count != 1
                    || insert.row_count != 1
                    || insert.column_spacing != 0.0
                    || insert.row_spacing != 0.0
                    || !insert.attributes.is_empty()
                    || insert.view_rep_handle.is_some()
                {
                    reasons.push(ExportLossReason::UnsupportedSemantic {
                        name: "block instance transform or attachments".into(),
                    });
                }
            }
            _ => unreachable!("filtered entity family"),
        }
        let scope_id = if common.owner_handle == model.block_handle {
            Some(0)
        } else {
            paper_scopes
                .get(&common.owner_handle)
                .or_else(|| block_scopes.get(&common.owner_handle))
                .copied()
        };
        if scope_id.is_none() {
            reasons.push(ExportLossReason::BlockOwnedEntity {
                owner: common.owner_handle,
            });
        }
        let layer_id = layer_ids.get(&common.layer.to_lowercase()).copied();
        if layer_id.is_none() {
            reasons.push(ExportLossReason::MissingEntityLayer {
                name: common.layer.clone(),
            });
        }
        let appearance = match direct_entity(common) {
            Ok(value) => Some(value),
            Err(losses) => {
                reasons.extend(losses);
                None
            }
        };
        if !reasons.is_empty() {
            loss(source, ExportAction::Skipped, reasons, &mut diagnostics);
            continue;
        }
        let layer_id = layer_id.expect("checked");
        let scope_id = scope_id.expect("checked");
        let appearance = appearance.expect("checked");
        let id = match entity {
            EntityType::Line(line) => {
                let mut target = LineDefinition::new(
                    layer_id,
                    [line.start.x, line.start.y, line.start.z],
                    [line.end.x, line.end.y, line.end.z],
                )
                .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_line(target)?
            }
            EntityType::Point(point) => {
                let mut target = PointDefinition::new(
                    layer_id,
                    [point.location.x, point.location.y, point.location.z],
                )
                .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_point(target)?
            }
            EntityType::Circle(circle) => {
                let mut target = CircleDefinition::new(
                    layer_id,
                    [circle.center.x, circle.center.y, circle.center.z],
                    circle.radius,
                )
                .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_circle(target)?
            }
            EntityType::Arc(arc) => {
                let sweep = (arc.end_angle - arc.start_angle).rem_euclid(std::f64::consts::TAU);
                let mut target = ArcDefinition::new(
                    layer_id,
                    [arc.center.x, arc.center.y, arc.center.z],
                    arc.radius,
                    arc.start_angle,
                    sweep,
                )
                .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_arc(target)?
            }
            EntityType::Ellipse(ellipse) => {
                let major = ellipse.major_axis.length();
                let x_axis = [
                    ellipse.major_axis.x / major,
                    ellipse.major_axis.y / major,
                    0.0,
                ];
                let difference = ellipse.end_parameter - ellipse.start_parameter;
                let full = difference == std::f64::consts::TAU;
                let mut target = EllipseDefinition::new(
                    layer_id,
                    [ellipse.center.x, ellipse.center.y, ellipse.center.z],
                    x_axis,
                    major,
                    major * ellipse.minor_axis_ratio,
                )
                .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                if full {
                    drawing.add_ellipse(target)?
                } else {
                    drawing.add_ellipse_arc(target.with_arc(
                        ellipse.start_parameter,
                        difference.rem_euclid(std::f64::consts::TAU),
                    ))?
                }
            }
            EntityType::LwPolyline(polyline) => {
                let vertices = polyline
                    .vertices
                    .iter()
                    .map(|vertex| [vertex.location.x, vertex.location.y, vertex.bulge])
                    .collect();
                let mut target =
                    PlanarPolylineDefinition::new(layer_id, vertices, polyline.is_closed)
                        .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_planar_polyline(target)?
            }
            EntityType::Polyline3D(polyline) => {
                let vertices = polyline
                    .vertices
                    .iter()
                    .map(|vertex| [vertex.position.x, vertex.position.y, vertex.position.z])
                    .collect();
                let mut target =
                    SpatialPolylineDefinition::new(layer_id, vertices, polyline.flags.closed)
                        .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_spatial_polyline(target)?
            }
            EntityType::Polyline2D(polyline) => {
                let vertices = polyline
                    .vertices
                    .iter()
                    .map(|vertex| [vertex.location.x, vertex.location.y, vertex.bulge])
                    .collect();
                let mut target =
                    PlanarPolylineDefinition::new(layer_id, vertices, polyline.flags.is_closed())
                        .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_planar_polyline(target)?
            }
            EntityType::Polyline(polyline) => {
                let vertices = polyline
                    .vertices
                    .iter()
                    .map(|vertex| [vertex.location.x, vertex.location.y, vertex.location.z])
                    .collect();
                let mut target =
                    SpatialPolylineDefinition::new(layer_id, vertices, polyline.flags.is_closed())
                        .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_spatial_polyline(target)?
            }
            EntityType::Insert(insert) => {
                let definition_scope_id = block_names[&insert.block_name.to_lowercase()];
                let mut target = BlockInstanceDefinition::new(
                    layer_id,
                    definition_scope_id,
                    [
                        insert.insert_point.x,
                        insert.insert_point.y,
                        insert.insert_point.z,
                    ],
                )
                .in_scope(scope_id);
                target.visible = !common.invisible;
                target.appearance = appearance;
                drawing.add_block_instance(target)?
            }
            _ => unreachable!("filtered entity family"),
        };
        entity_mapping.insert(common.handle, id);
    }
    if options.loss_policy == ConversionLossPolicy::Reject && !diagnostics.is_empty() {
        return Err(DirectExportError::LossRejected { diagnostics });
    }
    let drawing = drawing.finish()?;
    Ok(DirectExportOutcome {
        drawing,
        diagnostics,
        entity_mapping,
    })
}
