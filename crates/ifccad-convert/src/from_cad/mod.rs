mod entities;

use crate::diagnostics::diagnostic;
use crate::units::UNIT_TOKENS;
use crate::*;
use ocdraw::ifccad::*;
use opencadcodec::CadDocument;

/// Convert supported content with explicit semantic loss acceptance.
pub fn cad_document_to_encoded_ifccad(
    source: &CadDocument,
    metadata: IfccadTargetMetadata,
    options: CadToIfccadOptions,
) -> Result<CadToEncodedIfccadOutcome, IfccadConversionError> {
    let logical = cad_document_to_ifccad_document(source, metadata, options)?;
    let encoded =
        encode_ifccad_document(&logical.document).map_err(IfccadConversionError::CoreEncoding)?;
    let validated = load_ifccad_bytes(encoded.bytes(), Default::default())
        .map_err(IfccadConversionError::CoreReadback)?;
    Ok(CadToEncodedIfccadOutcome {
        text: logical.text,
        validated,
        encoded,
        diagnostics: logical.diagnostics,
        mappings: logical.mappings,
        geometry: logical.geometry,
        preservation: logical.preservation,
    })
}

/// Construct a logical CAD projection with explicit semantic loss acceptance.
pub fn cad_document_to_ifccad_document(
    source: &CadDocument,
    metadata: IfccadTargetMetadata,
    options: CadToIfccadOptions,
) -> Result<CadToIfccadDocumentOutcome, IfccadConversionError> {
    let mut preservation = crate::preservation::Capture {
        enabled: options.preservation == crate::IfccadPreservationCapture::SupportedTyped,
        ..Default::default()
    };
    let info = crate::source::inspect(source, preservation.enabled)?;
    for entity in source.entities() {
        if let opencadcodec::EntityType::Viewport(view) = entity {
            if source
                .block_records
                .iter()
                .any(|b| b.handle == view.common.owner_handle && b.is_paper_space())
            {
                cad_workspace_convert::prepare_viewport_aids_from_cad(view)?;
            }
        }
    }
    let mut issues = info.issues;
    let mut ids = IfccadIdCounters::default();
    let (mut line_patterns, patterns) = crate::mapping::line_pattern::from_cad(
        source,
        preservation.enabled,
        &mut ids,
        &mut issues,
    )?;
    let length_unit = UNIT_TOKENS
        .get(source.header.insertion_units as usize)
        .unwrap_or_else(|| {
            issues.push(crate::diagnostics::modification(
                "units",
                "header.insertion_units",
                "unknown CAD unit code replaced with unitless; coordinates are not scaled",
            ));
            &"unitless"
        })
        .to_string();
    let mut mappings = IfccadMappings::default();
    let text_styles =
        crate::mapping::text::from_styles(source, &mut ids, &mut mappings, &mut issues)?;
    let model_id = ids.allocate_layout_id().map_err(allocation_error)?;
    mappings.layouts.insert(model_id, info.model_layout);
    let mut layers = Vec::new();
    for l in source.layers.iter() {
        let id = ids.allocate_layer_id().map_err(allocation_error)?;
        mappings.layers.insert(id, l.handle);
        layers.push(IfccadLayer {
            id,
            name: l.name.clone(),
            appearance: crate::mapping::appearance::from_layer(l, &patterns, &mut issues),
        });
    }
    let supported: Vec<_> = info.blocks.iter().copied().filter(|h| {
        let b = source.block_records.iter().find(|b| b.handle == *h).unwrap();
        let dynamic = source.objects.values().any(|o| matches!(o, opencadcodec::objects::ObjectType::DynamicBlock(d) if d.owner == *h));
        let supported = !b.name.starts_with('*') && !b.is_anonymous()
            && !b.flags.is_xref && !b.flags.is_xref_overlay && !b.flags.is_external
            && !b.flags.is_xref_unloaded && b.xref_path.is_empty() && !dynamic;
        if !supported {
            issues.push(diagnostic("block-skipped", format!("block/{}", b.name), "anonymous, external or dynamic block definition omitted, together with referring instances"));
        }
        supported
    }).collect();
    for h in &supported {
        mappings
            .blocks
            .insert(ids.allocate_block_id().map_err(allocation_error)?, *h);
    }
    for paper in &info.papers {
        mappings.layouts.insert(
            ids.allocate_layout_id().map_err(allocation_error)?,
            paper.layout_handle,
        );
    }
    let mut geometry = crate::geometry_context::GeometryContext::new(
        options.geometry_tolerance,
        &length_unit,
        model_id,
    )?;
    let mut paper_metadata = std::collections::BTreeMap::new();
    for paper in &info.papers {
        let opencadcodec::objects::ObjectType::Layout(layout) =
            &source.objects[&paper.layout_handle]
        else {
            unreachable!("inspected Paper layout")
        };
        let settings = crate::mapping::layout::settings_from_cad(layout, false, &mut issues)?;
        geometry.add_paper(
            mappings
                .layouts
                .ifccad_id(paper.layout_handle)
                .expect("allocated Paper"),
            cad_geometry_convert::plot_units::paper_mapping(settings.plot_settings.as_ref()),
        )?;
        paper_metadata.insert(paper.layout_handle, settings);
    }
    let entities = entities::from_cad(
        entities::SourceScope {
            document: source,
            patterns: &patterns,
            entities: &info.entities,
        },
        &mut preservation,
        &mut ids,
        &mut mappings,
        &mut issues,
        &mut geometry,
    )?;
    let mut blocks = Vec::new();
    for h in &supported {
        let b = source
            .block_records
            .iter()
            .find(|b| b.handle == *h)
            .unwrap();
        let unit = UNIT_TOKENS.get(b.units as usize).unwrap_or_else(|| {
            issues.push(crate::diagnostics::modification(
                "units",
                format!("block/{}", b.name),
                "unknown block insertion unit replaced with unitless; coordinates are not scaled",
            ));
            &"unitless"
        });
        geometry.select(crate::IfccadGeometryOwner::BlockDefinition(
            mappings.blocks.ifccad_id(*h).unwrap(),
        ));
        blocks.push(IfccadBlockDefinition {
            bounds_quality: None,
            bounds: None,
            id: mappings.blocks.ifccad_id(*h).unwrap(),
            name: b.name.clone(),
            base_point: crate::mapping::geometry::p(b.base_point),
            insertion_unit: unit.to_string(),
            entities: entities::from_cad(
                entities::SourceScope {
                    document: source,
                    patterns: &patterns,
                    entities: &b.entity_handles,
                },
                &mut preservation,
                &mut ids,
                &mut mappings,
                &mut issues,
                &mut geometry,
            )?,
        });
    }
    // The current core projection enumerates dictionary paths lexicographically.
    let mut paper_layouts = Vec::new();
    for paper in &info.papers {
        let opencadcodec::objects::ObjectType::Layout(layout) =
            &source.objects[&paper.layout_handle]
        else {
            unreachable!("inspected layout")
        };
        debug_assert_eq!(layout.block_record, paper.block_handle);
        let id = mappings
            .layouts
            .ifccad_id(paper.layout_handle)
            .expect("allocated Paper owner");
        let settings = paper_metadata
            .remove(&paper.layout_handle)
            .expect("prepared Paper metadata");
        geometry.select(crate::IfccadGeometryOwner::PaperLayout(id));
        let entities = entities::from_cad(
            entities::SourceScope {
                document: source,
                patterns: &patterns,
                entities: &paper.entity_handles,
            },
            &mut preservation,
            &mut ids,
            &mut mappings,
            &mut issues,
            &mut geometry,
        )?;
        paper_layouts.push(IfccadPaperLayout {
            bounds_quality: None,
            canvas: None,

            bounds: None,
            id,
            name: layout.name.clone(),
            tab_index: paper.tab_index,
            settings,
            entities,
        });
    }
    layers.sort_by_key(|l| l.id.to_string());
    blocks.sort_by_key(|b| b.id.to_string());
    for p in &line_patterns {
        mappings
            .line_patterns
            .insert(p.id.0, source.line_types.get(&p.name).unwrap().handle);
    }
    line_patterns.sort_by_key(|p| p.id.0.to_string());
    crate::loss::from_cad(
        source,
        &info.blocks,
        &info.entities,
        &info
            .papers
            .iter()
            .map(|p| p.entity_handles.as_slice())
            .collect::<Vec<_>>(),
        &mut issues,
    );
    let mut drawing = IfccadDocument {
        text_styles,
        preservation: None,
        ucs_definitions: vec![],
        model_windows: vec![],
        workspace_state: None,
        model_view_state: None,

        header: metadata.header,
        drawing_id: metadata.drawing_id,
        id_counters: ids,
        length_unit,
        plot_style_mode: if source.header.plotstyle_mode {
            IfccadPlotStyleMode::ColorDependent
        } else {
            IfccadPlotStyleMode::Named
        },
        line_patterns,
        line_pattern_scale: source.header.linetype_scale,
        layers,
        model: IfccadLayout {
            bounds_quality: None,
            settings: {
                let opencadcodec::objects::ObjectType::Layout(l) =
                    &source.objects[&info.model_layout]
                else {
                    unreachable!()
                };
                let mut settings = crate::mapping::layout::settings_from_cad(l, true, &mut issues)?;
                if source.header.show_model_space {
                    settings.paper_space_linetype_scaling =
                        source.header.paper_space_linetype_scaling;
                }
                settings
            },
            bounds: None,
            id: model_id,
            tab_index: 0,
            entities,
        },
        paper_layouts,
        blocks,
    };
    preservation.finish(source, &mut drawing, &mappings)?;
    crate::mapping::workspace::from_cad(source, &mut drawing, &mut mappings, &mut issues)?;
    crate::diagnostics::enforce_policy(options.loss_policy, &issues)?;
    let members = drawing
        .blocks
        .iter()
        .map(|b| {
            (
                mappings.blocks.cad_handle(b.id).unwrap().value(),
                b.entities
                    .iter()
                    .filter_map(|e| mappings.entities.cad_handle(e.id()).map(|h| h.value()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let roots = |entities: &[IfccadEntity]| {
        entities
            .iter()
            .filter(|e| {
                e.as_native()
                    .is_some_and(|e| matches!(e.kind, IfccadEntityKind::BlockInstance { .. }))
            })
            .map(|e| mappings.entities.cad_handle(e.id()).unwrap().value())
            .collect::<Vec<_>>()
    };
    geometry.select(crate::IfccadGeometryOwner::ModelLayout(drawing.model.id));
    geometry.assess_roots(&members, &roots(&drawing.model.entities), &mut issues)?;
    for b in &drawing.blocks {
        geometry.select(crate::IfccadGeometryOwner::BlockDefinition(b.id));
        geometry.assess_roots(&members, &roots(&b.entities), &mut issues)?;
    }
    for p in &drawing.paper_layouts {
        geometry.select(crate::IfccadGeometryOwner::PaperLayout(p.id));
        geometry.assess_roots(&members, &roots(&p.entities), &mut issues)?;
    }
    crate::diagnostics::enforce_policy(options.loss_policy, &issues)?;
    recompute_ifccad_document_bounds(&mut drawing)
        .map_err(IfccadConversionError::CoreValidation)?;
    validate_ifccad_document(&drawing).map_err(IfccadConversionError::CoreValidation)?;
    Ok(CadToIfccadDocumentOutcome {
        text: crate::IfccadTextAssessment::new(
            &drawing,
            mappings.entities.iter().map(|(id, _)| id),
        ),
        geometry: geometry.finish().with_unassessed(&drawing),
        preservation: preservation.report,
        document: drawing,
        diagnostics: issues.into_iter().chain(info.recoveries).collect(),
        mappings,
    })
}
fn allocation_error(error: IfccadIdAllocationError) -> IfccadConversionError {
    IfccadConversionError::IdAllocation(error)
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn allocation_exhaustion_retains_domain_and_source() {
        let mut ids = IfccadIdCounters {
            next_text_style_id: 1,
            next_preservation_record_id: 1,
            next_ucs_id: 1,
            next_model_window_id: 1,

            next_entity_id: u64::MAX,
            next_layer_id: u64::MAX,
            next_layout_id: u64::MAX,
            next_block_id: u64::MAX,
            next_line_pattern_id: u64::MAX,
        };
        for (result, domain) in [
            (ids.allocate_entity_id(), IfccadIdDomain::Entity),
            (ids.allocate_layer_id(), IfccadIdDomain::Layer),
            (ids.allocate_layout_id(), IfccadIdDomain::Layout),
            (ids.allocate_block_id(), IfccadIdDomain::Block),
        ] {
            let error = allocation_error(result.unwrap_err());
            let IfccadConversionError::IdAllocation(source) = &error else {
                panic!("expected typed allocation failure");
            };
            assert_eq!(source.domain, domain);
            assert_eq!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<IfccadIdAllocationError>(),
                Some(source)
            );
        }
    }
}
