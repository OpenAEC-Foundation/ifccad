use crate::outcome::{diagnostic, UNITS};
use crate::*;
use cadcodec::CadDocument;
use ocdraw::ifcx_cad::*;

/// Convert the supported CAD subset and strict-read the emitted IFCX.
pub fn cad_document_to_ifcx_cad(
    source: &CadDocument,
    metadata: IfcxCadTargetMetadata,
) -> Result<CadToIfcxCadOutcome, IfcxCadConversionError> {
    let info = crate::source::inspect(source)?;
    let mut issues = info.issues;
    let length_unit = UNITS
        .get(source.header.insertion_units as usize)
        .unwrap_or_else(|| {
            issues.push(diagnostic(
                "units",
                "header.insertion_units",
                "unknown CAD unit code",
            ));
            &"unitless"
        })
        .to_string();
    let mut mappings = IfcxCadMappings::default();
    mappings.layouts.insert(1, info.model_layout);
    let mut layers: Vec<_> = source
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| {
            mappings.layers.insert(i as u64, l.handle);
            IfcxCadLayer {
                id: i as u64,
                name: l.name.clone(),
                appearance: crate::appearance::from_layer(l, &mut issues),
            }
        })
        .collect();
    for (i, h) in info.blocks.iter().enumerate() {
        mappings.blocks.insert(i as u64 + 1, *h);
    }
    let mut next_id = 1;
    let entities = convert_entities(
        source,
        &info.entities,
        &mut next_id,
        &mut mappings,
        &mut issues,
    );
    let mut blocks = Vec::new();
    for h in &info.blocks {
        let b = source
            .block_records
            .iter()
            .find(|b| b.handle == *h)
            .unwrap();
        let unit = UNITS.get(b.units as usize).unwrap_or_else(|| {
            issues.push(diagnostic(
                "units",
                format!("block/{}", b.name),
                "unknown insertion unit",
            ));
            &"unitless"
        });
        if b.is_anonymous() {
            issues.push(diagnostic(
                "block-name",
                format!("block/{}", b.name),
                "anonymous block semantics are unsupported",
            ));
        }
        blocks.push(IfcxCadBlockDefinition {
            id: mappings.blocks.ifcx_id(*h).unwrap(),
            name: b.name.clone(),
            base_point: crate::geometry::p(b.base_point),
            insertion_unit: unit.to_string(),
            entities: convert_entities(
                source,
                &b.entity_handles,
                &mut next_id,
                &mut mappings,
                &mut issues,
            ),
        });
    }
    // The current core projection enumerates dictionary paths lexicographically.
    layers.sort_by_key(|l| l.id.to_string());
    blocks.sort_by_key(|b| b.id.to_string());
    if !issues.is_empty() {
        return Err(IfcxCadConversionError::Unsupported(issues));
    }
    let drawing = IfcxCadDocument {
        header: metadata.header,
        drawing_id: metadata.drawing_id,
        length_unit,
        layers,
        model: IfcxCadLayout { id: 1, entities },
        paper_layouts: vec![],
        blocks,
    };
    let bytes = write_native_cad_ifcx(&drawing)
        .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
    let validated = read_native_cad_ifcx(&bytes)
        .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
    Ok(CadToIfcxCadOutcome {
        validated,
        bytes,
        diagnostics: info.recoveries,
        mappings,
    })
}
fn convert_entities(
    source: &CadDocument,
    handles: &[cadcodec::Handle],
    next_id: &mut u64,
    mappings: &mut IfcxCadMappings,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> Vec<IfcxCadEntity> {
    let mut entities = Vec::new();
    for h in handles {
        let e = source.get_entity(*h).expect("inspected entity");
        let loc = format!("entity/{h}");
        let appearance = crate::appearance::from_common(e.common(), &loc, issues);
        let kind = match e {
            cadcodec::EntityType::Insert(i) => Some(crate::blocks::from_insert(
                i,
                mappings
                    .blocks
                    .ifcx_id(
                        source
                            .block_records
                            .get(&i.block_name)
                            .expect("inspected target")
                            .handle,
                    )
                    .expect("local target"),
                &loc,
                issues,
            )),
            e => crate::geometry::from_entity(e, &loc, issues),
        };
        if let Some(kind) = kind {
            let id = *next_id;
            *next_id += 1;
            mappings.entities.insert(id, *h);
            entities.push(IfcxCadEntity {
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
    entities
}
