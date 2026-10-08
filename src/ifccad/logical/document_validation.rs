use super::*;
use crate::ocdraw::{BlockTransform, CoordinateFrame3, Point3, Scale3, Vector3};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Validate a logical CAD document without encoding it or invoking a CAD runtime.
/// Validation never repairs content, reorders collections or recomputes IDs.
pub fn validate_ifccad_document(document: &IfccadDocument) -> Result<(), IfccadReport> {
    validate_document(document, ValidationPhase::Complete).map_err(|report| {
        crate::ifccad::diagnostics::context(
            report,
            "IFCCAD-OWN-001",
            &format!("/cad/d{}", document.drawing_id),
        )
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ValidationPhase {
    BeforeBounds,
    Complete,
}
pub(super) fn validate_document(
    document: &IfccadDocument,
    phase: ValidationPhase,
) -> Result<(), IfccadReport> {
    validate_document_inner(document, phase).map_err(|report| {
        crate::ifccad::diagnostics::context(
            report,
            "IFCCAD-OWN-001",
            &format!("/cad/d{}", document.drawing_id),
        )
    })
}

fn validate_document_inner(
    document: &IfccadDocument,
    phase: ValidationPhase,
) -> Result<(), IfccadReport> {
    let prefix = format!("/cad/d{}", document.drawing_id);
    let header = &document.header;
    if header.id.is_empty()
        || header.data_version.is_empty()
        || header.author.is_empty()
        || header.timestamp.is_empty()
    {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-WIRE-001",
            "/header",
            "incomplete IFCX header",
        ));
    }
    if !unit(&document.length_unit) {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-GEOMETRY-001",
            &format!("{prefix}/ifccad::drawing/lengthUnit"),
            "invalid drawing length unit",
        ));
    }
    crate::ifccad::logical::patterns::scale(
        document.line_pattern_scale,
        &format!("{prefix}/ifccad::drawing/linePatternScale"),
    )?;
    validate_ifccad_line_patterns(&document.line_patterns)?;
    super::text::validate_styles(document)?;
    let patterns: BTreeSet<_> = document.line_patterns.iter().map(|p| p.id).collect();
    let mut layers = BTreeSet::new();
    for layer in &document.layers {
        let path = format!("{prefix}/layer/{}", layer.id);
        unique(&mut layers, layer.id, &path)?;
        if layer.name.is_empty() {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-APPEARANCE-001",
                &format!("{path}/ifccad::layer/name"),
                "empty layer name",
            ));
        }
        appearance_check(&layer.appearance, &path)?;
        pattern_reference(layer.appearance.line_pattern, &patterns, &path)?;
    }
    crate::plot_kernel::validate_layout_output(
        &document.model.settings,
        crate::plot_kernel::LayoutOutputKind::Model,
    )
    .map_err(|e| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-LAYOUT-003",
            &format!("{prefix}/layout/{}/ifccad::layout", document.model.id),
            format!("invalid layout output: {e:?}"),
        )
    })?;
    let mut layouts = BTreeSet::from([document.model.id]);
    if document.model.tab_index != 0 {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-LAYOUT-001",
            &format!(
                "{prefix}/layout/{}/ifccad::layout/tabIndex",
                document.model.id
            ),
            "Model layout tab index must be zero",
        ));
    }
    let mut tabs = BTreeSet::from([0]);
    let mut layout_names = BTreeSet::from([crate::ocdraw::names::name_key("Model")]);
    for paper in &document.paper_layouts {
        let path = format!("{prefix}/layout/{}", paper.id);
        unique(&mut layouts, paper.id, &path)?;
        if paper.name.trim().is_empty()
            || !layout_names.insert(crate::ocdraw::names::name_key(&paper.name))
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-LAYOUT-001",
                &format!("{path}/ifccad::layout/name"),
                format!("{path} Paper layout needs a unique nonblank name"),
            ));
        }
        if paper.tab_index == 0
            || !tabs.insert(paper.tab_index)
            || paper.tab_index as usize > document.paper_layouts.len()
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-LAYOUT-001",
                &format!("{path}/ifccad::layout/tabIndex"),
                "invalid or duplicate tab index",
            ));
        }
        crate::plot_kernel::validate_layout_output(
            &paper.settings,
            crate::plot_kernel::LayoutOutputKind::Paper,
        )
        .map_err(|e| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-LAYOUT-003",
                &format!("{path}/ifccad::layout"),
                format!("invalid layout output: {e:?}"),
            )
        })?;
    }
    let mut blocks = BTreeSet::new();
    for block in &document.blocks {
        let path = format!("{prefix}/block/{}", block.id);
        unique(&mut blocks, block.id, &path)?;
        if block.name.is_empty() || !unit(&block.insertion_unit) {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-GEOMETRY-004",
                &format!("{path}/ifccad::blockDefinition"),
                "invalid block definition",
            ));
        }
        finite3(block.base_point, &path)?;
    }
    let mut entities = BTreeSet::new();
    for entity in document
        .model
        .entities
        .iter()
        .chain(document.paper_layouts.iter().flat_map(|l| &l.entities))
        .chain(document.blocks.iter().flat_map(|b| &b.entities))
    {
        let path = format!("{prefix}/e{}", entity.id());
        unique(&mut entities, entity.id(), &path)?;
        let entity = match entity {
            IfccadEntity::Native(e) => e,
            IfccadEntity::Opaque(e) => {
                if e.layer_id.is_some_and(|id| !layers.contains(&id)) {
                    return Err(crate::ifccad::diagnostics::failure(
                        "IFCCAD-APPEARANCE-001",
                        &format!("{path}/ifccad::opaqueEntity/nativeLayer"),
                        "unresolved opaque layer",
                    ));
                }
                if let Some(a) = &e.appearance {
                    entity_appearance(&a.appearance, &path)?;
                    if let IfccadMode::Explicit(id) = a.appearance.line_pattern {
                        pattern_reference(id, &patterns, &path)?;
                    }
                    crate::ifccad::logical::patterns::scale(a.line_pattern_scale, &path)?;
                }
                continue;
            }
        };
        if !layers.contains(&entity.layer_id) {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-APPEARANCE-001",
                &format!("{path}/ifccad::entity/layer"),
                format!("{path} unresolved layer {}", entity.layer_id),
            ));
        }
        entity_appearance(&entity.appearance, &path)?;
        if let IfccadMode::Explicit(id) = entity.appearance.line_pattern {
            pattern_reference(id, &patterns, &path)?;
        }
        crate::ifccad::logical::patterns::scale(entity.line_pattern_scale, &path)?;
        match &entity.kind {
            IfccadEntityKind::Text(_) | IfccadEntityKind::MText(_) => {
                super::text::validate_entity(document, &entity.kind, &path)?
            }
            IfccadEntityKind::Viewport(v) => {
                validate_ifccad_viewport_parameters(v).map_err(|report| {
                    crate::ifccad::diagnostics::context(report, "IFCCAD-VIEWPORT-002", &path)
                })?
            }
            IfccadEntityKind::BlockInstance {
                definition_id,
                transform,
            } => {
                if !blocks.contains(definition_id) {
                    return Err(crate::ifccad::diagnostics::failure(
                        "IFCCAD-OWN-003",
                        &format!("{path}/ifccad::blockInstance/definition"),
                        format!("{path} unresolved block definition {definition_id}"),
                    ));
                }
                let frame = placement(&transform.placement, &path)?;
                BlockTransform::try_new(
                    frame,
                    transform.rotation,
                    Scale3::new(transform.scale[0], transform.scale[1], transform.scale[2]),
                )
                .map_err(|e| {
                    crate::ifccad::diagnostics::failure(
                        "IFCCAD-GEOMETRY-004",
                        &format!("{path}/ifccad::blockInstance/transform"),
                        format!("invalid block transform: {e}"),
                    )
                })?;
            }
            _ => {
                let geometry = entity
                    .kind
                    .as_shared_geometry()
                    .map_err(|e| {
                        crate::ifccad::diagnostics::context(e, "IFCCAD-GEOMETRY-001", &path)
                    })?
                    .expect("primitive geometry");
                crate::geometry_kernel::validate_geometry(geometry).map_err(|e| {
                    let message = match (e, &entity.kind) {
                        (
                            crate::geometry_kernel::GeometryValidationError::InvalidGeometry,
                            IfccadEntityKind::Circle { .. },
                        ) => "invalid circle radius".to_owned(),
                        _ => e.to_string(),
                    };
                    crate::ifccad::diagnostics::failure("IFCCAD-GEOMETRY-002", &path, message)
                })?;
            }
        }
    }
    super::preservation_validation::validate_preservation(document)?;
    super::viewports::validate_references(document)?;
    if cycle(&document.blocks) {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-OWN-003",
            "/",
            "block definition cycle",
        ));
    }
    super::workspace::validate_workspace(document)?;
    super::allocation::validate(document)?;
    if phase == ValidationPhase::Complete {
        super::bounds::validate_supplied(document)?;
    }
    Ok(())
}

fn unique(ids: &mut BTreeSet<u64>, id: u64, path: &str) -> Result<(), IfccadReport> {
    if !ids.insert(id) {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-ID-001",
            path,
            "duplicate ID",
        ));
    }
    Ok(())
}
fn pattern_reference(
    id: IfccadLinePatternId,
    patterns: &BTreeSet<IfccadLinePatternId>,
    path: &str,
) -> Result<(), IfccadReport> {
    if !patterns.contains(&id) {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-APPEARANCE-001",
            path,
            format!("{path} unresolved drawing-local line pattern {}", id.0),
        ));
    }
    Ok(())
}
fn finite3(value: [f64; 3], context: &str) -> Result<(), IfccadReport> {
    if value.iter().all(|n| n.is_finite()) {
        Ok(())
    } else {
        Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-GEOMETRY-001",
            context,
            "non-finite coordinate",
        ))
    }
}
fn placement(value: &IfccadPlacement, context: &str) -> Result<CoordinateFrame3, IfccadReport> {
    finite3(
        value.origin,
        &format!("{context}/ifccad::geom::placement/origin"),
    )?;
    finite3(
        value.x_axis,
        &format!("{context}/ifccad::geom::placement/xAxis"),
    )?;
    finite3(
        value.y_axis,
        &format!("{context}/ifccad::geom::placement/yAxis"),
    )?;
    let p = |v: [f64; 3]| Point3::new(v[0], v[1], v[2]);
    let v = |v: [f64; 3]| Vector3::new(v[0], v[1], v[2]);
    CoordinateFrame3::try_new(p(value.origin), v(value.x_axis), v(value.y_axis)).map_err(|e| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-GEOMETRY-001",
            &format!("{context}/ifccad::geom::placement"),
            e.to_string(),
        )
    })
}
fn unit(token: &str) -> bool {
    static UNITS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    UNITS
        .get_or_init(|| {
            let registry: Value =
                serde_json::from_str(include_str!("../../../schemas/ocdraw/registry-0.1.0.json"))
                    .expect("bundled OCDraw registry");
            registry["types"]["unit"]["values"]
                .as_array()
                .expect("OCDraw unit registry")
                .iter()
                .map(|value| value.as_str().expect("unit token").to_owned())
                .collect()
        })
        .iter()
        .any(|unit| unit == token)
}
pub(super) fn color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
fn appearance_check(layer: &IfccadLayerAppearance, context: &str) -> Result<(), IfccadReport> {
    for (field, valid) in [
        ("color", color(&layer.color)),
        (
            "opacity",
            layer.opacity.is_finite() && (0.0..=1.0).contains(&layer.opacity),
        ),
        (
            "lineWeight",
            layer.line_weight.is_finite() && layer.line_weight >= 0.,
        ),
    ] {
        if !valid {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-APPEARANCE-001",
                &format!("{context}/ifccad::layer/appearance/{field}"),
                "invalid layer appearance",
            ));
        }
    }
    Ok(())
}
fn entity_appearance(a: &IfccadEntityAppearance, context: &str) -> Result<(), IfccadReport> {
    for (field, invalid) in [
        (
            "color",
            matches!(&a.color, IfccadMode::Explicit(v) if !color(v)),
        ),
        (
            "opacity",
            matches!(&a.opacity, IfccadMode::Explicit(v) if !v.is_finite() || !(0.0..=1.0).contains(v)),
        ),
        (
            "lineWeight",
            matches!(&a.line_weight, IfccadMode::Explicit(v) if !v.is_finite() || *v < 0.0),
        ),
    ] {
        if invalid {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-APPEARANCE-001",
                &format!("{context}/ifccad::entity/appearance/{field}/value"),
                "invalid entity appearance",
            ));
        }
    }
    Ok(())
}
fn cycle(blocks: &[IfccadBlockDefinition]) -> bool {
    fn visit(
        id: u64,
        graph: &BTreeMap<u64, Vec<u64>>,
        active: &mut BTreeSet<u64>,
        done: &mut BTreeSet<u64>,
    ) -> bool {
        if done.contains(&id) {
            return false;
        }
        if !active.insert(id) {
            return true;
        }
        if graph.get(&id).is_some_and(|targets| {
            targets
                .iter()
                .any(|target| visit(*target, graph, active, done))
        }) {
            return true;
        }
        active.remove(&id);
        done.insert(id);
        false
    }
    let graph: BTreeMap<_, _> = blocks
        .iter()
        .map(|b| {
            (
                b.id,
                b.entities
                    .iter()
                    .filter_map(IfccadEntity::as_native)
                    .filter_map(|e| match e.kind {
                        IfccadEntityKind::BlockInstance { definition_id, .. } => {
                            Some(definition_id)
                        }
                        _ => None,
                    })
                    .collect(),
            )
        })
        .collect();
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    graph
        .keys()
        .any(|id| visit(*id, &graph, &mut active, &mut done))
}
