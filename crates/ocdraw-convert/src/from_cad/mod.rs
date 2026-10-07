//! CadDocument to standalone OCDraw conversion.
mod prepared_entities;
use prepared_entities::{PreparedCadEntity, PreparedCadEntityValue};

use crate::mapping::appearance::{direct_entity, direct_layer};
use crate::source::{
    direct_arc_losses, direct_circle_losses, direct_common_losses, direct_document_losses,
    direct_ellipse_losses, direct_generic_spatial_polyline_losses,
    direct_legacy_planar_polyline_losses, direct_line_losses, direct_planar_polyline_losses,
    direct_point_losses, direct_spatial_polyline_losses, inspect_markers, inspect_model_space,
    inspect_references, ordered_entities, with_recovered_model_space_handle,
    CadSourceStructureProblem, CadToOcdrawAction, CadToOcdrawDiagnostic,
    CadToOcdrawDiagnosticSource, CadToOcdrawLossReason, CadToOcdrawOptions,
};
use crate::units::UNIT_TOKENS;
use crate::{
    CadToEncodedOcdrawOutcome, CadToOcdrawDocumentOutcome, CadToOcdrawError, OcdrawLossPolicy,
};
use ocdraw::ocdraw::{
    BlockDefinition, CoordinateFrame3, GeometricEntityDefinition, LayoutRect, LayoutSettings,
    OcdrawBuildError, OcdrawBuildOptions, OcdrawBuilder, PlotStyleMode, Point3, UcsDefinition,
    Vector3,
};
use opencadcodec::objects::ObjectType;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::BTreeMap;

fn loss(
    source: CadToOcdrawDiagnosticSource,
    action: CadToOcdrawAction,
    reasons: Vec<CadToOcdrawLossReason>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) {
    if !reasons.is_empty() {
        diagnostics.push(CadToOcdrawDiagnostic::loss(source, action, reasons));
    }
}

fn resolved_layer_name<'a>(
    document: &'a CadDocument,
    common: &opencadcodec::entities::EntityCommon,
) -> Option<&'a str> {
    let layer = if let Some(handle) = common.layer_handle.filter(|h| !h.is_null()) {
        document
            .layers
            .iter()
            .find(|layer| layer.handle == handle)?
    } else {
        document.layers.get(&common.layer)?
    };
    layer
        .name
        .eq_ignore_ascii_case(&common.layer)
        .then_some(layer.name.as_str())
}

pub fn cad_document_to_ocdraw_document(
    document: &CadDocument,
    options: CadToOcdrawOptions,
) -> Result<CadToOcdrawDocumentOutcome, CadToOcdrawError> {
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
    options: CadToOcdrawOptions,
) -> Result<CadToOcdrawDocumentOutcome, CadToOcdrawError> {
    let recovered = with_recovered_model_space_handle(document);
    let document = recovered.as_ref();
    let model = inspect_model_space(document)
        .map_err(|problems| CadToOcdrawError::InvalidSourceStructure { problems })?;
    let mut problems = inspect_markers(document);
    problems.extend(inspect_references(document));
    if !problems.is_empty() {
        return Err(CadToOcdrawError::InvalidSourceStructure { problems });
    }
    let mut diagnostics = Vec::new();
    let mut preservation_report = crate::OcdrawPreservationReport::default();
    let mut preservation = None;
    let mut geometry =
        crate::mapping::geometry::ExchangeState::new(crate::OcdrawGeometryAssessment::new(
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
            CadToOcdrawDiagnosticSource::DocumentField {
                name: "header.insertion_units".into(),
            },
            CadToOcdrawAction::PartiallyExported,
            vec![CadToOcdrawLossReason::UnsupportedUnit {
                code: document.header.insertion_units,
            }],
            &mut diagnostics,
        );
    }
    let mut drawing = OcdrawBuilder::new(OcdrawBuildOptions::new(
        drawing_id,
        unit.unwrap_or("unitless"),
    ))?;
    let patterns = crate::mapping::line_pattern::export_line_patterns(
        document,
        &mut drawing,
        &mut diagnostics,
    )?;
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
                CadToOcdrawDiagnosticSource::DocumentField {
                    name: format!("ucs.{}", source.name),
                },
                CadToOcdrawAction::Skipped,
                vec![CadToOcdrawLossReason::UnsupportedSemantic {
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
                CadToOcdrawDiagnosticSource::DocumentField {
                    name: format!("ucs.{}", source.name),
                },
                CadToOcdrawAction::PartiallyExported,
                vec![CadToOcdrawLossReason::UnsupportedSemantic {
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
        for field in crate::mapping::layout::unrepresented_fields(
            layout,
            layout.block_record == model.block_handle,
        ) {
            loss(
                CadToOcdrawDiagnosticSource::DocumentField {
                    name: format!("layout.{}.{}", layout.name, field),
                },
                CadToOcdrawAction::PartiallyExported,
                vec![CadToOcdrawLossReason::UnsupportedSemantic { name: field.into() }],
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
            plot_settings: match crate::mapping::layout::plot_from_cad(layout, id == 0) {
                Ok(settings) => settings,
                Err(reason) => {
                    loss(
                        CadToOcdrawDiagnosticSource::DocumentField {
                            name: format!("layout.{}.plot", layout.name),
                        },
                        CadToOcdrawAction::PartiallyExported,
                        vec![CadToOcdrawLossReason::UnsupportedSemantic {
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
                CadToOcdrawDiagnosticSource::DocumentField {
                    name: format!("layout.{}.limits", layout.name),
                },
                CadToOcdrawAction::PartiallyExported,
                vec![CadToOcdrawLossReason::UnsupportedSemantic {
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
            block_losses.push(CadToOcdrawLossReason::UnsupportedSemantic {
                name: "block attribute flag, preview or insertion-count metadata".into(),
            });
        }
        if crate::units::from_cad_code(record.units).is_none() {
            block_losses.push(CadToOcdrawLossReason::UnsupportedUnit { code: record.units });
        }
        loss(
            CadToOcdrawDiagnosticSource::Object {
                handle: record.handle,
                kind: "BLOCK_RECORD".into(),
            },
            CadToOcdrawAction::PartiallyExported,
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
                    CadToOcdrawDiagnosticSource::Layer {
                        name: layer.name.clone(),
                    },
                    CadToOcdrawAction::PartiallyExported,
                    reasons,
                    &mut diagnostics,
                );
            }
            Err(reasons) => loss(
                CadToOcdrawDiagnosticSource::Layer {
                    name: layer.name.clone(),
                },
                CadToOcdrawAction::Skipped,
                reasons,
                &mut diagnostics,
            ),
        }
    }
    if let Some(&layer_id) = layer_ids.get(&document.header.current_layer_name.to_lowercase()) {
        drawing.set_current_layer(layer_id);
    } else if !document.header.current_layer_name.is_empty() {
        loss(
            CadToOcdrawDiagnosticSource::DocumentField {
                name: "header.current_layer_name".into(),
            },
            CadToOcdrawAction::Skipped,
            vec![CadToOcdrawLossReason::MissingTarget {
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
            CadToOcdrawDiagnosticSource::DocumentField {
                name: "header.paper_space_block_handle".into(),
            },
            CadToOcdrawAction::Skipped,
            vec![CadToOcdrawLossReason::MissingTarget {
                kind: "paper layout".into(),
                identifier: document.header.paper_space_block_handle.to_string(),
            }],
            &mut diagnostics,
        );
    }

    if let Some(display) = crate::mapping::point_display::direct_from_cad(
        document.header.point_display_mode,
        document.header.point_display_size,
    ) {
        drawing.set_point_display(display)?;
    } else {
        loss(
            CadToOcdrawDiagnosticSource::DocumentField {
                name: "header.point_display".into(),
            },
            CadToOcdrawAction::Skipped,
            vec![CadToOcdrawLossReason::UnsupportedHeaderField {
                name: "point_display".into(),
            }],
            &mut diagnostics,
        );
    }
    let mapped_vports = crate::mapping::workspace::export(
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
    let mut prepared_entities = Vec::new();
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
        let source = CadToOcdrawDiagnosticSource::Entity {
            handle: common.handle,
            kind: entity.as_entity().entity_type().into(),
        };
        if let EntityType::Spline(spline) = entity {
            if options.preservation_capture == crate::OcdrawPreservationCapture::SupportedTyped {
                use crate::preservation::{
                    capture_spline, spline_entry, unsupported_common_context, CODEC_REVISION,
                };
                use ocdraw::ocdraw::*;
                let owner = document
                    .block_records
                    .iter()
                    .find(|r| r.handle == common.owner_handle)
                    .ok_or_else(|| CadToOcdrawError::InvalidSourceStructure {
                        problems: vec![if common.owner_handle == Handle::NULL {
                            CadSourceStructureProblem::EntityOwnerMissing {
                                entity: common.handle,
                            }
                        } else {
                            CadSourceStructureProblem::EntityOwnerUnknown {
                                entity: common.handle,
                                owner: common.owner_handle,
                            }
                        }],
                    })?;
                let order_index = owner
                    .entity_handles
                    .iter()
                    .position(|h| *h == common.handle)
                    .ok_or_else(|| CadToOcdrawError::InvalidSourceStructure {
                        problems: vec![CadSourceStructureProblem::InconsistentRelationship {
                            description: "spline is missing from authoritative owner order".into(),
                        }],
                    })?;
                let payload = capture_spline(
                    spline,
                    u64::try_from(order_index).map_err(|_| OcdrawBuildError::IdExhausted)?,
                    CODEC_REVISION,
                )?;
                let p = preservation.get_or_insert_with(|| OcdrawPreservation {
                    version: 1,
                    next_record_id: 1,
                    sources: vec![OcdrawPreservationSource {
                        id: "cad-source-1".into(),
                        provider: "opencadcodec".into(),
                        provider_revision: CODEC_REVISION.into(),
                        origin: OcdrawPreservationOrigin::CadDocument,
                        source_version: None,
                    }],
                    records: vec![],
                });
                let record_id = p
                    .allocate_record_id()
                    .map_err(|_| OcdrawBuildError::IdExhausted)?;
                let key = format!("{:x}", common.handle);
                p.records.push(OcdrawPreservationRecord {
                    id: record_id,
                    source_id: "cad-source-1".into(),
                    source_key: key.clone(),
                    category: OcdrawPreservationCategory::Entity,
                    role: OcdrawPreservationRole::Complete,
                    representation: OcdrawPreservationRepresentation::CodecTyped,
                    subject: None,
                    dependency_coverage: OcdrawPreservationDependencyCoverage::Conservative,
                    bindings: vec![],
                    conditions: vec![],
                    payload: OcdrawPreservationPayload {
                        schema: "openaec.opencadcodec.spline".into(),
                        version: crate::preservation::SPLINE_PAYLOAD_VERSION,
                        kind: OcdrawPreservationPayloadKind::AdapterSnapshot,
                        bytes: payload,
                    },
                });
                preservation_report.entries.push(spline_entry(
                    record_id,
                    key.clone(),
                    crate::OcdrawPreservationResult::CapturedTyped,
                    format!("/preservation/records/{}", p.records.len() - 1),
                    "complete interpreted spline source captured",
                ));
                if common.raw_record.is_some() {
                    preservation_report.entries.push(spline_entry(
                        record_id,
                        key.clone(),
                        crate::OcdrawPreservationResult::StorageSupplementOmitted,
                        format!("/source/entities/{key}/rawRecord"),
                        "original entity storage record is outside the typed snapshot",
                    ));
                }
                if unsupported_common_context(spline) {
                    let mut entry = spline_entry(
                        record_id,
                        key.clone(),
                        crate::OcdrawPreservationResult::RestorationUnavailable,
                        format!("/source/entities/{key}/common"),
                        "retained attached or raw context has no qualified restoring provider",
                    );
                    entry.reason = Some(crate::OcdrawPreservationReason::UnsupportedContext);
                    preservation_report.entries.push(entry);
                }
                let (scope, kind) = if common.owner_handle == model.block_handle {
                    (Some(0), DrawingScopeKind::Model)
                } else if let Some(scope) = paper_scopes.get(&common.owner_handle) {
                    (Some(*scope), DrawingScopeKind::Paper)
                } else {
                    (
                        block_scopes.get(&common.owner_handle).copied(),
                        DrawingScopeKind::Block,
                    )
                };
                if let Some(scope_id) = scope {
                    let layer_id = resolved_layer_name(document, common)
                        .and_then(|name| layer_ids.get(&name.to_lowercase()).copied());
                    // Resolve structural name/handle contradictions even when another common value remains opaque.
                    let pattern =
                        patterns.resolve_preserved(&common.linetype, common.linetype_handle)?;
                    let appearance =
                        if matches!(common.line_weight, opencadcodec::LineWeight::Default)
                            || !common.linetype_scale.is_finite()
                            || common.linetype_scale <= 0.
                        {
                            None
                        } else {
                            pattern.and_then(|pattern| direct_entity(common, pattern).ok())
                        };
                    for (name, represented) in [
                        ("layer", layer_id.is_some()),
                        ("appearance", appearance.is_some()),
                    ] {
                        if !represented {
                            preservation_report.entries.push(spline_entry(
                                record_id,
                                key.clone(),
                                crate::OcdrawPreservationResult::NativePropertyUnrepresented,
                                format!("/source/entities/{key}/{name}"),
                                format!("source {name} remains in snapshot"),
                            ));
                        }
                    }
                    prepared_entities.push(PreparedCadEntity {
                        handle: common.handle,
                        value: PreparedCadEntityValue::Opaque {
                            definition: OpaqueEntityDefinition {
                                scope_id,
                                preservation_record_id: record_id,
                                layer_id,
                                appearance,
                                visible: !common.invisible,
                            },
                            kind,
                        },
                    });
                } else {
                    loss(
                        source,
                        CadToOcdrawAction::Skipped,
                        vec![CadToOcdrawLossReason::BlockOwnedEntity {
                            owner: common.owner_handle,
                        }],
                        &mut diagnostics,
                    );
                }
                continue;
            }
        }
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
                CadToOcdrawAction::Skipped,
                vec![CadToOcdrawLossReason::UnsupportedEntityType {
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
                    partial.push(CadToOcdrawLossReason::PolylineWidth);
                }
                let count = value.vertices.iter().filter(|v| v.vertex_id != 0).count();
                if count > 0 {
                    partial.push(CadToOcdrawLossReason::PolylineVertexIdentifiers { count });
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
                    partial.push(CadToOcdrawLossReason::PolylineWidth);
                }
                let count = value.vertices.iter().filter(|v| v.id != 0).count();
                if count > 0 {
                    partial.push(CadToOcdrawLossReason::PolylineVertexIdentifiers { count });
                }
                direct_legacy_planar_polyline_losses(value)
            }
            EntityType::Polyline3D(value) => direct_spatial_polyline_losses(value),
            EntityType::Polyline(value) => direct_generic_spatial_polyline_losses(value),
            EntityType::Viewport(value) => {
                if !paper_scopes.contains_key(&common.owner_handle) {
                    vec![CadToOcdrawLossReason::UnsupportedSemantic {
                        name: "viewport owner is not paper space".into(),
                    }]
                } else {
                    crate::mapping::viewport::losses(value)
                }
            }
            EntityType::Insert(value) => {
                let mut losses = Vec::new();
                if !block_names.contains_key(&value.block_name.to_lowercase()) {
                    losses.push(CadToOcdrawLossReason::MissingTarget {
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
                    losses.push(CadToOcdrawLossReason::UnsupportedSemantic {
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
                    CadSourceStructureProblem::EntityOwnerMissing {
                        entity: common.handle,
                    }
                } else {
                    CadSourceStructureProblem::EntityOwnerUnknown {
                        entity: common.handle,
                        owner: common.owner_handle,
                    }
                };
                return Err(CadToOcdrawError::InvalidSourceStructure {
                    problems: vec![problem],
                });
            }
            reasons.push(CadToOcdrawLossReason::BlockOwnedEntity {
                owner: common.owner_handle,
            });
        }
        let layer_id = resolved_layer_name(document, common)
            .and_then(|name| layer_ids.get(&name.to_lowercase()).copied());
        if layer_id.is_none() {
            reasons.push(CadToOcdrawLossReason::MissingEntityLayer {
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
            loss(
                source,
                CadToOcdrawAction::Skipped,
                reasons,
                &mut diagnostics,
            );
            continue;
        }
        let layer_id = layer_id.expect("checked");
        let scope_id = scope_id.expect("checked");
        let appearance = appearance.expect("checked");
        let value = if let EntityType::Viewport(viewport) = entity {
            partial.extend(crate::mapping::viewport::deferred_losses(viewport));
            let target = crate::mapping::viewport::from_cad(
                viewport,
                scope_id,
                layer_id,
                appearance,
                &crate::mapping::viewport::SourceIndex {
                    document,
                    layers: &layer_ids,
                },
                &mut diagnostics,
            );
            PreparedCadEntityValue::Viewport {
                definition: target,
                boundary: viewport.clip_boundary_handle,
            }
        } else {
            let (value, bound, normalized) =
                crate::mapping::geometry::from_cad(entity, &block_scopes, document, &mut geometry)?;
            if bound > 0.0 {
                partial.push(CadToOcdrawLossReason::GeometryRoundedWithinTolerance {
                    max_deviation_upper_bound: bound,
                });
            }
            if normalized {
                partial.push(CadToOcdrawLossReason::SourceNormalNormalized);
            }
            if matches!(entity,EntityType::Line(line) if line.normal!=opencadcodec::Vector3::UNIT_Z)
            {
                partial.push(CadToOcdrawLossReason::UnsupportedNormal);
            }
            let mut target = GeometricEntityDefinition::new(scope_id, layer_id, value);
            target.visible = !common.invisible;
            target.appearance = appearance;
            PreparedCadEntityValue::Geometry(target)
        };
        loss(
            source,
            CadToOcdrawAction::PartiallyExported,
            partial,
            &mut diagnostics,
        );
        prepared_entities.push(PreparedCadEntity {
            handle: common.handle,
            value,
        });
    }
    let (mut drawing_document, entity_mapping) = prepared_entities::append(
        drawing,
        prepared_entities,
        document,
        &mut diagnostics,
        preservation,
        unit.unwrap_or("unitless"),
    )?;
    crate::preservation::record_unassessed_occurrences(&drawing_document, &mut geometry.assessment);
    crate::preservation::bind_source_references(
        document,
        &mut drawing_document,
        &entity_mapping,
        &mut preservation_report,
    );
    ocdraw::ocdraw::validate_ocdraw_document(&drawing_document)
        .map_err(|e| OcdrawBuildError::Invalid(format!("{:?}", e.diagnostics())))?;
    for entry in &mut preservation_report.entries {
        if let Ok(handle) = u64::from_str_radix(&entry.source_key, 16) {
            entry.entity_id = entity_mapping.get(&Handle::new(handle)).copied();
        }
    }
    let members = document
        .block_records
        .iter()
        .map(|record| (record.handle, record.entity_handles.clone()))
        .collect();
    for (handle, bound) in geometry.assess_occurrences(&members)? {
        loss(
            CadToOcdrawDiagnosticSource::Entity {
                handle,
                kind: "INSERT".into(),
            },
            CadToOcdrawAction::PartiallyExported,
            vec![CadToOcdrawLossReason::GeometryRoundedWithinTolerance {
                max_deviation_upper_bound: bound,
            }],
            &mut diagnostics,
        );
    }
    let affected = diagnostics
        .iter()
        .filter(|d| d.blocks_reject())
        .filter_map(|diagnostic| match diagnostic.source() {
            CadToOcdrawDiagnosticSource::Entity { handle, .. } => document
                .get_entity(*handle)
                .map(|entity| entity.common().owner_handle),
            CadToOcdrawDiagnosticSource::Object { handle, kind } if kind == "BLOCK_RECORD" => {
                Some(*handle)
            }
            _ => None,
        })
        .filter(|owner| block_scopes.contains_key(owner))
        .collect();
    for (definition, instances) in geometry.affected_instances(&members, &affected) {
        loss(
            CadToOcdrawDiagnosticSource::Object {
                handle: definition,
                kind: "BLOCK_RECORD".into(),
            },
            CadToOcdrawAction::PartiallyExported,
            vec![CadToOcdrawLossReason::BlockContentLoss {
                definition,
                affected_instances: instances,
            }],
            &mut diagnostics,
        );
    }
    if options.loss_policy == OcdrawLossPolicy::Reject
        && diagnostics.iter().any(CadToOcdrawDiagnostic::blocks_reject)
    {
        return Err(CadToOcdrawError::LossRejected {
            diagnostics,
            preservation: preservation_report,
        });
    }
    Ok(CadToOcdrawDocumentOutcome {
        preservation: preservation_report,
        document: drawing_document,
        diagnostics,
        entity_mapping,
        geometry: geometry.assessment,
    })
}

impl From<Box<crate::OcdrawGeometryFailure>> for CadToOcdrawError {
    fn from(failure: Box<crate::OcdrawGeometryFailure>) -> Self {
        Self::Geometry(failure)
    }
}

/// Converts a fresh CAD source and encodes the resulting logical drawing.
pub fn cad_document_to_encoded_ocdraw(
    document: &CadDocument,
    options: CadToOcdrawOptions,
) -> Result<CadToEncodedOcdrawOutcome, CadToOcdrawError> {
    encode_export(cad_document_to_ocdraw_document(document, options)?)
}
pub fn cad_document_to_encoded_ocdraw_with_id(
    document: &CadDocument,
    drawing_id: &str,
    options: CadToOcdrawOptions,
) -> Result<CadToEncodedOcdrawOutcome, CadToOcdrawError> {
    encode_export(cad_document_to_ocdraw_document_with_id(
        document, drawing_id, options,
    )?)
}
fn encode_export(
    outcome: CadToOcdrawDocumentOutcome,
) -> Result<CadToEncodedOcdrawOutcome, CadToOcdrawError> {
    let drawing = ocdraw::ocdraw::encode_ocdraw_document(&outcome.document)
        .map_err(OcdrawBuildError::from)?;
    Ok(CadToEncodedOcdrawOutcome {
        preservation: outcome.preservation,
        encoded: drawing,
        diagnostics: outcome.diagnostics,
        entity_mapping: outcome.entity_mapping,
        geometry: outcome.geometry,
    })
}
