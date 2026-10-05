use crate::diagnostics::diagnostic;
use crate::units::unit_code;
use crate::*;
use ocdraw::ifcx_cad::{
    encode_ifcx_cad_document, validate_ifcx_cad_document, IfcxCadDocument, ValidatedIfcxCad,
};
use opencadcodec::CadDocument;

/// Convert supported content with explicit semantic loss acceptance.
pub fn ifcx_cad_source_to_cad_document(
    source: &ValidatedIfcxCad,
    options: IfcxCadToCadOptions,
) -> Result<IfcxCadToCadOutcome, IfcxCadConversionError> {
    let drawing = source.document();
    let canonical =
        encode_ifcx_cad_document(drawing).map_err(IfcxCadConversionError::CoreEncoding)?;
    let canonical: serde_json::Value =
        serde_json::from_slice(canonical.bytes()).expect("writer JSON");
    let mut issues = Vec::new();
    let raw = crate::loss::native_defaults(source.graph().composed_ifcx());
    // Compare composed node payloads, allowing fragment order but not losing
    // extensions, relations or extra schemas through the typed projection.
    if !crate::source::same_graph(&raw, &canonical) {
        issues.push(diagnostic(
            "foreign-ifcx",
            "IFCX graph",
            "graph contains information outside the CAD projection",
        ));
    }
    crate::loss::precision(&raw, &canonical, &mut issues);
    convert_document(drawing, options, issues)
}

/// Convert a logical CAD projection with explicit semantic loss acceptance.
pub fn ifcx_cad_document_to_cad_document(
    source: &IfcxCadDocument,
    options: IfcxCadToCadOptions,
) -> Result<IfcxCadToCadOutcome, IfcxCadConversionError> {
    validate_ifcx_cad_document(source).map_err(IfcxCadConversionError::CoreValidation)?;
    convert_document(source, options, Vec::new())
}

fn convert_document(
    drawing: &IfcxCadDocument,
    options: IfcxCadToCadOptions,
    mut issues: Vec<IfcxCadDiagnostic>,
) -> Result<IfcxCadToCadOutcome, IfcxCadConversionError> {
    // Native layers are identified by ID; CAD tables look them up by normalized
    // name. Reject ambiguity before the special layer-0 replacement or allocation.
    let mut layer_names = std::collections::BTreeMap::new();
    for layer in &drawing.layers {
        let key = opencadcodec::tables::normalize_name(&layer.name);
        if let Some(previous) = layer_names.insert(key, layer.id) {
            return Err(IfcxCadConversionError::InvalidStructure(format!(
                "target layer lookup collision between layer/{previous} and layer/{} ({:?})",
                layer.id, layer.name
            )));
        }
    }
    if !drawing.layers.iter().any(|l| l.name == "0") {
        issues.push(crate::diagnostics::modification(
            "layer-0",
            "drawing.layers",
            "missing CAD layer 0 generated as white, opaque, Continuous and 0.25 mm",
        ));
    }
    let mut document = CadDocument::new();
    if !drawing.layers.iter().any(|l| l.name == "0") {
        let layer = document.layers.get_mut("0").unwrap();
        layer.color = opencadcodec::Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        layer.line_weight = opencadcodec::LineWeight::Value(25);
        layer.transparency = opencadcodec::Transparency::Explicit(0);
    }
    document.header.insertion_units = unit_code(&drawing.length_unit);
    let mut mappings = IfcxCadMappings::default();
    document.header.linetype_scale = drawing.line_pattern_scale;
    crate::patterns::allocate(
        &mut document,
        &drawing.line_patterns,
        &mut mappings,
        &mut issues,
    )?;
    let layout = document
        .objects
        .values()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l)
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
            &crate::patterns::target(&document, &mappings, layer.appearance.line_pattern).0,
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
    let supported: Vec<_> = drawing
        .blocks
        .iter()
        .filter(|b| {
            if b.name.starts_with('*') {
                issues.push(diagnostic(
                    "block-skipped",
                    format!("block/{}", b.id),
                    "reserved or anonymous definition omitted together with referring instances",
                ));
                false
            } else {
                true
            }
        })
        .cloned()
        .collect();
    crate::blocks::allocate(&mut document, &supported, &mut mappings)?;
    let paper_owners = crate::layouts::allocate(
        &mut document,
        &drawing.paper_layouts,
        &mut mappings,
        &mut issues,
    )?;
    let model_owner = document.header.model_space_block_handle;
    let block_owners: Vec<_> = supported
        .iter()
        .map(|b| {
            (
                mappings.blocks.cad_handle(b.id).expect("allocated block"),
                b.entities.as_slice(),
            )
        })
        .collect();
    let owners: Vec<_> = std::iter::once((model_owner, drawing.model.entities.as_slice()))
        .chain(paper_owners.iter().map(|(id, owner)| {
            (
                *owner,
                drawing
                    .paper_layouts
                    .iter()
                    .find(|p| p.id == *id)
                    .expect("allocated layout")
                    .entities
                    .as_slice(),
            )
        }))
        .chain(block_owners)
        .collect();
    crate::entity_owners::to_cad(drawing, &mut document, &owners, &mut mappings, &mut issues)?;
    crate::loss::to_cad(drawing, &mut issues);
    crate::diagnostics::enforce_policy(options.loss_policy, &issues)?;
    Ok(IfcxCadToCadOutcome {
        document,
        diagnostics: issues,
        mappings,
    })
}
