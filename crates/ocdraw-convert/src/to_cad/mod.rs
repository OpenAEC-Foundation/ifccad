//! Direct standalone drawing to CadDocument conversion.

mod entities;
mod layouts;

use crate::{
    OcdrawLossPolicy, OcdrawToCadDiagnostic, OcdrawToCadError, OcdrawToCadOptions,
    OcdrawToCadOutcome,
};
use ocdraw::ocdraw::{
    DrawingLayoutKind, OcdrawDocument, PlotOffsetReference, PlotPlacement, PlotStyleMode,
};
use opencadcodec::objects::ObjectType;
use opencadcodec::{BlockRecord, CadDocument, EntityType, Layer, LineWeight, Transparency};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn diagnostic(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> OcdrawToCadDiagnostic {
    OcdrawToCadDiagnostic {
        code,
        location: location.into(),
        message: message.into(),
    }
}

fn line_weight(
    value: f64,
    location: &str,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> LineWeight {
    let hundredths = (value * 100.0).round();
    let bounded = hundredths.clamp(0.0, i16::MAX as f64) as i16;
    if f64::from(bounded) / 100.0 != value {
        diagnostics.push(diagnostic(
            "LINE_WEIGHT_ROUNDED",
            location,
            "line weight was rounded to CAD hundredths of a millimetre",
        ));
    }
    LineWeight::from_value(bounded)
}

pub fn ocdraw_document_to_cad_document(
    drawing: &OcdrawDocument,
    options: OcdrawToCadOptions,
) -> Result<OcdrawToCadOutcome, OcdrawToCadError> {
    ocdraw::ocdraw::validate_ocdraw_document(drawing)?;
    import_document(drawing, options)
}

fn import_document(
    drawing: &OcdrawDocument,
    options: OcdrawToCadOptions,
) -> Result<OcdrawToCadOutcome, OcdrawToCadError> {
    let mut document = CadDocument::new();
    let patterns = crate::mapping::line_pattern::import_line_patterns(drawing, &mut document)?;
    for source in &drawing.ucs_definitions {
        let mut target = opencadcodec::Ucs::new(&source.definition.name);
        target.handle = document.allocate_handle();
        let frame = source.definition.frame;
        let point = frame.origin();
        let x = frame.x_axis();
        let y = frame.y_axis();
        target.origin = opencadcodec::Vector3::new(point.x(), point.y(), point.z());
        target.x_axis = opencadcodec::Vector3::new(x.x(), x.y(), x.z());
        target.y_axis = opencadcodec::Vector3::new(y.x(), y.y(), y.z());
        target.elevation = source.definition.elevation;
        document
            .ucss
            .add(target)
            .map_err(|error| OcdrawToCadError::Cad(format!("UCS definition: {error}")))?;
    }
    let mut diagnostics = Vec::new();
    let mut preservation_report = crate::OcdrawPreservationReport::default();
    let code = crate::units::UNIT_TOKENS
        .iter()
        .position(|token| *token == drawing.unit.as_str())
        .expect("validated drawing unit") as i16;
    let mut geometry = crate::geometry_context::GeometryContext::new(
        options.geometry_tolerance,
        crate::units::from_cad_code(code).expect("validated drawing unit"),
    )?;
    for layout in &drawing.layouts {
        if layout.kind == DrawingLayoutKind::Paper {
            geometry.add_paper(
                u64::from(layout.scope_id),
                layout.id,
                cad_geometry_convert::plot_units::paper_mapping(
                    layout.settings.plot_settings.as_ref(),
                ),
            )?;
        }
    }
    let converted_ids = drawing
        .geometric_entities
        .iter()
        .map(|entity| entity.id())
        .chain(drawing.viewports.iter().map(|row| row.id))
        .chain(drawing.opaque_entities.iter().map(|row| row.id))
        .collect::<BTreeSet<_>>();
    for (scope_index, scope) in drawing.scopes.iter().enumerate() {
        for (position, id) in scope.entities.iter().enumerate() {
            if !converted_ids.contains(id) {
                diagnostics.push(diagnostic(
                    "UNSUPPORTED_ENTITY",
                    format!("/scopes/{scope_index}/entities/{position}"),
                    format!("entity {id} is not yet transferred to CAD"),
                ));
            }
        }
    }
    let unit = crate::units::from_cad_code(code).expect("validated unit");
    document.header.insertion_units = crate::units::cad_code(unit);
    if let Some(measurement) = crate::units::measurement(unit) {
        document.header.measurement = measurement;
    }
    document.header.plotstyle_mode = drawing.plot_style_mode == PlotStyleMode::ColorDependent;
    let mut layer_names = BTreeMap::new();
    for (index, source) in drawing.layers.iter().enumerate() {
        let id = u64::from(source.id);
        let name = source.name.as_str();
        let mut target = Layer::new(name);
        target.handle = if name == "0" {
            document
                .layers
                .get("0")
                .map(|l| l.handle)
                .unwrap_or_else(|| document.allocate_handle())
        } else {
            document.allocate_handle()
        };
        target.flags.off = !source.visible;
        target.flags.frozen = source.frozen;
        target.flags.locked = source.locked;
        target.flags.frozen_in_new_viewport = source.frozen_in_new_viewports;
        target.is_plottable = source.plottable;
        target.description = source.description.clone().unwrap_or_default();
        let (mapped_color, catalog, color_name) = entities::color(
            &source.color,
            &format!("/layers/{index}/color"),
            &mut diagnostics,
        );
        target.color = mapped_color;
        target.book_name = catalog;
        target.color_name = color_name;
        target.transparency = Transparency::from_percent(1.0 - source.opacity);
        target.line_type = patterns[&source.line_pattern_id].0.clone();
        target.line_weight = line_weight(
            source.line_weight,
            &format!("/layers/{index}/lineWeight"),
            &mut diagnostics,
        );
        if name == "0" {
            *document.layers.get_mut("0").ok_or_else(|| {
                OcdrawToCadError::Cad("fresh CAD document has no Layer 0".into())
            })? = target;
        } else {
            document
                .layers
                .add(target)
                .map_err(|error| OcdrawToCadError::Cad(format!("Layer {name}: {error}")))?;
        }
        layer_names.insert(id, name.to_owned());
    }
    if layer_names.is_empty() {
        diagnostics.push(diagnostic(
            "CAD_SCAFFOLD_LAYER",
            "/layers",
            "CAD runtime creates its default Layer 0",
        ));
    }

    layouts::prepare_primary_paper_layout(drawing, &mut document)?;
    let mut scope_layouts = BTreeMap::new();

    let mut layouts = drawing.layouts.iter().enumerate().collect::<Vec<_>>();
    layouts.sort_by_key(|(_, layout)| layout.tab_index);
    for (index, layout) in layouts {
        let scope_id = u64::from(layout.scope_id);
        let name = layout.name.as_str();
        if layout.kind == DrawingLayoutKind::Model {
            scope_layouts.insert(scope_id, None::<String>);
            if name != "Model" {
                diagnostics.push(diagnostic(
                    "MODEL_LAYOUT_NAME",
                    format!("/layouts/{index}/name"),
                    "CAD model layout name is runtime-defined",
                ));
            }
        } else {
            if !document.objects.values().any(
                |object| matches!(object, ObjectType::Layout(existing) if existing.name == name),
            ) {
                document.add_layout(name).map_err(|error| {
                    OcdrawToCadError::Cad(format!("paper layout {name}: {error}"))
                })?;
            }
            scope_layouts.insert(scope_id, Some(name.to_owned()));
        }
        let scaling = layout.settings.paper_space_linetype_scaling;
        if layout.kind == DrawingLayoutKind::Model {
            document.header.paper_space_linetype_scaling = scaling;
        }
        if let Some(ObjectType::Layout(target)) =
            document.objects.values_mut().find(|object| match object {
                ObjectType::Layout(target) if layout.kind == DrawingLayoutKind::Model => {
                    target.block_record == document.header.model_space_block_handle
                }
                ObjectType::Layout(target) => target.name == name,
                _ => false,
            })
        {
            if scaling {
                target.flags |= 1;
            } else {
                target.flags &= !1;
            }
            if let Some(media) = &layout.settings.media {
                if let Err(e) = crate::mapping::layout::apply_medium_to_cad(target, media) {
                    if e == cad_geometry_convert::plot_units::PlotNumericError::UnsupportedUnit {
                        diagnostics.push(diagnostic(
                            "LAYOUT_MEDIA",
                            format!("/layouts/{index}/media"),
                            e.to_string(),
                        ));
                    } else {
                        return Err(e.into());
                    }
                }
            }
            if layout.settings.limits_checking {
                target.flags |= 2;
            } else {
                target.flags &= !2;
            }
            if let Some(limits) = layout.settings.limits {
                target.min_limits = (limits.min_x, limits.min_y);
                target.max_limits = (limits.max_x, limits.max_y);
            }
            if let Some(plot) = &layout.settings.plot_settings {
                if let Err(e) = crate::mapping::layout::apply_plot_to_cad(
                    target,
                    layout
                        .settings
                        .media
                        .as_ref()
                        .expect("validated plot medium"),
                    plot,
                ) {
                    if e == cad_geometry_convert::plot_units::PlotNumericError::UnsupportedUnit {
                        diagnostics.push(diagnostic(
                            "LAYOUT_SETTINGS",
                            format!("/layouts/{index}/plotSettings"),
                            e.to_string(),
                        ));
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
        if layout.settings.plot_settings.is_some() {
            let target = document
                .objects
                .values()
                .find_map(|o| match o {
                    ObjectType::Layout(l) if l.name == name => Some(l),
                    _ => None,
                })
                .expect("allocated layout");
            if crate::mapping::layout::plot_is_default(target) {
                diagnostics.push(diagnostic("LAYOUT_DEFAULT_PLOT_AMBIGUOUS",format!("/layouts/{index}/plotSettings"),"authored plot equals CAD defaults; native absence cannot be distinguished on reimport"));
            }
        }
        if layout.kind == DrawingLayoutKind::Paper {
            let target = document
                .objects
                .values()
                .find_map(|o| match o {
                    ObjectType::Layout(l) if l.name == name => Some(l),
                    _ => None,
                })
                .expect("allocated layout");
            let mapping = if target.paper_width > 0. && target.paper_height > 0. {
                cad_geometry_convert::plot_units::paper_mapping(
                    layout.settings.plot_settings.as_ref(),
                )
            } else {
                cad_geometry_convert::plot_units::PaperMapping::Unknown
            };
            geometry.refine_paper(u64::from(layout.scope_id), mapping)?;
        }
        if layout.settings.plot_settings.as_ref().is_some_and(|plot| {
            matches!(
                plot.mapping.placement,
                PlotPlacement::Offset {
                    reference: PlotOffsetReference::PrintableArea,
                    ..
                }
            )
        }) {
            diagnostics.push(diagnostic(
                "LAYOUT_SETTINGS",
                format!("/layouts/{index}/plotSettings/mapping/placement"),
                "CAD plot offset uses media origin",
            ));
        }
        if layout
            .settings
            .plot_settings
            .as_ref()
            .is_some_and(|plot| plot.options.plot_transparency)
        {
            diagnostics.push(diagnostic(
                "LAYOUT_SETTINGS",
                format!("/layouts/{index}/plotSettings/options/plotTransparency"),
                "CAD plot transparency is unavailable",
            ));
        }
    }
    layouts::prepare_paper_canvases(drawing, &mut document)?;
    if let Some(workspace) = drawing.workspace_state.as_ref() {
        if let Some(id) = workspace.current_layer_id {
            if let Some(name) = layer_names.get(&u64::from(id)) {
                document.header.current_layer_name = name.clone();
            }
        }
        if let Some(id) = workspace.active_layout_id {
            if let Some(layout) = drawing.layouts.iter().find(|layout| layout.id == id) {
                if layout.kind == DrawingLayoutKind::Paper {
                    document.header.show_model_space = false;
                    let name = layout.name.as_str();
                    if let Some(block) = document.objects.values().find_map(|object| match object {
                        ObjectType::Layout(target) if target.name == name => {
                            Some(target.block_record)
                        }
                        _ => None,
                    }) {
                        document.header.paper_space_block_handle = block;
                    }
                } else {
                    document.header.show_model_space = true;
                }
            }
        }
    }
    if let Some(display) = drawing.point_display {
        let (mode, size) = crate::point_display::to_cad(display);
        document.header.point_display_mode = mode;
        document.header.point_display_size = size;
    }
    let mut block_handles = BTreeMap::new();
    let mut blocks = BTreeMap::new();
    for definition in &drawing.block_definitions {
        let scope_id = u64::from(definition.scope_id);
        let name = definition.name.as_str();
        let base = definition.base_point;
        let mut record = BlockRecord::new(name);
        record.handle = document.allocate_handle();
        record.block_entity_handle = document.allocate_handle();
        record.block_end_handle = document.allocate_handle();
        record.base_point = opencadcodec::Vector3::new(base[0], base[1], base[2]);
        record.description = definition.description.clone();
        record.flags.anonymous = definition.anonymous;
        record.explodable = definition.explodable;
        record.scale_uniformly = definition.uniform_scaling;
        let unit = definition.insertion_unit.as_str();
        record.units = crate::units::UNIT_TOKENS
            .iter()
            .position(|candidate| *candidate == unit)
            .unwrap_or(0) as i16;
        let mut marker = opencadcodec::entities::Block::new(name, record.base_point);
        marker.description = record.description.clone();
        marker.common.handle = record.block_entity_handle;
        marker.common.owner_handle = record.handle;
        let mut end = opencadcodec::entities::BlockEnd::new();
        end.common.handle = record.block_end_handle;
        end.common.owner_handle = record.handle;
        let handle = record.handle;
        document
            .block_records
            .add(record)
            .map_err(|error| OcdrawToCadError::Cad(format!("block definition {name}: {error}")))?;
        for entity in [EntityType::Block(marker), EntityType::BlockEnd(end)] {
            document
                .add_entity(entity)
                .map_err(|error| OcdrawToCadError::Cad(format!("block marker {name}: {error}")))?;
        }
        block_handles.insert(scope_id, handle);
        blocks.insert(scope_id, definition.clone());
    }

    let entity_mapping = entities::append_entities(
        drawing,
        &mut document,
        &entities::TargetIndex {
            patterns: &patterns,
            layers: &layer_names,
            layouts: &scope_layouts,
            block_handles: &block_handles,
            blocks: &blocks,
        },
        &mut geometry,
        &mut diagnostics,
        options.preservation_restore,
        &mut preservation_report,
    )?;
    geometry.select_drawing();
    geometry.record_unassessed(drawing);
    if let Some(p) = &drawing.preservation {
        for record in &p.records {
            let root = (record.role == ocdraw::ocdraw::OcdrawPreservationRole::Complete
                && record.category != ocdraw::ocdraw::OcdrawPreservationCategory::Entity)
                || (record.role == ocdraw::ocdraw::OcdrawPreservationRole::Supplement
                    && record
                        .subject
                        .is_some_and(|target| crate::preservation::target_exists(drawing, target)));
            if root {
                crate::preservation::report_restore(
                    &mut preservation_report,
                    record,
                    None,
                    Some(crate::OcdrawPreservationReason::UnsupportedPayload),
                );
                diagnostics.push(diagnostic(
                    "PRESERVATION_NOT_RESTORED",
                    format!("/preservation/records/{}", record.id.0),
                    "no restoring provider for this live non-entity/supplement root",
                ));
            }
        }
    }
    crate::mapping::workspace::apply(drawing, &mut document, &scope_layouts, &mut diagnostics);
    if let Some(id) = drawing.workspace_state.and_then(|w| w.active_layout_id) {
        if let Some(l) = drawing.layouts.iter().find(|l| l.id == id) {
            document.header.paper_space_linetype_scaling = l.settings.paper_space_linetype_scaling;
        }
    }
    let members = drawing
        .scopes
        .iter()
        .map(|scope| (u64::from(scope.id), scope.entities.clone()))
        .collect();
    for (entity, bound) in geometry.assess_occurrences(&members)? {
        diagnostics.push(diagnostic(
            "GEOMETRY_ROUNDED_WITHIN_TOLERANCE",
            format!("/entities/{entity}"),
            format!("block occurrence deviation is at most {bound} drawing units"),
        ));
    }
    if options.loss_policy == OcdrawLossPolicy::Reject
        && diagnostics
            .iter()
            .any(|d| d.code != "GEOMETRY_ROUNDED_WITHIN_TOLERANCE")
    {
        return Err(OcdrawToCadError::LossRejected {
            diagnostics,
            preservation: preservation_report,
        });
    }
    Ok(OcdrawToCadOutcome {
        preservation: preservation_report,
        document,
        diagnostics,
        entity_mapping,
        geometry: geometry.finish(),
    })
}

impl From<Box<crate::OcdrawGeometryFailure>> for OcdrawToCadError {
    fn from(failure: Box<crate::OcdrawGeometryFailure>) -> Self {
        Self::Geometry(failure)
    }
}

/// Converts content already validated by the production reader.
pub fn ocdraw_source_to_cad_document(
    drawing: &ocdraw::ocdraw::ValidatedOcdraw,
    options: OcdrawToCadOptions,
) -> Result<OcdrawToCadOutcome, OcdrawToCadError> {
    import_document(drawing.document(), options)
}
