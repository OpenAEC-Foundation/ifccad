//! CadDocument to standalone OCDraw conversion.

use super::appearance::{direct_entity, direct_layer};
use crate::source::{
    direct_arc_losses, direct_circle_losses, direct_common_losses, direct_document_losses,
    direct_ellipse_losses, direct_generic_spatial_polyline_losses,
    direct_legacy_planar_polyline_losses, direct_line_losses, direct_planar_polyline_losses,
    direct_point_losses, direct_spatial_polyline_losses, inspect_markers, inspect_model_space,
    inspect_references, ordered_entities, with_recovered_model_space_handle, ExportAction,
    ExportDiagnostic, ExportDiagnosticSource, ExportLossReason, ExportOptions,
    SourceStructureProblem,
};
use crate::ConversionLossPolicy;
use cadcodec::objects::ObjectType;
use cadcodec::{CadDocument, EntityType, Handle};
use ocdraw::ocdraw::{
    BlockDefinition, CoordinateFrame3, DrawingBuildError, DrawingBuilder, DrawingOptions,
    EncodedDrawing, GeometricEntityDefinition, LayoutRect, LayoutSettings, OcdrawDocument,
    PlotStyleMode, Point3, UcsDefinition, Vector3,
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
    geometry: crate::ConversionGeometryAssessment,
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
    pub fn geometry_assessment(&self) -> &crate::ConversionGeometryAssessment {
        &self.geometry
    }
    pub fn into_drawing(self) -> EncodedDrawing {
        self.drawing
    }
}

pub struct OcdrawDocumentExportOutcome {
    document: OcdrawDocument,
    diagnostics: Vec<ExportDiagnostic>,
    entity_mapping: BTreeMap<Handle, u64>,
    geometry: crate::ConversionGeometryAssessment,
}

impl OcdrawDocumentExportOutcome {
    pub fn document(&self) -> &OcdrawDocument {
        &self.document
    }
    pub fn diagnostics(&self) -> &[ExportDiagnostic] {
        &self.diagnostics
    }
    pub fn entity_mapping(&self) -> &BTreeMap<Handle, u64> {
        &self.entity_mapping
    }
    pub fn geometry_assessment(&self) -> &crate::ConversionGeometryAssessment {
        &self.geometry
    }
    pub fn into_document(self) -> OcdrawDocument {
        self.document
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DirectExportError {
    #[error(transparent)]
    GeometryTolerance(#[from] crate::ConversionToleranceError),
    #[error("geometric accuracy requirement failed: {0:?}")]
    Geometry(Box<crate::ConversionGeometryFailure>),
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

pub fn cad_document_to_ocdraw_document(
    document: &CadDocument,
    options: ExportOptions,
) -> Result<OcdrawDocumentExportOutcome, DirectExportError> {
    let drawing_id = if document.header.fingerprint_guid.is_empty() {
        "drawing-main"
    } else {
        &document.header.fingerprint_guid
    };
    cad_document_to_ocdraw_document_with_id(document, drawing_id, options)
}

pub fn cad_document_to_ocdraw_document_with_id(
    document: &CadDocument,
    drawing_id: &str,
    options: ExportOptions,
) -> Result<OcdrawDocumentExportOutcome, DirectExportError> {
    let recovered = with_recovered_model_space_handle(document);
    let document = recovered.as_ref();
    let model = inspect_model_space(document)
        .map_err(|problems| DirectExportError::InvalidSourceStructure { problems })?;
    let mut problems = inspect_markers(document);
    problems.extend(inspect_references(document));
    if !problems.is_empty() {
        return Err(DirectExportError::InvalidSourceStructure { problems });
    }
    let mut diagnostics = Vec::new();
    let mut geometry =
        super::geometry::ExchangeState::new(crate::ConversionGeometryAssessment::new(
            options.geometry_tolerance,
            crate::units::from_cad_code(document.header.insertion_units)
                .unwrap_or(ocdraw::ocdraw::DrawingLengthUnit::Unitless),
        )?);
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
    let patterns =
        super::line_pattern::export_line_patterns(document, &mut drawing, &mut diagnostics)?;
    let mut ucs_names = BTreeMap::new();
    let mut ucs_handles = BTreeMap::new();
    for source in document.ucss.iter() {
        let frame = CoordinateFrame3::try_new(
            Point3::new(source.origin.x, source.origin.y, source.origin.z),
            Vector3::new(source.x_axis.x, source.x_axis.y, source.x_axis.z),
            Vector3::new(source.y_axis.x, source.y_axis.y, source.y_axis.z),
        );
        if let Ok(frame) = frame {
            let id = drawing.add_ucs_definition(UcsDefinition::new(
                &source.name,
                frame,
                source.elevation,
            ))?;
            ucs_names.insert(source.name.to_lowercase(), id);
            ucs_handles.insert(source.handle, id);
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
    let mut layouts = document
        .objects
        .values()
        .filter_map(|object| match object {
            ObjectType::Layout(layout) => Some(layout),
            _ => None,
        })
        .collect::<Vec<_>>();
    layouts.sort_by_key(|layout| (layout.tab_order, layout.name.as_str()));
    for layout in layouts {
        if crate::source::is_untouched_scaffold(layout, document) {
            continue;
        }
        for field in
            super::layout::unrepresented_fields(layout, layout.block_record == model.block_handle)
        {
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
            plot_settings: match super::layout::plot_from_cad(layout, id == 0) {
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
    let mut block_scopes = BTreeMap::new();
    let mut block_names = BTreeMap::new();
    for record in document.block_records.iter() {
        if record.handle == model.block_handle
            || paper_scopes.contains_key(&record.handle)
            || crate::source::is_empty_reserved_paper_block(record, document)
        {
            continue;
        }
        if record.layout != Handle::NULL
            || record.flags.is_xref
            || record.flags.is_xref_overlay
            || record.flags.is_external
            || record.flags.is_xref_unloaded
            || !record.xref_path.is_empty()
        {
            continue;
        }
        let mut block_losses = Vec::new();
        if !record.preview_data.is_empty()
            || !record.insert_count_bytes.is_empty()
            || record.flags.has_attributes
        {
            block_losses.push(ExportLossReason::UnsupportedSemantic {
                name: "block attribute flag, preview or insertion-count metadata".into(),
            });
        }
        if crate::units::from_cad_code(record.units).is_none() {
            block_losses.push(ExportLossReason::UnsupportedUnit { code: record.units });
        }
        loss(
            ExportDiagnosticSource::Object {
                handle: record.handle,
                kind: "BLOCK_RECORD".into(),
            },
            ExportAction::PartiallyExported,
            block_losses,
            &mut diagnostics,
        );
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
        match direct_layer(layer, patterns.layer(&layer.line_type)?) {
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

    if let Some(display) = super::point_display::direct_from_cad(
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
    let mapped_vports = super::workspace::export(
        document,
        &mut drawing,
        &paper_scopes,
        &ucs_names,
        &ucs_handles,
        &mut diagnostics,
    );
    diagnostics.extend(direct_document_losses(
        document,
        &mapped_vports,
        paper_scopes.keys().chain(block_scopes.keys()).copied(),
    ));
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
        if matches!(entity,EntityType::Viewport(v) if v.id==1 && document.objects.values().any(|object|matches!(object,ObjectType::Layout(layout) if crate::source::overall_viewport_handle(document,layout)==Some(common.handle))))
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
                | EntityType::Viewport(_)
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
        let mut partial = direct_common_losses(common);
        let mut reasons = match entity {
            EntityType::Line(value) => direct_line_losses(value),
            EntityType::Point(value) => direct_point_losses(value),
            EntityType::Circle(value) => direct_circle_losses(value),
            EntityType::Arc(value) => direct_arc_losses(value),
            EntityType::Ellipse(value) => direct_ellipse_losses(value),
            EntityType::LwPolyline(value) => {
                if value.constant_width != 0.0
                    || value
                        .vertices
                        .iter()
                        .any(|v| v.start_width != 0.0 || v.end_width != 0.0)
                {
                    partial.push(ExportLossReason::PolylineWidth);
                }
                let count = value.vertices.iter().filter(|v| v.vertex_id != 0).count();
                if count > 0 {
                    partial.push(ExportLossReason::PolylineVertexIdentifiers { count });
                }
                direct_planar_polyline_losses(value)
            }
            EntityType::Polyline2D(value) => {
                if value.start_width != 0.0
                    || value.end_width != 0.0
                    || value
                        .vertices
                        .iter()
                        .any(|v| v.start_width != 0.0 || v.end_width != 0.0)
                {
                    partial.push(ExportLossReason::PolylineWidth);
                }
                let count = value.vertices.iter().filter(|v| v.id != 0).count();
                if count > 0 {
                    partial.push(ExportLossReason::PolylineVertexIdentifiers { count });
                }
                direct_legacy_planar_polyline_losses(value)
            }
            EntityType::Polyline3D(value) => direct_spatial_polyline_losses(value),
            EntityType::Polyline(value) => direct_generic_spatial_polyline_losses(value),
            EntityType::Viewport(value) => {
                if !paper_scopes.contains_key(&common.owner_handle) {
                    vec![ExportLossReason::UnsupportedSemantic {
                        name: "viewport owner is not paper space".into(),
                    }]
                } else {
                    super::viewport::losses(value, document, &entity_mapping)
                }
            }
            EntityType::Insert(value) => {
                let mut losses = Vec::new();
                if !block_names.contains_key(&value.block_name.to_lowercase()) {
                    losses.push(ExportLossReason::MissingTarget {
                        kind: "block definition".into(),
                        identifier: value.block_name.clone(),
                    });
                }
                if value.is_minsert()
                    || value.column_count != 1
                    || value.row_count != 1
                    || value.column_spacing != 0.0
                    || value.row_spacing != 0.0
                    || !value.attributes.is_empty()
                    || value.view_rep_handle.is_some()
                {
                    losses.push(ExportLossReason::UnsupportedSemantic {
                        name: "block instance array or attachments".into(),
                    });
                }
                losses
            }
            _ => unreachable!("filtered entity family"),
        };
        let scope_id = if common.owner_handle == model.block_handle {
            Some(0)
        } else {
            paper_scopes
                .get(&common.owner_handle)
                .or_else(|| block_scopes.get(&common.owner_handle))
                .copied()
        };
        if scope_id.is_none() {
            if !document
                .block_records
                .iter()
                .any(|record| record.handle == common.owner_handle)
            {
                let problem = if common.owner_handle == Handle::NULL {
                    SourceStructureProblem::EntityOwnerMissing {
                        entity: common.handle,
                    }
                } else {
                    SourceStructureProblem::EntityOwnerUnknown {
                        entity: common.handle,
                        owner: common.owner_handle,
                    }
                };
                return Err(DirectExportError::InvalidSourceStructure {
                    problems: vec![problem],
                });
            }
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
        let appearance = match direct_entity(
            common,
            patterns.resolve(&common.linetype, common.linetype_handle)?,
        ) {
            Ok(value) => Some(value),
            Err(losses) => {
                reasons.extend(losses);
                None
            }
        };
        if !reasons.is_empty() {
            reasons.extend(partial);
            loss(source, ExportAction::Skipped, reasons, &mut diagnostics);
            continue;
        }
        let layer_id = layer_id.expect("checked");
        let scope_id = scope_id.expect("checked");
        let appearance = appearance.expect("checked");
        let id = if let EntityType::Viewport(viewport) = entity {
            partial.extend(super::viewport::deferred_losses(viewport));
            let target = super::viewport::from_cad(
                viewport,
                scope_id,
                layer_id,
                appearance,
                &super::viewport::SourceIndex {
                    document,
                    layers: &layer_ids,
                    entities: &entity_mapping,
                },
                &mut diagnostics,
            );
            drawing.add_viewport(target)?
        } else {
            let (value, bound, normalized) =
                super::geometry::from_cad(entity, &block_scopes, document, &mut geometry)?;
            if bound > 0.0 {
                partial.push(ExportLossReason::GeometryRoundedWithinTolerance {
                    max_deviation_upper_bound: bound,
                });
            }
            if normalized {
                partial.push(ExportLossReason::SourceNormalNormalized);
            }
            if matches!(entity,EntityType::Line(line) if line.normal!=cadcodec::Vector3::UNIT_Z) {
                partial.push(ExportLossReason::UnsupportedNormal);
            }
            let mut target = GeometricEntityDefinition::new(scope_id, layer_id, value);
            target.visible = !common.invisible;
            target.appearance = appearance;
            drawing.add_geometric_entity(target)?
        };
        loss(
            source,
            ExportAction::PartiallyExported,
            partial,
            &mut diagnostics,
        );
        entity_mapping.insert(common.handle, id);
    }
    let members = document
        .block_records
        .iter()
        .map(|record| (record.handle, record.entity_handles.clone()))
        .collect();
    for (handle, bound) in geometry.assess_occurrences(&members)? {
        loss(
            ExportDiagnosticSource::Entity {
                handle,
                kind: "INSERT".into(),
            },
            ExportAction::PartiallyExported,
            vec![ExportLossReason::GeometryRoundedWithinTolerance {
                max_deviation_upper_bound: bound,
            }],
            &mut diagnostics,
        );
    }
    let affected = diagnostics
        .iter()
        .filter(|d| d.blocks_reject())
        .filter_map(|diagnostic| match diagnostic.source() {
            ExportDiagnosticSource::Entity { handle, .. } => document
                .get_entity(*handle)
                .map(|entity| entity.common().owner_handle),
            ExportDiagnosticSource::Object { handle, kind } if kind == "BLOCK_RECORD" => {
                Some(*handle)
            }
            _ => None,
        })
        .filter(|owner| block_scopes.contains_key(owner))
        .collect();
    for (definition, instances) in geometry.affected_instances(&members, &affected) {
        loss(
            ExportDiagnosticSource::Object {
                handle: definition,
                kind: "BLOCK_RECORD".into(),
            },
            ExportAction::PartiallyExported,
            vec![ExportLossReason::BlockContentLoss {
                definition,
                affected_instances: instances,
            }],
            &mut diagnostics,
        );
    }
    if options.loss_policy == ConversionLossPolicy::Reject
        && diagnostics.iter().any(ExportDiagnostic::blocks_reject)
    {
        return Err(DirectExportError::LossRejected { diagnostics });
    }
    let document = drawing.build_document()?;
    Ok(OcdrawDocumentExportOutcome {
        document,
        diagnostics,
        entity_mapping,
        geometry: geometry.assessment,
    })
}

impl From<Box<crate::ConversionGeometryFailure>> for DirectExportError {
    fn from(failure: Box<crate::ConversionGeometryFailure>) -> Self {
        Self::Geometry(failure)
    }
}

/// Converts a fresh CAD source and encodes the resulting logical drawing.
pub fn cad_document_to_drawing(
    document: &CadDocument,
    options: ExportOptions,
) -> Result<DirectExportOutcome, DirectExportError> {
    encode_export(cad_document_to_ocdraw_document(document, options)?)
}
pub fn cad_document_to_drawing_with_id(
    document: &CadDocument,
    drawing_id: &str,
    options: ExportOptions,
) -> Result<DirectExportOutcome, DirectExportError> {
    encode_export(cad_document_to_ocdraw_document_with_id(
        document, drawing_id, options,
    )?)
}
fn encode_export(
    outcome: OcdrawDocumentExportOutcome,
) -> Result<DirectExportOutcome, DirectExportError> {
    let drawing =
        ocdraw::ocdraw::encode_document(&outcome.document).map_err(DrawingBuildError::from)?;
    Ok(DirectExportOutcome {
        drawing,
        diagnostics: outcome.diagnostics,
        entity_mapping: outcome.entity_mapping,
        geometry: outcome.geometry,
    })
}
