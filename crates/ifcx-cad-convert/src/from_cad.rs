use crate::diagnostics::diagnostic;
use crate::units::UNITS;
use crate::*;
use ocdraw::ifcx_cad::*;
use opencadcodec::CadDocument;

/// Convert supported content with explicit semantic loss acceptance.
pub fn cad_document_to_encoded_ifcx_cad(
    source: &CadDocument,
    metadata: IfcxCadTargetMetadata,
    options: CadToIfcxCadOptions,
) -> Result<CadToEncodedIfcxCadOutcome, IfcxCadConversionError> {
    let logical = cad_document_to_ifcx_cad_document(source, metadata, options)?;
    let encoded = encode_ifcx_cad_document(&logical.document)
        .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
    let validated = load_ifcx_cad_bytes(encoded.bytes(), Default::default())
        .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
    Ok(CadToEncodedIfcxCadOutcome {
        validated,
        encoded,
        diagnostics: logical.diagnostics,
        mappings: logical.mappings,
    })
}

/// Construct a logical CAD projection with explicit semantic loss acceptance.
pub fn cad_document_to_ifcx_cad_document(
    source: &CadDocument,
    metadata: IfcxCadTargetMetadata,
    options: CadToIfcxCadOptions,
) -> Result<CadToIfcxCadDocumentOutcome, IfcxCadConversionError> {
    let info = crate::source::inspect(source)?;
    let mut issues = info.issues;
    let mut ids = IfcxCadIdCounters::default();
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
    let mut mappings = IfcxCadMappings::default();
    let model_id = ids.allocate_layout_id().map_err(allocation_error)?;
    mappings.layouts.insert(model_id, info.model_layout);
    let mut layers = Vec::new();
    for l in source.layers.iter() {
        let id = ids.allocate_layer_id().map_err(allocation_error)?;
        mappings.layers.insert(id, l.handle);
        layers.push(IfcxCadLayer {
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
        blocks.push(IfcxCadBlockDefinition {
            id: mappings.blocks.ifcx_id(*h).unwrap(),
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
    layers.sort_by_key(|l| l.id.to_string());
    blocks.sort_by_key(|b| b.id.to_string());
    for p in &line_patterns {
        mappings
            .line_patterns
            .insert(p.id.0, source.line_types.get(&p.name).unwrap().handle);
    }
    line_patterns.sort_by_key(|p| p.id.0.to_string());
    crate::loss::from_cad(source, &info.blocks, &info.entities, &mut issues);
    crate::diagnostics::enforce_policy(options.loss_policy, &issues)?;
    let drawing = IfcxCadDocument {
        header: metadata.header,
        drawing_id: metadata.drawing_id,
        id_counters: ids,
        length_unit,
        line_patterns,
        line_pattern_scale: source.header.linetype_scale,
        layers,
        model: IfcxCadLayout {
            id: model_id,
            entities,
        },
        paper_layouts: vec![],
        blocks,
    };
    validate_ifcx_cad_document(&drawing)
        .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
    Ok(CadToIfcxCadDocumentOutcome {
        document: drawing,
        diagnostics: issues.into_iter().chain(info.recoveries).collect(),
        mappings,
    })
}
fn allocation_error(error: IfcxCadIdAllocationError) -> IfcxCadConversionError {
    IfcxCadConversionError::CoreValidation(error.to_string())
}

fn convert_entities(
    source: &CadDocument,
    patterns: &crate::patterns::SourcePatterns,
    handles: &[opencadcodec::Handle],
    ids: &mut IfcxCadIdCounters,
    mappings: &mut IfcxCadMappings,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> Result<Vec<IfcxCadEntity>, IfcxCadConversionError> {
    let mut entities = Vec::new();
    for h in handles {
        let e = source.get_entity(*h).expect("inspected entity");
        let loc = format!("entity/{h}");
        let appearance = crate::appearance::from_common(e.common(), patterns, &loc, issues);
        let kind = match e {
            opencadcodec::EntityType::Insert(i) => mappings
                .blocks
                .ifcx_id(
                    source
                        .block_records
                        .get(&i.block_name)
                        .expect("inspected target")
                        .handle,
                )
                .and_then(|id| crate::blocks::from_insert(i, id, &loc, issues)),
            e => crate::geometry::from_entity(e, &loc, issues),
        };
        if kind.is_none() {
            issues.push(diagnostic(
                "entity-skipped",
                &loc,
                "whole entity omitted; unsupported geometry or entity family",
            ));
        }
        if let Some(kind) = kind {
            let id = ids.allocate_entity_id().map_err(allocation_error)?;
            mappings.entities.insert(id, *h);
            entities.push(IfcxCadEntity {
                line_pattern_scale: e.common().linetype_scale,
                id,
                layer_id: mappings
                    .layers
                    .ifcx_id(
                        source
                            .layers
                            .get(&e.common().layer)
                            .expect("inspected layer")
                            .handle,
                    )
                    .unwrap(),
                appearance,
                kind,
            });
        }
    }
    Ok(entities)
}
