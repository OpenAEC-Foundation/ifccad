mod entities;
mod layouts;

use crate::diagnostics::diagnostic;
use crate::units::unit_code;
use crate::*;
use ocdraw::ifccad::{
    encode_ifccad_document, validate_ifccad_document, IfccadDocument, ValidatedIfccad,
};
use opencadcodec::CadDocument;

/// Convert supported content with explicit semantic loss acceptance.
pub fn ifccad_source_to_cad_document(
    source: &ValidatedIfccad,
    options: IfccadToCadOptions,
) -> Result<IfccadToCadOutcome, IfccadConversionError> {
    let drawing = source.document();
    let canonical = encode_ifccad_document(drawing).map_err(IfccadConversionError::CoreEncoding)?;
    let canonical: serde_json::Value =
        serde_json::from_slice(canonical.bytes()).expect("writer JSON");
    let mut issues = Vec::new();
    let raw = crate::loss::native_defaults(source.graph().composed_ifcx(), &canonical);
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
pub fn ifccad_document_to_cad_document(
    source: &IfccadDocument,
    options: IfccadToCadOptions,
) -> Result<IfccadToCadOutcome, IfccadConversionError> {
    validate_ifccad_document(source).map_err(IfccadConversionError::CoreValidation)?;
    convert_document(source, options, Vec::new())
}

fn convert_document(
    drawing: &IfccadDocument,
    options: IfccadToCadOptions,
    mut issues: Vec<IfccadDiagnostic>,
) -> Result<IfccadToCadOutcome, IfccadConversionError> {
    // Native layers are identified by ID; CAD tables look them up by normalized
    // name. Reject ambiguity before the special layer-0 replacement or allocation.
    let mut layer_names = std::collections::BTreeMap::new();
    for layer in &drawing.layers {
        let key = opencadcodec::tables::normalize_name(&layer.name);
        if let Some(previous) = layer_names.insert(key, layer.id) {
            return Err(IfccadConversionError::InvalidStructure(format!(
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
    let mut geometry = crate::geometry_context::GeometryContext::new(
        options.geometry_tolerance,
        &drawing.length_unit,
        drawing.model.id,
    )?;
    for paper in &drawing.paper_layouts {
        geometry.add_paper(
            paper.id,
            cad_geometry_convert::plot_units::paper_mapping(paper.settings.plot_settings.as_ref()),
        )?;
    }
    let mut document = CadDocument::new();
    // Allocate style targets before any scope contents.
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
    document.header.plotstyle_mode =
        drawing.plot_style_mode == ocdraw::ifccad::IfccadPlotStyleMode::ColorDependent;
    document.header.paper_space_linetype_scaling =
        drawing.model.settings.paper_space_linetype_scaling;
    let model_layout = document
        .objects
        .values_mut()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Model" => Some(l),
            _ => None,
        })
        .expect("model layout");
    crate::mapping::layout::apply_settings(model_layout, &drawing.model.settings, &mut issues)?;
    let mut mappings = IfccadMappings::default();
    crate::mapping::text::to_styles(drawing, &mut document, &mut mappings, &mut issues)?;
    document.header.linetype_scale = drawing.line_pattern_scale;
    crate::mapping::line_pattern::allocate(
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
        let mut target = crate::mapping::appearance::to_layer(
            &layer.name,
            &layer.appearance,
            &crate::mapping::line_pattern::target(
                &document,
                &mappings,
                layer.appearance.line_pattern,
            )
            .0,
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
                .map_err(IfccadConversionError::CadConstruction)?;
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
    crate::mapping::blocks::allocate(&mut document, &supported, &mut mappings)?;
    let paper_owners = crate::to_cad::layouts::allocate(
        &mut document,
        &drawing.paper_layouts,
        &mut mappings,
        &mut issues,
    )?;
    for paper in &drawing.paper_layouts {
        let handle = mappings
            .layouts
            .cad_handle(paper.id)
            .expect("allocated layout");
        let opencadcodec::objects::ObjectType::Layout(l) = &document.objects[&handle] else {
            unreachable!()
        };
        let mapping = if l.paper_width > 0. && l.paper_height > 0. {
            cad_geometry_convert::plot_units::paper_mapping(paper.settings.plot_settings.as_ref())
        } else {
            cad_geometry_convert::plot_units::PaperMapping::Unknown
        };
        geometry.refine_paper(paper.id, mapping)?;
    }
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
    let preservation = entities::to_cad(
        drawing,
        options.preservation,
        &mut document,
        &owners,
        &mut mappings,
        &mut issues,
        &mut geometry,
    )?;
    let members = supported
        .iter()
        .map(|b| {
            (
                b.id,
                b.entities
                    .iter()
                    .filter(|e| mappings.entities.cad_handle(e.id()).is_some())
                    .map(|e| e.id())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let roots = |entities: &[ocdraw::ifccad::IfccadEntity]| {
        entities
            .iter()
            .filter(|e| {
                e.as_native().is_some_and(|e| {
                    matches!(
                        e.kind,
                        ocdraw::ifccad::IfccadEntityKind::BlockInstance { .. }
                    )
                }) && mappings.entities.cad_handle(e.id()).is_some()
            })
            .map(|e| e.id())
            .collect::<Vec<_>>()
    };
    geometry.select(crate::IfccadGeometryOwner::ModelLayout(drawing.model.id));
    geometry.assess_roots(&members, &roots(&drawing.model.entities), &mut issues)?;
    for block in &supported {
        geometry.select(crate::IfccadGeometryOwner::BlockDefinition(block.id));
        geometry.assess_roots(&members, &roots(&block.entities), &mut issues)?;
    }
    for (id, _) in &paper_owners {
        let paper = drawing.paper_layouts.iter().find(|p| p.id == *id).unwrap();
        geometry.select(crate::IfccadGeometryOwner::PaperLayout(*id));
        geometry.assess_roots(&members, &roots(&paper.entities), &mut issues)?;
    }
    crate::loss::to_cad(drawing, &mut issues);
    crate::diagnostics::enforce_policy(options.loss_policy, &issues)?;
    let mut assessment = geometry.finish();
    assessment.domains.retain(|domain| match domain.domain() {
        IfccadGeometryDomain::Drawing => true,
        IfccadGeometryDomain::PaperLayout(id) => mappings.layouts.cad_handle(id).is_some(),
    });
    Ok(IfccadToCadOutcome {
        text: crate::IfccadTextAssessment::new(drawing, mappings.entities.iter().map(|(id, _)| id)),
        geometry: assessment.with_unassessed(drawing),
        preservation,
        document,
        diagnostics: issues,
        mappings,
    })
}
