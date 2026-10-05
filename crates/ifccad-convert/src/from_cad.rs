use crate::diagnostics::diagnostic;
use crate::units::UNITS;
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
        validated,
        encoded,
        diagnostics: logical.diagnostics,
        mappings: logical.mappings,
    })
}

/// Construct a logical CAD projection with explicit semantic loss acceptance.
pub fn cad_document_to_ifccad_document(
    source: &CadDocument,
    metadata: IfccadTargetMetadata,
    options: CadToIfccadOptions,
) -> Result<CadToIfccadDocumentOutcome, IfccadConversionError> {
    let info = crate::source::inspect(source)?;
    let mut issues = info.issues;
    let mut ids = IfccadIdCounters::default();
    let (mut line_patterns, patterns) = crate::patterns::from_cad(source, &mut ids, &mut issues)?;
    let length_unit = UNITS
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
    let model_id = ids.allocate_layout_id().map_err(allocation_error)?;
    mappings.layouts.insert(model_id, info.model_layout);
    let mut layers = Vec::new();
    for l in source.layers.iter() {
        let id = ids.allocate_layer_id().map_err(allocation_error)?;
        mappings.layers.insert(id, l.handle);
        layers.push(IfccadLayer {
            id,
            name: l.name.clone(),
            appearance: crate::appearance::from_layer(l, &patterns, &mut issues),
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
    let entities = convert_entities(
        source,
        &patterns,
        &info.entities,
        &mut ids,
        &mut mappings,
        &mut issues,
    )?;
    let mut blocks = Vec::new();
    for h in &supported {
        let b = source
            .block_records
            .iter()
            .find(|b| b.handle == *h)
            .unwrap();
        let unit = UNITS.get(b.units as usize).unwrap_or_else(|| {
            issues.push(crate::diagnostics::modification(
                "units",
                format!("block/{}", b.name),
                "unknown block insertion unit replaced with unitless; coordinates are not scaled",
            ));
            &"unitless"
        });
        blocks.push(IfccadBlockDefinition {
            id: mappings.blocks.ifccad_id(*h).unwrap(),
            name: b.name.clone(),
            base_point: crate::geometry::p(b.base_point),
            insertion_unit: unit.to_string(),
            entities: convert_entities(
                source,
                &patterns,
                &b.entity_handles,
                &mut ids,
                &mut mappings,
                &mut issues,
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
        let (length_unit, medium) = crate::layouts::from_cad(layout, &mut issues);
        let entities = convert_entities(
            source,
            &patterns,
            &paper.entity_handles,
            &mut ids,
            &mut mappings,
            &mut issues,
        )?;
        paper_layouts.push(IfccadPaperLayout {
            id,
            name: layout.name.clone(),
            tab_index: paper.tab_index,
            length_unit,
            paper: medium,
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
    crate::diagnostics::enforce_policy(options.loss_policy, &issues)?;
    let drawing = IfccadDocument {
        header: metadata.header,
        drawing_id: metadata.drawing_id,
        id_counters: ids,
        length_unit,
        line_patterns,
        line_pattern_scale: source.header.linetype_scale,
        layers,
        model: IfccadLayout {
            id: model_id,
            tab_index: 0,
            entities,
        },
        paper_layouts,
        blocks,
    };
    validate_ifccad_document(&drawing).map_err(IfccadConversionError::CoreValidation)?;
    Ok(CadToIfccadDocumentOutcome {
        document: drawing,
        diagnostics: issues.into_iter().chain(info.recoveries).collect(),
        mappings,
    })
}
fn allocation_error(error: IfccadIdAllocationError) -> IfccadConversionError {
    IfccadConversionError::IdAllocation(error)
}

fn convert_entities(
    source: &CadDocument,
    patterns: &crate::patterns::SourcePatterns,
    handles: &[opencadcodec::Handle],
    ids: &mut IfccadIdCounters,
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Vec<IfccadEntity>, IfccadConversionError> {
    crate::entity_owners::from_cad(source, patterns, handles, ids, mappings, issues)
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn allocation_exhaustion_retains_domain_and_source() {
        let mut ids = IfccadIdCounters {
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
