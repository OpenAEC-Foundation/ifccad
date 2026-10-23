mod entities;
pub(crate) mod hatch_recipes;
mod layouts;

use crate::diagnostics::diagnostic;
use crate::units::unit_code;
use crate::*;
use ifccad::{encode_ifccad_document, validate_ifccad_document, IfccadDocument, ValidatedIfccad};
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
    let canonical = crate::loss::native_defaults(&canonical, &canonical);
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
    if let Some(display) = drawing.point_display {
        (
            document.header.point_display_mode,
            document.header.point_display_size,
        ) = crate::mapping::point_display::to_cad(display);
    }
    document.header.plotstyle_mode =
        drawing.plot_style_mode == ifccad::IfccadPlotStyleMode::ColorDependent;
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
    let symbols =
        crate::mapping::symbol_source::to_cad_symbol_sources(drawing, &mut document, &mut mappings);
    document.header.linetype_scale = drawing.line_pattern_scale;
    crate::mapping::line_pattern::allocate(
        &mut document,
        &drawing.line_patterns,
        &mut mappings,
        &mut issues,
        &symbols,
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
        target.flags.off = !layer.visible;
        target.flags.frozen = layer.frozen;
        target.flags.locked = layer.locked;
        target.flags.frozen_in_new_viewport = layer.frozen_in_new_viewports;
        target.is_plottable = layer.plottable;
        target.description = layer.description.clone().unwrap_or_default();
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
            let record = opencadcodec::BlockRecord::new(&b.name);
            if record.is_model_space() || record.is_paper_space() {
                issues.push(diagnostic(
                    "block-skipped",
                    format!("block/{}", b.id),
                    "reserved CAD Model/Paper role name omitted together with referring instances",
                ));
                false
            } else {
                true
            }
        })
        .cloned()
        .collect();
    crate::mapping::blocks::allocate(&mut document, &supported, &mut mappings)?;
    let dimension_styles =
        crate::mapping::dimension::to_styles(drawing, &mut document, &mut mappings, &mut issues)?;
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
        &dimension_styles,
        options.preservation,
        &mut document,
        &owners,
        &mut mappings,
        &mut issues,
        &mut geometry,
    )?;
    crate::mapping::viewport_overrides::to_cad(drawing, &mut document, &mappings, &mut issues);
    crate::mapping::workspace::to_cad(drawing, &mut document, &mut mappings, &mut issues)?;
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
    let roots = |entities: &[ifccad::IfccadEntity]| {
        entities
            .iter()
            .filter(|e| {
                e.as_native().is_some_and(|e| {
                    matches!(&e.kind, ifccad::IfccadEntityKind::BlockInstance { .. }) || matches!(&e.kind, ifccad::IfccadEntityKind::Dimension(v) if v.graphics.is_some())
                }) && mappings.entities.cad_handle(e.id()).is_some()
            })
            .map(|e| e.id())
            .collect::<Vec<_>>()
    };
    let all_roots = std::iter::once((
        crate::IfccadGeometryOwner::ModelLayout(drawing.model.id),
        roots(&drawing.model.entities),
    ))
    .chain(supported.iter().map(|block| {
        (
            crate::IfccadGeometryOwner::BlockDefinition(block.id),
            roots(&block.entities),
        )
    }))
    .chain(paper_owners.iter().map(|(id, _)| {
        let paper = drawing.paper_layouts.iter().find(|p| p.id == *id).unwrap();
        (
            crate::IfccadGeometryOwner::PaperLayout(*id),
            roots(&paper.entities),
        )
    }))
    .collect::<Vec<_>>();
    let pending = std::mem::take(&mut geometry.pending_hatch_recipes);
    hatch_recipes::resolve_hatch_recipes(
        pending,
        &mut document,
        &mappings,
        &members,
        &all_roots,
        &mut geometry,
        &mut issues,
    )?;
    for (owner, roots) in &all_roots {
        geometry.select(*owner);
        geometry.assess_roots(&members, roots, &mut issues)?;
    }
    crate::loss::to_cad(drawing, &mut issues);
    crate::diagnostics::enforce_policy(options.loss_policy, &issues)?;
    let graphics_transferred = geometry.graphics_owners.keys().copied().collect::<Vec<_>>();
    let mut assessment = geometry.finish();
    assessment.domains.retain(|domain| match domain.domain() {
        IfccadGeometryDomain::Drawing => true,
        IfccadGeometryDomain::PaperLayout(id) => mappings.layouts.cad_handle(id).is_some(),
    });
    assessment.filled_surfaces = crate::filled_surface_assessment::collect_filled_surfaces(
        drawing,
        &document,
        &mappings,
        crate::filled_surface_assessment::FilledSourceDirection::ToCad,
    )?;
    assessment.polyline_widths = crate::polyline_width_assessment::collect_polyline_widths(
        drawing,
        &document,
        &mappings,
        crate::polyline_width_assessment::PolylineWidthSourceDirection::ToCad,
        &assessment,
    )?;
    Ok(IfccadToCadOutcome {
        dimension: crate::IfccadDimensionAssessment::new(
            drawing,
            mappings.entities.iter().map(|(id, _)| id),
            graphics_transferred,
            std::iter::empty(),
        ),
        text: crate::IfccadTextAssessment::new(drawing, mappings.entities.iter().map(|(id, _)| id)),
        geometry: assessment.with_unassessed(drawing),
        preservation,
        document,
        diagnostics: issues,
        mappings,
    })
}
