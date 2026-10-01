use crate::outcome::{diagnostic, unit_code};
use crate::*;
use cadcodec::CadDocument;
use ocdraw::ifcx_cad::{write_native_cad_ifcx, ValidatedIfcxCad};

/// Convert a validated IFCX-CAD drawing directly, rejecting unsupported content.
pub fn ifcx_cad_to_cad_document(
    source: &ValidatedIfcxCad,
) -> Result<IfcxCadToCadOutcome, IfcxCadConversionError> {
    let drawing = source.document();
    let canonical = write_native_cad_ifcx(drawing)
        .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
    let canonical: serde_json::Value = serde_json::from_slice(&canonical).expect("writer JSON");
    let mut issues = Vec::new();
    // Compare composed node payloads, allowing fragment order but not losing
    // extensions, relations or extra schemas through the typed projection.
    if !crate::source::same_graph(source.raw_ifcx(), &canonical) {
        issues.push(diagnostic(
            "foreign-ifcx",
            "IFCX graph",
            "graph contains information outside the CAD projection",
        ));
    }
    if !drawing.paper_layouts.is_empty() {
        issues.push(diagnostic(
            "paper",
            "drawing",
            "Paper conversion is deferred",
        ));
    }
    if !drawing.layers.iter().any(|l| l.name == "0") {
        issues.push(diagnostic(
            "layer-0",
            "drawing.layers",
            "this slice requires explicit layer 0 to avoid inventing its appearance",
        ));
    }
    let mut document = CadDocument::new();
    document.header.insertion_units = unit_code(&drawing.length_unit);
    let mut mappings = IfcxCadMappings::default();
    let layout = document
        .objects
        .values()
        .find_map(|o| match o {
            cadcodec::objects::ObjectType::Layout(l)
                if l.block_record == document.header.model_space_block_handle =>
            {
                Some(l.handle)
            }
            _ => None,
        })
        .expect("default Model layout");
    mappings.layouts.insert(drawing.model.id, layout);
    for layer in &drawing.layers {
        let mut target = crate::appearance::to_layer(
            &layer.name,
            &layer.appearance,
            &format!("layer/{}", layer.id),
            &mut issues,
        );
        if layer.name == "0" {
            target.handle = document.layers.get("0").expect("default layer").handle;
            *document.layers.get_mut("0").unwrap() = target;
        } else {
            target.handle = document.allocate_handle();
            document
                .layers
                .add(target)
                .map_err(IfcxCadConversionError::CadConstruction)?;
        }
        mappings
            .layers
            .insert(layer.id, document.layers.get(&layer.name).unwrap().handle);
    }
    crate::blocks::allocate(&mut document, &drawing.blocks, &mut mappings)?;
    for (owner, entities) in std::iter::once(("*Model_Space", drawing.model.entities.as_slice()))
        .chain(
            drawing
                .blocks
                .iter()
                .map(|b| (b.name.as_str(), b.entities.as_slice())),
        )
    {
        for e in entities {
            let loc = format!("entity/{}", e.id);
            let layer = &drawing
                .layers
                .iter()
                .find(|l| l.id == e.layer_id)
                .expect("validated layer")
                .name;
            let common = crate::appearance::to_common(&e.appearance, layer, &loc, &mut issues);
            let target = match &e.kind {
                ocdraw::ifcx_cad::IfcxCadEntityKind::BlockInstance {
                    definition_id,
                    transform,
                } => Some(crate::blocks::to_insert(
                    &drawing
                        .blocks
                        .iter()
                        .find(|b| b.id == *definition_id)
                        .expect("validated definition")
                        .name,
                    transform,
                    &loc,
                    &mut issues,
                )),
                kind => crate::geometry::to_entity(kind, &loc, &mut issues),
            };
            if let Some(mut target) = target {
                *target.common_mut() = common;
                target.common_mut().owner_handle = document
                    .block_records
                    .get(owner)
                    .expect("allocated owner")
                    .handle;
                let h = document
                    .add_entity(target)
                    .map_err(|e| IfcxCadConversionError::CadConstruction(e.to_string()))?;
                mappings.entities.insert(e.id, h);
            }
        }
    }
    if !issues.is_empty() {
        return Err(IfcxCadConversionError::Unsupported(issues));
    }
    Ok(IfcxCadToCadOutcome {
        document,
        diagnostics: vec![],
        mappings,
    })
}
