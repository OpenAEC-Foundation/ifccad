use crate::outcome::{diagnostic, unit_code};
use crate::*;
use cadcodec::CadDocument;
use ocdraw::ifcx_cad::{write_native_cad_ifcx, ValidatedIfcxCad};

/// Convert supported content, returning diagnostics for omitted or modified data.
pub fn ifcx_cad_to_cad_document(
    source: &ValidatedIfcxCad,
) -> Result<IfcxCadToCadOutcome, IfcxCadConversionError> {
    ifcx_cad_to_cad_document_with_options(source, IfcxCadConversionOptions::default())
}

/// Convert supported content with explicit semantic loss acceptance.
pub fn ifcx_cad_to_cad_document_with_options(
    source: &ValidatedIfcxCad,
    options: IfcxCadConversionOptions,
) -> Result<IfcxCadToCadOutcome, IfcxCadConversionError> {
    let drawing = source.document();
    let canonical = write_native_cad_ifcx(drawing)
        .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
    let canonical: serde_json::Value = serde_json::from_slice(&canonical).expect("writer JSON");
    let mut issues = Vec::new();
    let raw = crate::loss::native_defaults(source.raw_ifcx());
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
    for paper in &drawing.paper_layouts {
        issues.push(diagnostic(
            "paper",
            format!("layout/{}", paper.id),
            "Paper layout and its entities omitted; conversion is deferred",
        ));
        for e in &paper.entities {
            issues.push(diagnostic(
                "entity-skipped",
                format!("entity/{}", e.id),
                "entity omitted with its unsupported Paper layout",
            ));
        }
    }
    if !drawing.layers.iter().any(|l| l.name == "0") {
        issues.push(crate::outcome::modification(
            "layer-0",
            "drawing.layers",
            "missing CAD layer 0 generated as white, opaque, Continuous and 0.25 mm",
        ));
    }
    let mut document = CadDocument::new();
    if !drawing.layers.iter().any(|l| l.name == "0") {
        let layer = document.layers.get_mut("0").unwrap();
        layer.color = cadcodec::Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        layer.line_weight = cadcodec::LineWeight::Value(25);
        layer.transparency = cadcodec::Transparency::Explicit(0);
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
    for (owner, entities) in std::iter::once(("*Model_Space", drawing.model.entities.as_slice()))
        .chain(
            supported
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
            let pattern =
                if let ocdraw::ifcx_cad::IfcxCadMode::Explicit(id) = e.appearance.line_pattern {
                    Some(crate::patterns::target(&document, &mappings, id))
                } else {
                    None
                };
            let mut common =
                crate::appearance::to_common(&e.appearance, pattern, layer, &loc, &mut issues);
            common.linetype_scale = e.line_pattern_scale;
            let target = match &e.kind {
                ocdraw::ifcx_cad::IfcxCadEntityKind::BlockInstance {
                    definition_id,
                    transform,
                } if mappings.blocks.cad_handle(*definition_id).is_some() => {
                    let before = issues.len();
                    let insert = crate::blocks::to_insert(
                        &drawing
                            .blocks
                            .iter()
                            .find(|b| b.id == *definition_id)
                            .expect("validated definition")
                            .name,
                        transform,
                        &loc,
                        &mut issues,
                    );
                    (issues.len() == before).then_some(insert)
                }
                ocdraw::ifcx_cad::IfcxCadEntityKind::BlockInstance { .. } => None,
                kind => crate::geometry::to_entity(kind, &loc, &mut issues),
            };
            if target.is_none() {
                issues.push(diagnostic(
                    "entity-skipped",
                    &loc,
                    "whole entity omitted; unsupported geometry or omitted block target",
                ));
            }
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
    crate::loss::to_cad(drawing, &mut issues);
    crate::outcome::enforce_policy(options, &issues)?;
    Ok(IfcxCadToCadOutcome {
        document,
        diagnostics: issues,
        mappings,
    })
}
