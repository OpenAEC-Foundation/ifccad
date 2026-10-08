use crate::diagnostics::diagnostic;
use crate::{IfccadConversionError, IfccadDiagnostic, IfccadMappings};
use cad_presentation_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::objects::{KnownXRecordKind as K, XRecordValue};
use opencadcodec::{CadDocument, Handle};
use std::collections::BTreeSet;

pub(crate) fn from_cad(
    source: &CadDocument,
    drawing: &mut IfccadDocument,
    mappings: &IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<BTreeSet<Handle>, IfccadConversionError> {
    let mut consumed = Vec::new();
    for section in cad_override_sections(source) {
        let location = format!(
            "object/{}.viewport/{}.layer/{}",
            section.record, section.viewport, section.layer
        );
        let viewport = mappings
            .entities
            .ifccad_id(section.viewport)
            .and_then(|id| {
                drawing
                    .paper_layouts
                    .iter_mut()
                    .flat_map(|p| p.entities.iter_mut())
                    .find_map(|e| {
                        let e = e.as_native_mut()?;
                        if e.id != id {
                            return None;
                        }
                        match &mut e.kind {
                            IfccadEntityKind::Viewport(v) => Some(v),
                            _ => None,
                        }
                    })
            });
        let Some(viewport) = viewport else {
            issues.push(diagnostic(
                "viewport-overrides",
                location,
                "override references an unavailable authored viewport",
            ));
            continue;
        };
        let Some(layer_id) = mappings.layers.ifccad_id(section.layer) else {
            issues.push(diagnostic(
                "viewport-overrides",
                location,
                "override references an unavailable layer",
            ));
            continue;
        };
        let mut row = viewport
            .layer_overrides
            .iter()
            .find(|r| r.layer_id == layer_id)
            .cloned()
            .unwrap_or(IfccadViewportLayerOverride {
                layer_id,
                frozen: false,
                color: None,
                opacity: None,
                line_pattern_id: None,
                line_weight: None,
            });
        let mut conflict = false;
        let represented = match section.kind {
            K::LayerViewportColorOverride => decode_override_color(&section.value).map(|v| {
                let v = super::appearance::native_color(v);
                conflict = row.color.as_ref().is_some_and(|old| *old != v);
                row.color = Some(v);
            }),
            K::LayerViewportAlphaOverride => decode_override_opacity(&section.value).map(|v| {
                conflict = row.opacity.is_some_and(|old| old != v);
                row.opacity = Some(v);
            }),
            K::LayerViewportLineweightOverride => {
                decode_override_lineweight(&section.value).map(|v| {
                    conflict = row.line_weight.is_some_and(|old| old != v);
                    row.line_weight = Some(v);
                })
            }
            K::LayerViewportLinetypeOverride => match section.value {
                XRecordValue::Handle(h) => mappings
                    .line_patterns
                    .ifccad_id(h)
                    .map(|v| {
                        let v = IfccadLinePatternId(v);
                        conflict = row.line_pattern_id.is_some_and(|old| old != v);
                        row.line_pattern_id = Some(v);
                    })
                    .ok_or(PresentationValueError::UnsupportedOverride),
                _ => Err(PresentationValueError::UnsupportedOverride),
            },
            _ => unreachable!(),
        };
        if conflict {
            return Err(IfccadConversionError::InvalidStructure(format!(
                "conflicting viewport override at {location}"
            )));
        }
        if represented.is_err() {
            issues.push(diagnostic(
                "viewport-overrides",
                location,
                format!(
                    "unqualified {:?} value or missing pattern; override omitted",
                    section.kind
                ),
            ));
            continue;
        }
        if let Some(existing) = viewport
            .layer_overrides
            .iter_mut()
            .find(|r| r.layer_id == layer_id)
        {
            *existing = row;
        } else {
            viewport.layer_overrides.push(row);
        }
        viewport.layer_overrides.sort_by_key(|r| r.layer_id);
        consumed.push(section);
    }
    Ok(consumed_override_objects(source, &consumed))
}

pub(crate) fn to_cad(
    drawing: &IfccadDocument,
    document: &mut CadDocument,
    mappings: &IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    for entity in drawing
        .paper_layouts
        .iter()
        .flat_map(|p| &p.entities)
        .filter_map(IfccadEntity::as_native)
    {
        let IfccadEntityKind::Viewport(view) = &entity.kind else {
            continue;
        };
        let Some(viewport) = mappings.entities.cad_handle(entity.id) else {
            continue;
        };
        for row in &view.layer_overrides {
            let location = format!("entity/{}.layerOverrides/{}", entity.id, row.layer_id);
            let Some(layer) = mappings.layers.cad_handle(row.layer_id) else {
                issues.push(diagnostic(
                    "viewport-overrides",
                    location,
                    "unavailable target layer",
                ));
                continue;
            };
            let mut values = Vec::new();
            if let Some(color) = &row.color {
                let (value, losses) =
                    encode_override_color(&super::appearance::scalar_color(color))
                        .expect("validated color");
                for loss in losses {
                    issues.push(crate::diagnostics::modification(
                        "viewport-overrides",
                        format!("{location}.color"),
                        format!("indexed identity omitted: {loss:?}"),
                    ));
                }
                if color.named.is_some() {
                    issues.push(diagnostic(
                        "viewport-overrides",
                        format!("{location}.color.named"),
                        "named viewport color metadata is not qualified; concrete color retained",
                    ));
                }
                values.push((K::LayerViewportColorOverride, value));
            }
            if let Some(value) = row.opacity {
                let value = super::appearance::cad_opacity(value, &location, issues);
                values.push((
                    K::LayerViewportAlphaOverride,
                    encode_override_opacity(value).expect("explicit opacity"),
                ));
            }
            if let Some(value) = row.line_weight {
                let value = super::appearance::cad_weight(value, &location, issues);
                values.push((
                    K::LayerViewportLineweightOverride,
                    encode_override_lineweight(value).expect("explicit weight"),
                ));
            }
            if let Some(id) = row.line_pattern_id {
                if let Some(handle) = mappings.line_patterns.cad_handle(id.0) {
                    values.push((
                        K::LayerViewportLinetypeOverride,
                        XRecordValue::Handle(handle),
                    ));
                } else {
                    issues.push(diagnostic(
                        "viewport-overrides",
                        format!("{location}.linePattern"),
                        "unavailable target line pattern",
                    ));
                }
            }
            for (kind, value) in values {
                assert!(document.set_layer_viewport_override(layer, kind, viewport, value));
            }
        }
    }
}
